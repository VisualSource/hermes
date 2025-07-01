export class HttpError extends Error {
    statusCode: number;
    statusText: string | undefined;
    constructor(message: string, opts?: { cause?: unknown, statusCode: number, statusText?: string }) {
        super(message, { cause: opts?.cause });
        this.statusCode = opts?.statusCode ?? 500;
        this.statusText = opts?.statusText;
    }
}