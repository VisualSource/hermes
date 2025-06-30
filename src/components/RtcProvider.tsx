import type { RTC } from "@/lib/rtc";
import { RtcContext } from "@/lib/rtcContext";
import { useEffect } from "react";

export const RtcProvider: React.FC<React.PropsWithChildren<{ client: RTC }>> = ({ client, children }) => {

    useEffect(() => {
        client.mount();
        return () => {
            client.unmount();
        }
    }, [client]);

    return (
        <RtcContext.Provider value={client}>
            {children}
        </RtcContext.Provider>
    );
}