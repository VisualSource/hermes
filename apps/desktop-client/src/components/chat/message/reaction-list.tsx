import { Button } from "@/components/ui/button";
import { useUser } from "@/hooks/use-user";
import { cn } from "@/lib/utils";
import { SmilePlus } from "lucide-react";
import { useAddReactionMutation } from "./add-reaction-mutation";

export const ReactionList = ({ reacts }: { reacts: string[] }) => {
	const user = useUser();
	const { mutateAsync } = useAddReactionMutation();

	return (
		<div className="flex flex-wrap mt-2 gap-1">
			{reacts.map((id) => (
				<Button
					onClick={() => mutateAsync(id)}
					size="sm"
					key={id}
					variant="outline"
					className={cn(
						user.id === id &&
							"border-blue-400! bg-blue-300/40! hover:bg-blue-300/50!",
					)}
				>
					<span className="mr-1">1</span> 😁
				</Button>
			))}
			<Button size="icon-sm" variant="outline">
				<SmilePlus />
			</Button>
		</div>
	);
};
