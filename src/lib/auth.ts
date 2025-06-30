import { fetch } from "@tauri-apps/plugin-http";

export default class Auth extends EventTarget {
    #authenticated = false;
    #user: { icon: string; id: string; username: string } | null = null
    #token: string | null = null;

    constructor() {
        super();

        const token = localStorage.getItem("token");

        if (token) {
            // ask for user

            this.#authenticated = true;
        }
    }

    async signup(username: string, password: string) {
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

        const data = await response.json() as { jwt: string; user: { id: string; username: string; icon: string } };

        this.#user = data.user;
        this.#token = data.jwt;

        this.#authenticated = true;

        localStorage.setItem("token", this.#token);

        this.dispatchEvent(new Event("login"));
    }

    async login(username: string, password: string) {

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

        const data = await response.json() as { jwt: string; user: { id: string; username: string; icon: string } };

        this.#user = data.user;
        this.#token = data.jwt;

        this.#authenticated = true;

        localStorage.setItem("token", this.#token);

        this.dispatchEvent(new Event("login"));
    }

    get isAuthenticated() {
        return this.#authenticated;
    }

    get user() {
        return this.#user;
    }
}