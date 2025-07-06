import { password } from "bun";
import { log } from "../lib/utils";
import { schemaSignup } from "../schemas";
import { HttpError } from "../utils";
import { database } from "../lib/database";
import { sign } from "../lib/auth";
import { User } from "../model";

export default {
    POST: async (req: Bun.BunRequest) => {
        log(req);
        const body = await req.json();

        const form = schemaSignup.safeParse(body);
        if (!form.success) {
            throw new HttpError(form.error.message, { statusCode: 400 });
        }
        const { username, password: psd } = form.data;

        //https://api.dicebear.com/9.x/lorelei-neutral/svg?seed=
        const psdHash = await password.hash(psd);

        const user = await database.prepare("INSERT INTO users (id,username,psdHash,icon) VALUES (?,?,?,?) RETURNING id")
            .as(User)
            .get(crypto.randomUUID(), username, psdHash, `https://api.dicebear.com/9.x/lorelei-neutral/svg?seed=${username}&backgroundColor=${Bun.color(Array.from({ length: 3 }).map(() => Math.floor(Math.random() * (255 - 1) + 1)), "hex")}`);

        if (!user) throw new HttpError("Failed to get user");

        const [token, refreshToken] = await sign(user.id);

        return Response.json({
            token,
            refreshToken,
            user: user.toJson()
        }, { status: 201 });
    }
}