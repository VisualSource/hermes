use actix_web::HttpRequest;
use actix_web::http::header::SEC_WEBSOCKET_PROTOCOL;

use crate::state::api_errors::{ApplicationError, ErrorDetail};
use crate::state::oauth::jwt::{Claims, JwtError, validate_jwt};

/// Subprotocol name the client offers alongside the bearer token, per the
/// pattern `Sec-WebSocket-Protocol: bearer, <jwt>`. Keeps the JWT off the URL
/// query so it never lands in access logs, Referer headers, or proxy caches.
pub const BEARER_SUBPROTOCOL: &str = "bearer";

/// Parse `Sec-WebSocket-Protocol: bearer, <jwt>` into the JWT string. Returns
/// `None` if the header is missing or the shape doesn't match.
fn extract_bearer_token(req: &HttpRequest) -> Option<String> {
    let header = req.headers().get(SEC_WEBSOCKET_PROTOCOL)?;
    let raw = header.to_str().ok()?;

    let mut parts = raw.split(',').map(str::trim);
    let scheme = parts.next()?;
    let token = parts.next()?;

    if scheme != BEARER_SUBPROTOCOL || token.is_empty() || parts.next().is_some() {
        return None;
    }

    Some(token.to_string())
}

pub fn validate(req: &HttpRequest) -> Result<Claims, ApplicationError> {
    let raw = extract_bearer_token(req).ok_or_else(|| {
        ApplicationError::unauthorized(
            "unauthorized",
            "sec-websocket-protocol",
            vec![ErrorDetail::new(
                4001,
                "header",
                "expected `Sec-WebSocket-Protocol: bearer, <jwt>`",
            )],
        )
    })?;

    let claims = match validate_jwt(&raw) {
        Ok(token) => token.claims,
        Err(JwtError::Jwt(err)) => {
            log::trace!("{}", err);
            return Err(ApplicationError::unauthorized(
                "unauthorized",
                "sec-websocket-protocol",
                Vec::default(),
            ));
        }
        Err(other) => {
            log::error!("{}", other);
            return Err(ApplicationError::internal_server_error(
                "internal server error",
                "server",
                other.to_string(),
            ));
        }
    };

    Ok(claims)
}
