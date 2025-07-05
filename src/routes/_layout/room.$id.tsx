import { AudioVisualizer } from '@/components/ui/audioVisualizer';
import { Button } from '@/components/ui/button';
import { useRoom } from '@/hooks/use-room';
import { createFileRoute, Link } from '@tanstack/react-router'


export const Route = createFileRoute('/_layout/room/$id')({
  async onEnter({ context, params }) {
    await context.app.rtc.joinRoom(params.id);
  },
  async onLeave({ context, params }) {
    await context.app.rtc.leaveRoom(params.id);
  },
  pendingComponent: () => (
    <p>Loading....</p>
  ),
  component: RouteComponent,
})

function RouteComponent() {
  const params = Route.useParams();
  const room = useRoom(params.id);

  return (
    <div className="h-full w-full relative">
      <header>
        <Button asChild>
          <Link to="/">Leave</Link>
        </Button>
      </header>
      <div className="flex flex-wrap content-center justify-center items-center gap-4 p-8 h-full w-full ">
        {[...room?.users ?? []].map((userId) => (
          <AudioVisualizer key={userId} userId={userId} />
        ))}
      </div>
    </div>
  )
}
