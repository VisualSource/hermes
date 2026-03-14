import { Check, ChevronsUpDown } from "lucide-react";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { useQuery } from "@tanstack/react-query";

import { faker } from "@faker-js/faker";
import { useState } from "react";
export const ChannelSwitcher = () => {
	const { data } = useQuery({
		queryKey: ["servers"],
		queryFn: async () => {
			return Array.from({ length: faker.number.int({ min: 2, max: 10 }) }).map(
				() => ({
					id: faker.string.uuid(),
					icon: faker.image.url(),
					name: faker.company.buzzNoun(),
				}),
			);
		},
	});

	const [selected, setSelected] = useState<string | undefined>();

	const selectedItem = data?.find((e) => e.id === selected);

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
								<AvatarImage
									src={selectedItem?.icon}
									alt={selectedItem?.name}
								/>
								<AvatarFallback className="rounded-none bg-sidebar-primary text-sidebar-primary-foreground">
									CN
								</AvatarFallback>
							</Avatar>
						</div>
						<div className="flex flex-col gap-0.5 leading-none">
							<span className="font-medium">{selectedItem?.name}</span>
						</div>
						<ChevronsUpDown className="ml-auto" />
					</button>
				}
			/>
			<DropdownMenuContent align="start">
				{data?.map((server) => (
					<DropdownMenuItem
						key={server.id}
						onClick={() => setSelected(server.id)}
						className="cursor-pointer"
					>
						<Avatar>
							<AvatarImage src={server.icon} alt={server.name} />
							<AvatarFallback>SN</AvatarFallback>
						</Avatar>
						<div>{server.name}</div>
						{selected === server.id && <Check className="ml-auto" />}
					</DropdownMenuItem>
				))}
			</DropdownMenuContent>
		</DropdownMenu>
	);
};
