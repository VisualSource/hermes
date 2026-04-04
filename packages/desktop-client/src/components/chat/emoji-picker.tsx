import { gemoji } from "gemoji";
import { Popover, PopoverContent, PopoverTrigger } from "../ui/popover";
import { TooltipButton } from "../ui/tooltip-button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../ui/tabs";

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
						😀
					</TooltipButton>
				}
			/>
			<PopoverContent align="end" className="w-56 h-52 max-h-52">
				<Tabs className="h-full">
					<TabsList className="w-full">
						<TabsTrigger value="default">Emojis</TabsTrigger>
						<TabsTrigger value="custom-emojis">Customs</TabsTrigger>
					</TabsList>
					<TabsContent value="default" className="overflow-y-auto">
						{gemoji.slice(0, 15).map((emoji) => (
							<TooltipButton
								variant="ghost"
								key={emoji.emoji}
								tooltip={emoji.names[0]}
								onClick={() => addEmoji(emoji.names[0])}
								className="h-10 w-10"
							>
								{emoji.emoji}
							</TooltipButton>
						))}
					</TabsContent>
					<TabsContent value="custom-emojis">
						<TooltipButton
							variant="ghost"
							tooltip="custom1"
							onClick={() => addEmoji("hce_1111")}
						>
							<img
								src="https://cdn3.emoji.gg/emojis/254673-spray.gif"
								className="w-4 h-4"
								alt=":hce_1111:"
							/>
						</TooltipButton>
					</TabsContent>
				</Tabs>
			</PopoverContent>
		</Popover>
	);
};
