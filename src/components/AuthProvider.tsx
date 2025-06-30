import Auth from "@/lib/auth";
import { AuthContext } from "@/lib/authContext";

export const AuthProvider: React.FC<React.PropsWithChildren<{ client: Auth }>> = ({ children, client }) => {
    return (
        <AuthContext.Provider value={client}>
            {children}
        </AuthContext.Provider>
    );
}