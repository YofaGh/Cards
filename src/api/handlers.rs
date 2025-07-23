use axum::response::Json;
use std::time::SystemTime;

use super::models::HealthResponse;

pub async fn health() -> Json<HealthResponse> {
    let start_time: SystemTime = SystemTime::UNIX_EPOCH;
    let uptime: u64 = SystemTime::now()
        .duration_since(start_time)
        .unwrap_or_default()
        .as_secs();
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: format!("{}s", uptime),
    })
}
