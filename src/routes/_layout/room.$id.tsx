import Squares from '@/components/squares';
import { AudioVisualizer } from '@/components/ui/audioVisualizer';
import { useRTC } from '@/hooks/useRtc';
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_layout/room/$id')({
  async loader({ context, params }) {
    await context.rtc.joinLobby(params.id);
  },
  onLeave({ context, params }) {
    context.rtc.leaveLobby(params.id);
  },
  pendingComponent: () => (
    <p>Loading....</p>
  ),
  component: RouteComponent,
})

function RouteComponent() {
  const ctx = useRTC();

  return (
    <div className="h-full w-full relative">
      <Squares speed={0.2}
        squareSize={40}
        direction='diagonal'
      />
      <div className="flex flex-wrap content-center justify-center items-center gap-4 p-8 h-full absolute z-10 w-full top-0 left-0">
        {ctx.localMedia ? <AudioVisualizer mediaSource={ctx.localMedia} icon='' /> : null}
      </div>
    </div>
  )
}
