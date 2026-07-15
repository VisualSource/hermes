//! Headless renderer that converts a RoomState into a BGRA8 pixel buffer
//! suitable for `asdf_overlay_client::surface::OverlaySurface::update_bitmap`.
//!
//! Two backends:
//!   * `Backend::Iced` — full iced_wgpu headless rendering (the thing we want).
//!   * `Backend::Stub` — hand-drawn BGRA (color changes with state). Only used
//!     to isolate whether an issue is in the injection layer or the iced layer.
//!     Toggle with env var HERMES_OVERLAY_STUB_RENDER=1.

use anyhow::{Context, Result};
use iced_core::{Color, Size, Theme, mouse, renderer};
use iced_graphics::{Shell, Viewport};
use iced_runtime::user_interface::{Cache, UserInterface};
use iced_wgpu::{Engine, Renderer, wgpu};

use crate::voice_state::RoomState;
use crate::widgets::app;

pub const BYTES_PER_PIXEL: u32 = 4;

pub enum Backend {
    Iced(IcedBackend),
    Stub(StubBackend),
}

impl Backend {
    pub fn choose(width: u32, height: u32) -> Result<Self> {
        if std::env::var("HERMES_OVERLAY_STUB_RENDER").ok().as_deref() == Some("1") {
            tracing::warn!("HERMES_OVERLAY_STUB_RENDER=1 — using stub BGRA renderer");
            return Ok(Backend::Stub(StubBackend { width, height }));
        }
        match IcedBackend::new(width, height) {
            Ok(b) => Ok(Backend::Iced(b)),
            Err(e) => {
                tracing::error!(
                    error = ?e,
                    "iced headless renderer failed to init — falling back to stub"
                );
                Ok(Backend::Stub(StubBackend { width, height }))
            }
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        match self {
            Backend::Iced(b) => {
                b.width = width;
                b.height = height;
                Ok(())
            }
            Backend::Stub(b) => {
                b.width = width;
                b.height = height;
                Ok(())
            }
        }
    }

    pub fn render(&mut self, state: &RoomState) -> Result<Vec<u8>> {
        match self {
            Backend::Iced(b) => b.render(state),
            Backend::Stub(b) => Ok(b.render(state)),
        }
    }

    pub fn width(&self) -> u32 {
        match self {
            Backend::Iced(b) => b.width,
            Backend::Stub(b) => b.width,
        }
    }
}

// -----------------------------------------------------------------------------
// Stub — hand-drawn BGRA so we can validate injection independently of iced.
// -----------------------------------------------------------------------------

pub struct StubBackend {
    pub width: u32,
    pub height: u32,
}

impl StubBackend {
    fn render(&self, state: &RoomState) -> Vec<u8> {
        let (w, h) = (self.width.max(1), self.height.max(1));
        let (r, g, b, count) = match state.active_room.as_ref() {
            None => (200u8, 40u8, 40u8, 0u32),
            Some(room) => (40, 180, 90, room.participants.len() as u32),
        };
        let alpha = 180u8;
        let mut buf = vec![0u8; (w * h * BYTES_PER_PIXEL) as usize];
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * BYTES_PER_PIXEL) as usize;
                buf[i] = b;
                buf[i + 1] = g;
                buf[i + 2] = r;
                buf[i + 3] = alpha;
            }
        }
        let bar_h = 6u32.min(h);
        let max_bars = (w / 12).saturating_sub(1);
        for k in 0..count.min(max_bars) {
            let x0 = 4 + k * 12;
            for y in 0..bar_h {
                for x in x0..(x0 + 8).min(w) {
                    let i = ((y * w + x) * BYTES_PER_PIXEL) as usize;
                    buf[i] = 255;
                    buf[i + 1] = 255;
                    buf[i + 2] = 255;
                    buf[i + 3] = 255;
                }
            }
        }
        buf
    }
}

// -----------------------------------------------------------------------------
// Iced headless — build wgpu manually, screenshot(), swizzle RGBA→BGRA.
// -----------------------------------------------------------------------------

pub struct IcedBackend {
    #[allow(dead_code)]
    device: wgpu::Device,
    #[allow(dead_code)]
    queue: wgpu::Queue,
    renderer: Renderer,
    cache: Option<Cache>,
    pub width: u32,
    pub height: u32,
}

const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

impl IcedBackend {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: None,
                force_fallback_adapter: false,
            },
        ))
        .context("no wgpu adapter available")?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("hermes-overlay"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            },
        ))
        .context("wgpu device request failed")?;

        let engine = Engine::new(
            &adapter,
            device.clone(),
            queue.clone(),
            TARGET_FORMAT,
            None,
            Shell::headless(),
        );
        let renderer = Renderer::new(engine, iced_core::Font::DEFAULT, iced_core::Pixels(14.0));

        Ok(Self {
            device,
            queue,
            renderer,
            cache: Some(Cache::new()),
            width,
            height,
        })
    }

    pub fn render(&mut self, state: &RoomState) -> Result<Vec<u8>> {
        let viewport =
            Viewport::with_physical_size(Size::new(self.width, self.height), 1.0);

        let element = app::view(state);
        let cache = self.cache.take().unwrap_or_else(Cache::new);
        let mut clipboard = iced_core::clipboard::Null;
        let mut messages: Vec<app::Message> = Vec::new();
        let cursor = mouse::Cursor::Unavailable;

        let mut ui = UserInterface::build(
            element,
            viewport.logical_size(),
            cache,
            &mut self.renderer,
        );

        let _ = ui.update(
            &[],
            cursor,
            &mut self.renderer,
            &mut clipboard,
            &mut messages,
        );
        ui.draw(
            &mut self.renderer,
            &Theme::Dark,
            &renderer::Style::default(),
            cursor,
        );
        self.cache = Some(ui.into_cache());

        // screenshot() returns tightly-packed RGBA8 bytes.
        let rgba = self.renderer.screenshot(&viewport, Color::TRANSPARENT);
        Ok(rgba_to_bgra(rgba))
    }
}

fn rgba_to_bgra(mut buf: Vec<u8>) -> Vec<u8> {
    for chunk in buf.chunks_exact_mut(4) {
        chunk.swap(0, 2);
    }
    buf
}
