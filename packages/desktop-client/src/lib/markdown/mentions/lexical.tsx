import {
	$create,
	$getState,
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

		override getTextContent(): string {
			return `<@${this.userId}>`;
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