import { createRouter, RouterProvider } from "@tanstack/react-router";
import React from "react";
import ReactDOM from "react-dom/client";
import { routeTree } from './routeTree.gen'
import "./index.css"
import { App } from "./lib/app";
import { AppProvider } from "./components/AppContext";

const router = createRouter({
  routeTree,
  context: {
    // biome-ignore lint/style/noNonNullAssertion: ignore
    app: null!,
  }
})

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router
  }
}

const app = new App();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <AppProvider client={app}>
      <RouterProvider router={router} context={{ app }} />
    </AppProvider>
  </React.StrictMode>,
);
