import type { App } from "./app";
import { RnnoiseManager } from "./rnnoise";
import { getWebSocket } from "./socketWapper";
import { type SocketCommand, type SocketCommandMap, socketMessage } from "hermes-shared";

const WebSocket = getWebSocket();

//https://medium.com/swlh/manage-dynamic-multi-peer-connections-in-webrtc-3ff4e10f75b7
const RTCConfig = {
    iceServers: [
        { urls: ["stun:stun.l.google.com:19302"] },
        { urls: ["stun:stun1.l.google.com:19302"] },
        { urls: ["stun:stun2.l.google.com:19302"] },
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

    #streams: Map<string, MediaStream[]> = new Map();
    #currentRoomId: string | null = null;
    #currentChannelId: string | null = "001417d2-c76c-4622-9e75-bb6303341cd0";

    #rooms = new Map<string, { users: Set<string>, name: string; id: string; }>();

    #rnnoise = new RnnoiseManager();

    constructor(private app: App) {
        super();

        this.#rooms.set("91b3c4ae-e243-47e0-8124-35d8f357f211", { users: new Set(), name: "Test Channel", id: "91b3c4ae-e243-47e0-8124-35d8f357f211" });
    }

    //#region React
    public async mount() {
        const { reject, resolve, promise } = Promise.withResolvers();

        await this.#rnnoise.init();

        console.log("mounting socket");

        this.#socket = new WebSocket(`wss://${import.meta.env.VITE_SERVER_HOST}/ws?token=${localStorage.getItem("token")}`);

        this.#socket.addEventListener("message", this.onMessage);
        this.#socket.addEventListener("open", (_ev) => {
            this.dispatchEvent(new Event("ready"));
            resolve();
        });
        this.#socket.addEventListener("error", (ev) => {
            reject(ev);
            console.error(ev);
        });
        this.#socket.addEventListener("close", (ev) => {
            console.log(ev);
        });

        await promise;
    }
    public unmount() {
        this.#count--;
        if (this.#count !== 0) return;

        this.#socket?.close(1000, "component unmount");
        this.#socket = null;

        console.log("unmount");
    }

    //#endregion

    //#region Public Api

    public getRoom(roomId: string) {
        return this.#rooms.get(roomId);
    }

    public getMediaStream(userId: string) {
        if (userId === this.app.auth.user?.id || userId === "self") {
            return this.#streams.get("self");
        }
        return this.#streams.get(userId);
    }

    public async joinRoom(roomId: string) {
        const userId = this.app.auth.user?.id;
        if (!userId) throw new Error("No user id was found");
        if (!this.#currentChannelId) throw new Error("No Channel Id");

        this.#currentRoomId = "91b3c4ae-e243-47e0-8124-35d8f357f211";

        const room = this.#rooms.get(roomId);
        if (room) {
            room.users.add(userId);
            this.#rooms.set(room.id, { ...room });
        }

        const config = {
            video: false,
            audio: {
                deviceId: "default",
                echoCancellation: {
                    exact: false,
                    ideal: true
                },
                noiseSuppression: true
            }
        } satisfies MediaStreamConstraints;

        const settings = localStorage.getItem("diviceInputSettings")
        if (settings) {
            const divices = JSON.parse(settings) as { camera: string; mic: string; };
            config.audio.deviceId = divices.mic;
        }

        /// https://github.com/shiguredo/rnnoise-wasm?tab=readme-ov-file
        const stream = await navigator.mediaDevices.getUserMedia(config);

        const output = await this.#rnnoise.createProcesser(stream);
        this.#streams.set("self", [output]);

        this.send("channel::room::join", { channelId: this.#currentChannelId, roomId });
        this.dispatchEvent(new Event(`media-stream::${userId}`));

        this.dispatchEvent(new CustomEvent(`room::${roomId}::update`, {
            detail: {
                type: "user_join",
                user: userId
            }
        }))
    }

    public async leaveRoom(roomId: string) {
        if (!this.#currentChannelId) throw new Error("No channel id");

        this.send("channel::room::leave", { channelId: this.#currentChannelId, roomId });

        this.#currentRoomId = null;

        const id = this.app.auth.user?.id;
        if (!id) throw new Error("invalid error");

        const room = this.#rooms.get(roomId);
        if (room) {
            room?.users.delete(id);
            this.#rooms.set(room.id, { ...room });
        }

        this.#streams.get("self")?.at(0)?.getTracks().forEach(e => e.stop());

        this.dispatchEvent(new CustomEvent(`room::${roomId}::update`, {
            detail: {
                type: "user_leave",
                user: id
            }
        }))
    }

    //#endregion

    //#region Internal

    private send<T extends SocketCommand["type"]>(type: T, payload: SocketCommandMap[T]) {
        this.#socket?.send(JSON.stringify({ type, data: payload }));
    }

    private onMessage = async (ev: MessageEvent<string>) => {
        try {
            const { type, data } = socketMessage.parse(JSON.parse(ev.data));

            console.debug("Event", type);

            switch (type) {
                case "channel::room::user_join": {
                    const room = this.#rooms.get(data.roomId);
                    if (!room) throw new Error(`Unable to update room. No room was found`);
                    room.users.add(data.user);

                    this.#rooms.set(room.id, { ...room });

                    this.dispatchEvent(new CustomEvent(`room::${data.roomId}::update`, {
                        detail: {
                            type: "user_join",
                            user: data.user
                        }
                    }));

                    if (room.id === this.#currentRoomId) {
                        const peer = new RTCPeerConnection(RTCConfig);
                        this.addLocalMedia(peer);

                        peer.addEventListener("icecandidate", (ev) => {
                            if (!ev.candidate) {
                                const description = peer.localDescription;
                                if (!description) throw new Error("Failed to get description");
                                if (!this.#currentChannelId || !this.#currentRoomId) throw new Error("No channel or room ids");
                                this.send("room::peer::offer", {
                                    channelId: this.#currentChannelId,
                                    roomId: this.#currentRoomId,
                                    forUser: data.user,
                                    sdp: description
                                })
                            }
                        });
                        peer.addEventListener("connectionstatechange", () => {
                            if (peer.connectionState === "closed") {
                                this.#connections.delete(data.user);
                            }
                            this.dispatchEvent(new CustomEvent(`media-stream::${data.user}::state`, { detail: { state: peer.connectionState } }));
                        });
                        peer.addEventListener("track", (ev) => this.addTrack(ev, data.user));

                        const offer = await peer.createOffer({
                            offerToReceiveAudio: true,
                            offerToReceiveVideo: true
                        });
                        await peer.setLocalDescription(offer);

                        this.#connections.set(data.user, peer);
                    }

                    break;
                }
                case "channel::room::user_leave": {
                    const room = this.#rooms.get(data.roomId);
                    if (!room) throw new Error("Failed to get room");

                    room.users.delete(data.user);
                    this.#rooms.set(room.id, { ...room });

                    this.dispatchEvent(new CustomEvent(`room::${data.roomId}::update`, {
                        detail: {
                            type: "user_leave",
                            user: data.user
                        }
                    }))

                    if (room.id === this.#currentRoomId) {
                        const connection = this.#connections.get(data.user);
                        connection?.close();
                        this.#connections.delete(data.user);
                    }

                    break;
                }
                case "room::peer::offer": {
                    const peer = new RTCPeerConnection(RTCConfig);
                    this.addLocalMedia(peer);

                    peer.addEventListener("icecandidate", (ev) => {
                        if (!ev.candidate) {
                            const description = peer.localDescription;
                            if (!description) throw new Error("failed to get local description");
                            if (!this.#currentChannelId || !this.#currentRoomId) throw new Error("No channel or room ids");

                            this.send("room::peer::answer", {
                                channelId: this.#currentChannelId,
                                roomId: this.#currentRoomId,
                                forUser: data.user,
                                sdp: description
                            });
                        }
                    });
                    peer.addEventListener("connectionstatechange", () => {
                        if (peer.connectionState === "closed") {
                            this.#connections.delete(data.user);
                        }
                        this.dispatchEvent(new CustomEvent(`media-stream::${data.user}::state`, { detail: { state: peer.connectionState } }));
                    });
                    peer.addEventListener("track", (ev) => this.addTrack(ev, data.user));

                    await peer.setRemoteDescription(data.sdp);

                    const description = await peer.createAnswer({
                        offerToReceiveAudio: true,
                        offerToReceiveVideo: true
                    });

                    await peer.setLocalDescription(description);
                    this.#connections.set(data.user, peer);

                    break;
                }
                case "room::peer::answer": {
                    const connection = this.#connections.get(data.user);
                    if (!connection) throw new Error(`Failed to get RTCPeerConnection for user '${data.user}'`);

                    await connection.setRemoteDescription(data.sdp);

                    break;
                }
                default:
                    console.error("Unhandled socket message", type);
                    break;
            }

        } catch (error) {
            console.error(error);
        }
    }

    private addTrack(ev: RTCTrackEvent, user: string) {
        let stream: MediaStream;
        if (ev.streams.length === 1) {
            stream = ev.streams[0];
        } else {
            stream = new MediaStream([ev.track]);
        }

        console.log("Adding track for user", user);

        let streams = this.#streams.get(user);
        if (!streams) {
            streams = [];
        }
        streams.push(stream);

        this.#streams.set(user, streams);

        this.dispatchEvent(new Event(`media-stream::${user}`));

    }

    private addLocalMedia(peer: RTCPeerConnection) {
        const mediaStream = this.#streams.get("self")?.at(0);
        if (!mediaStream) throw new Error("Failed to add self media");

        for (const track of mediaStream.getTracks()) {
            peer.addTrack(track, mediaStream);
        }
    }

    //#endregion
}