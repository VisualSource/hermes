const rtcConfig = {
    iceServers: [
        { urls: ["stun:stun.l.google.com:19302"] },
        /* { urls: ["stun:stun1.l.google.com:19302"] },
         { urls: ["stun:stun2.l.google.com:19302"] },
         { urls: ["stun:stun3.l.google.com:19302"] },
         { urls: ["stun:stun4.l.google.com:19302"] }*/
    ]
}
//https://medium.com/swlh/manage-dynamic-multi-peer-connections-in-webrtc-3ff4e10f75b7
class RTC {
    connections = new Map<string, RTCPeerConnection>();
    media: {
        stream: MediaStream | undefined;
        type: "audio" | "video" | "both"
    } = {
            type: "audio",
            stream: undefined
        }

    socket: WebSocket;

    roomId = "91b3c4ae-e243-47e0-8124-35d8f357f211"
    channelId = "001417d2-c76c-4622-9e75-bb6303341cd0";

    constructor() {
        const user = new URLSearchParams(window.location.search);

        const token = user.get("token");

        this.socket = new WebSocket(`wss://${window.location.host}/ws?token=${token}`);

        this.socket.addEventListener("message", this.onMessage);
        this.socket.addEventListener("open", this.onOpen);
        this.socket.addEventListener("error", this.onError);
        this.socket.addEventListener("close", this.onClose);
    }

    private onMessage = async (ev: MessageEvent<string>) => {
        const payload = JSON.parse(ev.data);

        switch (payload.type) {
            case "joined": {
                const user = payload.data.user;
                console.log("User joined", user);
                const peer = new RTCPeerConnection(rtcConfig);

                this.addLocalMedia(peer);

                peer.addEventListener("icecandidate", (ev) => {
                    if (!ev.candidate) {
                        const description = peer.localDescription;
                        if (!description) throw new Error("Failed to get description");
                        this.send("offer", {
                            channelId: this.channelId,
                            roomId: this.roomId,
                            forUser: user,
                            sdp: description
                        });
                    }
                });

                peer.addEventListener("track", (ev) => this.addTrack(ev, user));

                const offer = await peer.createOffer({
                    offerToReceiveAudio: true,
                    offerToReceiveVideo: true,
                });
                await peer.setLocalDescription(offer);

                this.connections.set(user, peer);

                break;
            }
            case "offer": {
                const user = payload.data.user;

                console.log("Got a offer from", user);

                const peer = new RTCPeerConnection(rtcConfig);

                this.addLocalMedia(peer);

                peer.addEventListener("icecandidate", (ev) => {
                    if (!ev.candidate) {
                        const description = peer.localDescription;
                        if (!description) throw new Error("failed to get local description");

                        this.send("answer", {
                            channelId: this.channelId,
                            roomId: this.roomId,
                            forUser: user,
                            sdp: description
                        });
                    }
                });
                peer.addEventListener("track", (ev) => this.addTrack(ev, user));
                await peer.setRemoteDescription(payload.data.sdp);

                const descp = await peer.createAnswer({
                    offerToReceiveAudio: true,
                    offerToReceiveVideo: true
                });

                peer.setLocalDescription(descp);

                this.connections.set(user, peer);

                break;
            }
            case "answer": {
                const sdp = payload.data.sdp;
                const user = payload.data.user;
                console.log("Got a answer from", user)

                const conn = this.connections.get(user);

                if (!conn) throw new Error("No connection found");

                await conn.setRemoteDescription(sdp);

                break;
            }
            default:
                break;
        }
    }

    addTrack = (ev: RTCTrackEvent, user: string) => {
        console.log(ev);
        const out = document.getElementById("output");

        const container = document.createElement("div");

        let stream
        if (ev.streams.length === 1) {
            stream = ev.streams[0];
        } else {
            stream = new MediaStream([ev.track]);
        }

        const owner = document.createElement("h4");
        owner.textContent = user;
        container.appendChild(owner);

        if (ev.track.kind !== "video") {
            const audio = document.createElement("audio");
            audio.setAttribute("autoplay", "true");
            audio.setAttribute("playinline", "true");
            const canvus = document.createElement("canvas");
            canvus.setAttribute("height", "300");
            canvus.setAttribute("width", "450");

            audio.srcObject = stream;
            container.appendChild(canvus);
            container.appendChild(audio);

            const audioCtx = new AudioContext();
            const analyser = audioCtx.createAnalyser();

            const source = audioCtx.createMediaStreamSource(stream);
            source.connect(analyser);

            analyser.fftSize = 2048;
            const bufferLength = analyser.frequencyBinCount;
            const dataArray = new Uint8Array(bufferLength);
            analyser.getByteFrequencyData(dataArray);

            const ctx = canvus.getContext("2d");
            if (!ctx) throw new Error("canvus");
            ctx?.clearRect(0, 0, 450, 300);
            const draw = () => {
                const drawVisual = requestAnimationFrame(draw);
                analyser.getByteFrequencyData(dataArray);

                ctx.fillStyle = "rgb(200,200,200)";
                ctx?.fillRect(0, 0, 450, 300);

                ctx.lineWidth = 2;
                ctx.strokeStyle = "rgb(0 0 0)";
                ctx?.beginPath();

                const sliceWidth = 450 / bufferLength;
                let x = 0;

                for (let i = 0; i < bufferLength; i++) {
                    const v = dataArray[i] / 128.0;
                    const y = v * (300 / 2);
                    if (i === 0) {
                        ctx?.moveTo(x, y);
                    } else {
                        ctx?.lineTo(x, y);
                    }

                    x += sliceWidth;
                }

                ctx?.lineTo(450, 300 / 2);
                ctx?.stroke();
            }

            draw();
        } else {
            const vid = document.createElement("video");
            vid.setAttribute("height", "430");
            vid.setAttribute("width", "712");
            vid.setAttribute("autoplay", "true");
            vid.setAttribute("controlslist", "nodownload")
            vid.setAttribute("controls", "false");
            vid.setAttribute("playsinline", "true");

            vid.srcObject = stream;
            container.appendChild(vid);
        }

        out?.appendChild(container);
    }

    send(type: string, payload: Record<string, unknown>) {
        this.socket.send(JSON.stringify({ type, data: payload }));
    }

    private onError = (ev: Event) => {
        console.error(ev);
    }
    private onOpen = () => { }
    private onClose = () => { }

    private addLocalMedia(peer: RTCPeerConnection) {
        if (!this.media.stream) return;

        for (const track of this.media.stream.getTracks()) {
            peer.addTrack(track, this.media.stream);
        }

    }

    async init(config: { video: boolean, audio: boolean }) {
        this.media.stream = await navigator.mediaDevices.getUserMedia({
            video: config.video,
            audio: config.audio
        });

        this.media.stream.getTracks()[0].getSettings().noiseSuppression = true;

        this.send("join", {
            channelId: this.channelId,
            roomId: this.roomId
        });
    }
    destroy() {

    }
}

const rtc = new RTC();

document.addEventListener("DOMContentLoaded", () => main());
function main() {
    const startBtn = document.getElementById("start");
    const endBtn = document.getElementById("end");

    startBtn?.addEventListener("click", () => {
        rtc.init({ audio: true, video: false });
        startBtn.toggleAttribute("disabled");
        endBtn?.toggleAttribute("disabled");
    });
    endBtn?.addEventListener("click", () => {
        rtc.destroy();
        startBtn?.toggleAttribute("disabled");
        endBtn?.toggleAttribute("disabled");
    });
}