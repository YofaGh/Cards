use tokio::{net::TcpListener, task::JoinHandle};
use tokio_rustls::TlsAcceptor;

use crate::{
    auth::{identify_and_decode_token, GameSessionClaims, ReconnectClaims, SessionTokenType},
    network::protocol::*,
    prelude::*,
};

async fn get_listener() -> Result<TcpListener> {
    let config: &'static Config = get_config();
    let address: &str = &format!("{}:{}", config.game_server.host, config.game_server.port);
    TcpListener::bind(address)
        .await
        .map_err(|err: std::io::Error| Error::bind_address(address, err))
}

async fn handshake(connection: &mut Stream) -> Result<()> {
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

pub async fn handle_client(connection: &mut Stream) -> Result<SessionTokenType> {
    handshake(connection).await?;
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
            identify_and_decode_token(&token)
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

pub fn get_game_session_info(claims: GameSessionClaims) -> Result<(String, String)> {
    let now: usize = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| Error::Other("System time error".to_string()))?
        .as_secs() as usize;
    if claims.exp < now {
        return Err(Error::Other("Game session token expired".to_string()));
    }
    Ok((claims.username, claims.game_choice))
}

pub fn get_reconnection_info(claims: ReconnectClaims) -> Result<(String, String)> {
    let now: usize = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| Error::Other("System time error".to_string()))?
        .as_secs() as usize;
    if claims.exp < now {
        return Err(Error::Other("Game session token expired".to_string()));
    }
    Ok((claims.sub, claims.game_id))
}

pub async fn init_game_server() -> Result<JoinHandle<()>> {
    super::tls::init_crypto_provider();
    let tls_acceptor: TlsAcceptor = super::tls::get_tls_acceptor()?;
    let listener: TcpListener = get_listener().await?;
    let game_server: JoinHandle<()> = tokio::spawn(async move {
        println!("Game server started successfully");
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
                            Ok(SessionTokenType::GameSession(claims)) => {
                                match get_game_session_info(claims) {
                                    Ok((username, game_choice)) => {
                                        println!("Player {username} wants to play {game_choice}");
                                        if let Err(err) = crate::core::get_game_registry()
                                            .add_player_to_queue(
                                                username.clone(),
                                                game_choice.clone(),
                                                tls_stream,
                                            )
                                            .await
                                        {
                                            eprintln!("Failed to add player {username} to {game_choice} queue: {err}");
                                        }
                                    }
                                    Err(err) => {
                                        eprintln!("Client handling failed for {addr}: {err}");
                                    }
                                }
                            }
                            Ok(SessionTokenType::Reconnection(claims)) => {
                                match get_reconnection_info(claims) {
                                    Ok((player_id, game_id)) => {
                                        println!(
                                            "Player {player_id} wants to reconnect to {game_id}"
                                        );
                                        if let Err(err) = crate::core::get_game_registry()
                                            .reconnect_player(
                                                player_id.clone(),
                                                game_id.clone(),
                                                tls_stream,
                                            )
                                            .await
                                        {
                                            eprintln!("Failed to reconnect player {player_id} to game {game_id}: {err}");
                                        }
                                    }
                                    Err(err) => {
                                        eprintln!("Client handling failed for {addr}: {err}");
                                    }
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
    });
    Ok(game_server)
}
