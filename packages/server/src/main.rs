use actix_web::{App, HttpServer, web};

mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(routes::index).route("/ws", web::get().to(routes::websocket::ws)))
        .bind(("localhost", 7433))?
        .run()
        .await
}
