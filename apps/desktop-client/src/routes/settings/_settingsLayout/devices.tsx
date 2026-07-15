import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/settings/_settingsLayout/devices")({
	component: RouteComponent,
});

function RouteComponent() {
	return (
		<div className="h-full w-full p-2">
			<Card>
				<div>
					<Button variant="destructive">Logout All</Button>
				</div>

				<ul>
					<li>
						<div>Linux * Desktop Client</div>
						<details>
							<summary>Keys</summary>
							<ul>
								<li>Key *</li>
							</ul>
						</details>
					</li>
					<li>
						<div>Android * Mobile Client</div>
						<details>
							<summary>Keys</summary>
							<ul>
								<li>Key *</li>
							</ul>
						</details>
					</li>
				</ul>
			</Card>
		</div>
	);
}
