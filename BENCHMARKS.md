# Droplet benchmarks

This page records what the release measures, how it was measured, and what the
numbers do not mean.

## 2.10.10 release

2.10.10 is a correctness release. Crystal toggle-off restore is now gated by
the same busy flag as the Crystal action, and retries until the gate is free;
DPI changes clear the slider drag; binding conflicts move ownership to the
newly assigned trigger; captured `vkXX` keys round-trip through the parser; and
`tools/check-release.ps1` checks the release metadata and local site links.

- Version: **2.10.10**
- Portable executable: **547,328 bytes**
- SHA-256: `a99a282c09f5b2ec0d417c6b6fac6cd6439ef19a48736826450dfbc0c5d40d32`
- Release tests: **37 / 37**
- No CPU, latency, render-speed, or size win is claimed. Trigger routing,
  watcher cadence, and the release profile are unchanged.

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

- Version: **2.10.10**
- Portable executable: **547,328 bytes**
- SHA-256: `a99a282c09f5b2ec0d417c6b6fac6cd6439ef19a48736826450dfbc0c5d40d32`
- Distribution: one direct exe; no MSIX/AppX, installer, or ZIP
- Release tests: **37 / 37**
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

2.10.9 is a behavior and GUI correction: Auto Crystal gains a Return slot with
hold/toggle semantics, and pointer hit testing stays physical through DPI
changes. Trigger routing, watcher cadence, and the release profile are
unchanged. The executable is 545,280 bytes and tests are 31 / 31. No CPU,
latency, size, or render-speed win is claimed.

2.10.10 fixes the concurrency and input edges around those features: a busy
Crystal action can no longer consume a toggle-off restore, DPI changes clear
the slider drag, duplicate trigger assignments move ownership deterministically,
and captured `vkXX` keys can be rebound. The binary is 547,328 bytes and tests
are 37 / 37; the increase is code and test coverage, not a performance claim.

2.10.8 removes the only `std::process::Command` spawn from the binary. The
self-destruct recycle waiter now uses the same dynamically loaded
`ShellExecuteW` path as links, still hidden, and still waits five seconds for
the process before using the Recycle Bin. The link uses bundled `rust-lld` with
ICF while keeping `opt-level=3`, fat LTO, one codegen unit, panic-abort, and
stripped symbols. The executable is **542,720 bytes**, 55,808 smaller than
2.10.7 and 54,272 below 596,992. Tests are 26 / 26. The event-gated sprint and
crosshair reads are unchanged.

A 60 s isolated sample after a 5 s warm-up, with the panel hidden and crosshair
off, measured 2.10.7 at **31.25 ms idle / 125 ms armed** process CPU and
**25/s idle / 140/s armed** process context switches. The sprint thread itself
measured **9/s idle / 126/s armed**. An armed repeat measured 156.25 ms and a
2.10.8 run measured 93.75 ms idle / 109.375 ms armed with 24/s / 138/s process
context switches and 9/s / 122/s on the sprint thread. CPU time is quantized by
Windows to 15.625 ms; these are short samples, not a speedup benchmark. The
gating difference is the stable result; no end-to-end or zero-latency claim is
made.

2.10.7 moves armed settings reads behind the event. The sprint watcher checks
`sprint_due_now` before cloning settings, so a held W causes at most one
settings read per 100 ms (10/s) instead of 125/s at the 8 ms poll. The crosshair
watcher samples the cursor first and only reads settings on a magenta hit
instead of every 16 ms (62.5/s). Poll cadence and bindings are unchanged. The
executable is 598,528 bytes, the same as 2.10.6 and 1,536 bytes larger than
2.10.5. Tests are 26 / 26. No CPU, latency, or size win is claimed.

2.10.6 parks the sprint watcher for 100 ms while the gate or sprint feature is
inactive, then unparks it immediately on focus or settings changes. The armed
cadence remains 8 ms, so the 100 ms pulse schedule is not rounded up.
Scheduled idle polls fall from about 125/s to 10/s. The executable is 598,528
bytes, 1,536 bytes larger than 2.10.5; 2.10.7 holds that size flat. Trigger
routing is unchanged; tests are 26 / 26. No end-to-end or zero-latency claim is
made.

2.10.5 caches the sprint and crosshair enable flags in `Engine`. The sprint
watcher no longer takes the settings lock every 8 ms when the gate is closed or
sprint is off, and the crosshair watcher skips it when unfocused or disabled.
The executable is 596,992 bytes, 512 bytes larger than 2.10.4; no size win was
claimed.

2.10.4 removes the web brand plate and duplicate brand CSS, and caches the
focus-only gate in an atomic updated when settings publish. The 160 ms presence
loop no longer clones the settings snapshot or takes the settings lock each
tick. The executable is 596,480 bytes (512 bytes smaller than 2.10.3). Trigger
routing, corner behavior, and the 25 / 25 tests are unchanged.

2.10.3 makes the DWM-composited rounded corners click-through. The hit test
returns `HTTRANSPARENT` only in the transparent quarter-rounds and keeps the
straight edges and center client-side; the hard-region fallback is unchanged.
The 8 px radius and captures are unchanged. The presence watcher also reuses an
already-verified foreground game HWND instead of re-resolving the same process
image every 160 ms.

2.10.2 prefers DWM's composited, antialiased corner on Windows 11 and uses the
GDI rounded region only as a fallback. This removes a redundant hard clip; it
is a visual fix, not a measured speedup. The web favicon and header/footer marks
now use the app's outlined droplet instead of a different solid mark. The
binary is 595,968 bytes; trigger routing is unchanged.

2.10.1 fixed false game presence. The watcher resolves the owning process image
instead of trusting a title, rejects browsers, chat clients, editors, and
`MinecraftLauncher.exe`, and only accepts Java (`javaw.exe` / `java.exe` with a
game signature) or Bedrock (`Minecraft.Windows.exe`). A cheap class/title
prefilter avoids opening unrelated processes; two samples confirm presence, and
a cold scan runs every 8 ticks (1.28 s).

2.10.0 ported Hydrogen's `A6` Anchor branch: the two original 37–43 ms settles
remain fixed, the final settle uses **47–97 / 53–103** with **47/53** defaults,
and the Air loop restores the held-repeat path. The shared corner radius dropped
from 13 px to 8 px. The binary is 593,408 bytes.
