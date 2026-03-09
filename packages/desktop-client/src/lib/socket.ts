import { auth } from "./clients";

export class SocketManager extends EventTarget {
		private static INSTANCE: SocketManager | null = null;

		static get() {
			if (!SocketManager.INSTANCE) throw new Error();
			return SocketManager.INSTANCE;
		}

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

			const { resolve, reject, promise } = Promise.withResolvers<void>();

			socket.addEventListener("close", this.onClose);
			socket.addEventListener("error", (ev) => {
				console.log(ev);
				reject(ev);
			});
			socket.addEventListener("message", this.onMessage);
			socket.addEventListener("open", () => resolve());

			await promise;
		}

		private onMessage = () => {};
		private onClose = () => {};
	}
