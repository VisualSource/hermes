import { useAppContext } from "./useAppContext";

export const useRTC = () => {
    const ctx = useAppContext();
    return ctx.rtc;
}