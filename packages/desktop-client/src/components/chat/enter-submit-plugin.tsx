
import { MENTION } from "@/lib/markdown/mentions/lexical";
import { $convertToMarkdownString, TRANSFORMERS } from "@lexical/markdown";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { $getRoot, COMMAND_PRIORITY_HIGH, KEY_ENTER_COMMAND } from "lexical";
import { useEffect, useEffectEvent } from "react";

const transforms = [...TRANSFORMERS, MENTION];

export const EnterSubmitPlugin = ({
	mutateAsync,
}: {
	mutateAsync: (opts: { message: string }) => void;
}) => {
	const [editor] = useLexicalComposerContext();

	const update = useEffectEvent((value: string) => {
		mutateAsync({ message: value });
	});

	useEffect(() => {
		const unregister = editor.registerCommand(
			KEY_ENTER_COMMAND,
			(ev) => {
				if (ev?.shiftKey) return false;
				ev?.preventDefault();
				const markdown = $convertToMarkdownString(transforms);

				console.log(markdown);

				update(markdown);

				const root = $getRoot();
				root.clear();

				return true;
			},
			COMMAND_PRIORITY_HIGH,
		);

		return () => {
			unregister();
		};
	}, [editor]);

	return null;
};
