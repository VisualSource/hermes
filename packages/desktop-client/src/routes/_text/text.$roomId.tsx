import {
	type InfiniteData,
	useInfiniteQuery,
	useMutation,
} from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";

import { VirtualList } from "@/components/chat/virtual-list";
import { serverTextChannelOptions } from "@/lib/api/queries";
import type { UUID } from "node:crypto";
import { useServerId } from "@/hooks/use-server-id";
import { TextInput } from "@/components/chat/text-input";
import { queryClient } from "@/lib/clients/queryClient";
import type { MessagesQuery } from "@/lib/api/types";

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

	const mutation = useMutation({
		mutationFn: async ({ message }: { message: string }) => {
			await new Promise((ok) => setTimeout(ok, 5000));
			const id = crypto.randomUUID();
			return id;
		},
		onSuccess(result, variables) {
			queryClient.setQueryData<InfiniteData<MessagesQuery>>(
				["server-text", serverId, roomId],
				(data) => {
					if (!data) return data;
					const lastPage = data.pages.at(-1);

					lastPage?.results.push({
						userId: crypto.randomUUID(),
						content: variables.message,
						id: result,
						timestamp: new Date().toUTCString(),
						reacts: [],
					});

					return { ...data };
				},
			);
		},
	});

	const items = data?.pages.flatMap((page) => page.results) ?? [];

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
			<TextInput mutateAsync={mutation.mutateAsync} />
		</main>
	);
}
