# NightZoom FPS Limiter — Installer (Tauri)

A small Windows GUI installer (`~5 MB .exe`) that finds the user's FiveM install,
downloads the **latest** release of the limiter from GitHub, copies the files into
`FiveM.app\plugins`, and writes the ReShade5 acknowledgement to `CitizenFX.ini` — i.e.
the whole `Enable-ReShade.bat` flow, plus the file copy, behind a branded UI.

## Layout
```
installer/
├── ui/index.html          # frontend (HTML/CSS/JS). Same file previews in a browser.
└── src-tauri/
    ├── src/lib.rs          # backend: detect_fivem, install, relaunch_as_admin
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
| `detect_fivem` | `manualPath?` | `{found, path, plugins, existingReshade, computerName, reshadeId}` |
| `install` | `fivemPath, replaceDxgi` | streams `progress` events; resolves on success |
| `relaunch_as_admin` | – | UAC-elevates and exits |

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

## Notes / TODO
- **Unsigned** → SmartScreen will warn ("More info → Run anyway"). Expected for now.
- **WebView2** runtime is required and assumed present — it ships by default on Win10 (2020+)
  and Win11, i.e. effectively every FiveM box. A cold machine without it shows a blank window;
  tell those users to install the Evergreen WebView2 runtime from Microsoft. (Decided: ship the
  raw portable exe + this doc note rather than an NSIS bundle with the bootstrapper embedded.)
- The frontend is a copy of the design mockup (`nightzoom-installer-ui.html`). When the
  design changes, re-sync `ui/index.html` (keep the `TAURI` controller block at the end).
- Distribute the raw `nz-installer.exe`; download-mode means it carries no DLLs itself.
