use std::ops::Add;

use actix_web::{HttpResponse, get, http::header, post, web};
use base64::Engine;
use serde::Deserialize;

use crate::{
    routes::error::OAuthError,
    state::{self, oauth},
};

use utoipa::ToSchema;

#[derive(Debug, Deserialize)]
struct OAuthAuthorizeQuery {
    // Must be code for authorization code flow
    response_type: String,
    // The client identifier
    client_id: String,
    // Where to redirect after authorization
    redirect_uri: String,
    // Random string for CSRF protection
    state: String,
    // Space-separated list of requested permissions
    scope: Option<String>,
    // PKCE challenge derived from code_verifier
    code_challenge: String,
    // Must be S256
    code_challenge_method: String,
}

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
pub async fn authorize(query: web::Query<OAuthAuthorizeQuery>) -> Result<HttpResponse, OAuthError> {
    if query.response_type != "code"
        || query.client_id != oauth::OAUTH_CLIENT_ID
        || !query.redirect_uri.starts_with(oauth::OAUTH_REDIRECT_URI)
        || query.code_challenge_method != "S256"
    {
        return Err(OAuthError::BadRequest);
    }

    //TODO: store code_challenge
    //TODO: validate scopes

    //TODO get return url from env
    let url = url::Url::parse_with_params(
        "http://localhost:7433/login",
        &[
            ("response_type", "code"),
            ("client_id", &query.client_id),
            ("redirect_uri", &query.redirect_uri),
            ("state", &query.state),
        ],
    )?;

    let resp = HttpResponse::Found()
        .insert_header((header::LOCATION, url.to_string()))
        .finish();

    Ok(resp)
}

#[derive(Debug, serde::Deserialize, ToSchema)]
struct OAuthToken {
    grant_type: String,
    client_id: String,
    code: String,
    // Optional (OAuth 2.1), required
    redirect_uri: Option<String>,
    code_verifier: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Claims {
    iss: String,
    // In OAuth 2.0, an identity provider (IdP) issues tokens with the aud claim set to the client ID.
    aud: String,
    iat: i64,
    exp: i64,
    sub: uuid::Uuid,
}

#[derive(Debug, serde::Serialize)]
struct OauthTokenResponse {
    access_token: jsonwebtoken::jws::Jws<Claims>,
    refresh_token: String,
    token_type: String,
    expires_in: i64,
    scope: String,
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
pub async fn token(body: web::Form<OAuthToken>) -> Result<HttpResponse, OAuthError> {
    if body.client_id != oauth::OAUTH_CLIENT_ID {
        return Err(OAuthError::BadRequest);
    }

    if body.grant_type == "refresh_token" {
        // return refresh token
        return Err(OAuthError::BadRequest);
    }

    if body.grant_type != "authorization_code" {
        return Err(OAuthError::BadRequest);
    }

    //  body.redirect_uri != oauth::OAUTH_REDIRECT_URI
    // TODO get code challenge and method
    let code_challenage = "";
    let code_method = "S256";

    if !state::oauth::validate_pkce(code_challenage, &body.code_verifier, code_method) {
        return Err(OAuthError::BadRequest);
    }

    // TODO: get user id

    let user_id = uuid::Uuid::now_v7();

    let now = time::UtcDateTime::now();
    let exp = now.add(time::Duration::days(1)).unix_timestamp();

    let claims = Claims {
        iss: "http://localhost:7433".to_string(), // TODO get from env
        aud: oauth::OAUTH_CLIENT_ID.to_string(),
        exp: exp,
        iat: now.unix_timestamp(),
        sub: user_id,
    };

    //TODO: replace with better key
    let key = jsonwebtoken::EncodingKey::from_secret(b"TODO REPLACE ME WITH A BETTER KEY");

    let mut header = jsonwebtoken::Header::default();
    header.alg = jsonwebtoken::Algorithm::HS512;

    let token = jsonwebtoken::jws::encode::<Claims>(&header, Some(&claims), &key)?;

    let res = OauthTokenResponse {
        access_token: token,
        refresh_token: "".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        scope: "".to_string(),
    };

    Ok(HttpResponse::Ok().json(res))
}
