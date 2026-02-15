use actix::Addr;
use actix_web::{ HttpMessage, HttpRequest, HttpResponse, Responder, dev::HttpServiceFactory, get, post, web};
use oxide_auth::endpoint::{QueryParameter, };
use oxide_auth_actix::{Authorize, OAuthOperation, OAuthRequest, Refresh, Token, WebError};

use crate::state::oauth::{Extras, OAuthState};


#[post("/login")]
pub async fn login_post((req): (HttpRequest)) -> impl Responder {

    //TODO: validate client_id and redirect_uri

    //TODO: send response_code

    
    HttpResponse::TemporaryRedirect()
}

#[get("/login")]
pub async fn login() -> impl Responder {

    //TODO: validate client_id and redirect_uri

    let body = r#"
        <!DOCTYPE html>
        <html>
            <head>
                <title>Hermes Login</title>
            </head>
            <body>
               <h1>Hermes Login</h1>
                <form method="post">
                    <label>Username</label>
                    <input type="text" name="username"/>
                    <label>Password</label>
                    <input type="password" name="psd"/>
                    <button type="submit">Login</button>
                </form>
               <span>Don't have an account client <a href="/signup">click here</a></span>
            </body>
        </html>
    "#;

    HttpResponse::Ok().content_type("text/html").body(body)
}


// https://auth0.com/docs/get-started/authentication-and-authorization-flow/authorization-code-flow
#[get("/authorize")]
pub async fn authorize((req, state): ( OAuthRequest, web::Data<Addr<OAuthState>>)) -> impl Responder {

   /**
    * Fetch user
    * Validate query
    * 
    */


    state.send(Authorize(req).wrap(Extras::Get)).await?
}

#[post("/token")]
pub async fn token((req, state): (OAuthRequest,web::Data<Addr<OAuthState>>)) -> impl Responder {
    let grant_type = req.body().and_then(|body| body.unique_value("grant_type"));

    if grant_type.is_none() {
        return Err(WebError::Query)
    }

    state.send(Token(req).wrap(Extras::Nothing)).await?
}

#[post("/refresh")]
pub async fn refresh((req, state): (OAuthRequest, web::Data<Addr<OAuthState>>)) -> impl Responder{
    state.send(Refresh(req).wrap(Extras::Nothing)).await?
}

pub fn get_routes() -> impl HttpServiceFactory {
    (refresh,token,authorize,login)
}


// authorize -> login -> callback -> token