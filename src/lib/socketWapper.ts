import TauriWebSocket from "@tauri-apps/plugin-websocket";

class Wrapper extends EventTarget implements WebSocket {
    binaryType: BinaryType = "arraybuffer";
    bufferedAmount: number = 0;
    extensions: string = "";
    onclose: ((this: WebSocket, ev: CloseEvent) => any) | null = null;
    onerror: ((this: WebSocket, ev: Event) => any) | null = null;
    onmessage: ((this: WebSocket, ev: MessageEvent) => any) | null = null;
    onopen: ((this: WebSocket, ev: Event) => any) | null = null;
    protocol: string = "";
    readyState: number = 0;

    private abort: AbortController | null = null;

    socket: TauriWebSocket | null = null;

    constructor(public url: string, prots?: string[]) {
        super();

        if (import.meta.env.DEV) {
            import.meta.hot?.on("vite:beforeFullReload", () => {
                this.close();
            });
        }

        this.readyState = Wrapper.CONNECTING;
        this.abort = new AbortController();
        const signal = this.abort.signal;
        TauriWebSocket.connect(url).then((socket) => {
            this.socket = socket;

            if (signal.aborted) {
                socket.disconnect().catch(e => {
                    console.log(e);
                    this.dispatchEvent(new Event("error"))
                });
                return;
            }

            socket.addListener((ev) => {
                switch (ev.type) {
                    case "Text":
                        this.dispatchEvent(new MessageEvent("message", {
                            data: ev.data
                        }));
                        break;
                    case "Close":
                        this.dispatchEvent(new CloseEvent("close", { code: ev.data?.code, reason: ev.data?.reason }));
                        break;
                    default:
                        break;
                }
            });
            this.readyState = Wrapper.OPEN;
            this.dispatchEvent(new Event("open"));
        }).catch(e => {
            console.error(e);
            this.readyState = Wrapper.CLOSED;
            this.dispatchEvent(new Event("error"));
        });

    }

    close(code?: number, reason?: string): void {
        this.abort?.abort(reason);
        this.socket?.disconnect().catch(e => {
            console.error(e);
            this.dispatchEvent(new Event("error"));
        });
        this.readyState = Wrapper.CLOSED;
        this.socket = null;
        this.dispatchEvent(new Event("close"));
    }
    send(data: string | ArrayBufferLike | Blob | ArrayBufferView): void {
        if (typeof data !== "string") throw new Error("Unsupported");
        this.socket?.send(data).catch(err => {
            console.error(err);
            this.dispatchEvent(new Event("error"));
        });
    }

    OPEN: 1 = Wrapper.OPEN;
    CLOSING: 2 = Wrapper.CLOSING;
    CLOSED: 3 = Wrapper.CLOSED;
    CONNECTING: 0 = Wrapper.CONNECTING;
    static CONNECTING: 0 = 0
    static OPEN: 1 = 1;
    static CLOSING: 2 = 2;
    static CLOSED: 3 = 3;

}

export function getWebSocket() {
    if (import.meta.env.VITE_USE_INVALID_CERT === "1") {
        return Wrapper;
    }

    return WebSocket;
}