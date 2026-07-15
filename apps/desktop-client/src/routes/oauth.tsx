import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/oauth")({
	component: RouteComponent,
	validateSearch(params: Record<string, string>) {
		if (
			"code" in params &&
			typeof params.code === "string" &&
			params.code.length > 1
		) {
			return {
				code: params.code,
			};
		}

		return {};
	},

	beforeLoad(ctx) {
		if (!ctx.search.code) {
			throw redirect({ to: "/" });
		}

		const channel = new BroadcastChannel("oauth");
		channel.postMessage(window.location.href);

		window.close();
	},
});

function RouteComponent() {
	return <div>Hello "/oauth"!</div>;
}
