import { cn } from "@/lib/utils";
import type { StreamCard as Props } from "./types";
import { Card, CardContent } from "../ui/card";

export const StreamCard = (
	item: Props & { watch: () => void; size?: "sm" },
) => {
	return (
		<Card
			id={item.id}
			className={cn(
				"p-0.5 col-span-1 row-span-1 group shadow-2xl",
				item?.size === "sm" ? "w-52 h-30" : "w-120 h-80",
			)}
		>
			<CardContent
				className="h-full p-0.5 relative overflow-hidden flex justify-center items-center"
				style={{
					backgroundImage: `url(${item.preview})`,
					backgroundSize: "cover",
					backgroundRepeat: "no-repeat",
				}}
			>
				<div className="backdrop-blur-xl h-full w-full absolute" />

				<button
					onClick={item.watch}
					className="absolute top-0 left-0 h-full w-full flex justify-center items-center bg-accent/50 border border-red-500 before:h-6 before:content-['Live'] before:bg-red-500 before:px-1 before:py-0.5 before:top-0 before:absolute before:left-0"
					type="button"
				>
					<div className="px-4 py-2 bg-background/60 font-medium">Watch</div>
				</button>
			</CardContent>
		</Card>
	);
};
