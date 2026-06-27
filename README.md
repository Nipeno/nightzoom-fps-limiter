# NightZoom FPS Limiter

Locks your game to a smooth **60 FPS**. A simple add-on for [ReShade](https://reshade.me) with
one button - turn it on to cap, turn it off to unlock.

> Made by **Nipeno** · 💬 [Join the Discord](https://discord.gg/nightzoom)

## ⚠️ Please read first

This add-on is made for the **NightZoom** server and is probably allowed there (waiting for BJJ to respond to my DM). On **other servers, use it at your own risk** - every server sets its own rules, and some
anticheats may flag add-ons that load into the game. If you're unsure about a server, ask its
staff first, or just don't use it there.

Also: this needs the **add-on-enabled** build of ReShade - and the download includes it, so
you're covered even if you've never used ReShade. Graphics packs like NVE and QuantV already
include ReShade too.

## How to install

Download **`NZ-FPS-Limiter…zip`** from the
[**Releases page**](https://github.com/Nipeno/nightzoom-fps-limiter/releases/latest), extract it,
and open the included **`INSTALL.html`** - it walks you through the whole setup, including the
one-time FiveM "ReShade was blocked" fix. The zip bundles ReShade, so it's all you need.

## How to use

1. In-game, open the ReShade menu. The key is **Home** by default - but some graphics packs set
   their own: **NVE uses `Insert`**. If one doesn't work, try the other. (You can see/change the
   key in ReShade's Settings.) The very first time, ReShade shows a welcome/tutorial popup - click
   **Continue** or **Skip Tutorial** to get past it.
2. Find the **NightZoom FPS Limiter** window.
3. Tick **Limit to 60 FPS** to cap your frame rate. Untick it to unlock.

Your choice is remembered - it stays the same next time you launch the game.

## Why 60 FPS?

GTA's physics is tied to your frame rate - higher FPS can change how a car behaves. On a racing
server that's unfair: people on stronger PCs would get an edge. Capping everyone to 60 FPS keeps
the physics consistent so nobody has an advantage.

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

Free and open source under **GPLv3** - see [LICENSE](LICENSE). You can use, study, and modify it,
but any shared version must stay open source too.
