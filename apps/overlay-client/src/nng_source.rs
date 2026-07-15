//! nng subscriber: spawn a std thread with a blocking recv() loop and forward
//! decoded VoiceEvents into a tokio mpsc channel. Per nng-rs guidance the Aio
//! callback isn't a tokio future so a bridge thread is the pragmatic path.

use anyhow::{Context, Result};
use nng::{
    Protocol, Socket,
    options::{Options, protocol::pubsub::Subscribe},
};
use tokio::sync::mpsc;

use crate::voice_state::VoiceEvent;

pub const DEFAULT_URL: &str = "ipc://hermes-overlay";

pub fn spawn(url: String) -> Result<mpsc::UnboundedReceiver<VoiceEvent>> {
    let (tx, rx) = mpsc::unbounded_channel();

    let sock = Socket::new(Protocol::Sub0).context("nng: create Sub0 socket")?;
    sock.dial(&url)
        .with_context(|| format!("nng: dial {url}"))?;
    sock.set_opt::<Subscribe>(Vec::new())
        .context("nng: subscribe to all topics")?;

    std::thread::Builder::new()
        .name("hermes-overlay-nng".into())
        .spawn(move || pump(sock, tx))
        .context("failed to spawn nng thread")?;

    Ok(rx)
}

fn pump(sock: Socket, tx: mpsc::UnboundedSender<VoiceEvent>) {
    loop {
        match sock.recv() {
            Ok(msg) => match serde_json::from_slice::<VoiceEvent>(msg.as_slice()) {
                Ok(event) => {
                    tracing::debug!(?event, "nng recv");
                    if tx.send(event).is_err() {
                        tracing::info!("nng consumer dropped, exiting pump");
                        return;
                    }
                }
                Err(e) => {
                    let preview = String::from_utf8_lossy(msg.as_slice());
                    tracing::warn!(error = %e, ?preview, "nng: bad payload, ignoring");
                }
            },
            Err(e) => {
                tracing::error!(error = %e, "nng recv failed, exiting pump");
                return;
            }
        }
    }
}
