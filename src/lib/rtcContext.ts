import { createContext } from "react";
import { RTC } from "./rtc";

export const RtcContext = createContext<RTC | null>(null);