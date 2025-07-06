import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { UserAvatar } from '@/components/UserIcon';
import { useAuth } from '@/hooks/useAuth';
import { useRTC } from '@/hooks/useRtc';
import { cn } from '@/lib/utils';
import { createFileRoute } from '@tanstack/react-router'
import { Loader, Send } from 'lucide-react'
import { Suspense, useEffect, useState } from 'react';

export const Route = createFileRoute('/_layout/room/text/$id')({
  component: RouteComponent,
  async onEnter({ context, params }) {
    await context.app.rtc.joinTextRoom(params.id);
  },
  async onLeave({ context, params }) {
    await context.app.rtc.leaveTextRoom(params.id);
  },
  loader: async ({ context, params }) => {
    const data = await context.app.auth.getTextRoomMessages(params.id);
    return data;
  },
  pendingComponent: () => (
    <div className="h-full w-full grid place-items-center">
      <Loader className="h-24 w-24" />
    </div>
  )
})

function RouteComponent() {
  const rtc = useRTC();
  const auth = useAuth();
  const msg = Route.useLoaderData();
  const { id } = Route.useParams();
  const [currentMessages, setCurrentMessages] = useState<{ message: string, user: string, id: string; }[]>([]);

  const userId = auth.user?.id;

  useEffect(() => {
    const handler = (ev: Event) => {
      const event = (ev as CustomEvent<{ type: "text", data: { message: string; user: string, id: string; } }>).detail;
      if (event.type !== "text") return;

      setCurrentMessages((d) => [...d, event.data])
    }

    rtc.addEventListener(`room::${id}::update`, handler);

    return () => {
      rtc.removeEventListener(`room::${id}::update`, handler);
    }
  }, [id, rtc.addEventListener, rtc.removeEventListener]);

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <div className="overflow-y-scroll overflow-x-hidden flex-1 scrollbar-custom py-4 px-8 gap-4 flex flex-col">
        {(msg as never as { message: string; user: string, id: string; }[]).map((msg) => (
          <div key={msg.id} className={cn("flex w-full shadow border bg-secondary-foreground border-card p-2 gap-2 items-center flex-row-reverse", { "flex-row": msg.user === userId })}>
            <div className='w-full text-ms text-muted-foreground'>
              {msg.message}
            </div>
            <Suspense>
              <UserAvatar userId={msg.user} />
            </Suspense>
          </div>
        ))}
        {currentMessages.map((msg) => (
          <div key={msg.id} className={cn("flex w-full bg-secondary-foreground shadow border border-card p-2 gap-2 items-center flex-row-reverse", { "flex-row": msg.user === userId })}>
            <div className='w-full text-ms text-muted-foreground'>
              {msg.message}
            </div>
            <Suspense>
              <UserAvatar userId={msg.user} />
            </Suspense>
          </div>
        ))}
      </div>
      <form className="flex gap-2 p-2" action={(ev) => {
        const message = ev.get("message")?.toString();
        if (!message || message.length === 0 || !userId) return;

        setCurrentMessages(e => [...e, { id: crypto.randomUUID(), message, user: userId }])

        rtc.sendTextMessage(message);
      }}>
        <Textarea placeholder='message...' name="message"></Textarea>
        <Button type="submit" className="cursor-pointer"><Send /> Send</Button>
      </form>
    </div>
  );
}
