use crate::state::oauth::{
    self,
    errors::{OAuthError, OAuthErrorType},
    jwt::Claims,
};
use actix_web::{
    HttpResponse, get,
    http::header::{self, CacheDirective},
    post, web,
};
use sqlx::SqlitePool;

use utoipa::ToSchema;

// https://auth0.com/docs/get-started/authentication-and-authorization-flow/authorization-code-flow
#[utoipa::path(
    get,
    tag="oauth",
    path = "/auth/authorize",
    description = "authorize a user for request a authorization code", 
    responses(
        (
            status = 302,
            headers(
                ("Location" = String, description = "redirect to login page if needed")
            )
        )
    )
)]
#[get("/authorize")]
pub async fn authorize(
    query: web::Query<oauth::OAuthAuthorizeQuery>,
) -> Result<HttpResponse, OAuthError> {
    let authed = false;
    if !authed {
        let iss = std::env::var("SERVER_ORIGIN")?;
        let mut return_to = format!(
            "{}/auth/authorize?response_type={}&client_id={}&redirect_uri={}&code_challenge={}&code_challenge_method={}",
            iss,
            query.response_type,
            query.client_id,
            query.redirect_uri,
            query.code_challenge,
            query.code_challenge_method
        );
        if let Some(state) = &query.state {
            return_to = format!("{return_to}&state={state}")
        }
        let redirect_uri = format!("{iss}/login?return_to={return_to}",);
        let resp = HttpResponse::Found()
            .insert_header((header::REFERRER_POLICY, "no-referrer"))
            .insert_header((header::X_FRAME_OPTIONS, "DENY"))
            .insert_header((header::CONTENT_SECURITY_POLICY, "frame-ancestors 'none'"))
            .insert_header((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
            .insert_header((header::LOCATION, redirect_uri))
            .finish();

        return Ok(resp);
    }

    query.validate()?;

    let code = oauth::code::generate_code();

    // insert request into db

    let mut redirect_uri = format!("{}?code={}", query.redirect_uri, code);
    if let Some(state) = &query.state {
        redirect_uri = format!("{redirect_uri}&state={state}");
    }

    return Ok(HttpResponse::Found()
        .insert_header((header::LOCATION, redirect_uri))
        .insert_header((header::REFERRER_POLICY, "no-referrer"))
        .insert_header((header::X_FRAME_OPTIONS, "DENY"))
        .insert_header((header::CONTENT_SECURITY_POLICY, "frame-ancestors 'none'"))
        .insert_header((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
        .finish());
}

#[derive(Debug, serde::Deserialize, ToSchema)]
struct OAuthTokenRequest {
    grant_type: String,
    client_id: String,
    code: String,
    code_verifier: String,
}

impl OAuthTokenRequest {
    fn validate(&self) -> Result<(), OAuthError> {
        if self.grant_type != "refresh_token" || self.grant_type != "authorization_code" {
            return Err(OAuthError::error(OAuthErrorType::UnsupportedGrantType));
        }

        if self.client_id != oauth::OAUTH_CLIENT_ID {
            return Err(OAuthError::error(OAuthErrorType::InvalidClientId));
        }

        if self.code_verifier.len() < 43 || self.code_verifier.len() > 128 {
            return Err(OAuthError::error(OAuthErrorType::MalformedCodeVerifier));
        }

        if self.code.len() != 32 {
            return Err(OAuthError::error(OAuthErrorType::MalformatedCode));
        }

        Ok(())
    }
}

#[derive(Debug, serde::Serialize)]
struct OAuthTokenResponse {
    access_token: jsonwebtoken::jws::Jws<Claims>,
    refresh_token: Option<jsonwebtoken::jws::Jws<Claims>>,
    token_type: String,
    /// number that represents the lifetime in seconds of the access token.
    expires_in: i64,
    scope: Option<String>,
}

#[utoipa::path(
    post,
    tag="oauth",
    path = "/auth/token",
    description = "request a access_token using a authoriztion code", 
    responses(
        (
            status = OK,
            content_type="application/json", 
            body = String
        )
    )
)]
#[post("/token")]
pub async fn token(
    body: web::Form<OAuthTokenRequest>,
    db: web::Data<SqlitePool>,
) -> Result<HttpResponse, OAuthError> {
    body.validate()?;

    let method = "S256";
    let challenge = "";

    if !oauth::code::validate_pkce(challenge, &body.code_verifier, method) {
        return Err(OAuthError::error(OAuthErrorType::AccessDenied));
    };

    // lookup code in db get request

    let res = OAuthTokenResponse {
        access_token: todo!(),
        refresh_token: None,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        scope: None,
    };

    Ok(HttpResponse::Ok()
        .insert_header(header::CacheControl(vec![CacheDirective::NoStore]))
        .json(res))
}
