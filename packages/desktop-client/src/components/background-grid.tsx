export const Background = (props: React.PropsWithChildren) => {
	return (
		<div className="h-full w-full bg-background relative flex place-items-center place-content-center">
			{/* Dark Sphere Grid Background */}

			<div
				className="absolute inset-0 z-2"
				style={{
					backgroundImage: `
        linear-gradient(to right, rgba(71,85,105,0.3) 1px, transparent 1px),
        linear-gradient(to bottom, rgba(71,85,105,0.3) 1px, transparent 1px),
        radial-gradient(circle at 50% 50%, rgba(139,92,246,0.15) 0%, transparent 70%)
      `,
					backgroundSize: "32px 32px, 32px 32px, 100% 100%",
				}}
			/>

			{props.children}
		</div>
	);
};
