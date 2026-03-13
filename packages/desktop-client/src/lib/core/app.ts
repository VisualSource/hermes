import { NoiseSuppressor } from "../audio/noise-suppressor";
import {
	VoiceChannelRequest,
	VoiceChannelEventType,
	RtcNewCandidate,
	RtcNewCandidate_IceCandidate,
	Envelope,
	type VoiceChannelUserEvent,
	RtcEvent,
	RtcEvent_RtcMessageType,
} from "../proto/hermes";
import { RTC, RtcSdpTypeMap } from "./rtc";
import { auth } from "../clients/auth";
import { nanoid } from "nanoid";
import { BinaryReader } from "@bufbuild/protobuf/wire";
export class App extends EventTarget {
	private static INSTANCE: App | null = null;

	private _socket: WebSocket | null = null;
	private inVoice: boolean = false;
	private rtc = new RTC();

	get socket(): WebSocket {
		if (!this._socket) throw new Error("Missing socket");
		return this._socket;
	}

	constructor() {
		super();
		if (App.INSTANCE !== null)
			throw new Error("a app instance already exists!");

		App.INSTANCE = this;

		this.rtc.addEventListener("rtc-new-ice-candidate", (ev) => {
			const sdp = ev.candidate.toJSON();
			this.send({
				rtcIce: RtcNewCandidate.create({
					target: ev.peerId,
					candidate: RtcNewCandidate_IceCandidate.create({
						candidate: sdp.candidate,
						sdpMLineIndex: sdp.sdpMLineIndex ?? undefined,
						sdpMid: sdp.sdpMid ?? undefined,
						usernameFragment: sdp.usernameFragment ?? undefined,
					}),
				}),
			});
		});
	}

	public async init() {
		await NoiseSuppressor.create();
		console.log("[Websocket] Starting websocket");
		await this.initSocket();
	}

	//#region PublicApi
	public joinVoice = (channelId: string) => {
		console.debug("Join Voice channel", channelId);

		this.send({
			voiceChannelRequest: VoiceChannelRequest.create({
				channelId,
				type: VoiceChannelEventType.Join,
			}),
		});

		this.inVoice = true;

		this.dispatchEvent(new Event("voice-state-change"));
	};

	public leaveVoice = (channelId: string) => {
		console.debug("Leaving voice channel", channelId);

		this.send({
			voiceChannelRequest: VoiceChannelRequest.create({
				channelId,
				type: VoiceChannelEventType.Leave,
			}),
		});

		this.inVoice = false;
		this.dispatchEvent(new Event("voice-state-change"));
	};
	//#endregion

	//#region ReactApi
	public getProp = (prop: string) => {
		switch (prop) {
			case "inVoice":
				return this.inVoice;
			default:
				return null;
		}
	};
	//#endregion

	//#region Socket
	private send(
		msg: Omit<Envelope, "messageId" | "timestamp" | "version" | "type">,
	) {
		const envelope = Envelope.create({
			messageId: nanoid(),
			timestamp: Date.now(),
			version: 1,
			...msg,
		});

		const data = Envelope.encode(envelope).finish();

		this.socket?.send(data);
	}

	private onClose = (ev: CloseEvent) => {
		console.log("Socket Closed!", ev);
	};

	private onMessage = async (ev: MessageEvent<ArrayBuffer>) => {
		if (!(ev.data instanceof ArrayBuffer)) {
			console.error("Unable to handle text frame");
			return;
		}

		const reader = new BinaryReader(new Uint8Array(ev.data));
		const envelope = Envelope.decode(reader);

		if (envelope.rtc) {
			await this.handleRtcEvent(envelope.rtc);
		} else if (envelope.rtcIce) {
			await this.handleRtcNewIceCandidate(envelope.rtcIce);
		} else if (envelope.voiceChannelEvent) {
			await this.handleVoiceChannelEvent(envelope.voiceChannelEvent);
		}
	};

	//#endregion

	//#region Internal
	private async handleRtcNewIceCandidate(ev: RtcNewCandidate) {
		const candidate = new RTCIceCandidate(ev.candidate);

		await this.rtc.addIceCandidate(ev.target, candidate);
	}

	private async handleRtcEvent(event: RtcEvent) {
		switch (event.type) {
			case RtcEvent_RtcMessageType.Offer: {
				const descp = await this.rtc.initConnectionFromRemote(
					event.target,
					new RTCSessionDescription({
						sdp: event.sdp,
						type: "offer",
					}),
				);

				this.send({
					rtc: RtcEvent.create({
						type: RtcSdpTypeMap[descp.type],
						sdp: descp.sdp,
					}),
				});
				break;
			}
			case RtcEvent_RtcMessageType.Answer: {
				await this.rtc.finishConnection(
					event.target,
					new RTCSessionDescription({
						sdp: event.sdp,
						type: "answer",
					}),
				);
				break;
			}
			case RtcEvent_RtcMessageType.PrAnswer:
			case RtcEvent_RtcMessageType.Rollback:
			case RtcEvent_RtcMessageType.UNRECOGNIZED:
				throw new Error("invalid rtc message", { cause: event });
		}
	}

	private async handleVoiceChannelEvent(event: VoiceChannelUserEvent) {
		switch (event.type) {
			case VoiceChannelEventType.Join: {
				if (!this.inVoice) return;

				const desp = await this.rtc.initConnection(event.userId);

				this.send({
					rtc: RtcEvent.create({
						sdp: desp.sdp,
						target: event.userId,
						type: RtcSdpTypeMap[desp.type],
					}),
				});

				break;
			}
			case VoiceChannelEventType.Leave:
			case VoiceChannelEventType.UNRECOGNIZED:
				throw new Error("Invalid Event");
		}
	}

	private async initSocket() {
		const url = new URL(
			`${import.meta.env.VITE_SERVER_URL}/api/ws?token=${auth.token}`,
		);
		url.protocol = "wss";

		const socket = new WebSocket(url);
		socket.binaryType = "arraybuffer";

		const { resolve, reject, promise } = Promise.withResolvers<void>();

		socket.addEventListener("close", this.onClose);
		socket.addEventListener("error", (ev) => {
			console.log(ev);
			reject(ev);
		});
		socket.addEventListener("message", this.onMessage);
		socket.addEventListener("open", () => resolve());

		await promise;

		this._socket = socket;
	}
	//#endregion
}
