# Changelog

## 2.5.0

- Replaced the per-event binding scan with a prebuilt event-to-action index.
  Bindings are parsed when settings publish; the low-level hook now resolves a
  key with one array read. The routing microbenchmark measured 0.45 ns for the
  indexed path versus 202.39 ns for the old scan (445× on the same release
  build and key event). This is routing only, not end-to-end latency.
- Added a 14th release test covering routing equivalence and the benchmark.
  The release binary is 585,728 bytes.
- Rebuilt the product site as an ion-lab interface with a benchmark chart, a
  runtime pipeline model, CSS-only motion, a native-capture preview lab, and
  responsive sponsor and audit layouts.
- Added a direct sponsor inventory at `/sponsors.html`. The site still loads no
  third-party ad scripts, analytics, cookies, external fonts, or client-side
  JavaScript.
- Rebuilt and signed the 2.5.0 MSIX. The package is 312,525 bytes and its
  signature verifies with the local development certificate.
- Kept the deliberate boundaries: no game-memory access, no network calls, no
  capture concealment, no anti-cheat bypass, no scan evasion, and no
  zero-latency claim. Live Minecraft hit registration remains unverified.

## 2.4.1

- The local cadence brain now uses a warm-up EWMA: early activations adapt
  quickly, then the step size settles to a 12.5% floor. The configured Min/Max
  still clamps every selected wait.
- System > Behaviour now reports `Capture: Visible`; there is no streamproof or
  capture-evasion mode.
- Rebuilt the product site around a dark violet/block system with timing
  diagrams, a release audit, a sitemap, and asset-aware cache headers.
- Added a 13th release test for cadence adaptation. The release binary is
  582,656 bytes.
- Published the 2.4.1 source tree at github.com/fuvji/hydrogen and aligned the
  licence, README, and site with public-source, proprietary distribution.

## 2.4.0

- Sprint now pulses the configured sprint key once per second while physical W
  is held, without holding the key between pulses. The watcher is lock-free and
  parses the binding only when a pulse is due.
- W-Tap now defaults to S-Tap with a 132–148 ms window. Configs carrying the old
  W / 100–120 ms default pair migrate once; custom modes and ranges are kept.
- Removed the hidden capture exclusion. The panel is ordinary visible desktop
  composition and has no streamproof/capture-evasion mode.
- Reduced the portable release binary from 604,672 bytes in 2.3.8 to 582,144
  bytes in 2.4.0.
- Added an actual 2.4.0 interface capture to the product site and README.
- Redesigned the product site with a colder, professional slate/ice-blue
  palette.
- Changed distribution to proprietary. The source repository was published
  separately in 2.4.1; see that entry.

## 2.3.9

- Removed Air Anchor and all toggle/loop machinery.
- Made Anchor the sole anchored sequence with distinct-slot validation.
- Added the static Hydrogen product site.

## 2.3.8

- Added the optional offline combat timing profile and local annotator.

## 2.3.7

- Released the zero-dependency Rust rewrite and signed MSIX workflow.
