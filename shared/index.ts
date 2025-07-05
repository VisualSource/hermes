import { z } from "zod/v4";

const sdp = z.strictObject({
    type: z.enum(["offer", "answer", "pranswer", "rollback"]),
    sdp: z.string().optional()
})

export const socketCommand = z.discriminatedUnion("type", [
    z.strictObject({
        type: z.literal("channel::room::join"),
        data: z.strictObject({
            channelId: z.uuidv4(),
            roomId: z.uuidv4()
        }),
    }),
    z.strictObject({
        type: z.literal("channel::room::leave"),
        data: z.strictObject({
            channelId: z.uuidv4(),
            roomId: z.uuidv4()
        })
    }),
    z.strictObject({
        type: z.literal("room::peer::offer"),
        data: z.strictObject({
            forUser: z.uuidv4(),
            roomId: z.uuidv4(),
            channelId: z.uuidv4(),
            sdp: sdp
        }),
    }),
    z.strictObject({
        type: z.literal("room::peer::answer"),
        data: z.strictObject({
            forUser: z.uuidv4(),
            roomId: z.uuidv4(),
            channelId: z.uuidv4(),
            sdp: sdp
        }),
    })
]);

export type SocketCommand = z.infer<typeof socketCommand>;
export type SocketCommandName = SocketCommand["type"];
export type SocketCommandMap = {
    [K in SocketCommand["type"]]: Extract<SocketCommand, { type: K }>["data"]
}

export const socketMessage = z.discriminatedUnion("type", [
    z.strictObject({
        type: z.literal("channel::room::user_join").describe("event name"),
        data: z.strictObject({
            channelId: z.uuidv4(),
            roomId: z.uuidv4(),
            user: z.uuidv4().describe("the user joining the room")
        })
    }),
    z.strictObject({
        type: z.literal("channel::room::user_leave"),
        data: z.strictObject({
            channelId: z.uuidv4(),
            roomId: z.uuidv4(),
            user: z.uuidv4()
        }),
    }),
    z.strictObject({
        type: z.literal("room::peer::offer"),
        data: z.strictObject({
            user: z.uuidv4(),
            sdp: sdp
        })
    }),
    z.strictObject({
        type: z.literal("room::peer::answer"),
        data: z.strictObject({
            user: z.uuidv4(),
            sdp: sdp
        })
    })
]);

export type SocketMessage = z.infer<typeof socketMessage>;
export type SocketMessageName = SocketMessage["type"];
export type SocketMessageMap = {
    [K in SocketMessage["type"]]: Extract<SocketMessage, { type: K }>["data"]
}