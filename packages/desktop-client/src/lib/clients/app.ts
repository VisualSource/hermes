import { App } from "../core/app";

export const app = new App();

export const makeSubscription = <T = null>(event: string, prop: string) => {
	return {
		subscription: (callback: () => void) => {
			app.addEventListener(event, callback);
			return () => {
				app.removeEventListener(event, callback);
			};
		},
		snapshot: () => app.getProp(prop) as T,
	};
};
