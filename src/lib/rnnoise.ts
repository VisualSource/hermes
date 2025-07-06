import rnnoiseWorkletPath from '@sapphi-red/web-noise-suppressor/rnnoiseWorklet.js?url';
import rnnoiseWasmPath from '@sapphi-red/web-noise-suppressor/rnnoise.wasm?url';
import rnnoiseWasmSimdPath from '@sapphi-red/web-noise-suppressor/rnnoise_simd.wasm?url';
import {
    loadRnnoise,
    RnnoiseWorkletNode,
} from '@sapphi-red/web-noise-suppressor';

export class RnnoiseManager {
    private binary: ArrayBuffer | null = null;

    async init() {
        this.binary = await loadRnnoise({
            url: rnnoiseWasmPath,
            simdUrl: rnnoiseWasmSimdPath
        });
    }

    async createProcesser(stream: MediaStream) {
        if (!this.binary) throw new Error("Wasm binary was not loaded");
        const ctx = new AudioContext();

        await ctx.audioWorklet.addModule(rnnoiseWorkletPath);

        const source = ctx.createMediaStreamSource(stream);
        const dest = ctx.createMediaStreamDestination();

        const node = new RnnoiseWorkletNode(ctx, {
            wasmBinary: this.binary,
            maxChannels: 2
        });
        const gain = new GainNode(ctx, { gain: 1 });

        source.connect(node);
        node.connect(gain);

        gain.connect(dest);

        return dest.stream;
    }
}