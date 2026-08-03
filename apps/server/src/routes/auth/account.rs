use crate::{
    models,
    routes::error::AuthPageError,
    state::{
        api_errors::{ApplicationError, ErrorDetail, InnerError},
        password::{hash_password, verify_password},
        turnstile,
    },
};
use actix_csrf_middleware::{CsrfToken, DEFAULT_CSRF_TOKEN_FIELD};
use actix_identity::Identity;
use actix_web::{
    HttpMessage, HttpRequest, HttpResponse, Responder, get,
    http::{StatusCode, header::ContentType},
    post, web,
};

use actix_web_validation::Validated;
use sqlx::SqlitePool;
use std::{env, io::Read};
use utoipa::ToSchema;
use validator::Validate;

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
    let mut file = actix_files::NamedFile::open_async("./public/login.html").await?;

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

#[derive(Debug, serde::Deserialize, ToSchema, Validate)]
struct LoginRequest {
    #[validate(length(min = 4, max = 255))]
    username: String,
    #[validate(length(min = 8, max = 384))]
    password: String,
    #[validate(length(max = 2048))]
    cf_token: String,
}

#[utoipa::path(
    tag="oauth",
    description = "login page submission endpoint",
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
    Validated(form): Validated<web::Form<LoginRequest>>,
    db: web::Data<SqlitePool>,
) -> Result<impl Responder, AuthPageError> {
    let info = req.connection_info();
    let remote_ip = info.realip_remote_addr().ok_or_else(|| {
        AuthPageError::Request(ApplicationError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal server error",
            "server",
            Vec::default(),
            Some(InnerError::new("failed to fetch remote ip".to_string())),
        ))
    })?;

    let result = turnstile::validate_token(remote_ip, &form.cf_token, "login")
        .await
        .map_err(|err| {
            AuthPageError::Request(ApplicationError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error",
                "server",
                Vec::default(),
                Some(InnerError::new(err.to_string())),
            ))
        })?;
    if !result.is_success() {
        let errs = result
            .get_errors()
            .iter()
            .map(|reason| ErrorDetail::new(4001, "request", reason))
            .collect::<Vec<ErrorDetail>>();
        return Err(AuthPageError::Request(ApplicationError::new(
            StatusCode::FORBIDDEN,
            "turnstile error",
            "request",
            errs,
            None,
        )));
    }

    let user = match models::user::User::find_by_username(&form.username, &db).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err(AuthPageError::Request(ApplicationError::new(
                StatusCode::BAD_REQUEST.as_u16(),
                "username or password is invalid",
                "body",
                vec![ErrorDetail::new(
                    404,
                    "body",
                    "invalid username or password",
                )],
                None,
            )));
        }
        Err(err) => {
            log::error!("{}", err);
            return Err(AuthPageError::DbError(err));
        }
    };

    match verify_password(&form.password, &user.psd_hash) {
        Ok(true) => {}
        Ok(false) => {
            return Err(AuthPageError::Request(ApplicationError::new(
                StatusCode::BAD_REQUEST.as_u16(),
                "username or password is invalid",
                "body",
                vec![ErrorDetail::new(
                    404,
                    "body",
                    "invalid username or password",
                )],
                None,
            )));
        }
        Err(err) => {
            log::error!("{}", err);
            return Err(AuthPageError::Argon(err.to_string()));
        }
    }

    Identity::login(&req.extensions(), user.id.into())?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, serde::Deserialize, ToSchema, Validate)]
struct SignupFormRequest {
    #[validate(length(min = 4, max = 255))]
    username: String,
    #[validate(length(min = 8, max = 255))]
    password: String,
    #[validate(email)]
    email: String,
    #[validate(length(max = 2048))]
    cf_token: String,
}

#[utoipa::path(description = "Signup endpoint")]
#[post("/signup")]
pub async fn signup_post(
    req: HttpRequest,
    form: Validated<web::Form<SignupFormRequest>>,
    db: web::Data<SqlitePool>,
) -> Result<HttpResponse, AuthPageError> {
    let info = req.connection_info();
    let remote_ip = info.realip_remote_addr().ok_or_else(|| {
        AuthPageError::Request(ApplicationError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal server error",
            "server",
            Vec::default(),
            Some(InnerError::new("failed to fetch remote ip".to_string())),
        ))
    })?;

    let result = turnstile::validate_token(remote_ip, &form.cf_token, "signup")
        .await
        .map_err(|err| {
            AuthPageError::Request(ApplicationError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error",
                "server",
                Vec::default(),
                Some(InnerError::new(err.to_string())),
            ))
        })?;
    if !result.is_success() {
        let errs = result
            .get_errors()
            .iter()
            .map(|reason| ErrorDetail::new(4001, "request", reason))
            .collect::<Vec<ErrorDetail>>();
        return Err(AuthPageError::Request(ApplicationError::new(
            StatusCode::FORBIDDEN,
            "turnstile error",
            "request",
            errs,
            None,
        )));
    }

    match models::user::User::user_already_exists(&form.username, &form.email, &db).await {
        Ok(false) => {}
        Ok(true) => {
            return Err(AuthPageError::Request(ApplicationError::new(
                StatusCode::BAD_REQUEST.as_u16(),
                "user already exists",
                "body",
                vec![ErrorDetail::new(
                    405,
                    "body",
                    "a user with given username or email already exists",
                )],
                None,
            )));
        }
        Err(err) => {
            log::error!("{}", err);
            return Err(AuthPageError::DbError(err));
        }
    }

    let hash = match hash_password(&form.password) {
        Ok(h) => h,
        Err(err) => {
            log::error!("{}", err);
            return Err(AuthPageError::Argon(err.to_string()));
        }
    };

    let avatar = format!(
        "https://api.dicebear.com/9.x/rings/svg?seed={}&backgroundType=gradientLinear&backgroundColor=b6e3f4,c0aede,d1d4f9",
        form.username
    );

    let uuid =
        match models::user::User::insert_user(&form.username, &form.email, &avatar, &hash, &db)
            .await
        {
            Err(err) => {
                log::error!("{}", err);
                return Err(AuthPageError::DbError(err));
            }
            Ok(u) => u,
        };

    Identity::login(&req.extensions(), uuid.into())?;

    Ok(HttpResponse::Created().finish())
}
