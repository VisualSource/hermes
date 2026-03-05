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

export const VoiceSettings = () => {
	return (
		<div className="flex flex-col p-2 gap-2">
			<Card>
				<CardHeader>
					<CardTitle>Voice Settings</CardTitle>
					<Separator />
				</CardHeader>
				<CardContent>
					<div>
						<Label>Mic</Label>
						<Select>
							<SelectTrigger>
								<SelectValue placeholder="Select Mic" />
							</SelectTrigger>
							<SelectContent>
								<SelectItem>Default</SelectItem>
							</SelectContent>
						</Select>
					</div>
					<div>
						<Label>Camera</Label>
						<Select>
							<SelectTrigger>
								<SelectValue placeholder="Select Camera" />
							</SelectTrigger>
							<SelectContent>
								<SelectItem>Default</SelectItem>
							</SelectContent>
						</Select>
					</div>
				</CardContent>
			</Card>
		</div>
	);
};
