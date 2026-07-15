import { createFileRoute } from "@tanstack/react-router";
import { ExternalLink } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import {
	Tooltip,
	TooltipContent,
	TooltipTrigger,
} from "@/components/ui/tooltip";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { useQuery } from "@tanstack/react-query";
import { getTauriVersion, getVersion } from "@tauri-apps/api/app";
import { getOpusVersion } from "@/lib/audio/audio";

export const Route = createFileRoute("/settings/_settingsLayout/app")({
	component: RouteComponent,
});

function RouteComponent() {
	const { data } = useQuery({
		queryKey: ["app-details"],
		queryFn: async () => {
			const results = await Promise.all([
				getTauriVersion(),
				getVersion(),
				getOpusVersion(),
			]);

			return {
				tauri: results[0],
				app: results[1],
				overlay: "0.0.0",
				channel: "stable",
				build: "git-1111",
				opus: results[2],
			};
		},
	});

	return (
		<div className="flex flex-col p-2 gap-2">
			<Card>
				<CardHeader>
					<CardTitle>Themes</CardTitle>
					<Separator />
				</CardHeader>
				<CardContent className="flex flex-col gap-2">
					<div className="flex flex-col gap-2">
						<Label>Application Theme</Label>
						<div className="flex gap-2 items-center">
							<Select>
								<SelectTrigger className="min-w-48">
									<SelectValue placeholder="Select a theme" />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value="default">Default</SelectItem>
								</SelectContent>
							</Select>
							<Tooltip>
								<TooltipTrigger
									render={
										<Button size="icon-lg" onClick={() => {}}>
											<ExternalLink />
										</Button>
									}
								/>
								<TooltipContent>Open theme folder</TooltipContent>
							</Tooltip>
						</div>
					</div>
					<div className="flex flex-col gap-2">
						<div className="flex gap-2">
							<Label>Dark Mode</Label>
							<Switch />
						</div>
					</div>
					<div></div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader>
					<CardTitle>Application Details</CardTitle>
					<Separator />
				</CardHeader>
				<CardContent>
					<Button variant="secondary">Check for updates</Button>

					<Separator />

					<div className="font-medium">
						App: <span className="text-muted-foreground">v{data?.app}</span>
					</div>
					<div className="font-medium">
						Tauri: <span className="text-muted-foreground">v{data?.tauri}</span>
					</div>
					<div className="font-medium">
						Build: <span className="text-muted-foreground">{data?.build}</span>
					</div>
					<div className="font-medium">
						Channel:{" "}
						<span className="text-muted-foreground">{data?.channel}</span>
					</div>
					<div className="font-medium">
						Overlay:{" "}
						<span className="text-muted-foreground">v{data?.overlay}</span>
					</div>
					<div className="font-medium">
						Opus: <span className="text-muted-foreground">v{data?.opus}</span>
					</div>
				</CardContent>
			</Card>
		</div>
	);
}
