use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};

use crate::{
    database::{User, UserRepository},
    prelude::*,
};

pub async fn get_user(
    State(user_repo): State<UserRepository>,
    Path(user_id): Path<UserId>,
    Extension(_admin_user): Extension<User>,
) -> Result<Json<User>, StatusCode> {
    match user_repo.get_user_by_id(user_id).await {
        Ok(Some(user)) => Ok(Json(user)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn lock_user(
    State(user_repo): State<UserRepository>,
    Path(user_id): Path<UserId>,
    Extension(_admin_user): Extension<User>,
) -> Result<StatusCode, StatusCode> {
    match user_repo.lock_user(user_id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn unlock_user(
    State(user_repo): State<UserRepository>,
    Path(user_id): Path<UserId>,
    Extension(_admin_user): Extension<User>,
) -> Result<StatusCode, StatusCode> {
    match user_repo.unlock_user(user_id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_user(
    State(user_repo): State<UserRepository>,
    Path(user_id): Path<UserId>,
    Extension(_admin_user): Extension<User>,
) -> Result<StatusCode, StatusCode> {
    match user_repo.delete_user(user_id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
