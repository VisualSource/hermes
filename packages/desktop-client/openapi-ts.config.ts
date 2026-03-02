import { defineConfig } from "@hey-api/openapi-ts";

export default defineConfig({
    input: "../../api/openapi.yaml",
    output: {
        path: "./src/api",
        postProcess: ["biome:format"]
    },
    plugins: [
        "@hey-api/client-fetch",
        "@tanstack/react-query"
    ]
});