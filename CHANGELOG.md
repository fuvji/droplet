# Changelog

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
- Changed distribution to proprietary. Source, build scripts, and internal
  notes are not distributed.

## 2.3.9

- Removed Air Anchor and all toggle/loop machinery.
- Made Anchor the sole anchored sequence with distinct-slot validation.
- Added the static Hydrogen product site.

## 2.3.8

- Added the optional offline combat timing profile and local annotator.

## 2.3.7

- Released the zero-dependency Rust rewrite and signed MSIX workflow.
