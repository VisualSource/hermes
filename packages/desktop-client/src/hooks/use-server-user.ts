import { serverUsersOptions } from "@/lib/api/queries";
import { useQuery } from "@tanstack/react-query";
import type { UUID } from "node:crypto";
import { useServerId } from "./use-server-id";

export const useServerUser = (userId: UUID) => {
	const serverId = useServerId();
	const results = useQuery({
		...serverUsersOptions(serverId),
		select: (data) => data.find((user) => user.id === userId),
	});

	return results;
};
