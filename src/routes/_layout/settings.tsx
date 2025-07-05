import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { createFileRoute } from '@tanstack/react-router'
import { Loader } from 'lucide-react';
import { useState } from 'react';
import { toast } from 'sonner';

export const Route = createFileRoute('/_layout/settings')({
  component: RouteComponent,
  pendingComponent: () => {
    return <div className='h-full w-full grid place-items-center'>
      <Loader className="animate-spin h-28 w-28" />
    </div>
  },
  loader: async () => {
    await navigator.mediaDevices.getUserMedia({ audio: true, video: true });
    const devices = await navigator.mediaDevices.enumerateDevices();

    console.log(devices);

    const items = Object.groupBy(devices, (e) => e.kind);

    let defaultDevices = {
      mic: items.audioinput?.find(e => e.deviceId === "default")?.deviceId,
      camera: items.videoinput?.find(e => e.deviceId === "default")?.deviceId
    }

    const value = localStorage.getItem("diviceInputSettings");
    if (value) {
      defaultDevices = {
        ...defaultDevices,
        ...JSON.parse(value) as { mic: string; camera: string }
      }
    }

    return {
      devices: items,
      defaults: defaultDevices
    }
  },
})

function RouteComponent() {
  const { devices, defaults } = Route.useLoaderData();
  const [mic, setMic] = useState(defaults.mic);
  const [camera, setCamera] = useState(defaults.camera);


  return (
    <div className="grid place-items-center h-full w-full">
      <Card className="w-96">
        <CardHeader>
          <CardTitle>Settings</CardTitle>
        </CardHeader>
        <CardContent>
          <form className="w-full flex flex-col gap-4" action={() => {
            localStorage.setItem("diviceInputSettings", JSON.stringify({ mic: mic, camera: defaults.camera }));
            toast("Saved");
          }}>

            <div className='flex flex-col gap-2'>
              <Label>Mic</Label>
              <Select value={mic} onValueChange={setMic}>
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Theme" />
                </SelectTrigger>
                <SelectContent>
                  {devices.audioinput?.map(e => (
                    <SelectItem key={e.deviceId} value={e.deviceId}>
                      {e.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className='flex flex-col gap-2'>
              <Label>Camera</Label>
              <Select value={camera} onValueChange={setCamera}>
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Theme" />
                </SelectTrigger>
                <SelectContent>
                  {devices.videoinput?.map(e => (
                    <SelectItem key={e.deviceId} value={e.deviceId}>
                      {e.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className='flex justify-end'>
              <Button className="cursor-pointer" type="submit" size="sm">Save</Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
