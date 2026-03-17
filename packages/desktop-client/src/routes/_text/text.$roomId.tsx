import { Textarea } from "@/components/ui/textarea";
import {
	keepPreviousData,
	useInfiniteQuery,
	useMutation,
} from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { useVirtualizer } from "@tanstack/react-virtual";
import Markdown from "react-markdown";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { faker } from "@faker-js/faker";
import {
	useActionState,
	useCallback,
	useEffect,
	useEffectEvent,
	useRef,
} from "react";
import { Button } from "@/components/ui/button";
import { Send } from "lucide-react";
import { Input } from "@/components/ui/input";
import {
	ContextMenu,
	ContextMenuContent,
	ContextMenuItem,
	ContextMenuTrigger,
} from "@/components/ui/context-menu";

export const Route = createFileRoute("/_text/text/$roomId")({
	component: RouteComponent,
});

type Msg = {
	timestamp: string;
	userId: string;
	message: string;
	reacts: string[];
};

const Message = ({
	item,
	ref,
	index,
	start,
}: {
	item: Msg;
	start: number;
	index: number;
	ref: React.Ref<HTMLButtonElement>;
}) => {
	return (
		<ContextMenu>
			<ContextMenuTrigger
				render={
					<button
						type="button"
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
								<h1 className="hover:underline">Username</h1>
								<div className="text-muted-foreground text-xs">
									{item.timestamp}
								</div>
							</div>
							<article className="text-sm text-left">
								<Markdown>{item.message}</Markdown>
							</article>
						</div>
					</button>
				}
			/>
			<ContextMenuContent>
				<ContextMenuItem>Edit</ContextMenuItem>
			</ContextMenuContent>
		</ContextMenu>
	);
}

const VirtualList = ({
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

function RouteComponent() {
	const { roomId } = Route.useParams();

	const { data, isLoading, hasNextPage } = useInfiniteQuery({
		queryKey: ["text-channel", roomId],
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
