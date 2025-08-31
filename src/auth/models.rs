use serde::{Deserialize, Serialize};

use crate::prelude::{GameId, UserId};

#[derive(Debug, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: UserId,
    pub username: String,
    pub is_admin: bool,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameSessionClaims {
    pub sub: UserId,
    pub username: String,
    pub game_choice: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReconnectClaims {
    pub sub: UserId,
    pub game_id: GameId,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SessionTokenType {
    GameSession(GameSessionClaims),
    Reconnection(ReconnectClaims),
}
