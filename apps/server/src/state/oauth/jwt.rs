use std::ops::Add;
use std::sync::OnceLock;

use base64::Engine;
use ed25519_dalek::pkcs8::spki::der::pem::LineEnding;
use ed25519_dalek::pkcs8::{DecodePrivateKey, EncodePublicKey};
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
    /// RFC 9068 §2.2.2: space-separated scope list. Present only when the
    /// grant carried a non-empty scope.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub scope: Option<String>,
}

#[derive(Debug, Error)]
pub enum JwtError {
    #[error(transparent)]
    Var(#[from] std::env::VarError),
    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("JWT signing keys have not been initialized")]
    KeysUninitialized,
    #[error("failed to load JWT signing key: {0}")]
    KeyLoad(String),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RefreshClaims {
    pub iss: String,
    pub aud: uuid::Uuid, // client_id
    pub sub: uuid::Uuid, // user_id

    pub exp: i64,
    pub iat: i64,
    pub jti: uuid::Uuid,
    /// Scope granted to this refresh-token family. Copied verbatim onto every
    /// access token minted from this family so RFC 6749 §6 ("scope of the
    /// access token MUST NOT include any scope not originally granted") holds.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub scope: Option<String>,
}

/// Bundle returned by `create_refresh_jwt` — the encoded JWT string plus the
/// two DB-side pieces the caller needs to record it.
pub struct NewRefreshToken {
    pub token: String,
    pub jti: uuid::Uuid,
    pub expires: time::UtcDateTime,
}

/// Ed25519 keypair + derived JWKS/JWT metadata, loaded once at startup and
/// reused for every token operation. Holding the keys in a `OnceLock` means
/// we don't re-parse PEM or hit the filesystem on the hot path.
pub struct JwtKeys {
    encoding: jsonwebtoken::EncodingKey,
    decoding: jsonwebtoken::DecodingKey,
    /// RFC 7638 JWK Thumbprint of the public key, base64url-encoded. Stable
    /// across restarts as long as the key on disk doesn't change.
    kid: String,
    /// Raw 32-byte Ed25519 public key, exposed in the JWKS `x` field.
    public_key_bytes: [u8; 32],
}

impl JwtKeys {
    pub fn kid(&self) -> &str {
        &self.kid
    }

    pub fn public_key_bytes(&self) -> &[u8; 32] {
        &self.public_key_bytes
    }
}

static JWT_KEYS: OnceLock<JwtKeys> = OnceLock::new();

/// Read the private key PEM at `path`, derive the public key, and stash both
/// for use by `create_jwt` / `validate_jwt`. Call once at startup; subsequent
/// calls are no-ops.
pub fn init_keys_from_path(path: &str) -> Result<(), JwtError> {
    let pem = std::fs::read_to_string(path)
        .map_err(|err| JwtError::KeyLoad(format!("failed to read {}: {}", path, err)))?;
    init_keys_from_pem(&pem)
}

/// Parse a PKCS#8 PEM Ed25519 private key and stash the derived keys.
/// Idempotent — re-invoking with the same or different PEM after the first
/// successful call is a no-op.
pub fn init_keys_from_pem(pem: &str) -> Result<(), JwtError> {
    if JWT_KEYS.get().is_some() {
        return Ok(());
    }

    let signing = ed25519_dalek::SigningKey::from_pkcs8_pem(pem)
        .map_err(|err| JwtError::KeyLoad(format!("invalid PKCS#8 Ed25519 PEM: {}", err)))?;
    let verifying = signing.verifying_key();
    let public_key_bytes = verifying.to_bytes();

    let public_pem = verifying
        .to_public_key_pem(LineEnding::LF)
        .map_err(|err| JwtError::KeyLoad(format!("failed to encode public PEM: {}", err)))?;

    let encoding = jsonwebtoken::EncodingKey::from_ed_pem(pem.as_bytes())
        .map_err(|err| JwtError::KeyLoad(format!("jsonwebtoken rejected private key: {}", err)))?;
    let decoding = jsonwebtoken::DecodingKey::from_ed_pem(public_pem.as_bytes())
        .map_err(|err| JwtError::KeyLoad(format!("jsonwebtoken rejected public key: {}", err)))?;

    let kid = compute_jwk_thumbprint(&public_key_bytes);

    JWT_KEYS
        .set(JwtKeys {
            encoding,
            decoding,
            kid,
            public_key_bytes,
        })
        .map_err(|_| JwtError::KeyLoad("keys already initialized".to_string()))
}

pub fn keys() -> Result<&'static JwtKeys, JwtError> {
    JWT_KEYS.get().ok_or(JwtError::KeysUninitialized)
}

