import { useQuery } from "@tanstack/react-query";
import { serverUsersOptions } from "@/lib/api/queries";
import { useApp } from "./use-app";

export const useServerUsers = () => {
	const serverId = useApp(store => store.activeServerId);
	const query = useQuery(serverUsersOptions(serverId));

	return query;
};
