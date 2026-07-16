use actix_web::http::header::{self, HeaderValue};
use actix_web::{Error, HttpRequest, HttpResponse, rt, web};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt as _;
use prost::Message;

use crate::state::messages::{Envelope, envelope};

use crate::state::{
    api_errors::{ApplicationError, InnerError},
    oauth::jwt::validate_jwt,
};

/// Subprotocol name the client offers alongside the bearer token, per the
/// pattern `Sec-WebSocket-Protocol: bearer, <jwt>`. Keeps the JWT off the URL
/// query so it never lands in access logs, Referer headers, or proxy caches.
const BEARER_SUBPROTOCOL: &str = "bearer";

/// Parse `Sec-WebSocket-Protocol: bearer, <jwt>` into the JWT string. Returns
/// `None` if the header is missing or the shape doesn't match.
fn extract_bearer_token(req: &HttpRequest) -> Option<String> {
    let header = req.headers().get(header::SEC_WEBSOCKET_PROTOCOL)?;
    let raw = header.to_str().ok()?;

    let mut parts = raw.split(',').map(str::trim);
    let scheme = parts.next()?;
    let token = parts.next()?;

    if scheme != BEARER_SUBPROTOCOL || token.is_empty() || parts.next().is_some() {
        return None;
    }

    Some(token.to_string())
}

pub async fn ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let raw_token = match extract_bearer_token(&req) {
        Some(t) => t,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApplicationError::new(
                401,
                "unauthorized",
                "sec-websocket-protocol",
                Vec::default(),
                Some(InnerError::new(
                    "expected `Sec-WebSocket-Protocol: bearer, <jwt>`".to_string(),
                )),
            )));
        }
    };

    let token = match validate_jwt(&raw_token) {
        Ok(token) => token,
        Err(err) => match err {
            crate::state::oauth::jwt::JwtError::Jwt(error) => {
                log::error!("{}", error);
                let resp = HttpResponse::Unauthorized().json(ApplicationError::new(
                    401,
                    "unauthorized",
                    "sec-websocket-protocol",
                    Vec::default(),
                    Some(InnerError::new(error.to_string())),
                ));
                return Ok(resp);
            }
            other => {
                log::error!("{}", other);
                let resp = HttpResponse::InternalServerError().json(ApplicationError::new(
                    500,
                    "internal server error",
                    "server",
                    Vec::default(),
                    Some(InnerError::new(other.to_string())),
                ));
                return Ok(resp);
            }
        },
    };
    log::debug!("User inited socket connection: {}", token.claims.sub);

    let (mut res, mut session, stream) = actix_ws::handle(&req, stream)?;

    // RFC 6455 §4.2.2: server MUST echo one of the offered subprotocols back
    // in the handshake response, or the browser aborts the connection.
    res.headers_mut().insert(
        header::SEC_WEBSOCKET_PROTOCOL,
        HeaderValue::from_static(BEARER_SUBPROTOCOL),
    );

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
                Ok(AggregatedMessage::Binary(bin)) => match Envelope::decode(bin) {
                    Ok(envelope) => {
                        if let Some(payload) = envelope.payload {
                            match payload {
                                envelope::Payload::Rtc(rtc_event) => {
                                    log::debug!("{:#?}", rtc_event);
                                }
                                envelope::Payload::RtcIce(rtc_new_candidate) => {
                                    log::debug!("{:#?}", rtc_new_candidate);
                                }
                                envelope::Payload::VoiceChannelRequest(voice_channel_request) => {
                                    log::debug!("{:#?}", voice_channel_request);
                                }
                                _ => {
                                    log::error!("got message that contained invalid message payload")
                                }
                            }
                        }
                    }
                    Err(err) => {
                        log::error!("{}", err);
                    }
                },
                Ok(AggregatedMessage::Ping(msg)) => {
                    session.pong(&msg).await.expect("failed to send pong")
                }

                _ => {}
            }
        }
    });

    Ok(res)
}
