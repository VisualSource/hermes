import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/voice/$roomId")({
  component: RouteComponent,
});

function RouteComponent() {
  return <div>Hello "/voice/$roomId"!</div>;
}
