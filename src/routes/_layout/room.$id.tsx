import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_layout/room/$id')({
  component: RouteComponent,
})

function RouteComponent() {
  return <div>Hello "/index/lobby"!</div>
}
