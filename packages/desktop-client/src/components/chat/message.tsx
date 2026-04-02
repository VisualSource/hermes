import { UserMarkdown } from "../markdown/user-markdown";
import type { Message as tMessage } from "@/lib/api/types";
import { UserAvatar } from "../user/user-avatar";
import { useServerUser } from "@/hooks/use-server-user";
import { formatRelative } from "date-fns";

type Props = {
	item: tMessage;
	start: number;
	index: number;
	ref?: React.Ref<HTMLDivElement>;
};

export const Message = ({
	item,
	ref,
	index,
	start,
	displayUser,
}: Props & {
	displayUser: boolean;
}) => {
	return (
		<div
			ref={ref}
			data-index={index}
			data-owner={item.userId}
			style={{ transform: `translateY(${start}px)` }}
			className="absolute left-0 top-0 flex w-full hover:bg-accent/60 gap-2 px-2 py-1 cursor-pointer"
		>
			{displayUser ? (
				<CollapsedMessage item={item} />
			) : (
				<ExpandedMessage item={item} />
			)}
		</div>
	);
};

const CollapsedMessage = ({ item }: { item: tMessage }) => {
	return (
		<div className="flex gap-2">
			<div className="w-10" />
			<article className="text-sm text-left">
				<UserMarkdown content={item.content} />
			</article>
		</div>
	);
};

const ExpandedMessage = ({ item }: { item: tMessage }) => {
	const { data } = useServerUser(item.userId);

	return (
		<div className="flex gap-2">
			<UserAvatar size="lg" src={data?.avatar} />
			<div className="flex flex-col">
				<div className="flex gap-2 items-center align-middle">
					<h1 className="hover:underline">{data?.username}</h1>
					<div className="text-muted-foreground text-xs">
						{formatRelative(item.timestamp, new Date())}
					</div>
				</div>
				<article className="text-sm text-left">
					<UserMarkdown content={item.content} />
				</article>
			</div>
		</div>
	);
};
