import Markdown, { type ExtraProps } from "react-markdown";
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
import { useQuery } from "@tanstack/react-query";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";
import { fetch } from "@tauri-apps/plugin-http";
import { sanitizeUrl } from "@braintree/sanitize-url";

const isSupportedType = (value: string): value is DOMParserSupportedType => {
	return [
		"application/xhtml+xml",
		"application/xml",
		"text/html",
		"text/xml",
	].includes(value);
};

const LinkDisplay = (props: React.ComponentProps<"a"> & ExtraProps) => {
	const { data } = useQuery({
		queryKey: ["external-url", props.href],
		queryFn: async ({ signal }) => {
			try {
				if (!props.href?.length || !URL.canParse(props.href)) {
					console.debug(`Missing href or can not parse ${props.href}`);
					return null;
				}

				const url = new URL(props.href);
				if (url.protocol !== "https:") {
					console.debug("url protocol is not https", url);
					return null;
				}

				const response = await fetch(url, {
					signal,
					method: "GET",
					headers: {
						Accept: "text/xml,text/html,application/xml,application/xhtml+xml",
					},
				});
				if (!response.ok) throw response;
				const contentType =
					response.headers.get("content-type") ??
					response.headers.get("Content-Type") ??
					"";

				const [type] = contentType.split(";");

				console.debug("ContentType", type, contentType);
				if (!isSupportedType(type)) return null;

				const content = await response.text();

				const parser = new DOMParser();

				const doc = parser.parseFromString(content, type);

				const head = doc.querySelector("head");
				if (!head) return null;

				const metatags = head.querySelectorAll("meta");

				const cardInfo: {
					siteName: string;
					title: string;
					img?: string;
					description?: string;
					imgAlt?: string;
					shareUrl?: string;
				} = {
					siteName: url.hostname,
					title: "",
				};

				for (const metatag of metatags) {
					const property = metatag.getAttribute("property");
					switch (property) {
						case "og:title": {
							const value = metatag.getAttribute("content");
							if (value) cardInfo.title = value;
							break;
						}
						case "og:description": {
							const value = metatag.getAttribute("content");
							if (value) cardInfo.description = value;
							break;
						}
						case "og:site_name": {
							const value = metatag.getAttribute("content");
							if (value) cardInfo.siteName = value;
							break;
						}
						case "og:image": {
							const value = metatag.getAttribute("content");
							if (value) cardInfo.img = sanitizeUrl(value);
							break;
						}
						case "og:image:alt": {
							const value = metatag.getAttribute("content");
							if (value) cardInfo.imgAlt = value;
							break;
						}
						case "og:url": {
							const value = metatag.getAttribute("content");
							if (value) cardInfo.shareUrl = sanitizeUrl(value);
							break;
						}
					}
				}

				if (cardInfo.title === "") {
					cardInfo.title =
						head.querySelector("title")?.textContent ?? props.href;
				}

				return cardInfo;
			} catch (error) {
				console.error(Error.isError(error) ? error.message : error);
				return null;
			}
		},
		enabled: props.href !== undefined,
	});

	return (
		<>
			<a
				className="hover:underline text-primary"
				target="_blank"
				rel="noopener noreferrer"
				href={sanitizeUrl(props.href)}
			>
				{props.children}
			</a>
			{data ? (
				<Card className="mt-4">
					<CardHeader>
						<CardTitle>
							<span className="text-xs text-muted-foreground">
								{data.siteName}
							</span>
							<h1 className="text-lg">
								<a
									className="hover:underline text-primary"
									href={data.shareUrl ?? sanitizeUrl(props.href)}
									target="_blank"
									rel="noopener noreferrer"
								>
									{data?.title}
								</a>
							</h1>
							<p className="text-sm font-light">{data.description}</p>
						</CardTitle>
						<CardContent>
							<div className="aspect-square max-h-96">
								<img
									className="h-full w-full"
									src={data.img}
									alt={data.imgAlt ?? data.title}
								/>
							</div>
						</CardContent>
					</CardHeader>
				</Card>
			) : null}
		</>
	);
};

export type Msg = {
	timestamp: string;
	userId: string;
	message: string;
	reacts: string[];
};
const markdownRemarkPlugins = [remarkGfm];
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
									components={{
										a: LinkDisplay,
									}}
									remarkPlugins={markdownRemarkPlugins}
								>
									{item.message}
								</Markdown>
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
