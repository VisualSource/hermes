import { LexicalComposer } from "@lexical/react/LexicalComposer";
import { RichTextPlugin } from "@lexical/react/LexicalRichTextPlugin";
import { ContentEditable } from "@lexical/react/LexicalContentEditable";
import { LexicalErrorBoundary } from "@lexical/react/LexicalErrorBoundary";
import { MarkdownShortcutPlugin } from "@lexical/react/LexicalMarkdownShortcutPlugin";
import { HistoryPlugin } from "@lexical/react/LexicalHistoryPlugin";
import { useMutation } from "@tanstack/react-query";
import {
	$convertFromMarkdownString,
	$convertToMarkdownString,
	TRANSFORMERS,
} from "@lexical/markdown";
import { Plus, Send, Users2 } from "lucide-react";
import { Button } from "../ui/button";
import { HeadingNode, QuoteNode } from "@lexical/rich-text";
import {
	TextNode,
	ParagraphNode,
	KEY_ENTER_COMMAND,
	COMMAND_PRIORITY_HIGH,
	$getRoot,
} from "lexical";
import { ListNode, ListItemNode } from "@lexical/list";
import { CodeNode } from "@lexical/code-core";
import { LinkNode } from "@lexical/link";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { useEffect } from "react";

const cfg = {
	namespace: "textInput",
	theme: {
		heading: {
			h1: "font-bold",
		},
	},
	nodes: [
		TextNode,
		ParagraphNode,
		HeadingNode,
		QuoteNode,
		ListNode,
		ListItemNode,
		CodeNode,
		LinkNode,
	],
	onError: (err) => console.error(err),
	editorState: () => $convertFromMarkdownString("# D", TRANSFORMERS),
};

const EnterSubmit = () => {
	const [editor] = useLexicalComposerContext();

	useEffect(() => {
		editor.registerCommand(
			KEY_ENTER_COMMAND,
			(ev) => {
				if (ev?.shiftKey) return false;
				ev?.preventDefault();
				const markdown = $convertToMarkdownString(TRANSFORMERS);

				const root = $getRoot();
				root.clear();

				console.log(markdown);

				return true;
			},
			COMMAND_PRIORITY_HIGH,
		);
	}, [editor]);

	return null;
};

export const TextInput = () => {
	const mutation = useMutation({
		mutationFn: async () => {
			await new Promise((ok) => setTimeout(ok, 5000));

			return {};
		},
	});

	return (
		<div className="flex border h-16 bg-accent items-center gap-2 px-2">
			<Button variant="secondary" size="icon-lg">
				<Plus />
			</Button>
			<div className="h-full w-full flex py-1">
				<LexicalComposer initialConfig={cfg}>
					<div className="h-full w-full relative border">
						<RichTextPlugin
							contentEditable={
								<ContentEditable
									className="h-full w-full px-1 ring-0 outline-0"
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
					<EnterSubmit />
					<MarkdownShortcutPlugin transformers={TRANSFORMERS} />
				</LexicalComposer>
			</div>
			<Button size="icon-lg" variant="secondary">
				<Users2 />
			</Button>
		</div>
	);
};
