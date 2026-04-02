import { useServerUser } from "@/hooks/use-server-user";
import type { UUID } from "node:crypto";

export const UserMention = ({ userId }: { userId: UUID }) => {
	const { data } = useServerUser(userId);

	//TODO: replace text color with color from server profile

	return (
		<span className="hover:underline text-green-600">@{data?.username}</span>
	);
};
