// LuxePOS Lite — Tauri entry point
// Évite la fenêtre console secondaire en release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    luxepos_lite_lib::run()
}
