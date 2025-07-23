pub mod users;

use axum::{
    routing::{delete, get, post},
    Router,
};

pub fn create_admin_router(
    admin_repo: crate::database::AdminRepository,
) -> Router<crate::database::UserRepository> {
    Router::new()
        .route("/users/{id}", get(users::get_user))
        .route("/users/{id}/lock", post(users::lock_user))
        .route("/users/create", post(users::create_user))
        .route("/users/{id}/unlock", post(users::unlock_user))
        .route("/users/{id}", delete(users::delete_user))
        .route("/health", get(super::handlers::health))
        .layer(axum::middleware::from_fn_with_state(
            admin_repo,
            super::middleware::admin_auth_middleware,
        ))
}
