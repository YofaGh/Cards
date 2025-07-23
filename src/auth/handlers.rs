use super::verify_password;
use crate::{
    database::{Admin, User},
    prelude::*,
};

pub async fn login_user(
    user_repository: &crate::database::UserRepository,
    username: String,
    password: String,
) -> Result<Option<User>> {
    super::validate_username(&username)?;
    let user: crate::database::User = match user_repository.get_user_by_username(username).await? {
        Some(user) => user,
        None => {
            return Ok(None);
        }
    };
    if !verify_password(&password, &user.password_hash)? {
        return Ok(None);
    }
    if user.is_locked {
        return Ok(None);
    }
    user_repository.update_last_login(user.id).await?;
    Ok(Some(user))
}

pub async fn register_user(
    user_repository: &crate::database::UserRepository,
    email: String,
    username: String,
    password: String,
) -> Result<Option<User>> {
    super::validate_email(&email)?;
    super::validate_username(&username)?;
    super::validate_password(&password)?;
    if user_repository.email_exists(&email).await? {
        return Ok(None);
    }
    if user_repository.username_exists(&username).await? {
        return Ok(None);
    }
    let password_hash: String = super::hash_password(&password)?;
    let user: User = user_repository
        .create_user(&email, &username, &password_hash)
        .await?;
    user_repository.update_last_login(user.id).await?;
    Ok(Some(user))
}

pub async fn login_admin(
    admin_repo: &crate::database::AdminRepository,
    username: String,
    password: String,
) -> Result<Option<Admin>> {
    let admin: Admin = match admin_repo.get_admin_by_username(username).await? {
        Some(admin) => admin,
        None => return Ok(None),
    };
    if verify_password(&password, &admin.password_hash)? {
        admin_repo.update_last_login(admin.id).await?;
        Ok(Some(admin))
    } else {
        Ok(None)
    }
}
