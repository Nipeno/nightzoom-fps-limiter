# NightZoom FPS Limiter — Installer (Tauri)

A small Windows GUI installer (`~5 MB .exe`) that finds the user's FiveM install,
downloads the **latest** release of the limiter from GitHub, copies the files into
`FiveM.app\plugins`, and writes the ReShade5 acknowledgement to `CitizenFX.ini` — i.e.
the whole `Enable-ReShade.bat` flow, plus the file copy, behind a branded UI.

**Install + update (download-mode).** Each run fetches the latest release, so re-running the
installer is how users update. It reads the version of the addon already in `plugins` (off the
DLL's VERSIONINFO resource) and the latest release version, then the welcome screen shows
*installed → latest* and the button reads **Install vX** / **Update to vX** / **Reinstall vX**.
The installed addon itself never phones home — only the installer does. (No uninstaller by design.)

## Layout
```
installer/
├── ui/index.html          # frontend (HTML/CSS/JS). Same file previews in a browser.
└── src-tauri/
    ├── src/lib.rs          # backend: detect_fivem, check_latest, install, relaunch_as_admin
    ├── src/main.rs         # entry point
    ├── tauri.conf.json     # window 760x540, frameless, custom titlebar
    ├── capabilities/       # Tauri v2 permissions (window drag/close, dialog, events)
    ├── icons/              # app icon (NZ logo on dark square)
    └── Cargo.toml
```

## How the UI talks to the backend
`ui/index.html` is the **production** UI — no preview scaffolding, no debug screen
switcher. It calls the Rust commands and drives the install screen from `progress`
events. Opening it bare in a browser just shows the first screen (no `window.__TAURI__`).
For design iteration use the standalone mockup (`nightzoom-installer-ui.html`), which keeps
the screen switcher; port visual changes back here afterwards.

Commands (`src-tauri/src/lib.rs`):
| Command | Args | Returns |
|---|---|---|
| `detect_fivem` | `manualPath?` | `{found, path, plugins, existingReshade, installedVersion, computerName, reshadeId}` |
| `check_latest` | `installed?` | `{version, action}` where `action` ∈ `install`/`update`/`reinstall` |
| `install` | `fivemPath, replaceDxgi` | streams `progress` events; resolves on success |
| `relaunch_as_admin` | – | UAC-elevates and exits |

Versions are compared in Rust with the `semver` crate (a JS string compare gets `1.10` vs `1.9`
wrong). `installedVersion` is read from the installed `NZ-FPS-Limiter.addon64`'s VERSIONINFO, so an
addon built before versioning existed reads as `null` → treated as a fresh install.

## Build (Windows only)
Cannot build on macOS — it targets Windows + WebView2. Real builds run in CI
(`.github/workflows/installer.yml`). To build locally on a Windows box:

```sh
# one-time
rustup target add x86_64-pc-windows-msvc
cargo install tauri-cli --version "^2"

# from installer/
cargo tauri build
# portable exe: src-tauri/target/release/nz-installer.exe
```

`cargo tauri dev` runs it with hot-reload during development.

## Testing while the repo is private

Download-mode pulls release assets from GitHub, which only works **anonymously on a public repo**.
Until the repo goes public, point the installer elsewhere with env vars (read in
`resolve_source()`; none are needed in production):

| Env var | Purpose |
|---|---|
| `NZ_INSTALLER_SOURCE` | A local `.zip` path **or** an `http(s)` URL to use as the release zip. Bypasses GitHub entirely — full detect → copy → enable flow with no repo access. |
| `NZ_INSTALLER_VERSION` | The version that override should report as "latest" (default `0.0.0`). Drives the install/update/reinstall comparison. |
| `NZ_GITHUB_TOKEN` | A PAT with `repo` scope. Makes the **real** GitHub path read the private repo's latest release and download its private asset (via the asset API + `Accept: octet-stream`). |

```sh
# Offline E2E against a locally built bundle, simulating "latest = 1.3.0":
NZ_INSTALLER_SOURCE="C:\path\NZ-FPS-Limiter_v1.3.0.zip" NZ_INSTALLER_VERSION=1.3.0 cargo tauri dev

# Real private-repo path (after pushing a v* tag):
NZ_GITHUB_TOKEN=ghp_xxx cargo tauri dev
```

## Notes / TODO
- **Unsigned** → SmartScreen will warn ("More info → Run anyway"). Expected for now.
- **WebView2** runtime is required and assumed present — it ships by default on Win10 (2020+)
  and Win11, i.e. effectively every FiveM box. A cold machine without it shows a blank window;
  tell those users to install the Evergreen WebView2 runtime from Microsoft. (Decided: ship the
  raw portable exe + this doc note rather than an NSIS bundle with the bootstrapper embedded.)
- The frontend is a copy of the design mockup (`nightzoom-installer-ui.html`). When the
  design changes, re-sync `ui/index.html` (keep the `TAURI` controller block at the end).
- Distribute the raw `nz-installer.exe`; download-mode means it carries no DLLs itself.
