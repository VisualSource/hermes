import { createRootRoute, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { WindowHeader } from "@/components/window-header";
import { auth } from "@/lib/clients";
import { Spinner } from "@/components/ui/spinner";
import { Button } from "@/components/ui/button";

const RootLayout: React.FC = () => {
	return (
		<div>
			<WindowHeader/>
			<Outlet />
			<TanStackRouterDevtools />
			<ReactQueryDevtools />
		</div>
	);
};

export const Route = createRootRoute({ 
	component: RootLayout,
	errorComponent: (err) => {
		return (
			<div>
				<WindowHeader/>
				{err.error.message}
				{err.info?.componentStack}
				<Button onClick={err.reset}>Reset</Button>
			</div>
		);
	},
	pendingComponent: () => {
		return (
			<div className="h-full w-full flex flex-col">
				<WindowHeader/>
				<div className="h-full w-full flex items-center justify-center">
					<Spinner className="size-9"/>
				</div>
			</div>
		);
	},
	beforeLoad: async () => {
		await auth.init();
		if(!auth.isAuthed){
			await auth.authorize();
		}
	}
});
