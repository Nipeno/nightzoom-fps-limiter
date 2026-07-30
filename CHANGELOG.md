# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Every release bundles whatever ReShade was latest at build time; the exact version is recorded in
`reshade-version.txt` inside the zip.

## [Unreleased]

## [2.4.0] - 2026-07-30

### Fixed
- **The add-on now loads on older ReShade.** It targeted addon API 18, which requires ReShade 6.6 or
  newer, so anyone on an older install got
  `Failed to register add-on, because the requested API version (18) is not supported (14)` and no
  overlay at all. It now targets **API 11**, so it loads on **any ReShade from 6.1.0 onwards**.
  Nothing about the add-on's behaviour or appearance changes, and users on the newest ReShade are
  unaffected — ReShade accepts any add-on older than itself.
- The install guide no longer assumes your existing ReShade is called `dxgi.dll`. It can be installed
  under several names (`dxgi.dll`, `d3d11.dll`, `d3d12.dll`, `d3d9.dll`, `opengl32.dll`), which meant
  the old "replace your `dxgi.dll`" step could quietly do nothing — the existing ReShade kept loading
  first. The guide now says to leave your ReShade alone in the normal case, and if it genuinely does
  need replacing, to rename the replacement to match whichever file you already have.

### Changed
- Vendored dependencies moved to the ReShade **v6.1.0** SDK headers and ImGui **1.90.4** (`19040`,
  docking branch). The low pin is intentional: it is the compatibility floor.

## [2.3.0] - 2026-07-30

First release with the repository public.

### Added
- The addon now carries a version. It shows in the overlay under "Made by Nipeno" and in the
  DLL's Windows file properties (right-click → Properties → Details), so a bug report can say
  which build it came from.
- `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, and this changelog.
- Issue forms and a pull request template.
- The bundle now ships `Dear-ImGui-LICENSE.txt` (previously only the ReShade notice) and
  `reshade-version.txt` recording which ReShade is inside.
- Documented that in-game VSync should be turned off — running it alongside the limiter can stack
  the two waits into stutter.

### Changed
- **Release zips are now named after the release** (`NZ-FPS-Limiter_v2.3.0.zip`). They were
  previously named after the bundled ReShade version, so different releases shipped files with
  identical names.
- The addon links the C runtime statically, so it no longer needs the Visual C++ redistributable
  installed. Without it, ReShade would silently decline to load the addon.
- Bug reports and questions are handled on GitHub Issues. The Discord is the NightZoom server's,
  for people not on the server yet.
- Release notes now list the actual changes per tag instead of a fixed boilerplate body.
- CI and releases are separate workflows; only the release workflow has write access.

## [2.2.0] - 2026-06-27

### Changed
- Frame pacing now uses a high-resolution waitable timer with drift compensation, replacing the
  plain sleep-and-spin loop. Smoother pacing and less CPU burn.

### Fixed
- Removed an outdated note about ReShade versions from the install guide.

## [2.1.2] - 2026-06-27

### Changed
- `INSTALL.html` renamed to `Install Guide.html`; README trimmed and credits added.
- Shorter, less repetitive release notes.

## [2.1.1] - 2026-06-27

### Changed
- Hardened `Enable-ReShade.bat`; polished the install guide and README.

## [2.1.0] - 2026-06-27

### Added
- Self-contained `INSTALL.html` guide, replacing the plain-text `INSTALL.txt`.

### Fixed
- README now says to run the `.bat` from the plugins folder.
- `.DS_Store` ignored and removed from the tree.

## [2.0.0] - 2026-06-27

### Changed
- Renamed to **NightZoom FPS Limiter**; the addon builds as `NZ-FPS-Limiter.addon64` and its
  config section is `[NZ-FPS-Limiter]`. **This resets a previously saved on/off choice.**
- Consolidated the two downloads into a single all-in-one bundle. Users who already have ReShade
  skip the `dxgi.dll` steps.

## [1.2.0] - 2026-06-27

### Added
- `Enable-ReShade.bat`, which writes FiveM's `CitizenFX.ini` unblock line automatically using an
  ID computed from the PC name — no manual console copying, and idempotent if already enabled.

## [1.1.1] - 2026-06-27

### Fixed
- Documented that NVE remaps the ReShade overlay key to <kbd>Insert</kbd>.

## [1.1.0] - 2026-06-27

### Added
- Drag-and-drop FiveM bundle that auto-fetches the latest ReShade at build time.
- Install guide covering the FiveM "ReShade was blocked" first-launch fix, how to use, why 60 FPS,
  and uninstall steps.
- Release notes on tagged releases.

## [1.0.0] - 2026-06-27

Initial release.

### Added
- ReShade addon that hard-caps the game to exactly 60 FPS, with a one-checkbox overlay whose state
  persists via ReShade's own config.
- Logo embedded in the DLL and decoded from memory, so no image file ships alongside it.
- GPLv3 license, a "View Source on GitHub" button in the overlay, and split user/developer docs.
- GitHub Actions build targeting ReShade addon API 18 (SDK v6.7.3).

[Unreleased]: https://github.com/Nipeno/nightzoom-fps-limiter/compare/v2.4.0...HEAD
[2.4.0]: https://github.com/Nipeno/nightzoom-fps-limiter/compare/v2.3.0...v2.4.0
[2.3.0]: https://github.com/Nipeno/nightzoom-fps-limiter/compare/v2.2.0...v2.3.0
[2.2.0]: https://github.com/Nipeno/nightzoom-fps-limiter/compare/v2.1.2...v2.2.0
[2.1.2]: https://github.com/Nipeno/nightzoom-fps-limiter/compare/v2.1.1...v2.1.2
[2.1.1]: https://github.com/Nipeno/nightzoom-fps-limiter/compare/v2.1.0...v2.1.1
[2.1.0]: https://github.com/Nipeno/nightzoom-fps-limiter/compare/v2.0.0...v2.1.0
[2.0.0]: https://github.com/Nipeno/nightzoom-fps-limiter/compare/v1.2.0...v2.0.0
[1.2.0]: https://github.com/Nipeno/nightzoom-fps-limiter/compare/v1.1.1...v1.2.0
[1.1.1]: https://github.com/Nipeno/nightzoom-fps-limiter/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/Nipeno/nightzoom-fps-limiter/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/Nipeno/nightzoom-fps-limiter/releases/tag/v1.0.0
