use crate::{models::Card, network::protocol::*, prelude::*};

pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub team_id: TeamId,
    pub hand: Vec<Card>,
    pub connection: Stream,
}

impl Player {
    pub fn new(name: String, team_id: TeamId, connection: Stream) -> Self {
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
