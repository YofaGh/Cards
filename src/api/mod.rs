pub mod admin;
pub mod auth;
pub mod handlers;
pub mod middleware;
pub mod models;

use axum::{
    routing::{get, post},
    Router,
};
use tokio::{net::TcpListener, task::JoinHandle};

use crate::{
    database::{AdminRepository, UserRepository},
    prelude::*,
};

fn create_router(pool: PgPool) -> Router {
    let user_repo: UserRepository = UserRepository::new(pool.clone());
    let admin_repo: AdminRepository = AdminRepository::new(pool.clone());
    let admin_auth_routes: Router<UserRepository> = Router::new()
        .route("/auth/admin/login", post(auth::admin_login))
        .with_state(admin_repo.clone());
    Router::new()
        .route("/health", get(handlers::health))
        .route("/auth/login", post(auth::login))
        .route("/auth/register", post(auth::register))
        .route("/games/available", get(handlers::get_available_games))
        .route("/games/join", post(handlers::join_game_queue))
        .merge(admin_auth_routes)
        .nest("/admin", admin::create_admin_router(admin_repo))
        .with_state(user_repo)
}

pub async fn init_api_server(pool: PgPool) -> Result<JoinHandle<()>> {
    let config: &'static Config = get_config();
    let address: &str = &format!("{}:{}", config.api_server.host, config.api_server.port);
    let api_listener: TcpListener = TcpListener::bind(address)
        .await
        .map_err(|err: std::io::Error| Error::bind_address(address, err))?;
    let app: Router = create_router(pool);
    let api_server: JoinHandle<()> = tokio::spawn(async move {
        axum::serve(api_listener, app)
            .await
            .expect("API server failed");
    });
    Ok(api_server)
}
