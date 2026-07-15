import { RtcEvent_RtcMessageType } from "../proto/hermes";
import type { TypedEmitter } from "../types";

interface RTCEventMap {
	"rtc-close": Event;
	"rtc-new-ice-candidate": RTCNewIceCandidate;
	"rtc-connection-closed": RTCEvent;
	"rtc-connection-failed": RTCEvent;
	"rtc-negotation": RTCNegotationEvent;
	"rtc-negotation-start-failed": Event;
	"rtc-track": RTCTrackEvent;
	"rtc-connection-state-change": RTCConnectionStateChangeEvent;
}
export const RtcSdpTypeMap = {
	answer: RtcEvent_RtcMessageType.Answer,
	offer: RtcEvent_RtcMessageType.Offer,
	pranswer: RtcEvent_RtcMessageType.PrAnswer,
	rollback: RtcEvent_RtcMessageType.Rollback,
} as const satisfies Record<RTCSdpType, RtcEvent_RtcMessageType>;

//https://medium.com/swlh/manage-dynamic-multi-peer-connections-in-webrtc-3ff4e10f75b7
const DefaultRTCIceServer: RTCIceServer[] = [
	{ urls: ["stun:stun.l.google.com:19302"] },
	{ urls: ["stun:stun1.l.google.com:19302"] },
	{ urls: ["stun:stun2.l.google.com:19302"] },
];
export class RTC extends (EventTarget as TypedEmitter<RTCEventMap>) {
	private conns: Map<string, RTCPeerConnection> = new Map();

	/**
	 * Init Rtc peer connetion and return session description that can be then set to peer
	 *
	 * @example
	 *  const desc = await rtc.initConnection(peerId);
	 *  socket.send({ rtc: desc, peerId });
	 *
	 * @param peerId
	 * @returns
	 */
	public async initConnection(peerId: string) {
		const peer = new RTCPeerConnection({
			iceServers: DefaultRTCIceServer,
		});
		this.initSharedEventHandler(peerId, peer);

		const description = await peer.createOffer({
			offerToReceiveAudio: true,
			offerToReceiveVideo: true,
		});
		await peer.setLocalDescription(description);
		if (!peer.localDescription) throw new Error("local description is null");

		this.conns.set(peerId, peer);

		return peer.localDescription;
	}

	/**
	 * Init a RTC peer connection using a remoteDescription
	 *
	 * @param peerId
	 * @param remoteDescription - offer from peer
	 * @returns
	 */
	public async initConnectionFromRemote(
		peerId: string,
		remoteDescription: RTCSessionDescriptionInit,
	) {
		const peer = new RTCPeerConnection({
			iceServers: DefaultRTCIceServer,
		});

		this.initSharedEventHandler(peerId, peer);

		await peer.setRemoteDescription(remoteDescription);

		const description = await peer.createAnswer();
		await peer.setLocalDescription(description);

		if (!peer.localDescription) throw new Error("local description is null");

		this.conns.set(peerId, peer);

		return peer.localDescription;
	}

	/**
	 * finish setup of rtc connection from peer
	 *
	 * @param peerId
	 * @param remoteDescription answer to an offer
	 */
	public async finishConnection(
		peerId: string,
		remoteDescription: RTCSessionDescription,
	) {
		const peer = this.conns.get(peerId);
		if (!peer) throw new Error(`unable to find peer with id of "${peerId}"`);

		await peer.setRemoteDescription(remoteDescription);
	}

	public async addIceCandidate(
		peerId: string,
		candidate: RTCLocalIceCandidateInit,
	) {
		const ca = new RTCIceCandidate(candidate);

		const peer = this.conns.get(peerId);
		if (!peer) throw new Error(`unable to find peer with id of "${peerId}"`);

		console.debug(peer.remoteDescription);

		await peer.addIceCandidate(ca);
	}

	public async closeConnection(peerId: string) {
		const conn = this.conns.get(peerId);
		if (!conn) return;

		conn.close();
		this.conns.delete(peerId);
	}

