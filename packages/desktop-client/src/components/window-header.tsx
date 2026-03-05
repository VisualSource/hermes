import { Maximize2, Minus, X } from "lucide-react";
import { Button } from "./ui/button"
import { getCurrentWindow } from "@tauri-apps/api/window"

const win = window.__TAURI_INTERNALS__ ? getCurrentWindow() : { minimize(){}, toggleMaximize(){}, close(){} };

export const WindowHeader = () => {
    return (
					<header
						data-tauri-drag-region
						className="shadow-2xl flex justify-end bg-card cursor-pointer border-b"
					>
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
					</header>
				);
}