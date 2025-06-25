use serde::{Deserialize, Serialize};

use crate::models::{Card, Hokm};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PlayerChoice {
    Pass,
    Choice(usize),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum GameMessage {
    Handshake,
    HandshakeResponse,
    Broadcast {
        message: BroadcastMessage,
    },
    Username,
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
        player_cards: Vec<Card>,
    },
    AddGroundCards {
        ground_cards: Vec<Card>,
    },
    Bet {
        error: String,
    },
    PlayerChoice {
        index: usize,
        passed: bool,
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
        card: Card,
    },
}

impl GameMessage {
    pub fn message_type(&self) -> String {
        match self {
            GameMessage::Handshake => "Handshake".to_string(),
            GameMessage::HandshakeResponse => "HandshakeResponse".to_string(),
            GameMessage::Broadcast { .. } => "Broadcast".to_string(),
            GameMessage::Username => "Username".to_string(),
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
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum BroadcastMessage {
    GameStarting,
    HandingOutCards,
    ShufflingCards,
    Starter { name: String },
    Hokm { hokm: Hokm },
    Bets { bets: Vec<(String, PlayerChoice)> },
    BetWinner { bet_winner: (String, usize) },
    GroundCards { ground_cards: Vec<(String, Card)> },
    RoundWinner { round_winner: String },
    GameWinner { game_winner: String },
    GameScore { teams_score: Vec<(String, usize)> },
    RoundScore { teams_score: Vec<(String, usize)> },
}
