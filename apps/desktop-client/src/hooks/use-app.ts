import { create } from "zustand";

export type UUID = "";

type Store = {
    inVoice: boolean;
    activeServerId: UUID | null;

    setInVoice(state: boolean): void
}

export const useApp = create<Store>((set)=>({
    inVoice: false,
    activeServerId: null,

    setInVoice(state: boolean){
        set({ inVoice: state });
    }
}));