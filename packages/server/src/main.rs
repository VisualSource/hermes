use actix::Actor;
use actix_csrf_middleware::{CsrfMiddleware, CsrfMiddlewareConfig};
use actix_governor::{Governor, GovernorConfigBuilder};
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
#[openapi(info(description = "Hermes server"), paths())]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(_err) = dotenvy::dotenv() {
        println!("Skipping loading .env file");
    }

    log4rs::init_file("./log4rs.yaml", Default::default())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;

    let oauth = state::oauth::OAuthState::preconfigured().start();

    let governor_conf = GovernorConfigBuilder::default()
        .seconds_per_request(2)
        .burst_size(5)
        .finish()
        .expect("failed to construct ratelimiter config");

    let db = db::connect().await.map_err(|err| std::io::Error::new(ErrorKind::Other, err))?;
    let pool = web::Data::new(db);

    let secert = std::env::var("APP_SECRET").expect("failed to get secert");
    let csrf_config = CsrfMiddlewareConfig::double_submit_cookie(secert.as_bytes());

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(oauth.clone()))
            .app_data(pool.clone())
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .wrap(Logger::default())
            .wrap(Governor::new(&governor_conf))
            .service(
                web::scope("")
                    .wrap(CsrfMiddleware::new(csrf_config.clone()))
                    .service(routes::static_files::get_static_files())
                    .service(routes::auth::get_account_routes()),
            )
            .service(web::scope("/auth").service(routes::auth::get_oauth_routes()))
            .service(web::scope("/api").service(routes::api::api_routes()))

        //.route("/ws", web::get().to(routes::websocket::ws))
    })
    .bind(("localhost", 7433))?
    .run()
    .await
}
