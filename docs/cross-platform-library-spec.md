# Cross-Platform Swallpaper Library Spec

The macOS and Windows clients should be able to share one library folder through
iCloud Drive, OneDrive, Dropbox, Google Drive, Nutstore, Baidu Netdisk, or any
custom synced folder.

## Directory Layout

```text
Swallpaper Library/
  manifest.json
  metadata/
    wallpapers.json
    media.json
    favorites.json
    downloads.json
  files/
    wallpapers/
    videos/
    live/
  thumbnails/
  cache/
  logs/
```

## Rules

- Store relative paths only.
- Do not store machine-specific absolute paths.
- Do not store API keys.
- JSON writes must be atomic.
- Missing files should be marked as missing instead of deleting records.

## Manifest

```json
{
  "schemaVersion": 1,
  "libraryID": "uuid",
  "appName": "Swallpaper",
  "createdAt": "2026-05-24T00:00:00Z",
  "updatedAt": "2026-05-24T00:00:00Z",
  "lastDeviceName": "Windows PC",
  "provider": "oneDrive",
  "records": {
    "wallpapers": 0,
    "media": 0,
    "favorites": 0,
    "downloads": 0
  }
}
```

## Record

```json
{
  "id": "pexels-123",
  "kind": "staticWallpaper",
  "source": "pexels",
  "title": "Mountain Lake",
  "remoteURL": "https://example.com/image.jpg",
  "relativeFilePath": "files/wallpapers/pexels-123.jpg",
  "thumbnailPath": "thumbnails/pexels-123.jpg",
  "createdAt": "2026-05-24T00:00:00Z",
  "updatedAt": "2026-05-24T00:00:00Z",
  "fileSize": 123456,
  "sha256": "",
  "status": "available"
}
```

## Status Values

- available
- missing
- needsDownload
- needsRelink
