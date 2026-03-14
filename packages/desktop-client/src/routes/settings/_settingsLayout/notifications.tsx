import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/settings/_settingsLayout/notifications")(
  {
    component: RouteComponent,
  },
);

function RouteComponent() {
  return <div>Hello "/settings/_settingsLayout/notifications"!</div>;
}
