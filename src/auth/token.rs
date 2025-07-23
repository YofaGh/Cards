use chrono::{DateTime, Duration, TimeDelta, Utc};
use jsonwebtoken::errors::Error as JsonWebTokenError;

use super::{Claims, GameSessionClaims, TokenPair};
use crate::prelude::{get_config, Config};

pub fn generate_token(
    user_id: String,
    username: String,
    is_admin: bool,
) -> Result<TokenPair, JsonWebTokenError> {
    let config: &'static Config = get_config();
    let now: DateTime<Utc> = Utc::now();
    let expire_time: TimeDelta = Duration::hours(config.jwt.expire_time.into());
    let expires_at: DateTime<Utc> = now + expire_time;
    let claims: Claims = Claims {
        sub: user_id,
        username,
        is_admin,
        exp: expires_at.timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    let access_token: String = encode_token(&claims)?;
    Ok(TokenPair {
        access_token,
        expires_in: expire_time.num_seconds(),
    })
}

pub fn generate_game_session_token(
    user_id: String,
    username: String,
    game_choice: String,
) -> Result<TokenPair, JsonWebTokenError> {
    let config: &'static Config = get_config();
    let now: DateTime<Utc> = Utc::now();
    let expire_time: TimeDelta = Duration::seconds(config.jwt.expire_time.into());
    let expires_at: DateTime<Utc> = now + expire_time;
    let claims: GameSessionClaims = GameSessionClaims {
        sub: user_id,
        username,
        game_choice,
        exp: expires_at.timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    let access_token: String = encode_token(&claims)?;
    Ok(TokenPair {
        access_token,
        expires_in: expire_time.num_seconds(),
    })
}

fn encode_token<T: serde::Serialize>(claims: &T) -> Result<String, JsonWebTokenError> {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        claims,
        &jsonwebtoken::EncodingKey::from_secret(get_config().jwt.secret.as_bytes()),
    )
}

pub fn validate_token<T: for<'de> serde::Deserialize<'de>>(
    token: &str,
) -> Result<T, JsonWebTokenError> {
    let token_data: jsonwebtoken::TokenData<T> = jsonwebtoken::decode::<T>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(get_config().jwt.secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )?;
    Ok(token_data.claims)
}
