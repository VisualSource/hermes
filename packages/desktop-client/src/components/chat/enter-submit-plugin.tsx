import { FULL_TRANSFORMS } from "@/lib/markdown/lexical-transforms";
import { $convertToMarkdownString } from "@lexical/markdown";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { $getRoot, COMMAND_PRIORITY_HIGH, KEY_ENTER_COMMAND } from "lexical";
import { useEffect, useEffectEvent } from "react";

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
		editor.registerCommand(
			KEY_ENTER_COMMAND,
			(ev) => {
				if (ev?.shiftKey) return false;
				ev?.preventDefault();
				const markdown = $convertToMarkdownString(FULL_TRANSFORMS);

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
