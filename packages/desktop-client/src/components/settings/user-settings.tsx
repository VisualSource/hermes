import { Avatar, AvatarFallback, AvatarImage } from "../ui/avatar";
import { Button } from "../ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Separator } from "../ui/separator";

export const UserSettings = () => {
	return (
		<div className="flex flex-col p-2 h-full gap-2">
			<Card>
				<CardHeader>
					<CardTitle>User</CardTitle>
					<Separator />
				</CardHeader>
				<CardContent className="flex flex-col items-center justify-center">
					<Avatar className="size-24">
						<AvatarImage />
						<AvatarFallback>CN</AvatarFallback>
					</Avatar>
					<p className="text-2xl">Username</p>
					<div className="gap-2 flex mt-6">
						<Button>Edit</Button>
						<Button variant="destructive">Logout</Button>
					</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader>
					<CardTitle>Server Profiles</CardTitle>
					<Separator />
				</CardHeader>
				<CardContent></CardContent>
			</Card>
		</div>
	);
};
