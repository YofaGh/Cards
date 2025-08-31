use axum::{
    response::Json,
    routing::{delete, get, post},
    Router,
};
use std::time::SystemTime;
use tokio::{net::TcpListener, task::JoinHandle};

use super::{admin, auth, games, models::HealthResponse};
use crate::{
    database::{AdminRepository, UserRepository},
    prelude::*,
};

pub fn get_token(headers: &axum::http::HeaderMap) -> Option<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header: &axum::http::HeaderValue| header.to_str().ok())
        .and_then(|header: &str| header.strip_prefix("Bearer "))
}

async fn health() -> Json<HealthResponse> {
    let start_time: SystemTime = SystemTime::UNIX_EPOCH;
    let uptime: u64 = SystemTime::now()
        .duration_since(start_time)
        .unwrap_or_default()
        .as_secs();
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: format!("{uptime}s"),
    })
}

fn create_router(pool: PgPool) -> Router {
    let user_repo: UserRepository = UserRepository::new(pool.clone());
    let admin_repo: AdminRepository = AdminRepository::new(pool.clone());
    let admin_auth_routes: Router<UserRepository> = Router::new()
        .route("/auth/admin/login", post(auth::admin_login))
        .with_state(admin_repo.clone());
    Router::new()
        .route("/health", get(health))
        .route("/auth/login", post(auth::login))
        .route("/auth/register", post(auth::register))
        .route("/games/available", get(games::get_available_games))
        .route("/games/join", post(games::join_game_queue))
        .route("/games/session/status", get(games::get_session_status))
        .route("/games/session/leave", delete(games::leave_game_session))
        .merge(admin_auth_routes)
        .nest("/admin", admin::create_admin_router(admin_repo))
        .with_state(user_repo)
}

pub async fn init_api_server(pool: PgPool) -> Result<JoinHandle<()>> {
    let config: &Config = get_config();
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
    println!("API server started successfully");
    Ok(api_server)
}
