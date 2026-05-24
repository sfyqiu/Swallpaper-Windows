use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    pub source: String,
    pub query: Option<String>,
    pub page: Option<u32>,
    pub api_key: Option<String>,
    pub media_type: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceCapabilities {
    pub requires_api_key: bool,
    pub supports_search: bool,
    pub supports_categories: bool,
    pub supports_color: bool,
    pub supports_nsfw: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: &'static str,
    pub media_types: &'static [&'static str],
    pub description: &'static str,
    pub capabilities: SourceCapabilities,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WallpaperItem {
    pub id: String,
    pub source: String,
    pub kind: String,
    pub title: String,
    pub author: Option<String>,
    pub detail_url: String,
    pub image_url: String,
    pub thumbnail_url: String,
    pub video_url: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub purity: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub source: String,
    pub page: u32,
    pub has_more: bool,
    pub items: Vec<WallpaperItem>,
}

pub fn list_sources() -> Vec<SourceInfo> {
    vec![
        SourceInfo {
            id: "wallhaven",
            name: "Wallhaven",
            kind: "static",
            media_types: &["static"],
            description: "Mac 版主静态壁纸源，支持搜索、分类、颜色与 NSFW key 解锁。",
            capabilities: SourceCapabilities {
                requires_api_key: false,
                supports_search: true,
                supports_categories: true,
                supports_color: true,
                supports_nsfw: true,
            },
        },
        SourceInfo {
            id: "pexels",
            name: "Pexels",
            kind: "static",
            media_types: &["static"],
            description: "高质量摄影壁纸源，需要用户自己的 Pexels API Key。",
            capabilities: SourceCapabilities {
                requires_api_key: true,
                supports_search: true,
                supports_categories: false,
                supports_color: false,
                supports_nsfw: false,
            },
        },
        SourceInfo {
            id: "unsplash",
            name: "Unsplash",
            kind: "static",
            media_types: &["static"],
            description: "摄影图库壁纸源，需要用户自己的 Unsplash Access Key。",
            capabilities: SourceCapabilities {
                requires_api_key: true,
                supports_search: true,
                supports_categories: false,
                supports_color: false,
                supports_nsfw: false,
            },
        },
        SourceInfo {
            id: "nasa_apod",
            name: "NASA APOD",
            kind: "static",
            media_types: &["static"],
            description: "NASA 每日天文图，支持 DEMO_KEY 或用户 NASA API Key。",
            capabilities: SourceCapabilities {
                requires_api_key: false,
                supports_search: false,
                supports_categories: false,
                supports_color: false,
                supports_nsfw: false,
            },
        },
        SourceInfo {
            id: "coverr",
            name: "Coverr",
            kind: "video",
            media_types: &["video"],
            description: "免费动态壁纸视频源，无需 API Key，支持搜索。",
            capabilities: SourceCapabilities {
                requires_api_key: false,
                supports_search: true,
                supports_categories: false,
                supports_color: false,
                supports_nsfw: false,
            },
        },
        SourceInfo {
            id: "pexels_videos",
            name: "Pexels Videos",
            kind: "video",
            media_types: &["video"],
            description: "Pexels 视频壁纸源，需要用户自己的 Pexels API Key。",
            capabilities: SourceCapabilities {
                requires_api_key: true,
                supports_search: true,
                supports_categories: false,
                supports_color: false,
                supports_nsfw: false,
            },
        },
    ]
}

pub async fn search(request: SearchRequest) -> Result<SearchResponse, String> {
    let source = request.source.as_str();
    match source {
        "wallhaven" => search_wallhaven(request).await,
        "pexels" => search_pexels(request).await,
        "unsplash" => search_unsplash(request).await,
        "nasa_apod" => fetch_nasa_apod(request).await,
        "coverr" => search_coverr(request).await,
        "pexels_videos" => search_pexels_videos(request).await,
        other => Err(format!("Unsupported wallpaper source: {other}")),
    }
}

fn clean_query(query: Option<String>, fallback: &str) -> String {
    let value = query.unwrap_or_default().trim().to_string();
    if value.is_empty() {
        fallback.to_string()
    } else {
        value
    }
}

fn clean_api_key(api_key: Option<String>) -> Option<String> {
    api_key
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty())
}

// ---- Wallhaven ----

