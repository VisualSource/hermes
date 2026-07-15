//! SPIKE support: sits in for the real hermes desktop app and publishes a
//! canned sequence of VoiceEvents over `ipc://hermes-overlay`. Lets you drive
//! the overlay by hand without booting the whole desktop client.
//!
//! Run:  cargo run --bin mock-publisher
//! Env:  HERMES_OVERLAY_NNG_URL to override the transport url.

use std::{env, thread, time::Duration};

use anyhow::{Context, Result};
use nng::{Protocol, Socket};

// Duplicated inline instead of importing from the library to keep the mock
// standalone. If we're wondering "do the two sides agree on schema" — that IS
// what this spike answers. Keep them in sync manually for now.
#[derive(serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum VoiceEvent<'a> {
    RoomJoined { room_id: &'a str, room_name: &'a str },
    RoomLeft,
    UserJoined { user_id: &'a str, display_name: &'a str },
    UserLeft { user_id: &'a str },
    UserSpeaking { user_id: &'a str, speaking: bool },
    UserMuted { user_id: &'a str, muted: bool },
}

fn main() -> Result<()> {
    let url = env::var("HERMES_OVERLAY_NNG_URL")
        .unwrap_or_else(|_| "ipc://hermes-overlay".into());

    let sock = Socket::new(Protocol::Pub0).context("nng: create Pub0 socket")?;
    sock.listen(&url)
        .with_context(|| format!("nng: listen {url}"))?;

    // Give a subscriber a moment to connect before the first send (PUB drops
    // messages that have no subscribers, so early sends are lost).
    thread::sleep(Duration::from_millis(500));

    let script: &[(Duration, VoiceEvent)] = &[
        (Duration::from_secs(0), VoiceEvent::RoomJoined { room_id: "r1", room_name: "general" }),
        (Duration::from_secs(1), VoiceEvent::UserJoined { user_id: "u1", display_name: "Alice" }),
        (Duration::from_secs(1), VoiceEvent::UserJoined { user_id: "u2", display_name: "Bob" }),
        (Duration::from_secs(1), VoiceEvent::UserSpeaking { user_id: "u1", speaking: true }),
        (Duration::from_secs(2), VoiceEvent::UserSpeaking { user_id: "u1", speaking: false }),
        (Duration::from_secs(1), VoiceEvent::UserMuted { user_id: "u2", muted: true }),
        (Duration::from_secs(2), VoiceEvent::UserJoined { user_id: "u3", display_name: "Carol" }),
        (Duration::from_secs(2), VoiceEvent::UserLeft { user_id: "u2" }),
        (Duration::from_secs(2), VoiceEvent::RoomLeft),
    ];

    loop {
        for (delay, event) in script {
            thread::sleep(*delay);
            let payload = serde_json::to_vec(event)?;
            println!("publish: {}", String::from_utf8_lossy(&payload));
            sock.send(payload.as_slice())
                .map_err(|(_, e)| anyhow::anyhow!("nng send: {e}"))?;
        }
        println!("--- loop ---");
    }
}
