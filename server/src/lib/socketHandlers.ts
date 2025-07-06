import type { SocketCommand, SocketMessageMap } from "hermes-shared";
import type { User } from "../model";
import { RoomManager } from "./roomManager";
import { database } from "./database";

type WS = Bun.ServerWebSocket<{ user: User; }>;
type Handler = {
    [K in SocketCommand["type"]]: (app: unknown, ws: WS, message: Extract<SocketCommand, { type: K }>["data"]) => Promise<void>
}

const formatResponse = <T extends keyof SocketMessageMap>(type: T, data: SocketMessageMap[T]) => {
    console.log(`[SERVER -> CLIENT][${type}]`);
    return JSON.stringify({
        type,
        data
    });
}

export const socketHandlers: Handler = {
    "channel::room::send_text": async (_app, ws, message) => {
        const id = crypto.randomUUID();
        await database.prepare("INSERT INTO messages VALUES (?,?,?,?)").run(
            id,
            message.message,
            ws.data.user.id,
            message.roomId
        );

        ws.publish(`${message.channelId}/${message.roomId}/text`, formatResponse("channel::room::text", {
            channelId: message.channelId,
            roomId: message.roomId,
            message: message.message,
            user: ws.data.user.id,
            id
        }));
    },
    "channel::room::join_text": async (_app, ws, message) => {
        ws.subscribe(`${message.channelId}/${message.roomId}/text`);
    },
    "channel::room::leave_text": async (_app, ws, message) => {
        ws.unsubscribe(`${message.channelId}/${message.roomId}/text`);
    },
    "channel::room::join": async (_app, ws, message) => {
        const userId = ws.data.user.id;
        // TODO: validate room and channel ids

        ws.subscribe(`${message.channelId}/${message.roomId}/peer/${userId}`);

        const manager = RoomManager.get();

        if (!manager.hasRoom(message.roomId)) {
            manager.addRoom(message.roomId);
        }

        manager.addUserToRoom(message.roomId, userId);

        const payload = {
            user: userId,
            channelId: message.channelId,
            roomId: message.roomId
        }

        ws.publish(message.channelId, formatResponse("channel::room::user_join", payload));
    },
    "channel::room::leave": async (_app, ws, msg) => {
        ws.unsubscribe(`${msg.channelId}/${msg.roomId}/peer/${ws.data.user.id}`);

        const manager = RoomManager.get();
        manager.removeUserFromRoom(msg.roomId, ws.data.user.id);

        ws.publish(msg.channelId, formatResponse("channel::room::user_leave", {
            channelId: msg.channelId,
            roomId: msg.roomId,
            user: ws.data.user.id
        }));
    },
    "room::peer::offer": async (_app, ws, msg) => {
        //TODO: validate channel and room ids

        const topic = `${msg.channelId}/${msg.roomId}/peer/${msg.forUser}`

        ws.publish(topic, formatResponse("room::peer::offer", {
            user: ws.data.user.id,
            sdp: msg.sdp
        }));
    },
    "room::peer::answer": async (_app, ws, message) => {
        const topic = `${message.channelId}/${message.roomId}/peer/${message.forUser}`;

        ws.publish(topic, formatResponse("room::peer::answer", {
            user: ws.data.user.id,
            sdp: message.sdp
        }));
    }
}