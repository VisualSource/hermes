import {
	Avatar,
	AvatarBadge,
	AvatarFallback,
	AvatarImage,
} from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { friendListOptions } from "@/lib/api/queries";
import { getStatusColor } from "@/lib/api/types";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute, Outlet, useNavigate } from "@tanstack/react-router";
import { Users2 } from "lucide-react";
import { useTranslation } from "react-i18next";

export const Route = createFileRoute("/_home")({
	component: RouteComponent,
});

function RouteComponent() {
	const { t } = useTranslation();
	const navigate = useNavigate();
	const { data } = useQuery(friendListOptions());

	return (
		<div className="grow grid grid-cols-12">
			<main className="col-span-10">
				<Outlet />
			</main>
			<aside className="bg-accent col-span-2">
				<div className="flex items-center py-2 px-3 bg-background shadow border-b justify-between">
					<h1 className="inline-flex gap-2 items-center">
						<Users2 className="h-4 w-4" /> {t("home.Friends")}
					</h1>

					<div className="flex gap-1">
						<Button variant="secondary">{t("home.Pending")}</Button>
						<Dialog>
							<DialogTrigger render={<Button>{t("home.AddFriend")}</Button>} />
							<DialogContent>
								<DialogHeader>
									<DialogTitle>{t("home.AddFriend")}</DialogTitle>
								</DialogHeader>
								<div className="flex flex-col gap-2">
									<Input placeholder={t("home.searchUser")} />
									<ul></ul>
								</div>
							</DialogContent>
						</Dialog>
					</div>
				</div>
				<ul className="space-y-1 px-2 py-2 overflow-y-auto">
					<h1>
						{t("home.Online")} -- {data?.length}
					</h1>
					<Separator />
					{data?.map((user) => (
						// biome-ignore lint/a11y/useKeyWithClickEvents: ignore
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
									<AvatarBadge className={getStatusColor(user.status)} />
								</Avatar>
								<div>
									<h1>{user.username}</h1>
									{user.statusText?.length ? (
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
