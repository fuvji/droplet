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

## 2.6.1 Sprint schedule

Sprint is the only repeating watcher. The deadline is now **100 ms** instead
of 1 second. The watcher samples every **8 ms**, so the next pulse is scheduled
against a QPC deadline rather than being delayed by a full poll interval. The
key is tapped once per pulse and is never held down.

The release test `sprint_pulses_every_hundred_ms` checks that the deadline is
`qpf / 10` and that the next deadline advances by the same amount. This is a
scheduler test, not an end-to-end game-latency measurement.

Reproduce with:

```bash
cargo test --release sprint_pulses_every_hundred_ms
```

## What this does not measure

- End-to-end input latency.
- Time inside Windows, the keyboard/mouse driver, USB, or the game client.
- Minecraft hit registration, tick alignment, or network response.
- A frame-rate, FPS, or “zero-latency” claim.

Minecraft observes input on a 50 ms tick. Droplet reduces its own routing
work; it cannot remove the game loop, the OS scheduler, or the device path.

## Release artifact

- Version: **2.10.0**
- Portable executable: **593,408 bytes**
- SHA-256: `6b43e15ebf6ad4e7ca0cb10b0be085acfa2b47839faf6dcb15a7c1fc52a9f0ab`
- Distribution: one direct exe; no MSIX/AppX, installer, or ZIP
- Release tests: **20 / 20**
- Runtime dependencies: **0**
- Network calls: **0**
- Telemetry: **none**

2.9.0 changed GUI controls and input batching, not the trigger-routing lookup,
so the 0.45 ns / 202.39 ns routing numbers remain the 2.6.0 measurement. The
larger 2.9.0 executable reflects the icon, power-control, and download surface
of that release; it was not presented as a size reduction.

2.9.1 removed the module glyph painter, every per-card outline stroke, and the
Anchor floor row. The executable was 9,216 bytes smaller (592,384 vs 601,600).
Trigger routing was unchanged. The app's own System-tab paint snapshot read
2.86 ms after that change versus 2.60 ms before; that single sample was not a
benchmark and 2.9.1 did not claim a render-speed win.

2.9.2 made Min/Max sliders independent, restored the 28×16 knob switch, and
removed the painted outer window hairline. It also removed the Anchor 50 ms
floor and three fixed 50 ms slot sleeps; each slot received one configured
settle before its right-click. The System-tab paint snapshot in the 2.9.2
capture read **2.15 ms**; that is one UI sample, not a frame-time benchmark.

2.10.0 ports Hydrogen's `A6` Anchor branch: the two original 37–43 ms settles
remain fixed, the final settle uses **47–97 / 53–103** with **47/53** defaults,
and the Air loop restores the held-repeat path. The shared corner radius drops
from 13 px to 8 px. The binary is 593,408 bytes; trigger routing is unchanged.
No startup or latency claim is made.
