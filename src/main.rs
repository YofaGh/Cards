use std::net::TcpListener;

mod constants;
mod enums;
mod errors;
mod game;
mod models;
mod prelude;
mod types;
mod utils;

use {game::Game, prelude::*};

fn main() -> Result<()> {
    let mut game: Game = Game::new();
    game.initialize_game()?;
    let listener: TcpListener = get_listener()?;
    while !game.is_full() {
        match listener.accept() {
            Ok((stream, _)) => game.handle_client(stream)?,
            Err(err) => println!("Connection error: {}", err),
        }
    }
    game.broadcast_message("All players connected. Game starting...!")?;
    game.run_game()
}
