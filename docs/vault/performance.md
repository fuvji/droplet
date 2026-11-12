# performance

## Measured

| Path | Time | Method |
| --- | ---: | --- |
| Old binding scan | 202.39 ns | 1,000,000 lookups, release profile |
| Pre-indexed route | 0.45 ns | One array read from a prebuilt table |
| Reduction | 445× | Same event, process, and profile |

Machine: AMD Ryzen 7 5700G, Windows 11. Release profile: `opt-level=3`, fat LTO,
one codegen unit, panic-abort, stripped.

## 2.10.8 changes

- Removed the only `std::process::Command` spawn: the self-destruct waiter now
  uses the existing hidden `ShellExecuteW` path. This is a size change, not a
  behavior change.
- Link with bundled `rust-lld` and ICF while retaining `opt-level=3`, fat LTO,
  one codegen unit, panic-abort, and stripped symbols. The exe falls from
  598,528 to **542,720 bytes** (55,808 B smaller; 54,272 B below 596,992).
- Isolated 60 s sample on this machine: 2.10.7 measured 31.25 ms idle /
  125 ms armed CPU, 25/s / 140/s process context switches, and 9/s / 126/s on
  the sprint thread. A 2.10.8 sample measured 93.75 / 109.375 ms, 24/s / 138/s,
  and 9/s / 122/s. CPU time is quantized at 15.625 ms; no speedup claim.

## 2.10.7 changes

- The sprint watcher checks `sprint_due_now` before cloning settings. While W
  is held, settings reads fall from 125/s to at most 10/s; the 8 ms poll and
  100 ms pulse schedule are unchanged.
- The crosshair watcher samples the cursor first and reads settings only on a
  magenta flash instead of every 16 ms. The 16 ms sampling cadence is unchanged.
- The executable remains 598,528 bytes, the same as 2.10.6. This is a
  runtime-read reduction; it is not a measured CPU, latency, or size win.

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

## 2.9.3 changes

- Restored a 55 ms Anchor floor. The 37–43 ms path could right-click inside the
  same 50 ms client tick as the slot change and use the previous item.
- Removed the top accent line. The single-file exe is unchanged in size at
  591,872 bytes; MSIX/AppX/ZIP packaging was retired.

## 2.10.0 changes

- Ported Hydrogen's `A6` Anchor order. The two early settles remain fixed at
  37–43 ms; the final settle is 47–97 / 53–103 with 47/53 defaults.
- Air loop repeats while the trigger is held. The shared corner radius drops
  from 13 px to 8 px across paint, window region, DWM, and capture mask.
- Trigger routing is unchanged at 0.45 ns indexed versus 202.39 ns scanned.
  No startup or in-game latency speedup is claimed. Tests: 20 / 20.
- Startup remains a native exe with no installer or package registration. No
  instant-start or latency guarantee is made.

## 2.10.1 changes

- Game presence now resolves the owning process image instead of trusting a
  window title. A cheap title/class prefilter limits process queries to likely
  candidates; two samples confirm presence and a miss clears immediately.
- Cold discovery runs every 8 ticks (1.28 s) instead of 12 (1.92 s). This is a
  responsiveness change, not an input-latency measurement. Trigger routing is
  unchanged; tests are 24 / 24. No startup or in-game speedup is claimed.

## 2.10.2 changes

- The window now uses DWM's composited rounded clip when available and falls
  back to the GDI region only when DWM cannot round the popup. This removes a
  redundant hard clip; it is a visual fix and no frame-time or startup speedup
  is claimed. The logical 8 px radius, capture mask, and routing are unchanged.

## 2.10.3 changes

- The DWM-rounded transparent quarter-rounds now return `HTTRANSPARENT`; the
  center and straight edges remain client-side. The 8 px fallback region is
  unchanged. This is an interaction fix, not a rendering-speed measurement.
- Once a foreground game HWND is verified, the presence watcher reuses it
  instead of resolving the same process image every 160 ms. Presence, focus,
  and cold-discovery semantics are unchanged. No input-latency or frame-time
  speedup is claimed.

## 2.10.6 changes

- The sprint watcher parks at `SPRINT_IDLE_MS` (100 ms) while the gate or
  sprint feature is inactive, then unparks immediately on focus/settings
  changes. The armed cadence remains 8 ms.
- Scheduled idle polls drop from roughly 125/s to 10/s. This is a scheduling
  change, not a measured CPU or end-to-end latency result.
- The binary is 598,528 B, 1,536 B larger than 2.10.5; no size win is claimed.
- Tests: 26 / 26.

## 2.10.5 changes

- `Engine` caches sprint and totem enablement. The sprint watcher and crosshair
  watcher skip the settings snapshot and lock while inactive.
- The 8 ms / 100 ms cadence and sprint reset are unchanged. Trigger routing and
  the release profile are unchanged.
- The binary is 596,992 B, 512 B larger than 2.10.4. This is a lock-traffic
  cleanup, not a size or end-to-end speed win.

## 2.10.4 changes

- The site brand mark is one transparent rule; the favicon tile and duplicate
  plate CSS are gone. This is source/dead-style debloat, not a render-speed
  claim.
- `Engine` caches `focus_only` in an atomic updated by `publish`. The presence
  loop no longer clones the settings snapshot or takes its lock every 160 ms.
- Trigger routing remains 0.45 ns indexed versus 202.39 ns scanned. The release
  profile is unchanged. No end-to-end latency, game-tick, or zero-latency claim
  is made.

## Not measured

- End-to-end input latency.
- Game-tick or hit-registration timing.
- Anything called “quantum”, “zero latency”, or “100% no restriction”.

See [[droplet]] and [[release]].
