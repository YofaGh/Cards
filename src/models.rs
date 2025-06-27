use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt::{Display, Formatter, Result as FmtResult},
};

use crate::{constants::NUMBERS, enums::Hokm, prelude::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub type_: Hokm,
    pub number: String,
    pub ord: usize,
}

impl Card {
    pub fn new(type_: Hokm, number: String, ord: usize) -> Self {
        Card { type_, number, ord }
    }
    pub fn code(&self) -> String {
        format!("{}-{}", self.type_.code(), self.number)
    }
}

impl TryFrom<String> for Card {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        if let Some((hokm_code, card_number)) = value.split_once("-") {
            if let Some(ord) = NUMBERS.iter().position(|&x| x == card_number) {
                return Ok(Card::new(
                    Hokm::from(hokm_code.to_string()),
                    card_number.to_string(),
                    ord,
                ));
            }
        }
        Err(Error::NoValidCard)
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{} {}", self.type_.unicode_char(), self.number)
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
    pub connection: TcpStream,
}

impl Player {
    pub fn new(name: String, team_id: TeamId, connection: TcpStream) -> Self {
        Player {
            id: PlayerId::new_v4(),
            name,
            team_id,
            hand: Vec::new(),
            connection,
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

    pub fn remove_card(&mut self, card: &Card) -> Result<Card> {
        if let Some(pos) = self.hand.iter().position(|c: &Card| c == card) {
            Ok(self.hand.remove(pos))
        } else {
            Err(Error::NoValidCard)
        }
    }

    fn sort_cards(&mut self) -> Result<()> {
        self.hand.sort_by(|a: &Card, b: &Card| {
            if a.type_.name() == b.type_.name() {
                a.ord.cmp(&b.ord)
            } else {
                a.type_.name().cmp(b.type_.name())
            }
        });
        Ok(())
    }

    pub async fn send_message(&mut self, message: &GameMessage) -> Result<()> {
        send_message(&mut self.connection, message).await
    }

    pub async fn receive_message(&mut self) -> Result<GameMessage> {
        receive_message(&mut self.connection).await
    }

    pub async fn close_connection(&mut self) -> Result<()> {
        close_connection(&mut self.connection).await
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
