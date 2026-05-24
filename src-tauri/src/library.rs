use crate::sources::WallpaperItem;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex as StdMutex,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryStatus {
    configured: bool,
    provider: Option<String>,
    root: Option<String>,
    records: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadResult {
    id: String,
    kind: String,
    file_path: String,
    records: u32,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryRecord {
    id: String,
    kind: String,
    source: String,
    title: String,
    author: Option<String>,
    detail_url: String,
    remote_url: String,
    video_url: Option<String>,
    relative_file_path: String,
    thumbnail_url: String,
    width: Option<u32>,
    height: Option<u32>,
    downloaded_at: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryWallpaper {
    id: String,
    kind: String,
    source: String,
    title: String,
    author: Option<String>,
    detail_url: String,
    remote_url: String,
    video_url: Option<String>,
    file_path: String,
    thumbnail_url: String,
    width: Option<u32>,
    height: Option<u32>,
    downloaded_at: String,
}

fn library_root() -> Result<PathBuf, String> {
    let base = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| env::var_os("APPDATA").map(PathBuf::from))
        .or_else(|| env::current_dir().ok())
        .ok_or("Unable to resolve a writable application data directory.")?;

    Ok(base.join("Swallpaper").join("Library"))
}

fn metadata_path(root: &Path) -> PathBuf {
    root.join("metadata").join("wallpapers.json")
}

fn ensure_layout(root: &Path) -> Result<(), String> {
    for path in [
        root.join("metadata"),
        root.join("files").join("wallpapers"),
        root.join("files").join("videos"),
        root.join("thumbnails"),
        root.join("logs"),
    ] {
        fs::create_dir_all(&path).map_err(|error| format!("Unable to create {}: {error}", path.display()))?;
    }
    Ok(())
}

fn read_records(root: &Path) -> Vec<LibraryRecord> {
    let path = metadata_path(root);
    let Ok(data) = fs::read(path) else {
        return Vec::new();
    };
    serde_json::from_slice(&data).unwrap_or_default()
}

fn write_records(root: &Path, records: &[LibraryRecord]) -> Result<(), String> {
    let path = metadata_path(root);
    let temp = path.with_extension("json.tmp");
    let data = serde_json::to_vec_pretty(records).map_err(|error| format!("Unable to encode library metadata: {error}"))?;
    fs::write(&temp, data).map_err(|error| format!("Unable to write {}: {error}", temp.display()))?;
    fs::rename(&temp, &path).map_err(|error| format!("Unable to replace {}: {error}", path.display()))
}

pub fn status() -> LibraryStatus {
    match library_root() {
        Ok(root) => {
            let _ = ensure_layout(&root);
            let records = read_records(&root);
            LibraryStatus {
                configured: true,
                provider: Some("local".to_string()),
                root: Some(root.display().to_string()),
                records: records.len() as u32,
            }
        }
        Err(_) => LibraryStatus {
            configured: false,
            provider: None,
            root: None,
            records: 0,
        },
    }
}

pub fn wallpapers() -> Result<Vec<LibraryWallpaper>, String> {
    let root = library_root()?;
    ensure_layout(&root)?;

    Ok(read_records(&root)
        .into_iter()
        .map(|record| {
            let file_path = root.join(&record.relative_file_path).display().to_string();
            LibraryWallpaper {
                id: record.id,
                kind: record.kind,
                source: record.source,
                title: record.title,
                author: record.author,
                detail_url: record.detail_url,
                remote_url: record.remote_url,
                video_url: record.video_url,
                file_path,
                thumbnail_url: record.thumbnail_url,
                width: record.width,
                height: record.height,
                downloaded_at: record.downloaded_at,
            }
        })
        .collect())
}

pub async fn download_wallpaper(item: WallpaperItem) -> Result<DownloadResult, String> {
    let root = library_root()?;
    ensure_layout(&root)?;

    let is_video = item.kind == "videoWallpaper";
    let download_url = if is_video {
        item.video_url.as_ref().ok_or("Video URL is missing.")?
    } else {
        &item.image_url
    };

    let response = reqwest::Client::new()
        .get(download_url)
        .header("User-Agent", "Swallpaper-Windows/0.1")
        .send()
        .await
        .map_err(|error| format!("Download request failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Download HTTP error: {error}"))?;

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok());

    let extension = file_extension(download_url, content_type, is_video);
    let safe_id = sanitize_filename(&item.id);
    let subdir = if is_video { "files/videos" } else { "files/wallpapers" };
    let relative = format!("{subdir}/{safe_id}.{extension}");
    let absolute = root.join(&relative);

    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("Unable to read download body: {error}"))?;

    let mut file = fs::File::create(&absolute)
        .map_err(|error| format!("Unable to create {}: {error}", absolute.display()))?;
    file.write_all(&bytes)
        .map_err(|error| format!("Unable to write {}: {error}", absolute.display()))?;

    let mut records = read_records(&root);
    records.retain(|record| record.id != item.id);
    records.insert(
        0,
        LibraryRecord {
            id: item.id.clone(),
            kind: item.kind.clone(),
            source: item.source,
            title: item.title,
            author: item.author,
            detail_url: item.detail_url,
            remote_url: item.image_url,
            video_url: item.video_url,
            relative_file_path: relative,
            thumbnail_url: item.thumbnail_url,
            width: item.width,
            height: item.height,
            downloaded_at: now_utc_string(),
        },
    );
    write_records(&root, &records)?;

    Ok(DownloadResult {
        id: item.id,
        kind: item.kind,
        file_path: absolute.display().to_string(),
        records: records.len() as u32,
    })
}

