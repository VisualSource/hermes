import { password } from "bun";
import { HttpError } from "../utils";
import { sign } from "../lib/auth";
import { database } from "../lib/database";
import { schemaLogin } from "../schemas";
import { log } from "../lib/utils";
import { User } from "../model";

export default {
    POST: async (req: Bun.BunRequest<"/api/login">) => {
        log(req);
        const body = await req.json();
        const form = schemaLogin.safeParse(body);
        if (!form.success) {
            throw new HttpError(form.error.message, { statusCode: 400 });
        }
        const { username, password: psd } = form.data;

        const user = await database.prepare("SELECT * FROM users WHERE username = ?").as(User).get(username);
        if (!user) {
            throw new HttpError("No user", { statusCode: 404 });
        }

        const valid = await password.verify(psd, user.psdHash);
        if (!valid) {
            throw new HttpError("Authorized", { statusCode: 401 });
        }

        const [token, refreshToken] = await sign(user.id);

        return Response.json({
            token,
            refreshToken,
            user: user.toJson()
        });
    }
}