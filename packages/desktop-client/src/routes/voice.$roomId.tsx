import { Card, CardContent } from "@/components/ui/card";
import { createFileRoute } from "@tanstack/react-router";
import { faker } from "@faker-js/faker";
import { useEffect, useRef, useState } from "react";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { X } from "lucide-react";

export const Route = createFileRoute("/voice/$roomId")({
  component: RouteComponent,
  onEnter(match){
    console.log("Init WEB RTC",match.params);
  },
  pendingComponent: ()=>(<div></div>),
  errorComponent: ()=>(<div></div>),
  remountDeps: ({ params }) => params.roomId

});


const items = [
	{
		type: "stream",
		preview: faker.image.urlPicsumPhotos({ blur: 8 }),
		id: faker.string.ulid(),
	},
	...Array.from({ length: 5 }).map(() => ({
		type: "user",
		id: faker.string.uuid(),
		avatar: faker.image.avatar(),
		username: faker.person.firstName(),
		color: faker.color.human(),
	})),
] as (
	| { type: "stream"; preview: string; id: string }
	| {
			type: "user";
			id: string;
			avatar: string;
			username: string;
			color: string;
	  }
)[];

const VoiceIndicator = () => {
	const ref = useRef<HTMLCanvasElement>(null);

	useEffect(() => {
		if (ref.current) {
			const ctx = ref.current.getContext("2d");
			if (ctx) {
				const height = ref.current.height / 2;

				ctx.moveTo(0, height);
				ctx.lineTo(ref.current.width, height);
				ctx.lineWidth = 0.5;
				ctx.strokeStyle = "darkgray";
				ctx.stroke();
			}
		}

		return () => {};
	}, []);

	return <canvas ref={ref} className="absolute h-full w-full z-5" />;
};

function RouteComponent() {
  const [watchingStream, setWatchingStream] = useState(false);

		if (watchingStream) {
			return (
				<div className="h-full w-full flex justify-center">
					<div className="flex flex-col h-full p-8 container gap-4">
						<div className="relative aspect-video">
							<video
								id={items[0].id}
								className="aspect-video bg-sidebar h-full w-full peer"
								style={{
									backgroundImage: `url(${items[0].type === "stream" ? items[0].preview : ""})`,
									backgroundSize: "cover",
									backgroundRepeat: "no-repeat",
								}}
							>
								<track kind="captions"></track>
							</video>
							<div className="opacity-0 peer-hover:opacity-100 flex absolute top-2 right-2 bg-accent/60 px-3 py-1.5 transition-opacity duration-150 backdrop-blur-md">
								720p, 30fps
							</div>
							<div className="opacity-0 peer-hover:opacity-100 hover:opacity-100 flex transition-opacity duration-150 absolute bottom-2 w-full justify-center">
								<Button
									size="lg"
									onClick={() => {
										document.startViewTransition(() => {
											setWatchingStream(false);
										});
									}}
									variant="destructive"
								>
									<X /> Close
								</Button>
							</div>
						</div>

						<div className="bg-card h-full p-2 flex gap-2 overflow-x-auto">
							{items.slice(1).map((user) => (
								<Card
									key={user.id}
									className="h-full w-52 shrink-0"
									style={{
										backgroundColor:
											user.type === "user" ? user.color : undefined,
									}}
								>
									<CardContent className="flex place-content-center place-items-center h-full w-full">
										<Avatar className="size-10">
											<AvatarImage
												src={user.type === "user" ? user.avatar : ""}
												alt="username"
											/>
											<AvatarFallback>UN</AvatarFallback>
										</Avatar>
									</CardContent>
								</Card>
							))}
						</div>
					</div>
				</div>
			);
		}

		return (
			<div className="h-full w-full flex place-content-center">
				<div className="h-full w-full grid grid-cols-3 grid-rows-3 place-content-center justify-center gap-2 p-8 container">
					{items.map((item, i) => (
						<Card
							id={item.id}
							key={item.id}
							className={cn(
								"p-0.5 col-span-1 row-span-1 group shadow-2xl",
								i === items.length - 1 && items.length % 2 !== 0
									? " col-start-2 col-end-3"
									: "col-span-1",
							)}
						>
							<CardContent
								className="h-full p-0.5 relative overflow-hidden flex justify-center items-center"
								style={{
									backgroundColor:
										item.type === "user" ? item.color : undefined,
									backgroundImage:
										item.type === "stream" ? `url(${item.preview})` : undefined,
									backgroundSize: "cover",
									backgroundRepeat: "no-repeat",
								}}
							>
								{item.type === "user" ? (
									<div className="hidden absolute bottom-1 left-1 group-hover:flex bg-accent/60 px-2 py-1 transition-all duration-100 z-10">
										{item.username}
									</div>
								) : null}
								<div className="backdrop-blur-xl h-full w-full absolute" />

								{item.type === "user" ? <VoiceIndicator /> : null}

								{item.type === "user" ? (
									<Avatar className="size-20">
										<AvatarImage src={item.avatar} alt={item.username} />
										<AvatarFallback>UN</AvatarFallback>
									</Avatar>
								) : null}

								{item.type === "stream" ? (
									<button
										onClick={() => {
											document.startViewTransition(() => {
												setWatchingStream(true);
											});
										}}
										className="absolute top-0 left-0 h-full w-full flex justify-center items-center bg-accent/50 border border-red-500 before:h-6 before:content-['Live'] before:bg-red-500 before:px-1 before:py-0.5 before:top-0 before:absolute before:left-0"
										type="button"
									>
										<div className="px-4 py-2 bg-background/60 font-medium">
											Watch
										</div>
									</button>
								) : null}
							</CardContent>
						</Card>
					))}
				</div>
			</div>
		);
}
