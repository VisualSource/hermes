import { makeSubscription } from "@/lib/app";
import { useSyncExternalStore } from "react";

const { subscription, snapshot } = makeSubscription<boolean>(
	"voice-state-change",
	"inVoice",
);

export const useInVoice = () => {
	const state = useSyncExternalStore(subscription, snapshot);

	return state;
};
