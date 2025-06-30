import { AuthContext } from "@/lib/authContext"
import { useContext } from "react"

export const useAuth = () => {
    const ctx = useContext(AuthContext);
    if (!ctx) throw new Error("useAuth needs to be in a AuthProvider");

    return ctx;
}