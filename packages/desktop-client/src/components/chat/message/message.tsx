import type { Message as tMessage } from "@/lib/api/types";
import { CollapsedMessage } from "./collapsed-mesasge";
import { ExpandedMessage } from "./expanded-message";

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
