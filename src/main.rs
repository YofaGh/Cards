mod api;
mod auth;
mod config;
mod core;
mod database;
mod errors;
mod games;
mod macros;
mod models;
mod network;
mod prelude;

#[tokio::main]
async fn main() -> core::types::Result<()> {
    #[cfg(all(debug_assertions, feature = "dev-certs"))]
    crate::network::tls::generate_self_signed_cert_rust()?;
    config::init_config()?;
    let pool: sqlx::PgPool = database::create_database_pool().await?;
    database::test_database_connection(&pool).await?;
    database::run_migrations(&pool).await?;
    let api_server = api::init_api_server(pool);
    let game_server = network::init_game_server();
    println!("Servers started successfully");
    tokio::select! {
        result = api_server.await? => {
            eprintln!("API server exited unexpectedly: {:?}", result);
        }
        result = game_server.await? => {
            eprintln!("Game server exited unexpectedly: {:?}", result);
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Shutdown signal received, stopping servers...");
        }
    }
    Ok(())
}
