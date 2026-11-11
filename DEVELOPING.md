# Developing Droplet

Internal build and maintenance notes. Droplet is proprietary software: this
document is not a licence to reuse the source. See [LICENSE](LICENSE).

## Build

Requirements: a recent stable Rust toolchain, the
`x86_64-pc-windows-gnu` target, Windows 10/11, and the project-local signing
tools used by `scripts/build-msix.ps1`.

```powershell
rustup target add x86_64-pc-windows-gnu
cargo test --release
cargo build --release
```

The release binary is written to
`target/x86_64-pc-windows-gnu/release/droplet.exe`. The project has no crate
dependencies; `.cargo/config.toml` selects the GNU target and self-contained
linking.

## Package

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/build-msix.ps1
```

The script stages `packaging/AppxManifest.xml`, the package assets, and the
release payload, then writes and signs `apps/droplet/droplet.msix`. The portable
copy is `apps/droplet/droplet.exe`. Install with `Install-Droplet.cmd`; a new
machine must trust the development certificate
(`apps/droplet/droplet-dev.cer`) in `LocalMachine\TrustedPeople` once.

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
release artifacts and site layout.

## Verification

The release currently carries 19 tests:

```powershell
cargo test --release
```

They cover config migration, timing bounds, RGB parsing, trigger-mode
normalization and latch generations, the allocation-free binding lookup, Anchor
slot validation, W-Tap planning, Sprint's 100 ms deadline, cadence adaptation,
profile parsing, and trigger-index equivalence plus its routing benchmark.

Before publishing, also check:

- `git diff --check`
- every local `href`/`src` in `site/public` resolves
- all four captures are 624×408
- the product page, `releases.html`, and `sponsors.html` have no console errors
- every download and checksum link resolves
- the MSIX signature is valid

## Repository

<https://github.com/fuvji/droplet>

Downloads are generated from the release build and published under
`site/public/downloads/`. The tag workflow in `.github/workflows/release.yml`
builds a portable Windows ZIP and publishes it with the notes in `releases/`.
Do not put credentials or signing private keys in the repository; the local
development certificate and SDK cache are ignored by Git.

## Design rules

- Native first: no crates, no WebView, no runtime downloader.
- Bounded work: every watcher, queue, and wait has a hard limit.
- Event-driven UI: idle frames are not scheduled.
- One-shot sequences use RAII busy guards; Sprint is the only repeating watcher.
- Inputs go through Win32 `SendInput` only; no game-memory access.
- Config writes are atomic and old Hydrogen/AHK settings migrate once.
- No capture concealment, process hiding, scan evasion, or zero-latency claim.
