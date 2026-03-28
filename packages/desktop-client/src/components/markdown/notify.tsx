import { useServerUser } from "@/hooks/use-server-user";
import { findAndReplace } from "mdast-util-find-and-replace";
import type { UUID } from "node:crypto";

type Nodes = Parameters<typeof findAndReplace>[0];

const userAt = /<@(?<id>[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12})>/g;

export const remarkNotify = () => {
	return (tree: Nodes) => {
		findAndReplace(tree, [
			userAt,
			(match: string, id: string) => {
				return {
					type: "text",
					value: match,
					data: {
						hName: "span",
						hProperties: { "data-user": id },
						hChildren: [{ type: "text", value: id }],
					},
				};
			},
		]);

		return tree;
	};
};

export const UserNotify = ({ userId }: { userId: UUID }) => {
	const { data } = useServerUser(userId);
	return (
		<span className="hover:underline text-green-600">@{data?.username}</span>
	);
};
