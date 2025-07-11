import { fetch } from "@tauri-apps/plugin-http";
import type { App } from "./app";

type User = {
    id: string;
    username: string;
    icon: string;
}

type RefreshResponse = {
    token: string;
    refreshToken: string;
}

type LoginResponse = {
    user: User
} & RefreshResponse;

export default class Auth extends EventTarget {
    #authenticated = false;
    #user: User | null = null
    #token: string | null = null;
    #loaded = false;

    #profiles = new Map<string, User>();
    #profileLoadingCache = new Map<string, Promise<User>>();

    constructor(private app: App) {
        super();
    }

    public async init() {
        if (this.#loaded) return this.isAuthenticated;
        try {
            const [token, refreshToken] = this.loadStoarge();

            this.#token = token;

            if (!token) {
                console.log("[AUTH] No auth token ignoring");
                return;
            }
            const value = token.split(".")[1];
            if (!value) {
                this.clearStoarge();
                throw new Error("Invalid token");
            }

            // ask for user
            const payload = JSON.parse(atob(value)) as { exp: number, sub: string };

            if (Date.now() >= payload.exp * 1000) {
                this.clearStoarge();

                if (!refreshToken) {
                    console.log("[AUTH] No refresh token, ignoring");
                    return;
                }

                const response = await fetch(`https://${import.meta.env.VITE_SERVER_HOST}/api/refresh`, {
                    method: "POST",
                    danger: {
                        acceptInvalidCerts: true,
                        acceptInvalidHostnames: false
                    },
                    body: JSON.stringify({ refreshToken })
                });

                if (!response.ok) {
                    throw new Error(response.statusText, { cause: response });
                }

                const data = await response.json() as RefreshResponse;
                this.setStoarge(data.token, data.refreshToken);
            }

            console.log("[AUTH] Loading user data for %s", payload.sub);
            const self = await this.getUser(payload.sub);

            this.#user = self;
            this.#profiles.set(this.#user.id, this.#user);

            this.#authenticated = true;
        } catch (error) {
            console.error(error);
        } finally {
            this.#loaded = true;
        }

        return this.isAuthenticated
    }

    //#region Api

    public async signup(username: string, password: string) {
        const response = await fetch(`https://${import.meta.env.VITE_SERVER_HOST}/api/signup`, {
            method: "POST",
            danger: {
                acceptInvalidCerts: true,
                acceptInvalidHostnames: false
            },
            body: JSON.stringify({
                username,
                password
            })
        });

        if (!response.ok) {
            throw new Error(`signup failed: ${response.statusText}`, { cause: response });
        }

        const data = await response.json() as LoginResponse;

        this.setStoarge(data.token, data.refreshToken);

        this.#user = data.user;
        this.#authenticated = true;

        this.dispatchEvent(new Event("login"));
    }

    public async login(username: string, password: string) {

        const response = await fetch(`https://${import.meta.env.VITE_SERVER_HOST}/api/login`, {
            method: "POST",
            danger: {
                acceptInvalidCerts: true,
                acceptInvalidHostnames: false
            },
            body: JSON.stringify({
                username,
                password
            })
        });

        if (!response.ok) {
            throw new Error(`Login failed: ${response.statusText}`, { cause: response });
        }

        const data = await response.json() as LoginResponse;

        console.log("[AUTH] User login", data.user);

        this.#user = data.user;

        this.#authenticated = true;

        this.setStoarge(data.token, data.refreshToken);

        this.dispatchEvent(new Event("login"));
    }

    public async getUser(userId: string) {
        if (!this.#token) throw new Error("No authorization");

        const response = await fetch(`https://${import.meta.env.VITE_SERVER_HOST}/api/user/${userId}`, {
            method: "GET",
            headers: {
                Authorization: `Bearer ${this.#token}`
            },
            danger: {
                acceptInvalidCerts: true,
                acceptInvalidHostnames: false
            }
        });

        if (!response.ok) {
            throw new Error(`Login failed: ${response.statusText}`, { cause: response });
        }

        const data = await response.json() as User;

        console.log("[AUTH] fetched user", data);

        return data;
    }

    public async getTextRoomMessages(roomId: string) {
        if (!this.#token) throw new Error("No authorization");

        const response = await fetch(`https://${import.meta.env.VITE_SERVER_HOST}/api/room/${roomId}/messages`, {
            method: "GET",
            headers: {
                Authorization: `Bearer ${this.#token}`
            },
            danger: {
                acceptInvalidCerts: true,
                acceptInvalidHostnames: false
            }
        });

        if (!response.ok) throw new Error(response.statusText, { cause: response });

        const data = await response.json() as { message: string; roomId: string; user: string }[];

        return data;
    }

    public async getChannelDetails(channelId: string) {
        if (!this.#token) throw new Error("No authorization");

        const response = await fetch(`https://${import.meta.env.VITE_SERVER_HOST}/api/channel/${channelId}`, {
            method: "GET",
            headers: {
                Authorization: `Bearer ${this.#token}`
            },
            danger: {
                acceptInvalidCerts: true,
                acceptInvalidHostnames: false
            }
        });

        if (!response.ok) throw new Error(response.statusText, { cause: response });

        const data = await response.json() as { rooms: { id: string; name: string; channelId: string; type: "text" | "media" }[] };

        return data;
    }

    //#endregion

    //#region React

    public hookGetUser(userId: string) {
        const promise = this.#profileLoadingCache.get(userId);
        if (!promise) {
            const user = this.#profiles.get(userId);
            if (user) {
                const p = Promise.try(() => user);
                this.#profileLoadingCache.set(userId, p);
                return p;
            }

            const f = this.getUser(userId).then((user) => {
                this.#profiles.set(userId, user);
                return user;
            });
            this.#profileLoadingCache.set(userId, f);
            return f;
        }

        return promise;
    }

    //#endregion

    //#region Stoarge
    private setStoarge(token: string, refreshToken: string) {
        this.#token = token;

        localStorage.setItem("token", token);
        localStorage.setItem("refreshToken", refreshToken);
    }

    private loadStoarge() {
        return [
            localStorage.getItem("token"),
            localStorage.getItem("refreshToken")
        ];
    }

    private clearStoarge() {
        this.#token = null;
        this.#authenticated = false;
        localStorage.removeItem("token");
        localStorage.removeItem("refreshToken");
    }

    //#endregion

    //#region Getters

    public get isAuthenticated() {
        return this.#authenticated;
    }

    public get user() {
        return this.#user;
    }

    //#endregion
}