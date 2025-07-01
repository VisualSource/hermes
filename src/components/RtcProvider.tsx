import type { RTC } from "@/lib/rtc";
import { RtcContext } from "@/lib/rtcContext";

export const RtcProvider: React.FC<React.PropsWithChildren<{ client: RTC }>> = ({ client, children }) => {
    return (
        <RtcContext.Provider value={client}>
            {children}
        </RtcContext.Provider>
    );
}