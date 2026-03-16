import { warn, error, info, trace, } from "@tauri-apps/plugin-log";

function forward(level: Extract<keyof Console,"log"| "info"|"error"|"warn"|"trace">, logger: (message: string)=>Promise<void>){
    const original = console[level];
    console[level] = (message: string) => {
        original(message);
        logger(message).catch((err)=>console.debug("[LOGGER ERROR]",err));
    }
}

forward("warn",warn);
forward("error",error);
forward("info",info);
forward("log",trace);