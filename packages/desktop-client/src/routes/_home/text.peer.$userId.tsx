import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { VirtualList } from "@/components/chat/virtual-list";
import { faker } from "@faker-js/faker";
import {
	keepPreviousData,
	useInfiniteQuery,
	useMutation,
} from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Hash, Send } from "lucide-react";
import { Msg } from "@/components/chat/message";

export const Route = createFileRoute("/_home/text/peer/$userId")({
	component: RouteComponent,
});

function RouteComponent() {
	const { userId } = Route.useParams();

	const {
		data,
		isLoading,
		hasNextPage,
		hasPreviousPage,
		fetchNextPage,
		fetchPreviousPage,
	} = useInfiniteQuery({
		queryKey: ["peer-channel", userId],
		queryFn: ({ pageParam }) => {
			return Array.from({ length: 200 }).map(() => ({
				userId: faker.string.uuid(),
				message: faker.lorem.text(),
				reacts: [],
				id: faker.string.ulid(),
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

	const fetchNext = async (direction: "up" | "down") => {
		if (direction === "up") {
			await fetchPreviousPage();
		} else {
			await fetchNextPage();
		}
	};

	const mutation = useMutation({
		mutationKey: ["peer-channel", "msg-mut", userId],
		mutationFn: async () => {
			await new Promise((ok) => setTimeout(ok, 5000));

			return {};
		},
	});

	const items = data?.pages.flat() ?? [];

	return (
		<div className=" h-full flex flex-col pb-6 col-span-10">
			<div className="h-10 bg-accent flex items-center px-2 justify-between border-b shadow">
				<div className="flex gap-1 items-center">
					<Hash className="h-4 w-4" /> Username
				</div>
			</div>
			<div className="h-full overflow-hidden @container-[size] mb-2 container">
				<VirtualList
					hasNextPage={hasNextPage}
					hasPreviousPage={hasPreviousPage}
					fetchMore={fetchNext}
					items={items}
				/>
			</div>
			<form
				className="flex gap-1 px-8"
				action={async (data) => {
					const msg = data.get("msg");

					console.log(msg);

					await mutation.mutateAsync();
				}}
			>
				<Input
					placeholder="Send message to 'username'"
					type="text"
					name="msg"
				/>
				<Button type="submit" disabled={mutation.isPending}>
					<Send />
				</Button>
			</form>
		</div>
	);
}
