#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aba_avner::presentation::tauri as tauri_presentation;

fn main() {
    aba_avner::infrastructure::init_logging().expect("failed to initialize application logging");

    tauri::Builder::default()
        .manage(tauri_presentation::RunCancellationState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            tauri_presentation::run_dedupe,
            tauri_presentation::prepare_dedupe_review,
            tauri_presentation::export_dedupe_review,
            tauri_presentation::cancel_dedupe,
            tauri_presentation::is_msix_install
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
