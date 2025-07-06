import type { BunRequest } from "bun";
import { HttpError } from "../utils";
import { jwtVerify, SignJWT } from "jose";
import { database } from "./database";
import { User } from "../model";

const alg = "HS256"

const secret = new TextEncoder().encode(btoa(process.env.JWT_SECRET as string));
const refreshSecret = new TextEncoder().encode(btoa(process.env.JWT_REFRESH_SECRET as string));
const issuer = "com:loadingzone:hermes";
const audience = "com:loadingzone:hermes";


export const auth = async (req: BunRequest | string | null) => {
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
            issuer,
            audience
        });

        if (!payload.sub) throw new HttpError("Invalid token", { statusCode: 401 });

        const user = await database.prepare("SELECT * FROM users WHERE id = ?").as(User).get(payload.sub);

        if (!user) throw new Error("User is not found");

        return user;
    } catch (error) {
        throw new HttpError("Auth failed", { statusCode: 401, cause: error });
    }
}

export const verifyRefreshToken = async (refreshToken: string) => {
    const oldToken = await jwtVerify(refreshToken, refreshSecret, {
        algorithms: [alg],
        issuer,
        audience
    });

    return oldToken;
}

export const sign = (userId: string) => {
    return Promise.all([
        new SignJWT()
            .setProtectedHeader({ alg })
            .setIssuedAt()
            .setSubject(userId)
            .setIssuer(issuer)
            .setAudience(audience)
            .setExpirationTime("2d")
            .sign(secret),
        new SignJWT()
            .setProtectedHeader({ alg })
            .setExpirationTime("7d")
            .setNotBefore("2d")
            .setSubject(userId)
            .setIssuer(issuer)
            .setAudience(audience).sign(refreshSecret)
    ]);
}