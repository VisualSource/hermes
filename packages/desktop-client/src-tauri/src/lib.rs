use tauri::Emitter;

mod cmd;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout
                ))
                .level(tauri_plugin_log::log::LevelFilter::Debug)
                .build(),
        )
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if let Some(request) = args.get(1) {
                match tauri::Url::parse(request) {
                    Ok(url) => {
                        let target = url.domain().unwrap_or_default();
                        match target {
                            "oauth" => {
                                if let Err(err) = app.emit("hermes://auth", cmd::auth::AuthFlowEvent::Done { url: url.to_string() }) {
                                    log::error!("{}",err);
                                }
                            }
                            _ => {
                                log::debug!("{:#?}",url)


                            }
                        }
                    }
                    Err(err) => {
                        log::error!("{}",err);
                    }
                }
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_upload::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app|{
            #[cfg(any(windows, target_os = "linux"))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                app.deep_link().register_all()?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![cmd::auth::start_auth_grant])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
