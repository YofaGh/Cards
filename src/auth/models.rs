use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub is_admin: bool,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameSessionClaims {
    pub sub: String,
    pub username: String,
    pub game_choice: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReconnectClaims {
    pub sub: String,
    pub game_id: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SessionTokenType {
    GameSession(GameSessionClaims),
    Reconnection(ReconnectClaims),
}