import { join, normalize } from "node:path";
import { HttpError } from "./utils";
import home from "../client/index.html";
import { socketCommand } from "hermes-shared";
import { z } from "zod/v4";
import { socketHandlers } from "./lib/socketHandlers";
import { auth } from "./lib/auth";
import loginHandler from "./routes/login";
import signupHandler from "./routes/signup";
import refreshHandler from "./routes/refresh";
import userHandler from "./routes/user";
import channelDirHandler from "./routes/channel";
import roomHandler from "./routes/room";
import { log } from "./lib/utils";
import type { User } from "./model";
import { initDB } from "./lib/database";
import messagesHandler from "./routes/messages";
await initDB();

Bun.serve({
    hostname: "0.0.0.0",
    port: 5000,
    development: false,
    //@ts-expect-error
    tls: {
        passphrase: process.env.TLS_PASSKEY,
        cert: Bun.file(join(import.meta.dir, process.env.TLS_CERT as string)),
        key: Bun.file(join(import.meta.dir, process.env.TLS_KEY as string))
    },
    websocket: {
        open(ws) {
            ws.subscribe(process.env.DEFAULT_CHANNEL_ID as string); // CAHNNEL ID
        },
        async message(ws, message) {
            if (typeof message !== "string") {
                console.error("Invalid message");
                return;
            }

            try {
                const { type, data } = socketCommand.parse(JSON.parse(message));

                console.log(`[CLEINT -> SERVER][${type}]`);

                const handler = socketHandlers[type];
                if (!handler) throw new Error(`Unable to handle event '${type}'`);
                await handler({}, ws as Bun.ServerWebSocket<{ user: User }>, data as never);
            } catch (error) {
                const errorMessage = error instanceof z.ZodError ? z.prettifyError(error) : Error.isError(error) ? error.message : "Unknown Error"

                ws.send(JSON.stringify({
                    type: "error",
                    data: {
                        error: errorMessage
                    }
                }));

                const server_error = error instanceof z.ZodError ? z.treeifyError(error) : error;
                console.error("[CLIENT -> SERVER] Socket Error:", server_error);
            }
        },
    },
    routes: {
        "/ws": async (req, server) => {
            log(req);
            const url = new URL(req.url);
            const token = url.searchParams.get("token");

            const user = await auth(token);

            const upgraded = server.upgrade(req, {
                data: {
                    user
                }
            });

            if (!upgraded) {
                return new Response("Upgrade failed", { status: 400 });
            }
        },
        "/api/room/:uuid/messages": messagesHandler,
        "/api/room/:uuid": roomHandler,
        "/api/channel/:uuid": channelDirHandler,
        "/api/user/:uuid": userHandler,
        "/api/refresh": refreshHandler,
        "/api/login": loginHandler,
        "/api/signup": signupHandler,

        "/": home,
        "/file/:filename": {
            GET: (req) => {
                const filename = req.params.filename;
                return new Response(Bun.file(join(import.meta.dirname, "..", "public", normalize(filename))), {
                    headers: {
                        "Content-Type": `application/${filename.endsWith("js") ? "javascript" : "wasm"}`
                    }
                })
            }
        },
        "/*": new Response("Not Found", { status: 404 }),
    },
    error(error) {
        if (error instanceof HttpError) {

            if (error.statusCode < 500) {
                console.error(`[${error.statusCode}] ${error.message}`)
            } else {
                console.error(error);
            }

            return Response.json({
                message: error.message,
                status: error.statusCode,
                name: error.name
            }, {
                status: error.statusCode,
                statusText: error.statusText
            });
        }

        console.error(error);

        return Response.json({
            message: "Internal Server Error",
            status: 500,

        }, {
            status: 500,
        })
    },
});

console.log("Server Ready...");