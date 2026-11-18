# release

## 2.10.16 — 2026-11-18

- Portable exe: 558,080 B,
  `fff3199a95970a337c2967bfa198b94bb2a937404428aba7630d1b7bdbea24ed`.
- Tests: 42 / 42.
- Changes: live 100/125/150 DPI captures and a PNG corner regression, plus the
  graphite/glass visual pass (filled rail selection, stronger titles, white
  switch knobs, deeper tracks). The alpha/premultiply path stays one buffer
  pass; a measured card-shadow experiment was removed. No network, telemetry,
  or input path changed.

## 2.10.15 — 2026-11-18

- Portable exe: 556,032 B,
  `a7061c54a2f5379e1e7df32dcb0b52b73b9e275a6034b1e9066dfec9c0ded31a`.
- Tests: 41 / 41.
- Changes: fix the corner spikes in the interim layered path by mirroring the
  right and bottom arcs one radius inside the window, then sampling all four
  12 px corners 4×4. Fold alpha, premultiply, and corner rewrite into one
  buffer pass. Reset now restores Keep panel open to its shipped on state. The
  silhouette is Windows 10-safe because it does not rely on DWM's corner
  preference. A local `%LOCALAPPDATA%\Droplet\dwm.log` records the DWM
  HRESULTs, region clear, size, and DPI. Captures use the layered alpha mask.
  The input harness recorder now pumps `GetMessage` directly instead of a
  hidden WinForms form, but its Shield case still intermittently records no
  output here; a separate low-level probe observed the app emit `h` down/up
  plus left click down/up. That official check is unresolved, not passed. No
  routing, timer, or input path changed.

## 2.10.14 — 2026-11-16

- Portable exe: 550,400 B,
  `7ab6aed9862148dce35872fed3dbe029082cb6dd2ac1ca79f4170ee7511d3ded`.
- Tests: 39 / 39.
- Changes: ask DWM for `DWMWCP_ROUND` on the region path instead of
  `DONOTROUND`, so its compositor shadow follows the 8 px region rather than
  drawing a square corner wedge. Border color remains clear and non-client
  rendering remains enabled for acrylic. Captures use the live region. No
  routing, timer, input, or render path changed. A magenta-backdrop capture
  showed magenta at the exact corner; no live game or separate Windows 10
  device was available.

## 2.10.13 — 2026-11-16

- Portable exe: 550,400 B,
  `0dba32e8dc665754d911122de55ba29760cfc5e384e5132662193e5d2b5e264e`.
- Tests: 39 / 39.
- Changes: remove DWM's light square non-client frame from the authoritative
  region path. The border color is cleared and DWM is told not to round while
  the region is installed; DWM rounds only on the fallback. Non-client
  rendering stays enabled so the acrylic backdrop is preserved. Capture alpha
  now comes from the live window region rather than a hardcoded mask. Timer,
  anti-sleep, DPI, and provenance behavior from 2.10.12 is unchanged. A
  magenta-backdrop capture showed the square frame is gone; no live game or
  separate Windows 10 device was available.

## 2.10.12 — 2026-11-16

- Portable exe: 550,400 B,
  `17de486afd22f1a2485e859fd708495dedeb3ded32343121cbdaacf125fc34df`.
- Tests: 39 / 39.
- Changes: authoritative GDI 8 px window region on every supported Windows
  version, fixing incomplete corners on Windows 10; timer target now uses the
  kernel's finest value (`max`) and a one-second guard reasserts 0.5 ms while
  low latency is on; a system-required execution state is held for the same
  interval. Tagged-source GNU build, formatted and tested first, reproducible
  path remapping and zeroed PE timestamp, verified artifact hash, GitHub
  build-provenance attestation, and the local input harness remain. Timer
  resolution was observed at 0.5 ms here; that is not an end-to-end latency
  claim. No live injection or separate Windows 10 device was available.

## 2.10.11 — 2026-11-15

- Portable exe: 547,840 B,
  `aa9f514c7ad890c60e99836c88e4ba4052dbb737d580d90f3b5ffbc1cbbccc71`.
