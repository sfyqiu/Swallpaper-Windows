import React from "react";
import ReactDOM from "react-dom/client";
import {
  Cloud, Download, ExternalLink, Heart, Image, Info, KeyRound,
  Pause, Play, RefreshCw, RotateCw, Search, Settings, Square, Star,
  Video, Wifi, Home, Film, FolderOpen,
} from "lucide-react";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import "./styles.css";
import { t, getLang, setLang, getAvailableLangs, type Lang } from "./i18n";

type CommandResult<T> = { ok: boolean; data?: T; error?: string; };
type LibraryStatus = { configured: boolean; provider: string | null; root: string | null; records: number; };
type SourceInfo = { id: string; name: string; kind: string; description: string; mediaTypes: string[]; capabilities: { requiresApiKey: boolean; supportsSearch: boolean; supportsCategories: boolean; supportsColor: boolean; supportsNsfw: boolean; }; };
type WallpaperItem = { id: string; source: string; kind: string; title: string; author: string | null; detailUrl: string; imageUrl: string; thumbnailUrl: string; videoUrl: string | null; width: number | null; height: number | null; purity: string | null; };
type SearchResponse = { source: string; page: number; hasMore: boolean; items: WallpaperItem[]; };
type SearchRequest = { source: string; query?: string; page?: number; apiKey?: string; mediaType?: string; nsfwEnabled?: boolean; };
type DownloadResult = { id: string; kind: string; filePath: string; records: number; };
type LibraryWallpaper = { id: string; kind: string; source: string; title: string; author: string | null; detailUrl: string; remoteUrl: string; videoUrl: string | null; filePath: string; thumbnailUrl: string; width: number | null; height: number | null; downloadedAt: string; };
type VideoWallpaperStatus = { active: boolean; paused: boolean; monitorCount: number; currentPath: string | null; };
type ProviderInfo = { id: string; name: string; description: string; suggestedPath: string | null; detected: boolean; detectedPath: string | null; };
type SyncConfig = { enabled: boolean; provider: string | null; providerName: string | null; rootPath: string | null; libraryPath: string | null; mode: string; };
type SyncStatus = { enabled: boolean; provider: string | null; providerName: string | null; libraryPath: string | null; mode: string; manifest: { schemaVersion: number; libraryId: string; appName: string; createdAt: string; updatedAt: string; lastDeviceName: string; provider: string; records: { wallpapers: number; media: number; downloads: number; }; } | null; scanResult: { totalRecords: number; availableCount: number; missingCount: number; records: any[]; } | null; };
type ApiTestResult = { sourceId: string; sourceName: string; ok: boolean; latencyMs: number; error: string | null; };
type SchedulerConfig = { enabled: boolean; intervalMinutes: number; includeStatic: boolean; includeVideo: boolean; changeOnStartup: boolean; };
type QueueItem = { id: string; title: string; kind: string; progress: number; status: string; error: string | null; };

const isTauri = "__TAURI_INTERNALS__" in window;
const keyStoragePrefix = "swallpaper.windows.apiKey.";

async function call<T>(command: string, args?: Record<string, unknown>): Promise<CommandResult<T>> {
  if (!isTauri) return { ok: false, error: "Run inside Tauri." };
  try { const data = await invoke<T>(command, args); return { ok: true, data }; }
  catch (error) { return { ok: false, error: error instanceof Error ? error.message : String(error) }; }
}

function readSavedKey(source: string) { return localStorage.getItem(`${keyStoragePrefix}${source}`) ?? ""; }
function saveKey(source: string, value: string) { const k = `${keyStoragePrefix}${source}`; if (value.trim()) localStorage.setItem(k, value.trim()); else localStorage.removeItem(k); }

