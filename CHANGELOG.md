# Changelog

## 2.9.0

- Replaced module pill switches with round power buttons and gave every module
  an original 18 px item-style line glyph. The glyphs are drawn from GDI+
  primitives, carry no background plate, and use no Mojang textures.
- Removed the tray icon, tray menu, and taskbar integration. Hiding the panel is
  now sticky: an alt-tab or focus change cannot resurrect it. Ctrl+Right Shift
  shows or hides it. Close stops the watchers and exits immediately.
- Removed the redundant Dash `Clicks 1x` row and the Behaviour `Capture` row.
  Dash is one row shorter; the public audit still states the visible-capture
  boundary.
- Restored the Anchor 37–43 ms default and slider window. The macro still
  enforces the 50 ms tick floor, and the card now states that floor explicitly.
- Dash, Shield, and Pearl select the slot and click in one SendInput batch.
  This is ordinary input, not a capture or anti-cheat bypass.
- The low-level hooks now reject unbound events with the prebuilt trigger table
  before cloning the settings Arc. Bound events take the existing path.
- Self-destruct is a one-second hold with a visible progress percentage.
- Deeper graphite palette, warm white text, a single accent hairline, and the
  same 13 px shared corner radius.
- Added a downloads section with local OS detection, a dedicated release ledger
  at `releases.html`, SHA-256 checksums, and actual Windows ZIP/MSIX artifacts.
- Added a tag-driven Windows release workflow and detailed
  `releases/v2.9.0.md` notes.
- The old Freebuff deployment was unpublished. `https://fuvji.github.io/droplet/`
  is the canonical product URL.
- Release binary: 601,600 bytes, SHA-256
  `3c40feceec36e830cbf571614372b41f68c96bc2db417bf1c3d2fa8822db5dd9`.
  Signed MSIX: 306,082 bytes, SHA-256
  `c569446ec120c8548b009e4714d3e6d3b3df5001fa4a51ae2bf60f6c2f84a215`.
  Release tests: 19/19.
- Boundaries unchanged: no streamproof mode, no capture evasion, no game-memory
  reads, no network calls, no anti-cheat bypass, no input disguise, and no
  zero-latency or auto-update claim. Linux and macOS remain unported.

## 2.8.0

- Unified the window silhouette on one logical 13 px radius. The painted
  hairline, the Win32 window region, and the four site capture masks now use the
  same value, so the clip and the border agree at the corners at every scale.
- Removed the last per-chip allocations from the panel paint path: binding and
  slot labels are borrowed from the settings instead of being formatted into a
  new String for every visible chip. The release binary is 2,048 bytes smaller
  at 594,944 bytes.
- Added a release test that keeps the borrowed binding lookup exactly in sync
  with the INI value for every painted chip: 19/19 passing.
- Rebuilt the product page as a cleaner editorial layout: larger display type,
  hairline structure, consistent spacing, restrained scroll and hover motion,
  a measurement-detail table, panel anatomy notes, and refined preview frames.
  The headline is now **Minecraft Zero Macro**.
- Added `site/public/refine.css` as a separate design layer over the existing
  stylesheet, so the base layout stays intact and the refinement is easy to
  audit. The page still ships no third-party scripts, external fonts, or video.
- Release binary: 594,944 bytes, SHA-256
  `57449188e2abdf7da33620cf4698e3610030edf689512a4a55e2045621bd8ad3`.
  Signed MSIX: 312,164 bytes, SHA-256
  `e3c23f85c31a67a0272058f8995c0df87b6d261609c9337553f9dad6d52572b3`.
- Boundaries unchanged: no streamproof or capture-evasion mode, no game-memory
  reads, no network calls, no anti-cheat bypass, no input-device disguise, and no
  zero-latency claim.

## 2.7.0

- Restored the accent color picker as a compact System > Color card: six
  presets plus Red/Green/Blue sliders, persisted in the same `thmC` key used by
  the earlier theme. Accent, borders, switches, chips, and charts all follow the
  selected color, and switch labels flip between light and dark text on a
  perceptual luma threshold.
- Restored per-trigger Hold/Toggle modes on all eleven trigger cards. Hold
  keeps the existing one-shot behavior. Toggle latches one worker per module,
  repeats that module, and stops on master off, module off, a mode change, a
  reset, or losing the foreground gate. A generation counter invalidates the
  old worker so a rapid double-tap cannot leave two loops injecting the same
  module.
- Fast Use, Potion, Pearl, Shield, and Crystal now know they are latched, so
  hold-gated sequences repeat instead of idling on a key that is already up by
  the time the worker starts.
- Idle toggles no longer post repaint messages on focus loss, settings changes,
  or resets; the stop path returns before touching the window when no latch is
  live.
- Added two release tests for toggle slot coverage and latch generations:
  18/18 passing.
- Replaced the stale 2.6.2 WebM reel with a zero-JavaScript CSS crossfade of the
  four current native captures. The page no longer ships a video asset.
- Release binary: 596,992 bytes, SHA-256
  `7fff6d11c36083326bb6bd85f273a8cd153854a057664c79be580a68d04f4a3f`.
  Signed MSIX: 313,046 bytes, SHA-256
  `35c7973ef43564e7ba25b32ea1e42a582b73ec57036eda6ff984dc171e3af18d`.
