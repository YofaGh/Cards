use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::models::Card;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum Hokm {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
    Naras,
    Saras,
    TakNaras,
    Default,
}

impl Hokm {
    pub fn name(&self) -> &'static str {
        match self {
            Hokm::Spades => "Spades",
            Hokm::Hearts => "Hearts",
            Hokm::Diamonds => "Diamonds",
            Hokm::Clubs => "Clubs",
            Hokm::Naras => "Naras",
            Hokm::Saras => "Saras",
            Hokm::TakNaras => "Tak Naras",
            Hokm::Default => "Hokm",
        }
    }

    pub fn unicode_char(&self) -> &'static str {
        match self {
            Hokm::Spades => "\u{2660}",
            Hokm::Hearts => "\u{2665}",
            Hokm::Diamonds => "\u{2666}",
            Hokm::Clubs => "\u{2663}",
            Hokm::Naras => "\u{2193}",
            Hokm::Saras => "\u{2191}",
            Hokm::TakNaras => "\u{21a7}",
            Hokm::Default => "",
        }
    }

    pub fn code(&self) -> String {
        match self {
            Hokm::Spades => "S",
            Hokm::Hearts => "H",
            Hokm::Diamonds => "D",
            Hokm::Clubs => "C",
            Hokm::Naras => "N",
            Hokm::Saras => "A",
            Hokm::TakNaras => "T",
            Hokm::Default => "",
        }
        .to_string()
    }
}

impl From<String> for Hokm {
    fn from(value: String) -> Self {
        match value.as_str() {
            "S" => Hokm::Spades,
            "H" => Hokm::Hearts,
            "D" => Hokm::Diamonds,
            "C" => Hokm::Clubs,
            "N" => Hokm::Naras,
            "A" => Hokm::Saras,
            "T" => Hokm::TakNaras,
            _ => Hokm::Default,
        }
    }
}

impl Default for Hokm {
    fn default() -> Self {
        Hokm::Default
    }
}

impl Display for Hokm {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{} {}", self.name(), self.unicode_char())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PlayerChoice {
    Pass,
    NumberChoice(usize),
    CardChoice(Card),
    HokmChoice(Hokm),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum GameMessage {
    Handshake,
    HandshakeResponse,
    Broadcast {
        message: BroadcastMessage,
    },
    Username {
        error: String,
    },
    UsernameResponse {
        username: String,
    },
    TeamChoice {
        available_teams: Vec<String>,
        error: String,
    },
    TeamChoiceResponse {
        team_index: usize,
    },
    Cards {
        player_cards: Vec<String>,
    },
    AddGroundCards {
        ground_cards: Vec<String>,
    },
    Bet {
        error: String,
    },
    PlayerChoice {
        choice: String,
    },
    Fold {
        error: String,
    },
    Hokm {
        error: String,
    },
    PlayCard {
        error: String,
    },
    RemoveCard {
        card: String,
    },
}

impl GameMessage {
    pub fn message_type(&self) -> String {
        match self {
            GameMessage::Handshake => "Handshake".to_string(),
            GameMessage::HandshakeResponse => "HandshakeResponse".to_string(),
            GameMessage::Broadcast { .. } => "Broadcast".to_string(),
            GameMessage::Username { .. } => "Username".to_string(),
            GameMessage::UsernameResponse { .. } => "UsernameResponse".to_string(),
            GameMessage::TeamChoice { .. } => "TeamChoice".to_string(),
            GameMessage::TeamChoiceResponse { .. } => "TeamChoice".to_string(),
            GameMessage::Cards { .. } => "Cards".to_string(),
            GameMessage::AddGroundCards { .. } => "AddGroundCards".to_string(),
            GameMessage::Bet { .. } => "Bet".to_string(),
            GameMessage::PlayerChoice { .. } => "PlayerChoice".to_string(),
            GameMessage::Fold { .. } => "Fold".to_string(),
            GameMessage::Hokm { .. } => "Hokm".to_string(),
            GameMessage::PlayCard { .. } => "PlayCard".to_string(),
            GameMessage::RemoveCard { .. } => "RemoveCard".to_string(),
        }
    }
    pub fn set_error(&mut self, new_error: String) {
        match self {
            GameMessage::Username { error } => {
                *error = new_error;
            }
            GameMessage::TeamChoice { error, .. } => {
                *error = new_error;
            }
            GameMessage::Bet { error } => {
                *error = new_error;
            }
            GameMessage::Fold { error } => {
                *error = new_error;
            }
            GameMessage::Hokm { error } => {
                *error = new_error;
            }
            GameMessage::PlayCard { error } => {
                *error = new_error;
            }
            _ => {}
        }
    }
    pub fn bet() -> Self {
        GameMessage::Bet {
            error: String::new(),
        }
    }
    pub fn fold() -> Self {
        GameMessage::Fold {
            error: String::new(),
        }
    }
    pub fn username() -> Self {
        GameMessage::Username {
            error: String::new(),
        }
    }
    pub fn hokm() -> Self {
        GameMessage::Hokm {
            error: String::new(),
        }
    }
    pub fn play_card() -> Self {
        GameMessage::PlayCard {
            error: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum BroadcastMessage {
    GameStarting,
    HandingOutCards,
    ShufflingCards,
    Starter { name: String },
    Hokm { hokm: String },
    Bets { bets: Vec<(String, PlayerChoice)> },
    BetWinner { bet_winner: (String, usize) },
    GroundCards { ground_cards: Vec<(String, String)> },
    RoundWinner { round_winner: String },
    GameWinner { game_winner: String },
    GameScore { teams_score: Vec<(String, usize)> },
    RoundScore { teams_score: Vec<(String, usize)> },
}
