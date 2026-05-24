mod library;
mod wallpaper;

#[tauri::command]
fn set_static_wallpaper(path: String) -> Result<String, String> {
    wallpaper::set_static_wallpaper(&path)
}

#[tauri::command]
fn start_video_wallpaper(path: String) -> Result<String, String> {
    wallpaper::start_video_wallpaper(&path)
}

#[tauri::command]
fn stop_video_wallpaper() -> Result<String, String> {
    wallpaper::stop_video_wallpaper()
}

#[tauri::command]
fn library_status() -> library::LibraryStatus {
    library::status()
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            set_static_wallpaper,
            start_video_wallpaper,
            stop_video_wallpaper,
            library_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running Swallpaper Windows");
}
