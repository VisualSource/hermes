import { nameToEmoji } from "gemoji";
import { findAndReplace } from "mdast-util-find-and-replace";

type Nodes = Parameters<typeof findAndReplace>[0];

const emojiRegex = /:(?<name>\w{2,}):/g;

export const remarkEmoji = () => {
	return (tree: Nodes) => {
		findAndReplace(tree, [
			emojiRegex,
			(match: string, name: string) => {
				if (name.startsWith("hce_")) {
					return {
						type: "text",
						value: match,
						data: {
							hName: "img",
							hProperties: {
								"data-role": "emoji",
								class:
									"inline-block h-[1.2em] w-[1.2em] select-text align-middle",
								src: `https://cdn3.emoji.gg/emojis/254673-spray.gif`,
								alt: match,
							},
							hChildren: [],
						},
					};
				}

				const emoji = nameToEmoji[name];

				return {
					type: "text",
					value: match,
					data: {
						hName: "span",
						hProperties: { role: "img" },
						hChildren: [{ type: "text", value: emoji }],
					},
				};
			},
		]);

		return tree;
	};
};
