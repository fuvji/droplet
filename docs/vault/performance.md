# performance

## Measured

| Path | Time | Method |
| --- | ---: | --- |
| Old binding scan | 202.39 ns | 1,000,000 lookups, release profile |
| Pre-indexed route | 0.45 ns | One array read from a prebuilt table |
| Reduction | 445× | Same event, process, and profile |

Machine: AMD Ryzen 7 5700G, Windows 11. Release profile: `opt-level=3`, fat LTO,
one codegen unit, panic-abort, stripped.

## 2.9.0 changes

- Unbound key events are rejected by `Engine::has_trigger` before cloning the
  settings `Arc`.
- Dash/Shield/Pearl slot select and click share one `SendInput` batch.
- The icon and power-control additions increased the exe from 594,944 to
  601,600 bytes. This is not presented as a size reduction.

## 2.9.1 changes

- Removed the module glyph painter, per-card outline strokes, and the Anchor
  floor row. The exe fell from 601,600 to 592,384 bytes.
- No routing change. A single System-tab paint snapshot read 2.86 ms after the
  change versus 2.60 ms before; that is not a benchmark and no render speedup
  is claimed.

## 2.9.2 changes

- Removed the anchor 50 ms tick floor and three hardcoded 50 ms slot sleeps.
  Each slot now gets one configured 37–43 ms settle before its action.
- That removes three fixed 50 ms sleeps and two extra post-action waits from a
  normal anchor cycle. It does not create an unrestricted mode and is not
  validated in a live client.
- The 2.9.2 System-tab paint snapshot read 2.15 ms. This is one sample, not a
  frame-time benchmark, and no render-speed claim is made.

## Not measured

- End-to-end input latency.
- Game-tick or hit-registration timing.
- Anything called “quantum”, “zero latency”, or “100% no restriction”.

See [[droplet]] and [[release]].
