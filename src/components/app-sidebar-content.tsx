import { SidebarGroup, SidebarMenu } from "./ui/sidebar";
import { SidebarRoomView } from "./SidebarRoomView";
import { Route } from "../routes/_layout";

export const AppSidebarContent: React.FC = () => {
    const details = Route.useLoaderData();

    return (
        <SidebarGroup>
            <SidebarMenu>
                {details.rooms.map(room => (
                    <SidebarRoomView key={room.id} roomId={room.id} />
                ))}
            </SidebarMenu>
        </SidebarGroup>
    );
}