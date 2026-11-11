<div align="center">

<img src="site/public/assets/droplet-core.png" width="624" alt="Droplet 2.9.0 Core panel: Shield, Mace, Potion, Breach, Auto Use, and Dash with round power buttons.">

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
| Windows 10/11 x64 | [Portable ZIP](https://fuvji.github.io/droplet/downloads/droplet-windows-x64.zip) | 289,296 B | **Available** |
| Windows 10/11 x64 | [Signed MSIX](https://fuvji.github.io/droplet/downloads/droplet-windows-x64.msix) | 306,082 B | **Available** |
| Linux | — | — | Port not started; the core is Win32/GDI+ |
| macOS | — | — | Port not started; no Cocoa/Quartz layer |

Verify with [SHA256SUMS.txt](https://fuvji.github.io/droplet/downloads/SHA256SUMS.txt).
The portable exe is unsigned; the MSIX uses the project development
certificate. No Linux `.deb`/AppImage/`.tar.gz` or macOS `.dmg`/`.zip`/`.img`
is published because those builds do not exist.

## What is in 2.9.0

- **Round power controls** on every module card, with distinct on, off, and
  hover states and no background plate.
- **Original item-style line glyphs** for all modules: pearl, shield, spear,
  mace, potion, crystal, anchor, totem, grass block, sword, player head,
  repeater, and chest. They are drawn from GDI+ primitives, not copied Mojang
  textures, and every icon is transparent.
- **Compact Dash** card without the redundant `Clicks 1x` row.
- **Sticky hide**: the tray icon is gone, and a manual hide is no longer undone
  by an alt-tab or focus change. `Ctrl + Right Shift` shows or hides the panel.
- **Instant close**: the close button stops the watchers and exits.
- **One-second self-destruct hold** with a visible progress percentage.
- **Anchor 37–43 ms** default and slider window, with the 50 ms Minecraft tick
  floor stated on the card and still applied by the macro.
- **Same-frame input** for Dash, Shield, and Pearl: slot selection and click
  share one `SendInput` batch. Ordinary input only; the focus gate remains.
- **Faster hook rejection**: unbound keys are rejected by the prebuilt trigger
  table before either settings `Arc` is cloned.
- **Downloads and a release ledger** on the website, plus a tag-driven Windows
  release workflow.

## GUI

![Droplet 2.9.0 System panel](site/public/assets/droplet-system.png)

The panel is a native 624×408 acrylic surface. The painted hairline and the
Win32 window region share one 13 px radius. Painting is event-driven; an idle
panel does not schedule animation frames. Binding and slot chips borrow their
labels instead of allocating a `String` per chip per frame.

## Modules

| Module | Sequence | Mode |
| --- | --- | --- |
| Anchor | Place, charge once, switch off glowstone, detonate | one-shot |
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
- Portable executable: **601,600 bytes**,
  `3c40feceec36e830cbf571614372b41f68c96bc2db417bf1c3d2fa8822db5dd9`.
- Signed MSIX: **306,082 bytes**,
  `c569446ec120c8548b009e4714d3e6d3b3df5001fa4a51ae2bf60f6c2f84a215`.
- Package version: **2.9.0.0**, signature **Valid**, installed status **Ok**.
- Four native 624×408 captures in `site/public/assets/`.
- No live Minecraft client is available here, so hit registration and the new
  same-frame input ordering remain unverified in game.

## Deliberate boundaries

Droplet is local and visible by design.

- No game-memory reads or writes.
- No network calls, telemetry, analytics, or silent auto-updater.
- No streamproof/capture-evasion mode, process hiding, scan evasion, log
  erasure, anti-cheat bypass, or input-device disguise.
- No zero-latency, “quantum speed”, or “100% no restriction” claim.
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
