import { Database } from "bun:sqlite";
import { join } from "node:path";

export const database = new Database(join(import.meta.dir, "../", "server.db"));

export const initDB = async () => {
    await database.exec("CREATE TABLE IF NOT EXISTS users (id text PRIMARY KEY, username TEXT UNIQUE NOT NULL, psdHash TEXT NOT NULL, icon TEXT)");
    await database.exec(`CREATE TABLE IF NOT EXISTS channels (
                            id text PRIMARY KEY, 
                            name TEXT NOT NULL, 
                            icon NOT NULL, 
                            admins TEXT NOT NULL DEFAULT '[]', 
                            members TEXT NOT NULL DEFAULT '[]'
                        )`);
    await database.exec(`CREATE TABLE IF NOT EXISTS rooms (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            type TEXT NOT NULL DEFAULT 'text',
            channelId TEXT NOT NULL
        );`);

    await database.exec("CREATE TABLE IF NOT EXISTS messages (id TEXT PRIMARY KEY, message TEXT, user TEXT NOT NULL, roomId TEXT NOT NULL);")
}

