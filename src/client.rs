use rustls::{
    client::{ServerCertVerified, ServerCertVerifier},
    ClientConfig, ClientConnection, StreamOwned,
};
use std::net::TcpStream;
use std::sync::Arc;
use std::{
    io::{self, Read, Write},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{
    config::{get_config, init_config, Config},
    constants::{HOKMS, TYPES},
    models::Card,
    prelude::*,
};

static PLAYER_CARDS: RwLock<Vec<Card>> = RwLock::new(Vec::new());
static GROUND_CARDS: RwLock<Vec<(String, Card)>> = RwLock::new(Vec::new());
static HOKM: RwLock<Hokm> = RwLock::new(Hokm::Default);
static CUR_BET: RwLock<usize> = RwLock::new(0);

struct NoVerifier;

impl ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}

fn set_cards(new_cards: Vec<Card>) {
    let mut cards: RwLockWriteGuard<Vec<Card>> = PLAYER_CARDS.write().unwrap();
    *cards = new_cards;
}

fn set_hokm(new_hokm: String) {
    let mut hokm: RwLockWriteGuard<Hokm> = HOKM.write().unwrap();
    *hokm = Hokm::from(new_hokm);
}

fn add_cards(new_cards: Vec<Card>) {
    let mut cards: RwLockWriteGuard<Vec<Card>> = PLAYER_CARDS.write().unwrap();
    cards.extend(new_cards);
}

fn set_ground_cards(new_cards: Vec<(String, String)>) {
    let mut ground_cards: RwLockWriteGuard<Vec<(String, Card)>> = GROUND_CARDS.write().unwrap();
    let mut cards: Vec<(String, Card)> = vec![];
    for (player_name, card_code) in new_cards {
        cards.push((player_name, Card::try_from(card_code).unwrap()));
    }
    *ground_cards = cards;
}

fn print_ground_cards() {
    let ground_cards: RwLockReadGuard<Vec<(String, Card)>> = GROUND_CARDS.read().unwrap();
    if ground_cards.is_empty() {
        return;
    }
    println!("Played Cards:");
    println!(
        "{}",
        ground_cards
            .iter()
            .map(|(player_name, card)| format!("{player_name}: {}", card.to_string()))
            .collect::<Vec<String>>()
            .join(", ")
    )
}

fn send_message(
    connection: &mut StreamOwned<ClientConnection, TcpStream>,
    message: &GameMessage,
) -> Result<()> {
    let data: Vec<u8> = rmp_serde::to_vec(message).map_err(Error::serialization)?;
    let length: u32 = data.len() as u32;
    connection
        .write_all(&length.to_be_bytes())
        .map_err(Error::connection)?;
    connection.write_all(&data).map_err(Error::connection)?;
    connection.flush().map_err(Error::connection)
}

fn receive_message(
    connection: &mut StreamOwned<ClientConnection, TcpStream>,
) -> Result<GameMessage> {
    let mut length_buf: [u8; 4] = [0u8; 4];
    connection
        .read_exact(&mut length_buf)
        .map_err(Error::connection)?;
    let message_length: usize = u32::from_be_bytes(length_buf) as usize;
    let mut message_buf: Vec<u8> = vec![0u8; message_length];
    connection
        .read_exact(&mut message_buf)
        .map_err(Error::connection)?;
    rmp_serde::from_slice(&message_buf).map_err(Error::deserialization)
}

fn handle_handshake(connection: &mut StreamOwned<ClientConnection, TcpStream>) -> Result<()> {
    send_message(connection, &GameMessage::HandshakeResponse)
}

fn get_formatted_scores(scores: Vec<(String, usize)>) -> String {
    scores
        .into_iter()
        .map(|(team_name, score)| format!("{team_name}: {score}"))
        .collect::<Vec<String>>()
        .join(", ")
}

fn print_hokm() {
    let hokm: RwLockReadGuard<Hokm> = HOKM.read().unwrap();
    println!("Hokm: {hokm}");
}

fn set_bet(new_bet: usize) {
    let mut bet: RwLockWriteGuard<usize> = CUR_BET.write().unwrap();
    *bet = new_bet;
}

