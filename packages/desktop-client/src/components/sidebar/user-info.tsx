import {
	HeadphoneOff,
	Headphones,
	Mic,
	MicOff,
	Network,
	PhoneOff,
	ScreenShare,
	Settings2,
	Video,
	VideoOff,
} from "lucide-react";
import { Avatar, AvatarFallback, AvatarImage } from "../ui/avatar";
import { Button } from "../ui/button";
import { useState } from "react";
import { Link, useMatchRoute } from "@tanstack/react-router";
import { useInVoice } from "@/hooks/use-in-voice";
import { app } from "@/lib/clients/app";

import { TooltipButton } from "../ui/tooltip-button";

export const UserInfo = () => {
	const matchRoute = useMatchRoute();
	const params = matchRoute({ to: "/voice/$roomId" });

	const [mute, setMute] = useState(false);
	const [depth, setDepth] = useState(false);

	const inVoice = useInVoice();

	const [video, setVideo] = useState(true);

	return (
		<div className="w-full shadow-2xl divide-y">
			{inVoice ? (
				<div className="py-3 px-2 bg-background">
					<div className="flex items-center gap-2 mb-2">
						<Network className="text-primary" />
						<div className="flex flex-col gap-0.5 leading-none">
							<span className="text-sm">Voice Connected</span>
							<span className="text-xs text-muted-foreground">
								Channel / Server
							</span>
						</div>

						<Button
							size="icon-lg"
							variant="ghost"
							className="ml-auto"
							onClick={() => params && app.leaveVoice(params.roomId)}
						>
							<PhoneOff />
						</Button>
					</div>

					<div className="flex">
						<Button
							size="lg"
							className="flex w-full shrink"
							variant="secondary"
							disabled
						>
							{video ? <VideoOff /> : <Video />}
						</Button>
						<Button
							size="lg"
							className="flex w-full shrink"
							variant="secondary"
						>
							<ScreenShare />
						</Button>
					</div>
				</div>
			) : null}

			<div className="flex gap-2 items-center bg-zinc-700 py-3 px-2">
				<Avatar>
					<AvatarImage />
					<AvatarFallback>CN</AvatarFallback>
				</Avatar>
				<div className="flex flex-col gap-0.5 leading-none">
					<span className="text-sm">Documentation</span>
					<span className="text-xs text-muted-foreground">Status</span>
				</div>
				<div className="flex ml-auto">
					<TooltipButton
						tooltip={mute ? "Mute" : "Unmute"}
						size="icon-lg"
						type="button"
						variant="ghost"
						onClick={() => setMute((e) => !e)}
					>
						{mute ? <Mic /> : <MicOff />}
					</TooltipButton>
					<TooltipButton
						size="icon-lg"
						type="button"
						variant="ghost"
						onClick={() => setDepth((e) => !e)}
						tooltip={depth ? "Depthen" : "Undepthen"}
					>
						{depth ? <Headphones /> : <HeadphoneOff />}
					</TooltipButton>

					<TooltipButton
						tooltip="Settings"
						size="icon-lg"
						type="button"
						variant="ghost"
						nativeButton={false}
						render={(props) => <Link to="/settings" {...props} />}
					>
						<Settings2 />
					</TooltipButton>
				</div>
			</div>
		</div>
	);
};
