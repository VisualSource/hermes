import { useQuery } from "@tanstack/react-query";
import { Avatar, AvatarFallback, AvatarImage } from "../ui/avatar";
import { serverUsersOptions } from "@/lib/api/queries";
import { useServerId } from "@/hooks/use-server-id";
import type { UUID } from "node:crypto";

export const UserAvatar = ({
	userId,
	...props
}: { userId: UUID } & React.ComponentProps<typeof Avatar>) => {
	const serverId = useServerId();
	const { data } = useQuery({
		...serverUsersOptions(serverId),
		select: (data) => data.find((user) => user.id === userId),
	});

	return (
		<Avatar {...props}>
			<AvatarImage src={data?.avatar} />
			<AvatarFallback>UN</AvatarFallback>
		</Avatar>
	);
};
