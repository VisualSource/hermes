import { createRouter, RouterProvider } from "@tanstack/react-router";
import React from "react";
import ReactDOM from "react-dom/client";

import { routeTree } from './routeTree.gen'
import Auth from "./lib/auth";

import "./index.css"
import { AuthProvider } from "./components/AuthProvider";
import { RTC } from "./lib/rtc";
import { RtcProvider } from "./components/RtcProvider";

const router = createRouter({
  routeTree,
  context: {
    // biome-ignore lint/style/noNonNullAssertion: ignore
    auth: null!,
    // biome-ignore lint/style/noNonNullAssertion: ignore
    rtc: null!
  }
})

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router
  }
}

const auth = new Auth();
const rtc = new RTC();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <AuthProvider client={auth}>
      <RtcProvider client={rtc}>
        <RouterProvider router={router} context={{ auth, rtc }} />
      </RtcProvider>
    </AuthProvider>
  </React.StrictMode>,
);
