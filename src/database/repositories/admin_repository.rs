#![allow(dead_code)]

use sqlx::{postgres::PgQueryResult, Error as SqlxError};

use crate::database::Admin;
use crate::prelude::*;

#[derive(Clone)]
pub struct AdminRepository {
    pool: PgPool,
}

impl AdminRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_admin(
        &self,
        email: &str,
        username: &str,
        password_hash: &str,
    ) -> Result<Admin> {
        let row = sqlx::query!(
            r#"
            INSERT INTO admins (email, username, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, email, password_hash, username, email_verified, is_active, created_at, updated_at, last_login, permissions
            "#,
            email,
            username,
            password_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err: SqlxError| Error::Other(format!("Failed to create admin: {err}")))?;
        Ok(Admin {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            username: row.username,
            email_verified: row.email_verified,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
            last_login: row.last_login,
            permissions: row.permissions,
        })
    }

    pub async fn delete_admin(&self, admin_id: UserId) -> Result<()> {
        let result: PgQueryResult = sqlx::query!(
            "UPDATE admins SET is_active = false, updated_at = NOW() WHERE id = $1 AND is_active = true",
            admin_id
        )
        .execute(&self.pool)
        .await
        .map_err(|err: SqlxError| Error::Other(format!("Failed to remove admin: {err}")))?;
        if result.rows_affected() == 0 {
            return Err(Error::UserIdNotFound(admin_id));
        }
        Ok(())
    }

    pub async fn get_admin_by_id(&self, admin_id: UserId) -> Result<Option<Admin>> {
        let row = sqlx::query!(
            r#"
            SELECT id, email, password_hash, username, email_verified, is_active, created_at, updated_at, last_login, permissions
            FROM admins
            WHERE id = $1 AND is_active = true
            "#,
            admin_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|err: SqlxError| Error::Other(format!("Failed to get admin by id: {err}")))?;
        Ok(row.map(|row| Admin {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            username: row.username,
            email_verified: row.email_verified,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
            last_login: row.last_login,
            permissions: row.permissions,
        }))
    }

    pub async fn get_admin_by_username(&self, username: String) -> Result<Option<Admin>> {
        let row = sqlx::query!(
            r#"
            SELECT id, email, password_hash, username, email_verified, is_active, created_at, updated_at, last_login, permissions
            FROM admins
            WHERE username = $1 AND is_active = true
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|err: SqlxError| Error::Other(format!("Failed to get admin by id: {err}")))?;
        Ok(row.map(|row| Admin {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            username: row.username,
            email_verified: row.email_verified,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
            last_login: row.last_login,
            permissions: row.permissions,
        }))
    }

    pub async fn email_exists(&self, email: &str) -> Result<bool> {
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM admins WHERE email = $1",
            email
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err: SqlxError| Error::Other(format!("Failed to check email existence: {err}")))?;
        Ok(count.count.unwrap_or(0) > 0)
    }

    pub async fn username_exists(&self, username: &str) -> Result<bool> {
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM admins WHERE username = $1",
            username
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err: SqlxError| {
            Error::Other(format!("Failed to check username existence: {err}"))
        })?;
        Ok(count.count.unwrap_or(0) > 0)
    }

    pub async fn update_password(&self, admin_id: UserId, password_hash: &str) -> Result<()> {
        let result: PgQueryResult = sqlx::query!(
            "UPDATE admins SET password_hash = $1, updated_at = NOW() WHERE id = $2 AND is_active = true",
            password_hash,
            admin_id
        )
        .execute(&self.pool)
        .await
        .map_err(|err: SqlxError| Error::Other(format!("Failed to update password: {err}")))?;
        if result.rows_affected() == 0 {
            return Err(Error::UserIdNotFound(admin_id));
        }
        Ok(())
    }

    pub async fn verify_email(&self, admin_id: UserId) -> Result<()> {
        let result: PgQueryResult = sqlx::query!(
            "UPDATE admins SET email_verified = true, updated_at = NOW() WHERE id = $1 AND is_active = true",
            admin_id
        )
        .execute(&self.pool)
        .await
        .map_err(|err: SqlxError| Error::Other(format!("Failed to verify email: {err}")))?;
        if result.rows_affected() == 0 {
            return Err(Error::UserIdNotFound(admin_id));
        }
        Ok(())
    }

    pub async fn update_last_login(&self, admin_id: UserId) -> Result<()> {
        let result: PgQueryResult = sqlx::query!(
            "UPDATE admins SET last_login = NOW(), updated_at = NOW() WHERE id = $1 AND is_active = true",
            admin_id
        )
        .execute(&self.pool)
        .await
        .map_err(|err: SqlxError| Error::Other(format!("Failed to update last login: {err}")))?;
        if result.rows_affected() == 0 {
            return Err(Error::UserIdNotFound(admin_id));
        }
        Ok(())
    }
}
