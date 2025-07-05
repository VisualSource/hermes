import { AppContext } from "@/lib/appContext"
import { useContext } from "react"

export const useAppContext = () => {
    const ctx = useContext(AppContext);
    if (!ctx) throw new Error("No context");

    return ctx;
}