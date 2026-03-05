import { createRootRoute, Outlet } from "@tanstack/react-router";
import { WindowHeader } from "@/components/window-header";
import { auth } from "@/lib/clients";
import { Spinner } from "@/components/ui/spinner";
import { Button } from "@/components/ui/button";
import { SideBar } from "@/components/side-bar";
import { TanStackDevtools } from "@tanstack/react-devtools";
import { TanStackRouterDevtoolsPanel } from "@tanstack/react-router-devtools";
import { ReactQueryDevtoolsPanel } from "@tanstack/react-query-devtools";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { TooltipProvider } from "@/components/ui/tooltip";

import {
  useBlocker,
} from '@tanstack/react-router'

const RootLayout: React.FC = () => {
	useBlocker({
		shouldBlockFn: ({ current, next }) => {
			if(current.fullPath === "/voice/$roomId" && next.fullPath === "/voice/$roomId") {
				if(current.params.roomId !== next.params.roomId){
					return !confirm("This action with switch voice channel are you sure you want to do this?")
				}
				return false;
			}
			return false;
		},
		enableBeforeUnload: false,
    	withResolver: true,
	})

	return (
		<div className="h-full w-full overflow-hidden flex flex-col">
			<WindowHeader />
			<TooltipProvider>
				<div className="h-full w-full flex overflow-hidden relative">
					<SideBar />
					<Outlet />
				</div>
			</TooltipProvider>
		</div>
	);
};
//<TanStackDevtools plugins={[{ name: "Query", render: <ReactQueryDevtoolsPanel/> },{ name: "Router", render: <TanStackRouterDevtoolsPanel/> }]}/>
export const Route = createRootRoute({
	component: RootLayout,
	errorComponent: (err) => {
		return (
			<div className="flex flex-col">
				<WindowHeader />
				{err.error.message}
				{err.info?.componentStack}
				<Button onClick={err.reset}>Reset</Button>
			</div>
		);
	},
	pendingComponent: () => {
		return (
			<div className="h-full w-full flex flex-col">
				<WindowHeader />
				<div className="h-full w-full flex items-center justify-center">
					<Spinner className="size-9" />
				</div>
			</div>
		);
	},
	beforeLoad: async () => {
		//await auth.init();
		//if (!auth.isAuthed) {
		//	await auth.authorize();
		//}
	},
});
