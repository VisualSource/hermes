import { Separator } from "@/components/ui/separator";
import { createFileRoute, Link, Outlet } from "@tanstack/react-router";
import {
	Bell,
	Cpu,
	Gamepad2,
	HatGlasses,
	LayoutGrid,
	MonitorSmartphone,
	TvMinimal,
	User2,
	Volume2,
} from "lucide-react";

export const Route = createFileRoute("/settings/_settingsLayout")({
	component: RouteComponent,
});

function RouteComponent() {
	return (
		<div className="flex h-full w-full">
			<aside className="flex flex-col bg-accent h-full w-52 p-2 space-y-1.5">
				<h1 className="font-bold px-2.5 py-2">User Settings</h1>
				<Link
					to="/settings/account"
					className="flex gap-2 px-2 py-1 hover:bg-background/60"
				>
					<User2 /> Account
				</Link>
				<Link
					to="/settings/user"
					className="flex gap-2 px-2 py-1 hover:bg-background/60"
				>
					<HatGlasses /> Profiles
				</Link>
				<Link
					to="/settings/notifications"
					className="flex gap-2 px-2 py-1 hover:bg-background/60"
				>
					<Bell /> Notifications
				</Link>
				<Link
					to="/settings/devices"
					className="flex gap-2 px-2 py-1 hover:bg-background/60"
				>
					<MonitorSmartphone /> Devices
				</Link>
				<Separator />
				<h1 className="font-bold  px-2.5 py-2">App Settings</h1>
				<Link
					to="/settings/app"
					className="flex gap-2 px-2 py-1 hover:bg-background/60"
				>
					<LayoutGrid /> App
				</Link>
				<Link
					to="/settings/startup"
					className="flex gap-2 px-2 py-1 hover:bg-background/60"
				>
					<Cpu /> System
				</Link>
				<Link
					to="/settings/voice"
					className="flex gap-2 px-2 py-1 hover:bg-background/60"
				>
					<Volume2 /> Voice
				</Link>
				<Separator />
				<h1 className="font-bold  px-2.5 py-2">Overlay</h1>
				<Link
					to="/settings/overlay"
					className="flex gap-2  px-2 py-1 hover:bg-background/60"
				>
					<TvMinimal /> Overlay
				</Link>
				<Link
					to="/settings/overlay-games"
					className="flex gap-2 px-2 py-1 hover:bg-background/60"
				>
					<Gamepad2 /> Registered Games
				</Link>
			</aside>
			<div className="w-full h-full">
				<Outlet />
			</div>
		</div>
	);
}
