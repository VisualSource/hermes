import { join } from "node:path";

const PUBLIC_DIR = join(import.meta.dir, "public");

Bun.serve({
  port: 3000,
  async fetch(req) {
    const url = new URL(req.url);
    let pathName = url.pathname;

    // Resolve the full file path safely
    const filePath = join(PUBLIC_DIR, pathName);
    const file = Bun.file(filePath);

    // Return the file if it exists, otherwise throw a 404
    if (await file.exists()) {
      return new Response(file);
    }

    return new Response("404 Not Found", { status: 404 });
  },
});

console.log("Server running at http://localhost:3000");