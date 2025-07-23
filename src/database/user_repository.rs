#![allow(dead_code)]

use sqlx::{postgres::PgQueryResult, Error as SqlxError};

use super::models::User;
use crate::prelude::*;

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(
        &self,
        email: &str,
        username: &str,
        password_hash: &str,
    ) -> Result<User> {
        let row = sqlx::query!(
            r#"
            INSERT INTO users (email, username, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, email, password_hash, username, email_verified, is_active, is_locked, is_admin, created_at, updated_at, last_login, games_played, games_won
            "#,
            email,
            username,
            password_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e: SqlxError| Error::Other(format!("Failed to create user: {}", e)))?;
        Ok(User {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            username: row.username,
            email_verified: row.email_verified,
            is_active: row.is_active,
            is_locked: row.is_locked,
            is_admin: row.is_admin,
            created_at: row.created_at,
            updated_at: row.updated_at,
            last_login: row.last_login,
            games_played: row.games_played,
            games_won: row.games_won,
        })
    }

    pub async fn delete_user(&self, user_id: UserId) -> Result<()> {
        let result: PgQueryResult = sqlx::query!(
            "UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1 AND is_active = true",
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Other(format!("Failed to remove user: {}", e)))?;
        if result.rows_affected() == 0 {
            return Err(Error::UserIdNotFound(user_id));
        }
        Ok(())
    }

    pub async fn get_user_by_username(&self, username: String) -> Result<Option<User>> {
        let row = sqlx::query!(
            r#"
            SELECT id, email, password_hash, username, email_verified, is_active, is_locked, is_admin, created_at, updated_at, last_login, games_played, games_won
            FROM users
            WHERE username = $1 AND is_active = true
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e: SqlxError| Error::Other(format!("Failed to get user by id: {}", e)))?;
        Ok(row.map(|row| User {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            username: row.username,
            email_verified: row.email_verified,
            is_active: row.is_active,
            is_locked: row.is_locked,
            is_admin: row.is_admin,
            created_at: row.created_at,
            updated_at: row.updated_at,
            last_login: row.last_login,
            games_played: row.games_played,
            games_won: row.games_won,
        }))
    }

    pub async fn get_user_by_id(&self, user_id: UserId) -> Result<Option<User>> {
        let row = sqlx::query!(
            r#"
            SELECT id, email, password_hash, username, email_verified, is_active, is_locked, is_admin, created_at, updated_at, last_login, games_played, games_won
            FROM users
            WHERE id = $1 AND is_active = true
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e: SqlxError| Error::Other(format!("Failed to get user by id: {}", e)))?;
        Ok(row.map(|row| User {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            username: row.username,
            email_verified: row.email_verified,
            is_active: row.is_active,
            is_locked: row.is_locked,
            is_admin: row.is_admin,
            created_at: row.created_at,
            updated_at: row.updated_at,
            last_login: row.last_login,
            games_played: row.games_played,
            games_won: row.games_won,
        }))
    }

    pub async fn email_exists(&self, email: &str) -> Result<bool> {
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM users WHERE email = $1",
            email
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e: SqlxError| Error::Other(format!("Failed to check email existence: {}", e)))?;
        Ok(count.count.unwrap_or(0) > 0)
    }

    pub async fn username_exists(&self, username: &str) -> Result<bool> {
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM users WHERE username = $1",
            username
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e: SqlxError| {
            Error::Other(format!("Failed to check username existence: {}", e))
        })?;
        Ok(count.count.unwrap_or(0) > 0)
    }

    pub async fn update_profile(
        &self,
        user_id: UserId,
        email: &str,
        username: &str,
    ) -> Result<User> {
        let row = sqlx::query!(
            r#"
            UPDATE users 
            SET email = $1, username = $2, updated_at = NOW()
            WHERE id = $3 AND is_active = true AND is_locked = false
            RETURNING id, email, password_hash, username, email_verified, is_active, is_locked, is_admin, created_at, updated_at, last_login, games_played, games_won
            "#,
            email,
            username,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e: SqlxError| {
            if matches!(e, sqlx::Error::RowNotFound) {
                Error::UserIdNotFound(user_id)
            } else {
                Error::Other(format!("Failed to update profile: {}", e))
            }
        })?;
        Ok(User {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            username: row.username,
            email_verified: row.email_verified,
            is_active: row.is_active,
            is_locked: row.is_locked,
            is_admin: row.is_admin,
            created_at: row.created_at,
            updated_at: row.updated_at,
            last_login: row.last_login,
            games_played: row.games_played,
            games_won: row.games_won,
        })
    }

    pub async fn update_password(&self, user_id: UserId, password_hash: &str) -> Result<()> {
        let result: PgQueryResult = sqlx::query!(
            "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2 AND is_active = true AND is_locked = false",
            password_hash,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e: SqlxError| Error::Other(format!("Failed to update password: {}", e)))?;
        if result.rows_affected() == 0 {
            return Err(Error::UserIdNotFound(user_id));
        }
        Ok(())
    }

    pub async fn update_game_stats(
        &self,
        user_id: UserId,
        games_played: i32,
        games_won: i32,
    ) -> Result<()> {
        let result: PgQueryResult = sqlx::query!(
            "UPDATE users SET games_played = $1, games_won = $2, updated_at = NOW() WHERE id = $3 AND is_active = true",
            games_played,
            games_won,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e: SqlxError| Error::Other(format!("Failed to update game stats: {}", e)))?;
        if result.rows_affected() == 0 {
            return Err(Error::UserIdNotFound(user_id));
        }
        Ok(())
    }

    pub async fn verify_email(&self, user_id: UserId) -> Result<()> {
        let result: PgQueryResult = sqlx::query!(
            "UPDATE users SET email_verified = true, updated_at = NOW() WHERE id = $1 AND is_active = true",
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e: SqlxError| Error::Other(format!("Failed to verify email: {}", e)))?;
        if result.rows_affected() == 0 {
            return Err(Error::UserIdNotFound(user_id));
        }
        Ok(())
    }

    pub async fn update_last_login(&self, user_id: UserId) -> Result<()> {
        let result: PgQueryResult = sqlx::query!(
            "UPDATE users SET last_login = NOW(), updated_at = NOW() WHERE id = $1 AND is_active = true",
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e: SqlxError| Error::Other(format!("Failed to update last login: {}", e)))?;
        if result.rows_affected() == 0 {
            return Err(Error::UserIdNotFound(user_id));
        }
        Ok(())
    }

    pub async fn lock_user(&self, user_id: UserId) -> Result<()> {
        let result: PgQueryResult = sqlx::query!(
            "UPDATE users SET is_locked = true, updated_at = NOW() WHERE id = $1 AND is_active = true",
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e: SqlxError| Error::Other(format!("Failed to lock user: {}", e)))?;
        if result.rows_affected() == 0 {
            return Err(Error::UserIdNotFound(user_id));
        }
        Ok(())
    }

    pub async fn unlock_user(&self, user_id: UserId) -> Result<()> {
        let result: PgQueryResult = sqlx::query!(
            "UPDATE users SET is_locked = false, updated_at = NOW() WHERE id = $1 AND is_active = true",
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e: SqlxError| Error::Other(format!("Failed to unlock user: {}", e)))?;
        if result.rows_affected() == 0 {
            return Err(Error::UserIdNotFound(user_id));
        }
        Ok(())
    }
}
