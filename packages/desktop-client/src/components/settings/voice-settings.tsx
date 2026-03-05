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
			const stream = await navigator.mediaDevices.getUserMedia({ audio: true, video: false });

			stream.getTracks().forEach(track => track.stop());
			
			//const stream = await navigator.mediaDevices.getUserMedia({ audio: true, video: false });
  			const devices = await navigator.mediaDevices.enumerateDevices();
  			const microphones = devices.filter(device => device.kind === 'audioinput');
			console.log(devices);
			return microphones;
		}
	});


	return (
		<SelectContent>
			{data.map((item,i)=>(
				<SelectItem key={i} className="w-full">{item.label}</SelectItem>
			))}
		</SelectContent>
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
						<Select>
							<SelectTrigger>
								<SelectValue placeholder="Select Mic" />
							</SelectTrigger>
							<Suspense>
								<MicSelect/>
							</Suspense>
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
