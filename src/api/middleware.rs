use axum::{extract::State, http::StatusCode};

use crate::prelude::*;

pub async fn admin_auth_middleware(
    State(admin_repo): State<crate::database::AdminRepository>,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    let auth_header: Option<&str> = super::handlers::get_token(request.headers());
    let token: &str = match auth_header {
        Some(token) => token,
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    let claims: crate::auth::Claims = match crate::auth::validate_token(token) {
        Ok(claims) => claims,
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    if !claims.is_admin {
        return Err(StatusCode::FORBIDDEN);
    }
    let admin_id: AdminId = match claims.sub.parse::<AdminId>() {
        Ok(id) => id,
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    let admin: crate::database::Admin = match admin_repo.get_admin_by_id(admin_id).await {
        Ok(Some(admin)) => admin,
        _ => return Err(StatusCode::FORBIDDEN),
    };
    if !admin.is_active {
        return Err(StatusCode::FORBIDDEN);
    }
    request.extensions_mut().insert(admin);
    Ok(next.run(request).await)
}
