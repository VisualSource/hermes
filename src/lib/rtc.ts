import { getWebSocket } from "./socketWapper";

type SocketMessage = { type: "joined", data: { user: string; } } |
{
    type: "offer", data: {
        user: string;
        sdp: RTCSessionDescriptionInit;
    }
} | {
    type: "answer",
    data: { sdp: RTCSessionDescriptionInit, user: string; }
} | {
    type: "leave",
    data: { user: string; }
}

const WebSocket = getWebSocket();

//https://medium.com/swlh/manage-dynamic-multi-peer-connections-in-webrtc-3ff4e10f75b7
const RTCConfig = {
    iceServers: [
        { urls: ["stun:stun.l.google.com:19302"] },
        /* { urls: ["stun:stun1.l.google.com:19302"] },
         { urls: ["stun:stun2.l.google.com:19302"] },
         { urls: ["stun:stun3.l.google.com:19302"] },
         { urls: ["stun:stun4.l.google.com:19302"] }*/
    ]
}

export class RTC extends EventTarget {
    #count = 0;
    #socket: WebSocket | null = null;

    #connections = new Map<string, RTCPeerConnection>();
    #media: MediaStream | null = null;
    #remoteStream: Map<string, MediaStream[]> = new Map();


    #currentRoomId: string | null = "91b3c4ae-e243-47e0-8124-35d8f357f211";
    #currentChannelId: string | null = "001417d2-c76c-4622-9e75-bb6303341cd0";

    mount() {
        this.#count++;
        if (this.#count !== 1) return;

        this.#socket = new WebSocket(`wss://${import.meta.env.VITE_SERVER_HOST}/ws?token=${localStorage.getItem("token")}`);

        this.#socket.addEventListener("message", this.onMessage);
        this.#socket.addEventListener("open", (ev) => {
            console.log(ev);
        });
        this.#socket.addEventListener("error", (ev) => {
            console.log(ev);
        });
        this.#socket.addEventListener("close", (ev) => {
            console.log(ev);
        });

        console.log("mount");
    }
    unmount() {
        this.#count--;
        if (this.#count !== 0) return;

        this.#socket?.close(1000, "component unmount");
        this.#socket = null;

        console.log("unmount");
    }

    private onMessage = async (ev: MessageEvent<string>) => {
        try {
            const { data, type } = JSON.parse(ev.data) as SocketMessage;

            switch (type) {
                case "leave": {
                    const conn = this.#connections.get(data.user);
                    conn?.close();
                    this.#connections.delete(data.user);

                    this.dispatchEvent(new CustomEvent("user-leave-room", { detail: data.user }));

                    break;
                }
                case "joined": {
                    const peer = new RTCPeerConnection(RTCConfig);

                    this.addLocalMedia(peer);

                    peer.addEventListener("icecandidate", (ev) => {
                        if (!ev.candidate) {
                            const description = peer.localDescription;
                            if (!description) throw new Error("Failed to get description");
                            if (!this.#currentChannelId || !this.#currentRoomId) throw new Error("No channel or room ids");
                            this.send("offer", {
                                channelId: this.#currentChannelId,
                                roomId: this.#currentRoomId,
                                forUser: data.user,
                                sdp: description
                            })
                        }
                    });

                    peer.addEventListener("track", (ev) => this.addTrack(ev, data.user));

                    const offer = await peer.createOffer({
                        offerToReceiveAudio: true,
                        offerToReceiveVideo: true
                    });

                    await peer.setLocalDescription(offer);

                    this.#connections.set(data.user, peer);

                    this.dispatchEvent(new CustomEvent("user-join-room", { detail: data.user }));

                    break;
                }
                case "offer": {

                    const peer = new RTCPeerConnection(RTCConfig);
                    this.addLocalMedia(peer);

                    peer.addEventListener("icecandidate", (ev) => {
                        if (!ev.candidate) {
                            const description = peer.localDescription;
                            if (!description) throw new Error("failed to get local description");
                            if (!this.#currentChannelId || !this.#currentRoomId) throw new Error("No channel or room ids");

                            this.send("answer", {
                                channelId: this.#currentChannelId,
                                roomId: this.#currentRoomId,
                                forUser: data.user,
                                sdp: description
                            });
                        }
                    });

                    peer.addEventListener("track", (ev) => this.addTrack(ev, data.user));

                    await peer.setRemoteDescription(data.sdp);

                    const sdp = await peer.createAnswer({
                        offerToReceiveAudio: true,
                        offerToReceiveVideo: true
                    });

                    await peer.setLocalDescription(sdp);

                    this.#connections.set(data.user, peer);

                    break;
                }
                case "answer": {
                    const conn = this.#connections.get(data.user);
                    if (!conn) throw new Error(`Missing peer connection for user ${data.user}`);
                    await conn.setRemoteDescription(data.sdp);
                    break;
                }
                default:
                    console.log("Unhandled socket message", type);
                    break;
            }

        } catch (error) {
            console.error(error);
        }
    }

    public send(type: string, payload: Record<string, unknown>) {
        this.#socket?.send(JSON.stringify({ type, data: payload }));
    }

    private addTrack(ev: RTCTrackEvent, user: string) {
        let stream: MediaStream;
        if (ev.streams.length === 1) {
            stream = ev.streams[0];
        } else {
            stream = new MediaStream([ev.track]);
        }


        let streams = this.#remoteStream.get(user);
        if (!streams) {
            streams = [];
        }
        streams.push(stream);

        this.#remoteStream.set(user, streams);

        this.dispatchEvent(new CustomEvent("add-media-source", { detail: stream }));

    }

    private addLocalMedia(peer: RTCPeerConnection) {
        if (!this.#media) return;

        for (const track of this.#media.getTracks()) {
            peer.addTrack(track, this.#media);
        }
    }

}