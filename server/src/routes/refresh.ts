import { log } from "../lib/utils";
import { schemaRefresh } from "../schemas";
import { HttpError } from "../utils";
import { sign, verifyRefreshToken } from "../lib/auth";
import { database } from "../lib/database";

export default {
    POST: async (req: Bun.BunRequest) => {
        log(req);
        try {
            const body = await req.json();
            const result = schemaRefresh.safeParse(body);
            if (!result.success) throw new HttpError(result.error.message, { statusCode: 400 });

            const oldToken = await verifyRefreshToken(result.data.refreshToken);
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
}