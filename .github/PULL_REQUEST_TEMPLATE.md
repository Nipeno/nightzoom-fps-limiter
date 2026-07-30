<!-- Thanks for contributing to NZ FPS Limiter. Keep PRs focused. -->

## What & why
<!-- Short summary of the change and the reason for it. -->

Closes #

## Type
- [ ] Bug fix
- [ ] Addon behavior / pacing
- [ ] CI / release tooling
- [ ] Installer (Tauri GUI — `installer/tauri-gui` branch, not on `main`)
- [ ] Docs only

## Checklist
- [ ] **60 FPS cap stays hardcoded** (`kTargetFps = 60.0`) — no slider/presets/config knob added
- [ ] License stays **GPLv3**; no new deps / telemetry
- [ ] If touching ReShade/ImGui headers: version pins kept in sync (SDK v6.7.3 = API 18, ImGui docking `19250`)
- [ ] Builds clean in GitHub Actions (`windows-2022`) — macOS clang LSP noise ignored
- [ ] Docs updated if user-facing behavior changed (README / BUILDING / packaging install txt)

## Testing
<!-- How was this verified? Real FiveM run? CI artifact? Note what's still UNtested. -->
