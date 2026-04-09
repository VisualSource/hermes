import { TooltipButton } from "@/components/ui/tooltip-button";
import type { UUID } from "node:crypto";
import { Edit2, SmilePlus } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useUser } from "@/hooks/use-user";
import { useAddReactionMutation } from "./add-reaction-mutation";

export const ButtonActions = ({ userId }: { userId: UUID }) => {
	const { t } = useTranslation();
	const user = useUser();
	const { mutateAsync } = useAddReactionMutation();

	return (
		<div className="absolute hidden group-hover:flex -top-4 right-1 border rounded-sm bg-accent">
			<TooltipButton size="icon-sm" variant="ghost" tooltip={t("AddReaction")}>
				<SmilePlus />
			</TooltipButton>
			{user.id === userId ? (
				<TooltipButton size="icon-sm" variant="ghost" tooltip={t("Edit")}>
					<Edit2 />
				</TooltipButton>
			) : null}
		</div>
	);
};
