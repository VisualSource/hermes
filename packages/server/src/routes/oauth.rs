use actix::Addr;
use actix_csrf_middleware::{CsrfToken, DEFAULT_CSRF_TOKEN_FIELD};
use actix_web::{HttpRequest, HttpResponse, Responder, get, http::header::ContentType, post, web};
use oxide_auth::endpoint::QueryParameter;
use oxide_auth_actix::{
    Authorize, OAuthOperation, OAuthRequest, OAuthResponse, Refresh, Token, WebError,
};

use sqlx::SqlitePool;

use crate::state::oauth::{Extras, OAuthState};

#[utoipa::path(
    tag="oauth",
    description = "login page submition endpoint",
    request_body(
        content(
            ("application/x-www-form-urlencoded")
        )
    ),
    responses(
        (
            status = 302, 
            body = String , 
            headers(
                ("Location" = String, description = "redirect to callback uri")
            )
        )
    )
)]
#[post("/login")]
pub async fn login_post(
    req: OAuthRequest,
    r: HttpRequest,
    state: web::Data<Addr<OAuthState>>,
    _db: web::Data<SqlitePool>,
) -> Result<OAuthResponse, WebError> {
    let body = req.body().unwrap();

    let usr = body.unique_value("username");
    let psd = body.unique_value("password");

    if usr.is_none() || psd.is_none() {
        return Err(WebError::Form);
    }

    // validate
    log::debug!("{:#?}", usr);

    state
        .send(Authorize(req).wrap(Extras::Post(r.query_string().to_owned())))
        .await?
}

#[utoipa::path(
    tag="oauth", 
    description = "login page", 
    responses(
        (
            status = OK, 
            content_type="text/html", 
            body = String
        )
    )
)]
#[get("/login")]
pub async fn login(csrf: CsrfToken) -> impl Responder {
    let body = include_str!("../static/login_form.html");

    let content = body
        .replace("{CSRF_TOKEN_FIELD}", DEFAULT_CSRF_TOKEN_FIELD)
        .replace("{CSRF_TOKEN_VALUE}", &csrf.0);

    //TODO: validate client_id and redirect_uri
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(content)
}

// https://auth0.com/docs/get-started/authentication-and-authorization-flow/authorization-code-flow
#[utoipa::path(
    tag="oauth", 
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
