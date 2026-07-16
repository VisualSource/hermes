use std::str::FromStr;

use actix_web::HttpResponse;
use actix_web::http::header;
use uuid::uuid;

use crate::state::oauth::errors::{OAuthAuthorizeError, OAuthErrorType};

pub mod code;
pub mod errors;
pub mod jwt;

pub const OAUTH_CLIENT_ID: uuid::Uuid = uuid!("00000000-0000-0000-0000-000000000000");
pub const OAUTH_REDIRECT_URI: &str = "hermes://oauth";
pub const OAUTH_DEV_REDIRECT_URI: &str = "http://localhost:1420/oauth";

/// Build a 302 Found response with the hardening headers we apply on every
/// OAuth redirect: no-referrer (so codes/tokens don't leak in `Referer`),
/// deny-framing (X-Frame-Options + CSP), and nosniff. Every OAuth-adjacent
/// redirect must go through this so the header set stays uniform.
pub fn secure_redirect(location: impl Into<String>) -> HttpResponse {
    HttpResponse::Found()
        .insert_header((header::LOCATION, location.into()))
        .insert_header((header::REFERRER_POLICY, "no-referrer"))
        .insert_header((header::X_FRAME_OPTIONS, "DENY"))
        .insert_header((header::CONTENT_SECURITY_POLICY, "frame-ancestors 'none'"))
        .insert_header((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
        .finish()
}

/// Advertised scopes. Any request scope outside this set is rejected with
/// `invalid_scope` per RFC 6749 §4.1.2.1.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Scope {
    Profile,
    OfflineAccess,
}

impl Scope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Scope::Profile => "profile",
            Scope::OfflineAccess => "offline_access",
        }
    }
}

impl FromStr for Scope {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "profile" => Ok(Scope::Profile),
            "offline_access" => Ok(Scope::OfflineAccess),
            _ => Err(()),
        }
    }
}

/// Parse a space-separated scope string (RFC 6749 §3.3). Returns
/// `invalid_scope` if any token is unknown. Duplicates are collapsed.
pub fn parse_scopes(raw: &str) -> Result<Vec<Scope>, OAuthErrorType> {
    let mut out = Vec::new();
    for token in raw.split_ascii_whitespace() {
        let scope = Scope::from_str(token).map_err(|_| OAuthErrorType::InvalidScope)?;
        if !out.contains(&scope) {
            out.push(scope);
        }
    }
    Ok(out)
}

pub fn scopes_to_string(scopes: &[Scope]) -> String {
    scopes
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn scopes_from_stored(stored: &Option<String>) -> Vec<Scope> {
    // Anything already in the DB must have been validated on the way in, but
    // treat unknowns as absent to keep old rows safe.
    stored
        .as_deref()
        .map(|s| {
            s.split_ascii_whitespace()
                .filter_map(|t| Scope::from_str(t).ok())
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Debug, serde::Deserialize)]
pub struct OAuthAuthorizeQuery {
    /// Must be "code" for authorization code flow.
    pub response_type: String,
    pub client_id: uuid::Uuid,
    pub redirect_uri: String,
    /// Opaque CSRF value chosen by the client. Required per OAuth 2.1 §7.5.3.
    pub state: String,
    /// Space-separated scope list (RFC 6749 §3.3).
    pub scope: Option<String>,
    pub code_challenge: String,
    /// Must be "S256".
    pub code_challenge_method: String,
}

impl OAuthAuthorizeQuery {
    pub fn validate(&self) -> Result<(), OAuthAuthorizeError> {
        log::debug!("request uri: {}", self.redirect_uri);

        // Pre-redirect checks: redirect_uri and client_id must be validated
        // before we're willing to redirect an error back anywhere (RFC 6749
        // §3.1.2.4, §4.1.2.1).
        if self.redirect_uri != OAUTH_REDIRECT_URI && self.redirect_uri != OAUTH_DEV_REDIRECT_URI {
            return Err(OAuthAuthorizeError::error(OAuthErrorType::InvalidRedirect));
        }

        if self.client_id != OAUTH_CLIENT_ID {
            return Err(OAuthAuthorizeError::error(OAuthErrorType::InvalidClientId));
        }

        if self.state.is_empty() {
            return Err(OAuthAuthorizeError::error(OAuthErrorType::InvalidRequest));
        }

        if self.response_type != "code" {
            return Err(OAuthAuthorizeError::redirect(
                OAuthErrorType::InvalidCodeGrant,
                &self.state,
                self.redirect_uri.clone(),
            ));
        }

        if self.code_challenge_method != "S256" {
            return Err(OAuthAuthorizeError::redirect(
                OAuthErrorType::UnsupportedCodeChallengeMethod,
                &self.state,
                self.redirect_uri.clone(),
            ));
        }

        if self.code_challenge.len() < 43
            || self.code_challenge.len() > 128
            || !code::is_valid_pkce_charset(&self.code_challenge)
        {
            return Err(OAuthAuthorizeError::redirect(
                OAuthErrorType::InvalidCodeChallenge,
                &self.state,
                self.redirect_uri.clone(),
            ));
        }

        if let Some(scope) = &self.scope {
            parse_scopes(scope).map_err(|err| {
                OAuthAuthorizeError::redirect(err, &self.state, self.redirect_uri.clone())
            })?;
        }

        Ok(())
    }
}
