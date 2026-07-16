use crate::state::api_errors::{ApplicationError, ErrorDetail, InnerError};
use crate::state::oauth::jwt::JwtError;
use actix_identity::error::GetIdentityError;
use actix_web::HttpResponse;
use actix_web::http::{StatusCode, header};

/// Compose a redirect Location that echoes an OAuth error back to the client
/// per RFC 6749 §4.1.2.1. Every parameter is percent-encoded via `url::Url`
/// so `state`, `error`, and `error_description` cannot smuggle extra query
/// parameters into the callback.
fn build_error_redirect(
    redirect_uri: &str,
    error: &str,
    error_description: &str,
    state: Option<&str>,
) -> String {
    let mut url = match url::Url::parse(redirect_uri) {
        Ok(u) => u,
        Err(_) => return redirect_uri.to_string(),
    };
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("error", error);
        q.append_pair("error_description", error_description);
        if let Some(state) = state {
            q.append_pair("state", state);
        }
    }
    url.to_string()
}

/// RFC 6749 §5.2 error payload for the token endpoint.
#[derive(serde::Serialize)]
struct OAuthErrorBody<'a> {
    error: &'a str,
    error_description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_uri: Option<&'static str>,
}

fn is_server_error(err: &OAuthErrorType) -> bool {
    matches!(
        err,
        OAuthErrorType::EnvVar(_)
            | OAuthErrorType::UrlParse(_)
            | OAuthErrorType::Db(_)
            | OAuthErrorType::Session(_)
            | OAuthErrorType::Uuid(_)
            | OAuthErrorType::Jwt(JwtError::Var(_))
    )
}

fn status_for(err: &OAuthErrorType) -> StatusCode {
    if is_server_error(err) {
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::BAD_REQUEST
    }
}

// -----------------------------------------------------------------------------
// Authorize-endpoint error (RFC 6749 §4.1.2.1: redirect back with error).
// -----------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
#[error("{}", .error_type)]
pub struct OAuthAuthorizeError {
    #[source]
    pub error_type: Box<OAuthErrorType>,
    pub state: Option<String>,
    pub valid_redirect_uri: Option<String>,
}

impl OAuthAuthorizeError {
    pub fn error(error_type: impl Into<OAuthErrorType>) -> Self {
        Self {
            error_type: Box::new(error_type.into()),
            valid_redirect_uri: None,
            state: None,
        }
    }

    pub fn redirect(
        err: impl Into<OAuthErrorType>,
        state: &str,
        valid_redirect_uri: String,
    ) -> Self {
        Self {
            error_type: Box::new(err.into()),
            state: Some(state.to_string()),
            valid_redirect_uri: Some(valid_redirect_uri),
        }
    }
}

impl<T> From<T> for OAuthAuthorizeError
where
    T: Into<OAuthErrorType>,
{
    fn from(value: T) -> Self {
        OAuthAuthorizeError::error(value.into())
    }
}

