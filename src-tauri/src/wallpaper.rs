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
    #[cfg(windows)]
    pub video_hwnd: isize,
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

    let monitors = enumerate_monitors()?;
    if monitors.is_empty() {
        return Err("No monitors detected.".to_string());
    }

    let workerw_map = find_workerw_map();

    let mut hosts: Vec<MonitorVideoHost> = Vec::new();
    let mut created_labels: Vec<String> = Vec::new();

    for (idx, monitor_rect) in monitors.iter().enumerate() {
        let label = format!("swallpaper-video-{}", idx);
        let video_path = path.to_string();

        let workerw_hwnd = workerw_map
            .iter()
            .find(|(wrect, _)| rects_overlap(wrect, monitor_rect))
            .map(|(_, hwnd)| *hwnd);

        let url = format!(
            "video.html?path={}",
            urlencoding::encode(&video_path)
        );

        let builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
            .title(format!("SwallpaperVideo-{}", idx))
            .decorations(false)
            .resizable(false)
            .visible(false)
            .skip_taskbar(true);

        let window = builder
            .build()
            .map_err(|e| format!("Failed to create video window {idx}: {e}"))?;

        let video_hwnd = find_window_by_title(&format!("SwallpaperVideo-{}", idx))
            .ok_or_else(|| format!("Failed to locate video window {idx} HWND"))?;

        inject_into_desktop(video_hwnd, workerw_hwnd, monitor_rect)?;

        window.show().map_err(|e| format!("Failed to show video window: {e}"))?;

        hosts.push(MonitorVideoHost {
            monitor_index: idx as u32,
            window_label: label,
            video_hwnd,
        });

        created_labels.push(format!("SwallpaperVideo-{}", idx));
    }

    let mut guard = state.lock().map_err(|e| format!("State lock error: {e}"))?;
    guard.active = true;
    guard.paused = false;
    guard.current_path = Some(path.to_string());
    guard.monitors = hosts;

    Ok(format!(
        "Video wallpaper started on {} monitor(s): {path}",
        created_labels.len()
    ))
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
    _state: &std::sync::Mutex<VideoWallpaperState>,
    _path: &str,
) -> Result<String, String> {
    Err("Video wallpaper is only available on Windows.".to_string())
}

#[cfg(not(windows))]
pub fn pause_video_wallpaper(
    _app: &tauri::AppHandle,
    _state: &std::sync::Mutex<VideoWallpaperState>,
) -> Result<String, String> {
    Err("Video wallpaper is only available on Windows.".to_string())
}

#[cfg(not(windows))]
pub fn resume_video_wallpaper(
    _app: &tauri::AppHandle,
    _state: &std::sync::Mutex<VideoWallpaperState>,
) -> Result<String, String> {
    Err("Video wallpaper is only available on Windows.".to_string())
}

#[cfg(not(windows))]
pub fn stop_video_wallpaper(
    _app: &tauri::AppHandle,
    _state: &std::sync::Mutex<VideoWallpaperState>,
) -> Result<String, String> {
    Err("Video wallpaper is only available on Windows.".to_string())
}

#[cfg(not(windows))]
pub fn video_wallpaper_status(
    _state: &std::sync::Mutex<VideoWallpaperState>,
) -> Result<VideoWallpaperStatus, String> {
    Ok(VideoWallpaperStatus {
        active: false,
        paused: false,
        monitor_count: 0,
        current_path: None,
    })
}

// ---- Win32 helpers ----

#[cfg(windows)]
fn enumerate_monitors() -> Result<Vec<(i32, i32, i32, i32)>, String> {
    use windows::Win32::Graphics::Gdi::{EnumDisplayMonitors, HDC};
    use windows::Win32::Foundation::LPARAM;

    let monitors: Mutex<Vec<(i32, i32, i32, i32)>> = Mutex::new(Vec::new());

    unsafe {
        EnumDisplayMonitors(
            HDC::default(),
            None,
            Some(monitor_enum_callback),
            LPARAM(&monitors as *const _ as isize),
        )
    }
    .map_err(|e| format!("EnumDisplayMonitors failed: {e}"))?;

    let result = monitors.lock().map_err(|e| format!("Lock error: {e}"))?.clone();
    Ok(result)
}

#[cfg(windows)]
unsafe extern "system" fn monitor_enum_callback(
    hmonitor: windows::Win32::Graphics::Gdi::HMONITOR,
    _hdc: windows::Win32::Graphics::Gdi::HDC,
    _rect: *mut windows::Win32::Foundation::RECT,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::BOOL {
    use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MONITORINFOEXW};

    let monitors_ptr = lparam.0 as *const Mutex<Vec<(i32, i32, i32, i32)>>;
    let monitors = &*monitors_ptr;

    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

    if GetMonitorInfoW(hmonitor, &mut info as *mut _ as *mut _).is_ok() {
        let r = info.monitorInfo.rcWork;
        if let Ok(mut guard) = monitors.lock() {
            guard.push((r.left, r.top, r.right, r.bottom));
        }
    }

    windows::Win32::Foundation::BOOL::from(true)
}

