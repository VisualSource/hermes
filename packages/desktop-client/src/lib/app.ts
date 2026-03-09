export class App extends EventTarget {
	private static INSTANCE: App | null = null;
	public static get(): App {
		if (!App.INSTANCE) throw new Error("failed to get app instance");

		return App.INSTANCE;
	}

	static create() {
		App.INSTANCE = new App();
	}

	inVoice: boolean = false;

	joinVoice = (id: string) => {
		console.log("Init WEB RTC", id);

		this.inVoice = true;

		this.dispatchEvent(new Event("voice-state-change"));
	};
	leaveVoice = (id: string) => {};

	getProp = (prop: string) => {
		switch (prop) {
			case "inVoice":
				return this.inVoice;
			default:
				return null;
		}
	};
}

export const makeSubscription = <T = null>(event: string, prop: string) => {
	return {
		subscription: (callback: () => void) => {
			const inst = App.get();
			inst.addEventListener(event, callback);
			return () => {
				inst.removeEventListener(event, callback);
			};
		},
		snapshot: () => App.get().getProp(prop) as T,
	};
};
