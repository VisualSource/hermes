use actix_governor::{Governor, GovernorConfigBuilder};
use actix_identity::IdentityMiddleware;
use actix_session::SessionMiddleware;
use actix_cors::Cors;
use actix_web::{
    App, HttpServer,
    middleware::{Logger, NormalizePath, TrailingSlash},
    web::{self},
};
use std::io::ErrorKind;
use std::time::Duration;
use utoipa::OpenApi;

mod db;
mod models;
mod routes;
mod state;

#[derive(OpenApi)]
#[openapi(info(description = "Hermes server"), paths())]
struct ApiDoc;

/// Verify security-critical env vars before we start listening. Any of these
/// missing or too weak means we refuse to boot, rather than fail late with
/// mysterious auth errors.
fn validate_env() -> Result<(), String> {
    if std::env::var("SERVER_ORIGIN").is_err() {
        return Err("SERVER_ORIGIN is not set".to_string());
    }

    if std::env::var("JWT_PRIVATE_KEY_PATH").is_err() {
        return Err(
            "JWT_PRIVATE_KEY_PATH is not set (path to a PKCS#8 PEM Ed25519 private key; \
             generate one with: `openssl genpkey -algorithm ed25519 -out ed25519.pem`)"
                .to_string(),
        );
    }

    if std::env::var("SESSION_SECRET").is_err() {
        return Err(
            "SESSION_SECRET is not set (must be a 64-byte hex or base64 string)".to_string(),
        );
    }

    Ok(())
}

fn load_session_key() -> Result<actix_web::cookie::Key, String> {
    let raw = std::env::var("SESSION_SECRET")
        .map_err(|_| "SESSION_SECRET is not set".to_string())?;
    // Accept either raw bytes (>=64 chars), base64, or hex. Simplest: require
    // raw string of at least 64 bytes.
    if raw.as_bytes().len() < 64 {
        return Err(format!(
            "SESSION_SECRET must be at least 64 bytes (got {})",
            raw.as_bytes().len()
        ));
    }
    Ok(actix_web::cookie::Key::from(raw.as_bytes()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(_err) = dotenvy::dotenv() {
        println!("Skipping loading .env file");
    }

    log4rs::init_file("./log4rs.yaml", Default::default())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;

    if let Err(err) = validate_env() {
        log::error!("startup validation failed: {}", err);
        return Err(std::io::Error::new(ErrorKind::Other, err));
    }

    // Load Ed25519 signing key + derived public key + kid into a process-wide
    // OnceLock. This is the single source of truth for JWT signing/verification
    // and for the /.well-known/jwks.json endpoint.
    let key_path = std::env::var("JWT_PRIVATE_KEY_PATH")
        .expect("JWT_PRIVATE_KEY_PATH validated above");
    if let Err(err) = state::oauth::jwt::init_keys_from_path(&key_path) {
        log::error!("failed to load JWT signing key from {}: {}", key_path, err);
        return Err(std::io::Error::new(ErrorKind::Other, err.to_string()));
    }

    // Default (read-mostly) governor: 5 burst, 1 request/2s. Applies to all
    // routes that aren't wrapped by a tighter governor below.
    let default_governor = GovernorConfigBuilder::default()
        .seconds_per_request(2)
        .burst_size(5)
        .finish()
        .expect("failed to construct default rate limiter");

    // Auth governor: covers /auth/login, /auth/signup, /auth/token, /auth/revoke.
    // Tighter to slow credential-stuffing and token brute-force. 3 burst, 1/5s.
    let auth_governor = GovernorConfigBuilder::default()
        .seconds_per_request(5)
        .burst_size(3)
        .finish()
        .expect("failed to construct auth rate limiter");

    let db = db::connect()
        .await
        .map_err(|err| std::io::Error::new(ErrorKind::Other, err))?;
    let pool = web::Data::new(db);

    let session_key = load_session_key()
        .map_err(|err| std::io::Error::new(ErrorKind::Other, err))?;

    // Periodic cleanup: expire old grants + prune expired refresh tokens.
    // Runs hourly; each failure is logged but doesn't take down the server.
    let cleanup_pool = pool.clone();
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(3600));
        loop {
            ticker.tick().await;
            if let Err(err) = models::auth::AuthorizationCode::remove_expired(&cleanup_pool).await {
                log::error!("grants cleanup failed: {}", err);
            }
            if let Err(err) = models::auth::RefreshToken::remove_expired(&cleanup_pool).await {
                log::error!("refresh token cleanup failed: {}", err);
            }
        }
    });

    let cors_origin = std::env::var("CORS_ALLOWED_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:1420".to_string());

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
            .allowed_origin(&cors_origin);

        App::new()
            .app_data(pool.clone())
            .wrap(cors)
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .wrap(Logger::default())
            .wrap(Governor::new(&default_governor))
            .wrap(IdentityMiddleware::default())
            .wrap(SessionMiddleware::new(
                actix_session::storage::CookieSessionStore::default(),
                session_key.clone(),
            ))
            .service(routes::oauth_server_details)
            .service(routes::jwks)
            .service(
                web::scope("/auth")
                    .wrap(Governor::new(&auth_governor))
                    .service(routes::auth::get_oauth_routes()),
            )
            .service(
                web::scope("/api")
                    .route("/ws", web::get().to(routes::websocket::ws))
                    .service(routes::api::api_routes()),
            )
            // Login/signup are at the root path (`/login`, `/signup`) because
            // the authorize handler redirects there. They also get the
            // auth-tightened rate limiter to slow credential brute-forcing.
            .service(
                web::scope("")
                    .wrap(Governor::new(&auth_governor))
                    .service(routes::auth::get_account_routes()),
            )
            .service(routes::static_files::get_static_files())
    })
    .bind(("0.0.0.0", 7433))?
    .run()
    .await
}
