#[cfg(all(debug_assertions, feature = "dev-certs"))]
use rcgen::{CertificateParams, DistinguishedName};
use rmp_serde::{from_slice, to_vec};
use rustls::{Certificate, PrivateKey, ServerConfig};
use std::{fs, io::Error as IoError, path::Path, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use tokio_rustls::TlsAcceptor;

use crate::{
    constants::INVALID_RESPONSE,
    models::{Card, Player},
    prelude::*,
};

pub async fn get_player_choice(
    player: &mut Player,
    message: &mut GameMessage,
    passable: bool,
    max_value: usize,
) -> Result<PlayerChoice> {
    loop {
        player.send_message(message).await?;
        match player.receive_message().await? {
            GameMessage::PlayerChoice { choice } => {
                if choice == "pass" {
                    if passable {
                        return Ok(PlayerChoice::Pass);
                    }
                    message.set_error("You can't pass this one".to_owned());
                } else if message.message_type() == "Hokm" {
                    return Ok(PlayerChoice::HokmChoice(Hokm::from(choice)));
                } else if message.message_type() == "Bet" {
                    if let Ok(choice) = choice.parse::<usize>() {
                        if choice <= max_value {
                            return Ok(PlayerChoice::NumberChoice(choice));
                        }
                        message.set_error(format!("Choice can't be greater than {max_value}"));
                    } else {
                        message.set_error(INVALID_RESPONSE.to_owned());
                    }
                } else {
                    match Card::try_from(choice) {
                        Ok(card) => {
                            if player.hand.contains(&card) {
                                return Ok(PlayerChoice::CardChoice(card));
                            }
                            message.set_error("You don't have this card!".to_owned());
                        }
                        Err(_) => message.set_error(INVALID_RESPONSE.to_owned()),
                    }
                }
            }
            invalid => {
                message.set_error(format!(
                    "Expected message type PlayerChoice, but received {}",
                    invalid.message_type()
                ));
            }
        }
    }
}

pub fn code_cards(cards: &Vec<Card>) -> Vec<String> {
    cards.iter().map(|card: &Card| card.code()).collect()
}

pub async fn send_message(connection: &mut Stream, message: &GameMessage) -> Result<()> {
    let data: Vec<u8> = to_vec(message).map_err(Error::serialization)?;
    let length: u32 = data.len() as u32;
    connection
        .write_all(&length.to_be_bytes())
        .await
        .map_err(Error::connection)?;
    connection
        .write_all(&data)
        .await
        .map_err(Error::connection)?;
    connection.flush().await.map_err(Error::connection)
}

pub async fn receive_message(connection: &mut Stream) -> Result<GameMessage> {
    let mut length_buf: [u8; 4] = [0u8; 4];
    connection
        .read_exact(&mut length_buf)
        .await
        .map_err(Error::connection)?;
    let message_length: usize = u32::from_be_bytes(length_buf) as usize;
    let mut message_buf: Vec<u8> = vec![0u8; message_length];
    connection
        .read_exact(&mut message_buf)
        .await
        .map_err(Error::connection)?;
    from_slice(&message_buf).map_err(Error::deserialization)
}

pub async fn close_connection(connection: &mut Stream) -> Result<()> {
    connection.shutdown().await.map_err(Error::connection)
}

pub async fn get_listener() -> Result<TcpListener> {
    let config: &'static Config = get_config();
    let address: &str = &format!("{}:{}", config.server.host, config.server.port);
    TcpListener::bind(address)
        .await
        .map_err(|err: IoError| Error::bind_address(address, err))
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

pub async fn handle_client(connection: &mut Stream) -> Result<String> {
    handshake(connection).await?;
    let mut message: GameMessage = GameMessage::username();
    loop {
        send_message(connection, &message).await?;
        match receive_message(connection).await? {
            GameMessage::UsernameResponse { username } => {
                if !username.is_empty() {
                    return Ok(username);
                }
                message.set_error("Username can not be empty!".to_owned());
            }
            invalid => {
                close_connection(connection).await?;
                return Err(Error::InvalidResponse(
                    "UsernameResponse".to_string(),
                    invalid.message_type(),
                ));
            }
        }
    }
}

pub fn read_file(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    fs::read(path).map_err(Error::read_file)
}

#[cfg(all(debug_assertions, feature = "dev-certs"))]
pub fn generate_self_signed_cert_rust() -> Result<()> {
    println!("Generating self-signed certificate with Rust...");
    let mut params: CertificateParams =
        CertificateParams::new(vec!["localhost".to_string(), "127.0.0.1".to_string()]);
    let mut distinguished_name: DistinguishedName = DistinguishedName::new();
    distinguished_name.push(rcgen::DnType::CommonName, "localhost");
    distinguished_name.push(rcgen::DnType::OrganizationName, "Game Server");
    distinguished_name.push(rcgen::DnType::CountryName, "IR");
    params.distinguished_name = distinguished_name;
    let cert: rcgen::Certificate = rcgen::Certificate::from_params(params).unwrap();
    let pem_serialized: String = cert.serialize_pem().unwrap();
    fs::write("cert.pem", pem_serialized).map_err(|err: IoError| {
        Error::FileOperation(format!("unable to write file error: {err}"))
    })?;
    fs::write("key.pem", cert.serialize_private_key_pem()).map_err(|err: IoError| {
        Error::FileOperation(format!("unable to write file error: {err}"))
    })?;
    println!("Certificate generated successfully!");
    Ok(())
}

fn load_tls_config() -> Result<Arc<ServerConfig>> {
    let config: &'static Config = get_config();
    let cert_file: Vec<u8> = read_file(&config.tls.cert)?;
    let key_file: Vec<u8> = read_file(&config.tls.key)?;
    let cert_chain: Vec<Certificate> = rustls_pemfile::certs(&mut cert_file.as_slice())
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e: IoError| Error::Tls(format!("Failed to parse certificates: {}", e)))?
        .into_iter()
        .map(|cert| Certificate(cert.to_vec()))
        .collect();
    let keys: Vec<Vec<u8>> = rustls_pemfile::pkcs8_private_keys(&mut key_file.as_slice())
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e: IoError| Error::Tls(format!("Failed to parse private keys: {}", e)))?
        .into_iter()
        .map(|key| key.secret_pkcs8_der().to_vec())
        .collect();
    if keys.is_empty() {
        return Err(Error::Tls("No private keys found".to_string()));
    }
    let first_key: Vec<u8> = keys
        .into_iter()
        .next()
        .ok_or_else(|| Error::Tls("No private keys available after parsing".to_string()))?;
    let tls_config: ServerConfig = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, PrivateKey(first_key))
        .map_err(|e: rustls::Error| Error::Tls(format!("Failed to build TLS config: {}", e)))?;
    Ok(Arc::new(tls_config))
}

pub fn get_tls_acceptor() -> Result<TlsAcceptor> {
    Ok(TlsAcceptor::from(load_tls_config()?))
}
