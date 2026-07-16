use std::ops::Add;

use jsonwebtoken::{Algorithm, TokenData, Validation};
use thiserror::Error;

use crate::state::oauth::OAUTH_CLIENT_ID;

/// JWT profile for OAuth 2.0 access tokens (RFC 9068 §2.2). The `aud` claim
/// carries the resource server identifier; the OAuth client is carried
/// separately in `client_id`.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Claims {
    pub iss: String,
    pub aud: String,
    pub sub: uuid::Uuid,
    pub client_id: uuid::Uuid,
    pub jti: uuid::Uuid,
    pub iat: i64,
    pub exp: i64,
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
    pub iat: i64,
    pub jti: uuid::Uuid,
}

fn secret_key() -> Result<jsonwebtoken::EncodingKey, JwtError> {
    let secret = std::env::var("JWT_SECRET_KEY")?;
    Ok(jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()))
}

fn decoding_key() -> Result<jsonwebtoken::DecodingKey, JwtError> {
    let secret = std::env::var("JWT_SECRET_KEY")?;
    Ok(jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()))
}

/// Resource-server audience for access tokens (RFC 9068 §2.2).
fn resource_audience() -> Result<String, JwtError> {
    Ok(std::env::var("SERVER_ORIGIN")?)
}

pub fn create_refresh_jwt(
    client_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> Result<(String, uuid::Uuid, time::UtcDateTime), JwtError> {
    let jti = uuid::Uuid::now_v7();

    let iss = std::env::var("SERVER_ORIGIN")?;
    let key = secret_key()?;

    let now = time::UtcDateTime::now();
    let exp = now.add(time::Duration::days(10));

    let claims = RefreshClaims {
        iss,
        aud: client_id,
        sub: user_id,
        exp: exp.unix_timestamp(),
        iat: now.unix_timestamp(),
        jti,
    };

    let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS512);
    header.typ = Some("rt+jwt".to_string());

    let token = jsonwebtoken::encode::<RefreshClaims>(&header, &claims, &key)?;

    Ok((token, jti, exp))
}

pub fn create_jwt(user_id: uuid::Uuid, client_id: uuid::Uuid) -> Result<String, JwtError> {
    let now = time::UtcDateTime::now();
    let exp = now.add(time::Duration::days(1)).unix_timestamp();
    let iss = std::env::var("SERVER_ORIGIN")?;
    let aud = resource_audience()?;
    let key = secret_key()?;

    let claims = Claims {
        iss,
        aud,
        sub: user_id,
        client_id,
        jti: uuid::Uuid::now_v7(),
        iat: now.unix_timestamp(),
        exp,
    };

    // RFC 9068 §2.1: access tokens SHOULD use `typ: at+jwt` so they can't be
    // confused with ID tokens or unrelated JWTs.
    let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS512);
    header.typ = Some("at+jwt".to_string());

    let token = jsonwebtoken::encode::<Claims>(&header, &claims, &key)?;

    Ok(token)
}

pub fn validate_jwt(token: &str) -> Result<TokenData<Claims>, JwtError> {
    let iss = std::env::var("SERVER_ORIGIN")?;
    let aud = resource_audience()?;
    let key = decoding_key()?;

    let mut validater = Validation::new(Algorithm::HS512);
    validater.set_issuer(&[iss]);
    validater.set_audience(&[aud]);
    validater.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);

    let jwt = jsonwebtoken::decode::<Claims>(token, &key, &validater)?;

    Ok(jwt)
}

