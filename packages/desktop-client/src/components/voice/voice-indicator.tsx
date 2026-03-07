import { useEffect, useRef } from "react";

export const VoiceIndicator = () => {
	const ref = useRef<HTMLCanvasElement>(null);

	useEffect(() => {
		if (ref.current) {
			const ctx = ref.current.getContext("2d");
			if (ctx) {
				const height = ref.current.height / 2;

				ctx.moveTo(0, height);
				ctx.lineTo(ref.current.width, height);
				ctx.lineWidth = 0.5;
				ctx.strokeStyle = "black";
				ctx.stroke();
			}
		}

		return () => {};
	}, []);

	return <canvas ref={ref} className="absolute h-full w-full z-5" />;
};
