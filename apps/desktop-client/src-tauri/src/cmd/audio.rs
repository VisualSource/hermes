use std::str::FromStr;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    DeviceId, StreamConfig,
};
use tauri::ipc::Channel;

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum AudioError {
    #[error("Was unable to find device with given id")]
    NoDeviceFound,

    #[error("Mutex lock was poisioned")]
    Lock,

    #[error("Device id error: {0}")]
    DeviceId(String),

    #[error("Device name error: {0}")]
    Name(String),

    #[error("{0}")]
    Opus(String),

    #[error("build stream error: {0}")]
    BuildStream(String),

    #[error("puase stream error: {0}")]
    Pause(String),

    #[error("play stream error: {0}")]
    Play(String),

    #[error("devices error: {0}")]
    Device(String),
}

impl From<cpal::DeviceIdError> for AudioError {
    fn from(value: cpal::DeviceIdError) -> Self {
        Self::DeviceId(value.to_string())
    }
}

impl From<cpal::DeviceNameError> for AudioError {
    fn from(value: cpal::DeviceNameError) -> Self {
        Self::Name(value.to_string())
    }
}

impl From<opus::Error> for AudioError {
    fn from(value: opus::Error) -> Self {
        Self::Opus(value.to_string())
    }
}

impl From<cpal::BuildStreamError> for AudioError {
    fn from(value: cpal::BuildStreamError) -> Self {
        Self::BuildStream(value.to_string())
    }
}

impl From<cpal::PauseStreamError> for AudioError {
    fn from(value: cpal::PauseStreamError) -> Self {
        Self::Pause(value.to_string())
    }
}

impl From<cpal::PlayStreamError> for AudioError {
    fn from(value: cpal::PlayStreamError) -> Self {
        Self::Play(value.to_string())
    }
}

impl From<cpal::DevicesError> for AudioError {
    fn from(value: cpal::DevicesError) -> Self {
        Self::Device(value.to_string())
    }
}

pub struct AudioState {
    stream: std::sync::Mutex<Option<cpal::Stream>>,
}

impl AudioState {
    pub fn new() -> Self {
        Self {
            stream: std::sync::Mutex::new(None),
        }
    }
    pub fn start(
        &self,
        device_id: String,
        sample_rate: u32,
        on_event: Channel<Vec<u8>>,
    ) -> Result<(), AudioError> {
        let host = cpal::default_host();

        let id = DeviceId::from_str(&device_id)?;

        let device = match host.device_by_id(&id) {
            Some(d) => d,
            None => return Err(AudioError::NoDeviceFound),
        };

        let cfg = StreamConfig {
            channels: 1,
            sample_rate: 48000,
            buffer_size: cpal::BufferSize::Default,
        };

        let mut encoder =
            opus::Encoder::new(sample_rate, opus::Channels::Mono, opus::Application::Voip)?;
        let stream = device.build_output_stream(
            &cfg,
            move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                let mut buffer = Vec::default();
                match encoder.encode(data, &mut buffer) {
                    Ok(_t) => {
                        if let Err(err) = on_event.send(buffer) {
                            log::error!("{}", err);
                        };
                    }
                    Err(err) => {
                        log::error!("{}", err)
                    }
                }
            },
            move |err| {
                log::error!("{}", err);
            },
            None,
        )?;

        stream.play()?;

        match self.stream.lock() {
            Ok(mut lock) => {
                if let Some(s) = &*lock {
                    s.pause()?;
                }

                *lock = Some(stream)
            }

            Err(err) => {
                log::error!("{}", err);
                return Err(AudioError::Lock);
            }
        }

        Ok(())
    }
    pub fn destory(&self) -> Result<(), AudioError> {
        let lock = self.stream.lock();

        match lock {
            Ok(mut lock) => {
                *lock = None;

                Ok(())
            }
            Err(err) => {
                log::error!("{}", err);
                Err(AudioError::Lock)
            }
        }
    }
}

// remember to call `.manage(MyState::default())`
#[tauri::command]
pub async fn start_microphone(
    state: tauri::State<'_, AudioState>,
    device_id: String,
    on_event: Channel<Vec<u8>>,
) -> Result<(), AudioError> {
    state.start(device_id, 48000, on_event)?;

    Ok(())
}

#[tauri::command]
pub async fn stop_microphone(state: tauri::State<'_, AudioState>) -> Result<(), AudioError> {
    state.destory()?;

    Ok(())
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OutputDevices {
    devices: Vec<(String, String)>,
    default_device: Option<String>,
}

#[tauri::command]
pub async fn get_audio_inputs() -> Result<OutputDevices, AudioError> {
    let host = cpal::default_host();
    let mut outputs = Vec::default();
    let devices = host.output_devices()?;

    for device in devices {
        let description = device.description()?;
        let id = device.id()?;

        let name = description.name();

        outputs.push((id.to_string(), name.to_string()))
    }

    let default_device_id = match host.default_output_device() {
        Some(device) => {
            let id = device.id()?;
            Some(id.to_string())
        }
        None => None,
    };

    Ok(OutputDevices {
        devices: outputs,
        default_device: default_device_id,
    })
}

#[tauri::command]
pub async fn get_opus_version() -> String {
    opus::version().to_string()
}
