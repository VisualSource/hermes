use std::ops::Add;

use jsonwebtoken::{Algorithm, TokenData, Validation};
use thiserror::Error;

use crate::state::oauth::OAUTH_CLIENT_ID;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Claims {
    iss: String,
    // In OAuth 2.0, an identity provider (IdP) issues tokens with the aud claim set to the client ID.
    aud: uuid::Uuid,
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

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RefreshClaims {
    pub iss: String,
    pub aud: uuid::Uuid, // client_id
    pub sub: uuid::Uuid, // user_id

    pub exp: i64,
    pub nbf: i64,
    pub iat: i64,
    pub jti: uuid::Uuid,
}

pub fn create_refresh_jwt(
    client_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> Result<String, JwtError> {
    let jti = uuid::Uuid::now_v7();

    let iss = std::env::var("SERVER_ORIGIN")?;
    let secret = std::env::var("JWT_SECRET_KEY")?;
    let key = jsonwebtoken::EncodingKey::from_secret(secret.as_bytes());

    let now = time::UtcDateTime::now();
    let exp = now.add(time::Duration::days(10)).unix_timestamp();

    let claims = RefreshClaims {
        iss,
        aud: client_id,
        sub: user_id,
        exp,
        nbf: 0,
        iat: now.unix_timestamp(),
        jti,
    };

    let mut header = jsonwebtoken::Header::default();
    header.alg = jsonwebtoken::Algorithm::HS512;

    let token = jsonwebtoken::encode::<RefreshClaims>(&header, &claims, &key)?;

    Ok(token)
}

pub fn create_jwt(
    user_id: uuid::Uuid,
    client_id: uuid::Uuid,
) -> Result<String, JwtError> {
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

    let token = jsonwebtoken::encode::<Claims>(&header, &claims, &key)?;

    Ok(token)
}

pub fn validate_refresh_token(token: &str) -> Result<TokenData<RefreshClaims>, JwtError> {
    let secret = std::env::var("JWT_SECRET_KEY")?;
    let key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());
    let iss = std::env::var("SERVER_ORIGIN")?;
    let mut validater = Validation::new(Algorithm::HS512);
    validater.set_issuer(&vec![iss]);
    validater.set_audience(&vec![OAUTH_CLIENT_ID]);

    let jwt = jsonwebtoken::decode::<RefreshClaims>(token, &key, &validater)?;

    Ok(jwt)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_create_jwt(){
        unsafe {
            std::env::set_var("JWT_SECRET_KEY", "TEST_KEY");
            std::env::set_var("SERVER_ORIGIN", "http://localhost:5000");
        }

        let user_id = uuid::uuid!("00000000-0000-0000-1000-000000000000");

        let jwt = super::create_jwt(user_id,super::OAUTH_CLIENT_ID).expect("Failed to create jwt");

        println!("{}",jwt);
    }
    #[test]
    fn test_create_refresh_jwt(){}
    #[test]
    fn test_validate_refresh_jwt(){}
    #[test]
    fn test_validate_jwt(){}
}
