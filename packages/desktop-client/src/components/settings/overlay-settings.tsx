import { Eye, EyeOff } from "lucide-react";
import { Button } from "../ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Label } from "../ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "../ui/select";
import { Separator } from "../ui/separator";

export const OverlaySettings = () => {
	return (
		<div className="flex flex-col p-2 gap-2">
			<Card>
				<CardHeader>
					<CardTitle>Overlay Settings</CardTitle>
					<Separator />
				</CardHeader>
				<CardContent>
					<div>
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

					<div>
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
			<Card>
				<CardHeader>
					<CardTitle>Inject</CardTitle>
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
};
