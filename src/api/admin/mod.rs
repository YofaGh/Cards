pub mod users;

use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::database::UserRepository;

pub fn create_admin_router(user_repo: UserRepository) -> Router<UserRepository> {
    Router::new()
        .route("/users/{id}", get(users::get_user))
        .route("/users/{id}/lock", post(users::lock_user))
        .route("/users/{id}/unlock", post(users::unlock_user))
        .route("/users/{id}", delete(users::delete_user))
        .route("/health", get(super::handlers::health))
        .layer(axum::middleware::from_fn_with_state(
            user_repo,
            super::middleware::admin_auth_middleware,
        ))
}
