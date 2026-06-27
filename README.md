# NightZoom FPS Limiter

Locks your game to a smooth **60 FPS**. A simple add-on for [ReShade](https://reshade.me) with
one button — turn it on to cap, turn it off to unlock.

> Made by **Nipeno** · 💬 [Join the Discord](https://discord.gg/nightzoom)

## ⚠️ Please read first

This add-on is made for the **NightZoom** server and is allowed there (cleared with the server
owner). On **other servers, use it at your own risk** — every server sets its own rules, and some
anticheats may flag add-ons that load into the game. If you're unsure about a server, ask its
staff first, or just don't use it there.

Also: you need the version of ReShade that supports add-ons (see step 1 below).

## How to install

1. **Install ReShade with add-on support.** Download it from <https://reshade.me>, run the
   installer, pick your game, and make sure **"add-on support"** is ticked during setup.
2. **Download** `NightZoom.addon64` from the [**Releases page**](https://github.com/Nipeno/nightzoom-fps-limiter/releases/latest).
3. **Drop the file into your game's folder** — the same folder where ReShade was installed
   (it's next to a file like `dxgi.dll` or `d3d11.dll`).
4. **Start the game.**

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
