# Swallpaper Windows

Windows port of Swallpaper.

This repository starts from the product direction and shared library schema of
`sfyqiu/Swallpaper-Mac-v2`, but the Windows client will be implemented as a new
app instead of attempting to compile SwiftUI/AppKit code on Windows.

## Source Reference

- macOS source: https://github.com/sfyqiu/Swallpaper-Mac-v2
- Windows target stack: Tauri + Rust + React + TypeScript

## First Milestone

1. Build a Windows prototype that can set a static wallpaper.
2. Add a desktop video wallpaper host using Win32 WorkerW/Progman.
3. Implement the shared Swallpaper Library format.
4. Add cloud-folder sync through local synced folders.
5. Add Wallhaven, Pexels, NASA APOD, and video sources after the local prototype works.

See:

- `docs/windows-porting-plan.md`
- `docs/cross-platform-library-spec.md`
- `windows-prototype/README.md`
