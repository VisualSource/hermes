use actix_web::{HttpResponse, ResponseError, error, http::StatusCode};
use thiserror::Error;
use utoipa::{ToResponse, ToSchema};

#[derive(Debug, serde::Serialize, Clone, ToSchema)]
pub struct ErrorDetail {
    code: u16,
    target: String,
    message: String,
}

impl ErrorDetail {
    pub fn new<S: Into<String>, R: Into<String>>(code: u16, target: S, message: R) -> Self {
        Self {
            code,
            target: target.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, ToSchema)]
pub struct InnerError {
    trace: Vec<String>,
}

impl InnerError {
    pub fn new(trace: String) -> Self {
        Self { trace: vec![trace] }
    }
}

#[derive(Debug, serde::Serialize, Clone, ToResponse)]
#[response(description = "Error response object containing reason for error")]
pub struct ApplicationError {
    code: u16,
    message: String,
    target: String,
    details: Vec<ErrorDetail>,

    #[serde(skip_serializing_if = "Option::is_none")]
    innererror: Option<InnerError>,
}

impl ApplicationError {
    pub fn new<S: Into<String>, R: Into<String>>(
        code: u16,
        message: S,
        target: R,
        details: Vec<ErrorDetail>,
        error_context: Option<InnerError>,
    ) -> Self {
        #[cfg(debug_assertions)]
        let ctx = { error_context };

        #[cfg(not(debug_assertions))]
        let ctx = { None };

        Self {
            code,
            message: message.into(),
            target: target.into(),
            details,
            innererror: ctx,
        }
    }
}

//TODO: impl better error object => https://docs.oasis-open.org/odata/odata-json-format/v4.0/errata02/os/odata-json-format-v4.0-errata02-os-complete.html#_Toc403940655
#[derive(Debug, Error)]
pub enum AuthPageError {
    #[error("invalid user")]
    Recaptcha,

    #[error(transparent)]
    WebError(#[from] oxide_auth_actix::WebError),

    #[error(transparent)]
    MailBoxError(#[from] actix::MailboxError),
    #[error(transparent)]
    DbError(#[from] sqlx::Error),
    #[error("argon error")]
    Argon(String),

    #[error("a error happened in the request")]
    Request(ApplicationError),
}

impl AuthPageError {
    fn get_body(&self) -> ApplicationError {
        match &self {
            Self::Request(error) => error.clone(),

            Self::WebError(web_error) => {
                let reason = web_error.to_string();

                match web_error {
                    oxide_auth_actix::WebError::Endpoint(oauth_error) => ApplicationError::new(
                        self.status_code().as_u16(),
                        "Internal Server Error: code (2252)",
                        "server",
                        Vec::default(),
                        Some(InnerError {
                            trace: vec![oauth_error.to_string()],
                        }),
                    ),
                    oxide_auth_actix::WebError::Header(invalid_header_value) => {
                        ApplicationError::new(
                            self.status_code().as_u16(),
                            "Internal Server Error: code (2253)",
                            "server",
                            Vec::default(),
                            Some(InnerError {
                                trace: vec![invalid_header_value.to_string()],
                            }),
                        )
                    }
                    oxide_auth_actix::WebError::Encoding
                    | oxide_auth_actix::WebError::Mailbox
                    | oxide_auth_actix::WebError::Form
                    | oxide_auth_actix::WebError::Canceled
                    | oxide_auth_actix::WebError::Authorization
                    | oxide_auth_actix::WebError::Body
                    | oxide_auth_actix::WebError::Query
                    | oxide_auth_actix::WebError::InternalError(_) => ApplicationError::new(
                        self.status_code().as_u16(),
                        "Internal Server Error: code (2254)",
                        "server",
                        Vec::default(),
                        Some(InnerError {
                            trace: vec![reason],
                        }),
                    ),
                }
            }

            Self::Recaptcha => ApplicationError::new(
                self.status_code().as_u16(),
                "Unable to complate operation",
                "user",
                Vec::default(),
                None,
            ),

            Self::DbError(err) => ApplicationError::new(
                self.status_code().as_u16(),
                "Internal Server Error: code (2255)",
                "server",
                Vec::default(),
                Some(InnerError::new(err.to_string())),
            ),

            Self::MailBoxError(err) => ApplicationError::new(
                self.status_code().as_u16(),
                "Internal Server Error: code (2256)",
                "server",
                Vec::default(),
                Some(InnerError::new(err.to_string())),
            ),

            Self::Argon(reason) => ApplicationError::new(
                self.status_code().as_u16(),
                "Internal Server Error: code (2257)",
                "server",
                Vec::default(),
                Some(InnerError::new(reason.to_owned())),
            ),
        }
    }
}

impl error::ResponseError for AuthPageError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).json(self.get_body())
    }
    fn status_code(&self) -> StatusCode {
        match &self {
            Self::Request(r) => {
                StatusCode::from_u16(r.code).expect("failed to convert u16 to status code")
            }
            Self::Recaptcha => StatusCode::FORBIDDEN,
            Self::WebError(web_error) => web_error.status_code(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
