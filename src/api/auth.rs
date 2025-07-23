use axum::{extract::State, http::StatusCode, response::Json};

use super::models::{
    AdminAuthResponse, AuthResponse, ErrorResponse, LoginRequest, RegisterRequest,
};
use crate::{
    auth::{generate_token, TokenPair},
    database::{Admin, User, UserRepository},
    prelude::*,
};

pub async fn login(
    State(user_repo): State<UserRepository>,
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
    let tokens: TokenPair = match generate_token(user.id.to_string(), user.username.clone(), false)
    {
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

pub async fn admin_login(
    State(admin_repo): State<crate::database::AdminRepository>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AdminAuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let auth_result: Option<Admin> =
        match crate::auth::login_admin(&admin_repo, payload.username, payload.password).await {
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
    let admin: Admin = match auth_result {
        Some(admin) => admin,
        None => {
            return Ok(Json(AdminAuthResponse {
                success: false,
                access_token: None,
                expires_in: None,
                admin: None,
            }));
        }
    };
    if !admin.is_active {
        return Ok(Json(AdminAuthResponse {
            success: false,
            access_token: None,
            expires_in: None,
            admin: None,
        }));
    }
    let tokens: TokenPair = match generate_token(admin.id.to_string(), admin.username.clone(), true)
    {
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
    Ok(Json(AdminAuthResponse {
        success: true,
        access_token: Some(tokens.access_token),
        expires_in: Some(tokens.expires_in),
        admin: Some(admin.into()),
    }))
}

pub async fn register(
    State(user_repo): State<UserRepository>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let auth_result: Option<User> = match crate::auth::register_user(
        &user_repo,
        payload.email,
        payload.username,
        payload.password,
    )
    .await
    {
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
    let tokens: TokenPair = match generate_token(user.id.to_string(), user.username.clone(), false)
    {
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