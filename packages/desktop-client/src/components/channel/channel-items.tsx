import { Link } from "@tanstack/react-router";
import { Boxes, Crown, Hash } from "lucide-react";
import {
	Accordion,
	AccordionContent,
	AccordionItem,
	AccordionTrigger,
} from "../ui/accordion";
import { Separator } from "../ui/separator";

export const TextChannel = ({ name, id }: { name: string; id: string }) => {
	return (
		<li>
			<Link
				to="/text/$roomId"
				params={{ roomId: id }}
				className="flex gap-2 text-sm items-center px-4 py-2 bg-sidebar-accent/10 hover:bg-sidebar-accent/60 w-full hover:underline"
			>
				<Hash className="size-4" />
				<span className="line-clamp-1">{name}</span>
			</Link>
		</li>
	);
};

export const TagsChannel = ({ name, id }: { id: string; name: string }) => {
	return (
		<li>
			<Link
				to="/roles/$roomId"
				params={{ roomId: id }}
				className="flex gap-2 text-sm items-center px-4 py-2 bg-sidebar-accent/10 hover:bg-sidebar-accent/60 w-full hover:underline"
			>
				<Crown className="size-4" />
				<span className="line-clamp-1">{name}</span>
			</Link>
		</li>
	);
};

export const GroupChannel = ({
	name,
	children,
}: React.PropsWithChildren<{ name: string }>) => {
	return (
		<li>
			<Accordion>
				<AccordionItem>
					<AccordionTrigger className="px-4 py-2 text-sm bg-sidebar-accent/10 hover:bg-sidebar-accent/60 gap-2">
						<Boxes className="size-4" />
						<span className="line-clamp-1">{name}</span>
					</AccordionTrigger>
					<AccordionContent>
						<ul className="ml-6">{children}</ul>
					</AccordionContent>
				</AccordionItem>
			</Accordion>
		</li>
	);
};
export const DividerChannel = () => {
	return (
		<li className="py-2">
			<Separator />
		</li>
	);
};