async fn search_wallhaven(request: SearchRequest) -> Result<SearchResponse, String> {
    #[derive(Deserialize)]
    struct WallhavenThumbs {
        small: String,
        large: String,
        original: String,
    }

    #[derive(Deserialize)]
    struct WallhavenItem {
        id: String,
        url: String,
        purity: String,
        dimension_x: u32,
        dimension_y: u32,
        path: String,
        thumbs: WallhavenThumbs,
    }

    #[derive(Deserialize)]
    struct WallhavenMeta {
        current_page: u32,
        last_page: u32,
    }

    #[derive(Deserialize)]
    struct WallhavenResponse {
        data: Vec<WallhavenItem>,
        meta: WallhavenMeta,
    }

    let page = request.page.unwrap_or(1).max(1);
    let query = request.query.unwrap_or_default();
    let mut url = format!(
        "https://wallhaven.cc/api/v1/search?q={}&categories=111&purity=100&sorting=date_added&order=desc&page={page}",
        urlencoding::encode(query.trim())
    );
    if let Some(key) = clean_api_key(request.api_key) {
        url.push_str("&apikey=");
        url.push_str(&urlencoding::encode(&key));
    }

    let response: WallhavenResponse = reqwest::Client::new()
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|error| format!("Wallhaven request failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Wallhaven HTTP error: {error}"))?
        .json()
        .await
        .map_err(|error| format!("Wallhaven JSON parse failed: {error}"))?;

    let items = response
        .data
        .into_iter()
        .map(|item| WallpaperItem {
            id: item.id.clone(),
            source: "wallhaven".to_string(),
            kind: "staticWallpaper".to_string(),
            title: format!("Wallhaven {}", item.id),
            author: None,
            detail_url: item.url,
            image_url: item.path,
            thumbnail_url: if item.thumbs.large.is_empty() {
                if item.thumbs.original.is_empty() {
                    item.thumbs.small
                } else {
                    item.thumbs.original
                }
            } else {
                item.thumbs.large
            },
            video_url: None,
            width: Some(item.dimension_x),
            height: Some(item.dimension_y),
            purity: Some(item.purity),
        })
        .collect();

    Ok(SearchResponse {
        source: "wallhaven".to_string(),
        page: response.meta.current_page,
        has_more: response.meta.current_page < response.meta.last_page,
        items,
    })
}

// ---- Pexels Photos ----

async fn search_pexels(request: SearchRequest) -> Result<SearchResponse, String> {
    #[derive(Deserialize)]
    struct PexelsSrc {
        original: String,
        large2x: Option<String>,
        large: String,
        medium: String,
    }

    #[derive(Deserialize)]
    struct PexelsPhoto {
        id: u64,
        width: u32,
        height: u32,
        url: String,
        photographer: String,
        alt: Option<String>,
        src: PexelsSrc,
    }

    #[derive(Deserialize)]
    struct PexelsResponse {
        page: u32,
        per_page: u32,
        total_results: u32,
        photos: Vec<PexelsPhoto>,
    }

    let key = clean_api_key(request.api_key).ok_or("Pexels API Key is required.")?;
    let page = request.page.unwrap_or(1).max(1);
    let query = clean_query(request.query, "wallpaper");
    let url = format!(
        "https://api.pexels.com/v1/search?query={}&orientation=landscape&per_page=30&page={page}",
        urlencoding::encode(&query)
    );

    let response: PexelsResponse = reqwest::Client::new()
        .get(url)
        .header("Authorization", key)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|error| format!("Pexels request failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Pexels HTTP error: {error}"))?
        .json()
        .await
        .map_err(|error| format!("Pexels JSON parse failed: {error}"))?;

    let total_pages = (response.total_results as f32 / response.per_page.max(1) as f32).ceil() as u32;
    let items = response
        .photos
        .into_iter()
        .map(|photo| WallpaperItem {
            id: format!("pexels-{}", photo.id),
            source: "pexels".to_string(),
            kind: "staticWallpaper".to_string(),
            title: photo.alt.unwrap_or_else(|| format!("Pexels {}", photo.id)),
            author: Some(photo.photographer),
            detail_url: photo.url,
            image_url: photo.src.original,
            thumbnail_url: photo.src.large2x.unwrap_or_else(|| {
                if photo.src.large.is_empty() {
                    photo.src.medium
                } else {
                    photo.src.large
                }
            }),
            video_url: None,
            width: Some(photo.width),
            height: Some(photo.height),
            purity: Some("sfw".to_string()),
        })
        .collect();

    Ok(SearchResponse {
        source: "pexels".to_string(),
        page: response.page,
        has_more: response.page < total_pages,
        items,
    })
}

// ---- Unsplash ----

