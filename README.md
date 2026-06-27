# NightZoom FPS Limiter

A small, open-source [ReShade](https://reshade.me) addon that hard-caps your game's frame
rate to **exactly 60 FPS**. It adds its own window to the ReShade overlay with a single
checkbox — tick it to cap, untick to remove the cap.

> Made by **Nipeno** · [Discord](https://discord.gg/nightzoom) · Licensed under **GPLv3**

## What it does (and what it doesn't)

The whole point of this being open source is that you can read exactly what you're installing.
In short:

- ✅ Caps the frame rate to 60 FPS by pacing the `present` event with a hybrid sleep + busy-wait.
- ✅ Remembers the checkbox state across restarts via ReShade's own config.
- ✅ Shows a logo, a credit line, a Discord link, and a link back to this repo.
- ❌ **No telemetry, no analytics, no network access.** The only outbound action is opening
  your browser when *you* click the Discord or GitHub buttons.
- ❌ No file writing of its own (state lives in ReShade's config).
- ❌ No game memory editing, no upscaling, no swapchain surgery — it only times frames.

Everything lives in a single source file: [`src/main.cpp`](src/main.cpp).

## ⚠️ Requirements & ban warning

- **Requires the ADDON-ENABLED build of ReShade.** Get it from <https://reshade.me> and tick
  "addon support" during setup. The addon also needs to target an API version your ReShade
  supports — see [Compatibility](#compatibility).
- **Anticheat / ban risk.** This is a DLL injected into the game process. Injecting addons
  into anticheat-protected **online** games (e.g. GTA Online / FiveM) can get your account
  or hardware banned — anticheats flag the *injection*, not the harmless feature. **Intended
  for single-player / development use.** For a 60 FPS cap in online games, use an external
  tool instead (RTSS, or the NVIDIA/AMD driver frame-rate limit) — nothing gets injected.

## Install

1. Get `NightZoom.addon64` (download a prebuilt one or [build it yourself](#build-from-source)).
2. Drop it into the game's folder next to the ReShade DLL (e.g. `dxgi.dll` / `d3d11.dll`).
3. Launch the game, open the ReShade overlay (`Home` by default), find the
   **NightZoom FPS Limiter** window, and tick **Limit to 60 FPS**.

That's it — a single `.addon64` file. The logo is baked into the DLL, so there's nothing else
to copy.

## Build from source

Prerequisites: Visual Studio 2022 ("Desktop development with C++" workload) and CMake 3.20+.
Windows x64 only.

```sh
cmake -S . -B build -G "Visual Studio 17 2022" -A x64
cmake --build build --config Release
# -> build/Release/NightZoom.addon64
```

Continuous integration builds the addon on every push via GitHub Actions
([`.github/workflows/build.yml`](.github/workflows/build.yml)); the compiled `.addon64` is
attached as a build artifact.

### Dependencies

Only the headers needed to compile are vendored under `deps/` — not the full ReShade source:

- **`deps/reshade/`** — addon SDK headers from [crosire/reshade](https://github.com/crosire/reshade),
  tag **v6.7.3** (addon **API version 18**).
- **`deps/imgui/`** — `imgui.h` + `imconfig.h` from the **docking branch** of
  [ocornut/imgui](https://github.com/ocornut/imgui) at the exact commit ReShade pins
  (`3912b3d`, `IMGUI_VERSION_NUM 19250`). The docking branch is required: the ReShade overlay
  header uses docking-only types. Only declarations are needed — the addon calls ReShade's
  bundled ImGui at runtime through the function table, so no ImGui `.cpp` is compiled.

Everything else is Windows system libraries: `winmm` (timer granularity), `shell32`
(open links), `windowscodecs` + `ole32` (WIC, to decode the embedded logo).

To refresh the vendored headers:

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
**v6.6.0 – v6.7.3**). If your ReShade refuses to load it with a message like
`requested API version (X) is not supported (Y)`, either update ReShade, or rebuild against the
matching `deps/reshade` tag (see refresh steps above).

## How it works

- **Frame cap** — in the `present` callback we measure time since the last present and, when
  enabled, block until one 1/60 s interval has elapsed: sleep until ~1 ms before target, then
  busy-wait the remainder for frame-accurate pacing (a plain `Sleep` stutters due to timer
  granularity). `timeBeginPeriod(1)`/`timeEndPeriod(1)` tighten sleep granularity.
- **Persistence** — the checkbox value is read/written with `reshade::get/set_config_value`
  under the `[NightZoom]` config section.
- **Logo** — embedded as a byte array in [`src/logo_data.h`](src/logo_data.h), decoded from
  memory via WIC and uploaded as a ReShade texture. If decoding ever fails, a bordered
  `[ NightZoom logo ]` placeholder is drawn instead.

### Changing the logo

Replace `src/logo_data.h` with a freshly generated header:

```sh
sips -z 512 512 your-logo.png --out /tmp/logo.png                                   # resize
pngquant --quality=70-90 --strip --force --output /tmp/logo.png /tmp/logo.png       # compress
{ echo '#pragma once'; echo; xxd -i -n g_logo_png /tmp/logo.png; } > src/logo_data.h
```

## Community

This addon is made for **NightZoom**. Questions, bug reports, or just want to hang out?

👉 **Join the Discord: <https://discord.gg/nightzoom>**

The same link is available in-app via the **Join the Discord** button in the overlay window.

## License

GPLv3 — see [LICENSE](LICENSE). You're free to use, study, modify, and redistribute this,
but any distributed fork or derivative must also be open-sourced under the GPL. No closed-source
forks.

## Credits

Made by **Nipeno** · [NightZoom Discord](https://discord.gg/nightzoom).
Built on [ReShade](https://reshade.me) and [Dear ImGui](https://github.com/ocornut/imgui).
