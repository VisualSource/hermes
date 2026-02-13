import { createRootRoute, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";

const RootLayout: React.FC = () => {
	return (
		<>
			<Outlet />
			<TanStackRouterDevtools />
			<ReactQueryDevtools />
		</>
	);
};

export const Route = createRootRoute({ component: RootLayout });
