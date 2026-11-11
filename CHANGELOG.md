# Changelog

## 2.10.3

- Make the transparent quarter-rounds click-through when DWM owns the corner
  clip. The center and straight edges remain client-side, and the Windows 10
  GDI-region fallback is unchanged.
- Reuse an already-verified foreground game HWND in the presence watcher instead
  of re-resolving the same process image every 160 ms. Presence/focus semantics
  and the cold-discovery scan are unchanged; no latency speedup is claimed.
- Add a pure unit test for the rounded-corner hit geometry. Portable executable:
  **596,992 bytes**, SHA-256
  `c78625faa251ff71934e6e2d08a06b13b8280deb1c247a27536199e039cc9613`.
  Tests: **25 / 25**.
- Boundaries unchanged: no game-memory access, network, telemetry, streamproof
  mode, anti-cheat bypass, input disguise, or zero-latency claim.

## 2.10.2

- Prefer DWM's compositor-rounded, antialiased corner on Windows 11. Keep the
  GDI rounded region only as a fallback, so a working DWM clip is not clipped a
  second time. The 8 px logical radius is unchanged.
- Replace the different solid web droplet with the app's outlined droplet in
  the favicon, header, and footer; cache-bust the icon URL.
- Portable executable: **595,968 bytes**, SHA-256
  `7fe0b990119f9e07a5dfb3a7a895fa1ba3aa1a8028e6cfee4570fe85884f7e1d`.
  Tests: **24 / 24**.
- Boundaries unchanged: no game-memory access, network, telemetry, streamproof
  mode, anti-cheat bypass, input disguise, or zero-latency claim.

## 2.10.1

- Fixed false **Game detected** status. Detection now resolves the owning
  process image and accepts only Java (`javaw.exe`/`java.exe` with a game
  signature) or Bedrock (`Minecraft.Windows.exe`); browsers, Discord, editors,
  and `MinecraftLauncher.exe` are rejected.
- Added a cheap title/class prefilter, two-sample presence confirmation, and an
  8-tick cold discovery scan. The `Only in Minecraft` gate uses the same state.
- Portable executable: **595,968 bytes**, SHA-256
  `239792e66f5684f6ecfde7f7c5395e5bb11f8870d50d89f8288b262ba841d696`.
  Tests: **24 / 24**.
- Boundaries unchanged: no game-memory access, process injection, hiding,
  anti-cheat bypass, input disguise, or capture evasion.

## 2.10.0

- Ported Hydrogen's `A6` Anchor branch: select Anchor, right-click, 37-43 ms,
  right-click, Glowstone, 37-43 ms, double Totem tap, configured wait, then
  right-click.
- Anchor sliders now expose Min **47-97 ms** and Max **53-103 ms**, with
  defaults **47/53**. The old 55 ms floor is gone; the exact Hydrogen fixed
  waits are retained.
- Added the **Air loop** switch from Hydrogen's `vE`. Holding the trigger with
  Air on repeats the sequence; Toggle mode still uses one latched worker.
- Reduced the shared corner radius from 13 px to **8 px**, matching Windows'
  smaller rounded clip across paint, window region, DWM, and capture mask.
- Rebuilt the site with an Obsidian-style vault graph. The graph is inline SVG
  and CSS; no third-party script or font was added.
- Portable executable: **593,408 bytes**, SHA-256
  `6b43e15ebf6ad4e7ca0cb10b0be085acfa2b47839faf6dcb15a7c1fc52a9f0ab`.
  Tests: **20 / 20**. No live Minecraft client was available for the Anchor
  sequence.
- Boundaries unchanged: no game-memory access, network, telemetry, streamproof
  mode, anti-cheat bypass, input disguise, or zero-latency claim.

## 2.9.3

- Restored a tick-safe Anchor settle. Defaults are **55/65 ms**, with Min
  **55-97 ms** and Max **65-103 ms**. The 55 ms floor crosses a 50 ms client
  tick before the right-click; 37-43 ms could place-while-switching and use the
  previous hotbar item.
- Removed the remaining 1 px accent line from the top of the panel. DWM's
  rounded clip and the edge-to-edge surface are now the only boundary, with no
  painted outer rim and no bright top edge.