- Tests: 38 / 38.
- Changes: rustfmt-clean source tree; load-time duplicate trigger binding
  normalization matching last-wins routing; deterministic isolated capture
  profile with Anchor on `XButton1`; final image/preview refresh. No CPU,
  latency, or speedup claim. Live injection remains unverified.

## 2.10.10 — 2026-11-14

- Portable exe: 547,328 B,
  `a99a282c09f5b2ec0d417c6b6fac6cd6439ef19a48736826450dfbc0c5d40d32`.
- Tests: 37 / 37.
- Changes: busy-safe Crystal toggle-off restore with retry and stale-cycle
  cancellation; slider/pointer drag reset on DPI changes and lost capture;
  trigger ownership on rebinding plus last-wins routing for legacy duplicate
  configs (`XButton1` on Anchor); `vkXX` capture round-trip; release consistency
  checker. No CPU, latency, or speedup claim. Live injection remains
  unverified.

## 2.10.9 — 2026-11-13

- Portable exe: 545,280 B,
  `2cfb0020918fc735ef73a4ffbae62ceb599d02dd44c6122b8794fcce3686861f`.
- Tests: 31 / 31.
- Changes: Auto Crystal Return slot (`v5S3`, default `1`) with a 160 ms hold
  threshold, generation-gated toggle-off restore, and no restore on a short
  tap; physical pointer hit testing across `WM_DPICHANGED`; DPI-aware release
  captures. No CPU, latency, or speedup claim. Live injection remains
  unverified.

## 2.10.8 — 2026-11-12

- Portable exe: 542,720 B,
  `d7648c2cad510014590fdbe578d14900d385cfab0d80a832b8a46989e1f41507`.
- Tests: 26 / 26.
- Changes: replace the self-destruct waiter's `std::process::Command` with
  hidden `ShellExecuteW`; use `rust-lld` + ICF. This is 55,808 B smaller than
  2.10.7 and 54,272 B below 596,992. Event-gated reads are unchanged.
- Isolated 60 s sample: 2.10.7 was 31.25 ms idle / 125 ms armed CPU, 25/s /
  140/s process context switches, and 9/s / 126/s on the sprint thread. No
  speedup claim.

## 2.10.7 — 2026-11-12

- Portable exe: 598,528 B,
  `88348e007a5b10fc1921509f7f8f8d9dd159f51adf9fbe93b1e451a5de6dea3f`.
- Tests: 26 / 26.
- Changes: sprint settings are read only when a pulse is due; crosshair
  settings only on a magenta flash. Poll cadence and bindings are unchanged.
  Same size as 2.10.6; no latency or size claim.

## 2.10.6 — 2026-11-11

- Portable exe: 598,528 B,
  `9452d73bf5d4126b27d2d350eeec0b2edca78ab65b7e4137b04a0045660bb6d5`.
- Tests: 26 / 26.
- Changes: the sprint watcher parks at 100 ms while inactive and unparks on
  focus/settings changes; armed cadence stays 8 ms. No latency claim. The exe
  is 1,536 B larger than 2.10.5; no size win.

## 2.10.5 — 2026-11-11

- Portable exe: 596,992 B,
  `49d55d7a77408ef9c5b9b9a6291351bda1796521626738a90f05b3ab854fc850`.
- Tests: 25 / 25.
- Changes: sprint/crosshair watcher gates cached; inactive loops skip the
  settings snapshot and lock. Cadence and reset behavior unchanged. No size or
  latency claim; artifact is 512 B larger than 2.10.4.

## 2.10.4 — 2026-11-11

- Portable exe: 596,480 B,
  `c9732c9c5041274d3a72e0b89ff855801966d24ec91ab9cfb3b70cc4092922f2`.
- Tests: 25 / 25.
- Changes: transparent brand mark with no favicon tile or plate CSS; cached
  focus-only gate avoids a settings lock in the presence loop. Trigger routing
  and 8 px corners are unchanged. No zero-latency claim.

## 2.10.3 — 2026-11-11

- Portable exe: 596,992 B,
  `c78625faa251ff71934e6e2d08a06b13b8280deb1c247a27536199e039cc9613`.
- Tests: 25 / 25.
- Changes: DWM-rounded transparent corners are click-through; the presence
  watcher reuses an already-verified foreground game HWND. 8 px radius and
  captures are unchanged. No latency speedup is claimed.

