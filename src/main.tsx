import React from "react";
import ReactDOM from "react-dom/client";
import {
  BatteryCharging,
  Cloud,
  Download,
  ExternalLink,
  FolderOpen,
  Image,
  KeyRound,
  Monitor,
  Play,
  RefreshCw,
  Search,
  Square,
  Video
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
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
  title: string;
  author: string | null;
  detailUrl: string;
  imageUrl: string;
  thumbnailUrl: string;
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
  const [page, setPage] = React.useState(1);
  const [hasMore, setHasMore] = React.useState(false);
  const [isSearching, setIsSearching] = React.useState(false);

  const activeSourceInfo = sources.find((source) => source.id === activeSource);
  const activeSourceNeedsKey = activeSourceInfo?.capabilities.requiresApiKey ?? false;

  async function refreshLibrary() {
    const result = await call<LibraryStatus>("library_status");
    if (result.ok && result.data) {
      setLibrary(result.data);
      setStatus("Library status refreshed");
    } else {
      setStatus(result.error ?? "Unable to read library status");
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

  async function startVideoHost() {
    const result = await call<string>("start_video_wallpaper", {
      path: "C:\\\\Users\\\\Public\\\\Videos\\\\sample.mp4"
    });
    setStatus(result.data ?? result.error ?? "Video wallpaper command completed");
  }

  async function stopVideoHost() {
    const result = await call<string>("stop_video_wallpaper");
    setStatus(result.data ?? result.error ?? "Stop command completed");
  }

  React.useEffect(() => {
    void loadSources();
    void refreshLibrary();
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
    ["Static wallpaper", "Win32 SPI_SETDESKWALLPAPER bridge", "Ready"],
    ["Video host", "WorkerW/Progman desktop layer", "Next"]
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
            <Cloud size={18} /> Cloud Sync
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
                    <img src={item.thumbnailUrl} alt={item.title} loading="lazy" />
                    <div className="wallpaper-card-body">
                      <div>
                        <h3>{item.title}</h3>
                        <p>
                          {item.author ?? item.source}
                          {item.width && item.height ? ` · ${item.width}x${item.height}` : ""}
                        </p>
                      </div>
                      <div className="card-actions">
                        <button onClick={() => setStaticWallpaper(item.imageUrl)} title="Set as static wallpaper">
                          <Image size={17} />
                        </button>
                        <a href={item.detailUrl} target="_blank" rel="noreferrer" title="Open source page">
                          <ExternalLink size={17} />
                        </a>
                        <a href={item.imageUrl} target="_blank" rel="noreferrer" title="Open full image">
                          <Download size={17} />
                        </a>
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
                <h2>Cloud Library</h2>
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
            </div>

            <div className="panel compact-panel" id="video">
              <div className="panel-title">
                <BatteryCharging size={19} />
                <h2>Native Roadmap</h2>
              </div>
              <div className="timeline">
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
              <div className="engine-actions">
                <button onClick={startVideoHost}>
                  <Play size={17} /> Start video placeholder
                </button>
                <button onClick={stopVideoHost}>
                  <Square size={17} /> Stop
                </button>
              </div>
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
