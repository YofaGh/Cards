use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum Hokm {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
    Naras,
    Saras,
    TakNaras,
    #[default]
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

impl std::fmt::Display for Hokm {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.name(), self.unicode_char())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PlayerChoice {
    Pass,
    NumberChoice(usize),
    CardChoice(crate::models::Card),
    HokmChoice(Hokm),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum GameStatus {
    #[default]
    NotStarted,
    Started,
    Finished,
    Ended,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum GameMessage {
    Handshake,
    HandshakeResponse,
    Broadcast {
        message: BroadcastMessage,
    },
    Demand {
        demand: DemandMessage,
        error: String,
    },
    Cards {
        player_cards: Vec<String>,
    },
    PlayerRequest {
        request: PlayerRequest,
    },
    PlayerResponse {
        response: PlayerResponse,
    },
    AddGroundCards {
        ground_cards: Vec<String>,
    },
    GameSessionToken {
        token: String,
    },
    PlayerChoice {
        choice: String,
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
            GameMessage::Demand { demand, .. } => demand.message_type(),
            GameMessage::Cards { .. } => "Cards".to_string(),
            GameMessage::AddGroundCards { .. } => "AddGroundCards".to_string(),
            GameMessage::GameSessionToken { .. } => "GameSessionToken".to_string(),
            GameMessage::PlayerChoice { .. } => "PlayerChoice".to_string(),
            GameMessage::RemoveCard { .. } => "RemoveCard".to_string(),
            GameMessage::PlayerRequest { .. } => "PlayerRequest".to_string(),
            GameMessage::PlayerResponse { .. } => "PlayerResponse".to_string(),
        }
    }
    pub fn set_demand_error(&mut self, new_error: String) {
        let GameMessage::Demand { error, .. } = self else {
            panic!("set_demand_error called on non-Demand message");
        };
        *error = new_error;
    }
    pub fn team(available_teams: Vec<String>, error: String) -> Self {
        GameMessage::Demand {
            demand: DemandMessage::Team { available_teams },
            error,
        }
    }
    pub fn demand(demand: DemandMessage) -> Self {
        GameMessage::Demand {
            demand,
            error: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum DemandMessage {
    GameSessionToken,
    Team { available_teams: Vec<String> },
    Bet,
    Fold,
    Hokm,
    PlayCard,
}

impl DemandMessage {
    pub fn message_type(&self) -> String {
        match self {
            DemandMessage::GameSessionToken => "GameSessionToken".to_string(),
            DemandMessage::Team { .. } => "Team".to_string(),
            DemandMessage::Bet => "Bet".to_string(),
            DemandMessage::Fold => "Fold".to_string(),
            DemandMessage::Hokm => "Hokm".to_string(),
            DemandMessage::PlayCard => "PlayCard".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum BroadcastMessage {
    GameStarting,
    GameError,
    GameTimeout,
    ServerShutdown,
    QueueTimeout,
    TeamSelectionStarting,
    EmptyGround,
    GameCancelled { reason: String },
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PlayerRequest {
    GameScore,
    RoundScore,
    CurrentHokm,
    GroundCards,
    GameStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PlayerResponse {
    GameScore { teams_score: Vec<(String, usize)> },
    RoundScore { teams_score: Vec<(String, usize)> },
    CurrentHokm { hokm: String },
    GroundCards { ground_cards: Vec<(String, String)> },
    GameStatus { game_status: GameStatus },
}
