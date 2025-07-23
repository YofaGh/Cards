use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::core::UserId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub password_hash: String,
    pub username: String,
    pub email_verified: bool,
    pub is_active: bool,
    pub is_locked: bool,
    pub is_admin: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub last_login: Option<NaiveDateTime>,
    pub games_played: i32,
    pub games_won: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: UserId,
    pub email: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        UserInfo {
            id: user.id,
            email: user.email,
            username: user.username,
            created_at: DateTime::from_naive_utc_and_offset(user.created_at, Utc),
            last_login: user
                .last_login
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        }
    }
}
