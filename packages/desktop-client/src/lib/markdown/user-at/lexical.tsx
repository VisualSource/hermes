import type { Transformer } from "@lexical/markdown";
import {
	$create,
	$getState,
	$setState,
	type BaseStaticNodeConfig,
	createState,
	DecoratorNode,
	type EditorConfig,
	type LexicalEditor,
	type LexicalNode,
} from "lexical";
import { UserNotify } from "@/components/markdown/notify";
import type { UUID } from "node:crypto";

const idState = createState("id", {
	parse: (value) => (typeof value === "string" ? value : ""),
});
export class UserAtNode extends DecoratorNode<React.ReactNode> {
	$config(): BaseStaticNodeConfig {
		return this.config("user-at", {
			extends: DecoratorNode,
			stateConfigs: [{ flat: true, stateConfig: idState }],
		});
	}
	createDOM(_config: EditorConfig, _editor: LexicalEditor): HTMLElement {
		return document.createElement("div");
	}
	updateDOM(
		_prevNode: unknown,
		_dom: HTMLElement,
		_config: EditorConfig,
	): boolean {
		return false;
	}
	decorate(editor: LexicalEditor, config: EditorConfig): React.ReactNode {
		const userId = $getState(this, idState) as UUID;

		return <UserNotify userId={userId} />;
	}
}

function $createUserAtNode(userId: string): UserAtNode {
	return $setState($create(UserAtNode), idState, userId);
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
	export(node, _traverseChildren) {
		if (!$isUserAtNode(node)) return null;

		const id = $getState(node, idState);

		return `<@${id}>`;
	},
	replace(parentNode, _children, match, isImport) {
		const node = $createUserAtNode(match[0]);
		parentNode.replace(node);
		if (!isImport) {
			node.selectNext(0, 0);
		}
	},
};
