use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct TokenPair {
    pub access_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub exp: usize,
    pub iat: usize,
}
