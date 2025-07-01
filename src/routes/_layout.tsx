import { createFileRoute, Outlet, redirect } from '@tanstack/react-router';
import { SidebarProvider } from '@/components/ui/sidebar';
import { AppSidebar } from '@/components/app-sidebar';


export const Route = createFileRoute('/_layout')({
  beforeLoad({ context }) {
    console.log("beforeload _layout");
    if (!context.auth.isAuthenticated) throw redirect({
      to: "/login"
    })
  },
  async loader({ context }) {
    await context.rtc.mount();
  },
  onLeave(match) {
    match.context.rtc.unmount();
  },
  component: RouteComponent,
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
