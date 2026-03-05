import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/voice/$roomId")({
  component: RouteComponent,
  onEnter(match){
    console.log("Init WEB RTC",match.params);
  },
  pendingComponent: ()=>(<div></div>),
  errorComponent: ()=>(<div></div>),
  remountDeps: ({ params }) => params.roomId

});

function RouteComponent() {
  return <div>Hello "/voice/$roomId"!</div>;
}
