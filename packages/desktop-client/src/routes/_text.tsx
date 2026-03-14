import { createFileRoute, Outlet } from "@tanstack/react-router";

export const Route = createFileRoute("/_text")({
	component: RouteComponent,
});

function RouteComponent() {
	return (
		<div className="grid h-full w-full grid-cols-12">
			<div className="col-span-10">
				<Outlet />
			</div>

			<aside className="col-span-2 bg-sidebar"></aside>
		</div>
	);
}
