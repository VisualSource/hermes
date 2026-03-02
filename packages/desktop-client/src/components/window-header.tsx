import {  Maximize2, X } from "lucide-react";
import { Button } from "./ui/button"
import { getCurrentWindow } from "@tauri-apps/api/window"

const win = getCurrentWindow();

export const WindowHeader = () => {
    return (
        <header data-tauri-drag-region className="h-8 shadow flex justify-end">
            <Button onClick={()=>win.toggleMaximize()}>
                <Maximize2/>
            </Button>
            <Button variant="destructive" onClick={()=>win.close()}>
                <X/>
            </Button>
        </header>
    )
}