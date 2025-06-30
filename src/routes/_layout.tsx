import { AppSidebar } from '@/components/app-sidebar';
import { RtcProvider } from '@/components/RtcProvider';
import { SidebarProvider } from '@/components/ui/sidebar';
import { RTC } from '@/lib/rtc';
import { createFileRoute, Outlet, redirect } from '@tanstack/react-router';

const rtc = new RTC();
export const Route = createFileRoute('/_layout')({
  beforeLoad(ctx) {
    if (!ctx.context.auth.isAuthenticated) {
      throw redirect({
        to: "/login"
      })
    }
  },
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <RtcProvider client={rtc}>
      <SidebarProvider>
        <AppSidebar />
        <main className="flex flex-1 flex-col gap-4 p-4">
          <Outlet />
        </main>
      </SidebarProvider>
    </RtcProvider>
  );
}
