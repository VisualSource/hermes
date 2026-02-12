use rocket::{get,launch,routes};
use rocket_ws::{WebSocket, Stream};
mod routes;

#[get("/ws")]
fn ws(ws: WebSocket) -> Stream!['static] {
    ws.stream(|id| id)
}

#[get("/")]
fn index() -> &'static str {
    "hello, World"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![
        index,
        ws,
        routes::oauth::token,
        routes::oauth::authorize,
        routes::oauth::authorize_consent,
        routes::oauth::refresh
    ])
}
