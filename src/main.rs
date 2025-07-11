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
    core::{create_tracked_game, get_game_registry},
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
    loop {
        println!("Creating new Qafoon game...");
        let (game_id, game_arc) = create_tracked_game("Qafoon").await?;
        game_arc.lock().await.initialize_game()?;
        {
            let mut game: MutexGuard<BoxGame> = game_arc.lock().await;
            while !game.is_full() {
                if let Some((tls_stream, user)) = player_rx.recv().await {
                    println!("Adding player {user} to game {game_id}");
                    if let Err(e) = game.handle_user(tls_stream, user).await {
                        eprintln!("Error adding player to game {game_id}: {e}");
                        continue;
                    }
                }
            }
            println!("Game {game_id} is full, starting...");
        }
        spawn(async move {
            let mut game: MutexGuard<BoxGame> = game_arc.lock().await;
            if let Err(e) = game.broadcast_message(BroadcastMessage::GameStarting).await {
                eprintln!("Error broadcasting game start for {game_id}: {e}");
                return;
            }
            if let Err(e) = game.run_game().await {
                eprintln!("Error running game {game_id}: {e}");
            } else {
                println!("Game {game_id} completed successfully");
            }
            get_game_registry().remove_game(game_id).await.ok();
        });
        println!("Game {game_id} started, ready for next game");
    }
}
