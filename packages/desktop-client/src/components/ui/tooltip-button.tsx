import { Button, type ButtonProps } from "./button";
import { Tooltip, TooltipContent, TooltipTrigger } from "./tooltip";

export const TooltipButton = ({
	children,
	tooltip,
	...props
}: React.PropsWithChildren<{ tooltip: string } & ButtonProps>) => {
	return (
		<Tooltip>
			<TooltipTrigger render={<Button {...props}>{children}</Button>} />
			<TooltipContent>{tooltip}</TooltipContent>
		</Tooltip>
	);
};