fn file_extension(url: &str, content_type: Option<&str>, is_video: bool) -> &'static str {
    if let Some(content_type) = content_type {
        if content_type.contains("mp4") || content_type.contains("video/mp4") {
            return "mp4";
        }
        if content_type.contains("webm") || content_type.contains("video/webm") {
            return "webm";
        }
        if content_type.contains("png") {
            return "png";
        }
        if content_type.contains("webp") {
            return "webp";
        }
        if content_type.contains("jpeg") || content_type.contains("jpg") {
            return "jpg";
        }
    }

    let lower = url.split('?').next().unwrap_or(url).to_lowercase();
    if lower.ends_with(".mp4") {
        "mp4"
    } else if lower.ends_with(".webm") {
        "webm"
    } else if lower.ends_with(".png") {
        "png"
    } else if lower.ends_with(".webp") {
        "webp"
    } else if is_video {
        "mp4"
    } else {
        "jpg"
    }
}

fn sanitize_filename(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn now_utc_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    format!("{seconds}")
}

// ---- Favorites ----

fn favorites_path(root: &Path) -> PathBuf {
    root.join("metadata").join("favorites.json")
}

fn read_favorite_ids(root: &Path) -> Vec<String> {
    let path = favorites_path(root);
    fs::read(&path)
        .ok()
        .and_then(|data| serde_json::from_slice::<Vec<String>>(&data).ok())
        .unwrap_or_default()
}

fn write_favorite_ids(root: &Path, ids: &[String]) -> Result<(), String> {
    let path = favorites_path(root);
    let data = serde_json::to_vec_pretty(ids)
        .map_err(|e| format!("Cannot encode favorites: {e}"))?;
    fs::write(&path, data)
        .map_err(|e| format!("Cannot write favorites: {e}"))
}

pub fn toggle_favorite(id: &str) -> Result<String, String> {
    let root = library_root()?;
    ensure_layout(&root)?;
    let mut ids = read_favorite_ids(&root);
    if ids.contains(&id.to_string()) {
        ids.retain(|i| i != id);
        write_favorite_ids(&root, &ids)?;
        Ok("Removed from favorites.".to_string())
    } else {
        ids.push(id.to_string());
        write_favorite_ids(&root, &ids)?;
        Ok("Added to favorites.".to_string())
    }
}

