import { createRootRouteWithContext, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { Toaster } from "@/components/ui/sonner";
import { App } from "@/lib/app";
interface RouterContext {
    app: App
}

export const Route = createRootRouteWithContext<RouterContext>()({
    async beforeLoad({ context }) {
        await context.app.auth.init();
    },

    pendingComponent: () => {
        return (<div>
            <p>Loading...</p>
        </div>);
    },
    component: () => (
        <>
            <Outlet />
            <TanStackRouterDevtools position="bottom-right" />
            <Toaster />
        </>
    )
})