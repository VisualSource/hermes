use crate::{
    models,
    routes::error::{ApplicationError, AuthPageError, ErrorDetail, InnerError},
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

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct LoginResponse {
    redirect: String,
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
    fn verify_payload(&self) -> Result<(), AuthPageError> {
        let mut errors = Vec::<ErrorDetail>::new();

        let psd_len = self.password.len();
        let usr_len = self.username.len();

        if psd_len < 8 {
            errors.push(ErrorDetail::new(0, "password", "password is too short"));
        }

        if psd_len > 384 {
            errors.push(ErrorDetail::new(1, "password", "password is too long"));
        }

        if usr_len < 4 {
            errors.push(ErrorDetail::new(2, "username", "username is too short"));
        }

        if usr_len > 255 {
            errors.push(ErrorDetail::new(2, "username", "username is too long"));
        }

        if errors.len() > 0 {
            return Err(AuthPageError::Request(ApplicationError::new(
                StatusCode::BAD_REQUEST.as_u16(),
                "one or more field failed to validate",
                "body",
                errors,
                None,
            )));
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
    form.verify_payload()?;

    if let Err(err) = recaptcha::get_recaptcha_assessment(&form.recaptcha, "login").await {
        match err {
            RecaptchaError::FailedAssessment => return Err(AuthPageError::Recaptcha),
            RecaptchaError::MissingEnv(err) => {
                log::error!("{}", err);
                return Err(AuthPageError::Request(ApplicationError::new(
                    StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    "internal server error",
                    "server",
                    Vec::default(),
                    Some(InnerError::new(err.to_string())),
                )));
            }
        }
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

    let mut payload = actix_web::dev::Payload::None;
    let request = OAuthRequest::from_request(&req, &mut payload).await?;

    let response = state
        .send(Authorize(request).wrap(Extras::Post(user.id)))
        .await??;

    let headers = response.get_headers();
    let header = match headers.get("location") {
        None => {
            return Err(AuthPageError::Request(ApplicationError::new(
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                "internal server error",
                "server",
                Vec::default(),
                Some(InnerError::new(
                    "Failed to get location header from oauth response".to_string(),
                )),
            )));
        }
        Some(header) => header,
    };

    let value = match header.to_str() {
        Ok(v) => v,
        Err(err) => {
            log::error!("{}", err);
            return Err(AuthPageError::Request(ApplicationError::new(
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                "internal server error",
                "server",
                Vec::default(),
                None,
            )));
        }
    };

    Ok(HttpResponse::Ok().json(LoginResponse {
        redirect: value.to_string(),
    }))
}

#[derive(Debug, serde::Deserialize, ToSchema)]
struct SignupFormRequest {
    username: String,
    password: String,
    email: String,
    recaptcha: String,
}

impl SignupFormRequest {
    fn verify_fields(&self) -> Result<(), AuthPageError> {
        let mut errors = Vec::with_capacity(5);
        let usr_len = self.username.len();
        let psd_len = self.password.len();

        if usr_len < 4 {
            errors.push(ErrorDetail::new(401, "username", "username is too short"));
        }
        if usr_len > 255 {
            errors.push(ErrorDetail::new(402, "username", "username is too long"));
        }

        if psd_len < 8 {
            errors.push(ErrorDetail::new(403, "password", "password is too short"));
        }

        if psd_len > 384 {
            errors.push(ErrorDetail::new(403, "password", "password is too long"));
        }

        if errors.len() > 0 {
            return Err(AuthPageError::Request(ApplicationError::new(
                StatusCode::BAD_REQUEST.as_u16(),
                "one or more fields failed validation",
                "body",
                errors,
                None,
            )));
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
    form.verify_fields()?;

    if let Err(err) = recaptcha::get_recaptcha_assessment(&form.recaptcha, "signup").await {
        match err {
            RecaptchaError::FailedAssessment => return Err(AuthPageError::Recaptcha),
            RecaptchaError::MissingEnv(err) => {
                log::error!("{}", err);
                return Err(AuthPageError::Request(ApplicationError::new(
                    StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    "internal server error",
                    "server",
                    Vec::default(),
                    Some(InnerError::new(err.to_string())),
                )));
            }
        }
    }

    match models::user::User::user_already_exists(&form.username, &form.email, &db).await {
        Ok(false) => {}
        Ok(true) => {
            return Err(AuthPageError::Request(ApplicationError::new(
                StatusCode::BAD_REQUEST.as_u16(),
                "user already exists",
                "body",
                vec![
                    ErrorDetail::new(405, "username", "a user with given username already exists"),
                    ErrorDetail::new(406, "email", "a user with given email already exists"),
                ],
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

    if req.query_string().len() != 0 {
        let mut payload = actix_web::dev::Payload::None;
        let request = OAuthRequest::from_request(&req, &mut payload).await?;

        let result = state
            .send(Authorize(request).wrap(Extras::Post(uuid)))
            .await??;

        let header = result.get_headers();

        let location = match header.get("location") {
            None => {
                return Err(AuthPageError::Request(ApplicationError::new(
                    StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    "internal server error",
                    "server",
                    Vec::default(),
                    Some(InnerError::new(
                        "failed to get location header from oauth response".to_string(),
                    )),
                )));
            }
            Some(loc) => loc,
        };

        let value = match location.to_str() {
            Ok(value) => value,
            Err(err) => {
                log::error!("{}", err);
                return Err(AuthPageError::Request(ApplicationError::new(
                    StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    "internal server error",
                    "server",
                    Vec::default(),
                    Some(InnerError::new(err.to_string())),
                )));
            }
        };

        return Ok(HttpResponse::Ok().json(LoginResponse {
            redirect: value.to_owned(),
        }));
    }

    Ok(HttpResponse::Created().finish())
}
