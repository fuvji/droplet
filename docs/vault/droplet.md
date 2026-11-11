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

- 2.10.1 portable exe: 595,968 bytes. Single artifact; no MSIX/AppX or ZIP.
- Tests: 24 / 24.
- Trigger routing: 0.45 ns indexed vs 202.39 ns scanned (445×, routing only).
- Anchor defaults: 47/53 ms, Min 47-97, Max 53-103. Hydrogen A6 order;
  Air loop repeats while held.
- Game detection checks the owning process (`javaw.exe`, `java.exe`, or
  `Minecraft.Windows.exe`) and rejects browser, chat, editor, and launcher
  titles. No live game process was available for the positive path.
- No network, telemetry, game-memory access, capture evasion, anti-cheat
  bypass, or input disguise.

## Design

One 8 px radius. Event-driven paint. Borderless module cards with the original
28×16 sliding knob switch and no module glyphs. Small original line icons remain
in the navigation rail only. Graphite surface, warm white text, no painted outer
rim, and no top accent line. Min/Max sliders are independent.

## Current work

- Windows is the only port. Linux/macOS need a real platform layer.
- Live Minecraft hit registration is unverified.
- The only Windows artifact is the direct `droplet.exe`; no ZIP/MSIX is
  published. No fake Linux/macOS artifacts.