fn handle_broadcast(message: BroadcastMessage) {
    match message {
        BroadcastMessage::GameStarting => println!("All players connected. Game starting...!"),
        BroadcastMessage::HandingOutCards => println!("Handing out cards..."),
        BroadcastMessage::ShufflingCards => println!("Shuffling cards..."),
        BroadcastMessage::Starter { name } => println!("Starter: {name}"),
        BroadcastMessage::Hokm { hokm } => {
            set_hokm(hokm);
            print_hokm();
        }
        BroadcastMessage::Bets { bets } => {
            let mut bets_str: Vec<String> = vec![];
            for (player_name, choice) in bets {
                match choice {
                    PlayerChoice::Pass => bets_str.push(format!("{player_name}: pass")),
                    PlayerChoice::NumberChoice(number) => {
                        bets_str.push(format!("{player_name}: {number}"))
                    }
                    _ => {}
                }
            }
            println!("{}", bets_str.join(", "))
        }
        BroadcastMessage::BetWinner { bet_winner } => {
            set_bet(bet_winner.1);
            println!("{} wins with {}", bet_winner.0, bet_winner.1);
        }
        BroadcastMessage::GroundCards { ground_cards } => {
            set_ground_cards(ground_cards);
            print_ground_cards();
        }
        BroadcastMessage::RoundWinner { round_winner } => {
            println!("Winner of this round is: {round_winner}")
        }
        BroadcastMessage::GameWinner { game_winner } => {
            println!("Winner of this round is: {game_winner}")
        }
        BroadcastMessage::GameScore { teams_score } => {
            println!("Game Score:\n {}", get_formatted_scores(teams_score))
        }
        BroadcastMessage::RoundScore { teams_score } => {
            println!("Round Score:\n {}", get_formatted_scores(teams_score))
        }
    }
}

fn username(
    error: String,
    connection: &mut StreamOwned<ClientConnection, TcpStream>,
) -> Result<()> {
    if !error.is_empty() {
        println!("Server error: {error}");
    }
    loop {
        let mut choice: String = String::new();
        print!("Enter your username: ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");
        choice = choice.trim().to_string();
        if !choice.is_empty() {
            return send_message(connection, &GameMessage::PlayerChoice { choice });
        }
        println!("Username can not be empty!")
    }
}

