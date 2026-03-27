import { useCallback, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { Message } from "./message";

import { useStickToBottom } from "use-stick-to-bottom";
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
	const containerRef = useRef<HTMLDivElement>(null);

	const count = items.length;

	const virtualizer = useVirtualizer({
		count,
		getItemKey: (index) => items[index].id,
		getScrollElement: () => containerRef.current,
		estimateSize: () => 52,
		overscan: 5,
	});

	const handleScroll = useCallback(() => {
		const indexes = virtualizer.getVirtualIndexes();
		if (indexes.length === 0) return;

		const firstVisable = indexes.at(0);
		const lastVisiable = indexes.at(-1);

		if (firstVisable === undefined || lastVisiable === undefined) return;

		if (firstVisable <= 5) {
			if (hasPreviousPage) fetchPreviousPage();
		}

		if (lastVisiable >= count - 5) {
			if (hasNextPage) fetchNextPage();
		}
	}, [
		count,
		fetchNextPage,
		fetchPreviousPage,
		hasNextPage,
		hasPreviousPage,
		virtualizer.getVirtualIndexes,
	]);

	/*const count = items.length;

	const virtualizer = useVirtualizer({
		count: hasNextPage ? count + 1 : count,
		getScrollElement: () => listRef.current,
		estimateSize: () => 52,
		measureElement:
			navigator.userAgent.indexOf("Firefox") === -1
				? (el) => el.getBoundingClientRect()?.height
				: undefined,
		overscan: 6,
	});

	const handleScroll = useCallback((ev: React.UIEvent<HTMLDivElement>) => {
		const el = ev.currentTarget;
		const distanceFromBottom =
			el.scrollHeight - (el.scrollTop + el.clientHeight);
		isAtBottom.current = distanceFromBottom < 8;
	}, []);

	const handleEventScroll = useEffectEvent(
		(index: number, behavior?: ScrollBehavior) => {
			virtualizer.scrollToIndex(index, { align: "end", behavior });
		},
	);
*/
	/*useEffect(() => {
		if (!listRef.current) return;
		//if (count === 0) return;

		if (!initScrollRef.current) {
			initScrollRef.current = true;
			listRef.current.scrollIntoView(false);
			//handleEventScroll(count - 1, "instant");
			return;
		}

		if (!isAtBottom.current) return;
		//handleEventScroll(count - 1, "smooth");
	}, []);*/

	return (
		<div
			ref={containerRef}
			className="h-[100cqh] w-[100-cqw] overflow-y-auto contain-strict overflow-anchor-none"
			//onScroll={handleScroll}
		>
			<div className="relative" style={{ height: virtualizer.getTotalSize() }}>
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
