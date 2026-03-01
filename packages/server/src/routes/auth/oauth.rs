use std::str::FromStr;

use crate::{
    models::auth::{AuthorizationCode, RefreshToken},
    state::oauth::{
        self,
        errors::{OAuthError, OAuthErrorType},
        jwt::{ create_jwt, create_refresh_jwt, validate_refresh_token},
    },
};
use actix_identity::Identity;
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
    user: Option<Identity>,
    query: web::Query<oauth::OAuthAuthorizeQuery>,
    db: web::Data<SqlitePool>,
) -> Result<HttpResponse, OAuthError> {
    query.validate()?;

    if let Some(user) = user {
        let user_id = user.id().map_err(|err| {
            OAuthError::redirect(
                OAuthErrorType::Session(err),
                &query.state,
                query.redirect_uri.clone(),
            )
        })?;

        let user_id = uuid::Uuid::from_str(&user_id)?;

        let code = oauth::code::generate_code();
        AuthorizationCode::insert_request(
            &code,
            &user_id,
            &query.code_challenge,
            &query.code_challenge_method,
            query.scopes.clone(),
            &db,
        )
        .await?;

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

    let encoded_url = urlencoding::encode(&return_to);
    let redirect_uri = format!("{iss}/login?return_to={encoded_url}");
    let resp = HttpResponse::Found()
        .insert_header((header::REFERRER_POLICY, "no-referrer"))
        .insert_header((header::X_FRAME_OPTIONS, "DENY"))
        .insert_header((header::CONTENT_SECURITY_POLICY, "frame-ancestors 'none'"))
        .insert_header((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
        .insert_header((header::LOCATION, redirect_uri))
        .finish();

    return Ok(resp);
}

#[derive(Debug, serde::Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "grant_type")]
enum OAuthTokenRequest {
    AuthorizationCode {
        client_id: uuid::Uuid,
        code: String,
        code_verifier: String,
    },
    RefreshToken {
        refresh_token: String,
    },
    #[serde(other)]
    Unsupported,
}

impl OAuthTokenRequest {
    fn validate(&self) -> Result<(), OAuthError> {
        match self {
            OAuthTokenRequest::AuthorizationCode {
                client_id,
                code,
                code_verifier,
            } => {
                if client_id != &oauth::OAUTH_CLIENT_ID {
                    return Err(OAuthError::error(OAuthErrorType::InvalidClientId));
                }

                if code_verifier.len() < 43 || code_verifier.len() > 128 {
                    return Err(OAuthError::error(OAuthErrorType::MalformedCodeVerifier));
                }

                if code.len() != 32 {
                    return Err(OAuthError::error(OAuthErrorType::MalformatedCode));
                }

                Ok(())
            }
            OAuthTokenRequest::RefreshToken { refresh_token } => {
                if refresh_token.len() == 0 {
                    return Err(OAuthError::error(OAuthErrorType::MalformatedRefreshToken));
                }
                Ok(())
            }
            OAuthTokenRequest::Unsupported => {
                Err(OAuthError::error(OAuthErrorType::UnsupportedGrantType))
            }
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct OAuthTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
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

    match body.0 {
        OAuthTokenRequest::AuthorizationCode {
            client_id,
            code,
            code_verifier,
        } => {
            let request = match AuthorizationCode::get_by_code_and_user(&code, &db).await? {
                Some(v) => v,
                None => return Err(OAuthError::error(OAuthErrorType::AccessDenied)),
            };

            if !request.is_valid() {
                return Err(OAuthError::error(OAuthErrorType::AccessDenied));
            }

            if !oauth::code::validate_pkce(
                &request.code_challenge,
                &code_verifier,
                &request.code_challenge_method,
            ) {
                return Err(OAuthError::error(OAuthErrorType::AccessDenied));
            };

            let jwt = create_jwt(request.user_id, client_id)?;
            let refresh = create_refresh_jwt(client_id, request.user_id)?;

            AuthorizationCode::mark_code_used(&request.code, &db).await?;

            let res = OAuthTokenResponse {
                access_token: jwt,
                refresh_token: Some(refresh),
                token_type: "Bearer".to_string(),
                expires_in: 3600,
                scope: request.scopes,
            };

            Ok(HttpResponse::Ok()
                .insert_header(header::CacheControl(vec![CacheDirective::NoStore]))
                .json(res))
        }
        OAuthTokenRequest::RefreshToken { refresh_token } => {
            // validate refresh_token
            // get jwt from db validate refresh_token has valid client_id and user_id check
            let info = validate_refresh_token(&refresh_token)?;

            let token = match RefreshToken::get_token(&info.claims.jti, &db).await? {
                Some(t) => t,
                None => return Err(OAuthError::error(OAuthErrorType::AccessDenied)),
            };
            if token.user_id != info.claims.sub {
                return Err(OAuthError::error(OAuthErrorType::AccessDenied));
            }

            let jwt = create_jwt(token.user_id, info.claims.aud)?;
            let refresh = create_refresh_jwt(info.claims.aud, token.user_id)?;

            let res = OAuthTokenResponse {
                access_token: jwt,
                refresh_token: Some(refresh),
                token_type: "Bearer".to_string(),
                expires_in: 3600,
                scope: None,
            };

            Ok(HttpResponse::Ok()
                .insert_header(header::CacheControl(vec![CacheDirective::NoStore]))
                .json(res))
        }
        _ => Err(OAuthError::error(OAuthErrorType::UnsupportedGrantType)),
    }
}
