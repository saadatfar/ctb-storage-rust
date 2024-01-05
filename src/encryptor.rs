use crypto_common::{
    Key as TKey
};

use aead::{
    Nonce as TNonce,
    Aead,
    KeyInit,
    Tag as TTag
};

use num_bigint::{BigUint};
use num_traits::{One};
use generic_array::{ArrayLength, GenericArray};

use std::fs::File;
use std::io::{self, Read};

pub type Result<T> = core::result::Result<T, aead::Error>;
type Crypto = chacha20poly1305::ChaCha20Poly1305;
pub type Key = TKey<Crypto>;
pub type Nonce = TNonce<Crypto>;
pub type Tag = TTag<Crypto>;

const CHUNK_SIZE: usize = 1024 * 1024;

pub struct EncryptedChunker<T> where T: Read {
    source: T,
    key: Key,
    nonce: Nonce
}

impl<T> EncryptedChunker<T> where T: Read {
    pub fn new(source: T, key: Key, nonce: Nonce) -> Self {
        return Self {
            source,
            key,
            nonce,
        };
    }
}

impl<T> Iterator for EncryptedChunker<T> where T: Read {
    type Item = Result<Vec<u8>>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = [0; CHUNK_SIZE];
        let res = self.source.read(&mut buffer);
        let ret = match res {
            Ok(count) => {
                if count > 0 {
                    Some(encrypt(buffer.to_vec(), &self.key, &self.nonce))
                } else {
                    None
                }
            }
            Err(e) => None,
        };
        increase_bytes_le(&mut self.nonce);
        return ret;
    }
}

fn encrypt_file(filename: &str, key: &Key, nonce: &Nonce) -> io::Result<Vec<u8>> {
    let mut file = File::open(filename)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    let encrypted = encrypt(contents, &key, &nonce).unwrap();
    Ok(encrypted)
}


pub fn increase_bytes_le<T>(nonce: &mut GenericArray<u8,T>) where T:ArrayLength<u8> {
    let mut number = BigUint::from_bytes_le(nonce);
    number += BigUint::one();
    let new_bytes = number.to_bytes_le();
    println!("{:x?}", &new_bytes);
    let min_len = std::cmp::min(nonce.len(), new_bytes.len());
    nonce[..min_len].copy_from_slice(&new_bytes[..min_len]);
}

pub fn encrypt(plain: Vec<u8>, key: &Key, nonce: &Nonce) -> Result<Vec<u8>> {
    let cipher = Crypto::new(&key);
    let cipher_result = cipher.encrypt(&nonce, plain.as_ref());

    return cipher_result;
}

pub fn decrypt(encrypted: Vec<u8>, key: &Key, nonce: &Nonce) -> Result<Vec<u8>> {
    let cipher = Crypto::new(&key);
    let plain_result = cipher.decrypt(&nonce, encrypted.as_ref());

    return plain_result;
}
