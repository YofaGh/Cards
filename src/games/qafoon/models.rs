use crate::{
    models::*,
    prelude::{BTreeMap, PlayerId, TeamId},
};

#[derive(Default)]
pub struct Qafoon {
    pub teams: BTreeMap<TeamId, Team>,
    pub players: BTreeMap<PlayerId, Player>,
    pub field: Vec<PlayerId>,
    pub cards: Vec<Card>,
    pub starter: PlayerId,
    pub hokm: Hokm,
    pub ground: Ground,
    pub status: GameStatus,
}
