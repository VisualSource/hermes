import { createRouter, RouterProvider } from "@tanstack/react-router";
import React from "react";
import ReactDOM from "react-dom/client";

import { routeTree } from './routeTree.gen'
import Auth from "./lib/auth";

import "./index.css"
import { AuthProvider } from "./components/AuthProvider";

const router = createRouter({
  routeTree,
  context: {
    auth: null!
  }
})

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router
  }
}

const auth = new Auth();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <AuthProvider client={auth}>
      <RouterProvider router={router} context={{ auth }} />
    </AuthProvider>
  </React.StrictMode>,
);
