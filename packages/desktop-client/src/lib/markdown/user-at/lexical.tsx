import type { Transformer } from "@lexical/markdown";
import {
	$applyNodeReplacement,
	DecoratorNode,
	type EditorConfig,
	type LexicalEditor,
	type LexicalNode,
	type NodeKey,
} from "lexical";
import { createBlockNode } from "../lexical-shared";

export class UserAtNode extends DecoratorNode<React.ReactNode> {
	__userId: string;

	static getType() {
		return "user-at";
	}
	static clone(node: UserAtNode): UserAtNode {
		return new UserAtNode(node.__userId, node.__key);
	}

	constructor(id: string = "", key?: NodeKey) {
		super(key);
		this.__userId = id;
	}

	createDOM(_config: EditorConfig, _editor: LexicalEditor): HTMLElement {
		const dom = document.createElement("span");
		dom.classList.add("text-green-500");
		return dom;
	}

	updateDOM(
		_prevNode: unknown,
		_dom: HTMLElement,
		_config: EditorConfig,
	): boolean {
		return false;
	}

	decorate(): React.ReactNode {
		return <div className="hover:underline"></div>;
	}
}

function $createUserAtNode(userId: string): UserAtNode {
	return $applyNodeReplacement(new UserAtNode(userId));
}

function $isUserAtNode(
	node: LexicalNode | null | undefined,
): node is UserAtNode {
	return node instanceof UserAtNode;
}

export const NOTIFY: Transformer = {
	dependencies: [UserAtNode],
	type: "element",
	regExp: /^@(?<userId>\w{3,})/,
	export(node, traverseChildren) {
		if (!$isUserAtNode(node)) return null;

		return `<@${node.__userId}>`;
	},
	replace: createBlockNode((match: RegExpMatchArray) => {
		const userId = match.groups?.userId;

		return $createUserAtNode(userId);
	}),
};
