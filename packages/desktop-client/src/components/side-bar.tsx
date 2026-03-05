import {
	Boxes,
	Crown,
	Hash,
	Headset,
	Mic,
	Network,
	Settings2,
} from "lucide-react";
import { Avatar, AvatarFallback, AvatarImage } from "./ui/avatar";
import { Separator } from "./ui/separator";
import { ChannelSwitcher } from "./channel-switcher";
import {
	Accordion,
	AccordionContent,
	AccordionItem,
	AccordionTrigger,
} from "./ui/accordion";
import { Button } from "./ui/button";
import { UserInfo } from "./user-info";
import { Link } from "@tanstack/react-router";

export const SideBar = () => {
	return (
		<div className="w-80 bg-sidebar px-2 pb-2 relative flex flex-col overflow-hidden">
			<ChannelSwitcher />
			<Separator />
			<ul>
				<li>
					<Link
						to="/roles/$roomId"
						params={{ roomId: "RoleRoomId" }}
						className="flex gap-2 text-sm items-center px-4 py-2 bg-sidebar-accent/10 hover:bg-sidebar-accent/60 w-full hover:underline"
					>
						<Crown className="size-4" />{" "}
						<span className="line-clamp-1">Roles Channel</span>
					</Link>
				</li>
				<li>
					<Link
						to="/text/$roomId"
						params={{ roomId: "TextRoomID" }}
						className="flex gap-2 text-sm items-center px-4 py-2 bg-sidebar-accent/10 hover:bg-sidebar-accent/60 w-full hover:underline"
					>
						<Hash className="size-4" />{" "}
						<span className="line-clamp-1">Some Text Channel</span>
					</Link>
				</li>

				<li>
					<div>
						<Link
							to="/voice/$roomId"
							params={{ roomId: "VoiceRoomId" }}
							className="flex gap-2 text-sm items-center px-4 py-2 bg-sidebar-accent/10 hover:bg-sidebar-accent/60 w-full hover:underline"
						>
							<Network className="size-4" />{" "}
							<span className="line-clamp-1">Voice Channel</span>
						</Link>
						<ul className="pl-8">
							<li>
								<div className="flex items-center gap-2">
									<Avatar>
										<AvatarImage />
										<AvatarFallback>CN</AvatarFallback>
									</Avatar>
									<div className="flex flex-col gap-0.5 leading-none">
										<span className="font-medium text-xs">Documentation</span>
									</div>
								</div>
							</li>
						</ul>
					</div>
				</li>

				<li className="py-2">
					<Separator />
				</li>

				<li>
					<Accordion>
						<AccordionItem>
							<AccordionTrigger className="px-4 py-2 text-sm bg-sidebar-accent/10 hover:bg-sidebar-accent/60 gap-2">
								<Boxes className="size-4" />
								<span className="line-clamp-1">Group</span>
							</AccordionTrigger>
							<AccordionContent></AccordionContent>
						</AccordionItem>
					</Accordion>
				</li>
			</ul>

			<UserInfo />
		</div>
	);
};