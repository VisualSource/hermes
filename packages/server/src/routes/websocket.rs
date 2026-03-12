use actix_web::{Error, HttpRequest, HttpResponse, rt, web};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt as _;
use prost::Message;

use crate::state::messages::{Envelope, envelope};

use crate::state::{api_errors::{ApplicationError, InnerError}, oauth::jwt::validate_jwt};

#[derive(Debug,serde::Deserialize)]
pub struct WsQuery {
    pub token: String
}

pub async fn ws(req: HttpRequest, stream: web::Payload, query: web::Query<WsQuery>) -> Result<HttpResponse, Error> {
    
    let token = match validate_jwt(&query.token){
        Ok(token)  => token,
        Err(err) => match err {
            crate::state::oauth::jwt::JwtError::Var(error) => {
            log::error!("{}",error);
            let resp = HttpResponse::InternalServerError().json(ApplicationError::new(401, "internal server error","server", Vec::default(), Some(InnerError::new(error.to_string()))));
            return Ok(resp);
        },
            crate::state::oauth::jwt::JwtError::Jwt(error) => {
            log::error!("{}",error);
            let resp = HttpResponse::Unauthorized().json(ApplicationError::new(401, "unauthorized","query", Vec::default(), Some(InnerError::new(error.to_string()))));
            return Ok(resp);
        },
        }
    };
    log::debug!("User inited socket connection: {}",token.claims.sub);
    
    
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => session
                    .text(text)
                    .await
                    .expect("failed to send text messge"),
                Ok(AggregatedMessage::Binary(bin)) => {
                    match Envelope::decode(bin) {
                        Ok(envelope) => {
                            if let Some(payload) = envelope.payload {
                                match payload {
                                    envelope::Payload::Rtc(rtc_event) => todo!(),
                                    envelope::Payload::RtcIce(rtc_new_candidate) => todo!(),
                                    envelope::Payload::VoiceChannelRequest(voice_channel_request) => todo!(),
                                    _ => {
                                        log::error!("got message that contained invalid message payload")
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            log::error!("{}",err);
                        }
                    }
                }
                Ok(AggregatedMessage::Ping(msg)) => {
                    session.pong(&msg).await.expect("failed to send pong")
                }

                _ => {}
            }
        }
    });

    Ok(res)
}
