use actix_web::{
    HttpResponse, error,
    http::{StatusCode, header::ContentType},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("internal error")]
    InternalError,
    #[error("bad request")]
    BadRequest,
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("not acceptable")]
    NotAcceptable,
    #[error("too many request")]
    TooManyRequest,
}

impl error::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            ApiError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::BadRequest => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::BAD_REQUEST,
            ApiError::Forbidden => StatusCode::FORBIDDEN,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::NotAcceptable => StatusCode::NOT_ACCEPTABLE,
            ApiError::TooManyRequest => StatusCode::TOO_MANY_REQUESTS,
        }
    }
}


//TODO: impl better error object => https://docs.oasis-open.org/odata/odata-json-format/v4.0/errata02/os/odata-json-format-v4.0-errata02-os-complete.html#_Toc403940655
#[derive(Debug,Error)]
pub enum AuthPageError {
    #[error("invalid form data")]
    InvalidFormData(Vec<(String,isize)>),
    #[error("invalid user")]
    Recaptcha,

    #[error(transparent)]
    WebError(#[from] oxide_auth_actix::WebError),

    #[error(transparent)]
    MailBoxError(#[from] actix::MailboxError),

    #[error("argron error")]
    Argon,
    #[error(transparent)]
    DbError(#[from] sqlx::Error),

    #[error("custom error")]
    Custom(StatusCode,String)
}

impl AuthPageError {
    fn get_body(&self) ->  impl serde::Serialize {
        match &self {
            Self::InvalidFormData(errors) => serde_json::json!({ "reason":"invalid_formdata", "errors": errors.to_owned()  }),
            Self::Custom(_, reason) => {
                serde_json::json!({
                    "reason": reason
                })
            }
            Self::WebError(web_error) => {
                let reason = web_error.to_string();
                serde_json::json!({ "reason": reason, })
            }
            _ => serde_json::json!({ "reason": "internal_server_error" })
         }
    }
}

impl error::ResponseError for AuthPageError{
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).json(self.get_body())
    }
    fn status_code(&self) -> StatusCode {
        match &self {
            Self::InvalidFormData(_) => StatusCode::BAD_REQUEST,
            Self::Recaptcha => StatusCode::FORBIDDEN,
            Self::WebError(web_error) => web_error.status_code(),
            Self::Custom(status, _) => *status,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}