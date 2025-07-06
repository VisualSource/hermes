import { z } from "zod/v4";
import { auth } from "../lib/auth"
import { database } from "../lib/database";
import { log } from "../lib/utils";
import { Channel, Room } from "../model";
import { schemaUuid } from "../schemas";
import { HttpError } from "../utils";

const creationSchema = z.discriminatedUnion("type",
    [
        z.strictObject({
            type: z.literal("create_room"),
            data: z.strictObject({
                name: z.string().min(3).max(255),
                type: z.enum(["media", "text"]),
            })
        })
    ]
)


export default {

    POST: async (req: Bun.BunRequest<"/api/channel/:uuid">, server: Bun.Server) => {
        log(req);
        // const user = await auth(req);
        const uuid = schemaUuid.parse(req.params.uuid);

        const body = await req.json();
        const { type, data } = creationSchema.parse(body);

        switch (type) {
            case "create_room": {
                const id = crypto.randomUUID();
                await database.prepare("INSERT INTO rooms (id,name,type,channelId) VALUES (?,?,?,?)").run(
                    id,
                    data.name,
                    data.type,
                    uuid
                );

                server.publish(uuid, JSON.stringify({
                    type: "channel::room::added",
                    data: {
                        name: data.name,
                        type: data.type,
                        id,
                    }
                }));

                return Response.json({}, { status: 201 });
            }

            default:
                break;
        }
    },

    GET: async (req: Bun.BunRequest<"/api/channel/:uuid">) => {
        log(req);
        //const user = await auth(req);

        const uuid = schemaUuid.parse(req.params.uuid);

        const result = database.prepare("SELECT * FROM channels WHERE id = ?").as(Channel).get(uuid);

        if (!result) throw new HttpError("Not found", { statusCode: 404 });

        /*if (!result.channelMembers.includes(user.id)) {
            throw new HttpError("Unauthorized", { statusCode: 401 });
        }*/

        const rooms = await database.prepare("SELECT * FROM rooms WHERE channelId = ?").as(Room).all(uuid);

        return Response.json({
            ...result.toJSON(),
            rooms: rooms
        });
    }


}