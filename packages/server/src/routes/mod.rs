pub mod api;
pub mod auth;
mod error;
pub mod static_files;
pub mod websocket;
use actix_web::{HttpResponse, get};

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
        "authorization_endpoint": format!("{}/auth/authorize",iss),
        "token_endpoint":  format!("{}/auth/token",iss),
        "scopes_supported": [
            "profile",
            "offline_access"
        ],
        "response_types_supported": ["code"],
        "response_modes_supported": ["query"],
        "grant_types_supported": ["authorization_code"],
        "code_challenge_methods_supported": ["S256"],
    }))
}
