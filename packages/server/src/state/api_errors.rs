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
