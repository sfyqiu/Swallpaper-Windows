use crate::sources::WallpaperItem;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs,
    io::Write,
    path::{Path, PathBuf},
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
    file_path: String,
    records: u32,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LibraryRecord {
    id: String,
    kind: String,
    source: String,
    title: String,
    author: Option<String>,
    detail_url: String,
    remote_url: String,
    relative_file_path: String,
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

pub async fn download_wallpaper(item: WallpaperItem) -> Result<DownloadResult, String> {
    let root = library_root()?;
    ensure_layout(&root)?;

    let response = reqwest::Client::new()
        .get(&item.image_url)
        .header("User-Agent", "Swallpaper-Windows/0.1")
        .send()
        .await
        .map_err(|error| format!("Download request failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Download HTTP error: {error}"))?;

    let extension = image_extension(&item.image_url, response.headers().get("content-type").and_then(|v| v.to_str().ok()));
    let safe_id = sanitize_filename(&item.id);
    let relative = format!("files/wallpapers/{safe_id}.{extension}");
    let absolute = root.join(&relative);
    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("Unable to read download body: {error}"))?;

    let mut file = fs::File::create(&absolute).map_err(|error| format!("Unable to create {}: {error}", absolute.display()))?;
    file.write_all(&bytes)
        .map_err(|error| format!("Unable to write {}: {error}", absolute.display()))?;

    let mut records = read_records(&root);
    records.retain(|record| record.id != item.id);
    records.insert(
        0,
        LibraryRecord {
            id: item.id.clone(),
            kind: "staticWallpaper".to_string(),
            source: item.source,
            title: item.title,
            author: item.author,
            detail_url: item.detail_url,
            remote_url: item.image_url,
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
        file_path: absolute.display().to_string(),
        records: records.len() as u32,
    })
}

fn image_extension(url: &str, content_type: Option<&str>) -> &'static str {
    if let Some(content_type) = content_type {
        if content_type.contains("png") {
            return "png";
        }
        if content_type.contains("webp") {
            return "webp";
        }
    }

    let lower = url.split('?').next().unwrap_or(url).to_lowercase();
    if lower.ends_with(".png") {
        "png"
    } else if lower.ends_with(".webp") {
        "webp"
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
