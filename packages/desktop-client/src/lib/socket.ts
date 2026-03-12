import { BinaryReader } from "@bufbuild/protobuf/wire";

import { auth } from "./clients";

import { Envelope } from "./proto/hermes";
import { nanoid } from "nanoid";
export class SocketManager extends EventTarget {
		private static INSTANCE: SocketManager | null = null;

		static get() {
			if (!SocketManager.INSTANCE) throw new Error();
			return SocketManager.INSTANCE;
		}

		private socket: WebSocket | null = null;

		static async create() {
			const man = new SocketManager();
			await man.init();

			SocketManager.INSTANCE = man;

			return SocketManager.INSTANCE;
		}

		async init() {
			console.log("[Websocket] Starting websocket");

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

			this.socket = socket;
		}

		public send(msg: Omit<Envelope,"messageId"|"timestamp"|"version"|"type">){
			const envelope = Envelope.create({
				messageId: nanoid(),
				timestamp: Date.now(),
				version: 1,
				...msg			
			});

			const data = Envelope.encode(envelope).finish();

			this.socket?.send(data);
		}

		private onMessage = (ev: MessageEvent<ArrayBuffer>) => { 
			if(!(ev.data instanceof ArrayBuffer)){
				console.log("Unable to handle text frame");
				return;
			} 

			const reader = new BinaryReader(new Uint8Array(ev.data));
			
			const envelope = Envelope.decode(reader);

			if(envelope.rtc){
				// emit to RTC handler
			} else if(envelope.rtcIce){

			}

		
		};
		private onClose = (ev: CloseEvent) => { console.log("Socket Closed!",ev) };
	}
