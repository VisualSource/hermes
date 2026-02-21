use actix::Actor;
use actix_csrf_middleware::{CsrfMiddleware, CsrfMiddlewareConfig};
use actix_web::{
    App, HttpServer,
    middleware::{Logger, NormalizePath, TrailingSlash},
    web::{self, Data},
};
use utoipa::OpenApi;

mod db;
mod models;
mod routes;
mod state;

#[derive(OpenApi)]
#[openapi(
    info(description = "Hermes server"),
    paths(
        routes::oauth::login,
        routes::oauth::login_post,
        routes::oauth::token,
        routes::oauth::authorize,
        routes::oauth::refresh,
        routes::oauth::signup,
    )
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(_err) = dotenvy::dotenv() {
        println!("Skipping loading .env file");
    }

    log4rs::init_file("./log4rs.yaml", Default::default())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;

    let oauth = state::oauth::OAuthState::preconfigured().start();

    let db = db::connect().await?;
    let pool = web::Data::new(db);

    let secert = std::env::var("APP_SECRET").expect("failed to get secert");
    let csrf_config = CsrfMiddlewareConfig::double_submit_cookie(secert.as_bytes());

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(oauth.clone()))
            .app_data(pool.clone())
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .wrap(Logger::default())
            .wrap(CsrfMiddleware::new(csrf_config.clone()))
            .service(routes::oauth::login)
            .service(routes::oauth::login_post)
            .service(routes::oauth::signup)
            .service(routes::oauth::signup_post)
            .service(routes::oauth::refresh)
            .service(routes::oauth::token)
            .service(routes::oauth::authorize)
            .service(routes::static_files::get_static_files())
            .service(web::scope("/api").service(routes::api::api_routes()))

        //.route("/ws", web::get().to(routes::websocket::ws))
    })
    .bind(("localhost", 7433))?
    .run()
    .await
}
