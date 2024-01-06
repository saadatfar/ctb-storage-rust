use crypto_common::{
    Key as TKey
};

use aead::{
    Nonce as TNonce,
    Aead,
    KeyInit
};

use num_bigint::{BigUint};
use num_traits::{One};
use generic_array::{ArrayLength, GenericArray};
use std::io::Read;
use anyhow::{anyhow, bail};


pub type Result<T> = anyhow::Result<T>;
type Crypto = chacha20poly1305::ChaCha20Poly1305;
pub type Key = TKey<Crypto>;
pub type Nonce = TNonce<Crypto>;

pub struct EncryptedIterator<T> where T: Read {
    source: T,
    key: Key,
    nonce_init: Nonce,
    pub chunk_size: usize
}

pub trait AsEncryptedIterator<T> where T: Read {
    fn to_encrypted_iterator(self, key: Key, nonce: Nonce, chunk_size: usize) -> EncryptedIterator<T>;
}

impl<T: Read> AsEncryptedIterator<T> for T {
    fn to_encrypted_iterator(self, key: Key, nonce: Nonce, chunk_size: usize) -> EncryptedIterator<T> {
        return EncryptedIterator {
            source: self,
            key,
            nonce_init: nonce,
            chunk_size
        };
    }
}


impl<T: Read> EncryptedIterator<T> {
    pub fn read_bytes_encrypted(&mut self, size: usize) -> Option<Result<Vec<u8>>> {
        let mut buffer = vec![0u8;size];
        let res = self.source.read(&mut buffer);
        let ret = match res {
            Ok(count) => {
                if count > 0 {
                    Some(encrypt(&buffer[..count].to_vec(), &self.key, &self.nonce_init))
                } else {
                    None
                }
            }
            Err(e) => Some(Err(anyhow!(e))),
        };
        increase_bytes_le(&mut self.nonce_init);
        return ret;
    }
}

impl<T> Iterator for EncryptedIterator<T> where T: Read {
    type Item = Result<Vec<u8>>;
    fn next(&mut self) -> Option<Self::Item> {
        return self.read_bytes_encrypted(self.chunk_size);
    }
}

pub fn increase_bytes_le<T>(nonce: &mut GenericArray<u8, T>) where T: ArrayLength<u8> {
    let mut number = BigUint::from_bytes_le(nonce);
    number += BigUint::one();
    let new_bytes = number.to_bytes_le();
    let min_len = std::cmp::min(nonce.len(), new_bytes.len());
    nonce[..min_len].copy_from_slice(&new_bytes[..min_len]);
}

pub fn encrypt(plain: &Vec<u8>, key: &Key, nonce: &Nonce) -> Result<Vec<u8>> {
    let cipher = Crypto::new(&key);
    let cipher_result = cipher.encrypt(&nonce, plain.as_ref())
        .or_else(|_x| bail!("Encryption error"))?;
    return Ok(cipher_result);
}

pub fn decrypt(encrypted: &Vec<u8>, key: &Key, nonce: &Nonce) -> Result<Vec<u8>> {
    let cipher = Crypto::new(&key);
    let plain_result = cipher.decrypt(&nonce, encrypted.as_ref())
        .or_else(|_x| bail!("Decryption error"))?;
    return Ok(plain_result);
}
