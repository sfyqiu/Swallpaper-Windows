mod library;
mod sources;
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

#[tauri::command]
fn list_library_wallpapers() -> Result<Vec<library::LibraryWallpaper>, String> {
    library::wallpapers()
}

#[tauri::command]
async fn download_wallpaper(item: sources::WallpaperItem) -> Result<library::DownloadResult, String> {
    library::download_wallpaper(item).await
}

#[tauri::command]
fn list_wallpaper_sources() -> Vec<sources::SourceInfo> {
    sources::list_sources()
}

#[tauri::command]
async fn search_wallpapers(request: sources::SearchRequest) -> Result<sources::SearchResponse, String> {
    sources::search(request).await
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            set_static_wallpaper,
            start_video_wallpaper,
            stop_video_wallpaper,
            library_status,
            list_library_wallpapers,
            download_wallpaper,
            list_wallpaper_sources,
            search_wallpapers
        ])
        .run(tauri::generate_context!())
        .expect("error while running Swallpaper Windows");
}