pub fn validate_refresh_token(token: &str) -> Result<TokenData<RefreshClaims>, JwtError> {
    let iss = std::env::var("SERVER_ORIGIN")?;
    let key = decoding_key()?;

    let mut validater = Validation::new(Algorithm::HS512);
    validater.set_issuer(&[iss]);
    validater.set_audience(&[OAUTH_CLIENT_ID]);
    validater.set_required_spec_claims(&["exp", "iss", "aud"]);

    let jwt = jsonwebtoken::decode::<RefreshClaims>(token, &key, &validater)?;

    Ok(jwt)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_create_jwt() {
        unsafe {
            std::env::set_var(
                "JWT_SECRET_KEY",
                "FaBLMQItyEDeDMm9SFMms10p3DH93eO31gp9su1IdpAmhnjwFA7ljyCM8dFdpoZUBhXweos97wRDwFeQKfjQZOvjy7bdHAcJjCpORa4FZy94pqRPJw1IUofY656BZSqc9WikB1qMNoVFOGXFkgk8J6K1Vn2YKRRRg8vdWQp02H0Mdg0SqFcpLe8nhtUv8TbSr2f4rLpn213RLN3hfnYygcyXeeKwKyWw2gJpqgVvcdzEfEVznwdJnHfak7MZCqO2",
            );
            std::env::set_var("SERVER_ORIGIN", "http://localhost:5000");
        }

        let user_id = uuid::uuid!("00000000-0000-0000-1000-000000000000");

        let jwt = super::create_jwt(user_id, super::OAUTH_CLIENT_ID).expect("Failed to create jwt");

        let decoded = super::validate_jwt(&jwt).expect("failed to validate jwt");
        assert_eq!(decoded.claims.sub, user_id);
        assert_eq!(decoded.claims.client_id, super::OAUTH_CLIENT_ID);
        assert_eq!(decoded.claims.aud, "http://localhost:5000");
        assert_eq!(decoded.header.typ.as_deref(), Some("at+jwt"));
    }
    #[test]
    fn test_create_refresh_jwt() {
        unsafe {
            std::env::set_var(
                "JWT_SECRET_KEY",
                "FaBLMQItyEDeDMm9SFMms10p3DH93eO31gp9su1IdpAmhnjwFA7ljyCM8dFdpoZUBhXweos97wRDwFeQKfjQZOvjy7bdHAcJjCpORa4FZy94pqRPJw1IUofY656BZSqc9WikB1qMNoVFOGXFkgk8J6K1Vn2YKRRRg8vdWQp02H0Mdg0SqFcpLe8nhtUv8TbSr2f4rLpn213RLN3hfnYygcyXeeKwKyWw2gJpqgVvcdzEfEVznwdJnHfak7MZCqO2",
            );
            std::env::set_var("SERVER_ORIGIN", "http://localhost:5000");
        }
        let user_id = uuid::uuid!("00000000-0000-0000-1000-000000000000");
        let (jwt, _, _) =
            super::create_refresh_jwt(super::OAUTH_CLIENT_ID, user_id).expect("failed to make jwt");
        println!("{}", jwt);
    }
    #[test]
    fn test_validate_refresh_jwt() {
        unsafe {
            std::env::set_var(
                "JWT_SECRET_KEY",
                "FaBLMQItyEDeDMm9SFMms10p3DH93eO31gp9su1IdpAmhnjwFA7ljyCM8dFdpoZUBhXweos97wRDwFeQKfjQZOvjy7bdHAcJjCpORa4FZy94pqRPJw1IUofY656BZSqc9WikB1qMNoVFOGXFkgk8J6K1Vn2YKRRRg8vdWQp02H0Mdg0SqFcpLe8nhtUv8TbSr2f4rLpn213RLN3hfnYygcyXeeKwKyWw2gJpqgVvcdzEfEVznwdJnHfak7MZCqO2",
            );
            std::env::set_var("SERVER_ORIGIN", "http://localhost:5000");
        }
        let user_id = uuid::uuid!("00000000-0000-0000-1000-000000000000");
        let (jwt, _, _) =
            super::create_refresh_jwt(super::OAUTH_CLIENT_ID, user_id).expect("failed to make jwt");

        let data = super::validate_refresh_token(&jwt).expect("failed to validate token");

        assert_eq!(data.claims.sub, user_id);
    }
}
