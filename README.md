# Hydrogen

Hydrogen is a low-latency, zero-dependency Rust macro suite for Windows and
Minecraft Java. It runs as a tray-native overlay and injects only synthetic
keyboard/mouse input; it does not read or modify game memory.

## Features

- Normal Anchor: place, charge, switch off glowstone, detonate.
- Air Anchor: prime once, then reuse the floating anchor and pop-and-place the
  next anchor in the same client tick.
- Crystal, Mace, Dash, Shield, Potion, Pearl, Totem, Breach, Fast Use, Auto Use,
  W-Tap, Sprint, and Hover Inventory modules.
- 648x432 acrylic panel, event-driven rendering, Low Latency mode, adaptive
  local timing, an optional offline combat timing profile, and AHK-compatible
  INI migration.
- Portable x64 executable plus a signed full-trust MSIX package.

## Combat timing profile

`tools/combat-lab.ps1` turns timestamped, human-confirmed hits from combat
footage into a compact local timing prior. `tools/combat-annotator.html` is a
local-only video page that exports the CSV; it never uploads the clip. Hydrogen
reads `%LOCALAPPDATA%\Hydrogen\combat.profile` once at startup, blends it with
the local cadence brain, and clamps it to the configured Min/Max. Delete the
file to disable it. The runtime never reads video or the network.

See [docs/COMBAT_LAB.md](docs/COMBAT_LAB.md) for the CSV contract and safety
rules, and [docs/AI.md](docs/AI.md) for the research boundary.

## Deliberate boundaries

Hydrogen is designed to stay small, auditable, and predictable:

- No crates, network calls, telemetry, updater, or runtime downloader.
- No camera, WebView, game-memory access, or external AI/LLM integration.
- Vision is limited to the existing 1x1 magenta crosshair sample used by the
  totem watcher.
- Input hooks and the safety gate stop macros when focus leaves Minecraft (or
  when focus-only mode is disabled).
- No process hiding, scan evasion, self-erasure, or disguised executable.

## Build

```bash
rustup target add x86_64-pc-windows-gnu
cargo test --release
cargo build --release
```

The release profile uses `opt-level = 3`, fat LTO, one codegen unit, aborting
panics, and stripped symbols. The project targets
`x86_64-pc-windows-gnu` via `.cargo/config.toml`.

## Package and install

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/build-msix.ps1
```

The script stages the manifest/assets/payload, packs and signs the MSIX with the
project-local signing tools, and writes the portable executable to
`apps/Hydrogen/Hydrogen.exe`. `Install-Hydrogen.cmd` installs the package and
launches it from the Start menu.

The development certificate is self-signed. Installing the package on another
machine requires trusting that certificate in `LocalMachine\TrustedPeople`.

## Configuration

Settings live in `%LOCALAPPDATA%\Hydrogen\config.ini`. Existing AHK
`%TEMP%\settings.dat` files are imported once using the same `v0`/`v6` key
names. Config writes are atomic and an invalid slot combination or an inactive
master/focus gate prevents injection.

## Verification

- `cargo test --release`: 12 tests covering config migration, timing bounds,
  loop generation, W-Tap planning, Air Anchor state, and combat-profile parsing.
- Air Anchor live key trace: first cycle `h,g,h,r`; normal Anchor `h,g,r`.
- Mouse trace: pop and replacement right-clicks occur at the same timestamp.
- Source version: `2.3.8`; installed package: `2.3.7.0`, status `Ok`.

Real in-game hit registration still needs verification in a live Minecraft
client; input injection and timing are verified independently.

## License

MIT.
