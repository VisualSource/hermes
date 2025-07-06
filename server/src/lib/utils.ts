export const log = (req: Bun.BunRequest) => {
    const url = new URL(req.url);

    console.log(`[${new Date().toUTCString()}][${req.method}]: ${url.pathname}`)
}