async fn search_unsplash(request: SearchRequest) -> Result<SearchResponse, String> {
    #[derive(Deserialize)]
    struct UnsplashUrls {
        raw: String,
        regular: String,
        small: String,
    }

    #[derive(Deserialize)]
    struct UnsplashUser {
        name: String,
    }

    #[derive(Deserialize)]
    struct UnsplashPhoto {
        id: String,
        width: u32,
        height: u32,
        description: Option<String>,
        alt_description: Option<String>,
        links: UnsplashLinks,
        urls: UnsplashUrls,
        user: UnsplashUser,
    }

    #[derive(Deserialize)]
    struct UnsplashLinks {
        html: String,
    }

    #[derive(Deserialize)]
    struct UnsplashResponse {
        total_pages: u32,
        results: Vec<UnsplashPhoto>,
    }

    let key = clean_api_key(request.api_key).ok_or("Unsplash Access Key is required.")?;
    let page = request.page.unwrap_or(1).max(1);
    let query = clean_query(request.query, "wallpaper");
    let url = format!(
        "https://api.unsplash.com/search/photos?query={}&orientation=landscape&per_page=30&page={page}",
        urlencoding::encode(&query)
    );

    let response: UnsplashResponse = reqwest::Client::new()
        .get(url)
        .header("Authorization", format!("Client-ID {key}"))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|error| format!("Unsplash request failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Unsplash HTTP error: {error}"))?
        .json()
        .await
        .map_err(|error| format!("Unsplash JSON parse failed: {error}"))?;

    let items = response
        .results
        .into_iter()
        .map(|photo| WallpaperItem {
            id: format!("unsplash-{}", photo.id),
            source: "unsplash".to_string(),
            kind: "staticWallpaper".to_string(),
            title: photo
                .description
                .or(photo.alt_description)
                .unwrap_or_else(|| format!("Unsplash {}", photo.id)),
            author: Some(photo.user.name),
            detail_url: photo.links.html,
            image_url: photo.urls.raw,
            thumbnail_url: if photo.urls.regular.is_empty() {
                photo.urls.small
            } else {
                photo.urls.regular
            },
            video_url: None,
            width: Some(photo.width),
            height: Some(photo.height),
            purity: Some("sfw".to_string()),
        })
        .collect();

    Ok(SearchResponse {
        source: "unsplash".to_string(),
        page,
        has_more: page < response.total_pages,
        items,
    })
}

// ---- NASA APOD ----

async fn fetch_nasa_apod(request: SearchRequest) -> Result<SearchResponse, String> {
    #[derive(Deserialize)]
    struct ApodItem {
        title: String,
        url: Option<String>,
        hdurl: Option<String>,
        media_type: String,
        date: String,
        copyright: Option<String>,
    }

    let key = clean_api_key(request.api_key).unwrap_or_else(|| "DEMO_KEY".to_string());
    let url = format!(
        "https://api.nasa.gov/planetary/apod?api_key={}&count=12&thumbs=true",
        urlencoding::encode(&key)
    );

    let response: Vec<ApodItem> = reqwest::Client::new()
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|error| format!("NASA APOD request failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("NASA APOD HTTP error: {error}"))?
        .json()
        .await
        .map_err(|error| format!("NASA APOD JSON parse failed: {error}"))?;

    let items = response
        .into_iter()
        .filter(|item| item.media_type == "image")
        .filter_map(|item| {
            let image_url = item.hdurl.or(item.url)?;
            Some(WallpaperItem {
                id: format!("nasa-apod-{}", item.date),
                source: "nasa_apod".to_string(),
                kind: "staticWallpaper".to_string(),
                title: item.title,
                author: item.copyright,
                detail_url: image_url.clone(),
                image_url: image_url.clone(),
                thumbnail_url: image_url,
                video_url: None,
                width: None,
                height: None,
                purity: Some("sfw".to_string()),
            })
        })
        .collect();

    Ok(SearchResponse {
        source: "nasa_apod".to_string(),
        page: 1,
        has_more: false,
        items,
    })
}

// ---- Coverr ----

async fn search_coverr(request: SearchRequest) -> Result<SearchResponse, String> {
    #[derive(Deserialize)]
    struct CoverrUrls {
        mp4: Option<String>,
        mp4_download: Option<String>,
        webm: Option<String>,
        preview: Option<String>,
    }

    #[derive(Deserialize)]
    struct CoverrItem {
        id: String,
        title: Option<String>,
        description: Option<String>,
        urls: CoverrUrls,
        thumbnail: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
    }

    #[derive(Deserialize)]
    struct CoverrResponse {
        hits: Vec<CoverrItem>,
        total: Option<u32>,
        page: Option<u32>,
    }

    let page = request.page.unwrap_or(1).max(1);
    let query = clean_query(request.query, "nature");

    let client = reqwest::Client::new();
    let mut url = format!(
        "https://api.coverr.co/videos?query={}&page={page}&per_page=24",
        urlencoding::encode(&query)
    );
    if let Some(key) = clean_api_key(request.api_key) {
        url.push_str("&api_key=");
        url.push_str(&key);
    }

    let response: CoverrResponse = client
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|error| format!("Coverr request failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Coverr HTTP error: {error}"))?
        .json()
        .await
        .map_err(|error| format!("Coverr JSON parse failed: {error}"))?;

    let total = response.total.unwrap_or(0);
    let items: Vec<WallpaperItem> = response
        .hits
        .into_iter()
        .map(|item| {
            let video_url = item
                .urls
                .mp4_download
                .or(item.urls.mp4)
                .or(item.urls.webm);
            let thumb = item.thumbnail.unwrap_or_default();
            WallpaperItem {
                id: format!("coverr-{}", item.id),
                source: "coverr".to_string(),
                kind: "videoWallpaper".to_string(),
                title: item.title.unwrap_or_else(|| format!("Coverr {}", item.id)),
                author: None,
                detail_url: format!("https://coverr.co/video/{}", item.id),
                image_url: thumb.clone(),
                thumbnail_url: thumb,
                video_url,
                width: item.width,
                height: item.height,
                purity: Some("sfw".to_string()),
            }
        })
        .collect();

    let has_more = page * 24 < total;

    Ok(SearchResponse {
        source: "coverr".to_string(),
        page,
        has_more,
        items,
    })
}

