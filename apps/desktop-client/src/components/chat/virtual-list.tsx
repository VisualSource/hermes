import { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { Message } from "./message";
import { useThrottledCallback } from "use-debounce";
import { useStickToBottom } from "use-stick-to-bottom";
import { useMergeRefs } from "react-merge-refs";
import type { MessagesQuery, Message as tMessage } from "@/lib/api/types";
import type {
	InfiniteData,
	InfiniteQueryObserverResult,
} from "@tanstack/react-query";

export const VirtualList = ({
	hasNextPage,
	hasPreviousPage,
	items,
	fetchNextPage,
	fetchPreviousPage,
}: {
	hasNextPage: boolean;
	hasPreviousPage: boolean;
	items: tMessage[];
	fetchPreviousPage: () => Promise<
		InfiniteQueryObserverResult<InfiniteData<MessagesQuery, unknown>, Error>
	>;
	fetchNextPage: () => Promise<
		InfiniteQueryObserverResult<InfiniteData<MessagesQuery, unknown>, Error>
	>;
}) => {
	const { scrollRef, contentRef, isNearBottom, state } = useStickToBottom();
	const containerRef = useRef<HTMLDivElement>(null);
	const rootRef = useMergeRefs<HTMLDivElement>([containerRef, scrollRef]);

	const count = items.length;

	const virtualizer = useVirtualizer({
		count,
		getItemKey: (index) => items[index].id,
		getScrollElement: () => containerRef.current,
		estimateSize: () => 52,
		overscan: 5,
	});

	const throttledScroll = useThrottledCallback(() => {
		if (isNearBottom && hasNextPage) {
			fetchNextPage();
		} else if (state.scrollTop < 300 && hasPreviousPage) {
			fetchPreviousPage();
		}
	}, 200);


	return (
		<div
			ref={rootRef}
			className="h-[100cqh] w-[100-cqw] overflow-y-auto contain-strict overflow-anchor-none"
			onScroll={throttledScroll}
		>
			<div
				ref={contentRef}
				className="relative"
				style={{ height: virtualizer.getTotalSize() }}
			>
				{virtualizer.getVirtualItems().map((virtualRow) => {
					const item = items[virtualRow.index];
					const sameUserAsPrevious =
						item.userId === items[virtualRow.index - 1]?.userId;

					return (
						<Message
							displayUser={sameUserAsPrevious}
							ref={virtualizer.measureElement}
							item={item}
							key={virtualRow.key}
							index={virtualRow.index}
							start={virtualRow.start}
						/>
					);
				})}
			</div>
		</div>
	);
};
