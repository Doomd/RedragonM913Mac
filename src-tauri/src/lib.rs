pub mod m913;

use std::collections::HashMap;

use m913::device::M913Device;
use m913::protocol::{factory_default_states, ALL_BUTTONS, Button, ButtonState};

#[tauri::command]
fn get_factory_defaults() -> Vec<ButtonState> {
    factory_default_states()
}

#[tauri::command]
fn read_mappings() -> Result<Vec<ButtonState>, String> {
    let dev = M913Device::open()?;
    dev.read_all_buttons()
}

#[tauri::command]
fn apply_mappings(updates: HashMap<String, String>) -> Result<Vec<ButtonState>, String> {
    let mut parsed = Vec::with_capacity(updates.len());
    for (btn_id, action) in updates {
        let btn = Button::parse(&btn_id).ok_or_else(|| format!("Unknown button '{btn_id}'"))?;
        parsed.push((btn, action));
    }
    let dev = M913Device::open()?;
    dev.apply_mappings(&parsed)
}

#[tauri::command]
fn reset_all_defaults() -> Result<Vec<ButtonState>, String> {
    let updates: Vec<(Button, String)> = ALL_BUTTONS
        .iter()
        .map(|&btn| (btn, btn.factory_default().to_string()))
        .collect();
    let dev = M913Device::open()?;
    dev.apply_mappings(&updates)
}

#[tauri::command]
fn open_repo() -> Result<(), String> {
    const REPO_URL: &str = "https://github.com/Doomd/RedragonM913Mac";
    std::process::Command::new("open")
        .arg(REPO_URL)
        .spawn()
        .map_err(|e| format!("Failed to open browser: {e}"))?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_factory_defaults,
            read_mappings,
            apply_mappings,
            reset_all_defaults,
            open_repo
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
