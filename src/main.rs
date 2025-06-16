use tokio::{
    net::TcpListener,
    spawn,
    sync::mpsc::{channel, Sender},
};
use {game::Game, prelude::*, utils::assets::handle_client};

mod constants;
mod enums;
mod errors;
mod game;
mod models;
mod prelude;
mod types;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let listener: TcpListener = get_listener().await?;
    let mut game: Game = Game::new();
    game.initialize_game()?;
    let (player_tx, mut player_rx) = channel(32);
    spawn(async move {
        loop {
            match listener.accept().await {
                Ok((mut stream, addr)) => {
                    let player_tx: Sender<(TcpStream, String)> = player_tx.clone();
                    spawn(async move {
                        match handle_client(&mut stream).await {
                            Ok(user) => {
                                if let Err(e) = player_tx.send((stream, user)).await {
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
        if let Some((stream, user)) = player_rx.recv().await {
            game.handle_user(stream, user).await?;
        }
    }
    game.broadcast_message("All players connected. Game starting...!")
        .await?;
    game.run_game().await
}
