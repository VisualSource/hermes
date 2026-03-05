import { Check, ChevronsUpDown, GalleryVerticalEnd } from "lucide-react";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from "./ui/dropdown-menu";
import { Avatar, AvatarFallback, AvatarImage } from "./ui/avatar";

export const ChannelSwitcher = () => {
	return (
		<DropdownMenu>
			<DropdownMenuTrigger
				render={
					<button
						type="button"
						className="bg-sidebar text-sidebar-accent-foreground w-full flex items-center p-2 gap-2 hover:bg-sidebar-accent/60 rounded"
					>
						<div className="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
							<Avatar className="rounded-none">
								<AvatarImage />
								<AvatarFallback className="rounded-sm bg-sidebar-primary text-sidebar-primary-foreground">
									CN
								</AvatarFallback>
							</Avatar>
						</div>
						<div className="flex flex-col gap-0.5 leading-none">
							<span className="font-medium">Documentation</span>
						</div>
						<ChevronsUpDown className="ml-auto" />
					</button>
				}
			/>
			<DropdownMenuContent align="start">
				<DropdownMenuItem>
					Server Name
					<Check className="ml-auto" />
				</DropdownMenuItem>
			</DropdownMenuContent>
		</DropdownMenu>
	);
};
