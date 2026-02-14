use actix_web::{Error, HttpRequest, HttpResponse, rt, web};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt as _;

pub async fn ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
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
                    session.binary(bin).await.expect("failed to send bin")
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
