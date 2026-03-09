
import { Separator } from "./ui/separator";
import { ChannelSwitcher } from "./channel-switcher";

import { UserInfo } from "./user-info";

import {
	DividerChannel,
	GroupChannel,
	TagsChannel,
	TextChannel,
} from "./channel/channel-items";
import { VoiceChannel } from "./channel/voice-channel-item";

export const SideBar = () => {
	return (
		<div className="w-80 bg-sidebar px-2 pb-2 relative flex flex-col overflow-hidden shrink-0 col-span-3">
			<ChannelSwitcher />
			<Separator />

			<ul className="overflow-y-auto mb-auto">
				<TagsChannel name="Tags" id="someId" />
				<TextChannel name="Some Text Channel" id="aaaa" />

				<DividerChannel />

				<GroupChannel name="Some Group">
					<TextChannel name="Some Text Channel" id="aaaa" />
				</GroupChannel>

				<VoiceChannel name="SomeChannelName" id="SomeGammeID" />

				<VoiceChannel name="OtherChannelId" id="SomeGammeID2" />
			</ul>

			<UserInfo />
		</div>
	);
};