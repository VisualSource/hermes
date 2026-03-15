import { TooltipButton } from "@/components/ui/tooltip-button";
import { createFileRoute, Outlet } from "@tanstack/react-router";
import { Hash, Pin } from "lucide-react";

export const Route = createFileRoute("/_text")({
	component: RouteComponent,
});

function RouteComponent() {
	return (
		<div className="grid h-full w-full grid-cols-12">
			<div className="col-span-10 flex flex-col h-full w-full">
				<div className="h-10 bg-accent flex items-center px-2 justify-between border-b shadow">
					<div className="flex gap-1 items-center">
						<Hash className="h-4 w-4" /> Channel Name
					</div>
					<div>
						<TooltipButton tooltip="Pins" variant="ghost" size="icon-lg">
							<Pin />
						</TooltipButton>
					</div>
				</div>
				<div className="h-full w-full flex">
					<Outlet />
				</div>
			</div>

			<aside className="col-span-2 bg-sidebar"></aside>
		</div>
	);
}
