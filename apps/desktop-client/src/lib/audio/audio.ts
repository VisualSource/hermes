import { invoke, Channel } from '@tauri-apps/api/core';

export const stopMicrophone = async () => {
    await invoke("stop_microphone");
}

export const getOpusVersion = async () => {
	const result = await invoke<string>("get_opus_version");
	return result;
};

export const getOutputDevices = async () => {
	const results = await invoke<{
		devices: [string, string][];
		default_device: string | null;
	}>("get_audio_inputs");

	return results;
};

export const startMicrophone = async (deviceId: string) => {
    const source = new MediaSource();
    
    const { resolve, reject, promise } = await Promise.withResolvers();

    source.addEventListener("sourceopen",resolve);
    source.addEventListener("sourceclose",()=>reject);
    source.addEventListener("sourceended",()=>reject);

    const sourceBuffer = source.addSourceBuffer("audio/opus");

    await promise;

    const onMessage = new Channel<ArrayBuffer>((data)=>{
        if(!sourceBuffer.updating){
            sourceBuffer.appendBuffer(data);
        }
    });
    
    await invoke("start_microphone",{ deviceId, onMessage });

    return source;
}