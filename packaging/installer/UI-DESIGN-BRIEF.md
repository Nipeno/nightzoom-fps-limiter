# NightZoom FPS Limiter — Installer UI Design Brief

> Handoff doc for the agent building the installer **frontend**. Backend logic is
> separate; this describes only what the UI must show, collect, and transition through.
> Goal: a polished, branded one-window installer (nohesi-style) that a user double-clicks
> from anywhere (Downloads, Desktop) and clicks through to a working 60 FPS cap in FiveM.

## What this installs (context)
A ReShade addon that hard-caps FiveM to exactly 60 FPS. The installer must: find the
user's FiveM install, copy two files into it, and write one line to a config file so
FiveM allows ReShade. That's it. No accounts, no internet account, no settings.

## Brand
- Name: **NightZoom FPS Limiter**. Tagline: *"Locks your game to a smooth 60 FPS."*
- Logo: 512×512 PNG (NZ logo) — ask for the asset, it exists in the repo (`src/logo_data.h`).
- Vibe: dark theme, racing/night, clean. Discord: `discord.gg/nightzoom`.
  GitHub: `github.com/Nipeno/nightzoom-fps-limiter`.
- One primary action per screen. Big, obvious. Non-technical audience (gamers).

---

## Screens (the whole flow)

### 1. Welcome / Start
- Logo, name, tagline.
- One line of what it does: "Installs the 60 FPS limiter into FiveM."
- Primary button: **Install**.
- Secondary (small): Discord link, GitHub link.
- Optional: show detected FiveM path here once found (can auto-detect on open).

### 2. Detecting FiveM
- Spinner / indeterminate bar: "Finding your FiveM install…"
- On success → show the found path with a ✓, auto-advance or enable Install.
- On failure → **Not-found state**: "Couldn't find FiveM automatically." +
  a **Browse…** button (folder picker) so the user points at their `FiveM.app` folder.
  Re-validate after they pick.

### 3. (Optional) Existing ReShade prompt
- Only appears if the user already has ReShade (graphics pack like NVE / QuantV).
- Message: "You already have ReShade installed."
- Two choices: **Keep mine** (recommended) / **Replace with bundled**.
- Needs a one-line explainer under each. Default highlighted = Keep mine.
- If this case doesn't occur, screen is skipped entirely.

### 4. Installing (progress)
- Progress bar with sub-step labels as they happen:
  - "Downloading files…" (only if download build — show %/MB) **or** skip if bundled
  - "Copying into FiveM…"
  - "Enabling ReShade…"
- No user input here. Just progress + current label.

### 5. Done / Success
- Big ✓ + "All set!"
- Instructions block (must be readable, this is the payoff):
  > Launch FiveM. Press **Home** (or **Insert** if you use NVE) to open the menu,
  > then tick **"Limit to 60 FPS."**
- Buttons: **Close**, **Join Discord**.

### 6. Error panel (shared, reused by any step)
- Title, plain-English message, and the right recovery action for the case:
  - **FiveM not found** → Browse button (→ back to step 2).
  - **File locked / permission denied** (FiveM is open, or needs admin) →
    "Close FiveM and retry" + **Retry** + **Run as administrator** (relaunch elevated).
  - **Download failed** (no internet) → **Retry**.
- Always offer a way out: Retry, Browse, or Close. Never a dead end.

---

## State machine
```
        ┌──────────────────────────── (Browse picks valid path) ──────────────┐
        │                                                                      │
IDLE → DETECTING ──found──→ [existing ReShade?] ──→ INSTALLING ──→ DONE        │
        │  └──not found──→ ERROR:not-found ──Browse──────────────────────────-┘
        │                          (prompt keep/replace if existing)
        └ any step fail ──→ ERROR(retry / browse / admin) ──retry──→ back to that step
```

---

## Data the UI shows (provided by backend after "detect")
The UI does not compute any of this — it receives it and displays/uses it:
- `fiveMFound` (bool) — drives detect success vs not-found.
- `fiveMPath` (string) — show on welcome/detect screens.
- `existingReShade` (bool) — whether to show screen 3.
- `computerName` + `reshadeId` (strings) — optional, can show as a small detail
  ("PC ID: 46750aa6"); not required visually.
- `mode` — `download` or `bundled` — tells the UI whether screen 4 shows a download
  progress bar or jumps straight to copying.

## Input the UI must collect (passed back to backend on "install")
- `manualFiveMPath` — only if auto-detect failed and the user used Browse.
- `replaceDxgi` — only if `existingReShade` was true: keep (false) or replace (true).
- An **elevation** path: if backend reports a locked/permission error, the UI must be
  able to relaunch the installer as administrator (and resume).

## Progress events the UI listens to during "install"
Backend streams step events; UI maps each to a label + advances the bar:
`download` (has percent), `copy`, `enable`, `done`. Each event carries:
`status` (start / ok / fail), `message` (display text), and `percent` (download only).
On any `fail` → show Error panel with the matching recovery action above.

---

## Constraints / notes for the UI agent
- **No heavy deps if avoidable** — project is zero-dependency. If you choose a stack that
  needs a runtime/build step (C# .NET, Electron, Inno Setup), flag it; PowerShell+WPF
  needs nothing extra on Windows 10/11.
- **Unsigned binary** → Windows SmartScreen will warn ("More info → Run anyway"). The
  success/welcome copy or a first-run note should reassure non-technical users this is
  expected (open-source, no cert). Don't try to hide it.
- **Runs from anywhere** — Downloads, Desktop, wherever. Don't assume a working directory.
- **Windows x64 only.** No macOS/Linux UI needed.
- **Audience = gamers, not devs.** Plain words. "FiveM," "60 FPS," "menu" — not "addon,"
  "DLL," "config section."
- Keep it to **one window**, no multi-page wizard sprawl unless you choose the Inno route.

## Deliverable expected from the UI agent
A branded installer frontend implementing screens 1–6 and the state machine above, wired
to the backend's `detect` (returns the data block) and `install` (streams progress events)
actions. The UI owns: layout, theme, animation, copy, the Browse picker, the keep/replace
prompt, the admin-relaunch button, and mapping backend events → screens.
