use itertools::Itertools;
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt::{Display, Formatter, Result as FmtResult},
    net::Shutdown,
    sync::{Mutex, MutexGuard},
};

use crate::prelude::*;

#[derive(Clone, PartialEq)]
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

impl Display for Hokm {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{} {}", self.name, self.unicode_char)
    }
}

#[derive(Clone)]
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

impl Display for Card {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{} {}", self.type_.unicode_char, self.number)
    }
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_ && self.ord == other.ord
    }
}

impl Eq for Card {}

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
            id: TeamId::new_v4(),
            name,
            score: 0,
            collected_hands: Vec::new(),
            players: Vec::new(),
        }
    }
}

impl Display for Team {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.name)
    }
}

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
            id: PlayerId::new_v4(),
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

    fn sort_cards(&mut self) -> Result<()> {
        self.hand.sort_by(|a: &Card, b: &Card| {
            if a.type_.name == b.type_.name {
                a.ord.cmp(&b.ord)
            } else {
                a.type_.name.cmp(b.type_.name)
            }
        });
        Ok(())
    }

    pub fn get_hand(&self) -> String {
        self.hand
            .iter()
            .enumerate()
            .map(|(index, card)| format!("{}:{}", card, index))
            .join(", ")
    }

    pub fn send_message(&self, message: &str, msg_type: u8) -> Result<()> {
        let conn: MutexGuard<TcpStream> = self.connection.lock().map_err(Error::lock_connection)?;
        send_message(&conn, &format!("{}$_$_${}", msg_type, message))
    }

    pub fn receive_message(&self) -> Result<String> {
        let conn: MutexGuard<TcpStream> = self.connection.lock().map_err(Error::lock_connection)?;
        receive_message(&conn)
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        if let Ok(conn) = self.connection.lock() {
            let _ = conn.shutdown(Shutdown::Both);
        }
    }
}

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
