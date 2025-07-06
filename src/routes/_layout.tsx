import { createFileRoute, Outlet, redirect } from '@tanstack/react-router';
import { SidebarProvider } from '@/components/ui/sidebar';
import { AppSidebar } from '@/components/app-sidebar';
import { Loader } from 'lucide-react';


export const Route = createFileRoute('/_layout')({
  beforeLoad({ context }) {
    console.log("beforeload _layout");
    if (!context.app.auth.isAuthenticated) throw redirect({
      to: "/login"
    })
  },
  async loader({ context }) {
    const channelDetails = await context.app.auth.getChannelDetails(import.meta.env.VITE_DEFAULT_CHANNEL_ID);
    context.app.rtc.addRooms(channelDetails.rooms);
    return channelDetails;
  },
  async onEnter({ context }) {
    await context.app.rtc.mount();
  },
  async onLeave(match) {
    match.context.app.rtc.unmount();
  },
  component: RouteComponent,
  pendingComponent: () => (
    <div className="h-full w-full grid place-items-center">
      <Loader className="w-10 h-10" />
    </div>
  )
});

function RouteComponent() {
  return (
    <SidebarProvider>
      <AppSidebar />
      <main className="flex flex-1 flex-col gap-4 overflow-hidden">
        <Outlet />
      </main>
    </SidebarProvider>
  );
}
