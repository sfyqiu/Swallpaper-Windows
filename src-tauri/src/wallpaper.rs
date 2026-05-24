#[cfg(windows)]
pub fn set_static_wallpaper(path: &str) -> Result<String, String> {
    use std::iter::once;
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPIF_SENDWININICHANGE, SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER,
    };

    let wide: Vec<u16> = path.encode_utf16().chain(once(0)).collect();
    let ok = unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(PCWSTR(wide.as_ptr()).0 as *mut _),
            SPIF_UPDATEINIFILE | SPIF_SENDWININICHANGE,
        )
    };

    if ok.as_bool() {
        Ok(format!("Static wallpaper set: {path}"))
    } else {
        Err("Windows rejected the wallpaper path.".to_string())
    }
}

#[cfg(not(windows))]
pub fn set_static_wallpaper(_path: &str) -> Result<String, String> {
    Err("Static wallpaper command is only available on Windows.".to_string())
}

pub fn start_video_wallpaper(path: &str) -> Result<String, String> {
    Ok(format!(
        "Video wallpaper host command received for {path}. WorkerW implementation is next."
    ))
}

pub fn stop_video_wallpaper() -> Result<String, String> {
    Ok("Video wallpaper host stopped.".to_string())
}
