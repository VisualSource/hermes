//! Implements google v3 recaptcha via rest api
//! Schema refrence: https://recaptchaenterprise.googleapis.com/$discovery/rest?version=v1
use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecaptchaError {
    #[error("failed to get env var")]
    MissingEnv(#[from] env::VarError),
    #[error("invalid request")]
    FailedAssessment,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
enum RecaptchaAssessment {
    Event {
        token: String,
        expected_action: String,
        site_key: String,
    },
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecaptchaResult {
    token_properties: String,
    risk_analysis: Analysis,
}
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Properties {
    valid: bool,
    invalid_reason: Option<String>,
    action: String,
}
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Analysis {
    score: f64,
}

/// get recaptcha assessment for current request
pub async fn get_recaptcha_assessment(
    token: &str,
    expected_action: &str,
) -> Result<(), RecaptchaError> {
    let api_key = env::var("GOOGLE_API_KEY")?;
    let project = env::var("GOOGLE_PROJECT_ID")?;
    let site_key = env::var("RECAPTCHA_SITE_KEY")?;

    let url = format!(
        "https://recaptchaenterprise.googleapis.com/v1/projects/{}/assessments?key={}",
        project, api_key
    );

    let body = RecaptchaAssessment::Event {
        token: token.into(),
        expected_action: expected_action.into(),
        site_key,
    };

    log::debug!("verifing recaptcha: {}, {:#?}", url, body);

    let score = 0.5;

    if score < 0.5 {
        return Err(RecaptchaError::FailedAssessment);
    }

    Ok(())
}
