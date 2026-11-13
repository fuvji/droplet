# droplet

Droplet is a local, visible Windows macro suite for Minecraft Java, branded
**Minecraft Zero Macro**. It is a zero-dependency Rust binary with a native
624×408 GDI+ panel.

Canonical site: <https://fuvji.github.io/droplet/>

## Graph

- [[performance]] — routing benchmark, tick floor, honest limits
- [[release]] — version ledger, hashes, downloads
- [[website]] — static site, downloads, 3D previews
- [[AGENTS]] — universal project rules

## Core facts

- 2.10.9 portable exe: 545,280 bytes. Single artifact; no MSIX/AppX or ZIP.
- Tests: 31 / 31.
- Auto Crystal has a Return slot (`v5S3`, default `1`). A hold of at least
  160 ms or a toggle-off restores it once; a short tap does not. The restore
  is a double tap of the Return binding.
- `WM_DPICHANGED` updates scale, physical window size, rounded clip, hover,
  and hit testing together. Pointer coordinates stay physical, fixing the
  125%/150% row and tab offset.
- The self-destruct waiter uses hidden `ShellExecuteW` instead of
  `std::process::Command`; the link uses bundled `rust-lld` + ICF. The 2.10.8
  size pass is 55,808 B smaller than 2.10.7 and keeps `opt-level=3`.
- Sprint settings are read only when a pulse is due; crosshair settings only
  on a magenta hit. Poll cadence and bindings are unchanged. Idle sprint and
  crosshair watchers skip the settings snapshot and lock; the sprint loop
  parks at 100 ms when inactive and unparks on activation.
- Trigger routing: 0.45 ns indexed vs 202.39 ns scanned (445×, routing only).
- Anchor defaults: 47/53 ms, Min 47-97, Max 53-103. Hydrogen A6 order;
  Air loop repeats while held.
- Game detection checks the owning process (`javaw.exe`, `java.exe`, or
  `Minecraft.Windows.exe`) and rejects browser, chat, editor, and launcher
  titles. No live game process was available for the positive path.
- No network, telemetry, game-memory access, capture evasion, anti-cheat
  bypass, or input disguise.

## Design

One 8 px radius. Corners prefer DWM's antialiased compositor clip; the transparent
quarter-rounds are click-through and the GDI region is a fallback only. The
presence watcher reuses a verified foreground game HWND, and its focus-only gate
is cached in an atomic. The favicon and site marks are one transparent outlined
droplet with no plate, border, background, or shadow. Event-driven paint.
Borderless module cards with the original 28×16 sliding knob switch and no
module glyphs. Small original line icons remain in the navigation rail only.
Graphite surface, warm white text, no painted outer rim, and no top accent line.
Min/Max sliders are independent.

## Current work

- Auto Crystal has a Return slot (`v5S3`, default `1`). A hold of at least
  160 ms restores it after release and a latched toggle restores it when
  switched off; a short tap intentionally leaves the selected slot alone.
- The panel handles `WM_DPICHANGED`: scale, window size, rounded clip, pointer
  mapping, and hit testing are updated together when it moves between monitors.
  Pointer coordinates stay physical, matching the scaled paint coordinates, so
  hover and clicks no longer shift on 125% or 150% displays.
- Windows is the only port. Linux/macOS need a real platform layer.
- Live Minecraft hit registration is unverified.
- The only Windows artifact is the direct `droplet.exe`; no ZIP/MSIX is
  published. No fake Linux/macOS artifacts.
