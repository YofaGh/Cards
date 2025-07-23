use axum::{extract::State, http::StatusCode};

use crate::prelude::*;

pub async fn admin_auth_middleware(
    State(user_repo): State<crate::database::UserRepository>,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    let auth_header: Option<&str> = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header: &axum::http::HeaderValue| header.to_str().ok())
        .and_then(|header: &str| header.strip_prefix("Bearer "));
    let token: &str = match auth_header {
        Some(token) => token,
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    let claims: crate::auth::Claims = match crate::auth::validate_token(token) {
        Ok(claims) => claims,
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    let user_id: UserId = match claims.sub.parse::<UserId>() {
        Ok(id) => id,
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    let user: crate::database::User = match user_repo.get_user_by_id(user_id).await {
        Ok(Some(user)) => user,
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    if !user.is_admin {
        return Err(StatusCode::FORBIDDEN);
    }
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}
