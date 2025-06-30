import type { BunRequest } from "bun";

import { join } from "node:path";
import { HttpError } from "./utils";
import { password } from "bun";
import { Database } from "bun:sqlite";
import { schemaLogin, schemaSignup } from "./schemas";
import { User } from "./model";
import home from "../client/index.html";

import { jwtVerify, SignJWT } from "jose";
const alg = "HS256"
const database = new Database(join(import.meta.dir, "server.db"));
const secret = new TextEncoder().encode(btoa("SomePasswordForSigningAndVerifyingJwtsThisShouldBeReplacedWithABetterSecert"))

await database.exec("CREATE TABLE IF NOT EXISTS users (id text PRIMARY KEY, username TEXT UNIQUNE NOT NULL, psdHash TEXT NOT NULL, icon TEXT)")

const auth = async (req: BunRequest | string | null) => {
    if (!req) throw new HttpError("missing auth header", { statusCode: 401 });

    let jwt: string
    if (typeof req === "string") {
        if (req.length === 0) throw new HttpError("missing auth header", { statusCode: 401 });
        jwt = req;
    } else {
        const authHeader = req.headers.get("Authorization");
        if (!authHeader) throw new HttpError("missing auth header", { statusCode: 401 });
        const token = authHeader.split(" ")[1];
        if (!token) throw new HttpError("missing auth header", { statusCode: 401 });
        jwt = token;
    }

    try {
        const { payload } = await jwtVerify(jwt, secret, {
            algorithms: [alg],
            issuer: "com:loadingzone:hermes",
            audience: "com:loadingzone:hermes"
        });

        const user = await database.prepare("SELECT * FROM users WHERE id = ?").as(User).get((payload as never as Record<string, string>).user as string);

        if (!user) throw new Error("User is not found");

        return user;
    } catch (error) {
        throw new HttpError("Auth failed", { statusCode: 401, cause: error });
    }
}

Bun.serve({
    hostname: "0.0.0.0",
    port: 5000,
    development: false,
    //@ts-expect-error
    tls: {
        passphrase: "hermdevclone",
        cert: Bun.file(join(import.meta.dir, "./ca-cert.pem")),
        key: Bun.file(join(import.meta.dir, "./ca-key.pem"))
    },
    websocket: {
        message(ws, message) {
            if (typeof message !== "string") {
                console.log("Invalid message");
                return;
            }
            const payload = JSON.parse(message);
            const user = (ws.data as Record<string, string>).usr;

            switch (payload.type) {
                case "join":
                    console.log(`User '${user}' joined ${payload.data.channelId} in room ${payload.data.roomId}`);

                    //const channel = channels.get(payload.data.channelId);

                    //channel?.join(user, {});

                    ws.subscribe(`${payload.data.channelId}/${payload.data.roomId}`);
                    ws.subscribe(`${payload.data.channelId}/${payload.data.roomId}/${user}`);

                    ws.publish(`${payload.data.channelId}/${payload.data.roomId}`, JSON.stringify({
                        type: "joined",
                        data: {
                            user
                        }
                    }));

                    break;
                case "offer": {
                    console.log(`User '${user}' sending offer to user '${payload.data.forUser}'`);
                    ws.publish(`${payload.data.channelId}/${payload.data.roomId}/${payload.data.forUser}`, JSON.stringify({
                        type: "offer",
                        data: {
                            user,
                            sdp: payload.data.sdp
                        }
                    }));
                    break;
                }
                case "answer": {
                    console.log(`User '${user}' sending answer to user '${payload.data.forUser}'`);
                    ws.publish(`${payload.data.channelId}/${payload.data.roomId}/${payload.data.forUser}`, JSON.stringify({
                        type: "answer",
                        data: {
                            user: (ws.data as Record<string, string>).usr,
                            sdp: payload.data.sdp
                        }
                    }));
                    break;
                }
                default:
                    break;
            }
        },
    },
    routes: {
        "/ws": (req, server) => {
            const url = new URL(req.url);
            const token = url.searchParams.get("token");

            const usr = auth(token);

            const upgraded = server.upgrade(req, {
                data: {
                    usr
                }
            })
            if (!upgraded) {
                return new Response("Upgrade failed", { status: 400 });
            }
        },
        "/api/channel": {
            DELETE: async (req: BunRequest) => {
                auth(req);

                const url = new URL(req.url);
                const channelId = url.searchParams.get("channelId");
                if (!channelId) throw new HttpError("Missing channel id", { statusCode: 404 });

                // validate channel id               


                return Response.json({ ok: true });
            },
            POST: async (req) => {
                auth(req);

                const uuid = crypto.randomUUID();

                const body = await req.json();
                // validate body

                return Response.json({});
            }
        },

        "/api/login": {
            POST: async (req: BunRequest) => {
                const body = await req.json();
                const form = schemaLogin.safeParse(body);
                if (!form.success) {
                    throw new HttpError(form.error.message, { statusCode: 400 });
                }
                const { username, password: psd } = form.data;

                const user = await database.prepare("SELECT * FROM users WHERE username = ?").as(User).get(username);
                if (!user) {
                    throw new HttpError("No user", { statusCode: 404 });
                }

                const valid = await password.verify(psd, user.psdHash);
                if (!valid) {
                    throw new HttpError("Authorized", { statusCode: 401 });
                }

                const jwt = await new SignJWT({ user: user.id })
                    .setProtectedHeader({ alg })
                    .setIssuedAt()
                    .setIssuer("com:loadingzone:hermes")
                    .setAudience("com:loadingzone:hermes")
                    .setExpirationTime("4h")
                    .sign(secret);

                return Response.json({
                    jwt,
                    user: user.toJson()
                });
            }
        },
        "/api/signup": {
            POST: async (req: BunRequest) => {
                const body = await req.json();

                const form = schemaSignup.safeParse(body);
                if (!form.success) {
                    throw new HttpError(form.error.message, { statusCode: 400 });
                }
                const { username, password: psd } = form.data;

                const psdHash = await password.hash(psd);

                const result = await database.prepare("INSERT INTO users (id,username,psdHash) VALUES (?,?,?) RETURNING id").as(User).get(crypto.randomUUID(), username, psdHash);

                if (!result) throw new HttpError("Failed to get user");

                const jwt = await new SignJWT({ user: result.id })
                    .setProtectedHeader({ alg })
                    .setIssuedAt()
                    .setIssuer("com:loadingzone:hermes")
                    .setAudience("com:loadingzone:hermes")
                    .setExpirationTime("4h")
                    .sign(secret);

                return Response.json({
                    jwt,
                    user: result.toJson()
                });
            }
        },

        "/": home,

        "/*": new Response("Not Found", { status: 404 }),
    },
    error(error) {
        console.error(error);

        if (error instanceof HttpError) {
            return Response.json({
                message: error.message,
                status: error.statusCode,
                name: error.name
            }, {
                status: error.statusCode,
                statusText: error.statusText
            });
        }

        return Response.json({
            message: "Internal Server Error",
            status: 500,

        }, {
            status: 500,
        })
    },
});

console.log("Server Ready...");