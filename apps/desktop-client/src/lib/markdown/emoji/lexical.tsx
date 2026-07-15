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
	TextNode,
} from "lexical";

import { nameToEmoji } from "gemoji";

const idState = createState("id", {
	parse: (value) => (typeof value === "string" ? value : ""),
});
export class EmojiNode extends DecoratorNode<React.ReactNode> {
		$config(): BaseStaticNodeConfig {
			return this.config("emoji", {
				extends: DecoratorNode,
				stateConfigs: [{ flat: true, stateConfig: idState }],
			});
		}

		createDOM(_config: EditorConfig, _editor: LexicalEditor): HTMLElement {
			const el = document.createElement("span");
			if (this.isCustomEmoji) el.setAttribute("data-emoji", "true");

			return el;
		}

		updateDOM(
			_prevNode: unknown,
			_dom: HTMLElement,
			_config: EditorConfig,
		): boolean {
			return false;
		}

		override getTextContent(): string {
			if (this.isCustomEmoji) {
				return `:${this.emojiId}:`;
			}

			return this.emojiId;
		}

		get emojiId(): string {
			return $getState(this, idState);
		}

		get isCustomEmoji(): boolean {
			return this.emojiId.startsWith("hce_");
		}

		decorate(_editor: LexicalEditor, _config: EditorConfig): React.ReactNode {
			if (this.isCustomEmoji) {
				return (
					<img
						className="inline-block h-[1em] w-[1em]"
						src="https://cdn3.emoji.gg/emojis/254673-spray.gif"
						alt={`:${this.emojiId}:`}
					/>
				);
			}

			return <span>{this.emojiId}</span>;
		}
	}

export function $createEmojiNode(id: string) {
	const isCustom = id.startsWith("hce_");
	const emoji = isCustom ? id : nameToEmoji[id];

	return $setState($create(EmojiNode), idState, emoji);
}

export function $isEmojiNode(
		node: LexicalNode | null | undefined,
	): node is EmojiNode {
		return node instanceof EmojiNode;
	}

const emojiRegex = /:(?<name>\w{2,}):/;

function emojiTextTransform(node: TextNode): void {
	if (!node.isSimpleText() || node.hasFormat("code")) {
		return;
	}

	const text = node.getTextContent();
	const match = emojiRegex.exec(text);

	if (!match?.groups?.name) return;

	let target: TextNode;
	if (match.index === 0) {
		[target] = node.splitText(match.index + match[0].length);
	} else {
		[, target] = node.splitText(match.index, match.index + match[0].length);
	}

	const emoji = $createEmojiNode(match.groups.name);

	target.replace(emoji);
}

export function registerEmoji(editor: LexicalEditor) {
	return editor.registerNodeTransform(TextNode, emojiTextTransform);
}