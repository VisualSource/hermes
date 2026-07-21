// Standalone prototype harness for GTCRN noise suppression via onnxruntime-web.
// Not part of the app — served as its own Vite entry (denoise-proto.html) so it
// stays clear of the router/auth. Run `pnpm dev` and open
// http://localhost:1420/denoise-proto.html
//
// Two modes:
//   1. File A/B  — decode a file, resample to 16k, run the streaming pipeline
//      offline, report RTF (real-time factor) and play original vs enhanced.
//   2. Live mic  — real streaming: mic -> 16k AudioContext -> capture worklet
//      -> main-thread ONNX -> playback worklet. Use HEADPHONES (it monitors).

import {
	GtcrnDenoiser,
	GTCRN_SAMPLE_RATE,
} from "@/lib/audio/gtcrn/gtcrn-denoiser";
import workletUrl from "./gtcrn-io-worklet.js?url";

const MODEL_URL = "/models/gtcrn.onnx";

const $ = (id: string) => document.getElementById(id)!;
const logEl = $("log");
function log(msg: string) {
	console.log(msg);
	logEl.textContent += `${msg}\n`;
	logEl.scrollTop = logEl.scrollHeight;
}

let denoiser: GtcrnDenoiser | null = null;

async function getDenoiser(): Promise<GtcrnDenoiser> {
	if (denoiser) return denoiser;
	log("Loading ONNX model + ORT wasm…");
	const t0 = performance.now();
	denoiser = await GtcrnDenoiser.create(MODEL_URL);
	log(`Model ready in ${(performance.now() - t0).toFixed(0)}ms`);
	log(`  inputs:  ${denoiser.inputNames.join(", ")}`);
	log(`  outputs: ${denoiser.outputNames.join(", ")}`);
	return denoiser;
}

// --- WAV encode so we can drop buffers straight into <audio controls> ---------
function encodeWav(samples: Float32Array, sampleRate: number): Blob {
	const buffer = new ArrayBuffer(44 + samples.length * 2);
	const view = new DataView(buffer);
	const writeStr = (off: number, s: string) => {
		for (let i = 0; i < s.length; i++) view.setUint8(off + i, s.charCodeAt(i));
	};
	writeStr(0, "RIFF");
	view.setUint32(4, 36 + samples.length * 2, true);
	writeStr(8, "WAVE");
	writeStr(12, "fmt ");
	view.setUint32(16, 16, true);
	view.setUint16(20, 1, true); // PCM
	view.setUint16(22, 1, true); // mono
	view.setUint32(24, sampleRate, true);
	view.setUint32(28, sampleRate * 2, true);
	view.setUint16(32, 2, true);
	view.setUint16(34, 16, true);
	writeStr(36, "data");
	view.setUint32(40, samples.length * 2, true);
	let off = 44;
	for (let i = 0; i < samples.length; i++) {
		const s = Math.max(-1, Math.min(1, samples[i]));
		view.setInt16(off, s < 0 ? s * 0x8000 : s * 0x7fff, true);
		off += 2;
	}
	return new Blob([buffer], { type: "audio/wav" });
}

async function decodeAndResample(file: File): Promise<Float32Array> {
	const bytes = await file.arrayBuffer();
	const tmpCtx = new AudioContext();
	const decoded = await tmpCtx.decodeAudioData(bytes);
	await tmpCtx.close();
	// Resample to 16k mono via OfflineAudioContext.
	const frames = Math.ceil(decoded.duration * GTCRN_SAMPLE_RATE);
	const off = new OfflineAudioContext(1, frames, GTCRN_SAMPLE_RATE);
	const src = off.createBufferSource();
	src.buffer = decoded;
	src.connect(off.destination);
	src.start();
	const rendered = await off.startRendering();
	return rendered.getChannelData(0).slice();
}

// --- File A/B mode -----------------------------------------------------------
async function runFile(file: File) {
	try {
		const d = await getDenoiser();
		log(`\nDecoding "${file.name}"…`);
		const input = await decodeAndResample(file);
		const secs = (input.length / GTCRN_SAMPLE_RATE).toFixed(2);
		log(`Resampled to 16k mono: ${input.length} samples (${secs}s)`);

		log("Denoising (streaming, frame-by-frame)…");
		const res = await d.processBuffer(input);
		log(
			`Done: ${res.frames} frames, total ${res.totalMs.toFixed(0)}ms, ` +
				`${res.perFrameMs.toFixed(3)}ms/frame, RTF=${res.rtf.toFixed(3)} ` +
				`(${res.rtf < 1 ? "REAL-TIME CAPABLE ✅" : "too slow ❌"})`,
		);

		const origUrl = URL.createObjectURL(encodeWav(input, GTCRN_SAMPLE_RATE));
		const enhUrl = URL.createObjectURL(encodeWav(res.out, GTCRN_SAMPLE_RATE));
		($("audioOrig") as HTMLAudioElement).src = origUrl;
		($("audioEnh") as HTMLAudioElement).src = enhUrl;
		$("abResult").classList.remove("hidden");
	} catch (err) {
		log(`ERROR: ${(err as Error).message}`);
		console.error(err);
	}
}

// --- Live mic mode -----------------------------------------------------------
let liveCtx: AudioContext | null = null;
let liveStream: MediaStream | null = null;

async function startLive() {
	try {
		const d = await getDenoiser();
		d.reset();
		log("\nStarting live mic (use headphones — this monitors your mic)…");

		liveCtx = new AudioContext({ sampleRate: GTCRN_SAMPLE_RATE });
		log(`AudioContext sampleRate = ${liveCtx.sampleRate}Hz`);

		liveStream = await navigator.mediaDevices.getUserMedia({
			audio: {
				echoCancellation: true, // keep browser AEC/AGC; GTCRN only does NS
				autoGainControl: true,
				noiseSuppression: false, // let GTCRN be the only NS stage
			},
		});

		await liveCtx.audioWorklet.addModule(workletUrl);
		const node = new AudioWorkletNode(liveCtx, "gtcrn-io", {
			numberOfInputs: 1,
			numberOfOutputs: 1,
			outputChannelCount: [1],
		});

		// Serialize hops so the stateful caches stay in order.
		let chain: Promise<void> = Promise.resolve();
		node.port.onmessage = (e: MessageEvent<Float32Array>) => {
			const hop = e.data;
			chain = chain.then(async () => {
				const enh = await d.processHop(hop);
				node.port.postMessage(enh, [enh.buffer]);
			});
		};

		const src = liveCtx.createMediaStreamSource(liveStream);
		src.connect(node);
		node.connect(liveCtx.destination);

		$("btnStart").setAttribute("disabled", "true");
		$("btnStop").removeAttribute("disabled");
		log("Live: mic → GTCRN → speakers. Talk!");
	} catch (err) {
		log(`ERROR: ${(err as Error).message}`);
		console.error(err);
	}
}

function stopLive() {
	for (const t of liveStream?.getTracks() ?? []) t.stop();
	liveCtx?.close();
	liveStream = null;
	liveCtx = null;
	$("btnStart").removeAttribute("disabled");
	$("btnStop").setAttribute("disabled", "true");
	log("Live stopped.");
}

// --- wire up -----------------------------------------------------------------
($("fileInput") as HTMLInputElement).addEventListener("change", (e) => {
	const file = (e.target as HTMLInputElement).files?.[0];
	if (file) runFile(file);
});
$("btnStart").addEventListener("click", startLive);
$("btnStop").addEventListener("click", stopLive);
log("Ready. Pick a file for A/B, or start the live mic.");
