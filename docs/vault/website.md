# website

Canonical: <https://fuvji.github.io/droplet/>

## Pages

- `index.html` — downloads, benchmark, previews, modules, audit, evidence vault,
  specs, sponsors, FAQ.
- `releases.html` — detailed release ledger and artifact table.
- `sponsors.html` — direct sponsor inventory.
- `downloads.js` — local-only OS hint; no storage, no network.
- `downloads/` — direct exe, README, and SHA-256 list; no ZIP or MSIX.

## Visual system

Graphite, warm white, one thin accent hairline. The brand mark and favicon are
transparent outlined droplets: no plate, border, background, or shadow. Module
cards are borderless with text ON/OFF pills and no module glyphs. Oversized
display type, thin rules, generous section spacing, restrained hover motion, 3D
preview tilt. Reduced motion disables transforms.

## Honesty rules

- Linux/macOS cards say “port tracked”, never “download”.
- The routing benchmark is labelled as routing only.
- No streamproof/anti-cheat claims.
- Founder credit: `@epufk`.

## Motion and cold-load audit

Measured 2026-11-12 with Helium 0.21.2.1 / Chromium 157 headless at
390×844 CSS px, DPR 3, 4× CPU throttle, cache disabled.

- Narrow screens now group card reveals into their section containers:
  53 animated targets fall to 23 on the index. Desktop keeps the per-card
  stagger.
- Warm live result: full motion 688 ms FCP, 219 ms longest frame, scroll
  p95 23.3 ms, 2 frames over 25 ms, 0 over 50 ms. Reduced motion: 480 ms
  FCP, 180 ms longest frame, scroll p95 10 ms. CLS is 0 in both; all 23
  reveals reach visible state.
- The first document in a fresh browser is still a separate cost: the
  minimal local probe is 137 ms, while the full local site is about 2.1 s
  with a 1.37–1.49 s frame. TTFB is only 63–72 ms and all CSS/JS finish by
  about 167 ms. The live cold first document measured 2.60 s with 309 ms
  TTFB, versus 0.69 s warm.
- Ablations: removing `motion.js`, removing the ambient/backdrop layers, and
  disabling motion CSS did not remove the cold cost. `content-visibility`
  helped a reduced-motion cold probe (2.09 s to 1.15 s) but not the full
  path, so it was not shipped. The remaining cost is first-document
  layout/raster, not Pages transfer or the reveal layer.
- Limit: 4× CPU headless emulation is not a physical phone. Cold samples are
  variable; warm results are the stable comparison.

See [[release]] and [[droplet]].
