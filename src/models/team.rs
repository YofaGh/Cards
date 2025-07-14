use crate::core::{PlayerId, TeamId};

pub struct Team {
    pub id: TeamId,
    pub name: String,
    pub score: usize,
    pub collected_hands: Vec<Vec<crate::models::Card>>,
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

impl std::fmt::Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
