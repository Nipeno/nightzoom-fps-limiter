# Building NightZoom FPS Limiter (developer guide)

This is the technical/developer documentation. If you just want to *use* the addon, see the
[README](README.md).

NightZoom FPS Limiter is a single-DLL [ReShade](https://reshade.me) addon written in C++17, Windows x64 only.
The output is `NZ-FPS-Limiter.addon64`. All logic lives in [`src/main.cpp`](src/main.cpp).

## Build

Prerequisites: Visual Studio 2022 ("Desktop development with C++" workload) and CMake 3.20+.

```sh
cmake -S . -B build -G "Visual Studio 17 2022" -A x64
cmake --build build --config Release
# -> build/Release/NZ-FPS-Limiter.addon64
```

CI builds the addon on every push and PR via GitHub Actions
([`.github/workflows/build.yml`](.github/workflows/build.yml)) on a `windows-2022` runner; the
assembled bundle is attached to each run as a build artifact.

### Versioning

The version comes from the git tag and nothing else. CI passes `-DNZ_VERSION=X.Y.Z`, which flows
into three places: the DLL's `VERSIONINFO` resource (generated from
[`src/version.rc.in`](src/version.rc.in), so Windows file properties show it), the
`NZ_VERSION_STR` compile definition the overlay prints, and the release zip's filename. Local and
CI builds don't pass it, so they report **`0.0.0`** — that's the intended "not a release build"
signal, not a bug. To build a versioned binary by hand:

```sh
cmake -S . -B build -G "Visual Studio 17 2022" -A x64 -DNZ_VERSION=2.3.0
```

### Runtime library

`CMAKE_MSVC_RUNTIME_LIBRARY` is set to `MultiThreaded` (`/MT`), so the CRT links statically. This
DLL is injected into someone else's game process; with the dynamic CRT it would need the VC++
redistributable present, and on a machine without it ReShade just silently declines to load the
addon — which is indistinguishable from "the addon is broken" for the person reporting it. Keep it
static.

## Dependencies

Only the headers needed to compile are vendored under `deps/` - not the full ReShade source:

