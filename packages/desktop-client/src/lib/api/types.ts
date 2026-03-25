import type { UUID } from "node:crypto";

export const getStatusColor = (status: UserStatus) => {
	switch (status) {
		case UserStatus.Online:
			return "bg-green-500";
		case UserStatus.Offline:
			return "bg-gray-500";
		case UserStatus.Ignoring:
			return "bg-red-500";
	}
};

export enum UserStatus {
	Online,
	Offline,
	Ignoring,
}

export type User = {
	id: UUID;
	username: string;
	avatar: string;
	status: UserStatus;
	statusText?: string;
	roles: bigint;
};

export type Message = {
	timestamp: string;
	id: string;
	userId: UUID;
	reacts: string[];
	content: string;
};

export type Server = {
	id: UUID;
	name: string;
	icon: string;
};

export namespace Channel {
	export type Item<T> = { type: T; name: string; id: string };
	export type GroupItem = Item<"group"> & {
		items: (GroupDivider | Item<"text" | "voice">)[];
	};
	export type GroupDivider = { type: "divider"; id: string };

	export type ChannelItem =
		| Item<"tags" | "text" | "voice">
		| GroupDivider
		| GroupItem;
}
