# Developing Droplet

Internal build and maintenance notes. Droplet is proprietary software: this
document is not a licence to reuse the source. See [LICENSE](LICENSE).

## Build

Requirements: a recent stable Rust toolchain, the
`x86_64-pc-windows-gnu` target, and Windows 10/11.

```powershell
rustup target add x86_64-pc-windows-gnu
cargo test --release
cargo build --release
```

The release binary is written to
`target/x86_64-pc-windows-gnu/release/droplet.exe`. The project has no crate
dependencies; `.cargo/config.toml` selects the GNU target and self-contained
linking. Copy that one file to `apps/droplet/droplet.exe` and
`site/public/downloads/droplet.exe` for distribution. There is no MSIX, AppX,
installer, or signing step.

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

The release currently carries **37 / 37** tests:

```powershell
cargo test --release
```

They cover config migration, timing bounds, RGB parsing, trigger-mode
normalization and latch generations, Auto Crystal's Return default and
hold-vs-tap policy, busy-safe return ordering, stale-cycle cancellation,
physical rail hit testing at 125%/150%, the allocation-free binding lookup,
Rebinding conflict ownership, `XButton1`/Anchor routing, captured-key
round-trips, Anchor slot validation, W-Tap planning, Sprint's 100 ms deadline,
cadence adaptation, profile parsing, and trigger-index equivalence plus its
routing benchmark.

Before publishing, run the release consistency checker:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/check-release.ps1
```

It verifies the version in `Cargo.toml`/`Cargo.lock`, checksum and size for both
copied executables, SHA256SUMS, the workflow hash, the test count claimed in
docs, all four 624×408 captures, and local site links. It also runs
`cargo test --release`. A release check should not use `-SkipTests`.

Also check:

- `git diff --check`
- the product page, `releases.html`, and `sponsors.html` have no console errors
- the direct `droplet.exe` and `SHA256SUMS.txt` links resolve on the published
  Pages URL, not only locally

## Repository

<https://github.com/fuvji/droplet>

The release artifact is generated from the release build and published under
`site/public/downloads/`. The tag workflow in `.github/workflows/release.yml`
builds and uploads the single Windows executable and its checksum with the notes
in `releases/`. Do not put credentials or signing private keys in the repository.

## Design rules

- Native first: no crates, no WebView, no runtime downloader.
- Bounded work: every watcher, queue, and wait has a hard limit.
- Event-driven UI: idle frames are not scheduled.
- One-shot sequences use RAII busy guards; Sprint is the only repeating watcher.
- Inputs go through Win32 `SendInput` only; no game-memory access.
- Config writes are atomic and old Hydrogen/AHK settings migrate once.
- No capture concealment, process hiding, scan evasion, or zero-latency claim.
