/// <reference types="vite/client" />

interface ViteTypeOptions {}

interface ImportMetaEnv {
    VITE_CLIENT_ID: string;
    VITE_SERVER_URL: string;
}

interface ImportMeta {
    readonly env: ImportMetaEnv;
}