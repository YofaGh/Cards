use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::{models::Card, prelude::*};

pub struct Team {
    pub id: TeamId,
    pub name: String,
    pub score: usize,
    pub collected_hands: Vec<Vec<Card>>,
    pub players: Vec<PlayerId>,
}

impl Team {
    pub fn new(name: String) -> Self {
        Team {
            id: TeamId::new_v4(),
            name,
            score: 0,
            collected_hands: Vec::new(),
            players: Vec::new(),
        }
    }
}

impl Display for Team {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.name)
    }
}
