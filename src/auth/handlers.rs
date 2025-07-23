use crate::{database::User, prelude::*};

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
    if !super::verify_password(&password, &user.password_hash)? {
        return Ok(None);
    }
    if user.is_locked {
        return Ok(None);
    }
    user_repository.update_last_login(user.id).await?;
    Ok(Some(user.into()))
}
