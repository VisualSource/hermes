import { RtcContext } from "@/lib/rtcContext";
import { useContext } from "react"

export const useRTC = () => {
    const ctx = useContext(RtcContext);
    if (!ctx) throw new Error("use rtc context is not provider");
    return ctx;
}