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

- 2.9.1 portable exe: 592,384 bytes (9,216 bytes smaller than 2.9.0).
- Signed MSIX: 301,774 bytes.
- Tests: 19 / 19.
- Trigger routing: 0.45 ns indexed vs 202.39 ns scanned (445×, routing only).
- No network, telemetry, game-memory access, capture evasion, anti-cheat
  bypass, or input disguise.

## Design

One 13 px radius. Event-driven paint. Borderless module cards with text ON/OFF
pills and no module glyphs. Small original line icons remain in the navigation
rail only. Graphite surface, warm white text, one thin accent hairline.

## Current work

- Windows is the only port. Linux/macOS need a real platform layer.
- Live Minecraft hit registration is unverified.
- Downloads are Windows ZIP/MSIX only; no fake Linux/macOS artifacts.
