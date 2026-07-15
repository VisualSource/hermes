//! ============================================================================
//! SPIKE — hermes-overlay integration spike
//! ============================================================================
//! Question: do `asdf-overlay-client` (DLL injection + shared-texture push),
//!           `iced` (headless UI), and `nng` (event bus from the main app)
//!           compose into a working game overlay?
//!
//! This binary is throwaway. Everything except `voice_state.rs` is expected
//! to be rewritten once the answer is known. Do NOT build on top of this;
//! copy the validated pieces (state model, message flow) into fresh files.
//!
//! Run:
//!   1. Launch the target process (a game, or notepad.exe for a smoke test).
//!   2. cargo run --bin mock-publisher            # in another terminal
//!   3. cargo run --bin hermes-overlay -- <pid>   # takes target pid as arg
//!
//! Requires `asdf-overlay-x64.dll` next to the binary. Grab from
//! https://github.com/storycraft/asdf-overlay/releases (v1.2.x).
//!
//! Env vars:
//!   HERMES_OVERLAY_STUB_RENDER=1     bypass iced, draw solid BGRA (bisect aid)
//!   HERMES_OVERLAY_NNG_URL=...       override ipc://hermes-overlay
//!   RUST_LOG=hermes_overlay=debug    tracing filter
//! ============================================================================

use std::{env, str::FromStr, time::Duration};

use anyhow::{Context, Result, anyhow};
use asdf_overlay_client::{
    OverlayDll, inject,
    event::{OverlayEvent, WindowEvent},
    surface::OverlaySurface,
};
use asdf_overlay_common::request::UpdateSharedHandle;
use tracing_subscriber::EnvFilter;

mod nng_source;
mod render;
mod voice_state;
mod widgets;

use crate::{render::Backend, voice_state::RoomState};

const FRAME_INTERVAL: Duration = Duration::from_millis(33); // ~30fps

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let pid = env::args()
        .nth(1)
        .ok_or_else(|| anyhow!("usage: hermes-overlay <target-pid>"))?;
    let pid = sysinfo::Pid::from_str(&pid).context("target pid must be an integer")?;

    verify_process_alive(pid)?;

    let dll_path = env::current_dir()?.join("asdf-overlay-x64.dll");
    if !dll_path.exists() {
        tracing::warn!(
            path = ?dll_path,
            "asdf-overlay-x64.dll not found next to CWD — inject() will fail. \
             Download it from github.com/storycraft/asdf-overlay/releases."
        );
    }

    let dll = OverlayDll {
        x64: Some(dll_path.as_path()),
        x86: None,
        arm64: None,
    };

    tracing::info!(?pid, "injecting overlay DLL");
    let (mut conn, mut events) =
        inject(pid.as_u32(), dll, Some(Duration::from_secs(10)))
            .await
            .context("injection failed")?;
    tracing::info!("injection succeeded, waiting for target window");

    let nng_url = env::var("HERMES_OVERLAY_NNG_URL")
        .unwrap_or_else(|_| nng_source::DEFAULT_URL.to_string());
    let mut nng_rx = nng_source::spawn(nng_url)?;

    let mut state = RoomState::default();
    let mut render_state = RenderState::WaitingForWindow;

    let mut ticker = tokio::time::interval(FRAME_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            biased;

            maybe_evt = events.recv() => match maybe_evt {
                Some(OverlayEvent::Window { id, event }) => {
                    handle_window_event(id, event, &mut render_state)?;
                }
                None => {
                    tracing::info!("overlay event stream closed, exiting");
                    return Ok(());
                }
            },

            Some(voice_evt) = nng_rx.recv() => {
                tracing::info!(event = ?voice_evt, "voice event -> state");
                state.apply(voice_evt);
            }

            _ = ticker.tick() => {
                if let RenderState::Ready { window_id, backend, surface } = &mut render_state {
                    match backend.render(&state) {
                        Ok(bytes) => {
                            match surface.update_bitmap(backend.width(), &bytes) {
                                Ok(Some(update)) => {
                                    if let Err(e) = push_update(&mut conn, *window_id, update).await {
                                        tracing::error!(error = ?e, "conn.request(UpdateSharedHandle) failed");
                                    }
                                }
                                Ok(None) => { /* handle unchanged, nothing to send */ }
                                Err(e) => tracing::error!(error = ?e, "surface.update_bitmap failed"),
                            }
                        }
                        Err(e) => tracing::error!(error = ?e, "render failed"),
                    }
                }
            }
        }
    }
}

enum RenderState {
    WaitingForWindow,
    Ready {
        window_id: u32,
        backend: Backend,
        surface: OverlaySurface,
    },
}

fn handle_window_event(
    id: u32,
    event: WindowEvent,
    render_state: &mut RenderState,
) -> Result<()> {
    match event {
        WindowEvent::Added { width, height, .. } => {
            tracing::info!(id, width, height, "target window ready");
            let backend = Backend::choose(width.max(1), height.max(1))?;
            let surface = OverlaySurface::new(None).context("OverlaySurface::new")?;
            *render_state = RenderState::Ready {
                window_id: id,
                backend,
                surface,
            };
        }
        WindowEvent::Resized { width, height } => {
            tracing::info!(id, width, height, "target window resized");
            if let RenderState::Ready { backend, .. } = render_state {
                if let Err(e) = backend.resize(width.max(1), height.max(1)) {
                    tracing::error!(error = ?e, "backend resize failed");
                }
            }
        }
        WindowEvent::Destroyed => {
            tracing::info!(id, "target window destroyed");
            *render_state = RenderState::WaitingForWindow;
        }
        WindowEvent::Input(_) | WindowEvent::InputBlockingEnded => {
            // Not listening for input in this spike.
        }
    }
    Ok(())
}

async fn push_update(
    conn: &mut asdf_overlay_client::client::IpcClientConn,
    window_id: u32,
    update: UpdateSharedHandle,
) -> Result<()> {
    conn.window(window_id).request(update).await?;
    Ok(())
}

fn verify_process_alive(pid: sysinfo::Pid) -> Result<()> {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, false);
    if sys.process(pid).is_none() {
        anyhow::bail!("no process with pid {pid}");
    }
    Ok(())
}
