mod library;
mod scheduler;
mod sources;
mod sync;
mod wallpaper;

use std::sync::Mutex;
use wallpaper::VideoWallpaperState;

#[tauri::command]
fn set_static_wallpaper(path: String) -> Result<String, String> {
    wallpaper::set_static_wallpaper(&path)
}

#[tauri::command]
fn start_video_wallpaper(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<VideoWallpaperState>>,
    path: String,
) -> Result<String, String> {
    wallpaper::start_video_wallpaper(&app, &state, &path)
}

#[tauri::command]
fn stop_video_wallpaper(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<VideoWallpaperState>>,
) -> Result<String, String> {
    wallpaper::stop_video_wallpaper(&app, &state)
}

#[tauri::command]
fn pause_video_wallpaper(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<VideoWallpaperState>>,
) -> Result<String, String> {
    wallpaper::pause_video_wallpaper(&app, &state)
}

#[tauri::command]
fn resume_video_wallpaper(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<VideoWallpaperState>>,
) -> Result<String, String> {
    wallpaper::resume_video_wallpaper(&app, &state)
}

#[tauri::command]
fn video_wallpaper_status(
    state: tauri::State<'_, Mutex<VideoWallpaperState>>,
) -> Result<wallpaper::VideoWallpaperStatus, String> {
    wallpaper::video_wallpaper_status(&state)
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

// ---- Scheduler ----

#[tauri::command]
fn get_scheduler_config() -> scheduler::SchedulerConfig {
    scheduler::SchedulerConfig::load()
}

#[tauri::command]
fn save_scheduler_config(config: scheduler::SchedulerConfig) -> Result<String, String> {
    config.save()?;
    Ok("Scheduler config saved.".to_string())
}

#[tauri::command]
fn rotate_wallpaper(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<VideoWallpaperState>>,
) -> Result<scheduler::RotateResult, String> {
    scheduler::rotate_wallpaper(&app, &state)
}

// ---- Download queue ----

#[tauri::command]
fn get_download_queue() -> Vec<library::QueueItem> {
    library::get_queue()
}

#[tauri::command]
fn clear_download_queue() {
    library::clear_queue()
}

#[tauri::command]
async fn download_batch(items: Vec<sources::WallpaperItem>) -> Result<Vec<library::DownloadResult>, String> {
    library::download_batch(items).await
}

// ---- Cloud sync commands ----

#[tauri::command]
fn list_sync_providers() -> Vec<sync::ProviderInfo> {
    sync::list_providers()
}

#[tauri::command]
fn get_sync_config() -> sync::SyncConfig {
    sync::get_sync_config()
}

#[tauri::command]
fn get_sync_status() -> sync::SyncStatus {
    sync::get_sync_status()
}

#[tauri::command]
fn enable_cloud_sync(
    provider: String,
    provider_name: String,
    root_path: String,
    mode: String,
) -> Result<String, String> {
    sync::enable_sync(&provider, &provider_name, &root_path, &mode)
}

#[tauri::command]
fn disable_cloud_sync() -> Result<String, String> {
    sync::disable_sync()
}

#[tauri::command]
fn scan_sync_library() -> Result<sync::SyncScanResult, String> {
    sync::scan_sync_library()
}

#[tauri::command]
fn import_local_to_sync() -> Result<String, String> {
    sync::import_local_to_sync()
}

#[tauri::command]
async fn test_api_connectivity() -> Vec<sources::ApiTestResult> {
    sources::test_api_connectivity().await
}

// ---- Favorites commands ----

#[tauri::command]
fn toggle_favorite(id: String) -> Result<String, String> {
    library::toggle_favorite(&id)
}

#[tauri::command]
fn list_favorites() -> Result<Vec<library::LibraryWallpaper>, String> {
    library::list_favorites()
}

pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(VideoWallpaperState::new()))
        .invoke_handler(tauri::generate_handler![
            set_static_wallpaper,
            start_video_wallpaper,
            stop_video_wallpaper,
            pause_video_wallpaper,
            resume_video_wallpaper,
            video_wallpaper_status,
            library_status,
            list_library_wallpapers,
            download_wallpaper,
            download_batch,
            get_download_queue,
            clear_download_queue,
            list_wallpaper_sources,
            search_wallpapers,
            test_api_connectivity,
            toggle_favorite,
            list_favorites,
            get_scheduler_config,
            save_scheduler_config,
            rotate_wallpaper,
            list_sync_providers,
            get_sync_config,
            get_sync_status,
            enable_cloud_sync,
            disable_cloud_sync,
            scan_sync_library,
            import_local_to_sync,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Swallpaper Windows");
}
