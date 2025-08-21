use serde::{Deserialize, Serialize};

use crate::{core::GameId, database::{AdminInfo, UserInfo}};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub version: String,
    pub uptime: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub user: Option<UserInfo>,
    pub access_token: Option<String>,
    pub expires_in: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct AdminAuthResponse {
    pub success: bool,
    pub admin: Option<AdminInfo>,
    pub access_token: Option<String>,
    pub expires_in: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AvailableGamesResponse {
    pub success: bool,
    pub games: Vec<String>,
}

#[derive(Deserialize)]
pub struct JoinGameRequest {
    pub game_choice: String,
}

#[derive(Serialize)]
pub struct JoinGameResponse {
    pub success: bool,
    pub game_token: Option<String>,
    pub game_choice: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub game_type: String,
    pub status: String,
    pub joined_at: u64,
    pub game_id: GameId,
}

#[derive(Debug, Serialize)]
pub struct UserSessionStatusResponse {
    pub success: bool,
    pub has_active_session: bool,
    pub session: Option<SessionInfo>,
}

#[derive(Debug, Serialize)]
pub struct LeaveGameSessionResponse {
    pub success: bool,
    pub message: String,
}