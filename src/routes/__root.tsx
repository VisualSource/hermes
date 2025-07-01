import { createRootRouteWithContext, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { Toaster } from "@/components/ui/sonner";
import type { RTC } from "@/lib/rtc";
import type Auth from "@/lib/auth";
interface RouterContext {
    auth: Auth
    rtc: RTC,
}

export const Route = createRootRouteWithContext<RouterContext>()({
    async beforeLoad({ context }) {
        console.log("beforeload __root");
        await context.auth.init();
    },

    pendingComponent: () => {
        return (<div>
            <p>Loading...</p>
        </div>);
    },
    component: () => (
        <>
            <Outlet />
            <TanStackRouterDevtools />
            <Toaster />
        </>
    )
})