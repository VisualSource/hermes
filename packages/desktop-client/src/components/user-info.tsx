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
import { Avatar, AvatarFallback, AvatarImage } from "./ui/avatar";
import { Button } from "./ui/button";
import { useState } from "react";
import { Link } from "@tanstack/react-router";

export const UserInfo = () => {
	const [mute, setMute] = useState(false);
	const [depth, setDepth] = useState(false);

	const [video, setVideo] = useState(true);

	const [inCall, setInCall] = useState(true);

	return (
		<div className="w-full shadow-2xl divide-y">
			{inCall ? (
				<div className="py-3 px-2 bg-background">
					<div className="flex items-center gap-2 mb-2">
						<Network className="text-primary" />
						<div className="flex flex-col gap-0.5 leading-none">
							<span className="text-sm">Voice Connected</span>
							<span className="text-xs text-muted-foreground">
								Channel / Server
							</span>
						</div>

						<Button size="icon-lg" variant="ghost" className="ml-auto">
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
					<Button
						size="icon-lg"
						type="button"
						variant="ghost"
						onClick={() => setMute((e) => !e)}
					>
						{mute ? <Mic /> : <MicOff />}
					</Button>
					<Button
						size="icon-lg"
						type="button"
						variant="ghost"
						onClick={() => setDepth((e) => !e)}
					>
						{depth ? <Headphones /> : <HeadphoneOff />}
					</Button>
					<Button
						size="icon-lg"
						type="button"
						variant="ghost"
						render={(props) => <Link to="/settings" {...props} />}
					>
						<Settings2 />
					</Button>
				</div>
			</div>
		</div>
	);
};