#[cfg(windows)]
fn find_workerw_map() -> Vec<((i32, i32, i32, i32), isize)> {
    use std::sync::Mutex as StdMutex;
    use windows::Win32::UI::WindowsAndMessaging::EnumWindows;
    use windows::Win32::Foundation::LPARAM;

    // Send magic message to Progman to ensure WorkerW exists
    unsafe {
        if let Some(progman) = find_window_by_class("Progman") {
            use windows::Win32::UI::WindowsAndMessaging::SendMessageW;
            use windows::Win32::Foundation::HWND;
            SendMessageW(
                HWND(progman as *mut _),
                0x052C,
                None,
                None,
            );
        }
    }

    // Collect all WorkerW windows with SHELLDLL_DefView
    let workerw_list: StdMutex<Vec<(isize, (i32, i32, i32, i32))>> = StdMutex::new(Vec::new());

    unsafe {
        let _ = EnumWindows(
            Some(workerw_enum_callback),
            LPARAM(&workerw_list as *const _ as isize),
        );
    }

    let workerw_list = workerw_list.lock().unwrap().clone();
    workerw_list
}

#[cfg(windows)]
unsafe extern "system" fn workerw_enum_callback(
    hwnd: windows::Win32::Foundation::HWND,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::BOOL {
    use windows::Win32::UI::WindowsAndMessaging::{FindWindowExW, GetClassNameW, GetWindowRect};
    use windows::Win32::Foundation::RECT;

    let list_ptr = lparam.0 as *const std::sync::Mutex<Vec<(isize, (i32, i32, i32, i32))>>;
    let list = &*list_ptr;

    let mut class_buf = [0u16; 64];
    let len = GetClassNameW(hwnd, Some(&mut class_buf));
    let class_name = String::from_utf16_lossy(&class_buf[..len as usize]);

    if class_name != "WorkerW" {
        return windows::Win32::Foundation::BOOL::from(true);
    }

    let defview = FindWindowExW(hwnd, None, None, None);
    if defview.is_none() {
        return windows::Win32::Foundation::BOOL::from(true);
    }

    let mut rect = RECT::default();
    if GetWindowRect(hwnd, &mut rect).is_ok() {
        if let Ok(mut guard) = list.lock() {
            guard.push((hwnd.0 as isize, (rect.left, rect.top, rect.right, rect.bottom)));
        }
    }

    windows::Win32::Foundation::BOOL::from(true)
}

#[cfg(windows)]
fn find_window_by_class(class: &str) -> Option<isize> {
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::FindWindowW;

    let class_wide: Vec<u16> = class.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        FindWindowW(Some(PCWSTR(class_wide.as_ptr())), None)
            .map(|hwnd| hwnd.0 as isize)
    }
}

#[cfg(windows)]
fn find_window_by_title(title: &str) -> Option<isize> {
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::FindWindowW;

    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();

    // Retry a few times since the window might not be immediately findable
    for _ in 0..20 {
        unsafe {
            if let Some(hwnd) = FindWindowW(None, Some(PCWSTR(title_wide.as_ptr()))) {
                return Some(hwnd.0 as isize);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    None
}

#[cfg(windows)]
fn inject_into_desktop(
    video_hwnd: isize,
    workerw_hwnd: Option<isize>,
    monitor_rect: &(i32, i32, i32, i32),
) -> Result<(), String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        SetParent, SetWindowPos, HWND_BOTTOM, SWP_NOACTIVATE, SWP_SHOWWINDOW,
    };

    let video = HWND(video_hwnd as *mut _);
    let (left, top, right, bottom) = *monitor_rect;
    let width = right - left;
    let height = bottom - top;

    unsafe {
        // Parent to WorkerW if available
        if let Some(workerw) = workerw_hwnd {
            SetParent(video, Some(HWND(workerw as *mut _)))
                .map_err(|e| format!("SetParent failed: {e}"))?;
        }

        // Position and size the window to cover the monitor work area
        SetWindowPos(
            video,
            HWND_BOTTOM,
            left,
            top,
            width,
            height,
            SWP_NOACTIVATE | SWP_SHOWWINDOW,
        )
        .map_err(|e| format!("SetWindowPos failed: {e}"))?;
    }

    Ok(())
}

#[cfg(windows)]
fn rects_overlap(a: &(i32, i32, i32, i32), b: &(i32, i32, i32, i32)) -> bool {
    let (a_left, a_top, a_right, a_bottom) = *a;
    let (b_left, b_top, b_right, b_bottom) = *b;
    a_left < b_right && a_right > b_left && a_top < b_bottom && a_bottom > b_top
}
