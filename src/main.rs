use tokio::{
    net::TcpListener,
    spawn,
    sync::{
        mpsc::{channel, Sender},
        MutexGuard,
    },
};
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
    let (player_tx, mut player_rx) = channel(32);
    spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let player_tx: Sender<(Stream, String, String)> = player_tx.clone();
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
                            Ok((username, game_choice)) => {
                                if let Err(e) =
                                    player_tx.send((tls_stream, username, game_choice)).await
                                {
                                    eprintln!("Failed to send player to matchmaking: {e}");
                                }
                            }
                            Err(e) => {
                                eprintln!("Client handling failed for {addr}: {e}");
                            }
                        }
                    });
                }
                Err(e) => eprintln!("Failed to accept connection: {e}"),
            }
        }
    });
    loop {
        if let Some((tls_stream, username, game_choice)) = player_rx.recv().await {
            println!("Player {username} wants to play {game_choice}");
            match get_game_registry().find_or_create_queue(&game_choice).await {
                Ok(game_arc) => {
                    let mut game: MutexGuard<BoxGame> = game_arc.lock().await;
                    if let Err(e) = game.handle_user(tls_stream, username.clone()).await {
                        eprintln!("Error adding player {username} to {game_choice} queue: {e}");
                        continue;
                    }
                    println!("Player {username} added to {game_choice} queue");
                    if game.is_full() {
                        println!("{game_choice} queue is full, promoting to active game");
                        drop(game);
                        match get_game_registry()
                            .promote_queue_to_active(&game_choice)
                            .await
                        {
                            Ok(game_id) => {
                                println!("Started {game_choice} game with ID: {game_id}");
                                spawn(async move {
                                    if let Err(e) = game_arc.lock().await.start().await {
                                        eprintln!("Error starting the game {game_id}: {e}");
                                        return;
                                    }
                                    get_game_registry().remove_game(game_id).await.ok();
                                });
                            }
                            Err(e) => {
                                eprintln!("Failed to promote {game_choice} queue to active: {e}");
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to find/create queue for {game_choice}: {e}");
                }
            };
        }
    }
}
