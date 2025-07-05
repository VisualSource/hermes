import { useCallback, useSyncExternalStore } from "react";
import { useRTC } from "./useRtc"

export const useRoom = (roomId: string) => {
    const ctx = useRTC();

    const listen = useCallback((callback: () => void) => {
        ctx.addEventListener(`room::${roomId}::update`, callback);
        return () => {
            ctx.removeEventListener(`room::${roomId}::update`, callback);
        }
    }, [roomId, ctx]);

    const room = useSyncExternalStore(listen, () => ctx.getRoom(roomId));

    return room;
}