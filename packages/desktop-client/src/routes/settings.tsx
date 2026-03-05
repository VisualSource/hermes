import { AppSettings } from "@/components/settings/app-settings";
import { OverlaySettings } from "@/components/settings/overlay-settings";
import { UserSettings } from "@/components/settings/user-settings";
import { VoiceSettings } from "@/components/settings/voice-settings";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { createFileRoute } from "@tanstack/react-router";
import { LayoutGrid, TvMinimal, User2, Volume2 } from "lucide-react";

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
				<UserSettings />
			</TabsContent>
			<TabsContent value="overlay">
				<OverlaySettings />
			</TabsContent>
			<TabsContent value="voice">
				<VoiceSettings />
			</TabsContent>
		</Tabs>
	);
}