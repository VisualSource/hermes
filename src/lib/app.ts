import Auth from "./auth";
import { RTC } from "./rtc";

export class App {
    #auth: Auth = new Auth(this);
    #rtc = new RTC(this);

    #channel: string | null = null;

    get auth() {
        return this.#auth;
    }

    get rtc() {
        return this.#rtc;
    }
}
