import { useCallback, useEffect, useEffectEvent, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { Message, type Msg } from "./message";
import { ScrollArea } from "../ui/scroll-area";
import { useStickToBottom } from "use-stick-to-bottom";

export const VirtualList = ({
	hasNextPage,
	items,
}: {
	hasNextPage: boolean;
	items: Msg[];
}) => {
	const { scrollRef, contentRef } = useStickToBottom();
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
		<ScrollArea className="h-[100cqh]" viewportRef={scrollRef}>
			<div ref={contentRef} className="flex flex-1 flex-col h-full px-8">
				{items.map((item, i) => (
					<Message key={item.id} item={item} index={i} start={0} />
				))}
			</div>
		</ScrollArea>
	);

	/*return (
		<div
			ref={listRef}
			className="h-[100cqh] w-[100-cqw] overflow-y-auto contain-strict"
			style={{ overflowAnchor: "none" }}
			onScroll={handleScroll}
		>
			<div className="relative" style={{ height: virtualizer.getTotalSize() }}>
				{virtualizer.getVirtualItems().map((virtualRow) => {
					const item = items[virtualRow.index];
					return (
						<Message
							key={virtualRow.key}
							ref={virtualizer.measureElement}
							index={virtualRow.index}
							start={virtualRow.start}
							item={item}
						/>
					);
				})}
			</div>
		</div>
	);*/
};
