import { createFileRoute } from "@tanstack/react-router";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";

export const Route = createFileRoute("/settings/_settingsLayout/account")({
	component: RouteComponent,
});

function RouteComponent() {
	return (
		<div className="flex flex-col p-2 h-full gap-2">
			<Card>
				<CardContent className="flex flex-col items-center justify-center">
					<Avatar className="size-24">
						<AvatarImage />
						<AvatarFallback>CN</AvatarFallback>
					</Avatar>
					<p className="text-2xl">Username</p>
					<div className="gap-2 flex mt-6">
						<Button>Edit</Button>
						<Button variant="destructive">Logout</Button>
					</div>
				</CardContent>
			</Card>
		</div>
	);
}