- **`deps/reshade/`** - addon SDK headers from [crosire/reshade](https://github.com/crosire/reshade),
  tag **v6.7.3** (addon **API version 18**).
- **`deps/imgui/`** - `imgui.h` + `imconfig.h` from the **docking branch** of
  [ocornut/imgui](https://github.com/ocornut/imgui) at the exact commit ReShade pins
  (`3912b3d`, `IMGUI_VERSION_NUM 19250`). The docking branch is required: the ReShade overlay
  header uses docking-only types (`ImGuiDockNodeFlags`, `ImGuiWindowClass`). Only declarations
  are needed - the addon calls ReShade's bundled ImGui at runtime through the function table,
  so no ImGui `.cpp` is compiled.

Everything else is Windows system libraries: `winmm` (timer granularity), `shell32`
(open links), `windowscodecs` + `ole32` (WIC, to decode the embedded logo).

### Refreshing the vendored headers

```sh
RESHADE_REF=v6.7.3   # addon API version 18
for f in reshade.hpp reshade_api.hpp reshade_api_device.hpp reshade_api_format.hpp \
         reshade_api_pipeline.hpp reshade_api_resource.hpp reshade_events.hpp reshade_overlay.hpp; do
  curl -fsSL -o "deps/reshade/$f" "https://raw.githubusercontent.com/crosire/reshade/$RESHADE_REF/include/$f"
done

IMGUI_SHA=3912b3d9a9c1b3f17431aebafd86d2f40ee6e59c   # must match IMGUI_VERSION_NUM in reshade_overlay.hpp
for f in imgui.h imconfig.h; do
  curl -fsSL -o "deps/imgui/$f" "https://raw.githubusercontent.com/ocornut/imgui/$IMGUI_SHA/$f"
done
```

## Compatibility

A ReShade addon must target an API version **≤** the ReShade build it loads into - ReShade
loads older addons but refuses newer ones. This addon is built against **API 18** (ReShade
**v6.6.0 – v6.7.3**). If ReShade refuses to load it with a message like
`requested API version (X) is not supported (Y)`, either update ReShade or rebuild against the
matching `deps/reshade` tag (see refresh steps above).

## How it works

- **Frame cap** - in the `reshade::addon_event::present` callback we measure time since the last
  present and, when enabled, block until one 1/60 s interval has elapsed: sleep until ~1 ms before
  the target, then busy-wait the remainder for frame-accurate pacing (a plain `Sleep` stutters due
  to Windows timer granularity). `timeBeginPeriod(1)`/`timeEndPeriod(1)` tighten sleep granularity
  at load/unload.
- **Persistence** - the checkbox value is read in `init_effect_runtime` and written on toggle via
  `reshade::get/set_config_value` under the `[NZ-FPS-Limiter]` config section.
- **Overlay** - registered with a named title via `reshade::register_overlay`, so it appears as its
  own window in the ReShade menu.
- **Logo** - embedded as a byte array in [`src/logo_data.h`](src/logo_data.h), decoded from memory
  via WIC and uploaded as a ReShade texture (`create_resource` / `create_resource_view`), freed in
  `destroy_effect_runtime`. If decoding ever fails, a bordered `[ NightZoom FPS Limiter logo ]` placeholder is
  drawn instead.

### Changing the logo

Replace `src/logo_data.h` with a freshly generated header:

```sh
sips -z 512 512 your-logo.png --out /tmp/logo.png                                   # resize
pngquant --quality=70-90 --strip --force --output /tmp/logo.png /tmp/logo.png       # compress
{ echo '#pragma once'; echo; xxd -i -n g_logo_png /tmp/logo.png; } > src/logo_data.h
```

## Release bundle (CI)

Two workflows, deliberately split so only one of them can write to the repo:

- [`build.yml`](.github/workflows/build.yml) — dev CI. Runs on push/PR, `permissions: contents:
  read`, builds at version `0.0.0` and uploads the bundle as an artifact. Publishes nothing.
- [`release.yml`](.github/workflows/release.yml) — runs on a `v*` tag with `contents: write`,
  builds at the tag's version and attaches the zip to the GitHub Release. A manual
  `workflow_dispatch` run is a dry run: it builds and uploads an artifact but creates no Release.

Both assemble the same drag-and-drop bundle for end users. After compiling the addon they:

1. **Resolve the latest ReShade** - scrape `https://reshade.me/` for
   `ReShade_Setup_<version>_Addon.exe`. If scraping fails (site change / rate-limit), it falls
   back to a pinned version (`6.7.3`) so the build never breaks.
2. **Download + extract** the add-on-enabled installer and pull out `ReShade64.dll` via
   `7z e ReShade_Setup.exe ReShade64.dll`, then copy it to **`dxgi.dll`** (the name FiveM loads
   ReShade under from its `plugins` folder).
3. **Zip** `dxgi.dll` + `NZ-FPS-Limiter.addon64` + `packaging/Enable-ReShade.bat` +
   `packaging/Install Guide.html` + both license notices + `reshade-version.txt` into
   `NZ-FPS-Limiter_v<ver>.zip`.

The zip name carries the **addon** version (from the tag), which is why the bundled ReShade
version is recorded in `reshade-version.txt` inside the zip instead. The end-user install guide
lives in `packaging/Install Guide.html` - users who already have ReShade just skip the `dxgi.dll`
/ enable steps (the guide says where).

### `Enable-ReShade.bat` (FiveM unblock helper)

`packaging/Enable-ReShade.bat` is bundled into the zip. FiveM blocks ReShade 5+ until the
user adds `[Addons] ReShade5=ID:<id> acknowledged ...` to `CitizenFX.ini`. The `<id>` is
**`Joaat(lowercase(%COMPUTERNAME%))`** - derived purely from the PC name (see FiveM
`code/components/rage-graphics-five/src/ReShadeFixups.cpp` + `HashString` in
`code/client/shared/Utils.h`), so it's computable offline with no FiveM launch. Validated against
ground truth: computer name `PC` → `46750aa6`.

The bat is a self-contained polyglot: a cmd header bootstraps an embedded PowerShell block (read
from the `#:PS:#` marker) that computes the ID, locates `CitizenFX.ini` (parent of the `plugins`
folder it ships in, then `%LOCALAPPDATA%`, then the `fivem://` registry handler), and writes the
key via `WritePrivateProfileString` (same Win32 API FiveM reads with - safe section merge).

## Cutting a release

Update [`CHANGELOG.md`](CHANGELOG.md), then push a tag:

```sh
git tag vX.Y.Z && git push origin vX.Y.Z
```

`release.yml` does the rest. Verify afterwards - the asset name must carry the tag's version, not
ReShade's:

```sh
gh release view vX.Y.Z --repo Nipeno/nightzoom-fps-limiter --json assets
```

> The bundle carries whatever ReShade was latest at release time. Because ReShade loads addons
> with an API version ≤ its own, bundling a newer ReShade never breaks this addon (API 18).

## Third-party licenses

Vendored headers and the bundled binary keep their upstream notices under `third_party/`. Both
ship inside the release zip:

- [`third_party/ReShade-LICENSE.txt`](third_party/ReShade-LICENSE.txt) - BSD 3-Clause, covers the
  vendored `deps/reshade` headers **and** the bundled `dxgi.dll` (ReShade binary).
- [`third_party/Dear-ImGui-LICENSE.txt`](third_party/Dear-ImGui-LICENSE.txt) - MIT, covers the
  vendored `deps/imgui` headers.

## License

NightZoom FPS Limiter itself is GPLv3 - see [LICENSE](LICENSE). Any distributed fork or derivative must also
be open-sourced under the GPL. (Bundling the BSD-licensed ReShade alongside it is "mere
aggregation" and permitted.)
