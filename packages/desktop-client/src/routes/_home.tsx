import {
	Avatar,
	AvatarBadge,
	AvatarFallback,
	AvatarImage,
} from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { faker } from "@faker-js/faker";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute, Outlet, useNavigate } from "@tanstack/react-router";
import { Users2 } from "lucide-react";

export const Route = createFileRoute("/_home")({
	component: RouteComponent,
});

function RouteComponent() {
	const navigate = useNavigate();
	const { data } = useQuery({
		queryKey: ["firend-list"],
		queryFn: async () => {
			return Array.from({ length: 6 }).map(() => ({
				id: faker.string.uuid(),
				avatar: faker.image.avatarGitHub(),
				status: faker.number.int(2),
				username: faker.person.firstName(),
				statusText: faker.lorem.words({ min: 0, max: 3 }),
			}));
		},
	});

	return (
		<div className="grow grid grid-cols-12">
			<main className="col-span-10">
				<Outlet />
			</main>
			<aside className="bg-accent col-span-2">
				<div className="flex items-center py-2 px-3 bg-background shadow border-b justify-between">
					<h1 className="inline-flex gap-2 items-center">
						<Users2 className="h-4 w-4" /> Friends
					</h1>

					<div className="flex gap-1">
						<Button variant="secondary">Pending</Button>
						<Button>Add Friend</Button>
					</div>
				</div>
				<ul className="space-y-1 px-2 py-2 overflow-y-auto">
					<h1>Online -- {data?.length}</h1>
					<Separator />
					{data?.map((user) => (
						<li
							key={user.id}
							className="cursor-pointer hover:bg-accent-foreground/20 rounded-xs"
							onClick={() =>
								navigate({
									to: "/text/peer/$userId",
									params: { userId: user.id },
								})
							}
						>
							<div className="flex items-center gap-2 px-2 py-1">
								<Avatar>
									<AvatarFallback>UN</AvatarFallback>
									<AvatarImage src={user.avatar} />
									<AvatarBadge
										className={
											user.status === 0 ? "bg-green-500" : "bg-gray-500"
										}
									/>
								</Avatar>
								<div>
									<h1>{user.username}</h1>
									{user.statusText.length ? (
										<p className="text-xs text-muted-foreground">
											{user.statusText}
										</p>
									) : null}
								</div>
							</div>
						</li>
					))}
				</ul>
			</aside>
		</div>
	);
}
