use serde::{Deserialize, Serialize};
use std::{
    env,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

// ---- Config ----

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncConfig {
    pub enabled: bool,
    pub provider: Option<String>,
    pub provider_name: Option<String>,
    pub root_path: Option<String>,
    pub library_path: Option<String>,
    pub mode: String, // "auto" or "manual"
}

impl SyncConfig {
    fn default_config() -> Self {
        Self {
            enabled: false,
            provider: None,
            provider_name: None,
            root_path: None,
            library_path: None,
            mode: "manual".to_string(),
        }
    }

    fn config_path() -> Result<PathBuf, String> {
        let base = local_data_dir()?;
        Ok(base.join("sync_config.json"))
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
            fs::create_dir_all(parent).map_err(|e| format!("Cannot create config dir: {e}"))?;
        }
        let data =
            serde_json::to_vec_pretty(self).map_err(|e| format!("Cannot encode sync config: {e}"))?;
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, data).map_err(|e| format!("Cannot write sync config: {e}"))?;
        fs::rename(&tmp, &path).map_err(|e| format!("Cannot save sync config: {e}"))
    }
}

fn local_data_dir() -> Result<PathBuf, String> {
    env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| env::var_os("APPDATA").map(PathBuf::from))
        .or_else(|| env::current_dir().ok())
        .map(|p| p.join("Swallpaper"))
        .ok_or_else(|| "Unable to resolve local data directory.".to_string())
}

// ---- Provider info ----

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub suggested_path: Option<String>,
    pub detected: bool,
    pub detected_path: Option<String>,
}

pub fn list_providers() -> Vec<ProviderInfo> {
    let providers = [
        ("onedrive", "OneDrive", "Microsoft 云存储"),
        ("icloud", "iCloud Drive", "Apple 云盘（Windows 版）"),
        ("dropbox", "Dropbox", "老牌同步云盘"),
        ("google_drive", "Google Drive", "Google 云存储"),
        ("nutstore", "坚果云", "国内老牌同步盘"),
        ("baidu", "百度网盘", "百度网盘同步空间"),
        ("custom", "自定义文件夹", "任意本机目录"),
    ];

    providers
        .iter()
        .map(|(id, name, desc)| {
            let detected_path = detect_provider_path(id);
            ProviderInfo {
                id: id.to_string(),
                name: name.to_string(),
                description: desc.to_string(),
                suggested_path: suggested_path_for(id),
                detected: detected_path.is_some(),
                detected_path,
            }
        })
        .collect()
}

fn detect_provider_path(id: &str) -> Option<String> {
    let home = env::var_os("USERPROFILE").map(PathBuf::from)?;

    let candidates: &[&str] = match id {
        "onedrive" => &["OneDrive"],
        "icloud" => &["iCloudDrive", "Apple\\Mobile Documents"],
        "dropbox" => &["Dropbox"],
        "google_drive" => &["Google Drive"],
        "nutstore" => &["Nutstore Files", "坚果云"],
        "baidu" => &["BaiduNetdiskDownload", "百度网盘"],
        _ => return None,
    };

    for candidate in candidates {
        let path = home.join(candidate);
        if path.is_dir() {
            return Some(path.display().to_string());
        }
    }
    None
}

fn suggested_path_for(id: &str) -> Option<String> {
    let home = env::var_os("USERPROFILE").map(PathBuf::from)?;
    let rel = match id {
        "onedrive" => "OneDrive",
        "icloud" => "iCloudDrive",
        "dropbox" => "Dropbox",
        "google_drive" => "Google Drive",
        "nutstore" => "Nutstore Files",
        "baidu" => "BaiduNetdiskDownload",
        _ => return None,
    };
    Some(home.join(rel).display().to_string())
}

// ---- Enable / Disable ----

pub fn enable_sync(
    provider: &str,
    provider_name: &str,
    root_path: &str,
    mode: &str,
) -> Result<String, String> {
    let root = PathBuf::from(root_path);
    if !root.is_dir() {
        return Err(format!("Directory not found: {root_path}"));
    }

    let lib_path = library_dir(&root);
    ensure_sync_structure(&lib_path)?;

    let manifest = SyncManifest::create(provider);
    save_manifest(&lib_path, &manifest)?;

    let mut config = SyncConfig::load();
    config.enabled = true;
    config.provider = Some(provider.to_string());
    config.provider_name = Some(provider_name.to_string());
    config.root_path = Some(root_path.to_string());
    config.library_path = Some(lib_path.display().to_string());
    config.mode = mode.to_string();
    config.save()?;

    Ok(format!(
        "Cloud sync enabled: {provider_name} → {}",
        lib_path.display()
    ))
}

