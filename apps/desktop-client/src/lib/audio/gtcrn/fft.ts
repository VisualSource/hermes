// Minimal iterative radix-2 Cooley-Tukey FFT for a fixed power-of-two size.
// Written for GTCRN's 512-point STFT; no external deps so it can run in a
// worklet or the main thread. Correctness over cleverness — 512-point is cheap.

export class FFT {
	readonly size: number;
	private readonly cosT: Float32Array;
	private readonly sinT: Float32Array;
	private readonly rev: Uint32Array;

	constructor(size: number) {
		if ((size & (size - 1)) !== 0)
			throw new Error("FFT size must be power of two");
		this.size = size;

		// Bit-reversal permutation table.
		const bits = Math.log2(size);
		this.rev = new Uint32Array(size);
		for (let i = 0; i < size; i++) {
			let x = i;
			let r = 0;
			for (let b = 0; b < bits; b++) {
				r = (r << 1) | (x & 1);
				x >>= 1;
			}
			this.rev[i] = r >>> 0;
		}

		// Twiddle factors for the forward transform (exp(-j*2*pi*k/N)).
		this.cosT = new Float32Array(size / 2);
		this.sinT = new Float32Array(size / 2);
		for (let k = 0; k < size / 2; k++) {
			const a = (-2 * Math.PI * k) / size;
			this.cosT[k] = Math.cos(a);
			this.sinT[k] = Math.sin(a);
		}
	}

	// In-place complex FFT. `inverse` flips the twiddle sign and scales by 1/N.
	private transform(
		re: Float32Array,
		im: Float32Array,
		inverse: boolean,
	): void {
		const n = this.size;
		const rev = this.rev;
		for (let i = 0; i < n; i++) {
			const j = rev[i];
			if (j > i) {
				let t = re[i];
				re[i] = re[j];
				re[j] = t;
				t = im[i];
				im[i] = im[j];
				im[j] = t;
			}
		}

		const sign = inverse ? -1 : 1;
		for (let len = 2; len <= n; len <<= 1) {
			const half = len >> 1;
			const step = n / len;
			for (let i = 0; i < n; i += len) {
				for (let k = 0, idx = 0; k < half; k++, idx += step) {
					const wr = this.cosT[idx];
					const wi = sign * this.sinT[idx];
					const a = i + k;
					const b = a + half;
					const xr = re[b] * wr - im[b] * wi;
					const xi = re[b] * wi + im[b] * wr;
					re[b] = re[a] - xr;
					im[b] = im[a] - xi;
					re[a] += xr;
					im[a] += xi;
				}
			}
		}

		if (inverse) {
			const inv = 1 / n;
			for (let i = 0; i < n; i++) {
				re[i] *= inv;
				im[i] *= inv;
			}
		}
	}

	forward(re: Float32Array, im: Float32Array): void {
		this.transform(re, im, false);
	}

	inverse(re: Float32Array, im: Float32Array): void {
		this.transform(re, im, true);
	}
}

// sqrt(periodic Hann) — matches torch.hann_window(N).pow(0.5) that GTCRN was
// trained with. periodic => denominator N (not N-1). With 50% overlap this
// window is its own COLA-perfect analysis+synthesis pair (product = Hann,
// overlap-add of Hann at hop N/2 == 1), so no synthesis normalization needed.
export function sqrtHannPeriodic(n: number): Float32Array {
	const w = new Float32Array(n);
	for (let i = 0; i < n; i++) {
		const h = 0.5 * (1 - Math.cos((2 * Math.PI * i) / n));
		w[i] = Math.sqrt(h);
	}
	return w;
}
