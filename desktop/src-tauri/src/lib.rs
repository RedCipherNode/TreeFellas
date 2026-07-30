use treefellas_core::analysis::analyze;
use treefellas_core::filesystem;
use treefellas_core::types::ScanResult;
use treefellas_core::Scanner;

#[tauri::command]
fn get_drives() -> Vec<String> {
    filesystem::drives()
        .into_iter()
        .map(|drive| drive.name)
        .collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_drives, scan,])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn scan(path: String) -> Result<ScanResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let tree = Scanner::scan(&path).map_err(|e| e.to_string())?;

        let analysis = analyze(&tree);

        Ok(ScanResult { tree, analysis })
    })
    .await
    .map_err(|e| e.to_string())?
}
