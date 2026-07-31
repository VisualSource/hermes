use std::env;

use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub ephemeral_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TurnstileResponse {
    Error {
        success: bool,
        #[serde(rename = "error-codes")]
        error_codes: Vec<String>,
    },
    Ok {
        success: bool,
        challenge_ts: String,
        hostname: String,
        #[serde(rename = "error-codes")]
        error_codes: Vec<String>,
        action: String,
        cdata: String,
        metadata: Metadata,
    },
}

impl TurnstileResponse {
    pub fn is_err(&self) -> bool {
        match self {
            TurnstileResponse::Error { .. } => true,
            TurnstileResponse::Ok { .. } => false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TurnstileError {
    #[error("failed to get env var")]
    MissingEnv(#[from] env::VarError),
    #[error("network error")]
    Request(#[from] reqwest::Error),
    #[error("verification failed for client")]
    Verify(TurnstileResponse),
}

pub async fn validate_token(
    remote_ip: String,
    token: String,
) -> Result<TurnstileResponse, TurnstileError> {
    let secret = env::var("CLOUDFLARE_TURNSTILE_API_KEY")?;

    let data: TurnstileResponse = reqwest::Client::new()
        .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
        .json(&json!({
            "secret":secret,
            "response": token,
            "remoteip": remote_ip
        }))
        .send()
        .await?
        .json()
        .await?;

    if data.is_err() {
        return Err(TurnstileError::Verify(data));
    }

    Ok(data)
}
