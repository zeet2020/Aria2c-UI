# Aria2 UI (AUI)

A **personal fork of [Motrix](https://github.com/agalwood/Motrix)**, ported from
**Electron to [Tauri](https://tauri.app)** for a much smaller footprint — the
same full-featured aria2 download manager UI, built on the OS-native webview +
a Rust core instead of bundling a full Chromium runtime.

> Aria2 UI is a graphical front-end for the [aria2](https://aria2.github.io/)
> download engine (bundled), supporting HTTP, FTP, BitTorrent and Magnet.

## Why this fork?

| | Motrix (upstream) | Aria2 UI (this fork) |
|---|---|---|
| Shell | Electron (bundled Chromium) | Tauri (OS webview + Rust) |
| Install size | ~150 MB+ | a few MB |
| Idle memory | higher | lower |
| Engine | aria2 sidecar | aria2 sidecar (unchanged) |
| Frontend | Vue 2 + Element-UI | **Vue 3 + Element Plus** (upgraded) |

The download UI and aria2 engine behaviour are kept intact. The desktop shell
was re-implemented (Electron → Tauri) and the frontend was upgraded from Vue 2
(EOL) + Element-UI to **Vue 3 + Element Plus** for an actively-maintained stack.
The auto-updater was removed (use your package manager / release downloads).

## Original project

This would not exist without **Motrix** by **Dr_rOot**:

- Source: <https://github.com/agalwood/Motrix>
- Website: <https://motrix.app>

All credit for the application design, UI and aria2 integration goes to the
original Motrix authors. This fork only swaps the runtime.

## Develop

Prerequisites: **Node.js ≥ 18**, **Rust (stable)**, and on Linux the Tauri
system deps.

```bash
# Linux system deps (Debian/Ubuntu)
sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev

# Install JS deps (skips the legacy Electron postinstall)
npm install --ignore-scripts --legacy-peer-deps

# Run in dev (Vite + Tauri)
npm run tauri:dev
```

## Build

```bash
npm run tauri:build
```

Before building, stage the aria2 sidecar for your target triple (CI does this
automatically — see `.github/workflows/build.yml`):

```bash
mkdir -p src-tauri/binaries
# example: Linux x64
cp extra/linux/x64/engine/aria2c \
   src-tauri/binaries/aria2c-x86_64-unknown-linux-gnu
cp extra/linux/x64/engine/aria2.conf src-tauri/resources/aria2.conf
```

### CI / Releases

`.github/workflows/build.yml` builds for **Linux** (`.deb`, `.AppImage`,
`.rpm`), **Windows** (`.msi`, NSIS `.exe`) and **macOS** (Intel + Apple
Silicon `.dmg`). Push a `v*` tag to cut a draft GitHub release with all
bundles attached, or run the workflow manually to get artifacts.

## Project layout

```
src/renderer/   Vue 3 frontend (talks to aria2 directly over JSON-RPC)
src/shared/     shared utils, aria2 client, i18n locales
src-tauri/      Rust core (engine sidecar, config, tray, protocol, autostart)
extra/          bundled aria2c binaries per OS/arch
```

## License

[MIT](./LICENSE) — same as the original Motrix.

Copyright © Dr_rOot (original Motrix author). You are free to **clone, modify,
redistribute and use** this project as needed under the terms of the MIT
license.
