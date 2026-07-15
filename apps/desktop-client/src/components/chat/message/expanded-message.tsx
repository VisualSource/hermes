import { useServerUser } from "@/hooks/use-server-user";
import { ButtonActions } from "./message-actions";
import { UserAvatar } from "@/components/user/user-avatar";
import { formatRelative } from "date-fns/formatRelative";
import { UserMarkdown } from "@/components/markdown/user-markdown";
import { ReactionList } from "./reaction-list";
import type { Message } from "@/lib/api/types";

export const ExpandedMessage = ({ item }: { item: Message }) => {
	const { data } = useServerUser(item.userId);

	return (
		<div className="flex gap-2 relative group w-full">
			<ButtonActions userId={item.userId} />
			<UserAvatar size="lg" src={data?.avatar} />
			<div className="flex flex-col">
				<div className="flex gap-2 items-center align-middle">
					<h1 className="hover:underline font-medium">{data?.username}</h1>
					<div className="text-muted-foreground text-xs">
						{formatRelative(item.timestamp, new Date())}
					</div>
				</div>
				<article className="text-left">
					<UserMarkdown content={item.content} />
				</article>
				{item.reacts.length > 0 ? <ReactionList reacts={item.reacts} /> : null}
			</div>
		</div>
	);
};
