#[patch["/account"]]
pub async fn update_user(){}

#[get("/account")]
pub async fn get_user(){}

#[get("/account/keys")]
pub async fn get_encypt_keys(){}

#[get("/account/key")]
pub async fn get_encypt_key(){}

#[post("/account/key")]
pub async fn add_encypt_key(){}

#[delete("/account/key")]
pub async fn delete_encypt_key(){}
