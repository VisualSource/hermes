use actix_web::{
    HttpResponse, error,
    http::{StatusCode, header::ContentType},
};
use derive_more::derive::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum ApiError {
    #[display("internal error")]
    InternalError,
    #[display("bad request")]
    BadRequest,
    #[display("unauthorized")]
    Unauthorized,
    #[display("forbidden")]
    Forbidden,
    #[display("not found")]
    NotFound,
    #[display("not acceptable")]
    NotAcceptable,
    #[display("too many request")]
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
