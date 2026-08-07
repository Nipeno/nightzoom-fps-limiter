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
  tag **v6.1.0** (addon **API version 11**). Deliberately old - see [Compatibility](#compatibility).
- **`deps/imgui/`** - `imgui.h` + `imconfig.h` from the **docking branch** of
  [ocornut/imgui](https://github.com/ocornut/imgui) at the exact commit ReShade v6.1.0 pins
  (`c6aa051`, ImGui 1.90.4, `IMGUI_VERSION_NUM 19040`). The docking branch is required: the ReShade
  overlay header uses docking-only types (`ImGuiDockNodeFlags`, `ImGuiWindowClass`). Only
  declarations are needed - the addon calls ReShade's bundled ImGui at runtime through the function
  table, so no ImGui `.cpp` is compiled.

`src/main.cpp` defines `ImTextureID ImU64` before including `imgui.h`. ImGui only made that the
default in 1.92; at 1.90.4 it is `void*`, which `g_logo_view.handle` (a `uint64_t`) cannot be
`static_cast` to. ReShade's overlay header has a `static_assert` demanding exactly this define.

Everything else is Windows system libraries: `winmm` (timer granularity), `shell32`
(open links), `windowscodecs` + `ole32` (WIC, to decode the embedded logo).

### Refreshing the vendored headers

```sh
RESHADE_REF=v6.1.0   # addon API version 11
for f in reshade.hpp reshade_api.hpp reshade_api_device.hpp reshade_api_format.hpp \
         reshade_api_pipeline.hpp reshade_api_resource.hpp reshade_events.hpp reshade_overlay.hpp; do
  curl -fsSL -o "deps/reshade/$f" "https://raw.githubusercontent.com/crosire/reshade/$RESHADE_REF/include/$f"
done

IMGUI_SHA=c6aa051629753f0ef0d26bf775a8b6a92aa213b2   # must match IMGUI_VERSION_NUM in reshade_overlay.hpp
for f in imgui.h imconfig.h; do
  curl -fsSL -o "deps/imgui/$f" "https://raw.githubusercontent.com/ocornut/imgui/$IMGUI_SHA/$f"
done
```

## Compatibility

A ReShade addon must target an API version **≤** the ReShade build it loads into - ReShade loads
older addons but refuses newer ones (`api_version > RESHADE_API_VERSION` is the only rejection;
anything older is accepted, all the way down to API 1).

So the SDK pin is a **compatibility floor, and lower is better**. This addon targets **API 11**,
which means it loads on **ReShade 6.1.0 and everything newer**. That is deliberate: plenty of users
run an older ReShade than the current release - typically because their graphics pack install is a
few versions behind - and an addon built against the newest SDK (API 18, ReShade 6.6+) is rejected
outright for all of them. Targeting low costs us nothing and removes that whole failure mode.

Targeting an old SDK does **not** penalise users on current ReShade. `ReShadeGetImGuiFunctionTable()`
in the latest builds still serves the older ImGui tables (18600, 18971, 19000, **19040**, 19180,
19191, 19222, 19250), so a 19040-built addon works on 6.7.3 and later just as well.

APIs 11-14 all share the same ImGui pin (19040), so API 11 costs nothing over API 14 while covering
three more ReShade releases. **API 8** (ReShade 5.9, ImGui 18971) is the true floor before
`get_config_value` has to be renamed to `config_get_value`; we stop at 11 because the extra reach
is negligible.

If ReShade still refuses to load the addon with `requested API version (X) is not supported (Y)`,
that user's ReShade predates 6.1 and needs updating.

## How it works

- **Frame cap** - in the `reshade::addon_event::present` callback we measure time since the last
  present and, when enabled, block until one 1/60 s interval has elapsed: sleep until ~1 ms before
  the target, then busy-wait the remainder for frame-accurate pacing (a plain `Sleep` stutters due
  to Windows timer granularity). `timeBeginPeriod(1)`/`timeEndPeriod(1)` tighten sleep granularity
  at load/unload.
- **Persistence** - the checkbox value is read in `init_effect_runtime` and written on toggle via
  `reshade::get/set_config_value` under the `[NZ-FPS-Limiter]` config section.
- **Overlay** - registered with a named title via `reshade::register_overlay`, so it appears as its
  own window in the ReShade menu. ReShade draws addon overlays with a plain `ImGui::Begin`, and
  pipes ImGui's settings handlers into `ReShade.ini` (`[OVERLAY] Window=` / `Docking=`) instead of
  an `imgui.ini` - so position, size, collapsed state and dock slot are persisted for us, keyed by
  the **window title**. Renaming the overlay would discard every user's saved layout. We only set a
  first-run size/position with `ImGuiCond_FirstUseEver`, which a saved entry overrides.
  Docking the window into ReShade's own Home/Add-ons/Settings tab group by default is not possible
  from an addon: ReShade builds that layout with the `DockBuilder*` API, which is ImGui-internal and
  absent from `ReShadeGetImGuiFunctionTable()`, and the node IDs it generates are not derivable.
  Users can still drag the window in themselves, and that choice sticks.
- **Logging** - `nz_log()` wraps `reshade::log_message`, so everything lands in `ReShade.log`
  alongside ReShade's own lines, prefixed with the add-on name. Load, effect-runtime init (with the
  graphics API in use), the first present, and any failure are logged at INFO/WARN; routine detail
  (config read, teardown) is DEBUG. Note that **ReShade has no runtime log level** - the
  `ReShadeLogMessage` export (`source/addon.cpp`) passes every message straight to
  `reshade::log::message`, and `RESHADE_VERBOSE_LOG` is a compile-time define affecting only
  ReShade's own callsites. So DEBUG lines are always written too, and the level is severity
  labelling for whoever reads the log rather than a filter - which is why nothing is logged per
  frame.
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

Two build workflows, deliberately split so only one of them can write to the repo, plus a
maintenance job:

- [`build.yml`](.github/workflows/build.yml) — dev CI. Runs on push to `main` and on PRs,
  `permissions: contents: read`, builds at version `0.0.0` and uploads the bundle as an artifact.
  Publishes nothing. A `concurrency` group cancels the previous in-flight run when you push again,
  and `paths-ignore` skips the whole build for docs-only changes (`*.md`, `docs/**`, issue/PR
  templates, `labels.yml`, `LICENSE`) — so a docs PR gets no Build check at all, by design.
  `packaging/**` is deliberately *not* ignored: those files ship inside the zip.
  Note the filter is evaluated per *event*, not per commit: on a PR it looks at every file the PR
  changes, so a docs-only commit pushed onto a PR that also touches code still builds. Only a PR
  (or a push to `main`) that is entirely docs is skipped.
- [`release.yml`](.github/workflows/release.yml) — runs on a `v*` tag with `contents: write`,
  builds at the tag's version and attaches the zip to the GitHub Release. A manual
  `workflow_dispatch` run is a dry run: it builds and uploads an artifact but creates no Release.
  Its concurrency group uses `cancel-in-progress: false` — a publish must never be killed half-way.
- [`reshade-watch.yml`](.github/workflows/reshade-watch.yml) — weekly (Mondays 06:00 UTC) and
  manual. Scrapes reshade.me and, when a newer version than the pinned fallback ships, opens a PR
  bumping that pin. If the scrape itself breaks, the run goes **red** — that is the point, since
  otherwise a broken scrape only shows up mid-release. A `pretend_version` dispatch input exercises
  the PR path on demand. Note that PRs it opens carry no Build check: PRs created with the workflow
  token don't start other workflows.
  This job needs **Settings → Actions → General → "Allow GitHub Actions to create and approve pull
  requests"** enabled (repo API field `can_approve_pull_request_reviews`). It is off by default, and
  without it the run gets as far as pushing the branch and then fails with
  `GitHub Actions is not permitted to create or approve pull requests`.

Both build workflows assemble the same drag-and-drop bundle for end users. After compiling the
addon they:

1. **Resolve the latest ReShade** via the shared composite action
   [`.github/actions/fetch-reshade`](.github/actions/fetch-reshade/action.yml) - scrape
   `https://reshade.me/` for `ReShade_Setup_<version>_Addon.exe`. If scraping fails (site change /
   rate-limit), it falls back to the pinned version in that action's `fallback` input default
   (currently `6.8.0`) so the build never breaks. **That default is the only copy of the pin** —
   the fetch used to be duplicated in both workflows, which is what the composite action exists to
   prevent. It is unrelated to the SDK pin in `deps/reshade` (6.1.0 / API 11), which must stay put.
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

### Labels

Issue/PR labels are declared in [`.github/labels.yml`](.github/labels.yml) and applied with
`./.github/sync-labels.sh`. The script creates or updates every label in the manifest, then reports
any label that exists on the repo but isn't declared — pass `--prune` to actually delete those. The
dry-run default is deliberate: deleting a label also strips it from every issue and PR that carried
it.

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
> with an API version ≤ its own, bundling a newer ReShade never breaks this addon (API 11).

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
