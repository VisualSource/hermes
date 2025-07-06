import { database } from "../lib/database";
import { RoomManager } from "../lib/roomManager";
import { log } from "../lib/utils";
import { schemaUuid } from "../schemas"
import { HttpError } from "../utils";

export default {
    GET: async (req: Bun.BunRequest<"/api/room/:uuid">) => {
        log(req);
        // const user = await auth(req);
        const uuid = schemaUuid.parse(req.params.uuid);
        const exists = database.prepare("SELECT EXISTS(SELECT id FROM rooms WHERE id = ? LIMIT 1)").get(uuid) as Record<string, number>;
        if (Object.entries(exists).at(0)?.at(1) === 0) {
            throw new HttpError("room does not exist", { statusCode: 404 });
        }

        const manager = RoomManager.get();

        if (!manager.hasRoom(uuid)) {
            manager.addRoom(uuid);
        }

        return Response.json({
            users: manager.getRoomUsers(uuid)
        })
    }
}