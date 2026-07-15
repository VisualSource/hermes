import { findAndReplace } from "mdast-util-find-and-replace";

type Nodes = Parameters<typeof findAndReplace>[0];

const mention = /<@(?<id>[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12})>/g;

export const remarkMention = () => {
	return (tree: Nodes) => {
		findAndReplace(tree, [
			mention,
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
