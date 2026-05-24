use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    pub source: String,
    pub query: Option<String>,
    pub page: Option<u32>,
    pub api_key: Option<String>,
    pub media_type: Option<String>,
    pub nsfw_enabled: Option<bool>,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiTestResult {
    pub source_id: String,
    pub source_name: String,
    pub ok: bool,
    pub latency_ms: u64,
    pub error: Option<String>,
}

pub fn list_sources() -> Vec<SourceInfo> {
    vec![
        SourceInfo {
            id: "wallhaven",
            name: "Wallhaven",
            kind: "static",
            media_types: &["static"],
            description: "主静态壁纸源，支持搜索、分类、颜色与 NSFW key 解锁。",
            capabilities: SourceCapabilities {
                requires_api_key: false,
                supports_search: true,
                supports_categories: true,
                supports_color: true,
                supports_nsfw: true,
            },
        },
        SourceInfo {
            id: "fourk",
            name: "4K Wallpapers",
            kind: "static",
            media_types: &["static"],
            description: "4K/8K 高清壁纸，无需 API Key，支持分类浏览。",
            capabilities: SourceCapabilities {
                requires_api_key: false,
                supports_search: true,
                supports_categories: true,
                supports_color: false,
                supports_nsfw: false,
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
            id: "nasa_images",
            name: "NASA Images",
            kind: "static",
            media_types: &["static"],
            description: "NASA 公开图片库，无需 API Key，海量天文图片。",
            capabilities: SourceCapabilities {
                requires_api_key: false,
                supports_search: true,
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
        SourceInfo {
            id: "motionbg",
            name: "MotionBG",
            kind: "video",
            media_types: &["video"],
            description: "动态壁纸源，无需 API Key，支持搜索。",
            capabilities: SourceCapabilities {
                requires_api_key: false,
                supports_search: true,
                supports_categories: true,
                supports_color: false,
                supports_nsfw: false,
            },
        },
        SourceInfo {
            id: "we_workshop",
            name: "WE Workshop",
            kind: "video",
            media_types: &["video", "web"],
            description: "Wallpaper Engine 创意工坊本地内容（需 Steam + WE 订阅）。",
            capabilities: SourceCapabilities {
                requires_api_key: false,
                supports_search: false,
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
        "fourk" => search_fourk(request).await,
        "pexels" => search_pexels(request).await,
        "unsplash" => search_unsplash(request).await,
        "nasa_apod" => fetch_nasa_apod(request).await,
        "nasa_images" => search_nasa_images(request).await,
        "coverr" => search_coverr(request).await,
        "pexels_videos" => search_pexels_videos(request).await,
        "motionbg" => search_motionbg(request).await,
        "we_workshop" => scan_we_workshop(request).await,
        other => Err(format!("Unsupported wallpaper source: {other}")),
    }
}

pub async fn test_api_connectivity() -> Vec<ApiTestResult> {
    let sources = list_sources();
    let mut results: Vec<ApiTestResult> = Vec::new();

    for source in &sources {
        if source.id == "we_workshop" {
            results.push(ApiTestResult {
                source_id: source.id.to_string(),
                source_name: source.name.to_string(),
                ok: true,
                latency_ms: 0,
                error: None,
            });
            continue;
        }

        let req = SearchRequest {
            source: source.id.to_string(),
            query: Some("test".to_string()),
            page: Some(1),
            api_key: None,
            media_type: None,
            nsfw_enabled: Some(false),
        };

        let start = std::time::Instant::now();
        let result = search(req).await;
        let latency_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(resp) => {
                results.push(ApiTestResult {
                    source_id: source.id.to_string(),
                    source_name: source.name.to_string(),
                    ok: !resp.items.is_empty(),
                    latency_ms,
                    error: if resp.items.is_empty() {
                        Some("No results returned".to_string())
                    } else {
                        None
                    },
                });
            }
            Err(err) => {
                results.push(ApiTestResult {
                    source_id: source.id.to_string(),
                    source_name: source.name.to_string(),
                    ok: false,
                    latency_ms,
                    error: Some(err),
                });
            }
        }
    }

    results
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

fn is_nsfw_enabled(request: &SearchRequest) -> bool {
    request.nsfw_enabled.unwrap_or(false)
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
    let nsfw = is_nsfw_enabled(&request);
    let query = request.query.unwrap_or_default();
    let purity = if nsfw { "110" } else { "100" };
    let mut url = format!(
        "https://wallhaven.cc/api/v1/search?q={}&categories=111&purity={purity}&sorting=date_added&order=desc&page={page}",
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

// ---- 4K Wallpapers ----

async fn search_fourk(request: SearchRequest) -> Result<SearchResponse, String> {
    let page = request.page.unwrap_or(1).max(1);
    let query = clean_query(request.query, "anime");
    let url = format!(
        "https://4kwallpapers.com/wallpapers/search/?q={}&page={page}",
        urlencoding::encode(&query)
    );

    let client = reqwest::Client::new();
    let html = client
        .get(&url)
        .header("User-Agent", "Swallpaper-Windows/0.1")
        .header("Accept", "text/html")
        .send()
        .await
        .map_err(|e| format!("4K Wallpapers request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("4K Wallpapers HTTP error: {e}"))?
        .text()
        .await
        .map_err(|e| format!("4K Wallpapers response error: {e}"))?;

    let items = parse_fourk_html(&html);
    let has_more = items.len() >= 20;

    Ok(SearchResponse {
        source: "fourk".to_string(),
        page,
        has_more,
        items,
    })
}

fn parse_fourk_html(html: &str) -> Vec<WallpaperItem> {
    let mut items = Vec::new();
    let mut pos = 0usize;

    // Parse simple wallpapers cards: look for img tags with data-src in wallpaper cards
    while let Some(card_start) = html[pos..].find("data-wallpaper-id=\"") {
        let abs_start = pos + card_start;
        let id_start = abs_start + "data-wallpaper-id=\"".len();
        let id_end = match html[id_start..].find('"') {
            Some(p) => id_start + p,
            None => break,
        };
        let id = &html[id_start..id_end];

        // Find thumbnail
        let thumb = html[id_end..]
            .find("data-src=\"")
            .and_then(|p| {
                let s = id_end + p + "data-src=\"".len();
                html[s..].find('"').map(|e| html[s..s + e].to_string())
            })
            .unwrap_or_default();

        // Find full image URL
        let full_url = format!("https://4kwallpapers.com/wallpapers/{id}/");

        // Find title
        let title = html[id_end..]
            .find("alt=\"")
            .and_then(|p| {
                let s = id_end + p + "alt=\"".len();
                html[s..].find('"').map(|e| html[s..s + e].to_string())
            })
            .unwrap_or_else(|| format!("4K Wallpaper {id}"));

        items.push(WallpaperItem {
            id: format!("fourk-{id}"),
            source: "fourk".to_string(),
            kind: "staticWallpaper".to_string(),
            title,
            author: None,
            detail_url: full_url,
            image_url: if thumb.is_empty() { format!("https://4kwallpapers.com/wallpapers/{id}/download/") } else { thumb.clone() },
            thumbnail_url: thumb,
            video_url: None,
            width: None,
            height: None,
            purity: Some("sfw".to_string()),
        });

        pos = id_end + 1;
        if items.len() >= 24 {
            break;
        }
    }

    items
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

// ---- NASA Images ----

async fn search_nasa_images(request: SearchRequest) -> Result<SearchResponse, String> {
    let page = request.page.unwrap_or(1).max(1);
    let query = clean_query(request.query, "galaxy");

    #[derive(Deserialize)]
    struct NasaCollectionItem {
        data: Vec<NasaDataItem>,
        links: Option<Vec<NasaLinkItem>>,
    }

    #[derive(Deserialize)]
    struct NasaDataItem {
        nasa_id: Option<String>,
        title: Option<String>,
        description: Option<String>,
        date_created: Option<String>,
    }

    #[derive(Deserialize)]
    struct NasaLinkItem {
        href: Option<String>,
        rel: Option<String>,
    }

    #[derive(Deserialize)]
    struct NasaResponse {
        collection: NasaCollection,
    }

    #[derive(Deserialize)]
    struct NasaCollection {
        items: Vec<NasaCollectionItem>,
        metadata: Option<NasaMetadata>,
    }

    #[derive(Deserialize)]
    struct NasaMetadata {
        total_hits: Option<u32>,
    }

    let url = format!(
        "https://images-api.nasa.gov/search?q={}&media_type=image&page={page}",
        urlencoding::encode(&query)
    );

    let response: NasaResponse = reqwest::Client::new()
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("NASA Images request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("NASA Images HTTP error: {e}"))?
        .json()
        .await
        .map_err(|e| format!("NASA Images JSON parse failed: {e}"))?;

    let total = response.collection.metadata.and_then(|m| m.total_hits).unwrap_or(0);
    let items: Vec<WallpaperItem> = response
        .collection
        .items
        .into_iter()
        .filter_map(|item| {
            let data = item.data.into_iter().next()?;
            let id = data.nasa_id.clone().unwrap_or_else(|| format!("nasa-{}", rand_id()));
            let thumb = item
                .links
                .as_ref()
                .and_then(|links| links.iter().find(|l| l.rel.as_deref() == Some("preview")))
                .and_then(|l| l.href.clone())
                .unwrap_or_default();
            let full_url = format!("https://images-assets.nasa.gov/image/{}/{}~orig.jpg", id, id);

            Some(WallpaperItem {
                id: format!("nasa-img-{id}"),
                source: "nasa_images".to_string(),
                kind: "staticWallpaper".to_string(),
                title: data.title.unwrap_or_else(|| "NASA Image".to_string()),
                author: None,
                detail_url: full_url.clone(),
                image_url: full_url,
                thumbnail_url: thumb,
                video_url: None,
                width: None,
                height: None,
                purity: Some("sfw".to_string()),
            })
        })
        .collect();

    Ok(SearchResponse {
        source: "nasa_images".to_string(),
        page,
        has_more: items.len() >= 100,
        items,
    })
}

fn rand_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let n = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    n as u64 ^ (std::process::id() as u64)
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

// ---- MotionBG ----

async fn search_motionbg(request: SearchRequest) -> Result<SearchResponse, String> {
    let page = request.page.unwrap_or(1).max(1);
    let query = clean_query(request.query, "anime");

    let url = format!(
        "https://motionbgs.com/api/videos?search={}&page={page}&per_page=24",
        urlencoding::encode(&query)
    );

    #[derive(Deserialize)]
    struct MbgsVideo {
        id: Option<u64>,
        title: Option<String>,
        thumbnail: Option<String>,
        video_url: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
    }

    #[derive(Deserialize)]
    struct MbgsResponse {
        videos: Option<Vec<MbgsVideo>>,
        total: Option<u32>,
        page: Option<u32>,
    }

    let client = reqwest::Client::new();
    let response: MbgsResponse = client
        .get(&url)
        .header("User-Agent", "Swallpaper-Windows/0.1")
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("MotionBG request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("MotionBG HTTP error: {e}"))?
        .json()
        .await
        .map_err(|e| format!("MotionBG JSON parse failed: {e}"))?;

    let videos = response.videos.unwrap_or_default();
    let total = response.total.unwrap_or(0);
    let items: Vec<WallpaperItem> = videos
        .into_iter()
        .map(|v| {
            let id_str = v.id.map(|i| i.to_string()).unwrap_or_else(|| rand_id().to_string());
            WallpaperItem {
                id: format!("motionbg-{id_str}"),
                source: "motionbg".to_string(),
                kind: "videoWallpaper".to_string(),
                title: v.title.unwrap_or_else(|| "MotionBG".to_string()),
                author: None,
                detail_url: v.video_url.clone().unwrap_or_default(),
                image_url: v.thumbnail.clone().unwrap_or_default(),
                thumbnail_url: v.thumbnail.unwrap_or_default(),
                video_url: v.video_url,
                width: v.width,
                height: v.height,
                purity: Some("sfw".to_string()),
            }
        })
        .collect();

    Ok(SearchResponse {
        source: "motionbg".to_string(),
        page,
        has_more: (page * 24) < total,
        items,
    })
}

// ---- WE Workshop (local Steam directory) ----

async fn scan_we_workshop(request: SearchRequest) -> Result<SearchResponse, String> {
    let page = request.page.unwrap_or(1).max(1);
    let query = request.query.unwrap_or_default().to_lowercase();

    let steam_paths = vec![
        r"C:\Program Files (x86)\Steam\steamapps\workshop\content\431960",
        r"C:\Steam\steamapps\workshop\content\431960",
        r"D:\Steam\steamapps\workshop\content\431960",
    ];

    let mut workshop_dir = None;
    for p in &steam_paths {
        let path = std::path::Path::new(p);
        if path.is_dir() {
            workshop_dir = Some(path.to_path_buf());
            break;
        }
    }

    let ws_dir = match workshop_dir {
        Some(d) => d,
        None => {
            return Ok(SearchResponse {
                source: "we_workshop".to_string(),
                page: 1,
                has_more: false,
                items: Vec::new(),
            });
        }
    };

    let mut items: Vec<WallpaperItem> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&ws_dir) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let dir = entry.path();
            let dir_name = dir.file_name().unwrap().to_string_lossy().to_string();

            // Check for project.json (scene type) — skip complex 3D scenes
            let project_json = dir.join("project.json");
            // Look for mp4 files (video type) or index.html (web type)
            let has_video = std::fs::read_dir(&dir)
                .map(|d| d.flatten().any(|e| {
                    e.path().extension().map(|ext| ext == "mp4").unwrap_or(false)
                }))
                .unwrap_or(false);
            let has_html = dir.join("index.html").is_file();

            if !has_video && !has_html {
                continue;
            }

            let title = dir_name.clone();
            if !query.is_empty() && !title.to_lowercase().contains(&query) {
                continue;
            }

            let kind = if has_video { "videoWallpaper" } else { "webWallpaper" };
            let video_url = if has_video {
                std::fs::read_dir(&dir)
                    .ok()
                    .and_then(|d| {
                        d.flatten()
                            .find(|e| e.path().extension().map(|ext| ext == "mp4").unwrap_or(false))
                            .map(|e| e.path().display().to_string())
                    })
            } else {
                None
            };

            let thumb = dir.join("thumb.png");
            let thumbnail_url = if thumb.is_file() {
                thumb.display().to_string()
            } else {
                String::new()
            };

            let preview_url = if has_html {
                dir.join("index.html").display().to_string()
            } else {
                video_url.clone().unwrap_or_default()
            };

            items.push(WallpaperItem {
                id: format!("we-{dir_name}"),
                source: "we_workshop".to_string(),
                kind: kind.to_string(),
                title,
                author: None,
                detail_url: preview_url.clone(),
                image_url: thumbnail_url.clone(),
                thumbnail_url,
                video_url,
                width: None,
                height: None,
                purity: Some("sfw".to_string()),
            });
        }
    }

    // Sort by title for consistent ordering
    items.sort_by(|a, b| a.title.cmp(&b.title));

    let per_page = 24usize;
    let start = ((page - 1) as usize) * per_page;
    let total = items.len();
    let paged: Vec<WallpaperItem> = items.into_iter().skip(start).take(per_page).collect();
    let has_more = start + per_page < total;

    Ok(SearchResponse {
        source: "we_workshop".to_string(),
        page,
        has_more,
        items: paged,
    })
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_query_default() {
        assert_eq!(clean_query(None, "fallback"), "fallback");
        assert_eq!(clean_query(Some("  ".to_string()), "fallback"), "fallback");
        assert_eq!(clean_query(Some("hello".to_string()), "fallback"), "hello");
    }

    #[test]
    fn test_clean_api_key() {
        assert_eq!(clean_api_key(None), None);
        assert_eq!(clean_api_key(Some("  ".to_string())), None);
        assert_eq!(clean_api_key(Some("abc123".to_string())), Some("abc123".to_string()));
    }

    #[test]
    fn test_is_nsfw_disabled_by_default() {
        let req = SearchRequest {
            source: "wallhaven".to_string(),
            query: None,
            page: None,
            api_key: None,
            media_type: None,
            nsfw_enabled: None,
        };
        assert!(!is_nsfw_enabled(&req));
    }

    #[test]
    fn test_is_nsfw_enabled() {
        let req = SearchRequest {
            source: "wallhaven".to_string(),
            query: None,
            page: None,
            api_key: None,
            media_type: None,
            nsfw_enabled: Some(true),
        };
        assert!(is_nsfw_enabled(&req));
    }

    #[test]
    fn test_parse_fourk_html_empty() {
        let items = parse_fourk_html("");
        assert!(items.is_empty());
    }

    #[test]
    fn test_parse_fourk_html_with_card() {
        let html = r#"<div data-wallpaper-id="12345"><img data-src="https://example.com/thumb.jpg" alt="Test Wallpaper" /></div>"#;
        let items = parse_fourk_html(html);
        assert!(!items.is_empty());
        assert_eq!(items[0].id, "fourk-12345");
        assert_eq!(items[0].kind, "staticWallpaper");
    }

    #[test]
    fn test_list_sources_count() {
        let sources = list_sources();
        assert!(sources.len() >= 10);
    }

    #[test]
    fn test_source_ids_unique() {
        let sources = list_sources();
        let mut ids: Vec<&str> = sources.iter().map(|s| s.id).collect();
        ids.sort();
        let orig_len = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), orig_len, "Source IDs must be unique");
    }
}
