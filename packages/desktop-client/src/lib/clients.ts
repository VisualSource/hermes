import { QueryClient } from "@tanstack/react-query";
import { OAuth } from "./auth";

export const queryClient = new QueryClient();
export const auth = new OAuth();