pub fn list_favorites() -> Result<Vec<LibraryWallpaper>, String> {
    let root = library_root()?;
    ensure_layout(&root)?;
    let fav_ids = read_favorite_ids(&root);
    let all_records = read_records(&root);
    Ok(all_records
        .into_iter()
        .filter(|r| fav_ids.contains(&r.id))
        .map(|record| {
            let file_path = root.join(&record.relative_file_path).display().to_string();
            LibraryWallpaper {
                id: record.id,
                kind: record.kind,
                source: record.source,
                title: record.title,
                author: record.author,
                detail_url: record.detail_url,
                remote_url: record.remote_url,
                video_url: record.video_url,
                file_path,
                thumbnail_url: record.thumbnail_url,
                width: record.width,
                height: record.height,
                downloaded_at: record.downloaded_at,
            }
        })
        .collect())
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        let result = sanitize_filename("hello/world:test.png");
        assert_eq!(result, "hello-world-test-png");
    }

    #[test]
    fn test_sanitize_filename_simple() {
        let result = sanitize_filename("my-wallpaper_01");
        assert_eq!(result, "my-wallpaper_01");
    }

    #[test]
    fn test_image_extension_from_url() {
        assert_eq!(file_extension("https://x.com/img.png", None, false), "png");
        assert_eq!(file_extension("https://x.com/img.webp", None, false), "webp");
        assert_eq!(file_extension("https://x.com/img.jpg", None, false), "jpg");
        assert_eq!(file_extension("https://x.com/video.mp4", None, false), "mp4");
        assert_eq!(file_extension("https://x.com/video.webm", None, false), "webm");
    }

    #[test]
    fn test_image_extension_from_content_type() {
        assert_eq!(file_extension("", Some("image/png"), false), "png");
        assert_eq!(file_extension("", Some("image/webp"), false), "webp");
        assert_eq!(file_extension("", Some("video/mp4"), false), "mp4");
    }

    #[test]
    fn test_video_extension_fallback() {
        // For videos with no recognizable extension, default to mp4
        assert_eq!(file_extension("https://x.com/stream", None, true), "mp4");
    }

    #[test]
    fn test_now_utc_string_is_numeric() {
        let ts = now_utc_string();
        let n: u64 = ts.parse().unwrap();
        assert!(n > 1700000000); // after 2023
    }
}

// ---- Download queue ----

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueItem {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub progress: f64,
    pub status: String,
    pub error: Option<String>,
}

static QUEUE: StdMutex<Vec<QueueItem>> = StdMutex::new(Vec::new());

pub fn get_queue() -> Vec<QueueItem> {
    QUEUE.lock().map(|q| q.clone()).unwrap_or_default()
}

pub fn clear_queue() {
    if let Ok(mut q) = QUEUE.lock() {
        q.clear();
    }
}

pub async fn download_batch(
    items: Vec<WallpaperItem>,
) -> Result<Vec<DownloadResult>, String> {
    let mut results = Vec::new();

    {
        let mut q = QUEUE.lock().map_err(|e| format!("Lock: {e}"))?;
        for item in &items {
            q.push(QueueItem {
                id: item.id.clone(),
                title: item.title.clone(),
                kind: item.kind.clone(),
                progress: 0.0,
                status: "queued".to_string(),
                error: None,
            });
        }
    }

    for item in &items {
        {
            let mut q = QUEUE.lock().map_err(|e| format!("Lock: {e}"))?;
            if let Some(qi) = q.iter_mut().find(|qi| qi.id == item.id) {
                qi.status = "downloading".to_string();
                qi.progress = 0.5;
            }
        }

        let result = download_wallpaper(item.clone()).await;

        {
            let mut q = QUEUE.lock().map_err(|e| format!("Lock: {e}"))?;
            if let Some(qi) = q.iter_mut().find(|qi| qi.id == item.id) {
                match &result {
                    Ok(_) => {
                        qi.status = "done".to_string();
                        qi.progress = 1.0;
                    }
                    Err(e) => {
                        qi.status = "error".to_string();
                        qi.error = Some(e.clone());
                    }
                }
            }
        }

        results.push(result);
    }

    results.into_iter().collect()
}
