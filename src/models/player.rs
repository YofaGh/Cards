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
