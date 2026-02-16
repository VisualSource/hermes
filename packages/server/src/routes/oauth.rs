use actix::Addr;
use actix_csrf_middleware::{CsrfToken, DEFAULT_CSRF_TOKEN_FIELD};
use actix_web::{
    HttpRequest, HttpResponse, Responder,
    dev::HttpServiceFactory,
    get,
    http::header::{self, ContentType},
    post,
    web::{self, Redirect},
};
use oxide_auth::endpoint::QueryParameter;
use oxide_auth_actix::{
    Authorize, OAuthOperation, OAuthRequest, OAuthResponse, Refresh, Token, WebError,
};
use serde::Deserialize;
use sqlx::{SqlitePool, query};

use crate::state::oauth::{Extras, OAuthState};
#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    response_type: String,
    redirect_uri: String,
    client_id: String,
}

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
#[get("/authorize")]
pub async fn authorize(
    (req, state): (OAuthRequest, web::Data<Addr<OAuthState>>),
) -> impl Responder {
    // Fetch user
    //Validate query

    state.send(Authorize(req).wrap(Extras::Get)).await?
}

#[post("/token")]
pub async fn token((req, state): (OAuthRequest, web::Data<Addr<OAuthState>>)) -> impl Responder {
    let grant_type = req.body().and_then(|body| body.unique_value("grant_type"));

    if grant_type.is_none() {
        return Err(WebError::Query);
    }

    state.send(Token(req).wrap(Extras::Nothing)).await?
}

#[post("/refresh")]
pub async fn refresh((req, state): (OAuthRequest, web::Data<Addr<OAuthState>>)) -> impl Responder {
    state.send(Refresh(req).wrap(Extras::Nothing)).await?
}

pub fn get_routes() -> impl HttpServiceFactory {
    (refresh, token, authorize, login, login_post)
}
