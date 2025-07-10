use tokio::{
    net::TcpListener,
    spawn,
    sync::mpsc::{channel, Sender},
};
use tokio_rustls::TlsAcceptor;

use {config::init_config, prelude::*, utils::game_registry::create_game};

mod client;
mod config;
mod constants;
mod enums;
mod errors;
mod games;
mod models;
mod prelude;
mod types;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "client" {
        return client::run();
    }
    #[cfg(all(debug_assertions, feature = "dev-certs"))]
    if !std::path::Path::new("cert.pem").exists() || !std::path::Path::new("key.pem").exists() {
        println!("Certificate files not found. Generating...");
        generate_self_signed_cert_rust()?;
    }
    init_config()?;
    let tls_acceptor: TlsAcceptor = get_tls_acceptor()?;
    let listener: TcpListener = get_listener().await?;
    let mut game: BoxGame = create_game("Qafoon")?;
    game.initialize_game()?;
    let (player_tx, mut player_rx) = channel(32);
    spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let player_tx: Sender<(Stream, String)> = player_tx.clone();
                    let acceptor: TlsAcceptor = tls_acceptor.clone();
                    spawn(async move {
                        let mut tls_stream: Stream = match acceptor.accept(stream).await {
                            Ok(tls_stream) => Stream::Server(tls_stream),
                            Err(e) => {
                                eprintln!("TLS handshake failed for {addr}: {e}");
                                return;
                            }
                        };
                        match handle_client(&mut tls_stream).await {
                            Ok(user) => {
                                if let Err(e) = player_tx.send((tls_stream, user)).await {
                                    eprintln!("Failed to send player to game: {e}");
                                }
                            }
                            Err(e) => {
                                eprintln!("Authentication failed for {addr}: {e}");
                            }
                        }
                    });
                }
                Err(e) => eprintln!("Failed to accept connection: {e}"),
            }
        }
    });
    while !game.is_full() {
        if let Some((tls_stream, user)) = player_rx.recv().await {
            game.handle_user(tls_stream, user).await?;
        }
    }
    game.broadcast_message(BroadcastMessage::GameStarting)
        .await?;
    game.run_game().await
}
