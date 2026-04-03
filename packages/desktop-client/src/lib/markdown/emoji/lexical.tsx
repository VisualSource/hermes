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
						className="inline h-[1.063rem] w-[1.063rem] select-text"
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