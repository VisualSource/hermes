import type { MessagesQuery } from "@/lib/api/types";
import { queryClient } from "@/lib/clients/queryClient";
import { type InfiniteData, useMutation } from "@tanstack/react-query";
import { useUser } from "../use-user";

export const usePeerTextMutation = (userId: string) => {
	const user = useUser();
	const mutation = useMutation({
		mutationFn: async (args: { message: string }) => {
			await new Promise((ok) => setTimeout(ok, 5000));
			const id = crypto.randomUUID();
			return id;
		},
		async onMutate(variables) {
			await queryClient.cancelQueries({
				queryKey: ["peer-text", userId],
			});

			const previousData = queryClient.getQueryData<
				InfiniteData<MessagesQuery>
			>(["peer-text", userId]);

			queryClient.setQueryData<InfiniteData<MessagesQuery>>(
				["peer-text", userId],
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
				["peer-text", userId],
				onMutateResult?.previousData,
			);
		},
		onSettled() {
			queryClient.invalidateQueries({
				queryKey: ["peer-text", userId],
			});
		},
	});

	return mutation;
};
