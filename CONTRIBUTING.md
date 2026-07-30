# Contributing

Thanks for taking a look. This is a small, deliberately narrow project — a single-file ReShade
addon that caps the frame rate to 60. That narrowness is the design, not an oversight, so please
read the ground rules before opening a PR.

## Ground rules

**The 60 FPS cap is hardcoded and stays that way.** No slider, no presets, no config knob. GTA's
physics is frame-rate dependent, so an adjustable cap would defeat the point of everyone racing on
the same footing. PRs adding one will be declined regardless of how well they're written.

**GPLv3 only, and no new dependencies.** Everything the addon needs is either vendored under
`deps/` (headers only) or a Windows system library. No package manager, no telemetry, no network
calls.

**Version pins move together.** The ReShade SDK headers and the Dear ImGui headers must match:
SDK **v6.7.3** = addon **API 18**, paired with the **docking branch** of ImGui at
`IMGUI_VERSION_NUM 19250`. `reshade_overlay.hpp` has an `#error` that fires if the ImGui version
doesn't match exactly, and the release-branch ImGui headers won't work at all because the overlay
uses docking-only types. [BUILDING.md](BUILDING.md) has the refresh commands.

## Building and testing

Build steps are in [BUILDING.md](BUILDING.md). Windows and MSVC only.

Two things worth knowing before you spend time debugging:

- **On macOS or Linux, your editor's clang diagnostics will light up** — missing `imgui.h`,
  undeclared `reshade`, `std::chrono::duration` arity errors. That's clang lacking the Windows SDK
  and the CMake include paths, not broken code. It builds clean under MSVC. Don't "fix" it.
- **There is no test suite.** Verification is a real FiveM run: does ReShade load the addon, does
  the overlay appear, does the frame rate actually sit at 60. If you can't do that, say so in the
  PR and note what's untested — that's genuinely useful information, not a failure.

CI builds every push and pull request on `windows-2022` and attaches the assembled bundle as an
artifact, so you can download and test a build even without a local Windows toolchain.

## Pull requests

Keep them focused — one change per PR. The template will ask you to confirm the ground rules above
and to describe how you tested.

Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)
(`fix:`, `feat:`, `docs:`, `ci:`, `perf:`, `refactor:`). Branch off `main`.

Note that `installer/tauri-gui` is a separate in-flight branch carrying a Tauri installer GUI. It
isn't on `main`; if your change targets the installer, base it there.

## Releases

Maintainer only. Update `CHANGELOG.md`, then push a tag:

```sh
git tag vX.Y.Z && git push origin vX.Y.Z
```

`.github/workflows/release.yml` builds the addon at that version, bundles the latest official
ReShade, and attaches `NZ-FPS-Limiter_vX.Y.Z.zip` to the GitHub Release. The tag is the single
source of truth for the version — it flows into the DLL's version resource and the overlay label.

## Reporting bugs

Use the [issue forms](https://github.com/Nipeno/nightzoom-fps-limiter/issues/new/choose). Bugs and
questions both go through GitHub Issues so answers stay searchable. For security problems, see
[SECURITY.md](SECURITY.md) instead.

## Code of conduct

Participation is covered by our [Code of Conduct](CODE_OF_CONDUCT.md).
