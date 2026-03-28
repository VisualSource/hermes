import type { MessagesQuery } from "@/lib/api/types";
import { queryClient } from "@/lib/clients/queryClient";
import { type InfiniteData, useMutation } from "@tanstack/react-query";
import { useUser } from "../use-user";

export const useChannelTextMutation = (serverId: string, roomId: string) => {
	const user = useUser();
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

	return mutation;
};
