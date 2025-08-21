use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: crate::prelude::UserId,
    pub username: String,
    pub game_id: crate::prelude::GameId,
    pub game_type: String,
    pub status: UserSessionStatus,
    pub joined_at: std::time::SystemTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UserSessionStatus {
    InQueue,
    InGame,
}

impl UserSessionStatus {
    pub fn as_str(&self) -> &str {
        match self {
            UserSessionStatus::InQueue => "in Queue",
            UserSessionStatus::InGame => "in Game",
        }
    }
}
