# Droplet website

The public product page is a static site deployed through Freebuff Sites:

<https://site-f9270d9239294ae0b43816a6d295ef6a.freebuff.page>

Source: <https://github.com/fuvji/droplet>

## Layout

- `public/` contains the HTML, CSS, transparent favicon, sitemap, robots file,
  plain-text `llms.txt`, sponsor inventory, native captures from all four panel
  tabs, and the silent UI reel.
- `worker.js` serves `env.ASSETS` and adds a strict CSP and cache policy.
- `deploy.json` is the Freebuff Sites deployment manifest.
- `SITE_ID.txt` records the Freebuff Sites project ID.

The 2.6.2 page uses a graphite/white acrylic visual system with measured
trigger-routing facts, a benchmark chart, a runtime pipeline model, native
capture crops, a silent four-tab UI reel, an audit, a hard-boundaries section,
and direct sponsor inventory.

The page has no external fonts, third-party images, analytics, ad clients,
cookies, scripts, or network calls. The images are four distinct local captures
at `public/assets/droplet-core.png`, `droplet-crystal.png`,
`droplet-assist.png`, and `droplet-system.png`. `sponsors.html` uses the same
stylesheet and no external resources.

The capture lab deliberately does not present fabricated gameplay footage. The
page explains that live Minecraft hit registration is still unverified and
invites a real clip through the issue link. The capture lab is a silent WebM reel that crossfades
the four native tab captures; the preview tabs are the same four separate
screenshots, never one repeated image.

Droplet is proprietary. The source and build scripts are published in the
public GitHub repository for transparency, but public availability is not an
open-source grant. The site does not present Droplet as open source or as
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

## Custom domain

A short root domain such as `droplet.net` requires registrar and DNS control.
`droplet.net` is already registered, so it cannot be claimed from this
repository. To connect a domain you do control:

1. Point the apex `A`/`AAAA` records and `www` `CNAME` at the Freebuff Sites
   target through the registrar.
2. Add the domain in the Sites dashboard and verify ownership there.
3. Update the canonical URL, Open Graph image URL, `robots.txt`, `sitemap.xml`,
   and `llms.txt` together so search and answer engines see one origin.

Do not add a `CNAME` file or a fake TLS configuration here; the hosting
provider must issue the certificate and route the hostname.
