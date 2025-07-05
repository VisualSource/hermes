import { createFileRoute, Outlet, redirect } from '@tanstack/react-router';
import { SidebarProvider } from '@/components/ui/sidebar';
import { AppSidebar } from '@/components/app-sidebar';


export const Route = createFileRoute('/_layout')({
  beforeLoad({ context }) {
    console.log("beforeload _layout");
    if (!context.app.auth.isAuthenticated) throw redirect({
      to: "/login"
    })
  },
  async onEnter({ context }) {
    await context.app.rtc.mount();
  },
  async onLeave(match) {
    match.context.app.rtc.unmount();
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
