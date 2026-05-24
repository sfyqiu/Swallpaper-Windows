import React from "react";
import ReactDOM from "react-dom/client";
import {
  Cloud,
  Download,
  ExternalLink,
  FolderOpen,
  Image,
  KeyRound,
  Monitor,
  Pause,
  Play,
  RefreshCw,
  Search,
  Square,
  Video
} from "lucide-react";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import "./styles.css";

type CommandResult<T> = {
  ok: boolean;
  data?: T;
  error?: string;
};

type LibraryStatus = {
  configured: boolean;
  provider: string | null;
  root: string | null;
  records: number;
};

type SourceInfo = {
  id: string;
  name: string;
  kind: string;
  description: string;
  capabilities: {
    requiresApiKey: boolean;
    supportsSearch: boolean;
    supportsCategories: boolean;
    supportsColor: boolean;
    supportsNsfw: boolean;
  };
};

type WallpaperItem = {
  id: string;
  source: string;
  kind: string;
  title: string;
  author: string | null;
  detailUrl: string;
  imageUrl: string;
  thumbnailUrl: string;
  videoUrl: string | null;
  width: number | null;
  height: number | null;
  purity: string | null;
};

type SearchResponse = {
  source: string;
  page: number;
  hasMore: boolean;
  items: WallpaperItem[];
};

type SearchRequest = {
  source: string;
  query?: string;
  page?: number;
  apiKey?: string;
};

type DownloadResult = {
  id: string;
  kind: string;
  filePath: string;
  records: number;
};

type LibraryWallpaper = {
  id: string;
  kind: string;
  source: string;
  title: string;
  author: string | null;
  detailUrl: string;
  remoteUrl: string;
  videoUrl: string | null;
  filePath: string;
  thumbnailUrl: string;
  width: number | null;
  height: number | null;
  downloadedAt: string;
};

type VideoWallpaperStatus = {
  active: boolean;
  paused: boolean;
  monitorCount: number;
  currentPath: string | null;
};

type ProviderInfo = {
  id: string;
  name: string;
  description: string;
  suggestedPath: string | null;
  detected: boolean;
  detectedPath: string | null;
};

type SyncConfig = {
  enabled: boolean;
  provider: string | null;
  providerName: string | null;
  rootPath: string | null;
  libraryPath: string | null;
  mode: string;
};

type SyncStatus = {
  enabled: boolean;
  provider: string | null;
  providerName: string | null;
  libraryPath: string | null;
  mode: string;
  manifest: {
    schemaVersion: number;
    libraryId: string;
    appName: string;
    createdAt: string;
    updatedAt: string;
    lastDeviceName: string;
    provider: string;
    records: { wallpapers: number; media: number; downloads: number };
  } | null;
  scanResult: {
    totalRecords: number;
    availableCount: number;
    missingCount: number;
    records: any[];
  } | null;
};

const isTauri = "__TAURI_INTERNALS__" in window;
const keyStoragePrefix = "swallpaper.windows.apiKey.";

async function call<T>(command: string, args?: Record<string, unknown>): Promise<CommandResult<T>> {
  if (!isTauri) {
    return {
      ok: false,
      error: "Run inside Tauri to use native wallpaper commands."
    };
  }

  try {
    const data = await invoke<T>(command, args);
    return { ok: true, data };
  } catch (error) {
    return { ok: false, error: error instanceof Error ? error.message : String(error) };
  }
}

function readSavedKey(source: string) {
  return localStorage.getItem(`${keyStoragePrefix}${source}`) ?? "";
}

function saveKey(source: string, value: string) {
  const key = `${keyStoragePrefix}${source}`;
  if (value.trim()) {
    localStorage.setItem(key, value.trim());
  } else {
    localStorage.removeItem(key);
  }
}

