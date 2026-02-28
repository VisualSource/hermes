use actix_web::dev::HttpServiceFactory;

mod account;
mod oauth;

pub fn get_account_routes() -> impl HttpServiceFactory {
    (
        account::login,
        account::login_post,
        account::signup,
        account::signup_post,
    )
}

pub fn get_oauth_routes() -> impl HttpServiceFactory {
    (oauth::authorize, oauth::token)
}
