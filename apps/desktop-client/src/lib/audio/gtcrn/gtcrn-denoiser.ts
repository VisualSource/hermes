// Streaming GTCRN speech denoiser running the ONNX model via onnxruntime-web.
//
// The ONNX graph is the per-frame streaming export from
// https://github.com/Xiaobin-Rong/gtcrn (stream/onnx_models/gtcrn.onnx).
// The model does NOT contain the STFT — the caller feeds one complex spectrum
// frame (1,257,1,2) plus three recurrent caches, and gets back the enhanced
// frame plus updated caches. STFT/iSTFT (n_fft=512, hop=256, sqrt-Hann, 16kHz)
// is done here in JS, matching the reference gtcrn_stream.py.

import type { InferenceSession, Tensor } from "onnxruntime-web";
import { FFT, sqrtHannPeriodic } from "./fft";

export const GTCRN_SAMPLE_RATE = 16000;
export const N_FFT = 512;
export const HOP = 256;
const N_BINS = N_FFT / 2 + 1; // 257

// Cache tensor shapes, verified against the model's declared inputs.
const CONV_CACHE_DIMS = [2, 1, 16, 16, 33];
const TRA_CACHE_DIMS = [2, 3, 1, 1, 16];
const INTER_CACHE_DIMS = [2, 1, 33, 16];

function numel(dims: number[]): number {
	return dims.reduce((a, b) => a * b, 1);
}

export class GtcrnDenoiser {
	private ort: typeof import("onnxruntime-web");
	private session: InferenceSession;
	private fft = new FFT(N_FFT);
	private win = sqrtHannPeriodic(N_FFT);

	// Streaming state.
	private frameBuf = new Float32Array(N_FFT); // sliding 512-sample analysis window
	private olaBuf = new Float32Array(N_FFT); // overlap-add accumulator
	private convCache!: Tensor;
	private traCache!: Tensor;
	private interCache!: Tensor;

	// scratch
	private re = new Float32Array(N_FFT);
	private im = new Float32Array(N_FFT);
	private mixData = new Float32Array(N_BINS * 2);

	private constructor(
		ort: typeof import("onnxruntime-web"),
		session: InferenceSession,
	) {
		this.ort = ort;
		this.session = session;
		this.reset();
	}

	static async create(modelUrl: string): Promise<GtcrnDenoiser> {
		const ort = await import("onnxruntime-web");
		// Single-threaded WASM: no SharedArrayBuffer / cross-origin isolation
		// needed. GTCRN is tiny (~48k params) so this is plenty fast.
		ort.env.wasm.numThreads = 1;
		if (!ort.env.wasm.wasmPaths) {
			// Dev-only convenience: pull the ORT wasm from a CDN so we don't have
			// to wire the assets through Vite for the prototype.
			ort.env.wasm.wasmPaths =
				"https://cdn.jsdelivr.net/npm/onnxruntime-web@1.27.0/dist/";
		}
		const session = await ort.InferenceSession.create(modelUrl, {
			executionProviders: ["wasm"],
			graphOptimizationLevel: "all",
		});
		return new GtcrnDenoiser(ort, session);
	}

	// Clear all streaming state so the next hop starts a fresh utterance.
	reset(): void {
		this.frameBuf.fill(0);
		this.olaBuf.fill(0);
		const f32 = (dims: number[]) =>
			new this.ort.Tensor("float32", new Float32Array(numel(dims)), dims);
		this.convCache = f32(CONV_CACHE_DIMS);
		this.traCache = f32(TRA_CACHE_DIMS);
		this.interCache = f32(INTER_CACHE_DIMS);
	}

	get inputNames(): readonly string[] {
		return this.session.inputNames;
	}
	get outputNames(): readonly string[] {
		return this.session.outputNames;
	}

	// Push one 256-sample hop of 16kHz mono audio, get back 256 enhanced samples.
	// Output is delayed by ~one analysis frame (STFT/OLA latency).
	async processHop(hop: Float32Array): Promise<Float32Array> {
		if (hop.length !== HOP) throw new Error(`hop must be ${HOP} samples`);

		// Slide the analysis window: drop oldest 256, append newest 256.
		this.frameBuf.copyWithin(0, HOP);
		this.frameBuf.set(hop, N_FFT - HOP);

		// Windowed forward FFT.
		for (let i = 0; i < N_FFT; i++) {
			this.re[i] = this.frameBuf[i] * this.win[i];
			this.im[i] = 0;
		}
		this.fft.forward(this.re, this.im);

		// Pack the one-sided spectrum into the model's (1,257,1,2) layout.
		for (let f = 0; f < N_BINS; f++) {
			this.mixData[f * 2] = this.re[f];
			this.mixData[f * 2 + 1] = this.im[f];
		}

		const mix = new this.ort.Tensor("float32", this.mixData, [1, N_BINS, 1, 2]);
		const out = await this.session.run({
			mix,
			conv_cache: this.convCache,
			tra_cache: this.traCache,
			inter_cache: this.interCache,
		});
		this.convCache = out.conv_cache_out as Tensor;
		this.traCache = out.tra_cache_out as Tensor;
		this.interCache = out.inter_cache_out as Tensor;

		const enh = out.enh.data as Float32Array; // (1,257,1,2)

		// Rebuild the full 512-bin Hermitian-symmetric spectrum.
		for (let f = 0; f < N_BINS; f++) {
			this.re[f] = enh[f * 2];
			this.im[f] = enh[f * 2 + 1];
		}
		for (let f = 1; f < N_BINS - 1; f++) {
			this.re[N_FFT - f] = this.re[f];
			this.im[N_FFT - f] = -this.im[f];
		}

		this.fft.inverse(this.re, this.im);

		// Synthesis window + overlap-add. sqrt-Hann on both sides => COLA==1.
		const output = new Float32Array(HOP);
		for (let i = 0; i < N_FFT; i++) this.olaBuf[i] += this.re[i] * this.win[i];
		output.set(this.olaBuf.subarray(0, HOP));
		this.olaBuf.copyWithin(0, HOP);
		this.olaBuf.fill(0, N_FFT - HOP);

		return output;
	}

	// Offline convenience: denoise a whole 16kHz mono buffer, returning the
	// enhanced signal plus timing so we can compute the real-time factor.
	async processBuffer(signal: Float32Array): Promise<{
		out: Float32Array;
		frames: number;
		totalMs: number;
		perFrameMs: number;
		rtf: number;
	}> {
		this.reset();
		const nHops = Math.floor(signal.length / HOP);
		const out = new Float32Array(nHops * HOP);
		const t0 = performance.now();
		for (let h = 0; h < nHops; h++) {
			const hop = signal.subarray(h * HOP, h * HOP + HOP);
			const enh = await this.processHop(hop);
			out.set(enh, h * HOP);
		}
		const totalMs = performance.now() - t0;
		const audioMs = (out.length / GTCRN_SAMPLE_RATE) * 1000;
		return {
			out,
			frames: nHops,
			totalMs,
			perFrameMs: nHops ? totalMs / nHops : 0,
			rtf: audioMs ? totalMs / audioMs : 0,
		};
	}
}
