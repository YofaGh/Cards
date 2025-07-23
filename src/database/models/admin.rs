use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::core::UserId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Admin {
    pub id: UserId,
    pub email: String,
    pub password_hash: String,
    pub username: String,
    pub email_verified: bool,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub last_login: Option<NaiveDateTime>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminInfo {
    pub id: UserId,
    pub email: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

impl From<Admin> for AdminInfo {
    fn from(user: Admin) -> Self {
        AdminInfo {
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
