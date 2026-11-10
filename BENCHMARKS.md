# Droplet benchmarks

This page records what the release measures, how it was measured, and what the
numbers do not mean.

## 2.6.0 trigger routing

| Path | Time per lookup | What it does |
| --- | ---: | --- |
| Binding scan (2.4.1) | 202.39 ns | Hashes and scans up to 24 binding strings through the per-thread cache. |
| Pre-indexed route (2.6.0) | 0.45 ns | Reads one slot from a prebuilt event-to-action table. |
| Reduction | 445× | Same key event, same process, same release profile. |

The table is built when settings publish, not on the input hook. It is rebuilt
when a binding changes. The event route itself allocates nothing.

### Method

- Release profile (`opt-level = 3`, fat LTO, one codegen unit, panic-abort,
  stripped).
- AMD Ryzen 7 5700G, Windows 11.
- 1,000,000 timed lookups for each path after a 10,000-iteration warm-up.
- Both paths resolve the same `x` event (Anchor).
- The indexed path is a direct array lookup. The scan path runs the complete
  old ordered binding check.
- `std::hint::black_box` prevents the optimizer from deleting either path.

Reproduce with:

```bash
cargo test --release trigger_index_matches_the_binding_scan_and_is_faster -- --nocapture
```

The test prints:

```text
trigger route: indexed=0.45 ns  scan=202.39 ns  speedup=445.2x
```

## What this does not measure

- End-to-end input latency.
- Time inside Windows, the keyboard/mouse driver, USB, or the game client.
- Minecraft hit registration, tick alignment, or network response.
- A frame-rate, FPS, or “zero-latency” claim.

Minecraft observes input on a 50 ms tick. Droplet reduces its own routing
work; it cannot remove the game loop, the OS scheduler, or the device path.

## Release artifact

- Portable executable: **584,704 bytes**
- Release tests: **14 / 14**
- Runtime dependencies: **0**
- Network calls: **0**
- Telemetry: **none**

The signed MSIX is built from the same executable. Package signing is a local
development-certificate build, not a Microsoft Store identity.
