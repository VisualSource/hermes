// Capture + playback AudioWorklet for the GTCRN live-mic prototype.
// Runs inside a 16 kHz AudioContext, so process() gets 128-sample blocks.
//
// Capture side: accumulate 256-sample hops and postMessage them to the main
// thread (which runs the ONNX denoiser).
// Playback side: the main thread posts enhanced 256-sample hops back; we buffer
// them in a ring and drain 128 samples per render quantum. A small jitter
// buffer absorbs the round-trip latency.

const HOP = 256;
const RING = 16384; // ~1s at 16kHz, power of two

class GtcrnIoProcessor extends AudioWorkletProcessor {
	constructor() {
		super();
		this.capAccum = new Float32Array(HOP);
		this.capCount = 0;

		this.ring = new Float32Array(RING);
		this.readIdx = 0;
		this.writeIdx = 0;
		this.available = 0;
		this.started = false; // wait for a little buffer before draining

		this.port.onmessage = (e) => {
			const hop = e.data; // Float32Array(256) enhanced
			for (let i = 0; i < hop.length; i++) {
				this.ring[this.writeIdx] = hop[i];
				this.writeIdx = (this.writeIdx + 1) % RING;
			}
			this.available = Math.min(this.available + hop.length, RING);
			if (this.available >= HOP * 3) this.started = true; // ~48ms jitter buffer
		};
	}

	process(inputs, outputs) {
		const input = inputs[0];
		const output = outputs[0];
		const inCh = input?.[0];
		const outCh = output?.[0];
		const n = outCh ? outCh.length : 128;

		// Capture: append incoming samples, emit full hops.
		if (inCh) {
			for (let i = 0; i < inCh.length; i++) {
				this.capAccum[this.capCount++] = inCh[i];
				if (this.capCount === HOP) {
					const out = this.capAccum.slice(0);
					this.port.postMessage(out, [out.buffer]);
					this.capCount = 0;
				}
			}
		}

		// Playback: drain the ring once we've buffered enough.
		if (outCh) {
			if (this.started && this.available >= n) {
				for (let i = 0; i < n; i++) {
					outCh[i] = this.ring[this.readIdx];
					this.readIdx = (this.readIdx + 1) % RING;
				}
				this.available -= n;
			} else {
				outCh.fill(0); // underrun -> silence
			}
		}

		return true;
	}
}

registerProcessor("gtcrn-io", GtcrnIoProcessor);
