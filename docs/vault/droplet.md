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

- 2.10.17 portable exe: 561,152 bytes. Single artifact; no MSIX/AppX or ZIP.
  The panel is an 84%-opaque per-pixel alpha surface through
  `UpdateLayeredWindow`, so the desktop reads through the gray aurora glass.
  The 18 px corners blend into the desktop and the silhouette does not depend
  on DWM's Windows 11-only corner preference. DWM backdrop requests are not
  used: they painted an opaque layer behind the layered surface. All
  four arcs use mirrored 4×4 coverage sampling; the earlier right/bottom
  centres were on the window edge and left visible spikes. The old GDI region
  is cleared on DPI changes. A local `dwm.log` records the DWM HRESULTs.
  Captures use the layered alpha mask. The 100/125/150 DPI captures are checked
  by `tools/check-dpi-corners.ps1` and a matching unit test.
- The tag workflow builds the GNU exe from source, checks formatting and tests,
  verifies the normalized artifact hash, and attests the exe and checksum with
  GitHub build provenance. `tools/build-release.ps1` remaps build paths and
  zeroes the PE timestamp so separate clean targets agree.
- Tests: 42 / 42.
- The full source tree is rustfmt-clean. Duplicate legacy trigger bindings are
  normalized on load so the later module owns the input, matching runtime
  last-wins routing. The four site captures use an isolated deterministic
  profile; Anchor is shown on `XButton1` and the old Breach owner is cleared.
- Auto Crystal has a Return slot (`v5S3`, default `1`). A hold of at least
  160 ms or a toggle-off restores it once; a short tap does not. The restore
  is a double tap of the Return binding. The toggle-off path acquires
  `CRYSTAL_BUSY` before consuming the generation, so a busy action cannot
  swallow the request, and a new cycle cancels a stale delayed restore.
- `WM_DPICHANGED` updates scale, physical window size, rounded clip, hover,
  hit testing, and clears slider/pointer drag state together. Pointer
  coordinates stay physical, fixing the 125%/150% row and tab offset.
- Trigger assignments take ownership: a newly assigned trigger clears the
  previous owner, load-time normalization removes legacy duplicates in the
  same order, and the later module wins. This is why Anchor can use
  `XButton1` although Breach ships with it as a default. Captured keys without
  a friendly name use `vkXX`, which the parser accepts.
- `tools/input-harness.ps1` records Droplet's injected keyboard/mouse output
  with low-level Windows hooks and verifies the focus gate, master gate, and a
  Shield sequence from a temporary profile. It is a local test tool, not part
  of the exe.
- `tools/check-release.ps1` checks version, artifact size and SHA-256, test
  count, 624×408 captures, workflow hash, and local site links.
- The self-destruct waiter uses hidden `ShellExecuteW` instead of
  `std::process::Command`; the link uses bundled `rust-lld` + ICF. The 2.10.8
  size pass is 55,808 B smaller than 2.10.7 and keeps `opt-level=3`.
- Sprint settings are read only when a pulse is due; crosshair settings only
  on a magenta hit. Poll cadence and bindings are unchanged. Idle sprint and
  crosshair watchers skip the settings snapshot and lock; the sprint loop
  parks at 100 ms when inactive and unparks on activation.
- Trigger routing: 0.45 ns indexed vs 202.39 ns scanned (445×, routing only).
- Anchor defaults: 47/53 ms, Min 47-97, Max 53-103. Hydrogen A6 order;
  Air loop repeats while held.
- Game detection checks the owning process (`javaw.exe`, `java.exe`, or
  `Minecraft.Windows.exe`) and rejects browser, chat, editor, and launcher
  titles. No live game process was available for the positive path.
- No network, telemetry, game-memory access, capture evasion, anti-cheat
  bypass, or input disguise.

## Design

One 18 px radius. The layered alpha surface is the visible clip on every
supported Windows version, because Windows 10 can report success for DWM's
corner attribute while still drawing a square popup. The alpha mask is
recomputed on DPI changes and the old GDI region is cleared, so a compositor
fallback cannot reintroduce a square edge. Transparent quarter-rounds are
click-through. The visual hierarchy is a translucent gray aurora wash, a
second-pane glass rail, a filled rounded selection, and thin Segoe UI Light
type with no bold weight, in a 624×408 fixed layout; the references informed
the material, not the product behavior. The presence
watcher reuses a verified foreground game HWND, and its focus-only gate is
cached in an atomic. The favicon and site marks are one transparent outlined
droplet with no plate, border, background, or shadow. Event-driven paint.
Borderless module cards with the original 28×16 sliding knob switch and no
module glyphs. Small original line icons remain in the navigation rail only.
Graphite surface, warm white text, no painted outer rim, and no top accent line.
Min/Max sliders are independent.

## Current work

- Low latency now requests the kernel's finest available timer resolution
  (normally 5,000 × 100 ns, or 0.5 ms), records the value actually granted, and
  displays it in System > Performance. A one-second guard reasserts the request
  while the feature is enabled, and the guard stops when it is switched off.
  `SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED)` is held for the
  same interval so an idle sleep transition cannot drop the timer request.
- Auto Crystal has a Return slot (`v5S3`, default `1`). A hold of at least
  160 ms restores it after release and a latched toggle restores it when
  switched off; a short tap intentionally leaves the selected slot alone.
- The panel handles `WM_DPICHANGED`: scale, window size, rounded clip, pointer
  mapping, and hit testing are updated together when it moves between monitors.
  Pointer coordinates stay physical, matching the scaled paint coordinates, so
  hover and clicks no longer shift on 125% or 150% displays.
- Windows is the only port. Linux/macOS need a real platform layer.
- Live Minecraft hit registration is unverified.
- The only Windows artifact is the direct `droplet.exe`; no ZIP/MSIX is
  published. No fake Linux/macOS artifacts.
