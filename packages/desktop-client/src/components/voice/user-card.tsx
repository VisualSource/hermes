import { cn } from "@/lib/utils";
import { Card, CardContent } from "../ui/card";
import type { UserCard as Props } from "./types";
import { VoiceIndicator } from "./voice-indicator";
import { Avatar, AvatarFallback, AvatarImage } from "../ui/avatar";

export const UserCard = (item: Props & { size?: "sm" }) => {
	return (
		<Card
			id={item.id}
			className={cn(
				"p-0.5 group shadow-2xl flex @container",
				item.size === "sm"
					? "w-52 h-30 shrink-0"
					: "w-[30cqw] h-[20cqw] min-h-23.25 min-w-33.75 max-h-40 max-w-65",
			)}
		>
			<CardContent
				className="flex place-content-center place-items-center h-full w-full relative"
				style={{
					backgroundColor: item.type === "user" ? item.color : undefined,
				}}
			>
				<div className="hidden absolute bottom-1 left-1 group-hover:flex bg-accent/60 px-2 py-1 transition-all duration-100 z-10">
					{item.username}
				</div>

				<div className="backdrop-blur-xl h-full w-full absolute" />

				<VoiceIndicator />

				<Avatar className="size-10 @md:size-20">
					<AvatarImage src={item.avatar} alt={item.username} />
					<AvatarFallback>UN</AvatarFallback>
				</Avatar>
			</CardContent>
		</Card>
	);
};
