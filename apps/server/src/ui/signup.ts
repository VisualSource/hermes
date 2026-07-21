import { initSharedAlpine, type ErrorObject, type AppStore, getErrorMessage } from "./shared.js";
import Alpine from "alpinejs";

type SignupState = {
    title: string;
    desc: string;
    open: boolean;
    openRoute: string;
}

document.addEventListener("DOMContentLoaded",()=>{
    initSharedAlpine();
    Alpine.store("signupState",{
        title: "Account Created",
        desc: "Download the Hermes desktop or mobile app to get started.",
        openRoute: "",
        open: false,
    } as SignupState);
    Alpine.data("appForm",()=>({
        pending: false,
        async onSubmit(ev: SubmitEvent){
            this.pending = true;
            try {
                const formData = new FormData(ev.target as HTMLFormElement);
                await submit(formData);
                (this.$store.app as AppStore).showDone();
            } catch (err) {
                console.error(err);
                (this.$store.app as AppStore).showError("Error",String(err instanceof TypeError ? "A network error occurred" : err));
            } finally {
                this.pending = false;
            }
        }
    }));
    Alpine.data("checker",()=>({
        errors: [] as string[],
        hasErrors: false,
        target: null as HTMLInputElement | null,
        init(){
            const target = document.getElementById("psd");
            this.target = target as HTMLInputElement;
        },
        onInput(ev: InputEvent){
            const field = ev.target as HTMLInputElement | null;
            if(!this.target || !field) return;
            this.errors = [];

            const psdsMatch = this.target.value === field.value;
            field.setCustomValidity(psdsMatch ? "" : "Passwords do not match");
            field.reportValidity();
        
           	if(!field.checkValidity()){
				field.setAttribute("aria-invalid", "true");
				const message = getErrorMessage(field);
				if (!message) return;
				this.errors.push(message);
				this.hasErrors = true;
			} else {
				field.removeAttribute("aria-invalid");
				this.errors = [];
				this.hasErrors = false;
			}
        }
    }));
    Alpine.start();
});

const submit = async (data: FormData) => {
    const csrf = data.get("CSRF_TOKEN_FIELD")?.toString();
    if(!csrf) throw new Error("missing required field");
    data.delete("CSRF_TOKEN_FIELD");

    const psdCheckField = data.get("psd-check")?.toString();
    const psd = data.get("password")?.toString();

    if(psd !== psdCheckField) throw new Error("passwords does not match");
    data.delete("psd-check");

    const fields = Object.fromEntries(data.entries().map(([key,value])=>[key,value.toString()]));
        
    const body = new URLSearchParams(fields);
    const response = await fetch("/signup",{
        method: "post",
        body,
        headers: new Headers({
            "X-CSRF-Token": csrf
        })
    });

    if(!response.ok) {
        const body = (await response.json()) as ErrorObject;
        throw new Error(body.message, { cause: body });
    }

    const store = Alpine.store("signupState") as SignupState
    if(response.status === 201) {
        store.title = "Account Created";
        store.desc = "Download the Hermes desktop or mobile app to get started.";
        store.open = false;
    } else {
        const body = await response.json() as { redirect: string; };

        store.title = "Opening Hermes";
        store.desc = "Returning you to the app";
        store.openRoute = body?.redirect ?? "hermes://open";
        store.open = true;
    }

    try {
        const query = new URLSearchParams(window.location.search);
        const returnTo = query.get("return_to");
        if(returnTo) {
            const url = new URL(returnTo,window.location.origin);
            if(url.origin !== window.location.origin) throw new Error("invalid return to origin");
            window.location.href = url.toString();
        }
    } catch (error) {
        console.error(error);
    }
}