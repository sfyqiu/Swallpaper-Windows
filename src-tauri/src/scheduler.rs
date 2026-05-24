use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::env;

use crate::library;

fn local_data_dir() -> Result<PathBuf, String> {
    env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| env::var_os("APPDATA").map(PathBuf::from))
        .or_else(|| env::current_dir().ok())
        .map(|p| p.join("Swallpaper"))
        .ok_or_else(|| "Unable to resolve local data directory.".to_string())
}

// ---- Config ----

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerConfig {
    pub enabled: bool,
    pub interval_minutes: u32,
    pub include_static: bool,
    pub include_video: bool,
    pub change_on_startup: bool,
}

impl SchedulerConfig {
    fn default_config() -> Self {
        Self {
            enabled: false,
            interval_minutes: 30,
            include_static: true,
            include_video: true,
            change_on_startup: false,
        }
    }

    fn config_path() -> Result<PathBuf, String> {
        let base = local_data_dir()?;
        Ok(base.join("scheduler_config.json"))
    }

    pub fn load() -> Self {
        let path = match Self::config_path() {
            Ok(p) => p,
            Err(_) => return Self::default_config(),
        };
        fs::read(&path)
            .ok()
            .and_then(|data| serde_json::from_slice(&data).ok())
            .unwrap_or_else(Self::default_config)
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Cannot create dir: {e}"))?;
        }
        let data = serde_json::to_vec_pretty(self)
            .map_err(|e| format!("Cannot encode scheduler config: {e}"))?;
        fs::write(&path, data).map_err(|e| format!("Cannot save: {e}"))
    }
}

// ---- Rotate action ----

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RotateResult {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub file_path: String,
    pub is_video: bool,
}

pub fn rotate_wallpaper(
    app: &tauri::AppHandle,
    video_state: &std::sync::Mutex<crate::wallpaper::VideoWallpaperState>,
) -> Result<RotateResult, String> {
    let config = SchedulerConfig::load();

    let all = library::wallpapers()?;
    if all.is_empty() {
        return Err("Library is empty.".to_string());
    }

    // Filter by kind
    let candidates: Vec<&library::LibraryWallpaper> = all
        .iter()
        .filter(|w| {
            (config.include_static && w.kind == "staticWallpaper")
                || (config.include_video && w.kind == "videoWallpaper")
        })
        .collect();

    if candidates.is_empty() {
        return Err("No matching wallpapers for current filter.".to_string());
    }

    // Pick random
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as usize;
    let idx = seed % candidates.len();
    let picked = candidates[idx];

    let file_path = &picked.file_path;
    let is_video = picked.kind == "videoWallpaper";

    if is_video {
        crate::wallpaper::stop_video_wallpaper(app, video_state)?;
        crate::wallpaper::start_video_wallpaper(app, video_state, file_path)?;
    } else {
        crate::wallpaper::set_static_wallpaper(file_path)?;
    }

    Ok(RotateResult {
        id: picked.id.clone(),
        title: picked.title.clone(),
        kind: picked.kind.clone(),
        file_path: file_path.clone(),
        is_video,
    })
}
