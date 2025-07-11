use crate::{models::Card, prelude::*};

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

impl Default for Ground {
    fn default() -> Self {
        Ground::new()
    }
}
