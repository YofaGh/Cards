use std::{fs, path::Path};

use crate::prelude::*;

pub trait GetOrError<K, V> {
    fn get_or_error(&self, key: &K, error_fn: impl FnOnce() -> Error) -> Result<&V>;
    fn get_mut_or_error(&mut self, key: &K, error_fn: impl FnOnce() -> Error) -> Result<&mut V>;
}

impl<K: Ord, V> GetOrError<K, V> for BTreeMap<K, V> {
    fn get_or_error(&self, key: &K, error_fn: impl FnOnce() -> Error) -> Result<&V> {
        self.get(key).ok_or_else(error_fn)
    }
    fn get_mut_or_error(&mut self, key: &K, error_fn: impl FnOnce() -> Error) -> Result<&mut V> {
        self.get_mut(key).ok_or_else(error_fn)
    }
}

pub fn read_file(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    fs::read(path).map_err(Error::read_file)
}
