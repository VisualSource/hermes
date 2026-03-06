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

	async init() {}
}
