import { isTauri } from "@tauri-apps/api/core";
import { warn, error, info, trace } from "@tauri-apps/plugin-log";

function forward(
	level: Extract<keyof Console, "log" | "info" | "error" | "warn" | "trace">,
	logger: (message: string) => Promise<void>,
) {
	const original = console[level];
	console[level] = (message: string, ...args: unknown[]) => {
		original(message, ...args);
		logger(message).catch((err) => console.debug("[LOGGER ERROR]", err));
	};
}

if (isTauri()) {
	forward("warn", warn);
	forward("error", error);
	forward("info", info);
	forward("log", trace);
}