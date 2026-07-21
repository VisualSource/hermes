import { initSharedAlpine, type ErrorObject, type AppStore } from "./shared.js";
import Alpine from "alpinejs"; 

document.addEventListener("DOMContentLoaded",()=>{
    initSharedAlpine();
    Alpine.data("appForm",()=>({
        pending: false, 
        async submit(ev: Event){ 
            this.pending = true; 
            try {
                await onSubmit(ev.target as HTMLFormElement);
                (this.$store.app as AppStore).showDone();
            } catch (error) {
                console.error(error);
                (this.$store.app as AppStore).showError("Error",String(error));
            } finally {
                this.pending = false;
            }
        }, 
    }));
    Alpine.start();
});
     
async function onSubmit(target: HTMLFormElement){
    try {
        const data = new FormData(target);
 
        const csrf = data.get("CSRF_TOKEN_FIELD")?.toString();
        if(!csrf) throw new Error("missing required field");
        data.delete("CSRF_TOKEN_FIELD");
        const fields = Object.fromEntries(data.entries().map(([key,value])=>[key,value.toString()]));
        
        const body = new URLSearchParams(fields);
        const path = new URL("/login",window.location.origin);
        const response = await fetch(path,{
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
    } catch (error) {
        if(error instanceof TypeError) {
            throw new Error("a network error occurred", { cause: error });
        }
        throw error;
    }
}