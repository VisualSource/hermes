import { Suspense, useEffect, useRef } from "react";
import { Avatar, AvatarFallback } from "./avatar";
import { useRTCMediaStream } from "@/hooks/use-rtc-media-stream";
import { UserIcon } from "../UserIcon";
import { useAuth } from "@/hooks/useAuth";

export const AudioVisualizer: React.FC<{ userId: string }> = ({ userId }) => {
    const audioRef = useRef<HTMLAudioElement>(null);
    const mediaSource = useRTCMediaStream(userId);
    const canvas = useRef<HTMLCanvasElement>(null);
    const auth = useAuth();

    const isSelf = auth.user?.id === userId;

    useEffect(() => {
        let frame: number;
        let audioCtx: AudioContext | undefined;
        const ctx = canvas.current?.getContext("2d");

        if (ctx) {
            ctx.lineWidth = 2;
            ctx.strokeStyle = "rgb(0 0 0)";
            ctx?.beginPath();
            ctx?.moveTo(0, (canvas.current?.height ?? 0) / 2);
            ctx?.lineTo(canvas.current?.width ?? 0, (canvas.current?.height ?? 0) / 2);
            ctx.stroke();
        }

        if (mediaSource && canvas.current && ctx) {
            if (audioRef.current) {
                audioRef.current.srcObject = mediaSource;
            }

            console.log(mediaSource);
            audioCtx = new AudioContext();

            const audioSource = audioCtx.createMediaStreamSource(mediaSource);
            const analyser = audioCtx.createAnalyser();

            audioSource.connect(analyser);
            analyser.smoothingTimeConstant = 0.85;
            analyser.fftSize = 2048;

            const bufferLength = analyser.frequencyBinCount;
            const dataArray = new Uint8Array(bufferLength);

            const animate = () => {
                if (!canvas.current) return;
                analyser.getByteTimeDomainData(dataArray);

                ctx.fillStyle = "rgb(200,200,200)";
                ctx?.clearRect(0, 0, canvas.current.width, canvas.current.height);

                ctx.lineWidth = 2;
                ctx.strokeStyle = "rgb(0 0 0)";
                ctx?.beginPath();

                const sliceWidth = (canvas.current.width * 1.0) / bufferLength;
                let x = 0;

                for (let i = 0; i < bufferLength; i++) {
                    const v = dataArray[i] / 128.0;

                    const y = (v * canvas.current.height) / 2;
                    if (i === 0) {
                        ctx.moveTo(x, y);
                    } else {
                        ctx.lineTo(x, y);
                    }

                    x += sliceWidth;
                }

                ctx.lineTo(canvas.current.width, canvas.current.height / 2);
                ctx.stroke();

                frame = requestAnimationFrame(animate);
            }

            animate();
        }

        return () => {
            audioCtx?.close().catch(e => console.error(e));
            cancelAnimationFrame(frame);
        }
    }, [mediaSource]);

    return (
        <div className="aspect-video relative shadow bg-card w-52">
            <div className="absolute z-10 w-full h-full justify-center items-center flex">
                <Avatar className="h-12 w-12">
                    <Suspense>
                        <UserIcon userId={userId} />
                    </Suspense>
                    <AvatarFallback>

                    </AvatarFallback>
                </Avatar>
            </div>
            {isSelf ? null : <audio ref={audioRef} autoPlay controls={false} playsInline />}


            <canvas ref={canvas} className="h-full w-full" />
        </div>
    );
}