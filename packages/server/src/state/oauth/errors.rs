use actix_web::HttpResponse;
use actix_web::http::{StatusCode, header};

use crate::state::api_errors::{ApplicationError, ErrorDetail, InnerError};

#[derive(Debug, thiserror::Error)]
#[error("{}",.error_type)]
pub struct OAuthError {
    #[source]
    pub error_type: Box<OAuthErrorType>,
    pub state: Option<String>,
    pub valid_redirect_uri: Option<String>,
}

impl<T> From<T> for OAuthError
where
    T: Into<OAuthErrorType>,
{
    fn from(value: T) -> Self {
        OAuthError::error(value.into())
    }
}

impl OAuthError {
    pub fn error(error_type: impl Into<OAuthErrorType>) -> Self {
        Self {
            error_type: Box::new(error_type.into()),
            valid_redirect_uri: None,
            state: None,
        }
    }
    pub fn redirect(
        err: impl Into<OAuthErrorType>,
        state: &Option<String>,
        valid_redirect_uri: String,
    ) -> Self {
        Self {
            error_type: Box::new(err.into()),
            state: state.clone(),
            valid_redirect_uri: Some(valid_redirect_uri),
        }
    }
}

impl actix_web::ResponseError for OAuthError {
    fn status_code(&self) -> StatusCode {
        match *self.error_type {
            _ => StatusCode::BAD_REQUEST,
        }
    }
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        if let Some(mut redirect_uri) = self.valid_redirect_uri.clone() {
            redirect_uri = format!(
                "{}?error={}&error_description={}",
                redirect_uri,
                self.error_type.name(),
                self.error_type
            );

            if let Some(state) = self.state.as_ref() {
                redirect_uri = format!("{redirect_uri}&state={state}")
            }

            HttpResponse::Ok()
                .append_header((header::LOCATION, redirect_uri.clone()))
                .insert_header((header::REFERRER_POLICY, "no-referrer"))
                .insert_header((header::X_FRAME_OPTIONS, "DENY"))
                .insert_header((header::CONTENT_SECURITY_POLICY, "frame-ancestors 'none'"))
                .insert_header((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
                .body(redirect_uri)
        } else {
            HttpResponse::build(self.status_code()).json(ApplicationError::new(
                400,
                self.error_type.name(),
                "query",
                vec![ErrorDetail::new(400, "query", self.error_type.to_string())],
                self.error_type.get_context(),
            ))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OAuthErrorType {
    #[error("The provided code was malformed")]
    MalformatedCode,
    #[error("The provided code verifier was malformed")]
    MalformedCodeVerifier,
    #[error("the provided redirect URI is invalid")]
    InvalidRedirect,
    #[error("The resource owner denied the request")]
    AccessDenied,
    #[error("The provided client id was invalid")]
    InvalidClientId,
    #[error("The provided code grant is not supported")]
    InvalidCodeGrant,

    #[error("The provided challenge method is not supported")]
    UnsupportedCodeChallengeMethod,
    #[error("The provided code challenge is invalid")]
    InvalidCodeChallenge,

    #[error("Internal Server Error")]
    EnvVar(#[from] std::env::VarError),
    #[error("Internal Server Error")]
    UrlParse(#[from] url::ParseError),

    #[error("Internal Server Error")]
    Db(#[from] sqlx::Error),

    #[error("The provided authorization grant type is not supported by the authorization server.")]
    UnsupportedGrantType,
}

impl OAuthErrorType {
    pub fn name(&self) -> String {
        match self {
            Self::UnsupportedGrantType => "unsupported_grant_type",
            Self::EnvVar(_) | Self::UrlParse(_) | Self::Db(_) => "server_error",
            Self::InvalidRedirect
            | Self::UnsupportedCodeChallengeMethod
            | Self::InvalidCodeChallenge
            | Self::MalformedCodeVerifier
            | Self::MalformatedCode => "invalid_request",
            Self::AccessDenied => "access_denied",
            Self::InvalidClientId => "invalid_client",
            Self::InvalidCodeGrant => "invalid_grant",
        }
        .to_string()
    }
    pub fn get_context(&self) -> Option<InnerError> {
        match self {
            Self::EnvVar(err) => Some(InnerError::new(err.to_string())),
            Self::UrlParse(err) => Some(InnerError::new(err.to_string())),
            Self::Db(err) => Some(InnerError::new(err.to_string())),
            _ => None,
        }
    }
}
