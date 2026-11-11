# Droplet website

The product site is published from the `gh-pages` branch at:

<https://fuvji.github.io/droplet/>

Source: <https://github.com/fuvji/droplet>

## Layout

- `public/index.html` — product page: downloads, benchmark, previews, modules,
  audit, evidence vault, specs, sponsor inventory, FAQ, founder credit.
- `public/releases.html` — detailed 2.9.1 release ledger and version history.
- `public/sponsors.html` — direct sponsor inventory.
- `public/style.css` — base visual system.
- `public/refine.css` — editorial layer, download/vault cards, 3D previews,
  release ledger styles, and reduced-motion fallbacks.
- `public/downloads.js` — local-only OS hint for the download cards. It sends
  nothing, stores nothing, and has no third-party imports.
- `public/downloads/` — portable ZIP, signed MSIX, README, and SHA-256 list.
- `public/assets/` — four native 624×408 captures, the transparent icon sprite,
  and the transparent favicon/mark assets.
- `public/sitemap.xml`, `robots.txt`, `llms.txt` — canonical GitHub Pages URLs.

## Release artifacts

`droplet-windows-x64.zip` (285,770 B) contains the unsigned portable
`droplet.exe`, the license, README, and checksums. `droplet-windows-x64.msix`
(301,774 B) is signed with the project development certificate. Linux and
macOS packages are not published because the Win32/GDI+ core has not been
ported.

The site is explicit about this. It does not relabel a Windows binary as a
Linux or macOS build, and it does not offer a streamproof or anti-cheat bypass
mode.

## Local preview

From the repository root:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/serve-site.ps1
```

Open <http://127.0.0.1:4173/>. The server is loopback-only.

## Deploy

Copy `site/public/` to the `gh-pages` branch and push it. Keep the download
artifacts in `public/downloads/`; GitHub Pages has no build step. After a push,
check the canonical URL, the download ZIP, the MSIX, the checksum file,
`releases.html`, and the module-icon sprite on the public origin.

Do not publish signing private keys, environment files, or source maps.
