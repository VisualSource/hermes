use crate::{
    models::{self},
    routes::error::AuthPageError,
    state::{
        oauth::{Extras, OAuthState},
        password::{hash_password, verify_password},
        recaptcha::{self, RecaptchaError},
    },
};
use actix::Addr;
use actix_csrf_middleware::{CsrfToken, DEFAULT_CSRF_TOKEN_FIELD};
use actix_web::{
    FromRequest, HttpRequest, HttpResponse, Responder, get,
    http::{StatusCode, header::ContentType},
    post, web,
};
use oxide_auth_actix::{Authorize, OAuthOperation, OAuthRequest};
use sqlx::SqlitePool;
use std::{env, io::Read};
use utoipa::ToSchema;

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

#[derive(Debug, serde::Deserialize, ToSchema)]
struct LoginRequest {
    username: String,
    password: String,
    recaptcha: String,
}

impl LoginRequest {
    fn verify_payload(&self) -> Result<(), Vec<(String, isize)>> {
        let mut errors = Vec::new();

        let psd_len = self.password.len();
        let usr_len = self.username.len();

        if psd_len < 8 || psd_len > 384 {
            errors.push(("password".to_string(), 0));
        }

        if usr_len < 4 || usr_len > 255 {
            errors.push(("username".to_string(), 0));
        }

        if errors.len() > 0 {
            return Err(errors);
        }

        Ok(())
    }
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
) -> Result<impl Responder, AuthPageError> {
    if let Err(errors) = form.verify_payload() {
        return Err(AuthPageError::InvalidFormData(errors));
    }

    match recaptcha::get_recaptcha_assessment(&form.recaptcha, "login").await {
        Err(RecaptchaError::FailedAssessment) => return Err(AuthPageError::Recaptcha),
        Err(RecaptchaError::MissingEnv(err)) => {
            log::error!("{}", err);
            return Err(AuthPageError::Custom(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_string(),
            ));
        }
        Ok(_) => {}
    }

    let user = match models::user::User::find_by_username(&form.username, &db).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err(AuthPageError::InvalidFormData(vec![(
                "invalid_psd_or_usr".to_string(),
                0,
            )]));
        }
        Err(err) => {
            log::error!("{}", err);
            return Err(AuthPageError::DbError(err));
        }
    };

    match verify_password(&form.password, &user.psd_hash) {
        Ok(_) => {}
        Ok(false) => {
            return Err(AuthPageError::InvalidFormData(vec![(
                "invalid_psd_or_usr".to_string(),
                0,
            )]));
        }
        Err(err) => {
            log::error!("{}", err);
            return Err(AuthPageError::Argon);
        }
    }

    let mut payload = actix_web::dev::Payload::None;
    let request = OAuthRequest::from_request(&req, &mut payload).await?;

    let response = state
        .send(Authorize(request).wrap(Extras::Post(user.id)))
        .await??;

    let headers = response.get_headers();

    let header = match headers.get("location") {
        None => {
            return Err(AuthPageError::Custom(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get redirect uri".to_string(),
            ));
        }
        Some(header) => header,
    };

    let value = match header.to_str() {
        Ok(v) => v,
        Err(err) => {
            log::error!("{}", err);
            return Err(AuthPageError::Custom(
                StatusCode::INTERNAL_SERVER_ERROR,
                "convertion error".to_string(),
            ));
        }
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "redirect": value
    })))
}

#[derive(Debug, serde::Deserialize, ToSchema)]
struct SignupFormRequest {
    username: String,
    password: String,
    email: String,
    recaptcha: String,
}

impl SignupFormRequest {
    fn verify_fields(&self) -> Result<(), Vec<(String, isize)>> {
        let mut errors = Vec::with_capacity(5);
        let usr_len = self.username.len();
        let psd_len = self.password.len();

        if usr_len < 4 || usr_len > 255 {
            errors.push(("username".to_string(), 0));
        }

        if psd_len < 8 || psd_len > 384 {
            errors.push(("password".to_string(), 0));
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
    state: web::Data<Addr<OAuthState>>,
) -> impl Responder {
    if let Err(err) = form.verify_fields() {
        return Err(AuthPageError::InvalidFormData(err));
    }

    match recaptcha::get_recaptcha_assessment(&form.recaptcha, "signup").await {
        Err(RecaptchaError::FailedAssessment) => return Err(AuthPageError::Recaptcha),
        Err(err) => {
            log::error!("{}", err);
            return Err(AuthPageError::Custom(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_string(),
            ));
        }
        Ok(_) => {}
    }

    match models::user::User::user_already_exists(&form.username, &form.email, &db).await {
        Ok(false) => {}
        Ok(true) => {
            return Err(AuthPageError::InvalidFormData(vec![(
                "username_email_already_exists".to_string(),
                0,
            )]));
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
            return Err(AuthPageError::Argon);
        }
    };

    let uuid =
        match models::user::User::insert_user(&form.username, &form.email, "", &hash, &db).await {
            Err(err) => {
                log::error!("{}", err);
                return Err(AuthPageError::DbError(err));
            }
            Ok(u) => u,
        };

    if req.query_string().len() != 0 {
        let mut payload = actix_web::dev::Payload::None;
        let request = OAuthRequest::from_request(&req, &mut payload).await?;

        let result = state
            .send(Authorize(request).wrap(Extras::Post(uuid)))
            .await??;

        let header = result.get_headers();

        let location = match header.get("location") {
            None => {
                return Err(AuthPageError::Custom(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                ));
            }
            Some(loc) => loc,
        };

        let value = match location.to_str() {
            Ok(value) => value,
            Err(err) => {
                log::error!("{}", err);
                return Err(AuthPageError::Custom(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                ));
            }
        };

        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "redirect": value
        })));
    }

    Ok(HttpResponse::Created().finish())
}
