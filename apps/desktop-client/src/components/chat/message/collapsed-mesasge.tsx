import { UserMarkdown } from "@/components/markdown/user-markdown";
import { ButtonActions } from "./message-actions";
import { ReactionList } from "./reaction-list";
import type { Message } from "@/lib/api/types";

export const CollapsedMessage = ({ item }: { item: Message }) => {
	return (
		<div className="flex gap-2 relative group w-full">
			<ButtonActions userId={item.userId} />
			<div className="w-10" />
			<div className="flex flex-col gap-2 w-full">
				<article className="text-left">
					<UserMarkdown content={item.content} />

					{item.reacts.length > 0 ? (
						<ReactionList reacts={item.reacts} />
					) : null}
				</article>
			</div>
		</div>
	);
};
