use axum::{extract::State, http::StatusCode, response::Json};
use std::time::SystemTime;

use crate::{
    api::models::{AvailableGamesResponse, ErrorResponse, JoinGameRequest, JoinGameResponse},
    auth::{generate_game_session_token, validate_token, Claims},
    database::UserRepository,
    prelude::*,
};

use super::models::HealthResponse;

pub fn get_token(headers: &axum::http::HeaderMap) -> Option<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header: &axum::http::HeaderValue| header.to_str().ok())
        .and_then(|header: &str| header.strip_prefix("Bearer "))
}

pub async fn health() -> Json<HealthResponse> {
    let start_time: SystemTime = SystemTime::UNIX_EPOCH;
    let uptime: u64 = SystemTime::now()
        .duration_since(start_time)
        .unwrap_or_default()
        .as_secs();
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: format!("{uptime}s"),
    })
}

pub async fn get_available_games(
    headers: axum::http::HeaderMap,
    State(user_repo): State<UserRepository>,
) -> Result<Json<AvailableGamesResponse>, (StatusCode, Json<ErrorResponse>)> {
    let token: &str = match get_token(&headers) {
        Some(token) => token,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    success: false,
                    message: "Missing or invalid authorization header".to_string(),
                }),
            ));
        }
    };
    let claims: Claims = match validate_token(token) {
        Ok(claims) => claims,
        Err(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    success: false,
                    message: "Invalid token".to_string(),
                }),
            ));
        }
    };
    match user_repo.get_user_by_id(claims.sub).await {
        Ok(Some(_)) => {
            let available_games: Vec<String> =
                crate::core::get_game_registry().get_available_games();
            Ok(Json(AvailableGamesResponse {
                success: true,
                games: available_games,
            }))
        }
        _ => Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                success: false,
                message: "User not found".to_string(),
            }),
        )),
    }
}

pub async fn join_game_queue(
    headers: axum::http::HeaderMap,
    State(user_repo): State<UserRepository>,
    Json(payload): Json<JoinGameRequest>,
) -> Result<Json<JoinGameResponse>, (StatusCode, Json<ErrorResponse>)> {
    let token: &str = match get_token(&headers) {
        Some(token) => token,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    success: false,
                    message: "Missing or invalid authorization header".to_string(),
                }),
            ));
        }
    };
    let claims: crate::auth::Claims = match validate_token(token) {
        Ok(claims) => claims,
        Err(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    success: false,
                    message: "Invalid token".to_string(),
                }),
            ));
        }
    };
    let user: crate::database::User = match user_repo.get_user_by_id(claims.sub).await {
        Ok(Some(user)) => user,
        _ => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    success: false,
                    message: "User not found".to_string(),
                }),
            ));
        }
    };
    let available_games: Vec<String> = crate::core::get_game_registry().get_available_games();
    if !available_games.contains(&payload.game_choice) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                message: format!(
                    "Invalid game choice '{}'. Available games: {}",
                    payload.game_choice,
                    available_games.join(", ")
                ),
            }),
        ));
    }
    let game_token: crate::auth::TokenPair =
        match generate_game_session_token(user.id, user.username, payload.game_choice.clone()) {
            Ok(token) => token,
            Err(_) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        success: false,
                        message: "Failed to generate game session token".to_string(),
                    }),
                ));
            }
        };
    Ok(Json(JoinGameResponse {
        success: true,
        game_token: Some(game_token.access_token),
        game_choice: payload.game_choice,
        message: "Ready to connect to game server".to_string(),
    }))
}
