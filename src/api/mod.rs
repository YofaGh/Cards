pub mod admin;
pub mod auth;
pub mod handlers;
pub mod middleware;
pub mod models;

use axum::{
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;

use crate::{
    database::{AdminRepository, UserRepository},
    prelude::*,
};

fn create_router(user_repo: UserRepository, admin_repo: AdminRepository) -> Router {
    let admin_auth_routes: Router<UserRepository> = Router::new()
        .route("/auth/admin/login", post(auth::admin_login))
        .with_state(admin_repo.clone());
    Router::new()
        .route("/health", get(handlers::health))
        .route("/auth/login", post(auth::login))
        .merge(admin_auth_routes)
        .nest("/admin", admin::create_admin_router(admin_repo))
        .with_state(user_repo)
}

pub async fn init_api_server(
    user_repo: UserRepository,
    admin_repo: AdminRepository,
) -> Result<tokio::task::JoinHandle<()>> {
    let config: &'static Config = get_config();
    let address: &str = &format!("{}:{}", config.api_server.host, config.api_server.port);
    let api_listener: TcpListener = TcpListener::bind(address)
        .await
        .map_err(|err: std::io::Error| Error::bind_address(address, err))?;
    let app: Router = create_router(user_repo, admin_repo);
    let api_server: tokio::task::JoinHandle<()> = tokio::spawn(async move {
        axum::serve(api_listener, app)
            .await
            .expect("API server failed");
    });
    Ok(api_server)
}
