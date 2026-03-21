import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Msg, VirtualList } from "@/components/chat/virtual-list";
import { faker } from "@faker-js/faker";
import {
	keepPreviousData,
	useInfiniteQuery,
	useMutation,
} from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Send } from "lucide-react";

export const Route = createFileRoute("/_home/text/peer/$userId")({
	component: RouteComponent,
});

function RouteComponent() {
	const { userId } = Route.useParams();

	const { data, isLoading, hasNextPage } = useInfiniteQuery({
		queryKey: ["peer-channel", userId],
		queryFn: ({ pageParam }) => {
			return Array.from({ length: 200 }).map(() => ({
				userId: faker.string.uuid(),
				message: faker.lorem.text(),
				reacts: [],
				timestamp: faker.date.recent().toUTCString(),
			})) as Msg[];
		},
		maxPages: 3,
		getNextPageParam: () => undefined,
		getPreviousPageParam: () => undefined,
		initialPageParam: undefined,
		placeholderData: keepPreviousData,
		refetchOnWindowFocus: false,
	});

	const mutation = useMutation({
		mutationKey: ["peer-channel", "msg-mut", userId],
		mutationFn: async () => {
			await new Promise((ok) => setTimeout(ok, 5000));

			return {};
		},
	});

	const items = data?.pages.flat() ?? [];

	return (
		<div className="container px-8 h-full flex flex-col pb-6 col-span-10">
			<div className="h-full overflow-hidden @container-[size] mb-2">
				<VirtualList hasNextPage={hasNextPage} items={items} />
			</div>
			<form
				className="flex gap-1"
				action={(data) => {
					mutation.mutateAsync();
				}}
			>
				<Input type="text" name="msg" />
				<Button type="submit" disabled={mutation.isPending}>
					<Send />
				</Button>
			</form>
		</div>
	);
}