	public async closeConnections() {
		for (const [_, conn] of this.conns) {
			conn.close();
		}

		this.conns.clear();

		this.dispatchEvent(new Event("rtc-close"));
	}

	public getConnection(peerId: string) {
		const peer = this.conns.get(peerId);
		if (!peer) throw new Error(`unable to find peer with id of "${peerId}"`);

		return peer;
	}

	/**
	 * Handle a negotation needed event from peer
	 *
	 * @param peerId
	 * @param remoteDescription
	 * @returns
	 */
	public async handleNegotationNeeded(
		peerId: string,
		remoteDescription: RTCSessionDescription,
	) {
		const peer = this.conns.get(peerId);
		if (!peer) throw new Error(`unable to find peer with id of "${peerId}"`);

		await peer.setRemoteDescription(remoteDescription);

		const description = await peer.createAnswer();
		await peer.setLocalDescription(description);

		if (!peer.localDescription)
			throw new Error("unable to get local description");

		return peer.localDescription;
	}

	private initSharedEventHandler(peerId: string, peer: RTCPeerConnection) {
		peer.addEventListener("icecandidate", (ev) => {
			if (!ev.candidate) return;
			this.dispatchEvent(new RTCNewIceCandidate(peerId, ev.candidate));
		});
		peer.addEventListener("icecandidateerror", (ev) => {
			console.error(ev);
		});
		peer.addEventListener("icegatheringstatechange", (ev) => {
			console.debug(ev);
		});
		peer.addEventListener("iceconnectionstatechange", () => {
			switch (peer.iceConnectionState) {
				case "closed":
				case "failed":
					this.closeConnection(peerId);
					this.dispatchEvent(
						new RTCEvent(
							peer.iceConnectionState === "failed"
								? "rtc-connection-failed"
								: "rtc-connection-closed",
							peerId,
						),
					);
					break;
			}
		});
		peer.addEventListener("signalingstatechange", () => {
			switch (peer.signalingState) {
				case "closed":
					this.closeConnection(peerId);
					this.dispatchEvent(new RTCEvent("rtc-connection-closed", peerId));
					break;
			}
		});

		peer.addEventListener("negotiationneeded", async () => {
			const offer = await peer.createOffer({
				offerToReceiveAudio: true,
				offerToReceiveVideo: true,
			});

			await peer.setLocalDescription(offer);
			if (!peer.localDescription) {
				this.dispatchEvent(new Event("rtc-negotation-start-failed"));
				return;
			}

			this.dispatchEvent(new RTCNegotationEvent(peerId, peer.localDescription));
		});

		peer.addEventListener("connectionstatechange", () => {
			if (peer.connectionState === "closed") {
				this.conns.delete(peerId);
			}

			this.dispatchEvent(
				new RTCConnectionStateChangeEvent(peerId, peer.connectionState),
			);
		});
		peer.addEventListener("track", (ev) => {
			this.dispatchEvent(new RTCTrackEvent(peerId, ev.track));
		});
	}
}

class RTCConnectionStateChangeEvent extends Event {
	constructor(
		public peerId: string,
		public state: RTCPeerConnectionState,
	) {
		super("rtc-connection-state-change");
	}
}
class RTCNegotationEvent extends Event {
	constructor(
		public peerId: string,
		public newDescription: RTCSessionDescription,
	) {
		super("rtc-negotation");
	}
}

class RTCTrackEvent extends Event {
	constructor(
		public peerId: string,
		public track: MediaStreamTrack,
	) {
		super("rtc-track");
	}
}

class RTCNewIceCandidate extends Event {
	constructor(
		public peerId: string,
		public candidate: RTCIceCandidate,
	) {
		super("rtc-new-ice-candidate");
	}
}

class RTCEvent extends Event {
	constructor(
		name: string,
		public peerId: string,
	) {
		super(name);
	}
}
