use bcrypt::BcryptError;

use crate::prelude::*;

pub fn hash_password(password: &str) -> Result<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e: BcryptError| Error::Other(format!("Failed to hash password: {}", e)))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    bcrypt::verify(password, hash)
        .map_err(|e: BcryptError| Error::Other(format!("Failed to verify password: {}", e)))
}
