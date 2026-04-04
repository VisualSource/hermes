import { Button } from "./button";
import { Tooltip, TooltipContent, TooltipTrigger } from "./tooltip";

export const TooltipButton = ({
	children,
	tooltip,
	...props
}: React.PropsWithChildren<
	{ tooltip: string } & React.ComponentProps<typeof Button>
>) => {
	return (
		<Tooltip>
			<TooltipTrigger render={<Button {...props}>{children}</Button>} />
			<TooltipContent>{tooltip}</TooltipContent>
		</Tooltip>
	);
};
