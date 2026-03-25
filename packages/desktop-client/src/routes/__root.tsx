import { createRootRoute, Outlet } from "@tanstack/react-router";
import { WindowHeader } from "@/components/window-header";
import { auth } from "@/lib/clients/auth";
import { Spinner } from "@/components/ui/spinner";
import { Button } from "@/components/ui/button";
import { SideBar } from "@/components/sidebar/side-bar";

import { Suspense, use } from "react";
import { app } from "@/lib/clients/app";
import { ChangeChannelAlertDialog } from "@/components/voice/change-channel-alert-dialog";
import { isTauri } from "@tauri-apps/api/core";

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

const isTauriContext = isTauri();

const RootLayout: React.FC = () => {
	return (
		<div className="h-full w-full overflow-hidden flex flex-col">
			{isTauriContext ? <WindowHeader /> : null}
			<ChangeChannelAlertDialog />
			<Suspense
				fallback={
					<div className="h-full w-full flex place-content-center place-items-center">
						<Spinner className="size-9" />
					</div>
				}
			>
				<AppState>
					<div className="h-full w-full overflow-hidden relative flex @container-[size]">
						<SideBar />
						<div className="w-full h-full flex flex-col col-span-7">
							<Outlet />
						</div>
					</div>
				</AppState>
			</Suspense>
		</div>
	);
};

export const Route = createRootRoute({
	component: RootLayout,
	errorComponent: (err) => {
		return (
			<div className="flex flex-col w-full">
				<WindowHeader />
				<main className="h-full w-full flex flex-col place-content-center place-items-center">
					<div>
						<h1>{err.error.message}</h1>
						<p> {err.info?.componentStack}</p>
					</div>
					<Button onClick={err.reset}>Reset</Button>
				</main>
			</div>
		);
	}
});
