import { useInfiniteQuery, useMutation } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";

import { Button } from "@/components/ui/button";
import { Send } from "lucide-react";
import { Input } from "@/components/ui/input";

import { VirtualList } from "@/components/chat/virtual-list";
import { serverTextChannelOptions } from "@/lib/api/queries";
import type { UUID } from "node:crypto";
import { useServerId } from "@/hooks/use-server-id";
import { TextInput } from "@/components/chat/text-input";

export const Route = createFileRoute("/_text/text/$roomId")({
	component: RouteComponent,
});

function RouteComponent() {
	const { roomId } = Route.useParams();
	const serverId = useServerId();

	const {
		data,
		hasNextPage,
		hasPreviousPage,
		fetchPreviousPage,
		fetchNextPage,
	} = useInfiniteQuery(
		serverTextChannelOptions(
			serverId,
			roomId as UUID,
			new Date().toUTCString(),
		),
	);

	const items = data?.pages.flat() ?? [];

	return (
		<main className="container px-8 h-full flex flex-col pb-6 col-span-10">
			<div className="h-full overflow-hidden @container-[size] mb-2">
				<VirtualList
					fetchNextPage={fetchNextPage}
					fetchPreviousPage={fetchPreviousPage}
					hasPreviousPage={hasPreviousPage}
					hasNextPage={hasNextPage}
					items={items}
				/>
			</div>
			<TextInput />
		</main>
	);
}
