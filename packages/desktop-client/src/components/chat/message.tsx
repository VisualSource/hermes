import { UserMarkdown } from "../markdown/user-markdown";
import { Avatar, AvatarFallback, AvatarImage } from "../ui/avatar";
import {
	ContextMenu,
	ContextMenuContent,
	ContextMenuItem,
	ContextMenuTrigger,
} from "../ui/context-menu";
import {
	HoverCard,
	HoverCardContent,
	HoverCardTrigger,
} from "../ui/hover-card";

export type Msg = {
	id: string;
	timestamp: string;
	userId: string;
	message: string;
	reacts: string[];
};
//absolute 	style={{ transform: `translateY(${start}px)` }}
export const Message = ({
	item,
	ref,
	index,
	start,
}: {
	item: Msg;
	start: number;
	index: number;
	ref?: React.Ref<HTMLDivElement>;
}) => {
	return (
		<ContextMenu>
			<ContextMenuTrigger
				render={
					<div
						ref={ref}
						data-index={index}
						className=" flex w-full hover:bg-accent/60 gap-2 px-2 py-1 cursor-pointer"
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
								<UserMarkdown content={item.message} />
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
