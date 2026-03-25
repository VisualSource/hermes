import {
	Avatar,
	AvatarBadge,
	AvatarFallback,
	AvatarImage,
} from "@/components/ui/avatar";
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from "@/components/ui/popover";
import { TooltipButton } from "@/components/ui/tooltip-button";
import { useServerId } from "@/hooks/use-server-id";
import { serverUsersOptions } from "@/lib/api/queries";
import { getStatusColor } from "@/lib/api/types";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute, Outlet } from "@tanstack/react-router";
import { Hash, Pin } from "lucide-react";

export const Route = createFileRoute("/_text")({
	component: RouteComponent,
});


const TextSidebar = () => {
	const serverId = useServerId();

	const { data } = useQuery(serverUsersOptions(serverId));

	return (
		<aside className="col-span-2 bg-sidebar p-2 overflow-hidden">
			<ul className="space-y-0.5 overflow-y-auto">
				<li className="border-b p-2">Member - {data?.length}</li>
				{data?.map((user) => (
					<li key={user.id}>
						<div className="w-full flex px-1.5 py-2 items-center gap-2 hover:shadow hover:bg-background/35">
							<Avatar>
								<AvatarFallback>UN</AvatarFallback>
								<AvatarImage src={user.avatar} />
								<AvatarBadge className={getStatusColor(user.status)} />
							</Avatar>

							<div>
								<h1>{user.username}</h1>
								{user.statusText?.length ? (
									<p className="text-xs text-muted-foreground">
										{user.statusText}
									</p>
								) : null}
							</div>
						</div>
					</li>
				))}
			</ul>
		</aside>
	);
}


function RouteComponent() {
	return (
		<div className="grid h-full w-full grid-cols-12">
			<div className="col-span-10 flex flex-col h-full w-full">
				<div className="h-10 bg-accent flex items-center px-2 justify-between border-b shadow">
					<div className="flex gap-1 items-center">
						<Hash className="h-4 w-4" /> Channel Name
					</div>
					<div>
						<Popover>
							<PopoverTrigger
								render={
									<TooltipButton tooltip="Pins" variant="ghost" size="icon-lg">
										<Pin />
									</TooltipButton>
								}
							/>
							<PopoverContent align="end">
								<ul>
									<li>Some pinned message</li>
								</ul>
							</PopoverContent>
						</Popover>
					</div>
				</div>
				<div className="h-full w-full flex">
					<Outlet />
				</div>
			</div>

			<TextSidebar />
		</div>
	);
}
