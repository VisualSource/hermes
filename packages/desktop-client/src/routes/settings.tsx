import { AppSettings } from "@/components/settings/app-settings";
import { OverlaySettings } from "@/components/settings/overlay-settings";
import { Avatar } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
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
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
	TooltipContent,
	TooltipTrigger,
	Tooltip,
} from "@/components/ui/tooltip";
import { createFileRoute } from "@tanstack/react-router";
import {
	ExternalLink,
	LayoutGrid,
	TvMinimal,
	User2,
	Volume2,
} from "lucide-react";

export const Route = createFileRoute("/settings")({
	component: Index,
});

function Index() {
	return (
		<Tabs orientation="vertical" defaultValue="app" className="grow flex">
			<TabsList className="h-full min-w-36">
				<TabsTrigger value="app">
					<LayoutGrid />
					App
				</TabsTrigger>
				<TabsTrigger value="user">
					<User2 /> User
				</TabsTrigger>
				<TabsTrigger value="overlay">
					<TvMinimal /> Overlay
				</TabsTrigger>
				<TabsTrigger value="voice">
					<Volume2 /> Voice
				</TabsTrigger>
			</TabsList>
			<TabsContent value="app">
				<AppSettings />
			</TabsContent>
			<TabsContent value="user">
				<div>
					<Avatar></Avatar>
					<p>Username</p>
				</div>
				<Button>Edit</Button>
				<Button>Logout</Button>
			</TabsContent>
			<TabsContent value="overlay">
				<OverlaySettings />
			</TabsContent>
			<TabsContent value="voice">
				<label>Input</label>
				<Select>
					<SelectTrigger>
						<SelectValue />
					</SelectTrigger>
				</Select>
				<div>
					<label>Video</label>
					<Select>
						<SelectTrigger>
							<SelectValue />
						</SelectTrigger>
					</Select>
				</div>
			</TabsContent>
		</Tabs>
	);
}