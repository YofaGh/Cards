use chrono::{DateTime, Utc};
use jsonwebtoken::errors::Error as JsonWebTokenError;

use super::{Claims, TokenPair};
use crate::config::get_config;

pub fn generate_token(
    user_id: String,
    username: String,
    is_admin: bool,
) -> Result<TokenPair, JsonWebTokenError> {
    let config: &'static crate::prelude::Config = get_config();
    let now: DateTime<Utc> = Utc::now();
    let expire_time: chrono::TimeDelta = chrono::Duration::seconds(60);
    let expires_at: DateTime<Utc> = now + expire_time;
    let claims: Claims = Claims {
        sub: user_id,
        username,
        is_admin,
        exp: expires_at.timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    let access_token: String = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(config.jwt.secret.as_bytes()),
    )?;
    Ok(TokenPair {
        access_token,
        expires_in: expire_time.num_seconds(),
    })
}

pub fn validate_token(token: &str) -> Result<Claims, JsonWebTokenError> {
    let token_data: jsonwebtoken::TokenData<Claims> = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(get_config().jwt.secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )?;
    Ok(token_data.claims)
}
