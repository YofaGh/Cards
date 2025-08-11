use tokio::{sync::oneshot::Sender, task::JoinHandle};

use crate::{models::Card, prelude::*};

pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub team_id: TeamId,
    pub hand: Vec<Card>,
}

impl Player {
    pub fn new(name: String) -> Self {
        Player {
            id: PlayerId::new_v4(),
            name,
            team_id: TeamId::nil(),
            hand: Vec::new(),
        }
    }

    pub fn set_cards(&mut self, cards: Vec<Card>) -> Result<()> {
        self.hand = cards;
        Ok(())
    }

    pub fn add_cards(&mut self, mut cards: Vec<Card>) -> Result<()> {
        self.hand.append(&mut cards);
        Ok(())
    }

    pub fn remove_card(&mut self, card: &Card) -> Result<Card> {
        if let Some(pos) = self.hand.iter().position(|c: &Card| c == card) {
            Ok(self.hand.remove(pos))
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
