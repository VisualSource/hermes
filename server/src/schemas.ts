import { z } from "zod/v4";

export const schemaSignup = z.strictObject({
    username: z.string().min(3).max(200),
    password: z.string().min(8).max(200),
});

export const schemaLogin = z.strictObject({
    username: z.string().min(3).max(200),
    password: z.string().min(8).max(200),
});