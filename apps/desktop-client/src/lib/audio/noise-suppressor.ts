import rnnoiseWorkletPath from "@sapphi-red/web-noise-suppressor/rnnoiseWorklet.js?url";
import rnnoiseWasmPath from "@sapphi-red/web-noise-suppressor/rnnoise.wasm?url";
import rnnoiseWasmSimdPath from "@sapphi-red/web-noise-suppressor/rnnoise_simd.wasm?url";
import {
	loadRnnoise,
	RnnoiseWorkletNode,
} from "@sapphi-red/web-noise-suppressor";

export class NoiseSuppressor {
	static INSTANCE: NoiseSuppressor | null = null;
	static get(): NoiseSuppressor {
		if (!NoiseSuppressor.INSTANCE)
			throw new Error("noise suppresspr is not ready");

		return NoiseSuppressor.INSTANCE;
	}

	static async create() {
		if (NoiseSuppressor.INSTANCE) throw new Error("already inited");
		const bin = await loadRnnoise({
			url: rnnoiseWasmPath,
			simdUrl: rnnoiseWasmSimdPath,
		});

		NoiseSuppressor.INSTANCE = new NoiseSuppressor(bin);
	}

	private constructor(private wasm: ArrayBuffer) {
		console.info("[Noise Suppressor] Ready");
	}

	public async createProcesser(stream: MediaStream) {
		const ctx = new AudioContext();

		await ctx.audioWorklet.addModule(rnnoiseWorkletPath);

		const source = ctx.createMediaStreamSource(stream);
		const dest = ctx.createMediaStreamDestination();

		const node = new RnnoiseWorkletNode(ctx, {
			wasmBinary: this.wasm,
			maxChannels: 2,
		});

		const gain = new GainNode(ctx, { gain: 1 });

		source.connect(node);
		node.connect(gain);

		gain.connect(dest);

		return dest.stream;
	}
}
