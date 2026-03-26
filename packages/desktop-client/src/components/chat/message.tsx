import { UserMarkdown } from "../markdown/user-markdown";
import type { Message as tMessage } from "@/lib/api/types";
import { UserAvatar } from "../user/user-avatar";


export const Message = ({
	item,
	ref,
	index,
	start,
	displayUser
}: {
	item: tMessage;
	start: number;
	index: number;
	ref?: React.Ref<HTMLDivElement>;
	displayUser: boolean;
}) => {
	return (
		<div
			ref={ref}
			data-index={index}
			style={{ transform: `translateY(${start}px)` }}
			className="absolute left-0 top-0 flex w-full hover:bg-accent/60 gap-2 px-2 py-1 cursor-pointer"
		>
			{ displayUser ? <UserAvatar size="lg" userId={item.userId} /> : <div className="w-10"/> }
			<div className="flex flex-col">
				{ displayUser ? <div className="flex gap-2 items-center align-middle">
					<h1 className="hover:underline">Username</h1>
					<div className="text-muted-foreground text-xs">{item.timestamp}</div>
				</div> : null }
				<article className="text-sm text-left">
					<UserMarkdown content={item.content} />
				</article>
			</div>
		</div>
	);
};
