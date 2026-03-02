import {
	generateCodeVerifier,
	
	OAuth2Client,
	OAuth2Fetch,
	type OAuth2Token,
} from "@badgateway/oauth2-client";
import { millisecondsToSeconds, getUnixTime } from "date-fns";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { nanoid } from "nanoid";
import { platform } from "@tauri-apps/plugin-os"
import { openUrl } from "@tauri-apps/plugin-opener";

type AuthFlowEvent = { type: "Cancel" } | { type: "Done", path: string } | { type: "Error", reason: string };

export class OAuth {
	token: OAuth2Token | null = null;
	private client = new OAuth2Client({
		clientId: import.meta.env.VITE_CLIENT_ID,
		server: import.meta.env.VITE_SERVER_URL,
		tokenEndpoint: "/auth/token",
		authorizationEndpoint: "/auth/authorize",
	});

	private fetcher: OAuth2Fetch;

	constructor(){
		this.fetcher = new OAuth2Fetch({
			client: this.client,
			getNewToken: () => this.authorize(),
			getStoredToken: () => {
				return this.token
			},
			scheduleRefresh: true,
			storeToken: (token) => {
				this.token = token;
			},
		});
	}


	get isAuthed() {
		return this.token !== null;
	}

	async init() {
		const tokenRaw = localStorage.getItem("auth");
		if (!tokenRaw) return false;

		const token = JSON.parse(tokenRaw) as OAuth2Token;
		
		if(token){
			this.token = token;
		}
		/*if (
			tokens.expiresAt &&
			getUnixTime(Date.now()) > millisecondsToSeconds(tokens.expiresAt)
		) {
			try {
				await this.refresh();
			} catch (error) {
				localStorage.removeItem("auth");
				this.token = null;
				console.error(error);
			}
		}*/
	}

	async refresh() {
		if (!this.token) return;

		this.token = await this.client.refreshToken(this.token);

		localStorage.setItem("auth", JSON.stringify(this.token));
	}

	async authorize() {
		const state = encodeURIComponent(nanoid());
		const codeVerifier = await generateCodeVerifier();
		const redirectUri = "hermes://oauth";
		const url = await this.client.authorizationCode.getAuthorizeUri({
			codeVerifier,
			scope: ["profile", "offline_access"],
			redirectUri,
			state
		});

		const { reject, resolve, promise } = Promise.withResolvers<string|null>();

		const unlisten = await listen<AuthFlowEvent>("hermes://auth",(ev)=>{
			switch(ev.payload.type){
				case "Cancel":
					resolve(null);
					break;
				case "Done":
					resolve(ev.payload.path);
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
					await invoke<void>("start_auth_grant",{ path: url });
					break;	
				default:
					await openUrl(url)
					break;
			}

			const callback_uri = await promise;
			await unlisten;

			console.log(callback_uri);

			if(!callback_uri) return null;

			this.token = await this.client.authorizationCode.getTokenFromCodeRedirect(
				callback_uri,
				{ redirectUri, codeVerifier, state },
			);
			console.log(this.token)

			localStorage.setItem("auth",JSON.stringify(this.token));

			return this.token;
		} catch (error) {
			console.error(error);
			return null; 
		} finally {
			unlisten();
		}
	}

	fetch(){
		return this.fetcher.fetch;
	}
}
