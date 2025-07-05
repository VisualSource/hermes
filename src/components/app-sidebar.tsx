import { GalleryVerticalEnd, Settings2 } from "lucide-react";
import { Sidebar, SidebarContent, SidebarFooter, SidebarHeader, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarRail } from "./ui/sidebar";
import { AppSidebarContent } from "./app-sidebar-content";
import { Link } from "@tanstack/react-router";
import { Button } from "./ui/button";

export const AppSidebar: React.FC = () => {
    return (
        <Sidebar>
            <SidebarHeader>
                <SidebarMenu>
                    <SidebarMenuItem>
                        <SidebarMenuButton size="lg" asChild>
                            <a href="#">
                                <div className="bg-sidebar-primary text-sidebar-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg">
                                    <GalleryVerticalEnd className="size-4" />
                                </div>
                                <div className="flex flex-col gap-0.5 leading-none">
                                    <span className="font-medium">LoadingZone: Hermes</span>
                                    <span className="">v0.1.0</span>
                                </div>
                            </a>
                        </SidebarMenuButton>
                    </SidebarMenuItem>
                </SidebarMenu>
            </SidebarHeader>
            <SidebarContent>
                <AppSidebarContent />
            </SidebarContent>
            <SidebarFooter className="border-t border-primary-foreground">
                <Button asChild size="icon">
                    <Link to="/settings">
                        <Settings2 />
                    </Link>
                </Button>
            </SidebarFooter>
            <SidebarRail />
        </Sidebar>
    );
}