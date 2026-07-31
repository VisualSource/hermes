use actix_web::{
    Error, HttpMessage, HttpResponse, ResponseError,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    http::{StatusCode, header},
    middleware::Next,
};

use crate::state::api_errors::{ApplicationError, InnerError};
use crate::state::oauth::jwt::validate_jwt;

#[derive(Debug, thiserror::Error)]
enum AuthError {
    #[error("expected `Authorization: Bearer <jwt>`")]
    Missing,
    #[error("invalid: {0}")]
    Invalid(String),
}

impl ResponseError for AuthError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        StatusCode::UNAUTHORIZED
    }
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        let detail = self.to_string();

        HttpResponse::Unauthorized().json(ApplicationError::new(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "header",
            Vec::default(),
            Some(InnerError::new(detail)),
        ))
    }
}

pub async fn require_jwt(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
        .ok_or(AuthError::Missing)?;

    let data = validate_jwt(&token).map_err(|e| AuthError::Invalid(e.to_string()))?;
    req.extensions_mut().insert(data.claims);

    next.call(req).await
}
