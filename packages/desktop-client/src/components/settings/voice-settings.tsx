import { getOutputDevices } from "@/lib/audio";
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
import { useSuspenseQuery } from "@tanstack/react-query";
import { Suspense } from "react";
const MicSelect = () => {
	const { data } = useSuspenseQuery({
		queryKey: ["mic-list"],
		queryFn: async ()=>{
			const outputs = await getOutputDevices();

			return outputs;
		}
	});


	return (
		<Select defaultValue={data.default_device}>
			<SelectTrigger className="w-72">
				<SelectValue placeholder="Select Mic" />
			</SelectTrigger>

			<SelectContent>
				{data.devices.map((item, i) => (
					<SelectItem key={item[0]} className="w-full">
						{item[1]}
					</SelectItem>
				))}
			</SelectContent>
		</Select>
	);
}

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
						<Suspense>
							<MicSelect />
						</Suspense>
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
