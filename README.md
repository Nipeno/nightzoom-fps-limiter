# NightZoom FPS Limiter

A minimal [ReShade](https://reshade.me) addon that hard-caps the game's frame rate to **exactly 60 FPS**.
It adds its own window to the ReShade overlay with a single checkbox toggle.

The overlay window **NightZoom FPS Limiter** contains, top to bottom:

1. A logo placeholder (bordered `[ NightZoom logo ]` box).
2. A **Limit to 60 FPS** checkbox.
3. A **Made by Nipeno** credit line.
4. A clickable Discord link to <https://discord.gg/nightzoom> (full URL shown as a copy/paste fallback).

When checked, the frame rate is paced to 60 FPS in the `present` callback using a hybrid
sleep + busy-wait. Unchecking removes the cap immediately. The checkbox state is saved
via ReShade's own config and survives a restart.

## ⚠️ Requirements & warning

- **Requires the ADDON-ENABLED build of ReShade.** The default ReShade installer has addon
  support; download it from <https://reshade.me> and tick "addon support" during setup.
- **Anticheat / ban risk.** Injecting addons into anticheat-protected online games
  (e.g. GTA Online / FiveM) can get you banned. This addon is intended for
  **single-player / development use only.**

## Build (Visual Studio 2022 / MSVC, x64)

Prerequisites: Visual Studio 2022 with the "Desktop development with C++" workload, and CMake 3.20+.

```sh
cmake -S . -B build -G "Visual Studio 17 2022" -A x64
cmake --build build --config Release
```

The output is `build/Release/NightZoom.addon64`.

### Dependencies (already vendored under `deps/`)

Only the headers needed to compile are included — not the full ReShade source tree:

- `deps/reshade/` — the addon SDK headers from the official
  [crosire/reshade](https://github.com/crosire/reshade) repo (`include/` folder, `main` branch).
- `deps/imgui/` — `imgui.h` + `imconfig.h` from the **docking branch** of
  [ocornut/imgui](https://github.com/ocornut/imgui) at the exact commit ReShade pins as its
  submodule (`3912b3d`, `IMGUI_VERSION_NUM 19250`). The docking branch is required — the
  ReShade overlay header uses docking-only types (`ImGuiDockNodeFlags`, `ImGuiWindowClass`).
  Only the declarations are needed; the addon calls ReShade's bundled ImGui at runtime
  through the function table, so no ImGui `.cpp` is compiled.

To refresh them:

```sh
# ReShade addon SDK headers
for f in reshade.hpp reshade_api.hpp reshade_api_device.hpp reshade_api_format.hpp \
         reshade_api_pipeline.hpp reshade_api_resource.hpp reshade_events.hpp reshade_overlay.hpp; do
  curl -fsSL -o "deps/reshade/$f" "https://raw.githubusercontent.com/crosire/reshade/main/include/$f"
done

# Matching Dear ImGui declarations (docking branch, exact ReShade submodule commit)
IMGUI_SHA=3912b3d9a9c1b3f17431aebafd86d2f40ee6e59c
for f in imgui.h imconfig.h; do
  curl -fsSL -o "deps/imgui/$f" "https://raw.githubusercontent.com/ocornut/imgui/$IMGUI_SHA/$f"
done
```

> If you update the ReShade headers and the overlay header bumps its required ImGui
> version, change the imgui tag above to match `IMGUI_VERSION_NUM` in `reshade_overlay.hpp`.

## Install

1. Build `NightZoom.addon64`.
2. Drop it into the game's folder next to the ReShade DLL (e.g. `dxgi.dll` / `d3d11.dll`).
3. Launch the game, open the ReShade overlay (`Home` by default), and find the
   **NightZoom FPS Limiter** window. Tick **Limit to 60 FPS**.

### Optional real logo

The logo is a placeholder by design. See `draw_logo()` in `src/main.cpp` for a `// TODO`
showing where to load `NightZoom_logo.png` and feed a real texture to `ImGui::Image()`.
