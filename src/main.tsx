import React from "react";
import ReactDOM from "react-dom/client";
import {
  BatteryCharging,
  Cloud,
  FolderOpen,
  Image,
  Monitor,
  Play,
  RefreshCw,
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

const isTauri = "__TAURI_INTERNALS__" in window;

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

function App() {
  const [status, setStatus] = React.useState<string>("Prototype ready");
  const [library, setLibrary] = React.useState<LibraryStatus | null>(null);

  async function refreshLibrary() {
    const result = await call<LibraryStatus>("library_status");
    if (result.ok && result.data) {
      setLibrary(result.data);
      setStatus("Library status refreshed");
    } else {
      setStatus(result.error ?? "Unable to read library status");
    }
  }

  async function setStaticWallpaper() {
    const result = await call<string>("set_static_wallpaper", {
      path: "C:\\\\Users\\\\Public\\\\Pictures\\\\Sample Pictures\\\\wallpaper.jpg"
    });
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
    void refreshLibrary();
  }, []);

  const milestones = [
    ["Static wallpaper", "Win32 SPI_SETDESKWALLPAPER bridge", "Ready for Windows build"],
    ["Video host", "WorkerW/Progman desktop layer", "Command boundary staged"],
    ["Cloud library", "Shared Swallpaper Library metadata", "Spec documented"],
    ["Sources", "Wallhaven, Pexels, NASA, Coverr", "After native prototype"]
  ];

  return (
    <main className="shell">
      <aside className="sidebar">
        <div className="brand-mark">S</div>
        <nav aria-label="Prototype sections">
          <a className="nav-item active" href="#dashboard">
            <Monitor size={18} /> Dashboard
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

      <section className="content" id="dashboard">
        <header className="hero">
          <div>
            <p className="eyebrow">Swallpaper Windows Prototype</p>
            <h1>桌面壁纸引擎的 Windows 基座</h1>
            <p className="lede">
              保留 macOS 版本的产品方向，重新用 Tauri、Rust 和 Win32 API 构建 Windows 客户端。
            </p>
          </div>
          <div className="status-panel" aria-live="polite">
            <span className="status-dot" />
            <strong>{status}</strong>
            <small>{isTauri ? "Native bridge available" : "Browser preview mode"}</small>
          </div>
        </header>

        <section className="command-grid" aria-label="Prototype commands">
          <button className="command-card" onClick={setStaticWallpaper}>
            <Image size={24} />
            <span>Set Static Wallpaper</span>
            <small>Calls Rust native command</small>
          </button>
          <button className="command-card" onClick={startVideoHost}>
            <Play size={24} />
            <span>Start Video Host</span>
            <small>WorkerW host placeholder</small>
          </button>
          <button className="command-card" onClick={stopVideoHost}>
            <Square size={24} />
            <span>Stop Video Host</span>
            <small>Stops native host session</small>
          </button>
          <button className="command-card" onClick={refreshLibrary}>
            <RefreshCw size={24} />
            <span>Refresh Library</span>
            <small>Reads shared metadata status</small>
          </button>
        </section>

        <section className="workspace">
          <div className="panel">
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
                <dt>Root</dt>
                <dd>{library?.root ?? "Choose a synced folder later"}</dd>
              </div>
              <div>
                <dt>Records</dt>
                <dd>{library?.records ?? 0}</dd>
              </div>
            </dl>
          </div>

          <div className="panel">
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
