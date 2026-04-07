import { Bug, Maximize2, Minus, X } from "lucide-react";
import { Button } from "./ui/button"
import { getCurrentWindow } from "@tauri-apps/api/window"
import { TooltipButton } from "./ui/tooltip-button";
import { isTauri } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";

const win = isTauri()
	? getCurrentWindow()
	: {
			minimize() {
				throw new Error("Unable to run in non tauri context");
			},
			toggleMaximize() {
				throw new Error("Unable to run in non tauri context");
			},
			close() {
				throw new Error("Unable to run in non tauri context");
			},
		};

export const WindowHeader = () => {
	const { t } = useTranslation();

    return (
					<header
						data-tauri-drag-region
						className="shadow-2xl flex bg-card justify-between cursor-pointer border-b"
					>
						<div>
							<TooltipButton
								size="icon-lg"
								variant="ghost"
								tooltip={t("titlebar.BugReport")}
							>
								<Bug />
							</TooltipButton>
						</div>
						<div className="flex gap-2">
							<Button
								size="icon-lg"
								variant="ghost"
								onClick={() => win.minimize()}
							>
								<Minus />
							</Button>
							<Button
								size="icon-lg"
								variant="ghost"
								onClick={() => win.toggleMaximize()}
							>
								<Maximize2 />
							</Button>
							<Button
								size="icon-lg"
								variant="destructive"
								onClick={() => win.close()}
							>
								<X />
							</Button>
						</div>
					</header>
				);
}