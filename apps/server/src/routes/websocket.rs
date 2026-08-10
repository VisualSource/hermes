use actix_web::http::StatusCode;
use actix_web::http::header::{self, HeaderValue};
use actix_web::{Error, HttpRequest, HttpResponse, Responder, rt, web};
use actix_ws::AggregatedMessage;
use futures_util::{FutureExt, SinkExt, StreamExt as _};
use prost::Message;
use prost::bytes::Bytes;

use crate::state::messages::{Envelope, RtcEvent, envelope};

use crate::state::api_errors::ApplicationError;
use crate::state::socket::session::SessionRegistry;
use crate::state::socket::{self, auth::BEARER_SUBPROTOCOL};

pub async fn ws(
    req: HttpRequest,
    stream: web::Payload,
    registry: web::Data<SessionRegistry>,
) -> Result<HttpResponse, ApplicationError> {
    let claims = socket::auth::validate(&req)?;

    log::debug!("User initd socket connection: {}", claims.sub);

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

    let (tx, mut rx) = tokio::sync::mpsc::channel::<Bytes>(256);

    let conn_id = match registry.register(claims.sub, tx.clone()) {
        Ok(id) => id,
        Err(err) => {
            //TODO: handler error better
            log::error!("{}", err);
            return Err(ApplicationError::internal_server_error(
                "",
                "",
                err.to_string(),
            ));
        }
    };

    let mut bcp = session.clone();
    rt::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if bcp.binary(msg).await.is_err() {
                log::error!("Failed to send message to closed client");
            }
        }
    });

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
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
                                    log::error!(
                                        "got message that contained invalid message payload"
                                    )
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

        registry.unregister(&conn_id).expect("failed to unregister");
    });

    Ok(res)
}
