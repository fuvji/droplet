# Droplet — agent instructions

Read this before changing the project. Droplet is a proprietary, public-source
Windows application for local keyboard-and-mouse macros. Keep it small, honest,
and visibly local.

## Build and verify

```bash
cargo test --release
cargo build --release
```

The release uses fat LTO, one codegen unit, panic-abort, stripped symbols, and
the `x86_64-pc-windows-gnu` target from `.cargo/config.toml`. Packaging:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/build-msix.ps1
```

Outputs are lowercase: `apps/droplet/droplet.exe` and
`apps/droplet/droplet.msix`. The site is static in `site/public/`; preview it
with `tools/serve-site.ps1` and publish the directory to `gh-pages`.

Before claiming a release is done, run the tests, rebuild the exe/MSIX,
regenerate the four captures, check the site, and record the new SHA-256 values.
Never claim a speedup without a measurement.

## Product boundaries

- No game-memory reads or writes.
- No network calls, telemetry, analytics, silent auto-updater, or runtime
  downloader.
- No streamproof/capture-evasion mode, process hiding, scan evasion, log
  erasure, anti-cheat bypass, input-device disguise, or privilege escalation.
- No zero-latency, “quantum speed”, or “100% no restriction” claim.
- No Linux/macOS package until a real platform port exists.
- “Minecraft Zero Macro” is a product name, not a latency promise.

If a request conflicts with these boundaries, refuse the conflicting part and
continue with the legitimate work. Do not implement evasion behind a hidden
flag.

## UI rules

- One logical `CORNER_RADIUS` across paint, window region, and capture mask.
- Event-driven paint; no idle animation timer.
- Icons are original GDI+ line art with transparent backgrounds. No Mojang
  textures.
- Round power controls; one accent hairline; warm white on graphite.
- Keep the panel at 624×408 logical pixels unless the layout is reworked and
  recaptured.

## Site rules

- Canonical origin is `https://fuvji.github.io/droplet/`.
- No third-party scripts, fonts, analytics, cookies, or ad clients.
- One local script (`downloads.js`) may detect the OS for download cards.
- Keep Linux/macOS download cards honest: a port is tracked, not shipped.
- Include real download links, SHA-256 checksums, and a release ledger.
- Credit the founder, `@epufk`.

See `docs/vault/` for the portable knowledge graph and `releases/` for the
detailed per-version notes.
