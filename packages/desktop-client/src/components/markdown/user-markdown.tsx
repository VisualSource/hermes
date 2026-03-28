import type { Components } from "react-markdown";
import {
	LinkPreview,
	rehypeLinkPreview,
	remarkLinkPreview,
} from "./link-preview";
import { sanitizeUrl } from "@braintree/sanitize-url";
import remarkGfm from "remark-gfm";
import Markdown from "react-markdown";
import { memo } from "react";
import { remarkEmoji } from "./emoji";
import { remarkNotify, UserNotify } from "./notify";
import type { UUID } from "node:crypto";

const markdownComponents: Components = {
	a: (props) => {
		return (
			<a
				{...props}
				rel="noopener noreferrer"
				target="_blank"
				className="text-primary hover:underline"
			/>
		);
	},
	span: (props) => {
		if ("data-user" in props) {
			return <UserNotify userId={props["data-user"] as UUID} />;
		}

		return <span {...props} />;
	},
	object: (props) => {
		if (props.type === "link-preview" && props.data?.length) {
			return <LinkPreview link={props.data} />;
		}

		return null;
	},
};

const rehypePlugins = [rehypeLinkPreview];
const markdownRemarkPlugins = [
	remarkEmoji,
	remarkNotify,
	remarkGfm,
	remarkLinkPreview,
];

export const UserMarkdown = memo(({ content }: { content: string }) => {
	return (
		<Markdown
			skipHtml
			urlTransform={sanitizeUrl}
			components={markdownComponents}
			remarkPlugins={markdownRemarkPlugins}
			rehypePlugins={rehypePlugins}
		>
			{content}
		</Markdown>
	);
});

UserMarkdown.displayName = "UserMarkdown";
