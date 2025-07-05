
import { SidebarMenuButton, SidebarMenuItem } from "./ui/sidebar";
import { DropdownMenu } from "./ui/dropdown-menu";
import { MoreHorizontal } from "lucide-react";
import { useNavigate } from "@tanstack/react-router";
import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
} from "@/components/ui/context-menu"
import { useRoom } from "@/hooks/use-room";
import { NamePlate } from "./UserIcon";
import { Suspense } from "react";

const roomCmds = [
    { title: "Join" }
]

export const SidebarRoomView: React.FC<{ roomId: string }> = ({ roomId }) => {
    const navigate = useNavigate();
    const room = useRoom(roomId);

    console.log(room);

    return (
        <DropdownMenu>
            <SidebarMenuItem>
                <ContextMenu>
                    <div>
                        <ContextMenuTrigger asChild>
                            <SidebarMenuButton onClick={() => navigate({ to: "/room/$id", params: { id: roomId } })} className="cursor-pointer data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground">
                                {room?.name} <MoreHorizontal className="ml-auto" />
                            </SidebarMenuButton>
                        </ContextMenuTrigger>
                        <ul className="ml-8">
                            {[...room?.users ?? []].map((userId) => (
                                <Suspense key={userId} fallback={<li className="inline-flex gap-2 items-center">


                                </li>}>
                                    <NamePlate userId={userId} />
                                </Suspense>
                            ))}
                        </ul>
                    </div>
                    <ContextMenuContent>
                        {roomCmds.map((e, i) => (
                            <ContextMenuItem key={`cmd-${i + 1}`}>{e.title}</ContextMenuItem>
                        ))}
                    </ContextMenuContent>
                </ContextMenu>
            </SidebarMenuItem>
        </DropdownMenu>
    );

}