function App() {
  const [status, setStatus] = React.useState("Prototype ready");
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

  const activeSourceInfo = sources.find((source) => source.id === activeSource);
  const activeSourceNeedsKey = activeSourceInfo?.capabilities.requiresApiKey ?? false;

  async function refreshLibrary() {
    const [statusResult, itemsResult] = await Promise.all([
      call<LibraryStatus>("library_status"),
      call<LibraryWallpaper[]>("list_library_wallpapers")
    ]);

    if (statusResult.ok && statusResult.data) {
      setLibrary(statusResult.data);
      setLibraryItems(itemsResult.ok && itemsResult.data ? itemsResult.data : []);
      setStatus("Library refreshed");
    } else {
      setStatus(statusResult.error ?? "Unable to read library status");
    }
  }

  async function loadSources() {
    const result = await call<SourceInfo[]>("list_wallpaper_sources");
    if (result.ok && result.data?.length) {
      setSources(result.data);
      setStatus("Wallpaper sources loaded");
    } else {
      const fallback: SourceInfo[] = [
        {
          id: "wallhaven",
          name: "Wallhaven",
          kind: "static",
          description: "Primary static wallpaper source.",
          capabilities: {
            requiresApiKey: false,
            supportsSearch: true,
            supportsCategories: true,
            supportsColor: true,
            supportsNsfw: true
          }
        }
      ];
      setSources(fallback);
      setStatus(result.error ?? "Using browser preview source list");
    }
  }

  async function searchWallpapers(nextPage = 1, append = false) {
    const request: SearchRequest = {
      source: activeSource,
      query,
      page: nextPage,
      apiKey: apiKey.trim() || undefined
    };
    setIsSearching(true);
    setStatus(`Searching ${activeSourceInfo?.name ?? activeSource}...`);
    const result = await call<SearchResponse>("search_wallpapers", { request });
    setIsSearching(false);

    if (result.ok && result.data) {
      setItems((current) => (append ? [...current, ...result.data!.items] : result.data!.items));
      setPage(result.data.page);
      setHasMore(result.data.hasMore);
      setStatus(`${result.data.items.length} wallpapers loaded from ${activeSourceInfo?.name ?? activeSource}`);
    } else {
      setStatus(result.error ?? "Search failed");
      if (!append) {
        setItems([]);
        setHasMore(false);
      }
    }
  }

  async function setStaticWallpaper(path: string) {
    const result = await call<string>("set_static_wallpaper", { path });
    setStatus(result.data ?? result.error ?? "Static wallpaper command completed");
  }

  async function downloadWallpaper(item: WallpaperItem, applyAfterDownload = false) {
    const isVideo = item.kind === "videoWallpaper";
    setStatus(applyAfterDownload ? `Preparing ${item.title}...` : `Downloading ${item.title}...`);
    const result = await call<DownloadResult>("download_wallpaper", { item });
    if (result.ok && result.data) {
      setLibrary((current) => ({
        configured: true,
        provider: current?.provider ?? "local",
        root: current?.root ?? null,
        records: result.data!.records
      }));
      await refreshLibrary();
      if (applyAfterDownload) {
        if (isVideo) {
          await startVideoWallpaper(result.data.filePath);
        } else {
          await setStaticWallpaper(result.data.filePath);
        }
      } else {
        setStatus(`Downloaded to ${result.data.filePath}`);
      }
    } else {
      setStatus(result.error ?? "Download failed");
    }
  }

  async function applyOnlineWallpaper(item: WallpaperItem) {
    await downloadWallpaper(item, true);
  }

  async function applyLibraryItem(record: LibraryWallpaper) {
    if (record.kind === "videoWallpaper") {
      await startVideoWallpaper(record.filePath);
    } else {
      await setStaticWallpaper(record.filePath);
    }
  }

  async function startVideoWallpaper(path: string) {
    if (!path.trim()) {
      setStatus("Enter a video file path to start.");
      return;
    }
    setStatus(`Starting video wallpaper: ${path}...`);
    const result = await call<string>("start_video_wallpaper", { path });
    if (result.ok) {
      setStatus(result.data ?? "Video wallpaper started");
      await refreshVideoStatus();
    } else {
      setStatus(result.error ?? "Failed to start video wallpaper");
    }
  }

  async function stopVideoWallpaper() {
    setStatus("Stopping video wallpaper...");
    const result = await call<string>("stop_video_wallpaper");
    if (result.ok) {
      setStatus(result.data ?? "Video wallpaper stopped");
      setVideoStatus(null);
    } else {
      setStatus(result.error ?? "Failed to stop video wallpaper");
    }
  }

  async function pauseVideoWallpaper() {
    const result = await call<string>("pause_video_wallpaper");
    if (result.ok) {
      setStatus("Video wallpaper paused");
      await refreshVideoStatus();
    } else {
      setStatus(result.error ?? "Failed to pause");
    }
  }

  async function resumeVideoWallpaper() {
    const result = await call<string>("resume_video_wallpaper");
    if (result.ok) {
      setStatus("Video wallpaper resumed");
      await refreshVideoStatus();
    } else {
      setStatus(result.error ?? "Failed to resume");
    }
  }

  async function refreshVideoStatus() {
    const result = await call<VideoWallpaperStatus>("video_wallpaper_status");
    if (result.ok && result.data) {
      setVideoStatus(result.data);
      if (result.data.currentPath) {
        setVideoPath(result.data.currentPath);
      }
    }
  }

  async function loadSyncInfo() {
    const [provResult, cfgResult, statusResult] = await Promise.all([
      call<ProviderInfo[]>("list_sync_providers"),
      call<SyncConfig>("get_sync_config"),
      call<SyncStatus>("get_sync_status"),
    ]);
    if (provResult.ok && provResult.data) setProviders(provResult.data);
    if (cfgResult.ok && cfgResult.data) setSyncConfig(cfgResult.data);
    if (statusResult.ok && statusResult.data) setSyncStatus(statusResult.data);
  }

  async function enableSync(provider: string, providerName: string, rootPath: string, mode: string) {
    setSyncMsg("Enabling...");
    const result = await call<string>("enable_cloud_sync", { provider, providerName, rootPath, mode });
    if (result.ok) {
      setSyncMsg(result.data ?? "Enabled");
      await loadSyncInfo();
    } else {
      setSyncMsg(result.error ?? "Failed to enable sync");
    }
  }

  async function disableSync() {
    setSyncMsg("Disabling...");
    const result = await call<string>("disable_cloud_sync");
    if (result.ok) {
      setSyncMsg(result.data ?? "Disabled");
      await loadSyncInfo();
    } else {
      setSyncMsg(result.error ?? "Failed to disable sync");
    }
  }

  async function scanLibrary() {
    setSyncMsg("Scanning...");
    const result = await call<any>("scan_sync_library");
    if (result.ok) {
      setSyncMsg(`Scan complete: ${result.data?.totalRecords ?? 0} records`);
      await loadSyncInfo();
    } else {
      setSyncMsg(result.error ?? "Scan failed");
    }
  }

  async function importToSync() {
    setSyncMsg("Importing...");
    const result = await call<string>("import_local_to_sync");
    setSyncMsg(result.data ?? result.error ?? "Import completed");
    await loadSyncInfo();
  }

  React.useEffect(() => {
    void loadSources();
    void refreshLibrary();
    void refreshVideoStatus();
    void loadSyncInfo();
  }, []);

  React.useEffect(() => {
    setApiKey(readSavedKey(activeSource));
    setItems([]);
    setPage(1);
    setHasMore(false);
  }, [activeSource]);

  React.useEffect(() => {
    saveKey(activeSource, apiKey);
  }, [activeSource, apiKey]);

  const milestones = [
    ["Static source APIs", "Wallhaven, Pexels, Unsplash, NASA APOD", "Implemented"],
    ["Source switching", "Unified source registry and key-aware UI", "Implemented"],
    ["Static wallpaper", "Download-first Win32 wallpaper bridge", "Ready"],
    ["Local library", "Browse downloaded wallpapers and re-apply", "Implemented"],
    ["Video wallpaper", "WorkerW desktop layer + video engine", "Implemented"]
  ];

  return (
    <main className="shell">
      <aside className="sidebar">
        <div className="brand-mark">S</div>
        <nav aria-label="Prototype sections">
          <a className="nav-item active" href="#sources">
            <Search size={18} /> Sources
          </a>
          <a className="nav-item" href="#library">
            <FolderOpen size={18} /> Library
          </a>
          <a className="nav-item" href="#wallpaper">
            <Image size={18} /> Wallpaper
          </a>
          <a className="nav-item" href="#video">
            <Video size={18} /> Video Host
          </a>
          <a className="nav-item" href="#sync">
            <Cloud size={18} /> Sync
          </a>
        </nav>
      </aside>

      <section className="content" id="sources">
        <header className="hero">
          <div>
            <p className="eyebrow">Swallpaper Windows Prototype</p>
            <h1>先把壁纸源跑起来</h1>
            <p className="lede">
              Windows 版现在开始迁移 Mac 端的资源层：统一源切换、API Key 管理、在线搜索与静态壁纸设置。
            </p>
          </div>
          <div className="status-panel" aria-live="polite">
            <span className="status-dot" />
            <strong>{status}</strong>
            <small>{isTauri ? "Native bridge available" : "Browser preview mode"}</small>
          </div>
        </header>

        <section className="source-strip" aria-label="Wallpaper sources">
          {sources.map((source) => (
            <button
              className={`source-pill ${source.id === activeSource ? "active" : ""}`}
              key={source.id}
              onClick={() => setActiveSource(source.id)}
            >
              <span>{source.name}</span>
              <small>{source.capabilities.requiresApiKey ? "API key" : "Ready"}</small>
            </button>
          ))}
        </section>

        <section className="search-panel">
          <label className="field">
            <span>Search</span>
            <div className="input-row">
              <Search size={18} />
              <input
                value={query}
                disabled={!activeSourceInfo?.capabilities.supportsSearch}
                onChange={(event) => setQuery(event.target.value)}
                placeholder={activeSourceInfo?.capabilities.supportsSearch ? "anime landscape, mountain, car..." : "This source does not search"}
              />
            </div>
          </label>

          <label className="field">
            <span>{activeSourceInfo?.name ?? "Source"} Key</span>
            <div className="input-row">
              <KeyRound size={18} />
              <input
                value={apiKey}
                onChange={(event) => setApiKey(event.target.value)}
                placeholder={activeSourceNeedsKey ? "Required API key" : "Optional"}
                type="password"
              />
            </div>
          </label>

          <button className="primary-action" disabled={isSearching} onClick={() => searchWallpapers(1, false)}>
            <RefreshCw size={18} />
            {isSearching ? "Loading" : "Search"}
          </button>
        </section>

        <section className="workspace">
          <div className="panel results-panel">
            <div className="panel-title">
              <Monitor size={19} />
              <h2>{activeSourceInfo?.name ?? "Wallpaper"} Results</h2>
            </div>

            {items.length === 0 ? (
              <div className="empty-state">
                <Image size={42} />
                <strong>No wallpapers loaded</strong>
                <span>选择一个源并搜索。Pexels 和 Unsplash 需要先填自己的 API Key。</span>
              </div>
            ) : (
              <div className="wallpaper-grid">
                {items.map((item) => (
                  <article className="wallpaper-card" key={item.id}>
                    <div className="card-thumb-wrap">
                      <img src={item.thumbnailUrl} alt={item.title} loading="lazy" />
                      {item.kind === "videoWallpaper" && (
                        <span className="media-badge video-badge">
                          <Video size={13} />
                        </span>
                      )}
                    </div>
                    <div className="wallpaper-card-body">
                      <div>
                        <h3>{item.title}</h3>
                        <p>
                          {item.author ?? item.source}
                          {item.width && item.height ? ` · ${item.width}x${item.height}` : ""}
                          {item.kind === "videoWallpaper" ? " · Video" : ""}
                        </p>
                      </div>
                      <div className="card-actions">
                        <button onClick={() => applyOnlineWallpaper(item)} title="Download and set as static wallpaper">
                          <Image size={17} />
                        </button>
                        <a href={item.detailUrl} target="_blank" rel="noreferrer" title="Open source page">
                          <ExternalLink size={17} />
                        </a>
                        <button onClick={() => downloadWallpaper(item)} title="Download to Swallpaper Library">
                          <Download size={17} />
                        </button>
                      </div>
                    </div>
                  </article>
                ))}
              </div>
            )}

            {items.length > 0 && hasMore ? (
              <button className="load-more" disabled={isSearching} onClick={() => searchWallpapers(page + 1, true)}>
                Load more
              </button>
            ) : null}
          </div>

          <div className="side-stack">
            <div className="panel compact-panel" id="library">
              <div className="panel-title">
                <Cloud size={19} />
                <h2>Local Library</h2>
              </div>
              <dl className="meta-list">
                <div>
                  <dt>Configured</dt>
                  <dd>{library?.configured ? "Yes" : "No"}</dd>
                </div>
                <div>
                  <dt>Provider</dt>
                  <dd>{library?.provider ?? "Not selected"}</dd>
                </div>
                <div>
                  <dt>Records</dt>
                  <dd>{library?.records ?? 0}</dd>
                </div>
              </dl>
              <div className="library-list" aria-label="Downloaded wallpapers">
                {libraryItems.length === 0 ? (
                  <p className="library-empty">下载壁纸后会出现在这里。</p>
                ) : (
                  libraryItems.slice(0, 6).map((record) => (
                    <article className="library-item" key={record.id}>
                      <img
                        src={record.thumbnailUrl || convertFileSrc(record.filePath)}
                        alt={record.title}
                        loading="lazy"
                      />
                      <div>
                        <h3>{record.title}</h3>
                        <p>{record.source}</p>
                      </div>
                      <button onClick={() => applyLibraryItem(record)} title={record.kind === "videoWallpaper" ? "Set as video wallpaper" : "Set as static wallpaper"}>
                        {record.kind === "videoWallpaper" ? <Video size={16} /> : <Image size={16} />}
                      </button>
                    </article>
                  ))
                )}
              </div>
            </div>

            <div className="panel compact-panel" id="video">
              <div className="panel-title">
                <Video size={19} />
                <h2>Video Wallpaper</h2>
              </div>

              {videoStatus?.active ? (
                <div className="video-status">
                  <dl className="meta-list">
                    <div>
                      <dt>State</dt>
                      <dd>{videoStatus.paused ? "Paused" : "Playing"}</dd>
                    </div>
                    <div>
                      <dt>Monitors</dt>
                      <dd>{videoStatus.monitorCount}</dd>
                    </div>
                    <div>
                      <dt>File</dt>
                      <dd className="truncate">{videoStatus.currentPath ?? "—"}</dd>
                    </div>
                  </dl>
                  <div className="engine-actions">
                    {videoStatus.paused ? (
                      <button onClick={resumeVideoWallpaper}>
                        <Play size={17} /> Resume
                      </button>
                    ) : (
                      <button onClick={pauseVideoWallpaper}>
                        <Pause size={17} /> Pause
                      </button>
                    )}
                    <button onClick={stopVideoWallpaper}>
                      <Square size={17} /> Stop
                    </button>
                  </div>
                </div>
              ) : (
                <div className="video-start">
                  <label className="field">
                    <span>Video file path</span>
                    <div className="input-row">
                      <Video size={18} />
                      <input
                        value={videoPath}
                        onChange={(e) => setVideoPath(e.target.value)}
                        placeholder="C:\Users\Public\Videos\wallpaper.mp4"
                      />
                    </div>
                  </label>
                  <div className="engine-actions">
                    <button onClick={() => startVideoWallpaper(videoPath)}>
                      <Play size={17} /> Start video wallpaper
                    </button>
                  </div>
                </div>
              )}

              <div className="timeline" style={{ marginTop: 18 }}>
                {milestones.map(([title, detail, state]) => (
                  <article className="timeline-item" key={title}>
                    <span />
                    <div>
                      <h3>{title}</h3>
                      <p>{detail}</p>
                      <small>{state}</small>
                    </div>
                  </article>
                ))}
              </div>
            </div>

            <div className="panel compact-panel" id="sync">
              <div className="panel-title">
                <Cloud size={19} />
                <h2>Cloud Sync</h2>
              </div>

              {syncConfig?.enabled ? (
                <div className="sync-status">
                  <dl className="meta-list">
                    <div>
                      <dt>Provider</dt>
                      <dd>{syncConfig.providerName ?? syncConfig.provider}</dd>
                    </div>
                    <div>
                      <dt>Mode</dt>
                      <dd>{syncConfig.mode === "auto" ? "Auto" : "Manual"}</dd>
                    </div>
                    <div>
                      <dt>Library</dt>
                      <dd className="truncate">{syncConfig.libraryPath ?? "—"}</dd>
                    </div>
                    {syncStatus?.manifest && (
                      <div>
                        <dt>Records</dt>
                        <dd>
                          {syncStatus.manifest.records.wallpapers} WP / {syncStatus.manifest.records.media} Video
                        </dd>
                      </div>
                    )}
                  </dl>
                  <div className="engine-actions">
                    <button onClick={scanLibrary}>
                      <RefreshCw size={17} /> Scan
                    </button>
                    {syncConfig.mode === "manual" && (
                      <button onClick={importToSync}>
                        <Download size={17} /> Import
                      </button>
                    )}
                    <button onClick={disableSync} style={{ borderColor: "rgba(255,107,107,0.4)", background: "rgba(255,107,107,0.1)" }}>
                      <Square size={17} /> Disable
                    </button>
                  </div>
                </div>
              ) : (
                <div className="sync-setup">
                  <p className="sync-hint">选择一个云盘目录，壁纸库会自动同步到云端。</p>
                  <div className="provider-list">
                    {providers.map((p) => (
                      <button
                        key={p.id}
                        className={`provider-item ${p.detected ? "detected" : ""}`}
                        onClick={() => {
                          const path = p.detectedPath ?? p.suggestedPath ?? "";
                          if (path) enableSync(p.id, p.name, path, "manual");
                        }}
                      >
                        <div>
                          <strong>{p.name}</strong>
                          <small>{p.detected ? p.detectedPath : "Not detected"}</small>
                        </div>
                        {p.detected ? (
                          <span className="provider-badge">Found</span>
                        ) : (
                          <span className="provider-badge muted">Select</span>
                        )}
                      </button>
                    ))}
                  </div>
                  <label className="field" style={{ marginTop: 10 }}>
                    <span>Custom folder path</span>
                    <div className="input-row">
                      <input
                        placeholder="C:\Users\xxx\OneDrive"
                        onKeyDown={(e) => {
                          if (e.key === "Enter") {
                            const val = (e.target as HTMLInputElement).value.trim();
                            if (val) enableSync("custom", "Custom Folder", val, "manual");
                          }
                        }}
                      />
                    </div>
                  </label>
                </div>
              )}

              {syncMsg && (
                <p style={{ marginTop: 10, color: "var(--muted)", fontSize: 13 }}>
                  {syncMsg}
                </p>
              )}
            </div>
          </div>
        </section>
      </section>
    </main>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
