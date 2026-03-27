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
import { useUser } from "@/hooks/use-user";

export const Route = createFileRoute("/_text/text/$roomId")({
	component: RouteComponent,
});

function RouteComponent() {
	const { roomId } = Route.useParams();
	const serverId = useServerId();
	const user = useUser();

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
		// https://github.com/TanStack/query/discussions/848
		mutationFn: async ({ message }: { message: string }) => {
			await new Promise((ok) => setTimeout(ok, 5000));
			const id = crypto.randomUUID();
			return id;
		},
		async onMutate(variables, context) {
			await queryClient.cancelQueries({
				queryKey: ["server-text", serverId, roomId],
			});

			const previousData = queryClient.getQueryData<
				InfiniteData<MessagesQuery>
			>(["server-text", serverId, roomId]);

			queryClient.setQueryData<InfiniteData<MessagesQuery>>(
				["server-text", serverId, roomId],
				(data) => {
					if (!data) return data;
					const lastPage = data.pages.at(-1);

					lastPage?.results.push({
						userId: user.id,
						content: variables.message,
						id: crypto.randomUUID(),
						timestamp: new Date().toUTCString(),
						reacts: [],
					});

					return { ...data };
				},
			);

			return { previousData };
		},
		onError(error, _variables, onMutateResult, _context) {
			console.error(error);
			queryClient.setQueryData(
				["server-text", serverId, roomId],
				onMutateResult?.previousData,
			);
		},
		onSettled() {
			queryClient.invalidateQueries({
				queryKey: ["server-text", serverId, roomId],
			});
		},
	});

	const items = data?.pages.flatMap((page) => page.results) ?? [];

	return (
		<main className="container px-8 h-full flex flex-col pb-6 col-span-10">
			<div className="h-full overflow-hidden @container-[size] py-4">
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
