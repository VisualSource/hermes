import { z } from "zod/v4";
import { socketCommand } from "hermes-shared";

export const schemaSignup = z.strictObject({
    username: z.string().min(3).max(200),
    password: z.string().min(8).max(200),
});

export const schemaLogin = z.strictObject({
    username: z.string().min(3).max(200),
    password: z.string().min(8).max(200),
});

export const schemaRefresh = z.strictObject({
    refreshToken: z.string().min(200),
});

export const schemaUuid = z.uuidv4();