import { Separator } from "../ui/separator";
import { ChannelSwitcher } from "./channel-switcher";

import { UserInfo } from "./user-info";

import {
	DividerChannel,
	GroupChannel,
	TagsChannel,
	TextChannel,
} from "../channel/channel-items";
import { VoiceChannel } from "../channel/voice-channel-item";
import { useQuery } from "@tanstack/react-query";
import { faker } from "@faker-js/faker";
import { Button } from "../ui/button";
import { Home } from "lucide-react";
import { Link } from "@tanstack/react-router";

type Group<T> = { type: T; name: string; id: string };
type GroupGroup = Group<"group"> & {
	items: (GroupDivider | Group<"text" | "voice">)[];
};
type GroupDivider = { type: "divider"; id: string };

type ServerGroup = Group<"tags" | "text" | "voice"> | GroupDivider | GroupGroup;

export const SideBar = () => {
	const { data } = useQuery({
		queryKey: ["channel-list", "CHANNELID"],
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
			] as ServerGroup[];
		},
	});

	return (
		<div className="w-80 bg-sidebar px-2 pb-2 relative flex flex-col overflow-hidden shrink-0 col-span-3">
			<div className="flex gap-2 items-center">
				<Button
					size="icon-lg"
					variant="outline"
					nativeButton={false}
					render={(props) => <Link to="/" {...props} />}
				>
					<Home />
				</Button>
				<ChannelSwitcher />
			</div>
			<Separator />

			<ul className="overflow-y-auto mb-auto">
				{data?.map((channel) => {
					switch (channel.type) {
						case "text":
							return (
								<TextChannel
									name={channel.name}
									id={channel.id}
									key={channel.id}
								/>
							);
						case "tags":
							return (
								<TagsChannel
									name={channel.name}
									id={channel.id}
									key={channel.id}
								/>
							);
						case "divider":
							return <DividerChannel key={channel.id} />;
						case "group":
							return (
								<GroupChannel
									name={channel.name}
									id={channel.id}
									key={channel.id}
								>
									{channel.items.map((subChannel) => {
										switch (subChannel.type) {
											case "text":
												return (
													<TextChannel
														id={subChannel.id}
														name={subChannel.name}
														key={subChannel.id}
													/>
												);
											case "voice":
												return (
													<VoiceChannel
														id={subChannel.id}
														name={subChannel.name}
														key={subChannel.id}
													/>
												);
											case "divider":
												return <DividerChannel key={subChannel.id} />;
											default:
												return null;
										}
									})}
								</GroupChannel>
							);
						case "voice":
							return (
								<VoiceChannel
									name={channel.name}
									id={channel.id}
									key={channel.id}
								/>
							);

						default:
							return null;
					}
				})}
			</ul>
			<UserInfo />
		</div>
	);
};
