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

    let workerw_list = find_workerw_windows();

    let mut hosts: Vec<MonitorVideoHost> = Vec::new();

    for (idx, monitor_rect) in monitors.iter().enumerate() {
        let label = format!("swallpaper-video-{}", idx);

        let workerw_hwnd = workerw_list
            .iter()
            .find(|(wrect, _)| rects_overlap(wrect, monitor_rect))
            .map(|(_, hwnd)| *hwnd);

        let url = format!(
            "video.html?path={}",
            urlencoding::encode(path)
        );

        let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
            .title(format!("SwallpaperVideo-{idx}"))
            .decorations(false)
            .resizable(false)
            .visible(false)
            .skip_taskbar(true)
            .build()
            .map_err(|e| format!("Failed to create video window {idx}: {e}"))?;

        let video_hwnd = find_window_by_title(&format!("SwallpaperVideo-{idx}"))
            .ok_or_else(|| format!("Failed to locate video window {idx} HWND"))?;

        inject_into_desktop(video_hwnd, workerw_hwnd, monitor_rect)?;

        window
            .show()
            .map_err(|e| format!("Failed to show video window {idx}: {e}"))?;

        hosts.push(MonitorVideoHost {
            monitor_index: idx as u32,
            window_label: label,
            video_hwnd,
        });
    }

    let mut guard = state.lock().map_err(|e| format!("State lock error: {e}"))?;
    guard.active = true;
    guard.paused = false;
    guard.current_path = Some(path.to_string());
    guard.monitors = hosts;

    Ok(format!(
        "Video wallpaper started on {} monitor(s): {path}",
        monitors.len()
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

// ---- Win32 helpers ----

#[cfg(windows)]
fn enumerate_monitors() -> Result<Vec<MonitorRect>, String> {
    use windows::Win32::Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, MONITORINFOEXW};
    use windows::Win32::Foundation::LPARAM;

    struct Ctx {
        monitors: Vec<MonitorRect>,
        error: Option<String>,
    }

    let ctx = Mutex::new(Ctx {
        monitors: Vec::new(),
        error: None,
    });

    unsafe {
        let _result = EnumDisplayMonitors(
            HDC::default(),
            None,
            Some(monitor_enum_callback),
            LPARAM(&ctx as *const _ as isize),
        );
    }

    let ctx = ctx.lock().map_err(|e| format!("Lock error: {e}"))?;
    if let Some(err) = &ctx.error {
        return Err(err.clone());
    }
    Ok(ctx.monitors.clone())
}

type MonitorRect = (i32, i32, i32, i32);

#[cfg(windows)]
unsafe extern "system" fn monitor_enum_callback(
    hmonitor: windows::Win32::Graphics::Gdi::HMONITOR,
    _hdc: windows::Win32::Graphics::Gdi::HDC,
    _rect: *mut windows::Win32::Foundation::RECT,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::BOOL {
    use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MONITORINFOEXW};

    struct Ctx {
        monitors: Vec<MonitorRect>,
        error: Option<String>,
    }

    let ctx_ptr = lparam.0 as *const Mutex<Ctx>;
    let ctx = &*ctx_ptr;

    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

    if GetMonitorInfoW(
        hmonitor,
        &mut info as *mut MONITORINFOEXW as *mut _,
    )
    .as_bool()
    {
        let r = info.monitorInfo.rcWork;
        if let Ok(mut guard) = ctx.lock() {
            guard.monitors.push((r.left, r.top, r.right, r.bottom));
        }
    }

    true.into()
}

#[cfg(windows)]
fn find_workerw_windows() -> Vec<(MonitorRect, isize)> {
    use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, SendMessageW};
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};

    // Send 0x052C to Progman to spawn WorkerW
    unsafe {
        let progman = find_window_by_class("Progman");
        if let Some(progman) = progman {
            let _ = SendMessageW(
                HWND(progman as *mut _),
                0x052C,
                WPARAM(0),
                LPARAM(0),
            );
        }
    }

    struct Ctx {
        workers: Vec<(MonitorRect, isize)>,
    }

    let ctx = Mutex::new(Ctx {
        workers: Vec::new(),
    });

    unsafe {
        let _ = EnumWindows(
            Some(workerw_enum_callback),
            LPARAM(&ctx as *const _ as isize),
        );
    }

    ctx.lock().map(|g| g.workers.clone()).unwrap_or_default()
}

