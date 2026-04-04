import {
	type InitialConfigType,
	LexicalComposer,
} from "@lexical/react/LexicalComposer";
import { TabIndentationPlugin } from "@lexical/react/LexicalTabIndentationPlugin";
import { RichTextPlugin } from "@lexical/react/LexicalRichTextPlugin";
import { ContentEditable } from "@lexical/react/LexicalContentEditable";
import { LexicalErrorBoundary } from "@lexical/react/LexicalErrorBoundary";
import { MarkdownShortcutPlugin } from "@lexical/react/LexicalMarkdownShortcutPlugin";
import { HistoryPlugin } from "@lexical/react/LexicalHistoryPlugin";
import { EditorRefPlugin } from "@lexical/react/LexicalEditorRefPlugin";
import { Plus, Users2 } from "lucide-react";
import { Button } from "../ui/button";
import { HeadingNode, QuoteNode } from "@lexical/rich-text";
import {
	TextNode,
	ParagraphNode,
	mergeRegister,
	type LexicalEditor,
	$getSelection,
} from "lexical";
import { ListNode, ListItemNode } from "@lexical/list";
import { CodeNode } from "@lexical/code-core";
import { LinkNode } from "@lexical/link";
import { MentionNode } from "@/lib/markdown/mentions/lexical";
import { EnterSubmitPlugin } from "./enter-submit-plugin";
import { TRANSFORMERS } from "@lexical/markdown";
import {
	$createEmojiNode,
	EmojiNode,
	registerEmoji,
} from "@/lib/markdown/emoji/lexical";
import { MentionsPlugin } from "@/lib/markdown/mentions/mention-plugin";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { useCallback, useEffect, useRef } from "react";
import { Popover, PopoverContent, PopoverTrigger } from "../ui/popover";
import { gemoji } from "gemoji";
import { TooltipButton } from "../ui/tooltip-button";
import { EmojiPicker } from "./emoji-picker";
const cfg: InitialConfigType = {
	namespace: "textInput",
	theme: {
		code: "text-red-200",
		heading: {
			h1: "font-bold",
		},
	},
	nodes: [
		EmojiNode,
		MentionNode,
		TextNode,
		ParagraphNode,
		HeadingNode,
		QuoteNode,
		ListNode,
		ListItemNode,
		CodeNode,
		LinkNode,
	],
	onError: (err: unknown) => console.error(err),
};


const Plugins = () => {
	const [editor] = useLexicalComposerContext();

	useEffect(() => {
		const unsubscribe = mergeRegister(registerEmoji(editor));

		return () => {
			unsubscribe();
		};
	}, [editor]);

	return null;
};

export const TextInput = ({
	mutateAsync,
}: {
	mutateAsync: (opts: { message: string }) => void;
}) => {
	const editorRef = useRef<LexicalEditor>(null);

	const addEmoji = useCallback((id: string) => {
		editorRef.current?.update(() => {
			const node = $createEmojiNode(id);

			const selection = $getSelection();
			selection?.insertNodes([node]);
		});
	}, []);

	return (
		<div className="flex border min-h-16 max-h-32 bg-accent items-center gap-2 px-2">
			<Popover>
				<PopoverTrigger
					render={
						<Button variant="secondary" size="icon-lg">
							<Plus />
						</Button>
					}
				/>
				<PopoverContent align="end" className="w-80"></PopoverContent>
			</Popover>
			<div className="h-full w-full flex py-1">
				<LexicalComposer initialConfig={cfg}>
					<div className="h-full w-full relative border">
						<RichTextPlugin
							contentEditable={
								<ContentEditable
									className="h-full w-full px-1 ring-0 outline-0 overflow-y-scroll"
									aria-placeholder={"Enter some text..."}
									placeholder={
										<div className="absolute top-0 left-1 select-none pointer-events-none text-muted-foreground">
											Maybe send something?...
										</div>
									}
								/>
							}
							ErrorBoundary={LexicalErrorBoundary}
						/>
					</div>
					<HistoryPlugin />
					<EnterSubmitPlugin mutateAsync={mutateAsync} />
					<MarkdownShortcutPlugin transformers={TRANSFORMERS} />
					<MentionsPlugin />
					<TabIndentationPlugin />
					<EditorRefPlugin editorRef={editorRef} />
					<Plugins />
				</LexicalComposer>
			</div>
			<EmojiPicker addEmoji={addEmoji} />
		</div>
	);
};
	<button type="button" className="h-10 w-10 hover:bg-background/50"></button>;