/// RFC 7638: JWK Thumbprint. Members are ordered lexicographically (`crv`,
/// `kty`, `x`) and serialized with no whitespace; the SHA-256 of that JSON,
/// base64url-encoded, is the thumbprint.
fn compute_jwk_thumbprint(public_key_bytes: &[u8; 32]) -> String {
    use sha2::{Digest, Sha256};

    let x = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(public_key_bytes);
    let canonical = format!(r#"{{"crv":"Ed25519","kty":"OKP","x":"{}"}}"#, x);
    let digest = Sha256::digest(canonical.as_bytes());
    base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(digest)
}

/// Resource-server audience for access tokens (RFC 9068 §2.2). The API is
/// served under `/api/*` on the same origin as the AS, so the resource
/// identifier is `SERVER_ORIGIN/api`. Anyone else validating an access token
/// (a future split-out resource server) uses this string as their expected
/// audience.
fn resource_audience() -> Result<String, JwtError> {
    let iss = std::env::var("SERVER_ORIGIN")?;
    Ok(format!("{}/api", iss.trim_end_matches('/')))
}

fn sign_jwt<T: serde::Serialize>(typ: &str, claims: &T) -> Result<String, JwtError> {
    let keys = keys()?;
    let mut header = jsonwebtoken::Header::new(Algorithm::EdDSA);
    header.typ = Some(typ.to_string());
    header.kid = Some(keys.kid.clone());
    Ok(jsonwebtoken::encode::<T>(&header, claims, &keys.encoding)?)
}

pub fn create_refresh_jwt(
    client_id: uuid::Uuid,
    user_id: uuid::Uuid,
    scope: Option<String>,
) -> Result<NewRefreshToken, JwtError> {
    let jti = uuid::Uuid::now_v7();
    let iss = std::env::var("SERVER_ORIGIN")?;

    let now = time::UtcDateTime::now();
    let exp = now.add(time::Duration::days(10));

    let claims = RefreshClaims {
        iss,
        aud: client_id,
        sub: user_id,
        exp: exp.unix_timestamp(),
        iat: now.unix_timestamp(),
        jti,
        scope,
    };

    let token = sign_jwt("rt+jwt", &claims)?;
    Ok(NewRefreshToken {
        token,
        jti,
        expires: exp,
    })
}

pub fn create_jwt(
    user_id: uuid::Uuid,
    client_id: uuid::Uuid,
    scope: Option<String>,
) -> Result<String, JwtError> {
    let now = time::UtcDateTime::now();
    let exp = now.add(time::Duration::days(1)).unix_timestamp();
    let iss = std::env::var("SERVER_ORIGIN")?;
    let aud = resource_audience()?;

    let claims = Claims {
        iss,
        aud,
        sub: user_id,
        client_id,
        jti: uuid::Uuid::now_v7(),
        iat: now.unix_timestamp(),
        exp,
        scope,
    };

    // RFC 9068 §2.1: access tokens SHOULD use `typ: at+jwt` so they can't be
    // confused with ID tokens or unrelated JWTs.
    sign_jwt("at+jwt", &claims)
}

pub fn validate_jwt(token: &str) -> Result<TokenData<Claims>, JwtError> {
    let keys = keys()?;
    let iss = std::env::var("SERVER_ORIGIN")?;
    let aud = resource_audience()?;

    let mut validater = Validation::new(Algorithm::EdDSA);
    validater.set_issuer(&[iss]);
    validater.set_audience(&[aud]);
    validater.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);

    let jwt = jsonwebtoken::decode::<Claims>(token, &keys.decoding, &validater)?;
    Ok(jwt)
}

pub fn validate_refresh_token(token: &str) -> Result<TokenData<RefreshClaims>, JwtError> {
    let keys = keys()?;
    let iss = std::env::var("SERVER_ORIGIN")?;

    let mut validater = Validation::new(Algorithm::EdDSA);
    validater.set_issuer(&[iss]);
    validater.set_audience(&[OAUTH_CLIENT_ID]);
    validater.set_required_spec_claims(&["exp", "iss", "aud"]);

    let jwt = jsonwebtoken::decode::<RefreshClaims>(token, &keys.decoding, &validater)?;
    Ok(jwt)
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::pkcs8::EncodePrivateKey;
    use ed25519_dalek::pkcs8::spki::der::pem::LineEnding;

    fn setup_test_key() {
        unsafe {
            std::env::set_var("SERVER_ORIGIN", "http://localhost:5000");
        }
        // Deterministic Ed25519 keypair — never use in production. OnceLock
        // makes the actual init a no-op after the first test sets it.
        let signing = ed25519_dalek::SigningKey::from_bytes(&[42u8; 32]);
        let pem = signing
            .to_pkcs8_pem(LineEnding::LF)
            .expect("to_pkcs8_pem");
        let _ = super::init_keys_from_pem(&pem);
    }

    #[test]
    fn test_create_and_validate_jwt() {
        setup_test_key();

        let user_id = uuid::uuid!("00000000-0000-0000-1000-000000000000");
        let jwt = super::create_jwt(
            user_id,
            super::OAUTH_CLIENT_ID,
            Some("profile offline_access".to_string()),
        )
        .expect("Failed to create jwt");

        let decoded = super::validate_jwt(&jwt).expect("failed to validate jwt");
        assert_eq!(decoded.claims.sub, user_id);
        assert_eq!(decoded.claims.client_id, super::OAUTH_CLIENT_ID);
        assert_eq!(decoded.claims.aud, "http://localhost:5000/api");
        assert_eq!(decoded.claims.scope.as_deref(), Some("profile offline_access"));
        assert_eq!(decoded.header.typ.as_deref(), Some("at+jwt"));
        assert!(decoded.header.kid.is_some());
    }

    #[test]
    fn test_create_and_validate_refresh_jwt() {
        setup_test_key();

        let user_id = uuid::uuid!("00000000-0000-0000-1000-000000000000");
        let issued = super::create_refresh_jwt(
            super::OAUTH_CLIENT_ID,
            user_id,
            Some("offline_access".to_string()),
        )
        .expect("failed to make jwt");

        let data =
            super::validate_refresh_token(&issued.token).expect("failed to validate token");
        assert_eq!(data.claims.sub, user_id);
        assert_eq!(data.claims.scope.as_deref(), Some("offline_access"));
    }
}
