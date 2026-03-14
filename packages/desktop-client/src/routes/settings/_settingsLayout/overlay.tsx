import { createFileRoute } from "@tanstack/react-router";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";

export const Route = createFileRoute("/settings/_settingsLayout/overlay")({
	component: RouteComponent,
});

function RouteComponent() {
	return (
		<div className="flex flex-col p-2 gap-2">
			<Card>
				<CardHeader>
					<CardTitle>Overlay Settings</CardTitle>
					<Separator />
				</CardHeader>
				<CardContent className="flex flex-col gap-2">
					<div className="flex flex-col gap-2">
						<Label>Overlay Position</Label>
						<Select>
							<SelectTrigger>
								<SelectValue placeholder="Select position" />
							</SelectTrigger>
							<SelectContent>
								<SelectItem>Top-Left</SelectItem>
								<SelectItem>Top-Center</SelectItem>
								<SelectItem>Top-Right</SelectItem>
								<SelectItem>Bottom-Left</SelectItem>
								<SelectItem>Bottom-Right</SelectItem>
							</SelectContent>
						</Select>
					</div>

					<div className="flex flex-col gap-2">
						<Label>Overlay Sizing</Label>
						<Select>
							<SelectTrigger>
								<SelectValue placeholder="Select sizing" />
							</SelectTrigger>
							<SelectContent>
								<SelectItem>Default</SelectItem>
								<SelectItem>Compact</SelectItem>
							</SelectContent>
						</Select>
					</div>
				</CardContent>
			</Card>
		</div>
	);
}
