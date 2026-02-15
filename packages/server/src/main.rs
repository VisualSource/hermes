
use actix::Actor;
use actix_web::{App, HttpServer, middleware::{Logger, NormalizePath, TrailingSlash}, web::{self, Data}};

mod state;
mod routes;
mod db;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(_err) = dotenvy::dotenv() {
        println!("Skipping loading .env file");
    }

    log4rs::init_file("./log4rs.yaml", Default::default())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;

    let oauth = state::oauth::OAuthState::preconfigured().start();
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(oauth.clone()))
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .wrap(Logger::default())


            .service(routes::index)
            .service(routes::oauth::get_routes())
     
            .route("/ws", web::get().to(routes::websocket::ws))
    })
    .bind(("localhost", 7433))?
    .run().await
}
