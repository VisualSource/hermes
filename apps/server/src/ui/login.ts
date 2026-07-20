import { initSharedAlpine, type ErrorObject } from "./shared.js";
import Alpine from "alpinejs"; 

type AppStore = {
    done: boolean;
    dialog: {
        title: string;
        desc: string;
    };
    showError(title: string, desc: string ): void;
    showDone(): void;
}

document.addEventListener("DOMContentLoaded",()=>{
    initSharedAlpine();
    Alpine.store("app",{
        done: false,
        dialog: {
            title: "",
            desc: "",
        },
        showError(title: string, desc: string){
            this.dialog.title = title;
            this.dialog.desc = desc;
        },
        showDone(){
            this.done = true;
        }
    } as AppStore);
    Alpine.data("appForm",()=>({
        pending: false, 
        async submit(ev: Event){ 
            this.pending = true; 
            try {
                await onSubmit(ev.target as HTMLFormElement);
                (this.$store.app as AppStore).showDone();
            } catch (error) {
                (this.$store.app as AppStore).showError("Error",String(error));
            } finally {
                this.pending = false;
            }
        }, 
    }));
    Alpine.start();
    console.log("started alpinejs: " + Alpine.version)
});
     
async function onSubmit(target: HTMLFormElement){
    try {
        const data = new FormData(target);
                
        const csrf = data.get("CSRF_TOKEN_FIELD")?.toString();
        if(!csrf) throw new Error("missing required field");
        data.delete("CSRF_TOKEN_FIELD");

        const path = new URL("/login",window.location.origin);
        const response = await fetch(path,{
            method: "post",
            body: new URLSearchParams(Object.entries(data.entries())),
            headers: new Headers({
                "X-CSRF-Token": csrf
            })
        });

        if(!response.ok) {
            const body = (await response.json()) as ErrorObject;
            throw new Error(body.message, { cause: body });
        }

        const query = new URLSearchParams(window.location.search);
        const returnTo = query.get("return_to");
        if(!returnTo) throw new Error("no return_to query param in response");
        const url = new URL(returnTo);
        if(url.origin !== window.location.origin) throw new Error("invalid return to origin");
        window.location.href = url.toString();
    } catch (error) {
        if(error instanceof TypeError) {
            throw new Error("a network error occurred", { cause: error });
        }
        throw error;
    }
}