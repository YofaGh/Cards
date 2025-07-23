pub mod admin;
pub mod auth;
pub mod handlers;
pub mod middleware;

use axum::{
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;

use crate::{database::UserRepository, prelude::*};

fn create_router(user_repo: UserRepository) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/auth/login", post(auth::login))
        .nest("/admin", admin::create_admin_router(user_repo.clone()))
        .with_state(user_repo)
}

pub async fn init_api_server(user_repo: UserRepository) -> Result<tokio::task::JoinHandle<()>> {
    let config: &'static Config = get_config();
    let address: &str = &format!("{}:{}", config.api_server.host, config.api_server.port);
    let api_listener: TcpListener = TcpListener::bind(address)
        .await
        .map_err(|err: std::io::Error| Error::bind_address(address, err))?;
    let app: Router = create_router(user_repo);
    let api_server: tokio::task::JoinHandle<()> = tokio::spawn(async move {
        axum::serve(api_listener, app)
            .await
            .expect("API server failed");
    });
    Ok(api_server)
}
