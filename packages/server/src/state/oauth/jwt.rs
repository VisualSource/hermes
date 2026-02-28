use std::ops::Add;

use jsonwebtoken::jws;
use thiserror::Error;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Claims {
    iss: String,
    // In OAuth 2.0, an identity provider (IdP) issues tokens with the aud claim set to the client ID.
    aud: String,
    iat: i64,
    exp: i64,
    sub: uuid::Uuid,
}

#[derive(Debug, Error)]
pub enum JwtError {
    #[error(transparent)]
    Var(#[from] std::env::VarError),
    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),
}

pub fn create_jwt(user_id: uuid::Uuid, client_id: String) -> Result<jws::Jws<Claims>, JwtError> {
    let now = time::UtcDateTime::now();
    let exp = now.add(time::Duration::days(1)).unix_timestamp();
    let iss = std::env::var("SERVER_ORIGIN")?;
    let secret = std::env::var("JWT_SECRET_KEY")?;
    let key = jsonwebtoken::EncodingKey::from_secret(secret.as_bytes());

    let claims = Claims {
        iss: iss.to_string(),
        aud: client_id,
        exp: exp,
        iat: now.unix_timestamp(),
        sub: user_id,
    };

    let mut header = jsonwebtoken::Header::default();
    header.alg = jsonwebtoken::Algorithm::HS512;

    let token = jsonwebtoken::jws::encode::<Claims>(&header, Some(&claims), &key)?;

    Ok(token)
}
