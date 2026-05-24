use std::sync::Mutex;

#[cfg(windows)]
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoWallpaperStatus {
    pub active: bool,
    pub paused: bool,
    pub monitor_count: u32,
    pub current_path: Option<String>,
}

#[derive(Debug)]
pub struct MonitorVideoHost {
    pub monitor_index: u32,
    pub window_label: String,
}

pub struct VideoWallpaperState {
    pub active: bool,
    pub paused: bool,
    pub current_path: Option<String>,
    pub monitors: Vec<MonitorVideoHost>,
}

impl VideoWallpaperState {
    pub fn new() -> Self {
        Self {
            active: false,
            paused: false,
            current_path: None,
            monitors: Vec::new(),
        }
    }
}

// ---- Static wallpaper ----

#[cfg(windows)]
pub fn set_static_wallpaper(path: &str) -> Result<String, String> {
    use std::iter::once;
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPIF_SENDWININICHANGE, SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER,
    };

    let wide: Vec<u16> = path.encode_utf16().chain(once(0)).collect();
    unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(PCWSTR(wide.as_ptr()).0 as *mut _),
            SPIF_UPDATEINIFILE | SPIF_SENDWININICHANGE,
        )
    }
    .map(|_| format!("Static wallpaper set: {path}"))
    .map_err(|error| format!("Windows rejected the wallpaper path: {error}"))
}

#[cfg(not(windows))]
pub fn set_static_wallpaper(_path: &str) -> Result<String, String> {
    Err("Static wallpaper command is only available on Windows.".to_string())
}

// ---- Video wallpaper ----

#[cfg(windows)]
pub fn start_video_wallpaper(
    app: &AppHandle,
    state: &Mutex<VideoWallpaperState>,
    path: &str,
) -> Result<String, String> {
    stop_video_wallpaper_inner(app, state)?;

    let label = "swallpaper-video-0";

    let url = format!(
        "video.html?path={}",
        urlencoding::encode(path)
    );

    let window = WebviewWindowBuilder::new(app, label, WebviewUrl::App(url.into()))
        .title("SwallpaperVideo")
        .decorations(false)
        .resizable(false)
        .fullscreen(true)
        .skip_taskbar(true)
        .visible(false)
        .build()
        .map_err(|e| format!("Failed to create video window: {e}"))?;

    window.show().map_err(|e| format!("Failed to show video window: {e}"))?;

    let host = MonitorVideoHost {
        monitor_index: 0,
        window_label: label.to_string(),
    };

    let mut guard = state.lock().map_err(|e| format!("State lock error: {e}"))?;
    guard.active = true;
    guard.paused = false;
    guard.current_path = Some(path.to_string());
    guard.monitors = vec![host];

    Ok(format!("Video wallpaper started: {path}"))
}

#[cfg(windows)]
pub fn pause_video_wallpaper(
    app: &AppHandle,
    state: &Mutex<VideoWallpaperState>,
) -> Result<String, String> {
    let guard = state.lock().map_err(|e| format!("State lock error: {e}"))?;
    if !guard.active {
        return Err("No active video wallpaper.".to_string());
    }

    for host in &guard.monitors {
        if let Some(window) = app.get_webview_window(&host.window_label) {
            let _ = window.eval("document.querySelector('video').pause()");
        }
    }

    drop(guard);
    let mut guard = state.lock().map_err(|e| format!("State lock error: {e}"))?;
    guard.paused = true;
    Ok("Video wallpaper paused.".to_string())
}

#[cfg(windows)]
pub fn resume_video_wallpaper(
    app: &AppHandle,
    state: &Mutex<VideoWallpaperState>,
) -> Result<String, String> {
    let guard = state.lock().map_err(|e| format!("State lock error: {e}"))?;
    if !guard.active {
        return Err("No active video wallpaper.".to_string());
    }

    for host in &guard.monitors {
        if let Some(window) = app.get_webview_window(&host.window_label) {
            let _ = window.eval("document.querySelector('video').play()");
        }
    }

    drop(guard);
    let mut guard = state.lock().map_err(|e| format!("State lock error: {e}"))?;
    guard.paused = false;
    Ok("Video wallpaper resumed.".to_string())
}

#[cfg(windows)]
pub fn stop_video_wallpaper(
    app: &AppHandle,
    state: &Mutex<VideoWallpaperState>,
) -> Result<String, String> {
    stop_video_wallpaper_inner(app, state)
}

#[cfg(windows)]
fn stop_video_wallpaper_inner(
    app: &AppHandle,
    state: &Mutex<VideoWallpaperState>,
) -> Result<String, String> {
    let mut guard = state.lock().map_err(|e| format!("State lock error: {e}"))?;

    for host in &guard.monitors {
        if let Some(window) = app.get_webview_window(&host.window_label) {
            let _ = window.close();
        }
    }

    guard.active = false;
    guard.paused = false;
    guard.current_path = None;
    guard.monitors.clear();

    Ok("Video wallpaper stopped.".to_string())
}

#[cfg(windows)]
pub fn video_wallpaper_status(
    state: &Mutex<VideoWallpaperState>,
) -> Result<VideoWallpaperStatus, String> {
    let guard = state.lock().map_err(|e| format!("State lock error: {e}"))?;
    Ok(VideoWallpaperStatus {
        active: guard.active,
        paused: guard.paused,
        monitor_count: guard.monitors.len() as u32,
        current_path: guard.current_path.clone(),
    })
}

// ---- Cross-platform stubs ----

#[cfg(not(windows))]
pub fn start_video_wallpaper(
    _app: &tauri::AppHandle,
    _state: &Mutex<VideoWallpaperState>,
    _path: &str,
) -> Result<String, String> {
    Err("Video wallpaper is only available on Windows.".to_string())
}

#[cfg(not(windows))]
pub fn pause_video_wallpaper(
    _app: &tauri::AppHandle,
    _state: &Mutex<VideoWallpaperState>,
) -> Result<String, String> {
    Err("Video wallpaper is only available on Windows.".to_string())
}

#[cfg(not(windows))]
pub fn resume_video_wallpaper(
    _app: &tauri::AppHandle,
    _state: &Mutex<VideoWallpaperState>,
) -> Result<String, String> {
    Err("Video wallpaper is only available on Windows.".to_string())
}

#[cfg(not(windows))]
pub fn stop_video_wallpaper(
    _app: &tauri::AppHandle,
    _state: &Mutex<VideoWallpaperState>,
) -> Result<String, String> {
    Err("Video wallpaper is only available on Windows.".to_string())
}

#[cfg(not(windows))]
pub fn video_wallpaper_status(
    _state: &Mutex<VideoWallpaperState>,
) -> Result<VideoWallpaperStatus, String> {
    Ok(VideoWallpaperStatus {
        active: false,
        paused: false,
        monitor_count: 0,
        current_path: None,
    })
}
