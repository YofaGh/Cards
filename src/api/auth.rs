use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};

use crate::{
    database::{User, UserInfo},
    prelude::*,
};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn login(
    State(user_repo): State<crate::database::UserRepository>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let auth_result: Option<User> =
        match crate::auth::login_user(&user_repo, payload.username, payload.password).await {
            Ok(result) => result,
            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        success: false,
                        message: e.to_string(),
                    }),
                ))
            }
        };
    let user: User = match auth_result {
        Some(user) => user,
        None => {
            return Ok(Json(AuthResponse {
                success: false,
                access_token: None,
                expires_in: None,
                user: None,
            }));
        }
    };
    let tokens: crate::auth::TokenPair =
        match crate::auth::generate_token(user.id.to_string(), user.username.clone()) {
            Ok(tokens) => tokens,
            Err(_) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        success: false,
                        message: "Failed to generate authentication tokens".to_string(),
                    }),
                ))
            }
        };
    Ok(Json(AuthResponse {
        success: true,
        access_token: Some(tokens.access_token),
        expires_in: Some(tokens.expires_in),
        user: Some(user.into()),
    }))
}
