# Mic Noise Suppression: Options & Recommendation

> **Where this lives / why.** There is no existing notes/ADR convention in this repo, so
> `apps/desktop-client/docs/` is being introduced here as the sensible home for
> desktop-client design research. This document is research only — no code changes.
>
> **Date:** 2026-07-21. Figures and licenses below were pulled from primary sources
> (official repos, crate registries, MDN, caniuse, papers). Every claim is inline-cited;
> anything I could **not** verify from a primary source is explicitly flagged.

---

## 1. Current setup, and what "better" means here

Hermes' desktop client (`apps/desktop-client/src/lib/audio/noise-suppressor.ts`) runs
**`@sapphi-red/web-noise-suppressor`**: it loads `rnnoise.wasm` / `rnnoise_simd.wasm`,
adds `rnnoiseWorklet.js` as an AudioWorklet module, and wires
`MediaStreamSource -> RnnoiseWorkletNode -> GainNode -> MediaStreamDestination`. The
`dest.stream` is what should become the outgoing WebRTC track in the peer mesh.

That package actually bundles **three** processors — a simple `NoiseGateWorkletNode`, an
`RnnoiseWorkletNode` (RNNoise, wrapping `xiph/rnnoise` via `shiguredo/rnnoise-wasm`), and
a `SpeexWorkletNode` (SpeexDSP preprocess, via `sapphi-red/speex-preprocess-wasm`). It
requires AudioWorklet to function. We only use the RNNoise node.
[[sapphi-red README](https://github.com/sapphi-red/web-noise-suppressor/blob/main/README.md)]

**What "better" can mean for THIS app (a WebRTC peer-mesh voice chat in a Tauri OS-webview):**

- **Better denoise quality**, especially on *non-stationary* noise (keyboard clatter,
  overlapping background speech, reverb) — RNNoise's single-gain-per-band design is weakest
  exactly there. [[jmvalin / Xiph, W3C ML workshop](https://www.w3.org/2020/Talks/mlws/jmv_rnnoise.pdf)]
- **Not regressing echo cancellation (AEC) or auto-gain (AGC).** *This is the trap.* RNNoise,
  DeepFilterNet, and GTCRN are **noise suppressors only — none do AEC**. In a full-duplex
  voice mesh, AEC is what stops you echoing the other person back to them. The browser's
  `getUserMedia` AEC/AGC is currently the only thing providing that, so any change must keep
  it. See §4.
- **Acceptable latency** — this is real-time conversation, so added algorithmic delay matters.
- **Runs in the Tauri OS webview** — WebView2/Chromium (Windows), WKWebView/WebKit (macOS),
  WebKitGTK (Linux). Capability varies by platform (critical for the native-audio idea in §3).
- **Permissive license** — this is a shippable product; avoid copyleft/non-commercial.

---

## 2. Avenue A — Better/other **browser** noise-suppression libraries

| Option | Approach | Quality | Latency (algo) | CPU | License | Maintenance | Effort in this stack |
|---|---|---|---|---|---|---|---|
| **RNNoise** (current, via sapphi-red) | GRU + DSP, 22 bands | Good on steady noise, weak on non-stationary | 10 ms frames (480 smp @48k) | Very low, 1 core | **BSD** (Xiph) | Model itself mature/old | *Already integrated* |
| `@shiguredo/rnnoise-wasm` | Same RNNoise, newer WASM | = RNNoise | 10 ms | Very low | Repo Apache-2.0; **wasm governed by RNNoise COPYING (BSD)** | v2025.1.x, but "PRs/issues only via Discord, Japanese only" | Same as now (sapphi-red already wraps this build) |
| `@jitsi/rnnoise-wasm` | Same RNNoise, emscripten | = RNNoise | 10 ms | Very low | Apache-2.0 (Jitsi) | v0.2.1, ~1 yr | Drop-in alt; still RNNoise-quality |
| **GTCRN** via `sherpa-onnx` WASM | Grouped TCRN, ONNX | **Beats RNNoise "by a substantial margin"** (VCTK-DEMAND, DNS3) | streaming; small | RTF **0.07** on i5-12400; 48.2K params, 33 MMAC/s | Model **MIT**; sherpa-onnx **Apache-2.0** | GTCRN active (Jan 2026), 692★; sherpa-onnx very active | **Best browser upgrade.** Swap WASM+worklet; medium effort |
| **DeepFilterNet** (browser) | Deep filtering, 48kHz | Highest of the RT models (see §metrics) | **40 ms** algo latency | RTF 0.19 on i5-8250U | Code MIT/Apache-2.0; **model license not explicitly stated** (see caveat) | Repo 4.5k★, last release v0.5.6 **Aug 2023** | **No official WASM build.** Community npm exists but unverified — not a first-class path |
| ONNX Runtime Web (generic, NSNet2/DTLN/…) | Run any ONNX SE model | Model-dependent | Model-dependent | **Heavy** (ort-web ≈ 11.8 MB) + historic AudioWorklet trouble | ORT MIT | Active | High effort; sherpa-onnx is the better-packaged version of this idea |
| Speex preprocess (bundled) | Classic DSP | Below RNNoise on hard noise | low | very low | BSD (SpeexDSP) | mature/old | Already available in the same package; a downgrade |

### Notes & metrics

**RNNoise** processes 10 ms frames (480 samples at 48 kHz), predicts gains over 22 bands with
a GRU, runs on a single CPU core, and is **BSD**-licensed (Xiph, Jean-Marc Valin). It is
strong on steady noise (fan/hum) and weak on overlapping speech, sudden clatter, and heavy
reverb.
[[jmvalin W3C](https://www.w3.org/2020/Talks/mlws/jmv_rnnoise.pdf)]
[[deepwiki xiph/rnnoise](https://deepwiki.com/xiph/rnnoise)]
A practical implementation detail (also handled internally by sapphi-red): AudioWorklet
delivers 128-sample quanta while RNNoise wants 480, so a ring buffer is required — Jitsi
documents exactly this.
[[Jitsi blog](https://jitsi.org/blog/enhanced-noise-suppression-in-jitsi-meet/)]

**The two RNNoise WASM builds** are functionally the same model as what we already ship. Note
sapphi-red's `RnnoiseWorkletNode` is *built on* `shiguredo/rnnoise-wasm`, whose README states
the generated wasm's license follows RNNoise's own `COPYING` (BSD), and whose maintenance
policy is "we won't respond to PRs/issues not first discussed on Discord, in Japanese only."
[[shiguredo README](https://github.com/shiguredo/rnnoise-wasm/blob/develop/README.md)]
Switching between these RNNoise builds buys nothing on quality.
[[jitsi/rnnoise-wasm](https://github.com/jitsi/rnnoise-wasm)]

**GTCRN** (Grouped Temporal Convolutional Recurrent Network) is the standout browser upgrade.
Its paper reports it "outperforms RNNoise by a substantial margin on the VCTK-DEMAND and DNS3
dataset" while using only **48.2 K parameters / 33.0 MMACs per second** — i.e. tiny.
[[GTCRN paper (Semantic Scholar)](https://www.semanticscholar.org/paper/GTCRN%3A-A-Speech-Enhancement-Model-Requiring-Rong-Sun/eeee2dcd0491857e9172c36e4f55c6eaaac77529)]
The official repo is **MIT**-licensed, reports RTF **0.07** on an i5-12400, ships streaming
inference, and is ONNX-exportable; it's actively maintained (LADSPA plugin added Jan 2026,
692★).
[[GTCRN repo](https://github.com/Xiaobin-Rong/gtcrn)]
Critically for us, it is already packaged to run **in the browser via WebAssembly**:
`sherpa-onnx` (Apache-2.0) publishes a `speech-enhancement-gtcrn` WASM build with the
`gtcrn_simple.onnx` model, plus an npm package (`sherpa-onnx`).
[[sherpa-onnx repo](https://github.com/k2-fsa/sherpa-onnx)]
[[k2-fsa GTCRN WASM demo](https://huggingface.co/spaces/k2-fsa/wasm-speech-enhancement-gtcrn)]
[[sherpa-onnx SE models release](https://github.com/k2-fsa/sherpa-onnx/releases/tag/speech-enhancement-models)]

**DeepFilterNet** is the quality leader among real-time SE models but is a poor *browser* fit.
Published metrics (VCTK/DEMAND) — WB-PESQ **3.17** (DFN3) vs **3.08** (DFN2), CBAK 3.61 vs 3.40,
STOI 0.944 — with **40 ms total algorithmic latency** and RTF **0.19** on an i5-8250U
(single-threaded); encoder ~200–300 K params, decoders ~100 K each.
[[DeepFilterNet framework summary, from the DFN papers](https://www.emergentmind.com/topics/deepfilternet-framework)]
The code is dual-licensed **MIT/Apache-2.0**; the repo README only ever says "all code" is
dual-licensed and 4.5k★, last release **v0.5.6 (Aug 2023)**.
[[DeepFilterNet repo](https://github.com/Rikorose/DeepFilterNet)]
**There is no official WASM build.** A community npm `deepfilternet3-noise-filter` claims to
be a browser WASM build, but I could not verify it (npm page returned HTTP 403) — treat as
unverified. For us, DeepFilterNet is realistically a *native-side* option (§3), not a browser one.

**ONNX Runtime Web** can run any speech-enhancement ONNX model (NSNet2 — Microsoft's DNS-challenge
baseline; DTLN — <1 M params, real-time) but the runtime is ~11.8 MB and has historically
struggled to run inside an AudioWorklet, which is exactly where real-time audio must live.
`sherpa-onnx`'s purpose-built WASM (used for GTCRN above) is the more practical embodiment of
the "run an ONNX denoiser in the browser" idea.
[[WorkAdventure engineering blog (secondary, implementer)](https://workadventu.re/tech/building-an-easy-to-use-browser-noise-suppression-library-in-an-audio-worklet/)]
[[DTLN repo](https://github.com/breizhn/DTLN)]
*(DTLN's exact SPDX and NSNet2's license were not verified from a primary source here.)*

---

## 3. Avenue B — **Native (Rust/Tauri) processing**

The idea: capture the mic in Rust (`cpal`, already present) or receive it from the webview,
denoise natively, and hand clean audio back. The Rust crates are strong; the **hard part is
getting native audio into the webview's WebRTC PeerConnection.**

### 3a. The Rust denoising crates

| Crate | What | Quality | License | Maintenance | Notes |
|---|---|---|---|---|---|
| `deep_filter` (DeepFilterNet's libDF) | DeepFilterNet in Rust | Best RT quality (see §2) | MIT/Apache-2.0 | crates.io lags repo (see below) | Real-time STFT/ISTFT, `--compensate-delay`; inference via ONNX/tract; LADSPA/PipeWire plugin exists for desktop |
| `nnnoiseless` | Pure-Rust RNNoise port | = RNNoise | (RNNoise-derived; verify) | jneem/nnnoiseless | No libclang, **compiles on Windows**, faster than the C version |
| `rnnoise-c` | C bindings to Xiph RNNoise | = RNNoise | — | **Deprecated** in favor of nnnoiseless | Don't use |
| `webrtc-audio-processing` (tonarino) | Full WebRTC APM: **NS + AEC + AGC + VAD** | Browser-grade APM | wrapper via `COPYING`; underlying WebRTC APM is **BSD-3-Clause** | v2.1.0 | Build needs meson/ninja/C++ (Linux-oriented); `bundled` feature builds from source. **Only option here that gives AEC natively.** |
| `sonora` (dignifiedquire) | **Pure-Rust** WebRTC APM (AEC/NS/AGC) | APM-grade | (verify) | Newer/emerging | Attractive if it matures — no C++ build |

Details:
- **`deep_filter`**: dual **MIT/Apache-2.0**, includes a "real-time STFT/ISTFT implementation"
  with `--compensate-delay`. Note a **version discrepancy**: lib.rs shows `deep_filter` 0.2.5
  (Jul 2022) with low downloads (~584/mo), while the DeepFilterNet repo's tagged release is
  v0.5.6 (Aug 2023) — the crates.io publication trails the git repo, so pin against the repo if
  you go this route.
  [[deep_filter on lib.rs](https://lib.rs/crates/deep_filter)]
  [[DeepFilterNet repo](https://github.com/Rikorose/DeepFilterNet)]
- **`nnnoiseless`**: a safe pure-Rust port of RNNoise; the author states it needs no libclang,
  compiles on Windows, and is faster than the C version. Same *quality* as RNNoise, so it's only
  interesting for native pipelines. *(Exact SPDX not verified from primary source.)*
  [[nnnoiseless repo](https://github.com/jneem/nnnoiseless)]
- **`webrtc-audio-processing`** (tonarino): wraps PulseAudio's repackaging of WebRTC's
  AudioProcessing module — NS, AEC, AGC2, VAD. v2.1.0. It uses a `license-file = "COPYING"`
  rather than an inline SPDX (the underlying Google WebRTC APM is BSD-3-Clause). Build needs
  `clang`/`gcc`, `pkg-config`, `meson`, `ninja`; the `bundled` feature compiles the C++ in-tree.
  This is Linux-first and the C++ build is friction on Windows.
  [[tonarino README](https://github.com/tonarino/webrtc-audio-processing/blob/main/README.md)]
- **`sonora`**: a pure-Rust reimplementation of WebRTC audio processing (AEC/NS/AGC) — no C++
  toolchain. Promising but younger; verify maturity before relying on it.
  [[sonora repo](https://github.com/dignifiedquire/sonora)]

### 3b. The integration problem (the real blocker)

The `RTCPeerConnection` runs **inside the webview's browser engine**. You cannot hand it a raw
PCM buffer produced by Rust as if it were a capture device — the browser WebRTC API exposes no
"set custom native audio source." So native denoising forces one of these:

1. **Feed native PCM back into the webview and build a track with Insertable Streams.**
   Rust denoises → send PCM over Tauri IPC / a `SharedArrayBuffer` → an AudioWorklet →
   `MediaStreamTrackGenerator` builds a `MediaStreamTrack` you add to the PeerConnection.
   - **Platform reality:** `MediaStreamTrackGenerator` is supported in **Chrome/Edge 94+**, so
     it works in **WebView2 on Windows**. It is **NOT supported in Safari / WebKit**, so it
     will **not** work in WKWebView (macOS) and is unreliable in WebKitGTK (Linux).
     [[caniuse: MediaStreamTrackGenerator](https://caniuse.com/mdn-api_mediastreamtrackgenerator)]
     [[W3C Mediacapture-transform](https://www.w3.org/TR/mediacapture-transform/)]
   - `SharedArrayBuffer` additionally requires **cross-origin isolation (COOP/COEP headers)**,
     which you'd have to arrange for Tauri's custom protocol.
   - Verdict: **Windows-only**, fiddly, and you're doing a round-trip (webview→Rust→webview)
     just to avoid the WASM worklet that already works everywhere. Poor cost/benefit.

2. **Bypass WebRTC audio entirely — encode Opus natively and send over your own transport.**
   The `opus` crate + `cpal` already exist on the Tauri side. You'd capture+denoise+encode in
   Rust and ship Opus frames over the app's own path (the existing WS relay or a data channel).
   - This is a **major architecture change**: you lose WebRTC's audio jitter buffer, PLC,
     RTP timing, congestion control, and — importantly — its **AEC**, all of which you'd have to
     re-implement or accept losing. It also splits voice out of the peer-mesh WebRTC model the
     app is built around. Only worth it if you were moving off webview-hosted WebRTC anyway.

3. **Replace the WebRTC audio track's source.** Not possible via standard browser WebRTC APIs;
   there is no supported hook to swap in a native source. (Would require a custom/patched
   engine — out of scope.)

**Bottom line on native:** the crates are excellent, but for a webview-hosted WebRTC mesh the
only "clean" native→PC bridge (Insertable Streams) is **Windows/WebView2-only** and adds a
webview↔Rust round-trip. It does not beat running the same-quality model as WASM in the worklet,
unless you specifically want **native AEC** (webrtc-audio-processing / sonora) *and* accept
option 2's rearchitecture.

---

## 4. Avenue C — Built-in browser / WebRTC audio processing

`getUserMedia({ audio: { noiseSuppression, echoCancellation, autoGainControl } })` toggles the
engine's built-in APM. These are **hints by default** (booleans) and only mandatory when wrapped
as `{ exact: true }`; unknown constraints are ignored. Support is "limited availability / not
Baseline," so check `navigator.mediaDevices.getSupportedConstraints()`.
[[MDN: noiseSuppression](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/noiseSuppression)]

| Aspect | Browser built-in APM |
|---|---|
| Quality (NS) | Decent on stationary noise; the whole reason Jitsi/WorkAdventure/etc. bolt on RNNoise is that built-in NS underperforms on harder noise. *(No primary head-to-head benchmark vs RNNoise found — flagged.)* |
| **AEC / AGC** | **Its key advantage — provides echo cancellation and auto-gain, which RNNoise/DFN/GTCRN do NOT.** |
| Latency | Integrated in the capture pipeline; negligible extra |
| CPU | Free (native, in-engine) |
| License | N/A (platform) |
| Tauri support | **WebView2 (Windows) = Chromium → implemented.** WKWebView / WebKitGTK → partial/varies; verify per platform. |

The important architectural point: **built-in APM and a WASM denoiser are complementary, not
either/or.** The right pattern is to keep `echoCancellation: true` and `autoGainControl: true`
in `getUserMedia` (so the browser handles AEC/AGC, which the ML denoisers can't), and then run
your ML noise suppressor on that already-echo-cancelled stream. Whether to also set the
browser's own `noiseSuppression: true` is a tuning question — running two NS stages can
over-suppress or add artifacts; typically you set the browser `noiseSuppression: false` and let
RNNoise/GTCRN be the sole NS stage, while keeping browser AEC/AGC on. Jitsi Meet's own approach
is to run RNNoise in an AudioWorklet rather than rely on built-in NS.
[[Jitsi blog](https://jitsi.org/blog/enhanced-noise-suppression-in-jitsi-meet/)]

> **Action item to check in our code:** confirm the `getUserMedia` call that feeds
> `noise-suppressor.ts` requests `echoCancellation: true` + `autoGainControl: true`. If the mic
> stream was captured with those off (e.g. `echoCancellation:false`), we are shipping voice with
> **no echo cancellation at all**, which RNNoise does nothing to fix. That would be the single
> most impactful thing to correct.

---

## 5. Recommendation

Given a **Tauri OS-webview app doing a WebRTC peer mesh**, ranked:

### #1 — Cheapest win (do this first, ~an afternoon)
**Fix the capture constraints, keep the existing RNNoise worklet.** Make sure the mic
`getUserMedia` keeps `echoCancellation: true` and `autoGainControl: true` (browser AEC/AGC),
and let RNNoise be the noise-suppression stage on top. This costs almost nothing and closes the
biggest possible gap (missing AEC). No new dependency. See §4 action item.

### #2 — Best *practical* quality upgrade (browser, cross-platform)
**Replace the RNNoise WASM node with GTCRN via `sherpa-onnx`'s WASM speech-enhancement build.**
- Meaningfully better than RNNoise on non-stationary noise (per the GTCRN paper), still tiny
  (48.2 K params, RTF ~0.07), **MIT** model + **Apache-2.0** runtime, actively maintained, and
  it already runs in an AudioWorklet-style WASM harness — so it drops into the exact place
  `noise-suppressor.ts` already occupies, on **all three** webview platforms.
  [[GTCRN repo](https://github.com/Xiaobin-Rong/gtcrn)]
  [[sherpa-onnx SE WASM](https://github.com/k2-fsa/sherpa-onnx/releases/tag/speech-enhancement-models)]
- Effort: medium (integrate sherpa-onnx WASM + model asset + worklet glue, keep the same
  `source -> node -> gain -> MediaStreamDestination` graph). Keep browser AEC/AGC on per #1.
- Combine with #1, not instead of it.

### #3 — Best raw quality, if you can absorb the cost
**DeepFilterNet.** Highest metrics of the real-time models, MIT/Apache-2.0 code. But: **no
official browser WASM**, 40 ms added latency, higher CPU, and last release Aug 2023. It's really
a *native* (`deep_filter`) option, which then runs into the §3b integration wall. Only pursue if
GTCRN quality proves insufficient in real use **and** you're willing to take on native
integration. Not recommended as a near-term move.

### #4 — Native Rust processing / native AEC
Only reconsider if you either (a) specifically need **better/native AEC** than the browser's
(then `webrtc-audio-processing` or, once mature, pure-Rust `sonora`), or (b) are already moving
voice **off** webview-hosted WebRTC onto a native Opus transport (`opus`+`cpal` are present).
For the current peer-mesh-in-webview design, the native→PeerConnection bridge is
**Windows/WebView2-only** (Insertable Streams / `MediaStreamTrackGenerator` unsupported in
WebKit) and adds a webview↔Rust round-trip for no quality gain over running the same model as
WASM. **Not recommended now.**

**Net:** do **#1 + #2** — keep browser AEC/AGC, swap RNNoise → GTCRN (sherpa-onnx WASM). That's
the best quality-per-effort for this stack and stays cross-platform. Hold DeepFilterNet and the
native path in reserve.

---

## 6. Claims I could NOT verify from a primary source

- **DeepFilterNet model-weight license.** The repo README dual-licenses "all code" MIT/Apache-2.0
  but does **not** explicitly state a license for the pretrained *weights*; I found no separate
  restrictive (CC-BY-NC / non-commercial) model license, but I also could not find an explicit
  affirmative statement that the weights are MIT/Apache. Verify before shipping commercially.
  [[DeepFilterNet repo](https://github.com/Rikorose/DeepFilterNet)]
- **`deepfilternet3-noise-filter` npm** (claimed browser WASM build of DFN3): npm page returned
  HTTP 403; could not confirm it's real/functional/official. Treated as unverified.
- **Exact SPDX for `nnnoiseless`, DTLN, NSNet2, and the tonarino `webrtc-audio-processing`
  wrapper** (the last uses `license-file = "COPYING"`, not an inline SPDX; the underlying Google
  WebRTC APM is BSD-3-Clause). Confirm each COPYING/LICENSE file before adoption.
- **A direct, primary-source benchmark of Chrome's built-in WebRTC NS vs RNNoise.** No clean
  head-to-head from an authoritative source was found; the "built-in NS is weaker on hard noise"
  claim rests on implementers (Jitsi/WorkAdventure) choosing to add RNNoise, not on a published
  A/B metric.

---

## Sources

- @sapphi-red/web-noise-suppressor README — https://github.com/sapphi-red/web-noise-suppressor/blob/main/README.md
- shiguredo/rnnoise-wasm README — https://github.com/shiguredo/rnnoise-wasm/blob/develop/README.md
- jitsi/rnnoise-wasm — https://github.com/jitsi/rnnoise-wasm
- Jitsi "Enhanced noise suppression in Jitsi Meet" — https://jitsi.org/blog/enhanced-noise-suppression-in-jitsi-meet/
- Jean-Marc Valin / Xiph, RNNoise (W3C ML workshop) — https://www.w3.org/2020/Talks/mlws/jmv_rnnoise.pdf
- xiph/rnnoise (DeepWiki mirror of the repo) — https://deepwiki.com/xiph/rnnoise
- DeepFilterNet repo — https://github.com/Rikorose/DeepFilterNet
- DeepFilterNet framework metrics (summarizing the DFN papers) — https://www.emergentmind.com/topics/deepfilternet-framework
- deep_filter crate — https://lib.rs/crates/deep_filter
- GTCRN repo (Xiaobin-Rong) — https://github.com/Xiaobin-Rong/gtcrn
- GTCRN paper — https://www.semanticscholar.org/paper/GTCRN%3A-A-Speech-Enhancement-Model-Requiring-Rong-Sun/eeee2dcd0491857e9172c36e4f55c6eaaac77529
- sherpa-onnx repo — https://github.com/k2-fsa/sherpa-onnx
- sherpa-onnx speech-enhancement models release — https://github.com/k2-fsa/sherpa-onnx/releases/tag/speech-enhancement-models
- k2-fsa GTCRN WASM demo (Hugging Face Space) — https://huggingface.co/spaces/k2-fsa/wasm-speech-enhancement-gtcrn
- nnnoiseless repo — https://github.com/jneem/nnnoiseless
- tonarino/webrtc-audio-processing README — https://github.com/tonarino/webrtc-audio-processing/blob/main/README.md
- sonora (pure-Rust WebRTC audio processing) — https://github.com/dignifiedquire/sonora
- DTLN repo — https://github.com/breizhn/DTLN
- WorkAdventure — browser noise suppression in an AudioWorklet — https://workadventu.re/tech/building-an-easy-to-use-browser-noise-suppression-library-in-an-audio-worklet/
- MDN — MediaTrackConstraints.noiseSuppression — https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/noiseSuppression
- caniuse — MediaStreamTrackGenerator — https://caniuse.com/mdn-api_mediastreamtrackgenerator
- W3C — MediaStreamTrack Insertable Media Processing (Mediacapture-transform) — https://www.w3.org/TR/mediacapture-transform/
