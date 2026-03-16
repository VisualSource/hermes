import "./lib/logger";
import ReactDOM from "react-dom/client";
import { StrictMode } from "react";
import { QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider, createRouter } from "@tanstack/react-router";
import { routeTree } from "./routeTree.gen";
import "./index.css";
import { queryClient } from "./lib/clients/queryClient";
import { TooltipProvider } from "./components/ui/tooltip";

const router = createRouter({ 
	routeTree,
	context: {
		auth: undefined
	}
});

declare module "@tanstack/react-router" {
	interface Register {
		router: typeof router;
	}
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
	<StrictMode>
		<QueryClientProvider client={queryClient}>
			<TooltipProvider>
				<RouterProvider router={router} />
			</TooltipProvider>
		</QueryClientProvider>
	</StrictMode>,
);
