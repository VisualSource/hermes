import Markdown, { type Components } from "react-markdown";
import { Avatar, AvatarFallback, AvatarImage } from "./ui/avatar";
import {
	ContextMenu,
	ContextMenuContent,
	ContextMenuItem,
	ContextMenuTrigger,
} from "./ui/context-menu";
import { useCallback, useEffect, useEffectEvent, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "./ui/hover-card";
import remarkGfm from "remark-gfm";

import { sanitizeUrl } from "@braintree/sanitize-url";
import {
	LinkPreview,
	rehypeLinkPreview,
	remarkLinkPreview,
} from "./markdown/link-preview";


const markdownComponents: Components = {
	a: (props) => {
		return <a {...props} className="text-primary hover:underline" />;
	},
	object: (props) => {
		if (props.type === "link-preview" && props.data?.length) {
			return <LinkPreview link={props.data} />;
		}

		return null;
	},
};


export type Msg = {
	timestamp: string;
	userId: string;
	message: string;
	reacts: string[];
};
const rehypePlugins = [rehypeLinkPreview];
const markdownRemarkPlugins = [remarkGfm, remarkLinkPreview];
const Message = ({
	item,
	ref,
	index,
	start,
}: {
	item: Msg;
	start: number;
	index: number;
	ref: React.Ref<HTMLDivElement>;
}) => {
	return (
		<ContextMenu>
			<ContextMenuTrigger
				render={
					<div
						ref={ref}
						data-index={index}
						className="absolute flex w-full hover:bg-accent/60 gap-2 px-2 py-1 cursor-pointer"
						style={{ transform: `translateY(${start}px)` }}
					>
						<Avatar>
							<AvatarFallback>CN</AvatarFallback>
							<AvatarImage />
						</Avatar>
						<div className="flex flex-col">
							<div className="flex gap-2 items-center align-middle">
								<HoverCard>
									<HoverCardTrigger
										delay={10}
										closeDelay={100}
										render={<h1 className="hover:underline">Username</h1>}
									/>
									<HoverCardContent
										align="start"
										className="flex w-64 flex-col gap-0.5"
									>
										<div className="font-semibold">@nextjs</div>
										<div>
											The React Framework – created and maintained by @vercel.
										</div>
										<div className="mt-1 text-xs text-muted-foreground">
											Joined December 2021
										</div>
									</HoverCardContent>
								</HoverCard>
								<div className="text-muted-foreground text-xs">
									{item.timestamp}
								</div>
							</div>
							<article className="text-sm text-left">
								<Markdown
									skipHtml
									urlTransform={sanitizeUrl}
									components={markdownComponents}
									remarkPlugins={markdownRemarkPlugins}
									rehypePlugins={rehypePlugins}
								>
									{item.message}
								</Markdown>
							</article>
						</div>
					</div>
				}
			/>
			<ContextMenuContent>
				<ContextMenuItem>Edit</ContextMenuItem>
			</ContextMenuContent>
		</ContextMenu>
	);
};

export const VirtualList = ({
	hasNextPage,
	items,
}: {
	hasNextPage: boolean;
	items: Msg[];
}) => {
	const count = items.length;

	const initScrollRef = useRef(false);
	const isAtBottom = useRef(false);

	const listRef = useRef<HTMLDivElement>(null);

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

	useEffect(() => {
		if (!listRef.current) return;
		if (count === 0) return;

		if (!initScrollRef.current) {
			initScrollRef.current = true;
			handleEventScroll(count - 1, "instant");
			return;
		}

		if (!isAtBottom.current) return;
		handleEventScroll(count - 1, "smooth");
	}, [count]);

	return (
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
	);
};
