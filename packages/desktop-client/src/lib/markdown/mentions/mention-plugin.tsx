import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { useServerUsers } from "@/hooks/user-server-users";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import {
	LexicalTypeaheadMenuPlugin,
	MenuOption,
	type MenuTextMatch,
} from "@lexical/react/LexicalTypeaheadMenuPlugin";
import { useQuery } from "@tanstack/react-query";
import { $isTextNode, COMMAND_PRIORITY_NORMAL, type TextNode } from "lexical";
import { User2 } from "lucide-react";
import type { UUID } from "node:crypto";
import { useCallback, useState } from "react";
import { createPortal } from "react-dom";
import { $createMentionNode } from "./lexical";

class Option extends MenuOption {
	constructor(
		key: string,
		public readonly avatar: string,
		public readonly username: string,
	) {
		super(key);
	}
}

const mentionRegex = /^@(?<username>\w{2,})/;

export const MentionsPlugin = () => {
	const [search, setSearch] = useState<string | null>(null);
	const [editor] = useLexicalComposerContext();
	const serverUsers = useServerUsers();
	const { data } = useQuery({
		enabled: !serverUsers.isLoading,
		initialData: [] as Option[],
		queryKey: ["mention-query", search],
		queryFn: ({ queryKey }) => {
			const key = queryKey[1] ?? "";

			return (
				serverUsers.data
					?.filter((item) => item.username.includes(key))
					.map((user) => new Option(user.id, user.avatar, user.username)) ?? []
			);
		},
	});

	const triggerFn = useCallback((text: string) => {
		console.log("trigger", text);
		const match = mentionRegex.exec(text);
		if (match !== null) {
			console.log("trigger match", match);
			return {
				leadOffset: match.index,
				matchingString: match.groups?.username ?? "",
				replaceableString: match[0],
			} as MenuTextMatch;
		}
		return null;
	}, []);

	const onSelection = useCallback(
		(
			option: Option,
			textNodeContainingQuery: TextNode | null,
			closeMenu: () => void,
			matchingString: string,
		) => {
			editor.update(() => {
				const node = $createMentionNode(option.key as UUID);

				if (textNodeContainingQuery) {
					const next = textNodeContainingQuery.getNextSibling();
					textNodeContainingQuery.replace(node);

					if ($isTextNode(next)) {
						const content = next.getTextContent();
						if (!/\s/.test(content) && matchingString.includes(content)) {
							next.remove();
						}
					}
				}
				closeMenu();
			});
		},
		[editor.update],
	);

	const onClose = useCallback(() => {}, []);

	return (
		<LexicalTypeaheadMenuPlugin<Option>
			commandPriority={COMMAND_PRIORITY_NORMAL}
			onQueryChange={setSearch}
			onSelectOption={onSelection}
			triggerFn={triggerFn}
			options={data}
			anchorClassName=""
			onClose={onClose}
			menuRenderFn={(
				anchorRef,
				{ selectedIndex, selectOptionAndCleanUp, setHighlightedIndex },
			) => {
				if (!anchorRef.current) return null;
				return createPortal(
					<ul className="z-10 bg-background w-52 px-2 py-2 border-2 border-accent">
						{data.map((item, i) => (
							<li
								ref={item.setRefElement}
								data-selected={selectedIndex === i}
								key={item.key}
								onClick={() => {
									setHighlightedIndex(i);
									selectOptionAndCleanUp(item);
								}}
								onKeyUp={() => {}}
								onKeyDown={() => {}}
								onMouseDown={(ev) => {
									ev.preventDefault();
								}}
								onMouseEnter={() => {
									setHighlightedIndex(i);
								}}
							>
								<div className="flex gap-2 cursor-pointer hover:bg-accent/30 py-1.5 px-1">
									<Avatar size="sm">
										<AvatarImage src={item.avatar} />
										<AvatarFallback>
											<User2 />
										</AvatarFallback>
									</Avatar>
									<span>{item.username}</span>
								</div>
							</li>
						))}
					</ul>,
					anchorRef.current,
				);
			}}
		></LexicalTypeaheadMenuPlugin>
	);
};
