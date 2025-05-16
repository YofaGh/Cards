use itertools::Itertools;
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt,
    hash::{Hash, Hasher},
    io::{Read, Write},
    net::TcpStream,
    sync::{Mutex, MutexGuard},
};
use uuid::Uuid;

use crate::{errors::Error, types::*};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hokm {
    pub name: &'static str,
    pub unicode_char: &'static str,
}

impl Default for Hokm {
    fn default() -> Self {
        Hokm {
            name: "Hokm",
            unicode_char: "",
        }
    }
}

impl fmt::Display for Hokm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.name, self.unicode_char)
    }
}

#[derive(Debug, Clone)]
pub struct Card {
    pub type_: Hokm,
    pub number: String,
    pub ord: usize,
}

impl Card {
    pub fn new(type_: Hokm, number: String, ord: usize) -> Self {
        Card { type_, number, ord }
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.type_.unicode_char, self.number)
    }
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_ && self.ord == other.ord
    }
}

impl Eq for Card {}

impl Hash for Card {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_.hash(state);
        self.ord.hash(state);
    }
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.ord.cmp(&other.ord))
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ord.cmp(&other.ord)
    }
}

#[derive(Debug)]
pub struct Team {
    pub id: TeamId,
    pub name: String,
    pub score: usize,
    pub collected_hands: Vec<Vec<Card>>,
    pub players: Vec<PlayerId>,
}

impl Team {
    pub fn new(name: String) -> Self {
        Team {
            id: Uuid::new_v4(),
            name,
            score: 0,
            collected_hands: Vec::new(),
            players: Vec::new(),
        }
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub team_id: TeamId,
    pub hand: Vec<Card>,
    pub connection: Mutex<TcpStream>,
}

impl Player {
    pub fn new(name: String, team_id: TeamId, connection: TcpStream) -> Self {
        Player {
            id: Uuid::new_v4(),
            name,
            team_id,
            hand: Vec::new(),
            connection: Mutex::new(connection),
        }
    }

    pub fn set_cards(&mut self, cards: Vec<Card>) -> Result<()> {
        self.hand = cards;
        self.sort_cards()
    }

    pub fn add_cards(&mut self, mut cards: Vec<Card>) -> Result<()> {
        self.hand.append(&mut cards);
        self.sort_cards()
    }

    pub fn sort_cards(&mut self) -> Result<()> {
        self.hand.sort_by(|a: &Card, b: &Card| {
            if a.type_.name == b.type_.name {
                a.ord.cmp(&b.ord)
            } else {
                a.type_.name.cmp(&b.type_.name)
            }
        });
        Ok(())
    }

    pub fn get_hand(&self) -> String {
        self.hand
            .iter()
            .enumerate()
            .map(|(index, card)| format!("{}:{}", card.to_string(), index))
            .join(", ")
    }

    pub fn send_message(&self, message: &str, msg_type: u8) -> Result<()> {
        let formatted_msg: String = format!("{}$_$_${}", msg_type, message);
        let msg_bytes: &[u8] = formatted_msg.as_bytes();
        let mut conn: MutexGuard<TcpStream> =
            self.connection.lock().map_err(Error::lock_connection)?;
        conn.write_all(&msg_bytes.len().to_be_bytes())
            .map_err(Error::connection)?;
        conn.write_all(msg_bytes).map_err(Error::connection)?;
        conn.flush().map_err(Error::connection)
    }

    pub fn receive_message(&self) -> Result<String> {
        let mut buf: [u8; 1024] = [0; 1024];
        let mut conn: MutexGuard<TcpStream> =
            self.connection.lock().map_err(Error::lock_connection)?;
        let bytes_read: usize = conn.read(&mut buf).map_err(Error::connection)?;
        Ok(String::from_utf8_lossy(&buf[..bytes_read]).to_string())
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Player {}

impl Hash for Player {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Default)]
pub struct Ground {
    pub cards: Vec<(PlayerId, Card)>,
    pub type_: Hokm,
}

impl Ground {
    pub fn new() -> Self {
        Ground {
            cards: Vec::new(),
            type_: Hokm::default(),
        }
    }

    pub fn add_card(&mut self, player_id: PlayerId, card: Card) -> Result<()> {
        if self.cards.is_empty() {
            self.type_ = card.type_.to_owned();
        }
        self.cards.push((player_id, card));
        Ok(())
    }
}

impl fmt::Display for Ground {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cards: String = self
            .cards
            .iter()
            .map(|(player_id, card)| format!("{}:{}", card, player_id))
            .join(", ");
        write!(f, "{}", cards)
    }
}

pub trait GetOrError<K, V> {
    fn get_or_error(&self, key: &K, error_fn: impl FnOnce() -> Error) -> Result<&V>;
    fn get_mut_or_error(&mut self, key: &K, error_fn: impl FnOnce() -> Error) -> Result<&mut V>;
}

impl<K: Ord, V> GetOrError<K, V> for BTreeMap<K, V> {
    fn get_or_error(&self, key: &K, error_fn: impl FnOnce() -> Error) -> Result<&V> {
        self.get(key).ok_or_else(error_fn)
    }
    fn get_mut_or_error(&mut self, key: &K, error_fn: impl FnOnce() -> Error) -> Result<&mut V> {
        self.get_mut(key).ok_or_else(error_fn)
    }
}
