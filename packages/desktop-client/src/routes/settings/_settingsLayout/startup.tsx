import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/settings/_settingsLayout/startup")({
  component: RouteComponent,
});

function RouteComponent() {
  return <div>Hello "/settings/_settingsLayout/startup"!</div>;
}
