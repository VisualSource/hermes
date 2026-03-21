import {
	keepPreviousData,
	useInfiniteQuery,
	useMutation,
} from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";

import { faker } from "@faker-js/faker";

import { Button } from "@/components/ui/button";
import { Send } from "lucide-react";
import { Input } from "@/components/ui/input";

import { type Msg, VirtualList } from "@/components/virtual-list";

export const Route = createFileRoute("/_text/text/$roomId")({
	component: RouteComponent,
});

function RouteComponent() {
	const { roomId } = Route.useParams();

	const { data, isLoading, hasNextPage } = useInfiniteQuery({
		queryKey: ["text-channel", roomId],
		queryFn: ({ pageParam }) => {
			return (
				Array.from({ length: 200 }).map(() => ({
					userId: faker.string.uuid(),
					message: faker.lorem.text(),
					reacts: [],
					timestamp: faker.date.recent().toUTCString(),
				})) as Msg[]
			).concat([
				{
					userId: faker.string.uuid(),
					message: "https://youtube.com/shorts/FiMXgmhSlo0",
					reacts: [],
					timestamp: faker.date.recent().toISOString(),
				},
				{
					userId: faker.string.uuid(),
					message:
						"Check this out https://youtube.com/shorts/FiMXgmhSlo0 some text ",
					reacts: [],
					timestamp: faker.date.recent().toISOString(),
				},
			]);
		},
		maxPages: 3,
		getNextPageParam: () => undefined,
		getPreviousPageParam: () => undefined,
		initialPageParam: undefined,
		placeholderData: keepPreviousData,
		refetchOnWindowFocus: false,
	});

	const mutation = useMutation({
		mutationKey: ["text-channel", "msg-mut", roomId],
		mutationFn: async () => {
			await new Promise((ok) => setTimeout(ok, 5000));

			return {};
		},
	});

	const items = data?.pages.flat() ?? [];

	return (
		<main className="container px-8 h-full flex flex-col pb-6 col-span-10">
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
		</main>
	);
}
