import { createRootRouteWithContext, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import Auth from "../lib/auth";
import { Toaster } from "@/components/ui/sonner";

interface RouterContext {
    auth: Auth
}


export const Route = createRootRouteWithContext<RouterContext>()({
    component: () => (
        <>
            <Outlet />
            <TanStackRouterDevtools />
            <Toaster />
        </>
    )
})