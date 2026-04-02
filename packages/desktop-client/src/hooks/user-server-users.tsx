import { useQuery } from "@tanstack/react-query";
import { useServerId } from "./use-server-id";
import { serverUsersOptions } from "@/lib/api/queries";

export const useServerUsers = () => {
	const serverId = useServerId();

	const query = useQuery(serverUsersOptions(serverId));

	return query;
};