// ---- Pexels Videos ----

async fn search_pexels_videos(request: SearchRequest) -> Result<SearchResponse, String> {
    #[derive(Deserialize)]
    struct PexelsVideoFile {
        id: u64,
        quality: String,
        file_type: String,
        width: Option<u32>,
        height: Option<u32>,
        link: String,
    }

    #[derive(Deserialize)]
    struct PexelsVideoPicture {
        id: u64,
        picture: String,
    }

    #[derive(Deserialize)]
    struct PexelsVideo {
        id: u64,
        width: u32,
        height: u32,
        url: String,
        image: Option<String>,
        duration: Option<u32>,
        user: PexelsUser,
        video_files: Vec<PexelsVideoFile>,
        video_pictures: Vec<PexelsVideoPicture>,
    }

    #[derive(Deserialize)]
    struct PexelsUser {
        name: String,
    }

    #[derive(Deserialize)]
    struct PexelsVideosResponse {
        page: u32,
        per_page: u32,
        total_results: u32,
        videos: Vec<PexelsVideo>,
    }

    let key = clean_api_key(request.api_key).ok_or("Pexels API Key is required.")?;
    let page = request.page.unwrap_or(1).max(1);
    let query = clean_query(request.query, "nature");
    let url = format!(
        "https://api.pexels.com/videos/search?query={}&orientation=landscape&per_page=30&page={page}",
        urlencoding::encode(&query)
    );

    let response: PexelsVideosResponse = reqwest::Client::new()
        .get(url)
        .header("Authorization", key)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|error| format!("Pexels Videos request failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Pexels Videos HTTP error: {error}"))?
        .json()
        .await
        .map_err(|error| format!("Pexels Videos JSON parse failed: {error}"))?;

    let total_pages = (response.total_results as f32 / response.per_page.max(1) as f32).ceil() as u32;

    let items: Vec<WallpaperItem> = response
        .videos
        .into_iter()
        .map(|video| {
            // Prefer HD quality, fall back to the largest available
            let best_file = video
                .video_files
                .iter()
                .find(|f| f.quality == "hd")
                .or_else(|| video.video_files.iter().max_by_key(|f| f.width.unwrap_or(0)))
                .or_else(|| video.video_files.first());

            let video_url = best_file.map(|f| f.link.clone());
            let thumb = video
                .image
                .or_else(|| {
                    video
                        .video_pictures
                        .first()
                        .map(|p| p.picture.clone())
                })
                .unwrap_or_default();

            WallpaperItem {
                id: format!("pexels-video-{}", video.id),
                source: "pexels_videos".to_string(),
                kind: "videoWallpaper".to_string(),
                title: format!("Pexels Video {}", video.id),
                author: Some(video.user.name),
                detail_url: video.url,
                image_url: thumb.clone(),
                thumbnail_url: thumb,
                video_url,
                width: Some(video.width),
                height: Some(video.height),
                purity: Some("sfw".to_string()),
            }
        })
        .collect();

    Ok(SearchResponse {
        source: "pexels_videos".to_string(),
        page: response.page,
        has_more: response.page < total_pages,
        items,
    })
}
