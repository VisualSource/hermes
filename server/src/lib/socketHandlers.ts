import type { SocketCommand, SocketMessageMap } from "hermes-shared";
import type { User } from "../model";

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
    "channel::room::join": async (_app, ws, message) => {
        const userId = ws.data.user.id;
        // TODO: validate room and channel ids

        ws.subscribe(`${message.channelId}/${message.roomId}/peer/${userId}`);

        const payload = {
            user: userId,
            channelId: message.channelId,
            roomId: message.roomId
        }

        ws.publish(message.channelId, formatResponse("channel::room::user_join", payload));
    },
    "channel::room::leave": async (_app, ws, msg) => {
        ws.unsubscribe(`${msg.channelId}/${msg.roomId}/peer/${ws.data.user.id}`);

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