# Hydrogen website

The public product page is a static site deployed through Freebuff Sites:

<https://site-f9270d9239294ae0b43816a6d295ef6a.freebuff.page>

Source: <https://github.com/fuvji/hydrogen>

## Layout

- `public/` contains the HTML, CSS, favicon, sitemap, robots file, and real
  Core-tab interface capture.
- `worker.js` serves `env.ASSETS`, sets a strict CSP and cache policy.
- `deploy.json` is the Freebuff Sites deployment manifest.
- `SITE_ID.txt` records the Freebuff Sites project ID.

The 2.4.1 page uses a dark violet/block visual system, with measured release
facts, native-size media, timing diagrams, a module index, an audit panel, and
an explicit distribution disclosure.

The page has no external fonts, third-party images, scripts, analytics, or
network calls. The only image is the local panel capture at
`public/assets/hydrogen-panel.png`.

Hydrogen is proprietary. The source and build scripts are published in the
public GitHub repository for transparency, but public availability is not an
open-source grant. The site does not present Hydrogen as open source or as
streamproof.

## Deploy

From the repository root, deploy the manifest through Freebuff Sites:

```text
site/deploy.json
```

The project's preview and production environments are independent. Publish the
production environment explicitly after a successful deploy.

## Local preview

From the repository root, the dependency-free loopback server serves
`public/` for layout and asset checks:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/serve-site.ps1
```

Open <http://127.0.0.1:4173/>. This server is local-only; it does not emulate
the worker's security headers or the production cache policy.
