import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { VirtualList } from "@/components/chat/virtual-list";
import { useInfiniteQuery, useMutation } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Hash, Send } from "lucide-react";
import { peerChannelOptions } from "@/lib/api/queries";
import { TextInput } from "@/components/chat/text-input";

export const Route = createFileRoute("/_home/text/peer/$userId")({
	component: RouteComponent,
});

function RouteComponent() {
	const { userId } = Route.useParams();

	const {
		data,
		hasNextPage,
		hasPreviousPage,
		fetchNextPage,
		fetchPreviousPage,
	} = useInfiniteQuery(peerChannelOptions(userId, new Date().toUTCString()));

	const mutation = useMutation({
		mutationFn: async (args: { message: string }) => {
			await new Promise((ok) => setTimeout(ok, 5000));

			return {};
		},
	});

	const items = data?.pages.flatMap((page) => page.results) ?? [];

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
					fetchNextPage={fetchNextPage}
					fetchPreviousPage={fetchPreviousPage}
					items={items}
				/>
			</div>
			<TextInput mutateAsync={mutation.mutateAsync} />
		</div>
	);
}
