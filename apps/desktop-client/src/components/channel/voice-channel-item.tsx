import { Link } from "@tanstack/react-router";
import { Volume2 } from "lucide-react";
import { Avatar, AvatarFallback, AvatarImage } from "../ui/avatar";
import { Suspense } from "react";
import { useSuspenseQuery } from "@tanstack/react-query";
import { faker } from "@faker-js/faker";
import { activeVoiceParticipantsOptions } from "@/lib/api/queries";

const ActiveUser = ({
	username,
	avatar,
}: {
	username: string;
	avatar: string;
	userId: string;
}) => {
	return (
		<li className="hover:bg-accent/60">
			<div className="flex items-center gap-2 px-2 py-0.5">
				<Avatar>
					<AvatarImage src={avatar} alt={username} />
					<AvatarFallback>CN</AvatarFallback>
				</Avatar>
				<div className="flex flex-col gap-0.5 leading-none">
					<span className="font-medium text-xs">{username}</span>
				</div>
			</div>
		</li>
	);
};

const ActiveUsersList = ({ channelId }: { channelId: string }) => {
	const { data } = useSuspenseQuery(activeVoiceParticipantsOptions(channelId));

	return (
		<ul className="pl-8 space-y-1">
			{data.map((user) => (
				<ActiveUser
					key={user.id}
					userId={user.id}
					avatar={user.avatar}
					username={user.username}
				/>
			))}
		</ul>
	);
};

export const VoiceChannel = ({ name, id }: { name: string; id: string }) => {
	return (
		<li>
			<div className="pb-2">
				<Link
					to="/voice/$roomId"
					params={{ roomId: id }}
					className="flex gap-2 text-sm items-center px-4 py-2 bg-sidebar-accent/10 hover:bg-sidebar-accent/60 w-full hover:underline"
				>
					<Volume2 className="size-4" />
					<span className="line-clamp-1">{name}</span>
				</Link>
				<Suspense>
					<ActiveUsersList channelId={id} />
				</Suspense>
			</div>
		</li>
	);
};
