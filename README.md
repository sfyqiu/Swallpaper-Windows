# Swallpaper Windows

<p align="center">
  <samp>
    <b>Windows 壁纸引擎</b><br>
    <b>静态壁纸 · 动态壁纸 · 视频壁纸</b><br>
    <b>多源聚合，全场景覆盖</b>
  </samp>
</p>

<p align="center">
  <samp>
    基于 <a href="https://github.com/sfyqiu/Swallpaper-Mac"><b>Swallpaper Mac</b></a> 的 Windows 移植版本<br>
    使用 Tauri 2 + React + Rust 构建
  </samp>
</p>

<p align="center">
  <a href="https://github.com/sfyqiu/Swallpaper-Windows/releases">
    <img src="https://img.shields.io/github/v/release/sfyqiu/Swallpaper-Windows?color=6366f1&style=flat-square" alt="Release">
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/License-GPL--3.0-06b6d4?style=flat-square" alt="License">
  </a>
  <a href="https://github.com/sfyqiu/Swallpaper-Windows/stargazers">
    <img src="https://img.shields.io/github/stars/sfyqiu/Swallpaper-Windows?color=f59e0b&style=flat-square" alt="Stars">
  </a>
  <a href="https://github.com/sfyqiu/Swallpaper-Windows/releases">
    <img src="https://img.shields.io/github/downloads/sfyqiu/Swallpaper-Windows/total?color=8b5cf6&style=flat-square" alt="Downloads">
  </a>
</p>

---

## Features

| Feature | Status | Description |
|---------|:------:|-------------|
| 🖼 **Multi-source wallpapers** | ✅ | 6 sources: Wallhaven / 4K Wallpapers / Unsplash / Pexels / NASA APOD / NASA Images |
| 🎬 **Dynamic wallpaper** | ✅ | Video wallpaper with WorkerW desktop layer injection, multi-monitor |
| 🎥 **Video sources** | ✅ | Coverr / Pexels Videos / MotionBG / WE Workshop (web & video) |
| ☁️ **Cloud sync** | ✅ | OneDrive / iCloud / Dropbox / Google Drive / Nutstore / Baidu / custom |
| ⭐ **Favorites** | ✅ | Save wallpapers and videos to personal collection |
| ⚡️ **One-click apply** | ✅ | Download and set as wallpaper in one click |
| 🖥️ **Multi-monitor** | ✅ | Per-monitor video wallpaper with WorkerW injection |
| 📥 **Local library** | ✅ | Download manager, metadata tracking, offline browsing |
| 🧪 **API test** | ✅ | One-click parallel connectivity test for all sources |
| 🔞 **Content filter** | ✅ | SFW by default, optional adult content with confirmation |
| 🔍 **Search** | ✅ | Keyword search across all sources |
| 🚀 **Auto-release** | ✅ | Git tag → CI build → Release with .exe + .msi |

---

## Wallpaper & Video Sources

### Wallpapers

| Source | API Key | Link |
|--------|:-------:|------|
| [Wallhaven](https://wallhaven.cc) | Optional | [Get key](https://wallhaven.cc/settings/account) |
| [4K Wallpapers](https://4kwallpapers.com) | None | — |
| [Unsplash](https://unsplash.com) | Required | [Get key](https://unsplash.com/developers) |
| [Pexels](https://www.pexels.com) | Required | [Get key](https://www.pexels.com/api/) |
| [NASA APOD](https://apod.nasa.gov) | Optional | [Get key](https://api.nasa.gov/) |
| [NASA Images](https://images.nasa.gov) | None | — |

### Videos

| Source | API Key | Link |
|--------|:-------:|------|
| [Coverr](https://coverr.co) | None | — |
| [Pexels Videos](https://www.pexels.com) | Required | [Get key](https://www.pexels.com/api/) |
| [MotionBG](https://motionbgs.com) | None | — |
| [WE Workshop](https://store.steampowered.com/app/431960/) | None | Steam + WE required |

> 💡 Enter API keys in the source panel. Use **Test APIs** to verify connectivity.

---

## Cloud Sync

Sync your library to cloud drives for seamless sharing across devices.

| Provider | Auto-detect |
|----------|:-----------:|
| OneDrive | ✅ |
| iCloud Drive | ✅ |
| Dropbox | ✅ |
| Google Drive | ✅ |
| Nutstore | ✅ |
| Baidu Netdisk | ✅ |
| Custom folder | ✅ |

- **Auto sync** — Downloads go directly to cloud directory
- **Manual sync** — Download locally, migrate on demand
- **Batch migration** — One-click import existing library

> 💡 Configure in the **Cloud Sync** panel.

---

## Install

👉 **[Latest Release](https://github.com/sfyqiu/Swallpaper-Windows/releases/latest)**

- `Swallpaper.Windows_{version}_x64-setup.exe` — NSIS installer
- `Swallpaper.Windows_{version}_x64_en-US.msi` — MSI installer

> ⚠️ Windows SmartScreen may warn on first run. Click "More info" → "Run anyway".

---

## System Requirements

- **Windows 10+** (21H2 or later)
- **x64** architecture
- **WebView2 Runtime** (pre-installed on Windows 10+)

---

## Content Filter

Swallpaper Windows is **SFW by default**. All sources filter explicit content automatically.

An adult content toggle is available in the search panel for sources that support it (Wallhaven). Enabling requires explicit confirmation.

---

## Development

```bash
npm install
npm run tauri:build     # Windows build
```

- Push to `main` → CI builds and uploads artifacts
- Push tag `v*` → CI builds, creates Release, uploads .exe + .msi

---

## License

[GNU General Public License v3.0 (GPL-3.0)](LICENSE)

---

## Disclaimer

Swallpaper Windows does **not store or host any content**. It aggregates from third-party public APIs. All copyright belongs to original sites and authors.

Wallpaper Engine Workshop scanning reads local Steam directories for interoperability only. Users must legally own WE and relevant content. This is **not an official WE product**.

**AS IS**, no liability. For personal use only.

---

<p align="center">
  <samp>
    Made with 💜 by <a href="https://github.com/sfyqiu">@sfyqiu</a>
  </samp>
</p>
