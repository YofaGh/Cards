use tokio::{sync::oneshot::Sender, task::JoinHandle};

use crate::{models::Card, prelude::*};

pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub team_id: TeamId,
    pub cards: Vec<Card>,
}

impl Player {
    pub fn new(name: String, id: PlayerId) -> Self {
        Self {
            id,
            name,
            team_id: TeamId::nil(),
            cards: Vec::new(),
        }
    }

    pub fn set_cards(&mut self, cards: Vec<Card>) {
        self.cards = cards;
    }

    pub fn add_cards(&mut self, mut cards: Vec<Card>) {
        self.cards.append(&mut cards);
    }

    pub fn remove_card(&mut self, card: &Card) -> Result<Card> {
        if let Some(pos) = self.cards.iter().position(|c: &Card| c == card) {
            Ok(self.cards.remove(pos))
        } else {
            Err(Error::NoValidCard)
        }
    }
}

pub struct PlayerConnection {
    pub reader_handle: JoinHandle<tokio::io::ReadHalf<Stream>>,
    pub writer_handle: JoinHandle<tokio::io::WriteHalf<Stream>>,
    pub reader_shutdown_tx: Sender<()>,
    pub writer_shutdown_tx: Sender<()>,
}
