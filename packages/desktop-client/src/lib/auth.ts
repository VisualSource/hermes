import {
	generateCodeVerifier,
	OAuth2Client,
	type OAuth2Token,
} from "@badgateway/oauth2-client";
import { millisecondsToSeconds, getUnixTime } from "date-fns";

export class OAuth {
	token: OAuth2Token | null = null;
	private client = new OAuth2Client({
		clientId: import.meta.env.TAURI_CLIENT_ID,
		server: import.meta.env.TAURI_SERVER_URL,
		tokenEndpoint: "/oauth/token",
		authorizationEndpoint: "/oauth/authorize",
		discoveryEndpoint: "/.well-known/oauth2-authorization-server",
	});

	get isAuthed() {
		return this.token !== null;
	}

	async init() {
		const tokenRaw = localStorage.getItem("auth");
		if (!tokenRaw) return;

		const tokens = JSON.parse(tokenRaw) as OAuth2Token;
		if (!tokens) {
			localStorage.removeItem("auth");
			return;
		}

		if (
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
		}
	}

	async refresh() {
		if (!this.token) return;

		this.token = await this.client.refreshToken(this.token);

		localStorage.setItem("auth", JSON.stringify(this.token));
	}

	async authorize() {
		const codeVerifier = await generateCodeVerifier();
		const redirectUri = "hermes_vc://oauth/";
		const url = await this.client.authorizationCode.getAuthorizeUri({
			codeVerifier,
			scope: ["user:read", "user:write"],
			redirectUri,
		});

		// open window
		// wait for callback
		const callback_uri = "";

		this.token = await this.client.authorizationCode.getTokenFromCodeRedirect(
			callback_uri,
			{ redirectUri, codeVerifier },
		);

		return url;
	}
}
