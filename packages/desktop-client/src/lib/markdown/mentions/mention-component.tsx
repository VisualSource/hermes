import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { useLexicalNodeSelection } from "@lexical/react/useLexicalNodeSelection";
import {
	$getNodeByKey,
	$getSelection,
	$isDecoratorNode,
	$isElementNode,
	$isNodeSelection,
	$isTextNode,
	$setSelection,
	BLUR_COMMAND,
	CLICK_COMMAND,
	COMMAND_PRIORITY_LOW,
	KEY_ARROW_LEFT_COMMAND,
	KEY_ARROW_RIGHT_COMMAND,
	KEY_BACKSPACE_COMMAND,
	KEY_DELETE_COMMAND,
	mergeRegister,
	type NodeKey,
} from "lexical";
import { useEffect, useEffectEvent, useRef } from "react";
import { $isMentionNode } from "./lexical";
import { useServerUser } from "@/hooks/use-server-user";
import type { UUID } from "node:crypto";

export const Mention = ({
	nodekey,
	userId,
}: {
	nodekey: NodeKey;
	userId: UUID | null;
}) => {
	const { data } = useServerUser(userId);
	const ref = useRef<HTMLSpanElement>(null);

	const [editor] = useLexicalComposerContext();
	const [isSelected, setSelected, clearSelection] =
		useLexicalNodeSelection(nodekey);

	const onDelete = useEffectEvent((ev: KeyboardEvent) => {
		if (!(isSelected && $isNodeSelection($getSelection()))) return false;
		ev.preventDefault();
		const node = $getNodeByKey(nodekey);
		if (!$isMentionNode(node)) return false;
		node.remove();
		return false;
	});

	const onArrowLeftPress = useEffectEvent((ev: KeyboardEvent) => {
		const node = $getNodeByKey(nodekey);
		if (!node?.isSelected()) return false;

		const previous = node.getPreviousSibling();
		if ($isElementNode(previous)) {
			previous.selectEnd();
		} else if ($isTextNode(previous)) {
			previous.select();
		} else if ($isDecoratorNode(previous)) {
			previous.selectNext();
		} else if (previous === null) {
			node.selectPrevious();
		} else {
			return false;
		}

		ev.preventDefault();

		return true;
	});

	const onArrowRightPress = useEffectEvent((ev: KeyboardEvent) => {
		const node = $getNodeByKey(nodekey);
		if (!node?.isSelected()) return false;

		const next = node.getNextSibling();
		if ($isElementNode(next)) {
			next.selectStart();
		} else if ($isTextNode(next)) {
			next.select(0, 0);
		} else if ($isDecoratorNode(next)) {
			next.selectPrevious();
		} else if (next === null) {
			node.selectNext();
		} else {
			return false;
		}

		ev.preventDefault();
		return true;
	});

	const onClick = useEffectEvent((ev: MouseEvent) => {
		if (
			!(
				ev.currentTarget === ref.current ||
				(ev.target && ref.current?.contains(ev.target as HTMLElement))
			)
		) {
			return false;
		}
		if (!ev.shiftKey) clearSelection();
		setSelected(true);

		return true;
	});

	const onBlur = useEffectEvent(() => {
		const node = $getNodeByKey(nodekey);
		if (!node?.isSelected()) return false;

		const selection = $getSelection();
		if (!$isNodeSelection(selection)) return false;

		$setSelection(null);
		return false;
	});

	useEffect(() => {
		const unregister = mergeRegister(
			editor.registerCommand(CLICK_COMMAND, onClick, COMMAND_PRIORITY_LOW),
			editor.registerCommand(
				KEY_DELETE_COMMAND,
				onDelete,
				COMMAND_PRIORITY_LOW,
			),
			editor.registerCommand(
				KEY_BACKSPACE_COMMAND,
				onDelete,
				COMMAND_PRIORITY_LOW,
			),
			editor.registerCommand(
				KEY_ARROW_LEFT_COMMAND,
				onArrowLeftPress,
				COMMAND_PRIORITY_LOW,
			),
			editor.registerCommand(
				KEY_ARROW_RIGHT_COMMAND,
				onArrowRightPress,
				COMMAND_PRIORITY_LOW,
			),
			editor.registerCommand(BLUR_COMMAND, onBlur, COMMAND_PRIORITY_LOW),
		);

		return () => {
			unregister();
		};
	}, [editor.registerCommand]);

	return (
		<span ref={ref} className="bg-amber-600">
			@{data?.username ?? "Unknown"}
		</span>
	);
};
