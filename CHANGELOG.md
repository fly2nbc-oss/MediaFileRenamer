# Changelog

All notable changes to this project are documented in this file. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versioning follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Fixed

- Stop the Windows startup access violation caused by libheif registering libde265 during C++ static initialization (ASLR-dependent crash at `fill_scan_pos` before the window is created). Plugins now initialize on first HEIC use; libde265 scan-table setup is guarded under MSVC.

[Unreleased]: https://github.com/fly2nbc-oss/MediaFileRenamer/compare/v1.0.7...HEAD

## [1.0.7] - 2026-08-04

### Fixed

- Apply timestamp offsets even when the selected filename pattern keeps the original name.
- Prevent filename-collision handling from ever selecting an existing destination.
- Derive backup filenames from the source path and validate undo paths before filesystem changes.

### Security

- Limit directory scans to 10,000 supported files and do not follow directory symlinks.
- Restrict the desktop webview with an explicit Content Security Policy.

### Changed

- Add automated tests for scanning, collision handling, timestamp offsets, backups, and undo validation.

[1.0.7]: https://github.com/fly2nbc-oss/MediaFileRenamer/compare/v1.0.6...v1.0.7

## [1.0.6] - 2026-05-08

### Changed

- Tauri bundle identifier set to `com.fly2nbc.media-file-renamer` (aligned with publisher namespace).

[1.0.6]: https://github.com/fly2nbc-oss/MediaFileRenamer/compare/v1.0.5...v1.0.6

## [1.0.5] - 2026-05-08

### Changed

- App icon and bundled platform icons regenerated from the updated square logo (`Media File Renamer Logo.png`).
- README reorganized for GitHub project presentation (badges, quick links to releases, community files).

### Added

- `CHANGELOG.md`, `CONTRIBUTING.md`, and `CODE_OF_CONDUCT.md` at repository root.
- Web favicon for the Vite frontend (`public/favicon.png`).

[1.0.5]: https://github.com/fly2nbc-oss/MediaFileRenamer/compare/v1.0.4...v1.0.5