- Boundaries unchanged: no streamproof or capture-evasion mode, no game-memory
  reads, no network calls, no anti-cheat bypass, and no zero-latency claim.

## 2.6.2

- Fixed the native window corners: the layered popup now enforces a rounded
  region after the acrylic path instead of trusting DWM's success result, so
  the painted hairline and the clipped edge agree at every scale.
- Unified the brand mark across the window, transparent package icons, the
  favicon, and the site header. The dark plate behind the droplet is gone.
- Added a four-slot native UI reel to the site, with a different captured tab
  on every slide. It is explicitly labelled as UI stills, not gameplay.
- Reframed the product page around **Minecraft Macro Suite** in the title,
  metadata, and opening statement, and tightened the graphite visual system.
- Removed the dead palette editor, its config key, strings, and theme actions.
  The fixed graphite palette no longer carries unused color machinery.
- Release binary: 581,632 bytes, SHA-256
  `264096e17917010b500bb971ae88dd868d8b4d2cc8369c009d4aa435872422c5`.
  Signed MSIX: 305,682 bytes, SHA-256
  `34ba5f4ec57651fcca40cbce7111b8403efaa63bb77a50c47b838e005ea60770`.
- Release tests: 14/14 passing. Boundaries are unchanged: no streamproof or
  capture-evasion mode, no game-memory reads, no network calls, no anti-cheat
  bypass, and no zero-latency claim.

## 2.6.1

- Sprint now re-pulses every 100 ms while physical W is held instead of once
  per second. The watcher polls every 8 ms so the 100 ms QPC deadline is not
  rounded up by a coarse sleep, and the sprint key is still tapped rather than
  held between pulses.
- Added a release test for the exact 100 ms deadline and moved all current
  source links to <https://github.com/fuvji/droplet>. The old repository URL
  redirects there.
- Added `SECURITY.md`, `DEVELOPING.md`, and `site/public/llms.txt` so the
  repository and static site cover vulnerability reporting, build/package
  steps, and machine-readable product facts.
- Release binary: 584,704 bytes, SHA-256
  `1bf9e99079801d4bb5b1d351cf0532aa8a792e31c5847eab0699fc149bce6d51`.
  Signed MSIX: 308,365 bytes, SHA-256
  `bf943f48027d9e503efebc984f57cdc165d086a2e628fd11e36e706eff529bf6`.
- Release tests: 14/14 passing. The trigger-routing benchmark is unchanged at
  0.45 ns indexed versus 202.39 ns scanned; live Minecraft validation remains
  separate.
- Boundaries unchanged: no streamproof/capture-evasion mode, no game-memory
  reads, no network calls, no anti-cheat bypass, and no zero-latency claim.

## 2.6.0

- Renamed the product from Hydrogen to Droplet throughout the binary, tray,
  window class, configuration, installer, documentation, and site. Existing
  `%LOCALAPPDATA%\Hydrogen` data is moved to `%LOCALAPPDATA%\Droplet` on first
  launch when the new directory does not already exist.
- Compacted the native panel from 648×432 to 624×408 and replaced the blue
  accent picker with a fixed graphite/white palette. The Appearance card now
  reports Graphite, Acrylic, Compact, and Soft corners. The existing Windows
  acrylic backdrop and rounded window/DWM corners are preserved.
- Trimmed two redundant Anchor waits while retaining the 50 ms game-tick
  floor: one duplicate settle after the final hotbar selection and one trailing
  busy hold. Anchor's normal path is 100 ms shorter. The timing sliders still
  clamp every wait to at least one tick.
- Added a monochrome Droplet site palette with neutral smoke gradients, grain,
  updated droplet marks, compact native captures, and the new panel dimensions.
  The site remains zero-JavaScript, zero-network, and capture-visible. Benchmark
  and signal-bar geometry now lives in the stylesheet, so the strict CSP does
  not block inline style attributes.
- Release binary: 584,704 bytes. Release tests: 14/14 passing. The routing
  benchmark remains 0.45 ns indexed versus 202.39 ns scan (445×, routing only).
- Deliberate boundaries unchanged: no streamproof/capture-evasion mode, no
  game-memory reads, no network calls, no anti-cheat bypass, and no zero-latency
  or zero-delay claim. Minecraft still observes input on its 50 ms tick, and
  live gameplay hit registration is unverified.

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
- Added a direct sponsor inventory at `/sponsors`. The site still loads no
  third-party ad scripts, analytics, cookies, external fonts, or client-side
  JavaScript.
- Polished the site after release: four distinct native panel captures, an
  animated aurora/grid background, rounded surfaces throughout, and clean
  sponsor routing. The GitHub repository homepage now points at the product
  site.
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
- Published the 2.4.1 source tree at github.com/fuvji/droplet and aligned the
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
- Added the static Droplet product site.

## 2.3.8

- Added the optional offline combat timing profile and local annotator.

## 2.3.7

- Released the zero-dependency Rust rewrite and signed MSIX workflow.
