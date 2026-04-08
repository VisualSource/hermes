import {
	generateCodeVerifier,
	OAuth2Client,
	type OAuth2Token,
} from "@badgateway/oauth2-client";
import { millisecondsToSeconds, getUnixTime } from "date-fns";
import { listen } from "@tauri-apps/api/event";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { nanoid } from "nanoid";
import { platform } from "@tauri-apps/plugin-os";
import { openUrl } from "@tauri-apps/plugin-opener";

type AuthFlowEvent =
	| { type: "Cancel" }
	| { type: "Done"; url: string }
	| { type: "Error"; reason: string };

export class OAuth {
		private _token: OAuth2Token | null = null;
		private client = new OAuth2Client({
			clientId: import.meta.env.VITE_CLIENT_ID,
			server: import.meta.env.VITE_SERVER_URL,
			tokenEndpoint: "/auth/token",
			authorizationEndpoint: "/auth/authorize",
		});

		get isAuthed() {
			return this._token !== null;
		}

		get token() {
			if (!this._token) throw new Error("failed to get access token");
			return this._token?.accessToken;
		}

		async init() {
			const tokenRaw = localStorage.getItem("auth");
			if (!tokenRaw) return false;

			const token = JSON.parse(tokenRaw) as OAuth2Token;

			if (token) {
				this._token = token;
			}
			if (
				token.expiresAt &&
				getUnixTime(Date.now()) > millisecondsToSeconds(token.expiresAt)
			) {
				try {
					await this.refresh();
				} catch (error) {
					localStorage.removeItem("auth");
					this._token = null;
					console.error(error);
				}
			}
		}

		async refresh() {
			if (!this._token) return;

			this._token = await this.client.refreshToken(this._token);

			localStorage.setItem("auth", JSON.stringify(this._token));
		}

		async authorize() {
			const state = encodeURIComponent(nanoid());
			const codeVerifier = await generateCodeVerifier();
			const redirectUri = isTauri()
				? "hermes://oauth"
				: new URL("/oauth", window.location.origin).toString();
			const url = await this.client.authorizationCode.getAuthorizeUri({
				codeVerifier,
				scope: ["profile", "offline_access"],
				redirectUri,
				state,
			});

			const { reject, resolve, promise } = Promise.withResolvers<
				string | null
			>();

			if (!isTauri()) {
				const win = window.open(url, "_blank", "width=480,height=600");
				if (!win) throw new Error("unable to process request");

				win.addEventListener("beforeunload", () => reject("no win"));

				setInterval(() => {
					const loc = win.location.origin;
					console.log(loc);
				}, 5000);

				await promise;

				return;
			}

			const unlisten = await listen<AuthFlowEvent>("hermes://auth", (ev) => {
				console.log(ev);
				switch (ev.payload.type) {
					case "Cancel":
						resolve(null);
						break;
					case "Done":
						resolve(ev.payload.url);
						break;
					case "Error":
						reject(new Error(ev.payload.reason));
						break;
					default:
						reject(new Error("Unkown event", { cause: ev }));
				}
			});

			try {
				const currentPlatform = platform();
				switch (currentPlatform) {
					case "windows":
						await invoke<void>("start_auth_grant", { path: url });
						break;
					default:
						await openUrl(url);
						break;
				}

				const callback_uri = await promise;
				if (!callback_uri) return null;

				this._token =
					await this.client.authorizationCode.getTokenFromCodeRedirect(
						callback_uri,
						{ redirectUri, codeVerifier, state },
					);

				localStorage.setItem("auth", JSON.stringify(this._token));

				return this.token;
			} catch (error) {
				console.error(error);
				return null;
			} finally {
				unlisten();
			}
		}
	}