pub fn disable_sync() -> Result<String, String> {
    let mut config = SyncConfig::load();
    config.enabled = false;
    config.provider = None;
    config.provider_name = None;
    config.root_path = None;
    config.library_path = None;
    config.save()?;
    Ok("Cloud sync disabled.".to_string())
}

pub fn get_sync_config() -> SyncConfig {
    SyncConfig::load()
}

pub fn get_sync_status() -> SyncStatus {
    let config = SyncConfig::load();
    if !config.enabled {
        return SyncStatus {
            enabled: false,
            provider: None,
            provider_name: None,
            library_path: None,
            mode: "manual".to_string(),
            manifest: None,
            scan_result: None,
        };
    }

    let lib_path = config.library_path.as_ref().map(PathBuf::from);
    let manifest = lib_path
        .as_ref()
        .and_then(|p| load_manifest(p).ok());

    SyncStatus {
        enabled: true,
        provider: config.provider.clone(),
        provider_name: config.provider_name.clone(),
        library_path: config.library_path.clone(),
        mode: config.mode.clone(),
        manifest,
        scan_result: None,
    }
}

// ---- Manifest ----

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncManifest {
    pub schema_version: u32,
    pub library_id: String,
    pub app_name: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_device_name: String,
    pub provider: String,
    pub records: SyncRecordCounts,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncRecordCounts {
    pub wallpapers: u32,
    pub media: u32,
    pub downloads: u32,
}

impl SyncManifest {
    fn create(provider: &str) -> Self {
        let now = now_iso_string();
        let device = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "Unknown".to_string());
        Self {
            schema_version: 1,
            library_id: uuid_v4(),
            app_name: "Swallpaper".to_string(),
            created_at: now.clone(),
            updated_at: now,
            last_device_name: device,
            provider: provider.to_string(),
            records: SyncRecordCounts {
                wallpapers: 0,
                media: 0,
                downloads: 0,
            },
        }
    }
}

fn save_manifest(lib_path: &Path, manifest: &SyncManifest) -> Result<(), String> {
    let mut m = manifest.clone();
    m.updated_at = now_iso_string();
    let path = lib_path.join("manifest.json");
    atomic_write_json(&m, &path)
}

fn load_manifest(lib_path: &Path) -> Result<SyncManifest, String> {
    let path = lib_path.join("manifest.json");
    let data = fs::read(&path).map_err(|e| format!("Cannot read manifest: {e}"))?;
    serde_json::from_slice(&data).map_err(|e| format!("Cannot parse manifest: {e}"))
}

