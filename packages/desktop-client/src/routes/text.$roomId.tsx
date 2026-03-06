import { Textarea } from "@/components/ui/textarea";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/text/$roomId")({
  component: RouteComponent,
});

function RouteComponent() {
  return (
			<div className="container px-8 h-full flex flex-col pb-6">
				<ul className="h-full"></ul>
				<Textarea></Textarea>
			</div>
		);
}
