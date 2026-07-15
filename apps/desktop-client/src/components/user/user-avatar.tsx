
import { User2 } from "lucide-react";
import { Avatar, AvatarFallback, AvatarImage } from "../ui/avatar";

export const UserAvatar = ({
	src,
	...props
}: { src?: string } & React.ComponentProps<typeof Avatar>) => {
	return (
		<Avatar {...props}>
			<AvatarImage src={src} />
			<AvatarFallback>
				<User2 />
			</AvatarFallback>
		</Avatar>
	);
};
