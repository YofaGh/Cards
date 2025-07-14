use tokio::net::TcpListener;

use crate::{network::protocol::*, prelude::*};

pub async fn get_listener() -> Result<TcpListener> {
    let config: &'static Config = get_config();
    let address: &str = &format!("{}:{}", config.server.host, config.server.port);
    TcpListener::bind(address)
        .await
        .map_err(|err: std::io::Error| Error::bind_address(address, err))
}

pub async fn handshake(connection: &mut Stream) -> Result<()> {
    send_message(connection, &GameMessage::Handshake).await?;
    match receive_message(connection).await? {
        GameMessage::HandshakeResponse => Ok(()),
        invalid => {
            close_connection(connection).await?;
            Err(Error::InvalidResponse(
                GameMessage::HandshakeResponse.message_type(),
                invalid.message_type(),
            ))
        }
    }
}

pub async fn get_username(connection: &mut Stream) -> Result<String> {
    let mut message: GameMessage = GameMessage::demand(DemandMessage::Username);
    loop {
        send_message(connection, &message).await?;
        match receive_message(connection).await? {
            GameMessage::PlayerChoice { choice } => {
                if !choice.is_empty() {
                    return Ok(choice);
                }
                message.set_demand_error("Username can not be empty!".to_owned());
            }
            invalid => {
                close_connection(connection).await?;
                return Err(Error::InvalidResponse(
                    "PlayerChoice".to_string(),
                    invalid.message_type(),
                ));
            }
        }
    }
}

pub async fn get_game_choice(connection: &mut Stream) -> Result<String> {
    loop {
        let available_games: Vec<String> = crate::core::get_game_registry().get_available_games();
        let mut message: GameMessage = GameMessage::demand(DemandMessage::Game {
            available_games: available_games.clone(),
        });
        send_message(connection, &message).await?;
        match receive_message(connection).await? {
            GameMessage::PlayerChoice { choice } => {
                if available_games.contains(&choice) {
                    return Ok(choice);
                }
                message.set_demand_error(format!(
                    "Invalid game choice '{}'. Available games: {}",
                    choice,
                    available_games.join(", ")
                ));
            }
            invalid => {
                close_connection(connection).await?;
                return Err(Error::InvalidResponse(
                    "PlayerChoice".to_string(),
                    invalid.message_type(),
                ));
            }
        }
    }
}

pub async fn handle_client(connection: &mut Stream) -> Result<(String, String)> {
    handshake(connection).await?;
    let username: String = get_username(connection).await?;
    let game_choice: String = get_game_choice(connection).await?;
    Ok((username, game_choice))
}
