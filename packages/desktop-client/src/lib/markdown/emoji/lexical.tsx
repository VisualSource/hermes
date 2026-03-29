import { $createTextNode } from "lexical";
import { createBlockNode } from "../lexical-shared";
import { nameToEmoji } from "gemoji";
import type { Transformer } from "@lexical/markdown";

export const EMOJI: Transformer = {
	dependencies: [],
	export: (node, exportChildren) => {
		return ":cat:";
	},
	regExp: /^:(?<name>\w{2,}):/,
	replace: createBlockNode((match: RegExpMatchArray) => {
		const name = match.groups?.name;
		return $createTextNode(nameToEmoji[name]);
	}),
	type: "element",
};
