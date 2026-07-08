mod launcher;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(launcher::GameStopSignal::default())
        .manage(launcher::CrashReportState::default())
        .manage(launcher::GameProcessState::default())
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();

            #[cfg(target_os = "macos")]
            main_window.set_title_bar_style(tauri::TitleBarStyle::Overlay)?;

            let oobe_needed = !launcher::check_oobe_completed().unwrap_or(false);

            if oobe_needed {
                {
                    use tauri::WebviewWindowBuilder;
                    let oobe_window = WebviewWindowBuilder::new(
                        app,
                        "oobe",
                        tauri::WebviewUrl::App("index.html".into()),
                    )
                    .title("Firefile Launcher - 初始化设置")
                    .inner_size(700.0, 560.0)
                    .resizable(false)
                    .center()
                    .build()?;

                    #[cfg(target_os = "macos")]
                    oobe_window.set_title_bar_style(tauri::TitleBarStyle::Overlay)?;
                    #[cfg(target_os = "linux")]
                    oobe_window.set_title_bar_style(tauri::TitleBarStyle::Overlay)?;
                    #[cfg(not(target_os = "macos"))]
                    let _ = oobe_window;
                }

                main_window.hide()?;
            } else {
                main_window.show()?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            launcher::launch_minecraft,
            launcher::get_minecraft_versions,
            launcher::get_fabric_versions,
            launcher::get_forge_versions,
            launcher::get_java_versions,
            launcher::get_system_memory,
            launcher::init_oobe_environment,
            launcher::save_oobe_settings,
            launcher::rollback_oobe,
            launcher::check_oobe_completed,
            launcher::finish_oobe,
            launcher::install_instance,
            launcher::get_instances_list,
            launcher::get_oobe_settings,
            launcher::get_minecraft_dir_string,
            launcher::get_current_account,
            launcher::get_accounts,
            launcher::add_account,
            launcher::remove_account,
            launcher::stop_game,
            launcher::get_instance_settings,
            launcher::save_instance_settings,
            launcher::check_game_running,
            launcher::open_crash_shell,
            launcher::get_crash_report,
            launcher::clear_crash_report,
            launcher::open_log_folder,
            launcher::export_crash_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
