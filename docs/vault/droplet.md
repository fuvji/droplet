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

- 2.9.0 portable exe: 601,600 bytes.
- Signed MSIX: 306,082 bytes.
- Tests: 19 / 19.
- Trigger routing: 0.45 ns indexed vs 202.39 ns scanned (445×, routing only).
- No network, telemetry, game-memory access, capture evasion, anti-cheat
  bypass, or input disguise.

## Design

One 13 px radius. Event-driven paint. Round power controls. Original
transparent item-style line glyphs. Graphite surface, warm white text, one
accent hairline.

## Current work

- Windows is the only port. Linux/macOS need a real platform layer.
- Live Minecraft hit registration is unverified.
- Downloads are Windows ZIP/MSIX only; no fake Linux/macOS artifacts.
