use tokio::net::TcpListener;

use crate::{database::UserRepository, network::protocol::*, prelude::*};

pub async fn get_listener() -> Result<TcpListener> {
    let config: &'static Config = get_config();
    let address: &str = &format!("{}:{}", config.game_server.host, config.game_server.port);
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

pub async fn get_username(connection: &mut Stream, user_repo: UserRepository) -> Result<String> {
    send_message(connection, &GameMessage::demand(DemandMessage::JwtToken)).await?;
    match receive_message(connection).await? {
        GameMessage::JwtToken { token } => {
            if !token.is_empty() {
                let claims: crate::auth::Claims = match crate::auth::validate_token(&token) {
                    Ok(claims) => claims,
                    _ => return Err(Error::Other("Invalid token".to_string())),
                };
                let user_id: UserId = match claims.sub.parse::<UserId>() {
                    Ok(id) => id,
                    _ => return Err(Error::Other("Invalid token".to_string())),
                };
                let user: crate::database::User = match user_repo.get_user_by_id(user_id).await {
                    Ok(Some(user)) => user,
                    _ => return Err(Error::Other("Invalid token".to_string())),
                };
                return Ok(user.username);
            }
            return Err(Error::Other("Invalid token".to_string()));
        }
        invalid => {
            close_connection(connection).await?;
            return Err(Error::InvalidResponse(
                "JwtToken".to_string(),
                invalid.message_type(),
            ));
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

pub async fn handle_client(
    connection: &mut Stream,
    user_repo: UserRepository,
) -> Result<(String, String)> {
    handshake(connection).await?;
    let username: String = get_username(connection, user_repo).await?;
    let game_choice: String = get_game_choice(connection).await?;
    Ok((username, game_choice))
}
