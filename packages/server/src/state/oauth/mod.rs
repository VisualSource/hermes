use crate::state::oauth::errors::{OAuthError, OAuthErrorType};

pub mod code;
pub mod errors;
pub mod jwt;

pub const OAUTH_CLIENT_ID: &str = "";
pub const OAUTH_REDIRECT_URI: &str = "";
pub const OAUTH_CLIENT_SECRET: &str = "";

#[derive(Debug, serde::Deserialize)]
pub struct OAuthAuthorizeQuery {
    // Must be code for authorization code flow
    pub response_type: String,
    // The client identifier
    pub client_id: String,
    // Where to redirect after authorization
    pub redirect_uri: String,
    // Random string for CSRF protection
    pub state: Option<String>,
    // Space-separated list of requested permissions
    pub scope: Option<String>,
    // PKCE challenge derived from code_verifier
    pub code_challenge: String,
    // Must be S256
    pub code_challenge_method: String,
}

impl OAuthAuthorizeQuery {
    pub fn validate(&self) -> Result<(), OAuthError> {
        if self.redirect_uri != OAUTH_REDIRECT_URI {
            return Err(OAuthError::error(OAuthErrorType::InvalidRedirect));
        }

        if self.client_id != OAUTH_CLIENT_ID {
            return Err(OAuthError::error(OAuthErrorType::InvalidClientId));
        }

        if self.response_type != "code" {
            return Err(OAuthError::redirect(
                OAuthErrorType::InvalidCodeGrant,
                &self.state,
                self.redirect_uri.clone(),
            ));
        }

        if self.code_challenge_method != "S256" {
            return Err(OAuthError::redirect(
                OAuthErrorType::UnsupportedCodeChallengeMethod,
                &self.state,
                self.redirect_uri.clone(),
            ));
        }

        if self.code_challenge.len() < 44 {
            return Err(OAuthError::redirect(
                OAuthErrorType::InvalidCodeChallenge,
                &self.state,
                self.redirect_uri.clone(),
            ));
        }

        Ok(())
    }
}
