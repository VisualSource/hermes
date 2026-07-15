import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/_text/roles/$roomId")({
  component: RouteComponent,
});

function RouteComponent() {
  return <div>Hello "/roles/$roomId"!</div>;
}
