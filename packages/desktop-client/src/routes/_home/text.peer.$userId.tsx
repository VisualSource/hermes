
import { VirtualList } from "@/components/chat/virtual-list";
import { useInfiniteQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Hash } from "lucide-react";
import { peerChannelOptions } from "@/lib/api/queries";
import { TextInput } from "@/components/chat/text-input";
import { useMemo } from "react";
import { usePeerTextMutation } from "@/hooks/mutations/use-peer-text-mutation";

export const Route = createFileRoute("/_home/text/peer/$userId")({
	component: RouteComponent,
});

function RouteComponent() {
	const { userId } = Route.useParams();
	const start = useMemo(() => new Date().toUTCString(), []);
	const {
		data,
		hasNextPage,
		hasPreviousPage,
		fetchNextPage,
		fetchPreviousPage,
	} = useInfiniteQuery(peerChannelOptions(userId, start));
	const { mutateAsync } = usePeerTextMutation(userId);

	const items = data?.pages.flatMap((page) => page.results) ?? [];

	return (
		<div className=" h-full flex flex-col pb-2 col-span-10">
			<div className="h-10 bg-accent flex items-center px-2 justify-between border-b shadow">
				<div className="flex gap-1 items-center">
					<Hash className="h-4 w-4" /> Username
				</div>
			</div>
			<div className="flex flex-col h-full px-4">
				<div className="h-full overflow-hidden @container-[size] py-4">
					<VirtualList
						hasNextPage={hasNextPage}
						hasPreviousPage={hasPreviousPage}
						fetchNextPage={fetchNextPage}
						fetchPreviousPage={fetchPreviousPage}
						items={items}
					/>
				</div>
				<TextInput mutateAsync={mutateAsync} />
			</div>
		</div>
	);
}
