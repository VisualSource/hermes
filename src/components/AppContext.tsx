import type { App } from "@/lib/app";
import { AppContext } from "@/lib/appContext";

export const AppProvider: React.FC<React.PropsWithChildren<{ client: App }>> = ({ client, children }) => {
    return (
        <AppContext.Provider value={client}>
            {children}
        </AppContext.Provider>
    );
}