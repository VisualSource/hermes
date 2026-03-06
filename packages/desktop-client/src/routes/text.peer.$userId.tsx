import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/text/peer/$userId")({
  component: RouteComponent,
});

function RouteComponent() {
  return <div>Hello "/text/peer/$userId"!</div>;
}
