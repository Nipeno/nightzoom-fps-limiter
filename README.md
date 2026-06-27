# NightZoom FPS Limiter

Locks your game to a smooth **60 FPS**. A simple add-on for [ReShade](https://reshade.me) with
one button — turn it on to cap, turn it off to unlock.

> Made by **Nipeno** · 💬 [Join the Discord](https://discord.gg/nightzoom)

## ⚠️ Please read first

This add-on is made for the **NightZoom** server and is probably allowed there (waiting for BJJ to respond to my DM). On **other servers, use it at your own risk** — every server sets its own rules, and some
anticheats may flag add-ons that load into the game. If you're unsure about a server, ask its
staff first, or just don't use it there.

Also: this needs the **add-on-enabled** build of ReShade — and the download includes it, so
you're covered even if you've never used ReShade. Graphics packs like NVE and QuantV already
include ReShade too.

## How to install (FiveM)

Everything goes in your FiveM **plugins** folder. Open it with **Win+R** →
paste `%localappdata%\FiveM\FiveM.app\plugins` → Enter. (Installed FiveM somewhere custom? That
path won't open — instead go into `FiveM.app\plugins` inside your own FiveM folder, the one with
`FiveM.exe`.)

Download **`NZ-FPS-Limiter…zip`** from the
[**Releases page**](https://github.com/Nipeno/nightzoom-fps-limiter/releases/latest) — it
includes ReShade, so it's all you need.

1. Extract **`NZ-FPS-Limiter.addon64`** into the `plugins` folder.
2. **Already have ReShade?** (NVE, QuantV, another graphics pack — the ReShade menu already
   opens in game.) That's it: **don't** copy the bundled `dxgi.dll` (two copies conflict).
   Start FiveM and jump to [How to use](#how-to-use).
3. **No ReShade yet?** Also extract the bundled **`dxgi.dll`** into `plugins`, then double-click
   **`Enable-ReShade.bat`** (same folder) to allow ReShade in FiveM. Start FiveM.

The included `INSTALL.txt` has the full step-by-step.

## How to use

1. In-game, open the ReShade menu. The key is **Home** by default — but some graphics packs set
   their own: **NVE uses `Insert`**. If one doesn't work, try the other. (You can see/change the
   key in ReShade's Settings.) The very first time, ReShade shows a welcome/tutorial popup — click
   **Continue** or **Skip Tutorial** to get past it.
2. Find the **NightZoom FPS Limiter** window.
3. Tick **Limit to 60 FPS** to cap your frame rate. Untick it to unlock.

Your choice is remembered — it stays the same next time you launch the game.

## Why 60 FPS?

GTA's physics is tied to your frame rate — higher FPS can change how a car behaves. On a racing
server that's unfair: people on stronger PCs would get an edge. Capping everyone to 60 FPS keeps
the physics consistent so nobody has an advantage.

## First launch: "ReShade was blocked" (one-time fix)

FiveM blocks modern ReShade by default, so you allow it once. **Two ways:**

### Easy (recommended) — run the included `.bat`

After the files are in your `plugins` folder, double-click **`Enable-ReShade.bat`** (it's in the
zip). It figures out the right per-PC value and writes it to `CitizenFX.ini` automatically — then
start FiveM.

> If Windows shows a blue **"Windows protected your PC"** box, click **More info → Run anyway**.
> The script is open source ([`packaging/Enable-ReShade.bat`](packaging/Enable-ReShade.bat)) — read
> it first if you like.

### Manual (if you prefer, or the .bat can't find FiveM)

The first time, FiveM's console shows red text like:

```
Blocked load of ReShade version 5 or higher ...
add the following section to <path>\CitizenFX.ini:
   [Addons]
   ReShade5=ID:xxxxxxxx acknowledged that ReShade 5.x has a bug that will lead to game crashes
```

1. Fully close FiveM.
2. Open **`CitizenFX.ini`** — it's in your `FiveM.app` folder; the console shows the exact path.
3. Add the two lines **exactly as your own console showed them**:
   ```ini
   [Addons]
   ReShade5=ID:xxxxxxxx acknowledged that ReShade 5.x has a bug that will lead to game crashes
   ```
   > ⚠️ The ID (`xxxxxxxx`) is **unique to your PC** — copy it from your own FiveM console, don't
   > use the example above.
4. Save and start FiveM again.

If you already use ReShade with FiveM (e.g. via NVE / QuantV), you've probably done this already
and can skip it.

## NightZoom FPS Limiter window doesn't show up?

If you enabled ReShade but the **NightZoom FPS Limiter** window still isn't there, your existing
ReShade is probably **too old** to load it (your `ReShade.log` will say something like
`requested API version … is not supported`). This happens with some older graphics-pack bundles.

**Fix:** copy the bundled **`dxgi.dll`** (from the `NZ-FPS-Limiter…zip` you downloaded) into your
`plugins` folder, replacing the old one. It's the latest official ReShade — it still runs NVE /
QuantV fine (newer ReShade loads older add-ons), it just also supports NightZoom FPS Limiter.

## How to uninstall

1. Close FiveM.
2. Open the plugins folder: `%localappdata%\FiveM\FiveM.app\plugins`.
3. Delete **`NZ-FPS-Limiter.addon64`**.
4. Only if you installed the bundled `dxgi.dll` just for this and want ReShade gone too,
   also delete `dxgi.dll`. If your graphics pack (NVE / QuantV) put ReShade there, **leave
   `dxgi.dll` alone**.

## Is it safe? What does it do?

This add-on is **open source**, so anyone can read exactly what it does. It only:

- caps your frame rate to 60 FPS,
- remembers your on/off choice,
- shows the logo, Discord link, and a link to the code.

It has **no ads, no tracking, no internet connection** (the only time it opens your browser is
when *you* click the Discord or GitHub buttons), and it doesn't touch your game's files.

The full source code is right here in this repo, and there's a **View Source on GitHub** button
inside the add-on too.

## Community

Made for **NightZoom**. Come say hi, get help, or report a bug:

👉 **<https://discord.gg/nightzoom>**

## For developers

Want to build it yourself or see how it works? See **[BUILDING.md](BUILDING.md)**.

## License

Free and open source under **GPLv3** — see [LICENSE](LICENSE). You can use, study, and modify it,
but any shared version must stay open source too.
