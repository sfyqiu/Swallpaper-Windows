# Windows Porting Plan

## Goal

Create a Windows version of Swallpaper without breaking the macOS app.

The current macOS app is built with SwiftUI, AppKit, macOS wallpaper APIs,
NSWindow behavior, and macOS-specific file permissions. It cannot be compiled
directly as a Windows app.

## Strategy

Use a new Windows client:

- Tauri 2
- Rust
- React
- TypeScript
- Win32 APIs for desktop wallpaper integration

The macOS repository remains the product reference and upstream source:

https://github.com/sfyqiu/Swallpaper-Mac-v2

## Windows Modules

- StaticWallpaperManager: set image wallpaper with Windows APIs.
- VideoWallpaperHost: attach a playback window behind desktop icons.
- DisplayManager: enumerate monitors and respond to display changes.
- PowerModeManager: pause video wallpaper on battery or low-power state.
- FullscreenDetector: pause video wallpaper when games/fullscreen apps are active.
- DownloadManager: download static and video wallpapers.
- LibraryManager: read and write the shared Swallpaper Library format.
- CloudLibrarySync: use local cloud-sync folders, not cloud provider APIs.

## Implementation Stages

1. Create Tauri prototype.
2. Set local static image as wallpaper.
3. Host local video as desktop wallpaper.
4. Add shared library metadata.
5. Add local cloud folder sync.
6. Add online data sources.
7. Add GitHub Actions Windows build.

## Non-Goals For First Prototype

- No Steam Workshop integration.
- No full Wallpaper Engine compatibility.
- No cloud account login.
- No OneDrive/Dropbox/Google Drive API integration.
- No rewrite of the macOS SwiftUI app.
