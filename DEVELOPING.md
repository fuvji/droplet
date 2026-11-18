# Developing Droplet

Internal build and maintenance notes. Droplet is proprietary software: this
document is not a licence to reuse the source. See [LICENSE](LICENSE).

## Build

Requirements: the pinned GNU-hosted Rust 1.99.1 toolchain and Windows 10/11.
The `rust-toolchain.toml` file installs the `x86_64-pc-windows-gnu` target and
`rustfmt` automatically. Use the GNU host, not an MSVC host with a GNU target;
the GNU host carries the self-contained Windows import libraries.

```powershell
cargo test --release
powershell -NoProfile -ExecutionPolicy Bypass -File tools/build-release.ps1
```

The release wrapper writes
`target/x86_64-pc-windows-gnu/release/droplet.exe`. It remaps the workspace and
target paths to `/droplet` and `/target`, builds the GNU target, and zeroes the
PE timestamp. Two clean builds in different target directories must produce
SHA-256
`a7061c54a2f5379e1e7df32dcb0b52b73b9e275a6034b1e9066dfec9c0ded31a`.
The project has no crate dependencies; `.cargo/config.toml` selects the GNU
target, bundled `rust-lld`, self-contained linking, and ICF. Copy that one file
to `apps/droplet/droplet.exe` and `site/public/downloads/droplet.exe` for
distribution. There is no MSIX, AppX, installer, or signing step.

## Site

The public page is static. One local script (`site/public/downloads.js`) detects
the visitor's OS for the download cards; it stores nothing and sends nothing.
There are no third-party scripts, external fonts, analytics, ad clients, or
network calls. Preview it with:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/serve-site.ps1
```

Open <http://127.0.0.1:4173/>. GitHub Pages serves the contents of
`site/public/` from the `gh-pages` branch. `site/README.md` documents the
release artifact and site layout.

## Verification

The release currently carries **41 / 41** tests:

```powershell
cargo test --release
```

They cover config migration, timing bounds, RGB parsing, trigger-mode
normalization and latch generations, Auto Crystal's Return default and
hold-vs-tap policy, busy-safe return ordering, stale-cycle cancellation,
physical rail hit testing at 125%/150%, the allocation-free binding lookup,
rebinding conflict ownership, load-time duplicate cleanup, `XButton1`/Anchor
routing, captured-key round-trips, Anchor slot validation, W-Tap planning, Sprint's 100 ms deadline,
cadence adaptation, profile parsing, and trigger-index equivalence plus its
routing benchmark, and the timer target floor.

Before publishing, run the release consistency checker:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/check-release.ps1
```

It verifies the version in `Cargo.toml`/`Cargo.lock`, checksum and size for both
copied executables, SHA256SUMS, the workflow hash, the test count claimed in
docs, all four 624×408 captures, and local site links. It also runs
`cargo test --release`. A release check should not use `-SkipTests`.

## Input harness

The input harness is a local Windows integration test, not part of the shipped
panel. It needs an interactive desktop because it installs `WH_KEYBOARD_LL` and
`WH_MOUSE_LL` hooks in a hidden message loop and injects one labelled trigger.
It does not use game memory, the network, or a live Minecraft client.

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools/input-harness.ps1
```

It starts Droplet with a temporary `LOCALAPPDATA` and a seeded config, then
records only events carrying Droplet's `dwExtraInfo` marker. The three cases
are: focus-only gate on with no game (no macro output), master off with the
gate open (no macro output), and the normal Shield sequence (exactly `h`
down/up followed by left click down/up). The harness cleans up the process and
profile and exits non-zero on any mismatch. Use `-OutFile` to save the complete
JSON event record.

Hosted CI does not run this harness: a GitHub-hosted Windows runner has no
interactive desktop or foreground game session. The release workflow covers
formatting, unit tests, the reproducible artifact, and attestation instead.

Captures use a temporary `LOCALAPPDATA` and seeded config; the caller's real
settings are never read or changed. This keeps the published four-image set
reproducible and lets the Crystal capture demonstrate Anchor owning `XButton1`
without carrying a duplicate Breach binding.

Also check:

- `git diff --check`
- `cargo fmt --check` is clean
- the product page, `releases.html`, and `sponsors.html` have no console errors
- the direct `droplet.exe` and `SHA256SUMS.txt` links resolve on the published
  Pages URL, not only locally

## Final handoff

For a complete release pass, run these in order from the repository root:

```powershell
cargo fmt --check
cargo test --release
powershell -NoProfile -ExecutionPolicy Bypass -File tools/build-release.ps1
Copy-Item target/x86_64-pc-windows-gnu/release/droplet.exe apps/droplet/droplet.exe
Copy-Item target/x86_64-pc-windows-gnu/release/droplet.exe site/public/downloads/droplet.exe
powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/capture-tabs.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/input-harness.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/check-release.ps1
```

The capture helper uses a temporary profile, so it does not read or rewrite the
developer's personal `%LOCALAPPDATA%\Droplet` settings. Update the release note,
site metadata, checksums, workflow hash, and the copied executables whenever the
version or artifact changes. Preview locally with `tools/serve-site.ps1`; publish
only the contents of `site/public/` to `gh-pages`.

## Repository

<https://github.com/fuvji/droplet>

The release artifact is generated from the release build and published under
`site/public/downloads/`. The tag workflow in `.github/workflows/release.yml`
installs the pinned toolchain, checks formatting and tests the GNU target,
builds through the reproducible wrapper, verifies the normalized hash, attests
the exe and checksum, and uploads those files plus `droplet.intoto.jsonl` with
the notes in `releases/`. The checked-in exe is only a comparison target; the
workflow never publishes it directly. Do not put credentials or signing private
keys in the repository.

## Design rules

- Native first: no crates, no WebView, no runtime downloader.
- Bounded work: every watcher, queue, and wait has a hard limit.
- Event-driven UI: idle frames are not scheduled.
- One-shot sequences use RAII busy guards; Sprint is the only repeating watcher.
- Inputs go through Win32 `SendInput` only; no game-memory access.
- Config writes are atomic and old Hydrogen/AHK settings migrate once.
- No capture concealment, process hiding, scan evasion, or zero-latency claim.
