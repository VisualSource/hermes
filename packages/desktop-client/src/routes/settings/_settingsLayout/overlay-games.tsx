import { createFileRoute } from "@tanstack/react-router";
import { Eye, EyeOff, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export const Route = createFileRoute("/settings/_settingsLayout/overlay-games")(
	{
		component: RouteComponent,
	},
);

function RouteComponent() {
	return (
		<div className="p-2">
			<Card>
				<CardHeader className="flex justify-between items-center">
					<CardTitle>Inject</CardTitle>
					<Button size="icon-lg" variant="secondary">
						<Plus />
					</Button>
				</CardHeader>
				<CardContent>
					<ul className="divide-y divide-accent">
						<li>
							<div className="flex justify-between items-center p-2 hover:bg-zinc-700/60">
								<div>Game Title</div>
								<Button variant="ghost" size="icon-lg">
									<Eye />
								</Button>
							</div>
						</li>
						<li>
							<div className="flex justify-between items-center p-2 hover:bg-zinc-700/60">
								<div>Game Title</div>
								<Button variant="ghost" size="icon-lg">
									<EyeOff />
								</Button>
							</div>
						</li>
					</ul>
				</CardContent>
			</Card>
		</div>
	);
}
