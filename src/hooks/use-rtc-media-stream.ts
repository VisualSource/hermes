import { useCallback, useSyncExternalStore } from "react";
import { useRTC } from "./useRtc"

export const useRTCMediaStream = (userId: string) => {
    const ctx = useRTC();

    const callback = useCallback((cl: () => void) => {
        ctx.addEventListener(`media-stream::${userId}`, cl);
        return () => {
            ctx.removeEventListener(`media-stream::${userId}`, cl);
        }
    }, [userId, ctx]);

    const getSnapshot = useCallback(() => ctx.getMediaStream(userId)?.[0], [ctx, userId]);

    const stream = useSyncExternalStore(callback, getSnapshot);
    return stream;
}