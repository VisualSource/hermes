import { faker } from "@faker-js/faker";
import {
	infiniteQueryOptions,
	keepPreviousData,
	queryOptions,
} from "@tanstack/react-query";
import type { UUID } from "node:crypto";
import type { Message, Server, User, Channel, MessagesQuery } from "./types";


const fakeData_serverListUsers = Array.from({
	length: faker.number.int({ min: 5, max: 15 }),
}).map(
	() =>
		({
			id: faker.string.uuid(),
			avatar: faker.image.avatarGitHub(),
			status: faker.number.int({ min: 0, max: 2 }),
			roles: faker.number.bigInt(),
			username: faker.person.firstName(),
			statusText: faker.lorem.words({ min: 0, max: 2 }),
		}) as User,
);
const fakeData_textChannel = Array.from({ length: 20 })
	.map(
		() =>
			({
				id: faker.string.ulid(),
				timestamp: faker.date.recent().toUTCString(),
				userId: faker.helpers.arrayElement(fakeData_serverListUsers).id,
				reacts: [],
				content: faker.lorem.sentences({ min: 1, max: 3 }),
			}) as Message,
	)
	.concat([
		{
			id: faker.string.ulid(),
			userId: faker.helpers.arrayElement(fakeData_serverListUsers).id,
			content: faker.lorem.lines(1),
			reacts: ["cat"],
			timestamp: faker.date.recent().toUTCString(),
		} as Message,
		{
			id: faker.string.ulid(),
			userId: faker.helpers.arrayElement(fakeData_serverListUsers).id,
			content: `<@${faker.helpers.arrayElement(fakeData_serverListUsers).id}>`,
			reacts: [],
			timestamp: faker.date.recent().toUTCString(),
		} as Message,
		{
			id: faker.string.ulid(),
			userId: faker.helpers.arrayElement(fakeData_serverListUsers).id,
			content: ":cat: :hce_00000000000:",
			reacts: [],
			timestamp: faker.date.recent().toUTCString(),
		} as Message,
		{
			id: faker.string.ulid(),
			userId: faker.helpers.arrayElement(fakeData_serverListUsers).id,
			content: "https://youtube.com/shorts/FiMXgmhSlo0",
			reacts: [],
			timestamp: faker.date.recent().toUTCString(),
		} as Message,
	]) as Message[];
const fakeData_friendList = faker.helpers.arrayElements(
	fakeData_serverListUsers,
);

export const serverUsersOptions = (serverId: UUID) => {
	return queryOptions({
		queryKey: ["user-list-server", serverId],
		queryFn: () => {
			return fakeData_serverListUsers;
		},
		refetchOnWindowFocus: false,
		refetchOnMount: false,
	});
};

export const textChannelOptions = (
	queryKey: string[],
	initialPageParam: string,
) => {
	return infiniteQueryOptions({
		refetchOnWindowFocus: false,
		refetchOnReconnect: true,
		getNextPageParam: (page) => page.nextCursor,
		getPreviousPageParam: (page) => page.prevCursor,
		queryKey,
		maxPages: 3,
		initialPageParam,
		placeholderData: keepPreviousData,
		queryFn: () => {
			return {
				results: fakeData_textChannel,
			} as MessagesQuery;
		},
	});
};

export const peerChannelOptions = (
	peerId: string,
	initialPageParam: string,
) => {
	return textChannelOptions(["peer-text", peerId], initialPageParam);
};


export const serverTextChannelOptions = (
	serverId: UUID,
	channelId: UUID,
	initialPageParam: string,
) => {
	return textChannelOptions(
		["server-text", serverId, channelId],
		initialPageParam,
	);
};

export const serverListOptions = () => {
	return queryOptions({
		queryKey: ["server-list"],
		queryFn: () => {
			return Array.from({ length: faker.number.int({ min: 3, max: 10 }) }).map(
				() =>
					({
						id: faker.string.uuid(),
						name: faker.company.name(),
						icon: faker.image.avatarGitHub(),
					}) as Server,
			);
		},
	});
};

export const serverChannelsOptions = (serverId: UUID) => {
	return queryOptions({
		queryKey: ["server-channel-list", serverId],
		queryFn: async () => {
			return [
				{
					type: "tags",
					name: "Tags",
					id: faker.string.uuid(),
				},
				{
					type: "text",
					name: "Some Text Channel",
					id: faker.string.uuid(),
				},
				{
					type: "divider",
					id: faker.string.uuid(),
				},
				{
					type: "voice",
					name: "Some Channel Name",
					id: faker.string.uuid(),
				},
				{
					type: "voice",
					name: "Other Voice",
					id: faker.string.uuid(),
				},
				{
					type: "group",
					id: faker.string.uuid(),
					name: "Some Group name",
					items: [
						{
							type: "text",
							name: "Sub Group Channel",
							id: faker.string.uuid(),
						},
					],
				},
			] as Channel.ChannelItem[];
		},
	});
};

export const friendListOptions = () => {
	return queryOptions({
		refetchOnWindowFocus: false,
		queryKey: ["friend-list"],
		queryFn: async () => {
			return fakeData_friendList;
		},
	});
};


export const activeVoiceParticipantsOptions = (channelId: string) => {
	return queryOptions({
		queryKey: ["active-voice-participant", channelId],
		queryFn: async () => {
			return faker.helpers.arrayElements(fakeData_serverListUsers);
		},
	});
};