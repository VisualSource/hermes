use actix_web::{HttpResponse, ResponseError, error, http::StatusCode};
use thiserror::Error;

use crate::state::api_errors::{ApplicationError, InnerError};

//TODO: impl better error object => https://docs.oasis-open.org/odata/odata-json-format/v4.0/errata02/os/odata-json-format-v4.0-errata02-os-complete.html#_Toc403940655
#[derive(Debug, Error)]
pub enum AuthPageError {
    #[error(transparent)]
    MailBoxError(#[from] actix::MailboxError),
    #[error(transparent)]
    DbError(#[from] sqlx::Error),
    #[error("argon error")]
    Argon(String),

    #[error("a error happened in the request")]
    Request(ApplicationError),

    #[error("failed to insert value")]
    Login(#[from] actix_identity::error::LoginError),
}

impl AuthPageError {
    fn get_body(&self) -> ApplicationError {
        match &self {
            Self::Login(err) => ApplicationError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
                "server",
                Vec::default(),
                Some(InnerError::new(err.to_string())),
            ),
            Self::Request(error) => error.clone(),

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
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