## 2.10.2 — 2026-11-11

- Portable exe: 595,968 B,
  `7fe0b990119f9e07a5dfb3a7a895fa1ba3aa1a8028e6cfee4570fe85884f7e1d`.
- Tests: 24 / 24.
- Changes: DWM-first antialiased corner with GDI region fallback; favicon and
  site marks matched to the app's outlined droplet. 8 px radius unchanged.

## 2.10.1 — 2026-11-11

- Portable exe: 595,968 B,
  `239792e66f5684f6ecfde7f7c5395e5bb11f8870d50d89f8288b262ba841d696`.
- Tests: 24 / 24.
- Changes: process-verified game detection; rejects browser, chat, editor, and
  launcher titles; two-sample confirmation; 8-tick cold discovery. Anchor and
  8 px surface unchanged from 2.10.0.

## 2.10.0 — 2026-11-11

- Portable exe: 593,408 B,
  `6b43e15ebf6ad4e7ca0cb10b0be085acfa2b47839faf6dcb15a7c1fc52a9f0ab`.
- Tests: 20 / 20.
- Changes: Hydrogen `A6` Anchor sequence; Anchor 47–97 / 53–103 with 47/53
  defaults; Air loop; 8 px shared corner; vault-graph site.

## 2.9.3 — 2026-11-11

- Portable exe: 591,872 B,
  `288b1019d61e46805fff31737ab926c52bc20541a53ef05841c8b356a27f5873`.
- Single direct executable; no MSIX/AppX, installer, or ZIP.
- Tests: 19 / 19.
- Changes: Anchor 55/65 ms safe settle; top accent line removed; packaging
  retired; wider Anchor domains remain independent.

## 2.9.2 — 2026-11-11

- Portable exe: 591,872 B,
  `2d4a915ca68b3c4ef944fe208b593f3c816f55b8b605a478d23a5cfd5d70cc46`.
- MSIX: 301,609 B,
  `94d47fd44f092b571ade122b2d020011da36b41cd185517895f28133040e6c87`.
- Portable ZIP: 285,751 B,
  `928f816eb2dc4fa028273a9af9f8236e18a2e6c4db78ee612bd6a8d9fb2f5841`.
- Tests: 19 / 19.
- Package: 2.9.2.0, signature Valid, installed Ok.
- Changes: independent sliders; 28×16 knob switch; Anchor 37–97 / 43–103,
  defaults 37/43; fixed Anchor sleeps removed; outer rim removed; Self-destruct
  label restored.

## 2.9.1 — 2026-11-11

- Portable exe: 592,384 B,
  `c0b5c64793d3a94f70c4141903715f58a3458665db3c0db5e51ca38ae297fe8a`.
- MSIX: 301,774 B,
  `92768f518581cfd2662f8e83506ec27127742833ea6d995b32d507a5dc3f3045`.
- Portable ZIP: 285,770 B,
  `8be91f12560842686ecf306b48f4026a78abef9358d79f4a1ee1a6e2c3d36920`.
- Tests: 19 / 19.
- Package: 2.9.1.0, signature Valid, installed Ok.
- Changes: module glyphs and card outlines removed; text ON/OFF pills; Anchor
  floor row retired; exe 9,216 bytes smaller.

## 2.9.0 — 2026-11-11

- Portable exe: 601,600 B,
  `3c40feceec36e830cbf571614372b41f68c96bc2db417bf1c3d2fa8822db5dd9`.
- MSIX: 306,082 B,
  `c569446ec120c8548b009e4714d3e6d3b3df5001fa4a51ae2bf60f6c2f84a215`.
- Portable ZIP: 289,296 B,
  `251932da68bbe2c27f5f3670a12f1ea2a454fda7ecb157227ae1c2a66e15e47d`.
- Tests: 19 / 19.
- Package: 2.9.0.0, signature Valid, installed Ok.

Download: <https://fuvji.github.io/droplet/releases.html>

## Boundaries

- No Linux/macOS artifacts until a real port exists.
- No silent auto-updater.
- No streamproof/anti-cheat bypass/input disguise.
- Portable exe is unsigned; MSIX uses the development certificate.

See [[performance]], [[website]], and [[droplet]].
