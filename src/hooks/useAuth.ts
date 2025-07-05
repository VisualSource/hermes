import { useAppContext } from "./useAppContext";

export const useAuth = () => {
    const ctx = useAppContext();
    return ctx.auth;
}