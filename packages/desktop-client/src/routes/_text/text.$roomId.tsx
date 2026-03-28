import { useInfiniteQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";

import { VirtualList } from "@/components/chat/virtual-list";
import { serverTextChannelOptions } from "@/lib/api/queries";
import type { UUID } from "node:crypto";
import { useServerId } from "@/hooks/use-server-id";
import { TextInput } from "@/components/chat/text-input";

import { useMemo } from "react";
import { useChannelTextMutation } from "@/hooks/mutations/use-channel-text-mutation";

export const Route = createFileRoute("/_text/text/$roomId")({
	component: RouteComponent,
});

function RouteComponent() {
	const { roomId } = Route.useParams();
	const serverId = useServerId();
	const { mutateAsync } = useChannelTextMutation(serverId, roomId);
	const startPoint = useMemo(() => new Date().toUTCString(), []);
	const {
		data,
		hasNextPage,
		hasPreviousPage,
		fetchPreviousPage,
		fetchNextPage,
	} = useInfiniteQuery(
		serverTextChannelOptions(serverId, roomId as UUID, startPoint),
	);

	const items = data?.pages.flatMap((page) => page.results) ?? [];

	return (
		<main className="container px-8 h-full flex flex-col pb-2 col-span-10">
			<div className="h-full overflow-hidden @container-[size] py-4">
				<VirtualList
					fetchNextPage={fetchNextPage}
					fetchPreviousPage={fetchPreviousPage}
					hasPreviousPage={hasPreviousPage}
					hasNextPage={hasNextPage}
					items={items}
				/>
			</div>
			<TextInput mutateAsync={mutateAsync} />
		</main>
	);
}
