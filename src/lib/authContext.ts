import { createContext } from "react";
import Auth from "./auth";

export const AuthContext = createContext<Auth | null>(null);