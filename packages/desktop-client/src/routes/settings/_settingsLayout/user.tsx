import { createFileRoute } from "@tanstack/react-router";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";

export const Route = createFileRoute("/settings/_settingsLayout/user")({
	component: RouteComponent,
});

function RouteComponent() {
	return (
		<div className="flex flex-col p-2 h-full gap-2">
			<Card>
				<CardContent></CardContent>
			</Card>
		</div>
	);
}