// ---- Scan ----

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub enabled: bool,
    pub provider: Option<String>,
    pub provider_name: Option<String>,
    pub library_path: Option<String>,
    pub mode: String,
    pub manifest: Option<SyncManifest>,
    pub scan_result: Option<SyncScanResult>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncScanResult {
    pub total_records: u32,
    pub available_count: u32,
    pub missing_count: u32,
    pub records: Vec<SyncRecord>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncRecord {
    pub id: String,
    pub kind: String,
    pub source: String,
    pub title: Option<String>,
    pub remote_url: Option<String>,
    pub relative_file_path: String,
    pub thumbnail_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub file_size: Option<u64>,
    pub sha256: Option<String>,
    pub status: String,
}

pub fn scan_sync_library() -> Result<SyncScanResult, String> {
    let config = SyncConfig::load();
    if !config.enabled {
        return Err("Cloud sync is not enabled.".to_string());
    }

    let lib_path = config
        .library_path
        .as_ref()
        .ok_or("Library path not set.")?;
    let lib = PathBuf::from(lib_path);
    if !lib.is_dir() {
        return Err("Library directory not found.".to_string());
    }

    let mut records: Vec<SyncRecord> = Vec::new();
    let metadata_dir = lib.join("metadata");

    for file_name in &["wallpapers.json", "media.json"] {
        let url = metadata_dir.join(file_name);
        if url.is_file() {
            let data = fs::read(&url).unwrap_or_default();
            if let Ok(mut file_records) =
                serde_json::from_slice::<Vec<SyncRecord>>(&data)
            {
                for record in &mut file_records {
                    let abs = lib.join(&record.relative_file_path);
                    record.status = if abs.is_file() {
                        "available".to_string()
                    } else {
                        "missing".to_string()
                    };
                }
                records.extend(file_records);
            }
        }
    }

    let available = records.iter().filter(|r| r.status == "available").count() as u32;
    let missing = records.iter().filter(|r| r.status == "missing").count() as u32;

    Ok(SyncScanResult {
        total_records: records.len() as u32,
        available_count: available,
        missing_count: missing,
        records,
    })
}

// ---- Import from local library ----

pub fn import_local_to_sync() -> Result<String, String> {
    let config = SyncConfig::load();
    if !config.enabled {
        return Err("Cloud sync is not enabled.".to_string());
    }

    let lib_path = config
        .library_path
        .as_ref()
        .ok_or("Library path not set.")?;
    let sync_lib = PathBuf::from(lib_path);
    ensure_sync_structure(&sync_lib)?;

    let local_lib = local_data_dir()?.join("Library");
    let local_files = local_lib.join("files");

    let mut wallpaper_records: Vec<SyncRecord> = Vec::new();
    let mut media_records: Vec<SyncRecord> = Vec::new();
    let mut counts = (0u32, 0u32); // (wallpapers, media)

    // Import static wallpapers
    let src_wp = local_files.join("wallpapers");
    if src_wp.is_dir() {
        let dest_wp = sync_lib.join("files").join("wallpapers");
        fs::create_dir_all(&dest_wp).ok();
        if let Ok(entries) = fs::read_dir(&src_wp) {
            for entry in entries.flatten() {
                let src = entry.path();
                if src.extension().is_none() {
                    continue;
                }
                let fname = src.file_name().unwrap();
                let dest = dest_wp.join(fname);
                if !dest.is_file() {
                    fs::copy(&src, &dest).ok();
                }
                let rel = format!("files/wallpapers/{}", fname.to_string_lossy());
                let id = src.file_stem().unwrap().to_string_lossy().to_string();
                wallpaper_records.push(SyncRecord {
                    id,
                    kind: "staticWallpaper".to_string(),
                    source: "imported".to_string(),
                    title: Some(fname.to_string_lossy().to_string()),
                    remote_url: None,
                    relative_file_path: rel,
                    thumbnail_path: None,
                    created_at: now_iso_string(),
                    updated_at: now_iso_string(),
                    file_size: src.metadata().ok().map(|m| m.len()),
                    sha256: None,
                    status: "available".to_string(),
                });
                counts.0 += 1;
            }
        }
    }

    // Import videos
    let src_vid = local_files.join("videos");
    if src_vid.is_dir() {
        let dest_vid = sync_lib.join("files").join("videos");
        fs::create_dir_all(&dest_vid).ok();
        if let Ok(entries) = fs::read_dir(&src_vid) {
            for entry in entries.flatten() {
                let src = entry.path();
                if src.extension().is_none() {
                    continue;
                }
                let fname = src.file_name().unwrap();
                let dest = dest_vid.join(fname);
                if !dest.is_file() {
                    fs::copy(&src, &dest).ok();
                }
                let rel = format!("files/videos/{}", fname.to_string_lossy());
                let id = src.file_stem().unwrap().to_string_lossy().to_string();
                media_records.push(SyncRecord {
                    id,
                    kind: "videoWallpaper".to_string(),
                    source: "imported".to_string(),
                    title: Some(fname.to_string_lossy().to_string()),
                    remote_url: None,
                    relative_file_path: rel,
                    thumbnail_path: None,
                    created_at: now_iso_string(),
                    updated_at: now_iso_string(),
                    file_size: src.metadata().ok().map(|m| m.len()),
                    sha256: None,
                    status: "available".to_string(),
                });
                counts.1 += 1;
            }
        }
    }

    // Write metadata
    if !wallpaper_records.is_empty() {
        let wp_path = sync_lib.join("metadata").join("wallpapers.json");
        atomic_write_json(&wallpaper_records, &wp_path)?;
    }
    if !media_records.is_empty() {
        let media_path = sync_lib.join("metadata").join("media.json");
        atomic_write_json(&media_records, &media_path)?;
    }

    // Update manifest
    let mut manifest = load_manifest(&sync_lib).unwrap_or_else(|_| SyncManifest::create(
        config.provider.as_deref().unwrap_or("custom"),
    ));
    manifest.records.wallpapers += counts.0;
    manifest.records.media += counts.1;
    manifest.records.downloads += counts.0 + counts.1;
    save_manifest(&sync_lib, &manifest)?;

    Ok(format!(
        "Imported {} wallpapers and {} videos to cloud library.",
        counts.0, counts.1
    ))
}

// ---- Sync-aware download path ----

/// Returns the active download path for the given media kind.
/// If sync is enabled and mode is "auto", returns the cloud library path.
/// Otherwise returns the local library path.
pub fn active_download_dir(kind: &str) -> Result<PathBuf, String> {
    let config = SyncConfig::load();
    if config.enabled && config.mode == "auto" {
        if let Some(ref lib_path) = config.library_path {
            let sync_lib = PathBuf::from(lib_path);
            ensure_sync_structure(&sync_lib)?;
            let subdir = if kind == "videoWallpaper" {
                "files/videos"
            } else {
                "files/wallpapers"
            };
            let dir = sync_lib.join(subdir);
            fs::create_dir_all(&dir)
                .map_err(|e| format!("Cannot create cloud dir: {e}"))?;
            return Ok(dir);
        }
    }
    // Fall back to local
    let local = local_data_dir()?.join("Library").join("files");
    let subdir = if kind == "videoWallpaper" {
        "videos"
    } else {
        "wallpapers"
    };
    Ok(local.join(subdir))
}

// ---- Helpers ----

fn library_dir(root: &Path) -> PathBuf {
    if root
        .file_name()
        .map(|n| n == "Swallpaper Library")
        .unwrap_or(false)
    {
        root.to_path_buf()
    } else {
        root.join("Swallpaper Library")
    }
}

fn ensure_sync_structure(lib_path: &Path) -> Result<(), String> {
    for dir in &[
        "metadata",
        "files/wallpapers",
        "files/videos",
        "thumbnails",
        "cache",
        "logs",
    ] {
        let path = lib_path.join(dir);
        fs::create_dir_all(&path)
            .map_err(|e| format!("Cannot create {}: {e}", path.display()))?;
    }
    Ok(())
}

fn atomic_write_json<T: Serialize>(value: &T, path: &Path) -> Result<(), String> {
    let tmp = path.with_extension("json.tmp");
    let data = serde_json::to_vec_pretty(value)
        .map_err(|e| format!("Cannot encode JSON: {e}"))?;
    fs::write(&tmp, data).map_err(|e| format!("Cannot write {}: {e}", tmp.display()))?;
    // Verify
    let _verify = fs::read(&tmp).map_err(|e| format!("Cannot verify {}: {e}", tmp.display()))?;
    fs::rename(&tmp, path).map_err(|e| format!("Cannot rename to {}: {e}", path.display()))
}

fn now_iso_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    // Simple ISO-like format
    let days_since_epoch = secs / 86400;
    let remaining_secs = secs % 86400;
    let hours = remaining_secs / 3600;
    let minutes = (remaining_secs % 3600) / 60;
    let seconds = remaining_secs % 60;

    // Calculate date from epoch days (since 1970-01-01)
    let mut y = 1970i64;
    let mut d = days_since_epoch as i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }
    let month_days = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 0;
    for (i, &md) in month_days.iter().enumerate() {
        if d < md {
            m = i + 1;
            break;
        }
        d -= md;
    }
    let day = d + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, day, hours, minutes, seconds
    )
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

fn uuid_v4() -> String {
    // Simple UUID v4 generation without external crate
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let nanos = now.as_nanos();
    let pid = std::process::id();

    let mut buf = [0u8; 16];
    // Mix time and pid for pseudo-randomness
    let mix = nanos as u64 ^ ((pid as u64) << 32);
    for i in 0..8 {
        buf[i] = ((mix >> (i * 8)) & 0xFF) as u8;
    }
    let mix2 = (nanos >> 16) as u64 ^ ((pid as u64) << 16);
    for i in 0..8 {
        buf[i + 8] = ((mix2 >> (i * 8)) & 0xFF) as u8;
    }
    // Set version 4
    buf[6] = (buf[6] & 0x0F) | 0x40;
    // Set variant
    buf[8] = (buf[8] & 0x3F) | 0x80;

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        buf[0], buf[1], buf[2], buf[3],
        buf[4], buf[5],
        buf[6], buf[7],
        buf[8], buf[9],
        buf[10], buf[11], buf[12], buf[13], buf[14], buf[15],
    )
}
