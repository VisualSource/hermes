use actix::Actor;
use actix_csrf_middleware::{CsrfMiddleware, CsrfMiddlewareConfig};
use actix_web::{
    App, HttpServer,
    middleware::{Logger, NormalizePath, TrailingSlash},
    web::{self, Data},
};

mod db;
mod routes;
mod state;

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
            .service(routes::index)
            .service(
                web::scope("/")
                    .wrap(CsrfMiddleware::new(csrf_config.clone()))
                    .service(routes::oauth::get_routes()),
            )
            .route("/ws", web::get().to(routes::websocket::ws))
    })
    .bind(("localhost", 7433))?
    .run()
    .await
}
