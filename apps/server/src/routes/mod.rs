pub mod api;
pub mod auth;
mod error;
pub mod static_files;
pub mod websocket;
use actix_web::{HttpResponse, get};
use base64::Engine;

use crate::state::oauth::jwt;

/// https://datatracker.ietf.org/doc/html/rfc8414
#[get("/.well-known/oauth2-authorization-server")]
pub async fn oauth_server_details() -> HttpResponse {
    let iss = match std::env::var("SERVER_ORIGIN") {
        Ok(v) => v,
        Err(err) => {
            log::error!("{}", err);
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok().json(serde_json::json!({
        "issuer": iss,
        "authorization_endpoint": format!("{}/auth/authorize", iss),
        "token_endpoint":  format!("{}/auth/token", iss),
        "revocation_endpoint": format!("{}/auth/revoke", iss),
        "jwks_uri": format!("{}/.well-known/jwks.json", iss),
        "scopes_supported": [
            "profile",
            "offline_access"
        ],
        "response_types_supported": ["code"],
        "response_modes_supported": ["query"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
        "code_challenge_methods_supported": ["S256"],
        "token_endpoint_auth_methods_supported": ["none"],
        "revocation_endpoint_auth_methods_supported": ["none"],
        "id_token_signing_alg_values_supported": ["EdDSA"],
    }))
}

/// RFC 7517 JSON Web Key Set. Serves the Ed25519 public key so downstream
/// verifiers (resource servers, plugins, offline-verifying clients) can
/// validate access tokens without holding any shared secret.
#[get("/.well-known/jwks.json")]
pub async fn jwks() -> HttpResponse {
    let keys = match jwt::keys() {
        Ok(k) => k,
        Err(err) => {
            log::error!("JWKS: keys not initialized: {}", err);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let x = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(keys.public_key_bytes());

    let body = serde_json::json!({
        "keys": [
            {
                "kty": "OKP",
                "crv": "Ed25519",
                "x": x,
                "kid": keys.kid(),
                "use": "sig",
                "alg": "EdDSA",
            }
        ]
    });

    HttpResponse::Ok()
        // JWKS responses are cacheable; verifiers refresh on unknown kid.
        .insert_header((actix_web::http::header::CACHE_CONTROL, "public, max-age=3600"))
        .json(body)
}
