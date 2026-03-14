import { Textarea } from "@/components/ui/textarea";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/_text/text/$roomId")({
	component: RouteComponent,
});

function RouteComponent() {
	return (
		<main className="container px-8 h-full flex flex-col pb-6 col-span-10">
			<ul className="h-full"></ul>
			<Textarea></Textarea>
		</main>
	);
}
