use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use utoipa::{ToResponse, ToSchema};

#[derive(Debug, serde::Serialize, Clone, ToSchema)]
pub struct ErrorDetail {
    pub code: u16,
    pub target: String,
    pub message: String,
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
    pub fn labeled(label: String, reason: String) -> Self {
        Self {
            trace: vec![label, reason],
        }
    }
}

#[derive(Debug, serde::Serialize, Clone, ToResponse)]
#[response(description = "Error response object containing reason for error")]
pub struct ApplicationError {
    pub code: u16,
    pub message: String,
    pub target: String,
    pub details: Vec<ErrorDetail>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub innererror: Option<InnerError>,
}

impl ResponseError for ApplicationError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).json(self)
    }
    fn status_code(&self) -> actix_web::http::StatusCode {
        StatusCode::from_u16(self.code).expect("failed to convert u16 to status code")
    }
}

impl ApplicationError {
    pub fn new<C: Into<u16>, S: Into<String>, R: Into<String>>(
        code: C,
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
            code: code.into(),
            message: message.into(),
            target: target.into(),
            details,
            innererror: ctx,
        }
    }
}

impl std::fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "application error: {} | {}", self.code, self.message)
    }
}

pub fn from_sqlx_error(err: sqlx::Error) -> actix_web::Error {
    match err {
        sqlx::Error::Database(database_error) => {
            actix_web::error::ErrorBadRequest(ApplicationError::new(
                400u16,
                "bad request",
                "request",
                vec![ErrorDetail::new(400, "query", database_error.to_string())],
                None,
            ))
        }
        _ => actix_web::error::ErrorInternalServerError(ApplicationError::new(
            500u16,
            "Interal Server Error",
            "server",
            Vec::default(),
            Some(InnerError::new(err.to_string())),
        )),
    }
}
