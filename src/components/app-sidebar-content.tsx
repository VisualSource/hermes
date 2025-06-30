
import { SidebarGroup, SidebarMenu, SidebarMenuButton, SidebarMenuItem, useSidebar } from "./ui/sidebar";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "./ui/dropdown-menu";
import { MoreHorizontal } from "lucide-react";
import { useNavigate } from "@tanstack/react-router";
import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
} from "@/components/ui/context-menu"

const items = [{
    title: "Channel Name",
    items: [{ title: "Some Action", url: "" }]
}]

export const AppSidebarContent: React.FC = () => {
    const navigate = useNavigate();

    return (
        <SidebarGroup>
            <SidebarMenu>
                {items.map((item) => (
                    <DropdownMenu key={item.title}>
                        <SidebarMenuItem>
                            <ContextMenu>
                                <ContextMenuTrigger asChild>
                                    <SidebarMenuButton onClick={() => navigate({ to: "/room/$id", params: { id: "some-id" } })} className="cursor-pointer data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground">
                                        {item.title} <MoreHorizontal className="ml-auto" />
                                    </SidebarMenuButton>
                                </ContextMenuTrigger>
                                <ContextMenuContent>
                                    {item.items.map((e, i) => (
                                        <ContextMenuItem key={i}>{e.title}</ContextMenuItem>
                                    ))}
                                </ContextMenuContent>
                            </ContextMenu>
                        </SidebarMenuItem>
                    </DropdownMenu>
                ))}
            </SidebarMenu>
        </SidebarGroup>
    );
}