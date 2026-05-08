# Media File Renamer

<p align="center">
  <img src="./Media File Renamer Logo.png" alt="Media File Renamer icon" width="128" height="128">
</p>

<p align="center">
  <strong>Batch-rename photos and videos using EXIF or file dates.</strong><br>
  Desktop app built with <strong>Tauri v2</strong> (Rust + TypeScript) for <strong>Linux</strong> and <strong>Windows</strong>.
</p>

<p align="center">
  <a href="https://github.com/fly2nbc-oss/media-file-renamer/blob/master/LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue.svg" alt="License Apache-2.0"></a>
  <a href="https://github.com/fly2nbc-oss/media-file-renamer/releases/latest"><img src="https://img.shields.io/github/v/release/fly2nbc-oss/media-file-renamer?sort=semver" alt="Latest release"></a>
  <img src="https://img.shields.io/badge/platform-linux%20%7C%20windows-lightgrey.svg" alt="Platforms Linux and Windows">
  <a href="https://github.com/fly2nbc-oss/media-file-renamer/actions/workflows/build.yml"><img src="https://img.shields.io/github/actions/workflow/status/fly2nbc-oss/media-file-renamer/build.yml?branch=master" alt="Build status"></a>
</p>

---

## Table of contents

- [Screenshots](#screenshots)
- [Features](#features)
- [Quick start / Installation](#quick-start--installation)
- [Usage](#usage)
- [Supported file types](#supported-file-types)
- [Development & Build](#development--build)
- [Project structure](#project-structure)
- [Tech stack](#tech-stack)
- [Versioning](#versioning)
- [Roadmap, issues & contributing](#roadmap-issues--contributing)
- [License](#license)

---

## Screenshots

Add high-quality screenshots (light and dark UI, Linux and Windows) under [`./screenshots/`](./screenshots/) and link them here. Example:

```markdown
![Main window](./screenshots/main-linux-dark.png)
```

Until those assets exist, use the built app locally (`npm run tauri dev`) to capture images.

---

## Features

- **Batch rename** by date: add files or folders via **Add** or **Drag & Drop**; choose a naming format; preview and rename in one go.
- **Three naming formats:**
  - `YYYY_MM_DD__hhmmss` (e.g. `2024_03_15__143052`) — default
  - `YYMMDD_hhmmss` (e.g. `240315_143052`)
  - `YYMMDD_originalname` (e.g. `240315_IMG_1234`)
- **Date from EXIF** for images (JPEG, TIFF, HEIF, etc.) and from **video metadata** (MP4/MOV). Fallback to file modification time.
- **Time offset:** correct wrong camera time (e.g. timezone) with a seconds field or expanded Years / Months / Days / Hours / Minutes / Seconds. Offset is applied to filenames, file timestamps, and EXIF (when `exiftool` is available).
- **Live preview** of the new names before renaming; names that actually change are highlighted in blue.
- **Optional backup:** create a `backup_YYYYMMDD_HHMMSS` folder and copy originals before renaming.
- **HEIC → JPG:** optional conversion (90% quality) via built-in `libheif-rs`; EXIF is preserved; original HEIC is removed after success.
- **Progress** overlay and **error log** panel after a run.
- **Undo** the last rename (one level only). For HEIC→JPG conversions, full HEIC restoration requires **Backup**; without backup, undo restores the converted JPG path only.
- **Light/Dark** UI follows system theme.

---

## Quick start / Installation

### Releases (recommended)

**Stable binaries:** see [**Releases**](https://github.com/fly2nbc-oss/media-file-renamer/releases/latest). Pushing a tag `v*` on `master` triggers CI and uploads Windows (EXE, MSI, NSIS) and Linux (standalone, `.deb`, `.AppImage`) assets to that GitHub Release.

Details: [`docs/RELEASE.md`](./docs/RELEASE.md).

### CI artifacts (latest successful build)

1. Open [**Actions**](https://github.com/fly2nbc-oss/media-file-renamer/actions).
2. Pick the latest successful workflow run.
3. Download the artifact you need:

| Artifact | Content | Use on |
|----------|---------|--------|
| **media-file-renamer-windows-exe** | Standalone `.exe` (no installer) | Windows (requires [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) if missing) |
| **media-file-renamer-windows-msi** | MSI installer | Windows |
| **media-file-renamer-windows-nsis** | NSIS installer (single `.exe` setup) | Windows |
| **media-file-renamer-linux-standalone** | Standalone binary | Linux |
| **media-file-renamer-linux-deb** | `.deb` package | Debian / Ubuntu / compatible |
| **media-file-renamer-linux-appimage** | `.AppImage` | Most Linux distros |

Unzip the artifact and run the executable or installer. Files use filesystem-friendly names (e.g. `media-file-renamer_1.0.5_amd64.deb`, `media-file-renamer_1.0.5_amd64.AppImage`, `media-file-renamer.exe`).

### Build from source (Linux)

**Prerequisites**

- [Node.js](https://nodejs.org/) (v22 LTS recommended) and npm
- [Rust](https://rustup.rs/) (stable)
- System libraries (examples for Arch/Manjaro and Debian/Ubuntu):

**Arch / Manjaro:**

```bash
sudo pacman -S webkit2gtk-4.1 gtk3 libappindicator-gtk3 librsvg patchelf
```

**Debian / Ubuntu:**

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev libappindicator3-dev librsvg2-dev patchelf
```

**HEIC support dependency (build/link):**

- **libheif ≥ 1.17 development package** is required to build from source (`libheif-dev` on Debian/Ubuntu, `libheif` on Arch/Manjaro; Windows via vcpkg). Runtime availability depends on your bundle/target environment.
  - **Arch / Manjaro:** `sudo pacman -S libheif`
  - **Debian / Ubuntu:** `sudo apt install libheif-dev`
  - **Windows:** install via [vcpkg](https://vcpkg.io/) — `vcpkg install libheif` (or use `cargo vcpkg build`)

**Optional (runtime):**

- **EXIF writing** (when using time offset): `perl-image-exiftool` (e.g. `sudo pacman -S perl-image-exiftool` or `sudo apt install libimage-exiftool-perl`)

**Build**

```bash
git clone https://github.com/fly2nbc-oss/media-file-renamer.git
cd media-file-renamer
npm install
npm run tauri:build
```

Outputs (under `src-tauri/target/release/` and `.../bundle/`):

- Binary: `media-file-renamer`
- **.deb:** `bundle/deb/media-file-renamer_1.0.5_amd64.deb`
- **.rpm:** `bundle/rpm/media-file-renamer-1.0.5-1.x86_64.rpm`
- **.AppImage:** `bundle/appimage/media-file-renamer_1.0.5_amd64.AppImage`

Bundle filenames use the product name `media-file-renamer` (no spaces). The build uses `NO_STRIP=1` so AppImage succeeds on modern distros.

### Build from source (Windows)

Install [Rust](https://rustup.rs/), [Node.js](https://nodejs.org/), and [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/). WebView2 is usually already present on Windows 11. Then:

```bash
npm install
npm run tauri -- build
```

Artifacts are under `src-tauri\target\release\` and `...\bundle\` (e.g. `media-file-renamer_1.0.5_x64_en-US.msi`, `media-file-renamer.exe`).

---

## Usage

1. **Start** the app.
2. **Add files:** click **Add** (file picker) or drag files/folders onto the window. Folders are scanned recursively.
3. **Format:** choose a naming format from the dropdown.
4. **Offset (optional):** enter seconds (e.g. `-7200` for −2 hours) or expand and set Years / Months / Days / Hours / Minutes / Seconds. The list preview updates automatically.
5. **Options:** enable **Backup** and/or **HEIC→JPG** if needed.
6. **Rename:** click **Rename N Files**. Confirm if asked (e.g. for large batches). Use **Undo Last** if you need to revert the last run.

**Date source badges in the table:**

- **EXIF** — date from image/video metadata.
- **File** — date from file system (no EXIF).
- **None** — no date; file is skipped on rename.

---

## Supported file types

| Images | Videos |
|--------|--------|
| JPG, JPEG, PNG, HEIC, HEIF, TIFF, TIF, WEBP, GIF, BMP | MP4, MOV, AVI, MKV |

---

## Development & Build

```bash
npm install
npm run tauri dev
```

Runs the app with hot-reload (Vite on port 1420). Edit `src/*` and `src-tauri/src/*` as needed.

**App icons:** the canonical raster logo is [`Media File Renamer Logo.png`](./Media File Renamer Logo.png) (square PNG). Regenerate platform icons with:

```bash
npx tauri icon "Media File Renamer Logo.png"
```

---

## Project structure

```
media-file-renamer/
├── src/                    # Frontend (Vanilla TypeScript)
│   ├── main.ts             # App init, events, UI, Tauri invoke
│   ├── types.ts            # Shared TypeScript types
│   └── styles.css          # Layout, theme (light/dark)
├── public/                 # Static assets (favicon for Vite)
├── screenshots/            # Screenshots for README (optional)
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── lib.rs          # Tauri app & plugins
│   │   ├── commands.rs     # Tauri commands (scan, preview, rename, undo)
│   │   ├── models.rs       # Data structures
│   │   ├── exif_handler.rs # EXIF read; video date; exiftool write
│   │   ├── heic_converter.rs # HEIC → JPG via libheif-rs
│   │   ├── renamer.rs      # Name formats, offset, duplicate handling
│   │   ├── backup.rs       # Backup folder creation
│   │   └── undo.rs         # Undo log save/restore
│   ├── Cargo.toml
│   └── tauri.conf.json
├── index.html
├── package.json
├── CHANGELOG.md
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
└── README.md
```

---

## Tech stack

- **App:** Tauri v2 (Rust + webview)
- **Frontend:** Vanilla TypeScript, Vite, CSS (no framework)
- **Backend:** Rust — `kamadak-exif`, `chrono`, `filetime`, `walkdir`, `libheif-rs`, `image`; optional system tool: `exiftool`

---

## Versioning

This project follows Semantic Versioning (`MAJOR.MINOR.PATCH`).

- `MAJOR`: breaking changes
- `MINOR`: new backward-compatible features
- `PATCH`: backward-compatible fixes/improvements

Automation commands:

- `npm run version:major`
- `npm run version:minor`
- `npm run version:patch`
- `npm run version:auto`

For the full policy and release mapping, see [`docs/versioning.md`](./docs/versioning.md).

---

## Roadmap, issues & contributing

- **Issues & ideas:** use [GitHub Issues](https://github.com/fly2nbc-oss/media-file-renamer/issues).
- **Contributing:** see [`CONTRIBUTING.md`](./CONTRIBUTING.md) and [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md).
- **Suggested GitHub repository topics:** `tauri`, `rust`, `typescript`, `desktop`, `exif`, `batch-rename`, `photos`, `video`, `media`, `linux`, `windows`.

---

## License

Licensed under the Apache License, Version 2.0. See [`LICENSE`](./LICENSE).
