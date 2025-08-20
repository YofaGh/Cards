use std::sync::Arc;

use crate::{
    core::GameId,
    models::*,
    prelude::{BTreeMap, PlayerId, Receiver, Sender, Stream, TeamId},
};

#[derive(Clone, Debug, Default)]
pub struct GameSharedState {
    pub game_score: Vec<(String, usize)>,
    pub round_score: Vec<(String, usize)>,
    pub current_hokm: Hokm,
    pub ground_cards: Vec<(String, String)>,
    pub game_status: GameStatus,
}

#[derive(Default)]
pub struct Qafoon {
    pub id: GameId,
    pub teams: BTreeMap<TeamId, Team>,
    pub players: BTreeMap<PlayerId, Player>,
    pub players_receiver: BTreeMap<PlayerId, Receiver<GameMessage>>,
    pub players_sender: BTreeMap<PlayerId, Sender<CorrelatedMessage>>,
    pub player_connections: BTreeMap<PlayerId, PlayerConnection>,
    pub players_reconnection_receiver: Option<Receiver<(PlayerId, Stream)>>,
    pub shared_state: Arc<tokio::sync::RwLock<GameSharedState>>,
    pub field: Vec<PlayerId>,
    pub cards: Vec<Card>,
    pub starter: PlayerId,
    pub hokm: Hokm,
    pub ground: Ground,
    pub status: GameStatus,
}