// =========== App ===========
function App() {
  type Tab = "home" | "wallpaper" | "media" | "library";

  const [tab, setTab] = React.useState<Tab>("home");
  const [showSettings, setShowSettings] = React.useState(false);
  const [status, setStatus] = React.useState("");
  const [library, setLibrary] = React.useState<LibraryStatus | null>(null);
  const [sources, setSources] = React.useState<SourceInfo[]>([]);
  const [activeSource, setActiveSource] = React.useState("wallhaven");
  const [query, setQuery] = React.useState("anime landscape");
  const [apiKey, setApiKey] = React.useState("");
  const [items, setItems] = React.useState<WallpaperItem[]>([]);
  const [libraryItems, setLibraryItems] = React.useState<LibraryWallpaper[]>([]);
  const [page, setPage] = React.useState(1);
  const [hasMore, setHasMore] = React.useState(false);
  const [isSearching, setIsSearching] = React.useState(false);
  const [videoPath, setVideoPath] = React.useState("");
  const [videoStatus, setVideoStatus] = React.useState<VideoWallpaperStatus | null>(null);
  const [providers, setProviders] = React.useState<ProviderInfo[]>([]);
  const [syncConfig, setSyncConfig] = React.useState<SyncConfig | null>(null);
  const [syncStatus, setSyncStatus] = React.useState<SyncStatus | null>(null);
  const [syncMsg, setSyncMsg] = React.useState("");
  const [nsfwEnabled, setNsfwEnabled] = React.useState(() => localStorage.getItem("swallpaper.nsfw") === "true");
  const [apiResults, setApiResults] = React.useState<ApiTestResult[] | null>(null);
  const [isTesting, setIsTesting] = React.useState(false);
  const [favoriteIds, setFavoriteIds] = React.useState<Set<string>>(new Set());
  const [schedulerConfig, setSchedulerConfig] = React.useState<SchedulerConfig>({ enabled: false, intervalMinutes: 30, includeStatic: true, includeVideo: true, changeOnStartup: false });
  const [queueItems, setQueueItems] = React.useState<QueueItem[]>([]);
  const [rotateInterval, setRotateInterval] = React.useState<ReturnType<typeof setInterval> | null>(null);
  const [lang, setLangState] = React.useState<Lang>(getLang());

  const activeSourceInfo = sources.find((s) => s.id === activeSource);
  const activeSourceNeedsKey = activeSourceInfo?.capabilities.requiresApiKey ?? false;
  const wallpaperSources = sources.filter((s) => s.mediaTypes.includes("static"));
  const videoSources = sources.filter((s) => s.mediaTypes.includes("video"));

  // ---- Data loading ----
  async function refreshLibrary() {
    const [sRes, iRes] = await Promise.all([call<LibraryStatus>("library_status"), call<LibraryWallpaper[]>("list_library_wallpapers")]);
    if (sRes.ok && sRes.data) { setLibrary(sRes.data); setLibraryItems(iRes.ok && iRes.data ? iRes.data : []); }
  }
  async function loadSources() {
    const r = await call<SourceInfo[]>("list_wallpaper_sources");
    if (r.ok && r.data?.length) setSources(r.data);
  }
  async function searchWallpapers(nextPage = 1, append = false) {
    const req: SearchRequest = { source: activeSource, query, page: nextPage, apiKey: apiKey.trim() || undefined, nsfwEnabled };
    setIsSearching(true);
    const r = await call<SearchResponse>("search_wallpapers", { request: req });
    setIsSearching(false);
    if (r.ok && r.data) { setItems((c) => append ? [...c, ...r.data!.items] : r.data!.items); setPage(r.data.page); setHasMore(r.data.hasMore); setStatus(`${r.data.items.length} items`); }
    else { setStatus(r.error ?? "Search failed"); if (!append) { setItems([]); setHasMore(false); } }
  }
  async function setStaticWallpaper(path: string) { const r = await call<string>("set_static_wallpaper", { path }); setStatus(r.data ?? r.error ?? "Done"); }
  async function downloadWallpaper(item: WallpaperItem, applyAfterDownload = false) {
    const isVideo = item.kind === "videoWallpaper";
    const r = await call<DownloadResult>("download_wallpaper", { item });
    if (r.ok && r.data) {
      setLibrary((c) => ({ configured: true, provider: c?.provider ?? "local", root: c?.root ?? null, records: r.data!.records }));
      await refreshLibrary();
      if (applyAfterDownload) { if (isVideo) await startVideoWallpaper(r.data.filePath); else await setStaticWallpaper(r.data.filePath); }
      else setStatus(`Downloaded: ${r.data.filePath}`);
    } else setStatus(r.error ?? "Download failed");
  }
  async function applyOnlineWallpaper(item: WallpaperItem) { await downloadWallpaper(item, true); }
  async function applyLibraryItem(record: LibraryWallpaper) { if (record.kind === "videoWallpaper") await startVideoWallpaper(record.filePath); else await setStaticWallpaper(record.filePath); }
  async function startVideoWallpaper(path: string) {
    if (!path.trim()) { setStatus("Enter a video path."); return; }
    const r = await call<string>("start_video_wallpaper", { path });
    setStatus(r.data ?? r.error ?? "Started"); await refreshVideoStatus();
  }
  async function stopVideoWallpaper() { const r = await call<string>("stop_video_wallpaper"); setStatus(r.data ?? "Stopped"); setVideoStatus(null); }
  async function pauseVideoWallpaper() { await call<string>("pause_video_wallpaper"); await refreshVideoStatus(); }
  async function resumeVideoWallpaper() { await call<string>("resume_video_wallpaper"); await refreshVideoStatus(); }
  async function refreshVideoStatus() { const r = await call<VideoWallpaperStatus>("video_wallpaper_status"); if (r.ok && r.data) { setVideoStatus(r.data); if (r.data.currentPath) setVideoPath(r.data.currentPath); } }

  // ---- Sync ----
  async function loadSyncInfo() {
    const [p, c, s] = await Promise.all([call<ProviderInfo[]>("list_sync_providers"), call<SyncConfig>("get_sync_config"), call<SyncStatus>("get_sync_status")]);
    if (p.ok && p.data) setProviders(p.data); if (c.ok && c.data) setSyncConfig(c.data); if (s.ok && s.data) setSyncStatus(s.data);
  }
  async function enableSync(provider: string, providerName: string, rootPath: string, mode: string) { await call<string>("enable_cloud_sync", { provider, providerName, rootPath, mode }); await loadSyncInfo(); }
  async function disableSync() { await call<string>("disable_cloud_sync"); await loadSyncInfo(); }
  async function scanLibrary() { await call<any>("scan_sync_library"); await loadSyncInfo(); }
  async function importToSync() { await call<string>("import_local_to_sync"); await loadSyncInfo(); }

  // ---- APIs / Favorites / Scheduler ----
  async function testApis() {
    setIsTesting(true); setApiResults(null);
    const r = await call<ApiTestResult[]>("test_api_connectivity"); setIsTesting(false);
    if (r.ok && r.data) { setApiResults(r.data); setStatus(`${r.data.filter((x) => x.ok).length}/${r.data.length} reachable`); }
  }
  async function toggleFavoriteItem(id: string) { await call<string>("toggle_favorite", { id }); setFavoriteIds((p) => { const n = new Set(p); if (n.has(id)) n.delete(id); else n.add(id); return n; }); }
  function toggleNsfw() { setNsfwEnabled((p: boolean) => { const n = !p; localStorage.setItem("swallpaper.nsfw", String(n)); if (n && !window.confirm(t("nsfwConfirm"))) return p; return n; }); }
  async function loadSchedulerConfig() { const r = await call<SchedulerConfig>("get_scheduler_config"); if (r.ok && r.data) setSchedulerConfig(r.data); }
  async function saveSchedulerConfig(cfg: SchedulerConfig) { await call<string>("save_scheduler_config", { config: cfg }); }
  function startRotate() { const c = { ...schedulerConfig, enabled: true }; setSchedulerConfig(c); saveSchedulerConfig(c); setRotateInterval(setInterval(rotateNow, c.intervalMinutes * 60 * 1000)); }
  function stopRotate() { const c = { ...schedulerConfig, enabled: false }; setSchedulerConfig(c); saveSchedulerConfig(c); if (rotateInterval) { clearInterval(rotateInterval); setRotateInterval(null); } }
  async function rotateNow() { const r = await call<any>("rotate_wallpaper"); if (r.ok) setStatus(`Rotated: ${r.data?.title}`); }
  async function refreshQueue() { const r = await call<QueueItem[]>("get_download_queue"); if (r.ok && r.data) setQueueItems(r.data); }

  React.useEffect(() => { loadSources(); refreshLibrary(); refreshVideoStatus(); loadSyncInfo(); loadSchedulerConfig(); }, []);
  React.useEffect(() => { setApiKey(readSavedKey(activeSource)); setItems([]); setPage(1); setHasMore(false); }, [activeSource]);
  React.useEffect(() => { saveKey(activeSource, apiKey); }, [activeSource, apiKey]);

  // =========== RENDER ===========
  const TABS: { id: Tab; label: string }[] = [
    { id: "home", label: "Home" },
    { id: "wallpaper", label: "Wallpapers" },
    { id: "media", label: "Media" },
    { id: "library", label: "Library" },
  ];

  return (
    <main className="shell">
      {/* ---- Top Navigation Bar (Mac v2 style) ---- */}
      <header className="topbar">
        <div className="topbar-logo">
          <img src="/icon.png" alt="" />
          <span>Swallpaper</span>
        </div>
        <div className="topbar-spacer" />
        <nav className="topbar-segmented">
          {TABS.map((t) => (
            <button key={t.id} className={`topbar-tab ${tab === t.id ? "active" : ""}`} onClick={() => setTab(t.id)}>
              {t.label}
            </button>
          ))}
        </nav>
        <div className="topbar-spacer" />
        <button className="topbar-settings" onClick={() => setShowSettings(!showSettings)} title="Settings">
          <Settings size={18} />
        </button>
      </header>

      {/* ---- Settings Overlay ---- */}
      {showSettings && (
        <div className="settings-overlay" onClick={(e) => { if (e.target === e.currentTarget) setShowSettings(false); }}>
          <div className="settings-sheet">
            <div className="settings-sheet-header">
              <h2>Settings</h2>
              <button onClick={() => setShowSettings(false)}>✕</button>
            </div>
            <div className="settings-sheet-body">
              <div className="settings-section">
                <h3><RotateCw size={16} /> Auto-Rotate</h3>
                <div className="settings-row"><span>Interval (min)</span>
                  <input type="number" min={1} max={1440} value={schedulerConfig.intervalMinutes}
                    onChange={(e) => setSchedulerConfig((c) => ({ ...c, intervalMinutes: Math.max(1, parseInt(e.target.value) || 30) }))}
                    style={{ width: 70, padding: "4px 8px", borderRadius: 8, border: "1px solid rgba(255,255,255,0.12)", background: "rgba(255,255,255,0.04)", color: "#fff" }} /></div>
                <div className="settings-row"><span>Static</span><input type="checkbox" checked={schedulerConfig.includeStatic} onChange={(e) => setSchedulerConfig((c) => ({ ...c, includeStatic: e.target.checked }))} /></div>
                <div className="settings-row"><span>Video</span><input type="checkbox" checked={schedulerConfig.includeVideo} onChange={(e) => setSchedulerConfig((c) => ({ ...c, includeVideo: e.target.checked }))} /></div>
                <div className="engine-actions" style={{ marginTop: 8 }}>
                  {schedulerConfig.enabled ? <button onClick={stopRotate}><Square size={16} /> Stop</button> : <button onClick={startRotate}><Play size={16} /> Start</button>}
                  <button onClick={rotateNow}><RotateCw size={16} /> Rotate Now</button>
                </div>
              </div>
              <div className="settings-section">
                <h3>Content</h3>
                <div className="settings-row"><span>Adult content</span>
                  <button className={`nsfw-toggle ${nsfwEnabled ? "active" : ""}`} onClick={toggleNsfw}>{nsfwEnabled ? "ON" : "OFF"}</button></div>
              </div>
              <div className="settings-section">
                <h3>Language</h3>
                <div className="settings-row" style={{ gap: 8 }}>
                  {getAvailableLangs().map((l) => (
                    <button key={l.code} onClick={() => { setLang(l.code); setLangState(l.code); }}
                      style={{ padding: "4px 12px", borderRadius: 8, border: lang === l.code ? "1px solid var(--accent-pink)" : "1px solid rgba(255,255,255,0.12)", background: lang === l.code ? "rgba(255,51,102,0.15)" : "transparent", color: lang === l.code ? "var(--accent-pink)" : "var(--text-secondary)", cursor: "pointer", fontSize: 13 }}>{l.name}</button>
                  ))}
                </div>
              </div>
              <div className="settings-section">
                <h3>About</h3>
                <dl className="meta-list">
                  <div><dt>Version</dt><dd>v0.1.3</dd></div>
                  <div><dt>Build</dt><dd>Tauri 2 + React + Rust</dd></div>
                </dl>
                <div className="engine-actions" style={{ marginTop: 8 }}>
                  <button onClick={() => window.open("https://github.com/sfyqiu/Swallpaper-Windows", "_blank")}><ExternalLink size={16} /> Repository</button>
                  <button onClick={() => window.open("https://github.com/sfyqiu/Swallpaper-Windows/issues", "_blank")}><Info size={16} /> Report Issue</button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* ---- Content Area ---- */}
      <section className="content">
        {/* HOME TAB */}
        {tab === "home" && (
          <div className="home-content">
            {/* Hero banner (Mac v2 style) */}
            <div className="home-hero">
              <div className="home-hero-content">
                <h1>Swallpaper</h1>
                <p>Static wallpapers · Dynamic wallpapers · Multi-source aggregation — {library?.records ?? 0} items in your library</p>
              </div>
            </div>

            {/* Quick source access */}
            <div className="home-shelf">
              <h3>Wallpaper Sources</h3>
              <div className="source-strip">
                {wallpaperSources.map((s) => (
                  <button key={s.id} className={`source-pill ${s.id === activeSource ? "active" : ""}`}
                    onClick={() => { setActiveSource(s.id); setTab("wallpaper"); }}>
                    <span>{s.name}</span><small>{s.capabilities.requiresApiKey ? "API key" : "Free"}</small>
                  </button>
                ))}
              </div>
            </div>

            <div className="home-shelf">
              <h3>Video Sources</h3>
              <div className="source-strip">
                {videoSources.map((s) => (
                  <button key={s.id} className={`source-pill ${s.id === activeSource ? "active" : ""}`}
                    onClick={() => { setActiveSource(s.id); setTab("media"); }}>
                    <span>{s.name}</span><small>{s.capabilities.requiresApiKey ? "API key" : "Free"}</small>
                  </button>
                ))}
              </div>
            </div>

            {/* Recent library items */}
            {libraryItems.length > 0 && (
              <div className="home-shelf">
                <h3>Recent Library</h3>
                <div className="wallpaper-grid">
                  {libraryItems.slice(0, 8).map((item) => (
                    <article key={item.id} className="wallpaper-card" onClick={() => applyLibraryItem(item)}>
                      <div className="card-thumb-wrap">
                        <img src={item.thumbnailUrl || convertFileSrc(item.filePath)} alt={item.title} loading="lazy" />
                        {item.kind === "videoWallpaper" && <span className="media-badge">Video</span>}
                      </div>
                      <div className="wallpaper-card-body"><h3>{item.title}</h3><p>{item.source}</p></div>
                    </article>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}

        {/* WALLPAPER TAB */}
        {tab === "wallpaper" && (
          <div className="explore-content">
            <div className="source-strip">
              {wallpaperSources.map((s) => (
                <button key={s.id} className={`source-pill ${s.id === activeSource ? "active" : ""}`} onClick={() => setActiveSource(s.id)}>
                  <span>{s.name}</span><small>{s.capabilities.requiresApiKey ? "API key" : "Free"}</small>
                </button>
              ))}
            </div>

            <div className="search-panel">
              <label className="field">
                <span>Search</span>
                <div className="input-row"><Search size={16} /><input value={query} onChange={(e) => setQuery(e.target.value)} placeholder="anime, landscape, mountain..." /></div>
              </label>
              <label className="field">
                <span>API Key</span>
                <div className="input-row"><KeyRound size={16} /><input value={apiKey} onChange={(e) => setApiKey(e.target.value)} placeholder={activeSourceNeedsKey ? "Required" : "Optional"} type="password" /></div>
              </label>
              <div className="search-actions">
                <button className="primary-action" disabled={isSearching} onClick={() => searchWallpapers(1, false)}><RefreshCw size={16} /> Search</button>
                <button className="secondary-action" disabled={isTesting} onClick={testApis}><Wifi size={14} /> Test APIs</button>
                {activeSourceInfo?.capabilities.supportsNsfw && (
                  <button className={`nsfw-toggle ${nsfwEnabled ? "active" : ""}`} onClick={toggleNsfw}>{nsfwEnabled ? "NSFW" : "SFW"}</button>
                )}
              </div>
            </div>

            {apiResults && (
              <div className="api-results">
                {apiResults.map((r) => (
                  <div key={r.sourceId} className="api-result-item">
                    <span className={`dot ${r.ok ? "ok" : "fail"}`} /><span className="name">{r.sourceName}</span>
                    <span className="latency">{r.latencyMs}ms</span>
                    {r.error && <span className="error-msg">{r.error}</span>}
                  </div>
                ))}
              </div>
            )}

            {items.length === 0 ? (
              <div className="empty-state"><Image size={40} /><strong>No wallpapers</strong><span>Select a source and search.</span></div>
            ) : (
              <>
                <div className="wallpaper-grid">
                  {items.map((item) => (
                    <article key={item.id} className="wallpaper-card">
                      <div className="card-thumb-wrap">
                        <img src={item.thumbnailUrl} alt={item.title} loading="lazy" />
                        {item.kind === "videoWallpaper" && <span className="media-badge">Video</span>}
                      </div>
                      <div className="wallpaper-card-body">
                        <h3>{item.title}</h3>
                        <p>{item.author ?? item.source}{item.width && item.height ? ` · ${item.width}x${item.height}` : ""}</p>
                        <div className="card-actions">
                          <button onClick={() => applyOnlineWallpaper(item)} title="Apply"><Image size={16} /></button>
                          <a href={item.detailUrl} target="_blank" rel="noreferrer"><ExternalLink size={16} /></a>
                          <button onClick={() => downloadWallpaper(item)} title="Download"><Download size={16} /></button>
                          <button onClick={() => toggleFavoriteItem(item.id)} style={favoriteIds.has(item.id) ? { color: "var(--accent-amber)" } : {}}><Star size={16} /></button>
                        </div>
                      </div>
                    </article>
                  ))}
                </div>
                {hasMore && <button className="load-more" disabled={isSearching} onClick={() => searchWallpapers(page + 1, true)}>Load more</button>}
              </>
            )}
          </div>
        )}

        {/* MEDIA TAB */}
        {tab === "media" && (
          <div className="explore-content">
            <div className="source-strip">
              {videoSources.map((s) => (
                <button key={s.id} className={`source-pill ${s.id === activeSource ? "active" : ""}`} onClick={() => setActiveSource(s.id)}>
                  <span>{s.name}</span><small>{s.capabilities.requiresApiKey ? "API key" : "Free"}</small>
                </button>
              ))}
            </div>

            <div className="search-panel">
              <label className="field">
                <span>Search</span>
                <div className="input-row"><Search size={16} /><input value={query} onChange={(e) => setQuery(e.target.value)} placeholder="nature, city, abstract..." /></div>
              </label>
              <label className="field">
                <span>API Key</span>
                <div className="input-row"><KeyRound size={16} /><input value={apiKey} onChange={(e) => setApiKey(e.target.value)} placeholder={activeSourceNeedsKey ? "Required" : "Optional"} type="password" /></div>
              </label>
              <div className="search-actions">
                <button className="primary-action" disabled={isSearching} onClick={() => searchWallpapers(1, false)}><RefreshCw size={16} /> Search</button>
              </div>
            </div>

            {/* Video wallpaper controls */}
            <div className="panel compact-panel" style={{ marginBottom: 14 }}>
              <div className="panel-title"><Video size={18} /><h2>Video Wallpaper</h2></div>
              {videoStatus?.active ? (
                <div>
                  <dl className="meta-list">
                    <div><dt>State</dt><dd>{videoStatus.paused ? "Paused" : "Playing"} · {videoStatus.monitorCount} monitor(s)</dd></div>
                    <div><dt>File</dt><dd className="truncate">{videoStatus.currentPath ?? "—"}</dd></div>
                  </dl>
                  <div className="engine-actions">
                    {videoStatus.paused ? <button onClick={resumeVideoWallpaper}><Play size={16} /> Resume</button> : <button onClick={pauseVideoWallpaper}><Pause size={16} /> Pause</button>}
                    <button onClick={stopVideoWallpaper}><Square size={16} /> Stop</button>
                  </div>
                </div>
              ) : (
                <div className="video-start">
                  <label className="field"><span>Video file path</span>
                    <div className="input-row"><Video size={16} /><input value={videoPath} onChange={(e) => setVideoPath(e.target.value)} placeholder="C:\Users\...\wallpaper.mp4" /></div></label>
                  <button className="primary-action" style={{ marginTop: 8 }} onClick={() => startVideoWallpaper(videoPath)}><Play size={16} /> Start</button>
                </div>
              )}
            </div>

            {items.length === 0 ? (
              <div className="empty-state"><Video size={40} /><strong>No videos</strong><span>Select a video source and search.</span></div>
            ) : (
              <>
                <div className="wallpaper-grid">
                  {items.map((item) => (
                    <article key={item.id} className="wallpaper-card">
                      <div className="card-thumb-wrap">
                        <img src={item.thumbnailUrl} alt={item.title} loading="lazy" />
                        <span className="media-badge">Video</span>
                      </div>
                      <div className="wallpaper-card-body">
                        <h3>{item.title}</h3>
                        <p>{item.author ?? item.source}</p>
                        <div className="card-actions">
                          <button onClick={() => applyOnlineWallpaper(item)} title="Download & apply"><Download size={16} /></button>
                          <a href={item.detailUrl} target="_blank" rel="noreferrer"><ExternalLink size={16} /></a>
                          <button onClick={() => toggleFavoriteItem(item.id)} style={favoriteIds.has(item.id) ? { color: "var(--accent-amber)" } : {}}><Star size={16} /></button>
                        </div>
                      </div>
                    </article>
                  ))}
                </div>
                {hasMore && <button className="load-more" disabled={isSearching} onClick={() => searchWallpapers(page + 1, true)}>Load more</button>}
              </>
            )}
          </div>
        )}

        {/* LIBRARY TAB */}
        {tab === "library" && (
          <div className="explore-content">
            <div className="panel" style={{ marginBottom: 14 }}>
              <div className="panel-title"><FolderOpen size={18} /><h2>Library</h2></div>
              <dl className="meta-list">
                <div><dt>Records</dt><dd>{library?.records ?? 0}</dd></div>
                <div><dt>Root</dt><dd className="truncate">{library?.root ?? "—"}</dd></div>
              </dl>
            </div>

            {/* Cloud Sync */}
            <div className="panel compact-panel" style={{ marginBottom: 14 }}>
              <div className="panel-title"><Cloud size={18} /><h2>Cloud Sync</h2></div>
              {syncConfig?.enabled ? (
                <div>
                  <dl className="meta-list">
                    <div><dt>Provider</dt><dd>{syncConfig.providerName}</dd></div>
                    <div><dt>Mode</dt><dd>{syncConfig.mode}</dd></div>
                  </dl>
                  <div className="engine-actions">
                    <button onClick={scanLibrary}><RefreshCw size={16} /> Scan</button>
                    {syncConfig.mode === "manual" && <button onClick={importToSync}><Download size={16} /> Import</button>}
                    <button onClick={disableSync} style={{ borderColor: "rgba(255,51,102,0.4)", background: "rgba(255,51,102,0.1)" }}><Square size={16} /> Disable</button>
                  </div>
                </div>
              ) : (
                <div>
                  <div className="provider-list" style={{ marginBottom: 10 }}>
                    {providers.map((p) => (
                      <button key={p.id} className={`provider-item ${p.detected ? "detected" : ""}`} onClick={() => { const path = p.detectedPath ?? p.suggestedPath ?? ""; if (path) enableSync(p.id, p.name, path, "manual"); }}>
                        <div><strong>{p.name}</strong><small>{p.detected ? p.detectedPath : "Not detected"}</small></div>
                        <span className={p.detected ? "provider-badge" : "provider-badge muted"}>{p.detected ? "Found" : "Select"}</span>
                      </button>
                    ))}
                  </div>
                </div>
              )}
            </div>

            {libraryItems.length === 0 ? (
              <div className="empty-state"><FolderOpen size={40} /><strong>Empty Library</strong><span>Download wallpapers to see them here.</span></div>
            ) : (
              <div className="wallpaper-grid">
                {libraryItems.map((item) => (
                  <article key={item.id} className="wallpaper-card" onClick={() => applyLibraryItem(item)}>
                    <div className="card-thumb-wrap">
                      <img src={item.thumbnailUrl || convertFileSrc(item.filePath)} alt={item.title} loading="lazy" />
                      {item.kind === "videoWallpaper" && <span className="media-badge">Video</span>}
                    </div>
                    <div className="wallpaper-card-body">
                      <h3>{item.title}</h3>
                      <p>{item.source}</p>
                      <div className="card-actions">
                        <button onClick={(e) => { e.stopPropagation(); applyLibraryItem(item); }}>{item.kind === "videoWallpaper" ? <Video size={16} /> : <Image size={16} />}</button>
                        <button onClick={(e) => { e.stopPropagation(); toggleFavoriteItem(item.id); }} style={favoriteIds.has(item.id) ? { color: "var(--accent-amber)" } : {}}><Star size={16} /></button>
                      </div>
                    </div>
                  </article>
                ))}
              </div>
            )}
          </div>
        )}
      </section>
    </main>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(<React.StrictMode><App /></React.StrictMode>);