fn choose(prompt: String, server_error: String, max_value: usize, passable: bool) -> usize {
    if !server_error.is_empty() {
        println!("Server error: {server_error}");
    }
    loop {
        let mut input: String = String::new();
        print!("{prompt} (0-{max_value}): ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        input = input.trim().to_string();
        if input == "pass" {
            if passable {
                return 0;
            }
            println!("You can't pass this one!");
            continue;
        }
        if let Ok(choice) = input.parse::<usize>() {
            if choice <= max_value {
                return choice;
            }
            println!("Choice can't be greater than {max_value}");
        } else {
            println!("Please enter a number from 0 to {max_value}");
        }
    }
}

fn team_choice(
    available_teams: Vec<String>,
    error: String,
    connection: &mut StreamOwned<ClientConnection, TcpStream>,
) -> Result<()> {
    println!(
        "{}",
        available_teams
            .iter()
            .enumerate()
            .map(|(i, team_name)| format!("{team_name}: {i}"))
            .collect::<Vec<String>>()
            .join(", ")
    );
    let choice: usize = choose(
        "Choose a team:".to_string(),
        error,
        available_teams.len() - 1,
        false,
    );
    send_message(
        connection,
        &GameMessage::PlayerChoice {
            choice: available_teams[choice].clone(),
        },
    )
}

fn bet(error: String, connection: &mut StreamOwned<ClientConnection, TcpStream>) -> Result<()> {
    print_player_cards(false);
    let choice: usize = choose("what is your bet: ".to_string(), error, 13, true);
    let choice: String = if choice == 0 {
        "pass".to_string()
    } else {
        choice.to_string()
    };
    send_message(connection, &GameMessage::PlayerChoice { choice })
}

fn fold(error: String, connection: &mut StreamOwned<ClientConnection, TcpStream>) -> Result<()> {
    print_player_cards(true);
    let player_cards_len: usize = PLAYER_CARDS.read().unwrap().len();
    let choice: usize = choose(
        "Choose a card to fold: ".to_string(),
        error,
        player_cards_len,
        false,
    );
    send_message(
        connection,
        &GameMessage::PlayerChoice {
            choice: PLAYER_CARDS.read().unwrap().get(choice).unwrap().code(),
        },
    )
}

fn hokm(error: String, connection: &mut StreamOwned<ClientConnection, TcpStream>) -> Result<()> {
    print_player_cards(false);
    let hokms: &[Hokm] = if *CUR_BET.read().unwrap() == 13 {
        &HOKMS
    } else {
        &TYPES
    };
    println!(
        "{}",
        hokms
            .iter()
            .enumerate()
            .map(|(i, hokm)| format!("{hokm}: {i}"))
            .collect::<Vec<String>>()
            .join(", ")
    );
    let choice: usize = choose("What is your hokm? ".to_string(), error, hokms.len(), false);
    send_message(
        connection,
        &GameMessage::PlayerChoice {
            choice: hokms[choice].code(),
        },
    )
}

fn play_card(
    error: String,
    connection: &mut StreamOwned<ClientConnection, TcpStream>,
) -> Result<()> {
    print_hokm();
    print_player_cards(true);
    print_ground_cards();
    let player_cards_len: usize = PLAYER_CARDS.read().unwrap().len();
    let ground_cards: RwLockReadGuard<Vec<(String, Card)>> = GROUND_CARDS.read().unwrap();
    let mut prompt: String = "Choose a card to play: ".to_string();
    loop {
        let choice: usize = choose(prompt.clone(), error.clone(), player_cards_len, false);
        if !ground_cards.is_empty() {
            let ground_card_type: &Hokm = &ground_cards.get(0).unwrap().1.type_;
            let card_type = &PLAYER_CARDS.read().unwrap()[choice].type_;
            let has_matching_card: bool = PLAYER_CARDS
                .read()
                .unwrap()
                .iter()
                .any(|player_card: &Card| player_card.type_ == *ground_card_type);
            if has_matching_card && *card_type != *ground_card_type {
                if !prompt.contains("You have ") {
                    prompt = format!("You have {}!\n{prompt}", ground_card_type.name());
                }
                continue;
            }
        }
        return send_message(
            connection,
            &GameMessage::PlayerChoice {
                choice: PLAYER_CARDS.read().unwrap().get(choice).unwrap().code(),
            },
        );
    }
}

fn handle_demand(
    demand: DemandMessage,
    error: String,
    connection: &mut StreamOwned<ClientConnection, TcpStream>,
) -> Result<()> {
    match demand {
        DemandMessage::Username => username(error, connection),
        DemandMessage::Team { available_teams } => team_choice(available_teams, error, connection),
        DemandMessage::Bet => bet(error, connection),
        DemandMessage::Fold => fold(error, connection),
        DemandMessage::Hokm => hokm(error, connection),
        DemandMessage::PlayCard => play_card(error, connection),
    }
}

fn set_player_cards(player_cards: Vec<String>) -> Result<()> {
    let mut cards: Vec<Card> = vec![];
    for player_card in player_cards {
        cards.push(Card::try_from(player_card).unwrap());
    }
    set_cards(cards);
    Ok(())
}

fn print_player_cards(indexed: bool) {
    println!("These are your cards:");
    let mut cards: Vec<String> = PLAYER_CARDS
        .read()
        .unwrap()
        .iter()
        .map(|card| card.to_string())
        .collect();
    if indexed {
        cards = cards
            .iter()
            .enumerate()
            .map(|(i, card)| format!("{card}: {i}"))
            .collect();
    }
    println!("{}", cards.join(", "));
}

fn add_ground_cards(player_cards: Vec<String>) -> Result<()> {
    let mut cards: Vec<Card> = vec![];
    for player_card in player_cards {
        cards.push(Card::try_from(player_card).unwrap());
    }
    add_cards(cards);
    Ok(())
}

fn remove_card(card_to_remove: &Card) {
    let mut cards = PLAYER_CARDS.write().unwrap();
    if let Some(pos) = cards.iter().position(|card| card == card_to_remove) {
        cards.remove(pos);
    }
}

fn remove_player_card(card: String) -> Result<()> {
    remove_card(&Card::try_from(card)?);
    Ok(())
}

pub fn run() -> Result<()> {
    init_config()?;
    let mut client_config: ClientConfig = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(rustls::RootCertStore::empty())
        .with_no_client_auth();
    client_config
        .dangerous()
        .set_certificate_verifier(Arc::new(NoVerifier));
    let config: &'static Config = get_config();
    let server_name: rustls::ServerName = config.server.host.as_str().try_into().unwrap();
    let conn: ClientConnection =
        ClientConnection::new(Arc::new(client_config), server_name).unwrap();
    let tcp_stream: TcpStream =
        TcpStream::connect(format!("{}:{}", config.server.host, config.server.port)).unwrap();
    let mut client_socket: StreamOwned<ClientConnection, TcpStream> =
        StreamOwned::new(conn, tcp_stream);
    loop {
        match receive_message(&mut client_socket) {
            Ok(message) => match message {
                GameMessage::Handshake => {
                    handle_handshake(&mut client_socket)?;
                }
                GameMessage::Broadcast { message } => {
                    handle_broadcast(message);
                }
                GameMessage::Demand { demand, error } => {
                    handle_demand(demand, error, &mut client_socket)?;
                }
                GameMessage::Cards { player_cards } => {
                    set_player_cards(player_cards)?;
                    print_player_cards(false);
                }
                GameMessage::AddGroundCards { ground_cards } => {
                    add_ground_cards(ground_cards)?;
                    print_player_cards(false);
                }
                GameMessage::RemoveCard { card } => {
                    remove_player_card(card)?;
                }
                _ => {
                    println!("Received: {:?}", message);
                }
            },
            Err(e) => {
                eprintln!("Connection error: {}", e);
            }
        }
    }
}
