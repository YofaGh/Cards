use tokio::net::TcpListener;

use crate::{network::protocol::*, prelude::*};

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

pub async fn handle_client(connection: &mut Stream) -> Result<(String, String)> {
    handshake(connection).await?;
    let (username, game_choice) = get_game_session_info(connection).await?;
    Ok((username, game_choice))
}

pub async fn get_game_session_info(connection: &mut Stream) -> Result<(String, String)> {
    send_message(
        connection,
        &GameMessage::demand(DemandMessage::GameSessionToken),
    )
    .await?;
    match receive_message(connection).await? {
        GameMessage::GameSessionToken { token } => {
            if token.is_empty() {
                return Err(Error::Other("Empty game session token".to_string()));
            }
            let claims: crate::auth::GameSessionClaims =
                match crate::auth::validate_token(&token) {
                    Ok(claims) => claims,
                    _ => return Err(Error::Other("Invalid game session token".to_string())),
                };
            let now: usize = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize;
            if claims.exp < now {
                return Err(Error::Other("Game session token expired".to_string()));
            }
            Ok((claims.username, claims.game_choice))
        }
        invalid => {
            close_connection(connection).await?;
            Err(Error::InvalidResponse(
                "GameSessionToken".to_string(),
                invalid.message_type(),
            ))
        }
    }
}
