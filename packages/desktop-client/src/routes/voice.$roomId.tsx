import { Card, CardContent } from "@/components/ui/card";
import { createFileRoute } from "@tanstack/react-router";
import { faker } from "@faker-js/faker";
import { useState } from "react";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { X } from "lucide-react";
import type { CardType } from "@/components/voice/types";
import { StreamCard } from "@/components/voice/stream-card";
import { UserCard } from "@/components/voice/user-card";

export const Route = createFileRoute("/voice/$roomId")({
  component: RouteComponent,
  onEnter(match){
    console.log("Init WEB RTC",match.params);
  },
  pendingComponent: ()=>(<div></div>),
  errorComponent: ()=>(<div></div>),
  remountDeps: ({ params }) => params.roomId

});


const items: CardType[] = [
	{
		type: "stream",
		preview: faker.image.urlPicsumPhotos({ blur: 2 }),
		id: faker.string.ulid(),
	},
	{
		type: "stream",
		preview: faker.image.urlPicsumPhotos({ blur: 2 }),
		id: faker.string.ulid(),
	},
	...(Array.from({ length: 6 }).map(() => ({
		type: "user",
		id: faker.string.uuid(),
		avatar: faker.image.avatarGitHub(),
		username: faker.person.firstName(),
		color: faker.color.human(),
	})) as CardType[]),
];

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

					<div className="bg-card h-full p-2 flex gap-2 overflow-x-auto overflow-y-hidden items-center @container">
						{items.slice(1).map((item) =>
							item.type === "user" ? (
								<UserCard size="sm" key={item.id} {...item} />
							) : (
								<StreamCard
									size="sm"
									key={item.id}
									{...item}
									watch={() => {
										document.startViewTransition(() => {
											setWatchingStream(true);
										});
									}}
								/>
							),
						)}
					</div>
				</div>
			</div>
		);
	}

	return (
		<div className="h-full w-full flex place-content-center">
			<div className="h-full w-full flex flex-wrap place-content-center justify-center gap-2 p-8 container @container">
				{items.map((item) =>
					item.type === "stream" ? (
						<StreamCard
							watch={() => {
								document.startViewTransition(() => {
									setWatchingStream(true);
								});
							}}
							key={item.id}
							{...item}
						/>
					) : (
						<UserCard key={item.id} {...item} />
					),
				)}
			</div>
		</div>
	);
}
