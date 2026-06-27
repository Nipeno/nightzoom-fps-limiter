# Building NightZoom FPS Limiter (developer guide)

This is the technical/developer documentation. If you just want to *use* the addon, see the
[README](README.md).

NightZoom is a single-DLL [ReShade](https://reshade.me) addon written in C++17, Windows x64 only.
The output is `NightZoom.addon64`. All logic lives in [`src/main.cpp`](src/main.cpp).

## Build

Prerequisites: Visual Studio 2022 ("Desktop development with C++" workload) and CMake 3.20+.

```sh
cmake -S . -B build -G "Visual Studio 17 2022" -A x64
cmake --build build --config Release
# -> build/Release/NightZoom.addon64
```

CI builds the addon on every push via GitHub Actions
([`.github/workflows/build.yml`](.github/workflows/build.yml)) on a `windows-2022` runner; the
compiled `.addon64` is attached to each run as a build artifact.

## Dependencies

Only the headers needed to compile are vendored under `deps/` — not the full ReShade source:

- **`deps/reshade/`** — addon SDK headers from [crosire/reshade](https://github.com/crosire/reshade),
  tag **v6.7.3** (addon **API version 18**).
- **`deps/imgui/`** — `imgui.h` + `imconfig.h` from the **docking branch** of
  [ocornut/imgui](https://github.com/ocornut/imgui) at the exact commit ReShade pins
  (`3912b3d`, `IMGUI_VERSION_NUM 19250`). The docking branch is required: the ReShade overlay
  header uses docking-only types (`ImGuiDockNodeFlags`, `ImGuiWindowClass`). Only declarations
  are needed — the addon calls ReShade's bundled ImGui at runtime through the function table,
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

A ReShade addon must target an API version **≤** the ReShade build it loads into — ReShade
loads older addons but refuses newer ones. This addon is built against **API 18** (ReShade
**v6.6.0 – v6.7.3**). If ReShade refuses to load it with a message like
`requested API version (X) is not supported (Y)`, either update ReShade or rebuild against the
matching `deps/reshade` tag (see refresh steps above).

## How it works

- **Frame cap** — in the `reshade::addon_event::present` callback we measure time since the last
  present and, when enabled, block until one 1/60 s interval has elapsed: sleep until ~1 ms before
  the target, then busy-wait the remainder for frame-accurate pacing (a plain `Sleep` stutters due
  to Windows timer granularity). `timeBeginPeriod(1)`/`timeEndPeriod(1)` tighten sleep granularity
  at load/unload.
- **Persistence** — the checkbox value is read in `init_effect_runtime` and written on toggle via
  `reshade::get/set_config_value` under the `[NightZoom]` config section.
- **Overlay** — registered with a named title via `reshade::register_overlay`, so it appears as its
  own window in the ReShade menu.
- **Logo** — embedded as a byte array in [`src/logo_data.h`](src/logo_data.h), decoded from memory
  via WIC and uploaded as a ReShade texture (`create_resource` / `create_resource_view`), freed in
  `destroy_effect_runtime`. If decoding ever fails, a bordered `[ NightZoom logo ]` placeholder is
  drawn instead.

### Changing the logo

Replace `src/logo_data.h` with a freshly generated header:

```sh
sips -z 512 512 your-logo.png --out /tmp/logo.png                                   # resize
pngquant --quality=70-90 --strip --force --output /tmp/logo.png /tmp/logo.png       # compress
{ echo '#pragma once'; echo; xxd -i -n g_logo_png /tmp/logo.png; } > src/logo_data.h
```

## License

GPLv3 — see [LICENSE](LICENSE). Any distributed fork or derivative must also be open-sourced
under the GPL.
