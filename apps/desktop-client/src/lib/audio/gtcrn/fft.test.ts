import { describe, it, expect } from "vitest";
import { FFT, sqrtHannPeriodic } from "./fft";

// Naive DFT reference (forward, exp(-j2pi kn/N), no normalization).
function naiveDft(re: number[]): { re: number[]; im: number[] } {
	const N = re.length;
	const outRe = new Array(N).fill(0);
	const outIm = new Array(N).fill(0);
	for (let k = 0; k < N; k++) {
		for (let n = 0; n < N; n++) {
			const a = (-2 * Math.PI * k * n) / N;
			outRe[k] += re[n] * Math.cos(a);
			outIm[k] += re[n] * Math.sin(a);
		}
	}
	return { re: outRe, im: outIm };
}

describe("FFT", () => {
	it("forward matches naive DFT", () => {
		const N = 16;
		const x = Array.from({ length: N }, (_, i) => Math.sin(i) + 0.3 * i);
		const ref = naiveDft(x);
		const re = Float32Array.from(x);
		const im = new Float32Array(N);
		new FFT(N).forward(re, im);
		for (let k = 0; k < N; k++) {
			expect(re[k]).toBeCloseTo(ref.re[k], 3);
			expect(im[k]).toBeCloseTo(ref.im[k], 3);
		}
	});

	it("inverse undoes forward", () => {
		const N = 512;
		const x = Float32Array.from({ length: N }, () => Math.random() * 2 - 1);
		const re = x.slice();
		const im = new Float32Array(N);
		const fft = new FFT(N);
		fft.forward(re, im);
		fft.inverse(re, im);
		for (let i = 0; i < N; i++) expect(re[i]).toBeCloseTo(x[i], 5);
	});
});

// Replicates the exact analysis/synthesis path in gtcrn-denoiser (minus the
// ONNX model, i.e. identity spectrum) to prove the STFT->iSTFT round trip
// reconstructs the signal. If this holds, the enhanced audio can only differ
// from the input by what the model does — not by DSP bugs.
describe("STFT/iSTFT reconstruction (sqrt-Hann, hop 256)", () => {
	it("reconstructs interior samples to near-identity", () => {
		const N_FFT = 512;
		const HOP = 256;
		const N_BINS = N_FFT / 2 + 1;
		const win = sqrtHannPeriodic(N_FFT);
		const fft = new FFT(N_FFT);

		const len = 4096;
		const signal = Float32Array.from(
			{ length: len },
			(_, i) => Math.sin(i * 0.05) * 0.5 + Math.sin(i * 0.31) * 0.3,
		);

		const frameBuf = new Float32Array(N_FFT);
		const ola = new Float32Array(N_FFT);
		const re = new Float32Array(N_FFT);
		const im = new Float32Array(N_FFT);
		const out = new Float32Array(len);

		const nHops = Math.floor(len / HOP);
		for (let h = 0; h < nHops; h++) {
			frameBuf.copyWithin(0, HOP);
			frameBuf.set(signal.subarray(h * HOP, h * HOP + HOP), N_FFT - HOP);

			for (let i = 0; i < N_FFT; i++) {
				re[i] = frameBuf[i] * win[i];
				im[i] = 0;
			}
			fft.forward(re, im);
			// identity: keep one-sided bins, rebuild Hermitian, inverse
			for (let f = 1; f < N_BINS - 1; f++) {
				re[N_FFT - f] = re[f];
				im[N_FFT - f] = -im[f];
			}
			fft.inverse(re, im);
			for (let i = 0; i < N_FFT; i++) ola[i] += re[i] * win[i];
			out.set(ola.subarray(0, HOP), h * HOP);
			ola.copyWithin(0, HOP);
			ola.fill(0, N_FFT - HOP);
		}

		// The pipeline delays output by one hop (256 samples): the first full,
		// zero-pad-free analysis frame is signal[0..511], and WOLA places its
		// output starting at index HOP. Compare interior against that shift.
		const delay = HOP;
		let maxErr = 0;
		for (let i = delay; i < len - N_FFT; i++) {
			maxErr = Math.max(maxErr, Math.abs(out[i] - signal[i - delay]));
		}
		expect(maxErr).toBeLessThan(1e-3);
	});
});
