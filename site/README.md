# Hydrogen website

The public product page is a static site deployed through Freebuff Sites:

<https://site-f9270d9239294ae0b43816a6d295ef6a.freebuff.page>

## Layout

- `public/` contains the HTML, CSS, favicon, robots file, and real 2.4.0
  interface capture.
- `worker.js` serves `env.ASSETS` and adds conservative security headers.
- `deploy.json` is the Freebuff Sites deployment manifest.
- `SITE_ID.txt` records the Freebuff Sites project ID.

The page has no external fonts, third-party images, scripts, analytics, or
network calls. The only image is the local panel capture at
`public/assets/hydrogen-panel.png`.

Hydrogen is proprietary. The site discloses that source code and build scripts
are not distributed; it does not present the product as open source.

## Deploy

From the repository root, deploy the manifest through Freebuff Sites:

```text
site/deploy.json
```

The project's preview and production environments are independent. Publish the
production environment explicitly after a successful deploy.
