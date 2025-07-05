import { createContext } from "react";
import type { App } from "./app";

export const AppContext = createContext<App | null>(null);