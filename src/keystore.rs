mod serialize;
mod recovery;
mod generate_key;
mod persistence;

pub use recovery::DataKeyRecoveryGenerator;
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::marker::PhantomData;
use aead::Aead;
use aead::rand_core::{CryptoRng, RngCore};
use crypto_common::{KeyInit, KeySizeUser};
use generic_array::{ArrayLength, GenericArray};

type Key<N> = GenericArray<u8, N>;

pub struct KeyStore<N: ArrayLength<u8>, C: KeySizeUser<KeySize=N>> {
    root_key: Key<N>,
    recovery_key: Option<Key<N>>,
    data_key_map: HashMap<String, Key<N>>,
    alg: PhantomData<C>,
}

impl<N: ArrayLength<u8>, C: KeySizeUser<KeySize=N> + KeyInit + Aead> Default for KeyStore<N, C> {
    fn default() -> Self {
        return KeyStore {
            data_key_map: Default::default(),
            root_key: Default::default(),
            recovery_key: Default::default(),
            alg: PhantomData,
        };
    }
}

impl<N: ArrayLength<u8>, C: KeySizeUser<KeySize=N> + KeyInit + Aead> KeyStore<N, C> {
    pub fn new(root_key: Key<N>) -> Self <> {
        return KeyStore {
            root_key,
            ..Default::default()
        };
    }

    pub fn set_recover_key(&mut self, recovery_key: Key<N>) -> Result<()> {
        if self.recovery_key.is_some() {
            bail!("Cannot update recovery key");
        }
        self.recovery_key = Some(recovery_key);
        self.persist_recovery_key()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn insert(&mut self, key_id: &str, key: Key<N>) -> Result<()> {
        self.persist_key(key_id, &key)?;
        self.data_key_map.insert(key_id.to_string(), key);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get(&self, key_id: &str) -> Option<&Key<N>> {
        let key = self.data_key_map.get(key_id);
        key
    }
}
