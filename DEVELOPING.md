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
release payload, then writes and signs `apps/Droplet/Droplet.msix`. The portable
copy is `apps/Droplet/Droplet.exe`. Install with `Install-Droplet.cmd`; a new
machine must trust the development certificate in
`LocalMachine\TrustedPeople` once.

## Site

The public page is static and has no client-side JavaScript, external fonts,
third-party requests, analytics, or network calls. Preview it with:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/serve-site.ps1
```

Open <http://127.0.0.1:4173/>. The production manifest is `site/deploy.json`;
`site/README.md` documents deployment and custom-domain requirements.

## Verification

The release currently carries 18 tests:

```powershell
cargo test --release
```

They cover config migration, timing bounds, RGB parsing, trigger-mode
normalization and latch generations, Anchor slot validation, W-Tap planning,
Sprint's 100 ms deadline, cadence adaptation, profile parsing, and trigger-index
equivalence plus its routing benchmark.

Before publishing, also check:

- `git diff --check`
- every local `href`/`src` in `site/public` resolves
- all four captures are 624×408
- the production page has no console or CSP errors
- the MSIX signature is valid

## Repository

<https://github.com/fuvji/droplet>

The Freebuff Sites project ID is in `site/SITE_ID.txt`. Do not put credentials
or signing private keys in the repository; the local development certificate
and SDK cache are ignored by Git.

## Design rules

- Native first: no crates, no WebView, no runtime downloader.
- Bounded work: every watcher, queue, and wait has a hard limit.
- Event-driven UI: idle frames are not scheduled.
- One-shot sequences use RAII busy guards; Sprint is the only repeating watcher.
- Inputs go through Win32 `SendInput` only; no game-memory access.
- Config writes are atomic and old Hydrogen/AHK settings migrate once.
- No capture concealment, process hiding, scan evasion, or zero-latency claim.