impl actix_web::ResponseError for OAuthAuthorizeError {
    fn status_code(&self) -> StatusCode {
        status_for(&self.error_type)
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        if let Some(redirect_uri) = self.valid_redirect_uri.as_deref() {
            let location = build_error_redirect(
                redirect_uri,
                self.error_type.name_static(),
                &self.error_type.to_string(),
                self.state.as_deref(),
            );

            HttpResponse::Found()
                .append_header((header::LOCATION, location))
                .insert_header((header::REFERRER_POLICY, "no-referrer"))
                .insert_header((header::X_FRAME_OPTIONS, "DENY"))
                .insert_header((header::CONTENT_SECURITY_POLICY, "frame-ancestors 'none'"))
                .insert_header((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
                .finish()
        } else {
            let status = self.status_code();
            HttpResponse::build(status).json(ApplicationError::new(
                status.as_u16(),
                self.error_type.name_static().to_string(),
                "query",
                vec![ErrorDetail::new(
                    status.as_u16(),
                    "query",
                    self.error_type.to_string(),
                )],
                self.error_type.get_context(),
            ))
        }
    }
}

// -----------------------------------------------------------------------------
// Token-endpoint error (RFC 6749 §5.2: JSON body, never redirect).
// -----------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
#[error("{}", .error_type)]
pub struct OAuthTokenError {
    #[source]
    pub error_type: Box<OAuthErrorType>,
}

impl OAuthTokenError {
    pub fn error(error_type: impl Into<OAuthErrorType>) -> Self {
        Self {
            error_type: Box::new(error_type.into()),
        }
    }
}

impl<T> From<T> for OAuthTokenError
where
    T: Into<OAuthErrorType>,
{
    fn from(value: T) -> Self {
        OAuthTokenError::error(value.into())
    }
}

impl actix_web::ResponseError for OAuthTokenError {
    fn status_code(&self) -> StatusCode {
        status_for(&self.error_type)
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        let status = self.status_code();
        let body = OAuthErrorBody {
            error: self.error_type.name_static(),
            error_description: self.error_type.to_string(),
            error_uri: None,
        };

        HttpResponse::build(status)
            .insert_header((header::CACHE_CONTROL, "no-store"))
            .insert_header((header::PRAGMA, "no-cache"))
            .json(body)
    }
}

// -----------------------------------------------------------------------------
// Shared error taxonomy
// -----------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum OAuthErrorType {
    #[error("The provided refresh token was invalid")]
    MissingRefreshToken,

    #[error("The provided refresh token was malformed")]
    MalformedRefreshToken,
    #[error("The provided code was malformed")]
    MalformedCode,
    #[error("The provided code verifier was malformed")]
    MalformedCodeVerifier,
    #[error("the provided redirect URI is invalid")]
    InvalidRedirect,
    #[error("The request is missing a required parameter or is otherwise malformed")]
    InvalidRequest,
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
    #[error("The requested scope is invalid, unknown, or malformed")]
    InvalidScope,

    #[error("Internal Server Error")]
    Session(#[from] GetIdentityError),
    #[error("Internal Server Error")]
    EnvVar(#[from] std::env::VarError),
    #[error("Internal Server Error")]
    UrlParse(#[from] url::ParseError),

    #[error("Internal Server Error")]
    Db(#[from] sqlx::Error),

    #[error("Internal Server Error")]
    Jwt(#[from] JwtError),

    #[error("Internal Server Error")]
    Uuid(#[from] uuid::Error),

    #[error("The provided authorization grant type is not supported by the authorization server.")]
    UnsupportedGrantType,
}

impl OAuthErrorType {
    /// RFC 6749 §5.2 error code (allocated) or "server_error"/"invalid_request"
    /// for internal failures, as a `&'static str` for zero-alloc JSON output.
    pub fn name_static(&self) -> &'static str {
        match self {
            Self::UnsupportedGrantType => "unsupported_grant_type",
            Self::EnvVar(_)
            | Self::UrlParse(_)
            | Self::Db(_)
            | Self::Session(_)
            | Self::Uuid(_) => "server_error",
            Self::Jwt(err) => match err {
                JwtError::Jwt(_) => "invalid_request",
                _ => "server_error",
            },
            Self::InvalidRedirect
            | Self::InvalidRequest
            | Self::MissingRefreshToken
            | Self::UnsupportedCodeChallengeMethod
            | Self::InvalidCodeChallenge
            | Self::MalformedCodeVerifier
            | Self::MalformedCode
            | Self::MalformedRefreshToken => "invalid_request",
            Self::InvalidScope => "invalid_scope",
            Self::AccessDenied => "access_denied",
            Self::InvalidClientId => "invalid_client",
            Self::InvalidCodeGrant => "invalid_grant",
        }
    }

    pub fn get_context(&self) -> Option<InnerError> {
        match self {
            Self::Jwt(err) => match err {
                JwtError::Var(var_error) => {
                    Some(InnerError::labeled("env".into(), var_error.to_string()))
                }
                JwtError::Jwt(error) => Some(InnerError::labeled("jwt".into(), error.to_string())),
                JwtError::KeysUninitialized | JwtError::KeyLoad(_) => {
                    Some(InnerError::labeled("jwt".into(), err.to_string()))
                }
            },
            Self::EnvVar(err) => Some(InnerError::labeled("env".into(), err.to_string())),
            Self::UrlParse(err) => Some(InnerError::labeled(
                "url parse".to_string(),
                err.to_string(),
            )),
            Self::Db(err) => Some(InnerError::labeled("db".into(), err.to_string())),
            Self::Session(err) => Some(InnerError::labeled("session".into(), err.to_string())),
            Self::Uuid(err) => Some(InnerError::labeled("uuid".to_string(), err.to_string())),
            _ => None,
        }
    }
}
