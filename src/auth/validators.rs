#![allow(dead_code)]

use crate::prelude::*;

pub fn validate_password(password: &str) -> Result<()> {
    if password.len() < 8 {
        return Err(Error::Other(
            "Password must be at least 8 characters long".to_string(),
        ));
    }
    if password.len() > 128 {
        return Err(Error::Other(
            "Password must be less than 128 characters".to_string(),
        ));
    }
    let has_upper: bool = password.chars().any(|c: char| c.is_uppercase());
    let has_lower: bool = password.chars().any(|c: char| c.is_lowercase());
    let has_digit: bool = password.chars().any(|c: char| c.is_numeric());
    if !has_upper || !has_lower || !has_digit {
        return Err(Error::Other(
            "Password must contain at least one uppercase letter, one lowercase letter, and one digit".to_string()
        ));
    }
    Ok(())
}

pub fn validate_email(email: &str) -> Result<()> {
    if email.is_empty() {
        return Err(Error::Other("Email cannot be empty".to_string()));
    }
    if email.len() > 254 {
        return Err(Error::Other(
            "Email must be less than 254 characters".to_string(),
        ));
    }
    if !email.contains('@') {
        return Err(Error::Other("Invalid email format".to_string()));
    }
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(Error::Other("Invalid email format".to_string()));
    }
    Ok(())
}

pub fn validate_username(username: &str) -> Result<()> {
    if username.is_empty() {
        return Err(Error::Other("Username cannot be empty".to_string()));
    }
    if username.len() < 3 {
        return Err(Error::Other(
            "Username must be at least 3 characters long".to_string(),
        ));
    }
    if username.len() > 50 {
        return Err(Error::Other(
            "Username must be less than 50 characters".to_string(),
        ));
    }
    if !username
        .chars()
        .all(|c: char| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(Error::Other(
            "Username can only contain letters, numbers, underscores, and hyphens".to_string(),
        ));
    }

    Ok(())
}
