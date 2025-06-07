use itertools::Itertools;
use std::net::TcpListener;

mod constants;
mod enums;
mod errors;
mod game;
mod macros;
mod models;
mod prelude;
mod tcp_messenger;
mod types;

use {constants::INVALID_RESPONSE, game::Game, models::Player, prelude::*};

fn client_handler(connection: TcpStream, game: &Game) -> Result<()> {
    let message: &str = "1$_$_$Choose your name:";
    send_message(&connection, message)?;
    let name: String = receive_message(&connection)?;
    let mut pre: &str = "";
    loop {
        let available_teams: Vec<(TeamId, String)> = game.get_available_team()?;
        let available_teams_str: String = available_teams
            .iter()
            .enumerate()
            .map(|(i, (_, name))| format!("{}:{}", name, i))
            .join(", ");
        let message: String = format!("1$_$_${pre}Choose your team: {available_teams_str}");
        send_message(&connection, &message)?;
        match receive_message(&connection)?.parse::<usize>() {
            Ok(team) if team < available_teams.len() => {
                game.add_player(Player::new(name, available_teams[team].0, connection))?;
                return Ok(());
            }
            _ => pre = INVALID_RESPONSE,
        }
    }
}

fn main() -> Result<()> {
    let mut game: Game = Game::new();
    game.initialize_game()?;
    let listener: TcpListener = get_listener()?;
    while !game.is_full()? {
        match listener.accept() {
            Ok((stream, _)) => {
                client_handler(stream, &game)?;
            }
            Err(err) => println!("Connection error: {}", err),
        }
    }
    game.broadcast_message("All players connected. Game starting...!")?;
    game.run_game()
}
