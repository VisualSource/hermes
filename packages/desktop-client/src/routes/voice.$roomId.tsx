import { Card, CardContent } from "@/components/ui/card";
import { createFileRoute } from "@tanstack/react-router";
import { faker } from "@faker-js/faker";
import { useState } from "react";

export const Route = createFileRoute("/voice/$roomId")({
  component: RouteComponent,
  onEnter(match){
    console.log("Init WEB RTC",match.params);
  },
  pendingComponent: ()=>(<div></div>),
  errorComponent: ()=>(<div></div>),
  remountDeps: ({ params }) => params.roomId

});

function RouteComponent() {
  const [watchingStream, setWatchingStream] = useState(true);

		if (watchingStream) {
			return (
				<div className="flex flex-col h-full p-8">
					<div className="h-full flex justify-center">
						<video className="aspect-video bg-accent">
							<track kind="captions"></track>
						</video>
					</div>
					<Card className="h-56 flex">
						<CardContent className="overflow-hidden flex">
							<div className="overflow-x-scroll flex w-full">
								{Array.from({ length: 15 }).map((_, key) => (
									<div className="h-32 w-64" key={key}>
										<img
											src={faker.image.url()}
											alt="user"
											className="h-full w-full"
										/>
									</div>
								))}
							</div>
						</CardContent>
					</Card>
				</div>
			);
		}

		return (
			<div className="h-full w-full container grid grid-cols-4 justify-center place-content-center p-8 gap-2">
				{Array.from({ length: 15 }).map((_, key) => (
					<Card key={key} className="h-36 p-0.5 w-64 mx-auto">
						<CardContent className="h-full p-0.5 relative">
							<img
								src={faker.image.url()}
								alt="user"
								className="h-full w-full"
							/>
							{key === 14 ? (
								<button
									onClick={() => setWatchingStream((e) => !e)}
									className="absolute top-0 left-0 h-full w-full bg-accent/50 border border-red-500 before:h-6 before:content-['Live'] before:bg-red-500 before:px-1 before:py-0.5 before:top-0 before:absolute before:left-0"
									type="button"
								></button>
							) : null}
						</CardContent>
					</Card>
				))}
			</div>
		);
}
