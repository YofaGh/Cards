use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

use {
    config::init_config,
    core::get_game_registry,
    network::{get_listener, get_tls_acceptor, handle_client},
    prelude::*,
};

mod config;
mod core;
mod errors;
mod games;
mod macros;
mod models;
mod network;
mod prelude;

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(all(debug_assertions, feature = "dev-certs"))]
    if !std::path::Path::new("cert.pem").exists() || !std::path::Path::new("key.pem").exists() {
        println!("Certificate files not found. Generating...");
        generate_self_signed_cert_rust()?;
    }
    init_config()?;
    let tls_acceptor: TlsAcceptor = get_tls_acceptor()?;
    let listener: TcpListener = get_listener().await?;
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let acceptor: TlsAcceptor = tls_acceptor.clone();
                tokio::spawn(async move {
                    let mut tls_stream: Stream = match acceptor.accept(stream).await {
                        Ok(tls_stream) => Stream::Server(tls_stream),
                        Err(err) => {
                            eprintln!("TLS handshake failed for {addr}: {err}");
                            return;
                        }
                    };
                    match handle_client(&mut tls_stream).await {
                        Ok((username, game_choice)) => {
                            println!("Player {username} wants to play {game_choice}");
                            if let Err(err) = get_game_registry()
                                .add_player_to_queue(
                                    username.clone(),
                                    game_choice.clone(),
                                    tls_stream,
                                )
                                .await
                            {
                                eprintln!(
                                    "Failed to add player {username} to {game_choice} queue: {err}"
                                );
                            }
                        }
                        Err(err) => {
                            eprintln!("Client handling failed for {addr}: {err}");
                        }
                    }
                });
            }
            Err(err) => eprintln!("Failed to accept connection: {err}"),
        }
    }
}