#[cfg(windows)]
unsafe extern "system" fn workerw_enum_callback(
    hwnd: windows::Win32::Foundation::HWND,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::BOOL {
    use windows::Win32::UI::WindowsAndMessaging::{FindWindowExW, GetClassNameW, GetWindowRect};
    use windows::Win32::Foundation::{HWND, RECT};
    use windows::core::PCWSTR;

    struct Ctx {
        workers: Vec<(MonitorRect, isize)>,
    }

    let ctx_ptr = lparam.0 as *const Mutex<Ctx>;
    let ctx = &*ctx_ptr;

    let mut class_buf = [0u16; 64];
    let len = GetClassNameW(hwnd, &mut class_buf);
    let class_name = String::from_utf16_lossy(&class_buf[..len as usize]);

    if class_name != "WorkerW" {
        return true.into();
    }

    // Verify this WorkerW has a SHELLDLL_DefView child
    if FindWindowExW(
        hwnd,
        HWND(std::ptr::null_mut()),
        PCWSTR(std::ptr::null()),
        PCWSTR(std::ptr::null()),
    )
    .is_err()
    {
        return true.into();
    }

    let mut rect = RECT::default();
    if GetWindowRect(hwnd, &mut rect).is_ok() {
        if let Ok(mut guard) = ctx.lock() {
            guard
                .workers
                .push(((rect.left, rect.top, rect.right, rect.bottom), hwnd.0 as isize));
        }
    }

    true.into()
}

#[cfg(windows)]
fn find_window_by_class(class: &str) -> Option<isize> {
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::FindWindowW;

    let class_wide: Vec<u16> = class.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        FindWindowW(PCWSTR(class_wide.as_ptr()), PCWSTR::null())
            .ok()
            .map(|hwnd| hwnd.0 as isize)
    }
}

#[cfg(windows)]
fn find_window_by_title(title: &str) -> Option<isize> {
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::FindWindowW;

    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();

    for _ in 0..20 {
        unsafe {
            if let Ok(hwnd) = FindWindowW(PCWSTR::null(), PCWSTR(title_wide.as_ptr())) {
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
    monitor_rect: &MonitorRect,
) -> Result<(), String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        SetParent, SetWindowPos, HWND_BOTTOM, SWP_NOACTIVATE, SWP_SHOWWINDOW,
    };

    let video = HWND(video_hwnd as *mut _);
    let &(left, top, right, bottom) = monitor_rect;
    let width = right - left;
    let height = bottom - top;

    unsafe {
        if let Some(workerw) = workerw_hwnd {
            SetParent(video, HWND(workerw as *mut _))
                .map_err(|e| format!("SetParent failed: {e}"))?;
        }

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
fn rects_overlap(a: &MonitorRect, b: &MonitorRect) -> bool {
    a.0 < b.2 && a.2 > b.0 && a.1 < b.3 && a.3 > b.1
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rects_overlap_yes() {
        let a = (0, 0, 100, 100);
        let b = (50, 50, 150, 150);
        assert!(rects_overlap(&a, &b));
    }

    #[test]
    fn test_rects_overlap_no() {
        let a = (0, 0, 100, 100);
        let b = (200, 200, 300, 300);
        assert!(!rects_overlap(&a, &b));
    }

    #[test]
    fn test_rects_overlap_contained() {
        let a = (0, 0, 1000, 1000);
        let b = (100, 100, 200, 200);
        assert!(rects_overlap(&a, &b));
    }

    #[test]
    fn test_rects_overlap_touching_edge() {
        let a = (0, 0, 100, 100);
        let b = (100, 100, 200, 200);
        assert!(!rects_overlap(&a, &b));
    }

    #[test]
    fn test_rects_overlap_identical() {
        let a = (0, 0, 1920, 1080);
        assert!(rects_overlap(&a, &a));
    }
}
