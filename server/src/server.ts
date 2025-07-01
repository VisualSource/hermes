import type { BunRequest } from "bun";

import { join } from "node:path";
import { HttpError } from "./utils";
import { password } from "bun";
import { Database } from "bun:sqlite";
import { schemaLogin, schemaRefresh, schemaSignup, schemaUuid } from "./schemas";
import { User } from "./model";
import home from "../client/index.html";

import { jwtVerify, SignJWT } from "jose";
const alg = "HS256"
const database = new Database(join(import.meta.dir, "server.db"));
const secret = new TextEncoder().encode(btoa("SomePasswordForSigningAndVerifyingJwtsThisShouldBeReplacedWithABetterSecert"));
const refreshSecret = new TextEncoder().encode(btoa("DontQutgetTheREfreshTokenJustBeingOtherJWT"));

await database.exec("CREATE TABLE IF NOT EXISTS users (id text PRIMARY KEY, username TEXT UNIQUNE NOT NULL, psdHash TEXT NOT NULL, icon TEXT)")

const log = (req: BunRequest) => {
    console.log(`[${new Date().toUTCString()}][${req.method}]: ${req.url}`)
}

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

        if (!payload.sub) throw new HttpError("Invalid token", { statusCode: 401 });

        const user = await database.prepare("SELECT * FROM users WHERE id = ?").as(User).get(payload.sub);

        if (!user) throw new Error("User is not found");

        return user;
    } catch (error) {
        throw new HttpError("Auth failed", { statusCode: 401, cause: error });
    }
}

const sign = (userId: string) => {
    return Promise.all([
        new SignJWT()
            .setProtectedHeader({ alg })
            .setIssuedAt()
            .setSubject(userId)
            .setIssuer("com:loadingzone:hermes")
            .setAudience("com:loadingzone:hermes")
            .setExpirationTime("2d")
            .sign(secret),
        new SignJWT()
            .setProtectedHeader({ alg })
            .setExpirationTime("7d")
            .setNotBefore("2d")
            .setSubject(userId)
            .setIssuer("com:loadingzone:hermes")
            .setAudience("com:loadingzone:hermes").sign(refreshSecret)
    ]);
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
        "/ws": async (req, server) => {
            log(req);
            const url = new URL(req.url);
            const token = url.searchParams.get("token");

            const usr = await auth(token);

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
                log(req);
                auth(req);

                const url = new URL(req.url);
                const channelId = url.searchParams.get("channelId");
                if (!channelId) throw new HttpError("Missing channel id", { statusCode: 404 });

                // validate channel id               


                return Response.json({ ok: true });
            },
            POST: async (req) => {
                log(req);
                auth(req);

                const uuid = crypto.randomUUID();

                const body = await req.json();
                // validate body

                return Response.json({});
            }
        },

        "/api/user/:uuid": {
            GET: async (req: BunRequest<"/api/user/:uuid">) => {
                log(req);
                const result = schemaUuid.safeParse(req.params.uuid);
                if (!result.success) {
                    throw new HttpError(result.error.message, { statusCode: 400 });
                }
                const uuid = result.data;

                const user = await auth(req);

                if (uuid === user.id) {
                    return Response.json(user.toJson());
                }

                const otherUser = await database.prepare("SELECT (id,username,icon) FROM users WHERE id = ?;").as(User).get(uuid);

                if (!otherUser) {
                    throw new HttpError("User not found", { statusCode: 404 });
                }

                return Response.json(otherUser.toJson());
            }
        },

        "/api/refresh": {
            POST: async (req: BunRequest) => {
                log(req);
                try {
                    const body = await req.json();
                    const result = schemaRefresh.safeParse(body);
                    if (!result.success) throw new HttpError(result.error.message, { statusCode: 400 });

                    const oldToken = await jwtVerify(result.data.refreshToken, refreshSecret, {
                        algorithms: [alg],
                        issuer: "com:loadingzone:hermes",
                        audience: "com:loadingzone:hermes",
                    });

                    if (!oldToken.payload.sub) throw new HttpError("Invalid token", { statusCode: 400 });

                    const data = await database.prepare("SELECT COUNT(*) as count FROM users WHERE id = ?;").get(oldToken.payload.sub) as { count: number };

                    if (data.count !== 1) throw new HttpError("Invalid token", { statusCode: 404 });

                    const [token, refreshToken] = await sign(oldToken.payload.sub);

                    return Response.json({
                        token,
                        refreshToken
                    });
                } catch (error) {
                    if (error instanceof HttpError) {
                        throw error;
                    } else if (Error.isError(error)) {
                        throw new HttpError(error?.message, { cause: error, statusCode: 401 });
                    }

                    throw new HttpError("Unknown Error", { statusCode: 500 });
                }
            }
        },

        "/api/login": {
            POST: async (req: BunRequest) => {
                log(req);
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

                const [token, refreshToken] = await sign(user.id);

                return Response.json({
                    token,
                    refreshToken,
                    user: user.toJson()
                });
            }
        },
        "/api/signup": {
            POST: async (req: BunRequest) => {
                log(req);
                const body = await req.json();

                const form = schemaSignup.safeParse(body);
                if (!form.success) {
                    throw new HttpError(form.error.message, { statusCode: 400 });
                }
                const { username, password: psd } = form.data;

                const psdHash = await password.hash(psd);

                const user = await database.prepare("INSERT INTO users (id,username,psdHash) VALUES (?,?,?) RETURNING id").as(User).get(crypto.randomUUID(), username, psdHash);

                if (!user) throw new HttpError("Failed to get user");

                const [token, refreshToken] = await sign(user.id);

                return Response.json({
                    token,
                    refreshToken,
                    user: user.toJson()
                });
            }
        },

        "/": home,

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