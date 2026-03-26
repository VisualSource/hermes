import { LexicalComposer } from "@lexical/react/LexicalComposer";
import { RichTextPlugin } from "@lexical/react/LexicalRichTextPlugin";
import { ContentEditable } from "@lexical/react/LexicalContentEditable";
import { LexicalErrorBoundary } from "@lexical/react/LexicalErrorBoundary";
import { MarkdownShortcutPlugin } from "@lexical/react/LexicalMarkdownShortcutPlugin";
import { HistoryPlugin } from "@lexical/react/LexicalHistoryPlugin";
import {

	$convertToMarkdownString,
	TRANSFORMERS,
} from "@lexical/markdown";
import { Plus, Users2 } from "lucide-react";
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
import { useEffect, useEffectEvent } from "react";

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
	onError: (err: unknown) => console.error(err),
};

const EnterSubmit = ({
	mutateAsync,
}: {
	mutateAsync: (opts: { message: string }) => void;
}) => {
	const [editor] = useLexicalComposerContext();

	const update = useEffectEvent((value: string) => {
		mutateAsync({ message: value });
	});

	useEffect(() => {
		editor.registerCommand(
			KEY_ENTER_COMMAND,
			(ev) => {
				if (ev?.shiftKey) return false;
				ev?.preventDefault();
				const markdown = $convertToMarkdownString(TRANSFORMERS);

				update(markdown);

				const root = $getRoot();
				root.clear();

				return true;
			},
			COMMAND_PRIORITY_HIGH,
		);
	}, [editor]);

	return null;
};

export const TextInput = ({
	mutateAsync,
}: {
	mutateAsync: (opts: { message: string }) => void;
}) => {
	return (
		<div className="flex border min-h-16 max-h-32 bg-accent items-center gap-2 px-2">
			<Button variant="secondary" size="icon-lg">
				<Plus />
			</Button>
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
					<EnterSubmit mutateAsync={mutateAsync} />
					<MarkdownShortcutPlugin transformers={TRANSFORMERS} />
				</LexicalComposer>
			</div>
			<Button size="icon-lg" variant="secondary">
				<Users2 />
			</Button>
		</div>
	);
};
