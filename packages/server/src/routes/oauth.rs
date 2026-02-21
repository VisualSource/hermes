use std::{env, io::Read};

use crate::{
    models,
    routes::error::ApiError,
    state::{password::verify_password, recaptcha},
};
use actix::Addr;
use actix_csrf_middleware::{CsrfToken, DEFAULT_CSRF_TOKEN_FIELD};
use actix_web::{
    FromRequest, HttpRequest, HttpResponse, Responder, get,
    http::header::{self, ContentType},
    post,
    web::{self},
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use oxide_auth::endpoint::QueryParameter;
use oxide_auth_actix::{Authorize, OAuthOperation, OAuthRequest, Refresh, Token, WebError};
use utoipa::ToSchema;

use sqlx::SqlitePool;

use crate::state::oauth::{Extras, OAuthState};

#[derive(Debug, serde::Deserialize, ToSchema)]
struct LoginRequest {
    username: String,
    password: String,
}

#[utoipa::path(
    tag="oauth",
    description = "login page submition endpoint",
    request_body(
        content(
            ("application/x-www-form-urlencoded",)
        )
    ),
    responses(
        (
            status = 302,
            headers(
                ("Location" = String, description = "redirect to callback uri")
            )
        )
    )
)]
#[post("/login")]
pub async fn login_post(
    req: HttpRequest,
    form: web::Form<LoginRequest>,
    state: web::Data<Addr<OAuthState>>,
    db: web::Data<SqlitePool>,
) -> Result<impl Responder, ApiError> {
    if form.username.len() < 4 || form.password.len() < 8 {
        return Err(ApiError::BadRequest);
    }

    let user = models::user::User::find_by_username(&form.username, &db)
        .await
        .or_else(|err| {
            log::error!("{}", err);
            Err(ApiError::InternalError)
        })?
        .ok_or_else(|| ApiError::BadRequest)?;

    let hashed_pad = PasswordHash::new(&user.psd_hash);
    if let Err(err) = hashed_pad {
        log::error!("{}", err);
        return Err(ApiError::InternalError);
    }
    let hashed_psd = hashed_pad.expect("failed to get hashed password");

    if let Err(err) = Argon2::default().verify_password(form.password.as_bytes(), &hashed_psd) {
        log::error!("{}", err);
        return Err(ApiError::BadRequest);
    }

    let mut payload = actix_web::dev::Payload::None;
    let request = OAuthRequest::from_request(&req, &mut payload)
        .await
        .map_err(|err| {
            log::error!("{}", err);
            ApiError::InternalError
        })?;
    state
        .send(Authorize(request).wrap(Extras::Post(req.query_string().to_owned())))
        .await
        .map_err(|err| {
            log::error!("{}", err);
            ApiError::InternalError
        })
}

#[derive(Debug, serde::Deserialize, ToSchema)]
struct SignupFormRequest {
    username: String,
    password: String,
    recaptcha: String,
    csrf_token: String,
}

impl SignupFormRequest {
    fn verify_fields(&self) -> Result<(), Vec<(&str, isize)>> {
        let mut errors = Vec::with_capacity(5);
        let usr_len = self.username.len();
        let psd_len = self.password.len();

        if usr_len < 4 || usr_len > 255 {
            errors.push(("username", 0));
        }

        if psd_len < 8 || psd_len > 384 {
            errors.push(("password", 0));
        }

        if errors.len() > 0 {
            return Err(errors);
        }

        Ok(())
    }
}

#[utoipa::path(description = "Signup endpoint")]
#[post("/signup")]
pub async fn signup_post(
    req: HttpRequest,
    form: web::Form<SignupFormRequest>,
    db: web::Data<SqlitePool>,
) -> impl Responder {
    let errors = form.verify_fields();

    match recaptcha::get_recaptcha_assessment(&form.recaptcha, "signup").await {
        Err(_) => {}
        Ok(_) => {}
    }

    match models::user::User::find_by_username(&form.username, &db).await {
        Ok(Some(user)) => match verify_password(&form.password, &user.psd_hash) {
            Ok(_) => {}
            Err(err) => {
                log::error!("{}", err);
            }
        },
        Ok(None) => {}
        Err(err) => {
            log::error!("{}", err);
        }
    }

    //TODO: check for redirect query
    // to auth provider

    HttpResponse::Created().finish()
}

#[utoipa::path(
    tag = "oauth",
    description = "Signup page",
    responses(
        (status = OK, content_type = "text/html", body = String)
    )
)]
#[get("/signup")]
pub async fn signup(csrf: CsrfToken) -> std::io::Result<impl Responder> {
    let site_key = env::var("RECAPTCHA_SITE_KEY");
    if let Err(err) = site_key {
        log::error!("{}", err);
        return Ok(HttpResponse::InternalServerError().finish());
    }
    let site_key = site_key.unwrap_or_default();

    let mut file = actix_files::NamedFile::open_async("./public/signup.html").await?;

    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;

    let content = buffer
        .replace("{CSRF_TOKEN_FIELD}", DEFAULT_CSRF_TOKEN_FIELD)
        .replace("{CSRF_TOKEN_VALUE}", &csrf.0)
        .replace("{RECAPTCHA_SITE_KEY}", &site_key);

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(content))
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
pub async fn login(csrf: CsrfToken) -> std::io::Result<impl Responder> {
    let site_key = env::var("RECAPTCHA_SITE_KEY");
    if let Err(err) = site_key {
        log::error!("{}", err);
        return Ok(HttpResponse::InternalServerError().finish());
    }
    let site_key = site_key.unwrap_or_default();
       let mut file = actix_files::NamedFile::open_async("./public/signup.html").await?;

    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;

    let content = buffer
        .replace("{CSRF_TOKEN_FIELD}", DEFAULT_CSRF_TOKEN_FIELD)
        .replace("{CSRF_TOKEN_VALUE}", &csrf.0)
        .replace("{RECAPTCHA_SITE_KEY}", &site_key);

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(content))
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
