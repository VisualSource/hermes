import { createRootRoute, Outlet } from "@tanstack/react-router";
import { WindowHeader } from "@/components/window-header";
import { auth } from "@/lib/clients/auth";
import { Spinner } from "@/components/ui/spinner";
import { Button } from "@/components/ui/button";
import { SideBar } from "@/components/side-bar";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Suspense, use } from "react";
import { app } from "@/lib/clients/app";

const onInit = (async () => {
	await auth.init();
	if (!auth.isAuthed) {
		await auth.authorize();
	}
	await app.init();
})();


const AppState = ({ children }:React.PropsWithChildren) => {
	use(onInit);

	return (
		<>
			{children}
		</>
	)
}

const RootLayout: React.FC = () => {
	return (
		<div className="h-full w-full overflow-hidden flex flex-col">
			<WindowHeader />
			<Suspense
				fallback={
					<div className="h-full w-full flex place-items-center">
						<Spinner className="size-9" />
					</div>
				}
			>
				<AppState>
					<TooltipProvider>
						<div className="h-full w-full overflow-hidden relative flex @container-[size]">
							<SideBar />
							<div className="w-full h-full flex flex-col col-span-7">
								<Outlet />
							</div>
						</div>
					</TooltipProvider>
				</AppState>
			</Suspense>
		</div>
	);
};
//<TanStackDevtools plugins={[{ name: "Query", render: <ReactQueryDevtoolsPanel/> },{ name: "Router", render: <TanStackRouterDevtoolsPanel/> }]}/>
export const Route = createRootRoute({
	component: RootLayout,
	errorComponent: (err) => {
		return (
			<div className="flex flex-col w-full">
				<WindowHeader />
				<main className="h-full w-full flex flex-col place-items-center">
					<div>
						<h1>{err.error.message}</h1>
						<p>	{err.info?.componentStack}</p>
					</div>
					<Button onClick={err.reset}>Reset</Button>
				</main>
			</div>
		);
	}
});
