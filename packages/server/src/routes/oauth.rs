use rocket::{Data, get, post};

#[get("/authorize")]
pub fn authorize<'r>() {}

#[get("/authorize?<allow>")]
pub fn authorize_consent(allow: Option<bool>) {}

#[post("/token", data = "<body>")]
pub fn token(body: Data) {}

#[post("/refresh", data = "<body>")]
pub fn refresh(body: Data) {}
