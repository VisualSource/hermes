
import { SidebarGroup, SidebarMenu } from "./ui/sidebar";
import { SidebarRoomView } from "./SidebarRoomView";

const items = [{
    title: "Channel Name",
    id: "91b3c4ae-e243-47e0-8124-35d8f357f211"
}]

export const AppSidebarContent: React.FC = () => {

    return (
        <SidebarGroup>
            <SidebarMenu>
                {items.map(room => (
                    <SidebarRoomView key={room.id} roomId={room.id} />
                ))}
            </SidebarMenu>
        </SidebarGroup>
    );
}