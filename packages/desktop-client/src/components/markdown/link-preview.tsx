import { useQuery } from "@tanstack/react-query";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { fetch } from "@tauri-apps/plugin-http";
import { sanitizeUrl } from "@braintree/sanitize-url";
import { type Node, select } from "unist-util-select";
import { visit } from "unist-util-visit";
import { isTauri } from "@tauri-apps/api/core";

// https://www.ryanfiller.com/blog/remark-and-rehype-plugins

export const rehypeLinkPreview = () => {
	return (tree: Node) => {
		visit(
			tree,
			"element",
			(
				node: Node & { tagName: string; properties: Record<string, string> },
			) => {
				if (
					node.tagName === "div" &&
					node.properties?.type === "link-preview"
				) {
					node.tagName = "object";
				}
			},
		);
		return tree;
	};
};

export const remarkLinkPreview = () => {
	return (tree: Node) => {
		const linkNode = select("link", tree);
		if (linkNode) {
			visit(tree, "root", (node: Node & { children: Node[] }) => {
				node.children.push({
					type: "linkPreview",
					data: {
						hProperties: {
							type: "link-preview",
							data: "url" in linkNode ? linkNode.url : undefined,
						},
					},
				});
			});
		}

		return tree;
	};
};

const isSupportedType = (value: string): value is DOMParserSupportedType => {
	return [
		"application/xhtml+xml",
		"application/xml",
		"text/html",
		"text/xml",
	].includes(value);
};

export const LinkPreview = ({ link }: { link: string }) => {
	const { data } = useQuery({
		queryKey: ["preview-url", link],
		queryFn: async ({ signal }) => {
			try {
				if (!URL.canParse(link)) {
					console.debug(`Missing href or can not parse ${link}`);
					return null;
				}

				const url = new URL(link);
				if (url.protocol !== "https:") {
					console.debug("url protocol is not https", url);
					return null;
				}

				if (!isTauri()) return null;

				const response = await fetch(url, {
					signal,
					method: "GET",
					headers: {
						Accept: "text/xml,text/html,application/xml,application/xhtml+xml",
					},
				});
				if (!response.ok) throw response;
				const contentType =
					response.headers.get("content-type") ??
					response.headers.get("Content-Type") ??
					"";

				const [type] = contentType.split(";");

				console.debug("ContentType", type, contentType);
				if (!isSupportedType(type)) return null;

				const content = await response.text();

				const parser = new DOMParser();

				const doc = parser.parseFromString(content, type);

				const head = doc.querySelector("head");
				if (!head) return null;

				const metatags = head.querySelectorAll("meta");

				const cardInfo: {
					siteName: string;
					title: string;
					img?: string;
					description?: string;
					imgAlt?: string;
					shareUrl?: string;
				} = {
					siteName: url.hostname,
					title: "",
				};

				for (const metatag of metatags) {
					const property = metatag.getAttribute("property");
					switch (property) {
						case "og:title": {
							const value = metatag.getAttribute("content");
							if (value) cardInfo.title = value;
							break;
						}
						case "og:description": {
							const value = metatag.getAttribute("content");
							if (value) cardInfo.description = value;
							break;
						}
						case "og:site_name": {
							const value = metatag.getAttribute("content");
							if (value) cardInfo.siteName = value;
							break;
						}
						case "og:image": {
							const value = metatag.getAttribute("content");
							if (value) cardInfo.img = sanitizeUrl(value);
							break;
						}
						case "og:image:alt": {
							const value = metatag.getAttribute("content");
							if (value) cardInfo.imgAlt = value;
							break;
						}
						case "og:url": {
							const value = metatag.getAttribute("content");
							if (value) cardInfo.shareUrl = sanitizeUrl(value);
							break;
						}
					}
				}

				if (cardInfo.title === "") {
					cardInfo.title = head.querySelector("title")?.textContent ?? link;
				}

				return cardInfo;
			} catch (error) {
				console.error(Error.isError(error) ? error.message : error);
				return null;
			}
		},
	});

	return (
		<Card className="mt-4">
			<CardHeader>
				<CardTitle>
					<span className="text-xs text-muted-foreground">
						{data?.siteName}
					</span>
					<h1 className="text-lg">
						<a
							className="hover:underline text-primary"
							href={data?.shareUrl ?? sanitizeUrl(link)}
							target="_blank"
							rel="noopener noreferrer"
						>
							{data?.title}
						</a>
					</h1>
					<p className="text-sm font-light">{data?.description}</p>
				</CardTitle>
				<CardContent>
					<div className="aspect-square max-h-96">
						<img
							className="h-full w-full"
							src={data?.img}
							alt={data?.imgAlt ?? data?.title}
						/>
					</div>
				</CardContent>
			</CardHeader>
		</Card>
	);
};
