import { gemoji } from "gemoji";
import { Popover, PopoverContent, PopoverTrigger } from "../ui/popover";
import { TooltipButton } from "../ui/tooltip-button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../ui/tabs";
import { Button } from "../ui/button";
import { SmilePlus } from "lucide-react";

const groupdedGemoji = () => {
	const emojis = [];

	for (let i = 0; i < gemoji.length; i += 6) {
		emojis.push(gemoji.slice(i, i + 6));
	}

	return emojis;
};

const emojis = groupdedGemoji();


export const EmojiPicker = ({
	addEmoji,
}: {
	addEmoji: (id: string) => void;
}) => {
	return (
		<Popover>
			<PopoverTrigger
				render={
					<TooltipButton
						tooltip="Emojis"
						size="icon-lg"
						variant="secondary"
						className="text-lg"
					>
						<SmilePlus />
					</TooltipButton>
				}
			/>
			<PopoverContent align="end" className="w-96 h-69 max-h-96">
				<Tabs className="h-full">
					<TabsList className="w-full">
						<TabsTrigger value="default">Emojis</TabsTrigger>
						<TabsTrigger value="custom-emojis">Customs</TabsTrigger>
					</TabsList>
					<TabsContent value="default" className="overflow-y-auto">
						{emojis.map((group, item) => (
							<div key={item}>
								{group.map((emoji) => (
									<Button
										variant="ghost"
										key={emoji.emoji}
										onClick={() => addEmoji(emoji.names[0])}
										className="w-14 h-14"
									>
										<span className="text-3xl">{emoji.emoji}</span>
									</Button>
								))}
							</div>
						))}
					</TabsContent>
					<TabsContent value="custom-emojis">
						<Button
							variant="ghost"
							onClick={() => addEmoji("hce_1111")}
							className="w-14 h-14 "
						>
							<img
								src="https://cdn3.emoji.gg/emojis/254673-spray.gif"
								className="h-full w-full object-contain"
								alt=":hce_1111:"
							/>
						</Button>
					</TabsContent>
				</Tabs>
				<div className="h-10 border-t"></div>
			</PopoverContent>
		</Popover>
	);
};
