import { database } from "../lib/database";
import { schemaUuid } from "../schemas"

export default {
    GET: async (req: Bun.BunRequest<"/api/room/:uuid/messages">) => {
        const uuid = schemaUuid.parse(req.params.uuid);

        const messages = database.prepare("SELECT * FROM messages WHERE roomId = ?").all(uuid);

        return Response.json(messages);
    }
}