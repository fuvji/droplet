<div align="center">

<img src="site/public/assets/droplet-core.png" width="624" alt="Droplet 2.9.2 Core panel: Shield, Mace, Potion, Breach, Auto Use, and Dash with sliding knob switches.">

# droplet

**Minecraft Zero Macro** — a local, visible Windows input macro suite.

[Website](https://fuvji.github.io/droplet/) · [Downloads](https://fuvji.github.io/droplet/#download) · [Releases](https://fuvji.github.io/droplet/releases.html) · [Benchmarks](BENCHMARKS.md) · [Source](https://github.com/fuvji/droplet)

</div>

Droplet runs exact keyboard-and-mouse sequences for Minecraft Java, entirely
on your machine. It reads no game memory, makes no network calls, and ships no
telemetry. The panel and every action remain visible to ordinary screen
capture.

## Download

| Platform | Package | Size | Status |
| --- | --- | ---: | --- |
| Windows 10/11 x64 | [Portable ZIP](https://fuvji.github.io/droplet/downloads/droplet-windows-x64.zip) | 285,751 B | **Available** |
| Windows 10/11 x64 | [Signed MSIX](https://fuvji.github.io/droplet/downloads/droplet-windows-x64.msix) | 301,609 B | **Available** |
| Linux | — | — | Port not started; the core is Win32/GDI+ |
| macOS | — | — | Port not started; no Cocoa/Quartz layer |

Verify with [SHA256SUMS.txt](https://fuvji.github.io/droplet/downloads/SHA256SUMS.txt).
The portable exe is unsigned; the MSIX uses the project development
certificate. No Linux `.deb`/AppImage/`.tar.gz` or macOS `.dmg`/`.zip`/`.img`
is published because those builds do not exist.

## What is in 2.9.2

- **Independent Min/Max sliders**. Editing one no longer rewrites the other.
  Each field is clamped only to its own domain; an inverted pair is sorted when
  the macro reads it.
- **The old knob switch is back**. Every module uses the original 28×16 track
  with a plain 5 px-radius knob. There is no ON/OFF text and no glyph; position
  carries the state.
- **Wider Anchor timing**: Min **37–97 ms**, Max **43–103 ms**, defaults **37/43**.
  Each Anchor slot now gets exactly one configured wait before its action. The
  old 50 ms floor and the three hardcoded 50 ms slot sleeps are gone.
- **Cleaner edge**: the painted outer hairline is removed. DWM's border was
  already suppressed, so the panel no longer has a double rim at the corners.
- **Danger zone restored**: the action is labelled **Self-destruct** again and
  keeps its one-second hold, data purge, and portable-executable recycle.
- **Smaller binary**: the 2.9.2 portable executable is 591,872 bytes, 512 bytes
  smaller than 2.9.1. The signed MSIX is 301,609 bytes.
- **Compact Dash** card without the redundant `Clicks 1x` row.
- **Sticky hide**: the tray icon is gone, and a manual hide is no longer undone
  by an alt-tab or focus change. `Ctrl + Right Shift` shows or hides the panel.
- **Same-frame input** for Dash, Shield, and Pearl: slot selection and click
  share one `SendInput` batch. Ordinary input only; the focus gate remains.
- **Faster hook rejection**: unbound keys are rejected by the prebuilt trigger
  table before either settings `Arc` is cloned.
- **No unrestricted-speed mode**. Other modules keep their configured delays;
  the 50 ms game tick and Windows scheduling remain the real boundary.

## GUI

![Droplet 2.9.2 System panel](site/public/assets/droplet-system.png)

The panel is a native 624×408 acrylic surface. DWM supplies the rounded edge
and the app paints edge-to-edge, with no outer stroke. Painting is event-driven;
an idle panel does not schedule animation frames. Binding and slot chips borrow
their labels instead of allocating a `String` per chip per frame.

## Modules

| Module | Sequence | Mode |
| --- | --- | --- |
| Anchor | Place, charge once, switch off glowstone, detonate; 37–97 / 43–103 ms | one-shot |
| Auto Crystal | Obsidian, then crystal | one-shot / toggle |
| Mace | Settled swap and hit | one-shot / toggle |
| Shield | Axe select and hit in one batch; optional held stun | one-shot / toggle |
| Dash | Spear select, hit, restore; 50/100/150 ms cooldown | one-shot / toggle |
| Breach | Mace swap and return | one-shot / toggle |
| Potion | Two-slot drink and return | one-shot / toggle |
| Pearl | Pearl select and throw in one batch, then catch | one-shot / toggle |
| Hover Totem | Totem swap watcher | watcher |
| Fast Use | Bounded held right-click cadence | one-shot / toggle |
| Auto Use | Use an item, swap back | one-shot / toggle |
| W-Tap | S-Tap default, 132–148 ms | per-click |
| Sprint | One tap every 100 ms while physical W is held | bounded watcher |
| Ghost Click | Small local click offset, 1–12 px | per-click |
| Hover inventory | Peek without leaving combat | one-shot |

## Performance, without fairy tales

| Path | Time per lookup | Method |
| --- | ---: | --- |
| Old binding scan | 202.39 ns | Hash and scan up to 24 binding strings |
| Pre-indexed route | 0.45 ns | One array read from a prebuilt table |
| Routing reduction | 445× | Same event, process, and release profile |

This is a **routing microbenchmark**, not end-to-end input latency, game-tick
latency, or hit registration. Minecraft observes input on a 50 ms tick; Windows
and the device path add their own scheduling. “Zero” in the product name is a
name, not a latency claim. Details and reproduction:
[BENCHMARKS.md](BENCHMARKS.md).

## Verify

- `cargo test --release`: **19 / 19**.
- Portable executable: **591,872 bytes**,
  `2d4a915ca68b3c4ef944fe208b593f3c816f55b8b605a478d23a5cfd5d70cc46`.
- Signed MSIX: **301,609 bytes**,
  `94d47fd44f092b571ade122b2d020011da36b41cd185517895f28133040e6c87`.
- Portable ZIP: **285,751 bytes**,
  `928f816eb2dc4fa028273a9af9f8236e18a2e6c4db78ee612bd6a8d9fb2f5841`.
- Package version: **2.9.2.0**, signature **Valid**, installed status **Ok**.
- Four native 624×408 captures in `site/public/assets/`.
- No live Minecraft client is available here, so hit registration and the new
  same-frame input ordering remain unverified in game.

## Deliberate boundaries

Droplet is local and visible by design.

- No game-memory reads or writes.
- No network calls, telemetry, analytics, or silent auto-updater.
- No streamproof/capture-evasion mode, process hiding, scan evasion, log
  erasure, anti-cheat bypass, or input-device disguise.
- No zero-latency, “quantum speed”, “100% raw speed”, or “no restriction”
  claim. Droplet removes its own avoidable routing and redundant sleeps; it
  cannot remove the game loop, device path, or OS scheduler.
- The focus and master gates stop every macro when they are not satisfied.
- The portable exe is unsigned; SmartScreen may warn.
- Windows only for now. Linux and macOS require a real platform port, not a
  renamed archive.

## Build

```bash
rustup target add x86_64-pc-windows-gnu
cargo test --release
cargo build --release
```

Package and install:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/build-msix.ps1
```

Outputs are lowercase: `apps/droplet/droplet.exe` and
`apps/droplet/droplet.msix`. See [DEVELOPING.md](DEVELOPING.md).

## Repository map

| Path | Contents |
| --- | --- |
| `src/` | Win32 shell, GDI+ paint, input hooks, macro modules, settings |
| `site/public/` | Static product site, release ledger, downloads, captures |
| `releases/` | Detailed per-version release notes |
| `docs/` | Combat-lab notes, AI boundaries, portable Obsidian vault |
| `.github/workflows/` | Tag-driven Windows release pipeline |
| `scripts/`, `tools/` | MSIX packaging, signing, captures, local preview |

The site uses one local script for OS detection on the download page. It sends
nothing and stores nothing. There are no third-party scripts, fonts, analytics,
or ad clients.

## Founder

Designed and built by [@epufk](https://github.com/epufk).

Not an official Minecraft product. Not approved by or associated with Mojang or
Microsoft. Minecraft is a trademark of Mojang Synergies AB.

## License

Proprietary. The source is public for inspection; public availability is not an
open-source grant. See [LICENSE](LICENSE).
