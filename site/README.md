# Droplet website

The public product page is a static site deployed through Freebuff Sites:

<https://site-f9270d9239294ae0b43816a6d295ef6a.freebuff.page>

Source: <https://github.com/fuvji/hydrogen>

## Layout

- `public/` contains the HTML, CSS, favicon, sitemap, robots file, sponsor
  inventory, and real 2.6.0 captures from all four panel tabs.
- `worker.js` serves `env.ASSETS` and adds a strict CSP and cache policy.
- `deploy.json` is the Freebuff Sites deployment manifest.
- `SITE_ID.txt` records the Freebuff Sites project ID.

The 2.6.0 page uses a cyan ion-lab visual system with measured trigger-routing
facts, a benchmark chart, a runtime pipeline model, native capture crops,
CSS-only motion, an audit, a hard-boundaries section, and direct sponsor
inventory.

The page has no external fonts, third-party images, analytics, ad clients,
cookies, scripts, or network calls. The images are four distinct local captures
at `public/assets/droplet-core.png`, `droplet-crystal.png`,
`droplet-assist.png`, and `droplet-system.png`. `sponsors.html` uses the same
stylesheet and no external resources.

The capture lab deliberately does not present fabricated gameplay footage. The
page explains that live Minecraft hit registration is still unverified and
invites a real clip through the issue link. The motion preview is a CSS study of
the native Assist-tab capture; the preview tabs are four separate screenshots.

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
