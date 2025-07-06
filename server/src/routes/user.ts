import { auth } from "../lib/auth";
import { database } from "../lib/database";
import { log } from "../lib/utils";
import { schemaUuid } from "../schemas";
import { HttpError } from "../utils";

export default {
    PATCH: async (req: Bun.BunRequest<"/api/user/:uuid">) => {
        log(req);
        const user = await auth(req);

        const result = schemaUuid.safeParse(req.params.uuid);
        if (!result.success) {
            throw new HttpError(result.error.message, { statusCode: 400 });
        }
        if (user.id !== result.data) throw new HttpError("Invalid request", { statusCode: 401 });
        const body = await req.json() as { icon: string };

        await database.prepare("UPDATE users SET icon = ? WHERE id = ?").run(body.icon, user.id);

        return Response.json({ status: "ok" });
    },
    GET: async (req: Bun.BunRequest<"/api/user/:uuid">) => {
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

        const otherUser = await database.prepare("SELECT id,username,icon FROM users WHERE id = ?;").get(uuid);

        if (!otherUser) {
            throw new HttpError("User not found", { statusCode: 404 });
        }

        return Response.json(otherUser);
    }
}