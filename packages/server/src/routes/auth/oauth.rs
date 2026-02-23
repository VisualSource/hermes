use actix::Addr;
use actix_web::{Responder, get, post, web};
use oxide_auth::endpoint::QueryParameter;
use oxide_auth_actix::{Authorize, OAuthOperation, OAuthRequest, Refresh, Token, WebError};

use crate::state::oauth::{Extras, OAuthState};

// https://auth0.com/docs/get-started/authentication-and-authorization-flow/authorization-code-flow
#[utoipa::path(
    tag="oauth",
    path = "/auth/authorize"
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
    (req, state): (OAuthRequest, web::Data<Addr<OAuthState>>),
) -> impl Responder {
    // Fetch user
    //Validate query

    state.send(Authorize(req).wrap(Extras::Get)).await?
}

#[utoipa::path(
    tag="oauth",
    path = "/auth/token"
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
pub async fn token((req, state): (OAuthRequest, web::Data<Addr<OAuthState>>)) -> impl Responder {
    let grant_type = req.body().and_then(|body| body.unique_value("grant_type"));

    if grant_type.is_none() {
        return Err(WebError::Query);
    }

    state.send(Token(req).wrap(Extras::Nothing)).await?
}

#[utoipa::path(
    tag = "oauth", 
    path = "/auth/refresh"
    description = "refresh a access_token using a refresh token", 
    responses(
        (
            status = OK,
            content_type="application/json",
            body = String
        )
    )
)]
#[post("/refresh")]
pub async fn refresh((req, state): (OAuthRequest, web::Data<Addr<OAuthState>>)) -> impl Responder {
    state.send(Refresh(req).wrap(Extras::Nothing)).await?
}
