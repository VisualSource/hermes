import { useEffect, useState } from "react";
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
} from "../ui/alert-dialog";
import { Switch } from "../ui/switch";
import { Label } from "../ui/label";

const EventKeys = {
	Request: "hermes::change-channel-ask-request",
	Response: "hermes::change-channel-ask-response",
} as const;

const dispatchResponse = (value: boolean) =>
	window.dispatchEvent(new CustomEvent(EventKeys.Response, { detail: value }));

export const requestChannelSwitch = async () => {
	const { resolve, promise } = Promise.withResolvers<boolean>();

	window.addEventListener(
		EventKeys.Response,
		(ev) => resolve((ev as CustomEvent<boolean>).detail),
		{ once: true },
	);
	window.dispatchEvent(new Event(EventKeys.Request));

	return await promise;
};

export const ChangeChannelAlertDialog = () => {
	const [open, setOpen] = useState(false);

	useEffect(() => {
		const onEvent = () => setOpen(true);

		window.addEventListener(EventKeys.Request, onEvent);
		return () => {
			window.removeEventListener(EventKeys.Request, onEvent);
		};
	}, []);

	return (
		<AlertDialog
			open={open}
			onOpenChange={(state, details) => {
				switch (details.reason) {
					case "trigger-press":
					case "outside-press":
					case "escape-key":
					case "close-press":
					case "focus-out":
					case "imperative-action":
						dispatchResponse(false);
						break;
				}

				setOpen(state);
			}}
		>
			<AlertDialogContent>
				<AlertDialogHeader>
					<AlertDialogTitle>Are you absolutely sure?</AlertDialogTitle>
					<AlertDialogDescription>
						You will leave the current voice channel!
					</AlertDialogDescription>
				</AlertDialogHeader>

				<div className="flex gap-2">
					<Switch />
					<Label>Don't ask me again</Label>
				</div>

				<AlertDialogFooter>
					<AlertDialogAction
						onClick={() => {
							dispatchResponse(true);
							setOpen(false);
						}}
					>
						Continue
					</AlertDialogAction>
					<AlertDialogCancel>Cancel</AlertDialogCancel>
				</AlertDialogFooter>
			</AlertDialogContent>
		</AlertDialog>
	);
};
