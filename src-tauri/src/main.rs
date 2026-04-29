mod commands;
mod state;

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("failed to run SeatLoom Tauri shell");
}
