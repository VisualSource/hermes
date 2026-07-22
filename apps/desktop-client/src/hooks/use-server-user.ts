import { serverUsersOptions } from "@/lib/api/queries";
import { useQuery } from "@tanstack/react-query";
import { useApp, type UUID } from "./use-app";


export const useServerUser = (userId: UUID | null) => {
	const serverId = useApp(store=>store.activeServerId);
	const results = useQuery({
		enabled: userId !== null,
		...serverUsersOptions(serverId),
		select: (data) => data.find((user) => user.id === userId),
	});

	return results;
};
