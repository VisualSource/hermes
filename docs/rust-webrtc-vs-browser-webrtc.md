# Rust-native WebRTC vs. browser/WebView WebRTC for Hermes

Research note. Question: *Should Hermes move voice off the browser's `RTCPeerConnection` and onto a Rust
WebRTC stack (`webrtc-rs` or `str0m`), using Tauri purely as a UI shell?*

Every non-obvious claim below is tagged either **[primary]** (crate docs, source repo, W3C/vendor docs) or
**[secondary]** (blog/forum/paper). URLs are collected in [Sources](#sources).

---

## TL;DR / Recommendation

**Keep voice in the browser WebView for the desktop client. Do not rewrite the RTC transport in Rust right now.**

The dominant reason is not that the Rust crates are bad — `str0m` and `webrtc-rs` are genuinely capable
transport/protocol stacks — but that **neither of them does media capture, audio device I/O, codec
encode/decode, *or the audio-processing pipeline (echo cancellation / noise suppression / AGC)***. Those are
exactly the parts that make voice chat usable, and libwebrtc-in-the-WebView gives them to you for free and
battle-tested. Moving to Rust means you inherit ownership of AEC/NS/AGC, jitter buffering, device handling,
and Opus, on top of the transport — a large, ongoing surface for a small-scale peer-mesh app.

**The four strongest reasons to stay on browser WebRTC:**

1. **Audio Processing Module (AEC3/NS/AGC) is the hard part and libwebrtc already ships it.** str0m and
   webrtc-rs both *explicitly exclude* audio capture, encode/decode, rendering, and jitter buffering. **[primary]**
   The only mature Rust route to AEC3 is FFI bindings to the same C++ libwebrtc APM (`webrtc-audio-processing`),
   which you'd have to wire into `cpal` yourself. **[primary]**
2. **On Windows (Hermes' primary desktop target) the WebView is WebView2/Chromium, i.e. full libwebrtc.**
   `RTCPeerConnection`, `getUserMedia`, AEC3, GCC congestion control, jitter buffer, and hardware codec access
   all work out of the box. **[primary]**
3. **Rust crates give you transport, not a phone.** str0m is sans-IO: *you* supply UDP sockets, ICE candidate
   gathering (NIC enumeration), TURN, timers, and all media I/O. **[primary]** That's a lot of plumbing to
   re-own to replace something the browser already does.
4. **Peer-to-peer is the *less-tested* path for both crates** — they are hardened for server/SFU use, which is
   the opposite of Hermes' peer-mesh topology. **[primary]**

**When the answer flips (and it partly does):** the **overlay-client** is a headless Rust binary with **no
WebView at all** — it cannot use browser WebRTC, so if it ever needs to send/receive voice, a Rust stack
(str0m + cpal + Opus + `webrtc-audio-processing`) is the *only* option. Same logic applies if Hermes later adds
a headless/background voice mode, a server-side SFU (to escape the peer-mesh's O(n²) scaling), or must ship on
Linux where the default WebKitGTK WebView has no WebRTC at all (see §2). A pragmatic middle path exists:
libwebrtc via a Rust FFI crate (e.g. `libwebrtc-sys`/`cxx`-style bindings) rather than a pure-Rust reimpl — you
keep AEC3 without a WebView, but you take on the build complexity of libwebrtc.

---

## 1. What Rust WebRTC options exist, and how mature are they

There are two serious general-purpose Rust WebRTC libraries: **`str0m`** and **`webrtc-rs`**. Both are
converging on a **sans-IO** protocol-core design.

### str0m (algesten/str0m)

- **Design:** "A Sans I/O WebRTC implementation in Rust." The `Rtc` instance "itself is not doing any network
  talking… no internal threads or async tasks"; all input is modeled as `handle_input(&mut self, …)` and you
  drain `poll_output()` for packets/events/timeouts. No `Rc`, `Mutex`, `mpsc`, or `Arc` internally. **[primary — GitHub README / docs.rs]**
- **Implemented:** ICE agent (once candidates are supplied), DTLS-SRTP, SDP negotiation (plus a "Direct API"),
  data channels, RTP/RTCP at frame *and* packet level, RTCP send/receive reports, **Transport-Wide Congestion
  Control (TWCC)**, **bandwidth estimation (BWE/GCC)**, NACK, simulcast, and RTP packetization for **Opus,
  H.264, VP8, VP9**. **[primary — README]** Notably, str0m *does* claim real congestion control — this is a
  meaningful differentiator vs. webrtc-rs.
- **Explicitly NOT implemented (out of scope):** video/audio **encode**, video/audio **decode**, audio
  **capture**, audio **rendering**, **adaptive jitter buffer**, **ICE candidate gathering / NIC enumeration**,
  and **TURN** integration (you feed it discovered candidates). No `RTCPeerConnection`-style API by design. **[primary — docs.rs]**
- **Maturity / activity:** Actively maintained (well over 1,800 commits), MIT OR Apache-2.0, created by Martin
  Algesten (Lookback). Heavily tested for **server-side SFU**; the README states **peer-to-peer "has received
  less testing."** The bundled chat example is "not intended for production use." **[primary — README]** Used in
  production by tools such as BitWHIP. **[secondary]**

### webrtc-rs (webrtc-rs/webrtc + webrtc-rs/rtc)

- **What it is:** A full-stack, historically **async/Tokio** WebRTC implementation, "originally inspired by and
  largely rewriting the Pion stack" (the Go implementation). Modular: separate crates for ICE, DTLS, SCTP, RTP,
  RTCP, SRTP, STUN, TURN, SDP. It exposes a `RTCPeerConnection`-shaped API, closer to the W3C/browser model
  than str0m. **[primary — crates.io / docs.rs / repo]**
- **State of the project (as of mid-2026):** The mature, production-recommended line is **v0.17.x**, now in
  "bug-fix-only maintenance." The project is doing a **sans-IO rewrite** (the `rtc` crate, and a v0.20.0-rc
  async layer on top) that mirrors str0m's architecture, with a Quinn-style runtime abstraction (Tokio default,
  Smol optional). The rewrite claims "95%+ W3C API compliance." **[primary — webrtc.rs blog "Building
  Async-Friendly webrtc on Sans-I/O rtc" (2026-01-31); crates.io]**
- **Congestion control caveat:** webrtc-rs inherits Pion's architecture, and full **GCC/sender-side bandwidth
  estimation has historically been a known gap in the Pion lineage** (tracked in pion/webrtc issues). Unlike
  str0m, webrtc-rs does not prominently advertise a complete BWE/GCC implementation. **Treat webrtc-rs's
  congestion-control completeness as needs-verification** for your specific version. **[secondary — pion issues #359/#1922; needs verification against the exact webrtc-rs release]**
- **Same media exclusions:** like str0m, webrtc-rs is a transport/protocol stack — it does **not** provide
  microphone capture, speaker rendering, Opus encode/decode, or the AEC/NS/AGC audio pipeline. **[primary — the crate is a protocol stack, not a media engine]**

### Sans-IO vs. full-stack — the practical implication

Sans-IO (str0m, and the new webrtc-rs `rtc` core) means the library is a **pure state machine**: deterministic,
easy to test, runtime-agnostic — but **you** own the event loop, UDP sockets, timers, ICE gathering, and TURN.
That's excellent for an SFU or an embeddable engine and more work for a "just give me a peer connection" client.
webrtc-rs's async `RTCPeerConnection` API is friendlier to port browser code to, at the cost of pulling in Tokio
and being the less-actively-evolving of the two designs right now.

---

## 2. The browser-WebRTC baseline (what the WebView gives you for free)

The WebView engine differs per OS in Tauri: **Windows → WebView2 (Edge/Chromium)**, **macOS → WKWebView
(WebKit)**, **Linux → webkit2gtk (WebKitGTK)**. **[primary — Tauri "Webview Versions"]** This is the crux of the
cross-platform story, because their WebRTC support is *not* equal.

When the WebView is Chromium (Windows/WebView2, Android), you get the **complete libwebrtc stack** behind
`getUserMedia` and `RTCPeerConnection`, including:

- **Audio Processing Module (APM):** acoustic **echo cancellation**, **noise suppression**, and **automatic gain
  control** — "required for VoIP calling," runs on the capture/render streams, thread-safe, auto-reconfiguring
  for sample rates < 384 kHz. **[primary — WebRTC APM g3doc]** (Chromium ships the modern AEC3 echo canceller;
  the g3doc names AEC/NS/AGC as the three effects.) **[primary]**
- Adaptive **jitter buffer**, **GCC** congestion control (delay-based + loss-based estimators driven by TWCC
  feedback), NACK/PLI, device enumeration via `getUserMedia`, and hardware codec access. **[secondary — webrtcHacks / Forasoft on GCC/BWE; general knowledge of libwebrtc]**

### The cross-platform pain point (primary evidence)

- **Linux / WebKitGTK: no WebRTC by default.** wry issue **#85 "WebRTC support on Linux"** (open since 2020,
  still open) reports `navigator.mediaDevices` is `undefined` under webkit2gtk; the limitation is upstream in
  WebKit, not fixable in wry/Tauri directly. **[primary — wry#85]** The Tauri community workaround
  (discussion **#8426**) confirms you must **custom-compile WebKitGTK** with `-DENABLE_WEB_RTC=ON` and
  `-DENABLE_MEDIA_STREAM=ON`, pull in GStreamer plugins (base/good/bad for `webrtcbin`/`webrtcdsp`), enable
  `set_enable_webrtc(true)` / `set_enable_media_stream(true)` in the webview, implement a
  `permissions-request` handler (else `getUserMedia` returns `NotAllowedError`), and it **only works under X11
  (Wayland throws GBM buffer errors)**. This is not viable for a normal distributable. **[primary — Tauri
  discussion #8426]**
  - **Project plan for Hermes: swap to a Chromium/CEF WebView on Linux instead of WebKitGTK — verified as a real,
    active core-team feature branch (pre-release).** The characterization checks out:
    - The work lives on a public branch, **`feat/cef` in `tauri-apps/tauri`**, actively developed by Tauri core
      maintainer *amrbashir*, with commits as recent as 2026-07-16 (e.g. "refactor(cef): bring
      external_message_pump to be a closer port of cefclient official impl", "chore: move cef-helper into
      crates/ directory", macOS `NSView`/`NSWindow` parenting fixes) and regularly merged up from `dev`. This is
      genuine backend-integration work, not a stub. **[primary — github.com/tauri-apps/tauri/tree/feat/cef; GitHub commits API]**
    - It is backed by an official bindings crate, **`tauri-apps/cef-rs`** ("Use CEF in Rust"), actively released
      (latest tag `cef-v150.2.1+150.0.14`, 2026-07-21) with bundling tooling (`export-cef-dir`,
      `bundle-cef-app`), and CEF handling is landing in the Tauri CLI/bundler (cf. tauri issue #15287 on CEF
      framework copy). **[primary — github.com/tauri-apps/cef-rs; tauri#15287]**
    - **But it is pre-release, not stable/merged, and possibly partly commercial.** Maintainer FabianLars: CEF is
      "still not off the table but there's no ETA" (Jan 2024); it was "started… but halted… twice"; and "it
      likely won't be part of tauri (the open source org) directly but a closed source or company offering." By
      Nov 2025→Jan 2026 the team moved to "working on it" and "dogfooding while we're working on customer
      projects. The open source work will follow a bit later." So "feature branch, not stable/released" is
      accurate. **[primary — Tauri discussion #8524]**
    - **Do not conflate CEF with Tauri's other alt-webview effort, Verso/Servo** ("Experimental Tauri Verso
      Integration," 2025-03-17, NLnet-funded, with Igalia). That one is **Servo-based, not Chromium**, explicitly
      experimental, and would **not** provide libwebrtc — Servo "won't be able to compete with the normal
      browsers feature wise anytime soon, if ever." The CEF branch, not Verso, is the one relevant to closing the
      WebRTC hole. **[primary — Tauri Verso blog; discussion #8524]**
    - **Does CEF close the Linux WebRTC hole?** CEF embeds Chromium's content layer, which includes libwebrtc, so
      a working CEF-backed webview should give `getUserMedia`/`RTCPeerConnection` the same complete voice stack as
      WebView2 on Windows. **This last step is [inferred]** — no Tauri primary source I found explicitly states
      "the CEF backend delivers WebRTC on Linux," and CEF media features can require build/runtime flags. Net: the
      direction is real, core-team-driven, and progressing, but it is a **pre-release feature branch with no
      committed ETA** — forward-looking, not something to ship on today.
- **macOS / WKWebView: works, but must be wired up.** Historically `getUserMedia` was disabled in embedded
  WKWebView (noted in wry#85). Modern macOS/iOS (WKWebView, ~macOS 12+/iOS 14.3+) **does** support it *if* the
  app implements the `webView(_:requestMediaCapturePermissionFor:…:decisionHandler:)` delegate, sets
  `NSCameraUsageDescription`/`NSMicrophoneUsageDescription` in Info.plist, adds the camera/mic entitlements, and
  serves over a secure context (HTTPS/local). wry has added media-permission handling to cover this. **[primary
  — Apple Developer Forums / WebKit; secondary — solarana.dev writeup]**
- **Windows / WebView2:** full Chromium WebRTC, no special handling. **[primary — Tauri "Webview Versions"]**

**Net:** browser WebRTC is a free, complete voice stack **on Windows and (with config) macOS**, but is **broken
by default on Linux**. If Linux desktop is a target, the WebView path has a real hole — and that's the strongest
platform-specific argument *for* a Rust stack.

---

## 3. Audio processing (the decisive factor for voice quality)

libwebrtc's **APM** — AEC (AEC3), NS, AGC — is the thing that makes a mesh voice call not scream with echo and
not clip. It is "responsible for applying speech enhancement effects to the microphone signal… required for VoIP
calling." **[primary — WebRTC APM g3doc]** In the browser path you get it automatically; in a Rust path you get
**none of it** from str0m/webrtc-rs (both exclude audio capture/encode/decode/render). **[primary]**

Rust options to recover APM:

- **`webrtc-audio-processing`** (tonarino) — Rust bindings to the C++ WebRTC AudioProcessing module (via
  PulseAudio's repackaging). Exposes **echo cancellation, noise suppression, automatic gain control, and voice
  activity detection**, with an `experimental-aec3-config` feature for detailed `EchoCanceller3` configuration.
  Builds by linking a system lib (`libwebrtc-audio-processing-dev`) or via the `bundled` feature (needs
  clang/gcc, pkg-config, meson, ninja). API is versioned loosely to upstream and can break between minor
  versions. **[primary — crate README / docs.rs]** This is a real, usable path — but it's still **FFI into the
  same C++ libwebrtc code**, plus you must feed it frames from `cpal` and align render/capture streams yourself.
- **`sonora`** (dignifiedquire) — a *pure-Rust* reimplementation of WebRTC audio processing (AEC/NS/AGC).
  Promising for avoiding the C++ toolchain, but far younger/less battle-tested than libwebrtc's APM. **[primary
  — repo exists; secondary — maturity is unproven, needs verification]**

Hermes already uses `@sapphi-red/web-noise-suppressor` for NS in the browser, but that is **noise suppression
only** — it is not echo cancellation. Real AEC still comes from libwebrtc under `getUserMedia({echoCancellation:
true})`. Reimplementing/binding AEC3 is the single biggest hidden cost of the Rust route.

---

## 4. Effort / architecture implications of moving RTC into Rust

Going Rust-native means Hermes' own code (or its dependencies) must own **everything the browser currently
hides**:

| Concern | Browser WebView (today) | Rust stack (str0m/webrtc-rs) |
|---|---|---|
| ICE / STUN | libwebrtc | webrtc-rs: yes; **str0m: you gather candidates / enumerate NICs** [primary] |
| TURN relay | libwebrtc | webrtc-rs has a TURN crate; **str0m: you integrate TURN** [primary] |
| DTLS-SRTP | libwebrtc | Both implement it [primary] |
| Congestion control | GCC (libwebrtc) | **str0m: TWCC+BWE** [primary]; **webrtc-rs: incomplete/verify** [secondary] |
| Jitter buffer | libwebrtc adaptive | **Neither** — you build it (str0m excludes it) [primary] |
| Opus encode/decode | libwebrtc | **Neither** — you bring your own (Hermes already has the `opus` crate) [primary] |
| Audio device I/O | `getUserMedia` | **Neither** — you use `cpal` (already in Hermes' `src-tauri`) [primary] |
| **AEC / NS / AGC** | **libwebrtc APM (AEC3)** | **Neither** — FFI to `webrtc-audio-processing`, or `sonora` [primary] |
| Cross-platform capture/permissions | WebView handles it | You handle OS mic permissions per platform |

Hermes *does* already have two of the pieces in `src-tauri` (`opus` + `cpal`), which lowers the cost somewhat.
But the assembly work — device I/O ↔ APM ↔ Opus ↔ RTP/SRTP ↔ ICE, with echo cancellation correctly aligning
render and capture clocks, plus a jitter buffer — is a multi-month effort to *reach parity* with what the WebView
already does on Windows/macOS. For a small-scale peer-mesh chat app, that's a poor trade unless a hard constraint
forces it.

---

## 5. When the Rust approach genuinely wins

- **The overlay-client has no WebView.** It's a standalone Rust binary (egui + asdf-overlay-client). If it must
  ever *originate or play* voice (not just display state), browser WebRTC is simply unavailable to it —
  str0m/webrtc-rs + cpal + Opus + `webrtc-audio-processing` is the *only* route. This is the clearest win.
- **Headless / background voice.** Any "voice keeps running with the window closed / no WebView" mode needs a
  non-WebView stack.
- **Linux desktop as a first-class target.** Default WebKitGTK ships no WebRTC (wry#85), and the custom-compiled
  WebKitGTK workaround is X11-only and undistributable (Tauri #8426). A Rust stack sidesteps the WebView entirely
  and gives *consistent* behavior across all three OSes. **[primary]**
- **Server-side SFU.** If the peer mesh's O(n²) fan-out becomes the scaling ceiling, an SFU is the fix — and
  str0m is explicitly built and battle-tested for exactly that. **[primary]** This is also where webrtc-rs/str0m
  are strongest and where their P2P-is-less-tested caveat stops mattering.
- **Deterministic control / testing / custom codecs.** Sans-IO gives a pure, mockable state machine and lets you
  choose codecs and pacing precisely — valuable for reproducible tests or bespoke media handling.

---

## 6. Recommendation for Hermes specifically

Hermes today: **Tauri desktop, peer-mesh voice, small scale, Windows-primary.** For that profile:

1. **Stay on browser `RTCPeerConnection` for the desktop client.** On Windows (WebView2) it's full libwebrtc
   including AEC3/GCC/jitter buffer for free; on macOS it works with the delegate/Info.plist wiring you'd do once.
   The Rust crates would force you to re-own AEC/NS/AGC, jitter buffering, and Opus/device glue to merely match
   what you already have. **[primary basis: §2, §3]**
2. **Treat Linux as the exception, not the rule — and the plan for it is the CEF WebView, not a Rust stack.**
   The Hermes plan for Linux is to run Tauri on the **CEF (Chromium) WebView backend** — verified as the real,
   core-team `feat/cef` branch in `tauri-apps/tauri` (active as of July 2026, backed by the `cef-rs` crate),
   though still a **pre-release feature branch with no committed ETA and a possible commercial component** (see
   §2). CEF embeds Chromium's libwebrtc, so it *should* bring the same full voice stack as Windows/WebView2 and
   close the WebKitGTK hole — but that WebRTC payoff is **[inferred]**, not yet confirmed by a Tauri primary
   source. Do **not** bet on custom-compiled WebKitGTK, and note this is *not* the Verso/Servo effort (Servo has
   no libwebrtc). Until `feat/cef` is stable and production-viable, options are (a) accept Linux-later, or (b)
   fall back to a single Rust voice engine (option 3) — but the CEF path is what avoids option 3 entirely.
3. **If a Rust engine becomes necessary (overlay voice, headless mode, or Linux), prefer `str0m` for the
   transport** (active, real BWE/TWCC, clean sans-IO) **paired with `cpal` + the `opus` crate (already present)
   + `webrtc-audio-processing` for AEC3.** Consider **libwebrtc via FFI** instead of pure-Rust if you want the
   most proven media pipeline and can stomach the C++ build. Reserve **webrtc-rs** for when you specifically want
   a `RTCPeerConnection`-shaped API to port existing JS logic — but verify its congestion-control story first.
4. **The condition that flips the whole decision: needing voice *outside* a Chromium WebView.** The moment voice
   must live in the overlay-client, in a headless/background process, or in an SFU, the browser stack stops being
   an option and the Rust stack becomes the right (and only) answer — with the explicit acceptance that you're now
   maintaining the audio-processing pipeline yourself. Note that **Linux is no longer on this list once the CEF
   backend lands** (see point 2): CEF keeps Linux inside a Chromium WebView, so it does not by itself force the
   Rust path.

---

## Sources

Primary (crate docs / source repos / vendor & spec docs):

- str0m — GitHub: https://github.com/algesten/str0m
- str0m — docs.rs: https://docs.rs/str0m
- str0m — crates.io: https://crates.io/crates/str0m
- webrtc-rs `webrtc` — GitHub: https://github.com/webrtc-rs/webrtc
- webrtc-rs `rtc` (sans-IO) — GitHub: https://github.com/webrtc-rs/rtc
- webrtc-rs — crates.io: https://crates.io/crates/webrtc
- webrtc-rs — docs.rs: https://docs.rs/webrtc/latest/webrtc/
- WebRTC.rs blog, "Building Async-Friendly webrtc on Sans-I/O rtc" (2026-01-31): https://webrtc.rs/blog/2026/01/31/async-friendly-webrtc-architecture.html
- WebRTC.rs project site: https://webrtc.rs/
- Tauri v2, "Webview Versions": https://v2.tauri.app/reference/webview-versions/
- wry issue #85, "WebRTC support on Linux": https://github.com/tauri-apps/wry/issues/85
- Tauri discussion #8426, "Functional WebRTC in WebkitGTK on Linux!": https://github.com/orgs/tauri-apps/discussions/8426
- Tauri CEF backend branch `feat/cef` (tauri-apps/tauri): https://github.com/tauri-apps/tauri/tree/feat/cef
- Tauri `feat/cef` commit history (GitHub API): https://api.github.com/repos/tauri-apps/tauri/commits?sha=feat/cef
- `tauri-apps/cef-rs` (official CEF Rust bindings): https://github.com/tauri-apps/cef-rs
- tauri issue #15287 (CEF framework copy in tauri-cli): https://github.com/tauri-apps/tauri/issues/15287
- wry issue #703, "Chromium Embedded Framework as a universal fallback" (closed, priority: low): https://github.com/tauri-apps/wry/issues/703
- Tauri discussion #8524, "Webkit is totally unstable… use chromium or firefox instead" (CEF status quotes): https://github.com/orgs/tauri-apps/discussions/8524
- Tauri blog, "Experimental Tauri Verso Integration" (2025-03-17, Servo-based, not Chromium): https://v2.tauri.app/blog/tauri-verso-integration/
- NLnet, "Servo Webview for Tauri" / "Servo improvements for Tauri (Verso)": https://nlnet.nl/project/Tauri-Servo/ and https://nlnet.nl/project/Verso/
- Tauri discussion #5572, "Use of media devices and desktop sharing": https://github.com/orgs/tauri-apps/discussions/5572
- WebRTC Audio Processing Module g3doc: https://webrtc.googlesource.com/src//+/7c793a7dbe548735fe9e1d107e00d17937202f47/modules/audio_processing/g3doc/audio_processing_module.md
- `webrtc-audio-processing` (tonarino) — README: https://github.com/tonarino/webrtc-audio-processing/blob/main/README.md
- `webrtc-audio-processing` — crates.io: https://crates.io/crates/webrtc-audio-processing
- `webrtc-audio-processing` — docs.rs: https://docs.rs/webrtc-audio-processing
- `sonora` (pure-Rust WebRTC audio processing): https://github.com/dignifiedquire/sonora
- Apple Developer Forums — WKWebView getUserMedia / requestMediaCapturePermissionFor: https://developer.apple.com/forums/thread/734363 and https://developer.apple.com/forums/thread/134216

Secondary (blogs / forums / papers — labeled as such above):

- webrtcHacks, "Probing WebRTC Bandwidth Probing — why and how in gcc": https://webrtchacks.com/probing-webrtc-bandwidth-probing-why-and-how-in-gcc/
- Forasoft, "Bandwidth Estimation and Congestion Control in WebRTC": https://www.forasoft.com/learn/video-streaming/articles-streaming/webrtc-bandwidth-estimation
- pion/webrtc issue #359 (GCC congestion algorithm support): https://github.com/pion/webrtc/issues/359
- pion/webrtc issue #1922 (rate-limiting/BWE/congestion-control): https://github.com/pion/webrtc/issues/1922
- solarana.dev, "Camera and Microphone in WKWebView" (2021): https://solarana.dev/2021/01/01/camera-and-microphone-in-wkwebview/
- libp2p/rust-libp2p issue #3659 ("Switch from webrtc-rs to str0m"): https://github.com/libp2p/rust-libp2p/issues/3659
</content>
</invoke>