- Retired MSIX/AppX, installer, and ZIP packaging. The published Windows
  artifact is one direct **591,872-byte** `droplet.exe`, with SHA-256
  `288b1019d61e46805fff31737ab926c52bc20541a53ef05841c8b356a27f5873`.
  The installed AppX package was removed from the build machine.
- Removed the package manifest, AppX assets, signing scripts, installer, and
  package tools from the repository.
- Updated the site, release ledger, workflow, and vault for the single-exe
  download path. Tests remain **19 / 19**.
- Boundaries unchanged: no streamproof mode, no game-memory access, no network
  calls, no anti-cheat bypass, no input disguise, and no zero-latency or
  instant-speed claim.

## 2.9.2

- Fixed the coupled Min/Max sliders. `normalize_timing` now clamps each field
  to its own domain and never rewrites the partner. Execution sorts an inverted
  pair at read time, so only the slider the user drags changes.
- Restored the original 28×16 sliding knob switch on every module and switch
  row. The text ON/OFF pills and power glyphs are gone again.
- Anchor now exposes Min **37–97 ms** and Max **43–103 ms**, with defaults
  **37/43**. The macro uses the configured value exactly; the old 50 ms floor,
  the three hardcoded 50 ms slot sleeps, and the second wait after each action
  were removed. Each slot gets one configured settle before its action.
- Restored the **Self-destruct** label in the Danger zone. The one-second hold,
  data purge, and portable-executable recycle are unchanged.
- Removed the painted outer window hairline. DWM's border is already suppressed,
  so the panel no longer shows a double-thickness rim.
- Release binary: 591,872 bytes (512 bytes smaller than 2.9.1), SHA-256
  `2d4a915ca68b3c4ef944fe208b593f3c816f55b8b605a478d23a5cfd5d70cc46`.
  Signed MSIX: 301,609 bytes, SHA-256
  `94d47fd44f092b571ade122b2d020011da36b41cd185517895f28133040e6c87`.
  Portable ZIP: 285,751 bytes, SHA-256
  `928f816eb2dc4fa028273a9af9f8236e18a2e6c4db78ee612bd6a8d9fb2f5841`.
  Tests: 19/19; package 2.9.2.0 installed Ok.
- Boundaries unchanged: no streamproof mode, no capture evasion, no game-memory
  access, no network calls, no anti-cheat bypass, no input disguise, and no
  zero-latency, unrestricted-speed, or auto-update claim.

## 2.9.1

- Removed every module glyph and the per-card outline. Module cards are now a
  single borderless surface with the title on the left and a compact text
  ON/OFF pill on the right. The navigation rail keeps its small line icons.
- Retired the Anchor `Tick floor` row. The macro still applies the 50 ms
  Minecraft tick floor before the next hotbar action.
- Deleted the glyph painter, the card-code helper, the diamond path helper,
  the per-card `round_stroke` call, and the unused luminance field. The
  portable executable is **592,384 bytes**, 9,216 bytes smaller than 2.9.0.
- Thinned the single outer window hairline from 1.0 to 0.75 logical pixels.
- Removed `site/public/assets/module-icons.svg` and the module icon markup, so
  the module section ships less HTML, CSS, and one fewer request.
- Release binary: 592,384 bytes, SHA-256
  `c0b5c64793d3a94f70c4141903715f58a3458665db3c0db5e51ca38ae297fe8a`.
  Signed MSIX: 301,774 bytes, SHA-256
  `92768f518581cfd2662f8e83506ec27127742833ea6d995b32d507a5dc3f3045`.
  Portable ZIP: `285,770` bytes, SHA-256 `8be91f12560842686ecf306b48f4026a78abef9358d79f4a1ee1a6e2c3d36920`. Tests: 19/19.
- No routing or behavior change: 0.45 ns indexed vs 202.39 ns scanned. The
  app's System-tab paint snapshot was 2.86 ms after this change versus 2.60 ms
  before; that single sample is not presented as a speedup.
- Boundaries unchanged: no streamproof mode, no capture evasion, no
  game-memory access, no network calls, no anti-cheat bypass, no input
  disguise, and no zero-latency or auto-update claim.

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
