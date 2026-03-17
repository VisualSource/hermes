import { createFileRoute, useBlocker } from "@tanstack/react-router";
import { StreamCard } from "@/components/voice/stream-card";
import { UserCard } from "@/components/voice/user-card";
import { useVoice } from "@/hooks/use-voice.";
import { Button } from "@/components/ui/button";
import { X } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { app } from "@/lib/clients/app";
import { confirm } from "@tauri-apps/plugin-dialog";
import { Background } from "@/components/background-grid";
export const Route = createFileRoute("/voice/$roomId")({
	component: RouteComponent,
	pendingComponent: () => <div></div>,
	errorComponent: () => <div></div>,
});



function RouteComponent() {
	const { roomId } = Route.useParams();
	const { watchingStream, items, watch, inVoice } = useVoice();

	useBlocker({
		shouldBlockFn: async ({ current, next }) => {
			if (
				inVoice &&
				current.fullPath === "/voice/$roomId" &&
				next.fullPath === "/voice/$roomId"
			) {
				if (current.params.roomId !== next.params.roomId) {
					const result = await confirm(
						"Are you sure? You will leave the current voice channel!",
						{
							kind: "info",
							title: "Switch Channel?",
							okLabel: "Yes",
							cancelLabel: "No",
						},
					);

					return result;
				}
				return false;
			}
			return false;
		},
		enableBeforeUnload: false,
		withResolver: true,
	});

	if (!inVoice) {
		return (
			<Background>
				<Card className="z-3">
					<CardContent>
						<Button variant="secondary" onClick={() => app.joinVoice(roomId)}>
							Join
						</Button>
					</CardContent>
				</Card>
			</Background>
		);
	}

	if (watchingStream) {
		return (
			<Background>
				<div className="flex flex-col h-full p-8 container gap-4 place-content-center-safe z-3">
					<div className="h-full w-full bg-accent relative">
						<video
							id={watchingStream.id}
							className="peer h-full w-full"
							style={{
								backgroundImage: `url(${watchingStream.preview})`,
								backgroundSize: "cover",
								backgroundRepeat: "no-repeat",
							}}
						>
							<track kind="captions"></track>
						</video>
						<div className="absolute top-2 right-2 opacity-0 peer-hover:opacity-100 hover:opacity-100 transition-opacity duration-200 bg-accent/60 px-2 py-1">
							{watchingStream.res}p, {watchingStream.fps}fps
						</div>
						<div className="absolute opacity-0 peer-hover:opacity-100 hover:opacity-100 transition-opacity duration-200 w-full flex justify-center-safe bottom-2 left-0">
							<Button
								variant="secondary"
								onClick={() => document.startViewTransition(() => watch(null))}
							>
								<X /> Close
							</Button>
						</div>
					</div>

					<div className="bg-card h-full p-2 flex gap-2 overflow-x-auto overflow-y-hidden items-center @container min-h-33.75 max-h-33.75">
						{items
							.filter((el) => el.id !== watchingStream.id)
							.map((item) =>
								item.type === "user" ? (
									<UserCard size="sm" key={item.id} {...item} />
								) : (
									<StreamCard
										size="sm"
										key={item.id}
										{...item}
										watch={() =>
											document.startViewTransition(() => watch(item.id))
										}
									/>
								),
							)}
					</div>
				</div>
			</Background>
		);
	}

	return (
		<Background>
			<div className="h-full w-full flex flex-wrap place-content-center-safe gap-2 p-8 container @container-[size] z-10">
				{items.map((item) =>
					item.type === "stream" ? (
						<StreamCard
							watch={() => document.startViewTransition(() => watch(item.id))}
							key={item.id}
							{...item}
						/>
					) : (
						<UserCard key={item.id} {...item} />
					),
				)}
			</div>
		</Background>
	);
}
