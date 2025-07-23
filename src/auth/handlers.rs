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
