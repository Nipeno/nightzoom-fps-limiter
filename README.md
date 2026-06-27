# NightZoom FPS Limiter

Locks your game to a smooth **60 FPS**. A simple add-on for [ReShade](https://reshade.me) with
one button — turn it on to cap, turn it off to unlock.

> Made by **Nipeno** · 💬 [Join the Discord](https://discord.gg/nightzoom)

## ⚠️ Please read first

This add-on is made for the **NightZoom** server and is probably allowed there (waiting for BJJ to respond to my DM). On **other servers, use it at your own risk** — every server sets its own rules, and some
anticheats may flag add-ons that load into the game. If you're unsure about a server, ask its
staff first, or just don't use it there.

Also: this needs the **add-on-enabled** build of ReShade. Graphics packs like NVE and QuantV
already include it; if you don't have ReShade at all, use the all-in-one bundle in
[Option B](#option-b--you-dont-have-reshade-yet-easiest-all-in-one) below.

## How to install (FiveM)

Everything goes in your FiveM **plugins** folder. Open it with **Win+R** →
paste `%localappdata%\FiveM\FiveM.app\plugins` → Enter.

Pick the option that matches you. Both downloads are on the
[**Releases page**](https://github.com/Nipeno/nightzoom-fps-limiter/releases/latest).

### Option A — You already have ReShade (NVE, QuantV, other graphics pack)

1. Download **`NightZoom.addon64`**.
2. Drop it into the `plugins` folder.
3. Start FiveM.

### Option B — You don't have ReShade yet (easiest, all-in-one)

1. Download **`NightZoom-FiveM-Bundle…zip`**.
2. Extract everything into the `plugins` folder.
3. Start FiveM.

> The bundle includes `dxgi.dll`, which is just the official ReShade. If you already have
> ReShade from a graphics pack, **delete the bundled `dxgi.dll`** (use Option A instead) — two
> copies will conflict.

## How to use

1. In-game, open the ReShade menu (press **Home** by default).
2. Find the **NightZoom FPS Limiter** window.
3. Tick **Limit to 60 FPS** to cap your frame rate. Untick it to unlock.

Your choice is remembered — it stays the same next time you launch the game.

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
