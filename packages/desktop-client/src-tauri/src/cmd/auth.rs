use tauri::{Emitter, Runtime, WebviewWindowBuilder, WindowEvent};

#[derive(Debug, serde::Serialize, Clone)]
#[serde(tag = "type")]
pub enum AuthFlowEvent {
    Cancel,
    Done { url: String },
    Error { reason: String },
}

#[tauri::command]
pub async fn start_auth_grant<R: Runtime>(
    app: tauri::AppHandle<R>,
    path: String,
) -> Result<(), tauri::Error> {
    let url = tauri::Url::parse(&path).map_err(|err| tauri::Error::Anyhow(err.into()))?;

    let mut config = app
        .config()
        .app
        .windows
        .get(1)
        .ok_or_else(|| tauri::Error::WindowNotFound)?
        .to_owned();
    config.url = tauri::WebviewUrl::External(url);
    let window = WebviewWindowBuilder::from_config(&app, &config)?.build()?;

    let handle = app.clone();
    window.on_window_event(move |ev| {
        if let WindowEvent::Destroyed = ev {
            if let Err(err) = handle.emit("hermes://auth", AuthFlowEvent::Cancel) {
                log::error!("{}", err);
            }
        }
    });

    Ok(())
}
