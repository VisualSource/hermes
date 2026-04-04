import { Background } from "@/components/background-grid";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { createFileRoute } from "@tanstack/react-router";
import { useTranslation } from "react-i18next";
import Markdown from "react-markdown";

export const Route = createFileRoute("/_home/")({
	component: Index,
});

const markdown = `
#V1.12 
## Changes 
- change 1 
- change 2 
- change 3 
- change 4 
- change 5
`;

function Index() {
	const { t } = useTranslation();

	return (
		<Background>
			<div className="flex flex-col gap-2 w-full px-20">
				<Card>
					<CardHeader>
						<CardTitle>{t("Welcome")}</CardTitle>
					</CardHeader>
					<CardContent></CardContent>
				</Card>

				<Card>
					<CardHeader>
						<CardTitle>{t("ChangeLog")}</CardTitle>
					</CardHeader>
					<CardContent>
						<article className="container mx-auto">
							<Markdown>{markdown}</Markdown>
						</article>
					</CardContent>
				</Card>
			</div>
		</Background>
	);
}
