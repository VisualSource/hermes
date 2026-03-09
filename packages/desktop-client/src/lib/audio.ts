import { invoke, Channel } from '@tauri-apps/api/core';

export const stopMicrophone = async () => {
    await invoke("stop_microphone");
}

/*
To stream audio from
cpal to Opus, you need to use the cpal crate to capture raw audio samples (PCM data) and the audiopus (or similar libopus bindings) crate to encode these samples into Opus format. 
Here is a general outline of the process and key libraries:
Key Crates

    cpal: A low-level, cross-platform library for handling audio input/output streams.
    audiopus: Bindings for the libopus C library, which provides robust and efficient Opus encoding/decoding functionality.
    hound / ogg: Libraries for handling audio file containers, which you may need if you plan to save the stream to a file (Opus streams are often wrapped in an Ogg container). 

Steps and Concepts

    Configure cpal for Audio Capture:
        Initialize a cpal Host and select a default input Device.
        Configure the stream using a specific sample rate and number of channels that the Opus encoder expects (e.g., 48000 Hz, mono or stereo).
        Set up an input stream with a data callback function that receives raw PCM audio data.
    Initialize the Opus Encoder:
        Use audiopus to create an Encoder instance with the same configuration as your cpal stream (sample rate, channels, application type, etc.).
    Process Audio in the cpal Callback:
        Inside the cpal input stream's data callback, you will receive a buffer of raw PCM samples (often f32 or i16).
        Pass this raw data to the audiopus encoder's encode method. This function will output an Opus packet.
        The cpal documentation and examples on GitHub can guide you on handling the data buffers.
    Stream the Encoded Data:
        Once you have the Opus packet (a Vec<u8> or slice of bytes), you can send it over a network (e.g., using WebSockets, UDP, or gRPC).
        If saving to a file, you'll need an Ogg container library to write the packets correctly, as raw Opus packets are not a playable file format on their own.
    Handle Resampling/Format Mismatches:
        If your system's default audio device cannot provide audio in the exact format required by the Opus encoder (e.g., wrong sample rate), you may need an external library like rubato to resample the audio. 

*/

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