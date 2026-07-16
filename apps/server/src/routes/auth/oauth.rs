use std::str::FromStr;

use crate::{
    models::auth::{AuthorizationCode, RefreshToken},
    state::oauth::{
        self, Scope,
        errors::{OAuthAuthorizeError, OAuthErrorType, OAuthTokenError},
        jwt::{create_jwt, create_refresh_jwt, validate_refresh_token},
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
) -> Result<HttpResponse, OAuthAuthorizeError> {
    query.validate()?;

    let scopes = query
        .scope
        .as_deref()
        .map(oauth::parse_scopes)
        .transpose()
        .map_err(|err| {
            OAuthAuthorizeError::redirect(err, &query.state, query.redirect_uri.clone())
        })?
        .unwrap_or_default();
    let stored_scope = if scopes.is_empty() {
        None
    } else {
        Some(oauth::scopes_to_string(&scopes))
    };

    if let Some(user) = user {
        let user_id = user.id().map_err(|err| {
            OAuthAuthorizeError::redirect(
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
            &query.client_id,
            &query.redirect_uri,
            &query.code_challenge,
            &query.code_challenge_method,
            stored_scope,
            &db,
        )
        .await?;

        let mut redirect = url::Url::parse(&query.redirect_uri)?;
        redirect
            .query_pairs_mut()
            .append_pair("code", &code)
            .append_pair("state", &query.state);

        return Ok(HttpResponse::Found()
            .insert_header((header::LOCATION, redirect.to_string()))
            .insert_header((header::REFERRER_POLICY, "no-referrer"))
            .insert_header((header::X_FRAME_OPTIONS, "DENY"))
            .insert_header((header::CONTENT_SECURITY_POLICY, "frame-ancestors 'none'"))
            .insert_header((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
            .finish());
    }

    let iss = std::env::var("SERVER_ORIGIN")?;
    let mut return_to = url::Url::parse(&iss)?.join("/auth/authorize")?;
    {
        let client_id = query.client_id.to_string();
        let mut q = return_to.query_pairs_mut();
        q.append_pair("response_type", &query.response_type);
        q.append_pair("client_id", &client_id);
        q.append_pair("redirect_uri", &query.redirect_uri);
        q.append_pair("code_challenge", &query.code_challenge);
        q.append_pair("code_challenge_method", &query.code_challenge_method);
        q.append_pair("state", &query.state);
        if let Some(scope) = &query.scope {
            q.append_pair("scope", scope);
        }
    }

    let mut login = url::Url::parse(&iss)?.join("/login")?;
    login
        .query_pairs_mut()
        .append_pair("return_to", return_to.as_str());

    let resp = HttpResponse::Found()
        .insert_header((header::REFERRER_POLICY, "no-referrer"))
        .insert_header((header::X_FRAME_OPTIONS, "DENY"))
        .insert_header((header::CONTENT_SECURITY_POLICY, "frame-ancestors 'none'"))
        .insert_header((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
        .insert_header((header::LOCATION, login.to_string()))
        .finish();

    Ok(resp)
}

#[derive(Debug, serde::Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "grant_type")]
enum OAuthTokenRequest {
    AuthorizationCode {
        client_id: uuid::Uuid,
        code: String,
        code_verifier: String,
        redirect_uri: String,
    },
    RefreshToken {
        // Optional per RFC 6749 §6: the JWT's `aud` already binds this token
        // to a client, so we only require client_id when the caller sends it,
        // and then only verify it matches the JWT.
        client_id: Option<uuid::Uuid>,
        refresh_token: String,
    },
    #[serde(other)]
    Unsupported,
}

impl OAuthTokenRequest {
    fn validate(&self) -> Result<(), OAuthTokenError> {
        match self {
            OAuthTokenRequest::AuthorizationCode {
                code,
                code_verifier,
                ..
            } => {
                if code_verifier.len() < 43 || code_verifier.len() > 128 {
                    return Err(OAuthTokenError::error(OAuthErrorType::MalformedCodeVerifier));
                }

                if code.len() != 32 {
                    return Err(OAuthTokenError::error(OAuthErrorType::MalformatedCode));
                }

                Ok(())
            }
            OAuthTokenRequest::RefreshToken { refresh_token, .. } => {
                if refresh_token.is_empty() {
                    return Err(OAuthTokenError::error(OAuthErrorType::MalformatedRefreshToken));
                }
                Ok(())
            }
            OAuthTokenRequest::Unsupported => {
                Err(OAuthTokenError::error(OAuthErrorType::UnsupportedGrantType))
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
) -> Result<HttpResponse, OAuthTokenError> {
    body.validate()?;

    match body.0 {
        OAuthTokenRequest::AuthorizationCode {
            client_id,
            code,
            code_verifier,
            redirect_uri,
        } => {
            let request = match AuthorizationCode::get_by_code(&code, &db).await? {
                Some(v) => v,
                None => return Err(OAuthTokenError::error(OAuthErrorType::AccessDenied)),
            };

            // RFC 6749 §4.1.2: if a code is presented more than once, the auth
            // server SHOULD attempt to revoke all tokens previously issued
            // based on it. The refresh-token family is keyed by grant.id, so a
            // reused code revokes every descendant refresh token.
            if request.used {
                RefreshToken::revoke_family(&request.id, &db).await?;
                return Err(OAuthTokenError::error(OAuthErrorType::AccessDenied));
            }
            if !request.is_valid() {
                return Err(OAuthTokenError::error(OAuthErrorType::AccessDenied));
            }

            // RFC 6749 §4.1.3: the auth server MUST verify the code was issued
            // to this client and that the redirect_uri matches the one used at
            // /authorize. Compare against the values stored on the grant, not
            // against any global constant.
            if request.client_id != client_id {
                return Err(OAuthTokenError::error(OAuthErrorType::InvalidClientId));
            }
            if request.redirect_uri != redirect_uri {
                return Err(OAuthTokenError::error(OAuthErrorType::InvalidRedirect));
            }

            // RFC 7636 §4.6: on PKCE mismatch, respond with `invalid_grant`.
            if !oauth::code::validate_pkce(
                &request.code_challenge,
                &code_verifier,
                &request.code_challenge_method,
            ) {
                return Err(OAuthTokenError::error(OAuthErrorType::InvalidCodeGrant));
            };

            // Family = grant.id, so any reuse of this code (see the branch
            // above) or reuse of any refresh token in the chain revokes the
            // whole family in one query.
            let family_id = request.id;

            let granted = oauth::scopes_from_stored(&request.scopes);
            let issue_refresh = granted.contains(&Scope::OfflineAccess);

            let jwt = create_jwt(request.user_id, client_id)?;

            AuthorizationCode::mark_code_used(&request.code, &db).await?;

            let refresh = if issue_refresh {
                let (refresh, refresh_jti, expires) =
                    create_refresh_jwt(client_id, request.user_id)?;
                RefreshToken::insert_token(
                    &refresh_jti,
                    &request.user_id,
                    &family_id,
                    expires,
                    &db,
                )
                .await?;
                Some(refresh)
            } else {
                None
            };

            let res = OAuthTokenResponse {
                access_token: jwt,
                refresh_token: refresh,
                token_type: "Bearer".to_string(),
                expires_in: 3600,
                scope: request.scopes,
            };

            // RFC 6749 §5.1 requires no-store + no-cache on successful token
            // responses to keep credentials out of intermediate caches.
            Ok(HttpResponse::Ok()
                .insert_header(header::CacheControl(vec![CacheDirective::NoStore]))
                .insert_header((header::PRAGMA, "no-cache"))
                .json(res))
        }
        OAuthTokenRequest::RefreshToken {
            client_id,
            refresh_token,
        } => {
            let info = validate_refresh_token(&refresh_token)?;

            // If the caller sent client_id, it must match the token's audience.
            // If they didn't, we trust the signed JWT alone (single-client
            // deployment, HS512-authenticated).
            if let Some(sent) = client_id {
                if info.claims.aud != sent {
                    return Err(OAuthTokenError::error(OAuthErrorType::InvalidClientId));
                }
            }
            let client_id = info.claims.aud;

            let token = match RefreshToken::get_token(&info.claims.jti, &db).await? {
                Some(t) => t,
                None => {
                    return Err(OAuthTokenError::error(OAuthErrorType::MissingRefreshToken));
                }
            };

            // Reuse detection: if this token has already been used or the family
            // has been revoked, treat as a compromised family and revoke every
            // token in it (OAuth 2.1 §6.1 / RFC 6819 §5.2.2.3).
            if token.used || token.revoked {
                RefreshToken::revoke_family(&token.family_id, &db).await?;
                return Err(OAuthTokenError::error(OAuthErrorType::AccessDenied));
            }

            if token.user_id != info.claims.sub {
                return Err(OAuthTokenError::error(
                    OAuthErrorType::MalformatedRefreshToken,
                ));
            }

            RefreshToken::mark_token_used(&token.id, &db).await?;

            let jwt = create_jwt(token.user_id, client_id)?;
            let (refresh, jti, expires) = create_refresh_jwt(client_id, token.user_id)?;
            RefreshToken::insert_token(&jti, &token.user_id, &token.family_id, expires, &db)
                .await?;

            let res = OAuthTokenResponse {
                access_token: jwt,
                refresh_token: Some(refresh),
                token_type: "Bearer".to_string(),
                expires_in: 3600,
                scope: None,
            };

            // RFC 6749 §5.1 requires no-store + no-cache on successful token
            // responses to keep credentials out of intermediate caches.
            Ok(HttpResponse::Ok()
                .insert_header(header::CacheControl(vec![CacheDirective::NoStore]))
                .insert_header((header::PRAGMA, "no-cache"))
                .json(res))
        }
        _ => Err(OAuthTokenError::error(OAuthErrorType::UnsupportedGrantType)),
    }
}

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct OAuthRevokeRequest {
    token: String,
    #[serde(default)]
    #[allow(dead_code)]
    token_type_hint: Option<String>,
}

/// RFC 7009 token revocation. Only refresh tokens are meaningfully revocable
/// here — access tokens are stateless short-lived JWTs. A caller presenting
/// any valid refresh-token JWT gets the whole family revoked (belt and
/// suspenders on top of reuse detection).
#[utoipa::path(
    post,
    tag = "oauth",
    path = "/auth/revoke",
    description = "revoke a refresh token (RFC 7009)",
    responses((status = OK))
)]
#[post("/revoke")]
pub async fn revoke(
    body: web::Form<OAuthRevokeRequest>,
    db: web::Data<SqlitePool>,
) -> HttpResponse {
    // RFC 7009 §2.2: the authorization server responds with HTTP 200 whether
    // or not the token was actually revoked, to avoid leaking information
    // about token validity. So we swallow every failure and always return OK.
    if let Ok(info) = validate_refresh_token(&body.token) {
        if let Ok(Some(token)) = RefreshToken::get_token(&info.claims.jti, &db).await {
            let _ = RefreshToken::revoke_family(&token.family_id, &db).await;
        }
    }

    HttpResponse::Ok()
        .insert_header(header::CacheControl(vec![CacheDirective::NoStore]))
        .insert_header((header::PRAGMA, "no-cache"))
        .finish()
}
