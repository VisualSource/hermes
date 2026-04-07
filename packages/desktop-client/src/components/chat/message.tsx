import { UserMarkdown } from "../markdown/user-markdown";
import type { Message as tMessage } from "@/lib/api/types";
import { UserAvatar } from "../user/user-avatar";
import { useServerUser } from "@/hooks/use-server-user";
import { formatRelative } from "date-fns";
import { TooltipButton } from "../ui/tooltip-button";
import { useTranslation } from "react-i18next";
import { Edit2, SmilePlus } from "lucide-react";
import { Button } from "../ui/button";

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

const ButtonActions = () => {
	const { t } = useTranslation();
	return (
		<div className="absolute hidden group-hover:flex -top-4 right-1 border rounded-sm bg-accent">
			<TooltipButton size="icon-sm" variant="ghost" tooltip={t("AddReaction")}>
				<SmilePlus />
			</TooltipButton>
			<TooltipButton size="icon-sm" variant="ghost" tooltip={t("Edit")}>
				<Edit2 />
			</TooltipButton>
		</div>
	);
};

const ReactionList = ({ reacts }: { reacts: string[] }) => {
	return (
		<div className="flex flex-wrap mt-2 gap-1">
			{reacts.map((id) => (
				<button
					key={id}
					type="button"
					className="border border-blue-400 bg-blue-300/40 hover:bg-blue-300/50 rounded-sm px-2"
				>
					<span>1</span> 😁
				</button>
			))}
			<Button size="icon-sm" variant="outline">
				<SmilePlus />
			</Button>
		</div>
	);
};

const CollapsedMessage = ({ item }: { item: tMessage }) => {
	return (
		<div className="flex flex-col gap-2 relative group w-full">
			<ButtonActions />
			<div className="flex gap-2 w-full">
				<div className="w-10" />
				<article className="text-sm text-left">
					<UserMarkdown content={item.content} />
				</article>
			</div>

			{item.reacts.length > 0 ? <ReactionList reacts={item.reacts} /> : null}
		</div>
	);
};

const ExpandedMessage = ({ item }: { item: tMessage }) => {
	const { data } = useServerUser(item.userId);

	return (
		<div className="flex gap-2 relative group w-full">
			<ButtonActions />
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
				{item.reacts.length > 0 ? <ReactionList reacts={item.reacts} /> : null}
			</div>
		</div>
	);
};
