import type { Transformer } from "@lexical/markdown";
import {
	$create,
	$getState,
	$isElementNode,
	$isParagraphNode,
	$setState,
	createState,
	DecoratorNode,
	type EditorConfig,
	type LexicalEditor,
	type LexicalNode,
} from "lexical";
import type { UUID } from "node:crypto";
import { Mention } from "./mention-component";

const idState = createState("id", {
	parse: (value) => (typeof value === "string" ? (value as UUID) : null),
});

export class MentionNode extends DecoratorNode<React.ReactNode> {
	$config() {
		return this.config("mention", {
			extends: DecoratorNode,
			stateConfigs: [{ flat: true, stateConfig: idState }],
		});
	}

	createDOM(_config: EditorConfig, _editor: LexicalEditor): HTMLElement {
		return document.createElement("span");
	}

	updateDOM(
		_prevNode: unknown,
		_dom: HTMLElement,
		_config: EditorConfig,
	): boolean {
		return false;
	}

	get userId(): UUID | null {
		return $getState(this, idState);
	}

	decorate(_editor: LexicalEditor, _config: EditorConfig): React.ReactNode {
		return <Mention nodekey={this.getKey()} userId={this.userId} />;
	}
}

export function $createMentionNode(id: UUID): MentionNode {
	return $setState($create(MentionNode), idState, id);
}
export function $isMentionNode(
	node: LexicalNode | null | undefined,
): node is MentionNode {
	return node instanceof MentionNode;
}

export const MENTION: Transformer = {
	dependencies: [MentionNode],
	type: "element",
	regExp: /^@(?<userId>\w{3,})/,
	export(node, traverseChildren) {
		console.log("MENTION", node);

		if ($isParagraphNode(node)) {
			const children = node.getChildren();

			for (const child of children) {
				if ($isMentionNode(child)) {
					return `<@${child.userId}>`;
				}
			}

			return null;
		}

		if (!$isMentionNode(node)) return null;

		return `<@${node.userId}>`;
	},
	replace(parentNode, _children, match, isImport) {
		const node = $createMentionNode(match[0] as UUID);
		parentNode.replace(node);
		if (!isImport) {
			node.selectNext(0, 0);
		}
	},
};
