import {
	$create,
	$getState,
	$setState,
	type BaseStaticNodeConfig,
	createState,
	type EditorConfig,
	ElementNode,
	type LexicalEditor,
	type LexicalNode,
} from "lexical";

import { nameToEmoji } from "gemoji";
import type { Transformer } from "@lexical/markdown";

const idState = createState("id", {
	parse: (value) => (typeof value === "string" ? value : ""),
});
export class EmojiNode extends ElementNode {
	$config(): BaseStaticNodeConfig {
		return this.config("emoji", {
			extends: ElementNode,
			stateConfigs: [{ flat: true, stateConfig: idState }],
		});
	}

	createDOM(config: EditorConfig, editor?: LexicalEditor): HTMLElement {
		const el = $getState(this, idState);

		if (el.startsWith("hce_")) {
			const dom = document.createElement("img");
			dom.className = "inline h-[1.063rem] w-[1.063rem] select-text";
			dom.src = "https://cdn3.emoji.gg/emojis/254673-spray.gif";
			return dom;
		}

		const dom = document.createElement("span");
		dom.textContent = el;
		dom.setAttribute("role", "image");

		return dom;
	}
	updateDOM(prevNode: this, dom: HTMLElement, config: EditorConfig): boolean {
		return false;
	}
}

export function $createEmojiNode(id: string) {
	const emoji = id.startsWith("hce_") ? id : nameToEmoji[id];

	return $setState($create(EmojiNode), idState, emoji);
}

export function $isEmojiNode(
	node: LexicalNode | null | undefined,
): node is EmojiNode {
	return node instanceof EmojiNode;
}

export const EMOJI: Transformer = {
	dependencies: [],
	export: (node, exportChildren) => {
		if (!$isEmojiNode(node)) return null;

		const name = $getState(node, idState);

		return `:${name}:`;
	},
	regExp: /:(?<name>\w{2,}):/,
	replace(parentNode, children, match, isImport) {
		const node = $createEmojiNode(match[1]);
		parentNode.replace(node);
		if (!isImport) {
			node.select(0, 0);
		}
	},
	type: "element",
};
