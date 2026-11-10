const SECURITY_HEADERS = {
  "Content-Security-Policy":
    "default-src 'self'; style-src 'self'; img-src 'self' data:; media-src 'self'; font-src 'self'; script-src 'none'; connect-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'",
  "Cross-Origin-Opener-Policy": "same-origin",
  "Cross-Origin-Resource-Policy": "same-origin",
  "Permissions-Policy": "camera=(), microphone=(), geolocation=(), payment=()",
  "Referrer-Policy": "strict-origin-when-cross-origin",
  "Strict-Transport-Security": "max-age=31536000",
  "X-Content-Type-Options": "nosniff",
  "X-Frame-Options": "DENY",
};

function cacheControl(pathname) {
  if (pathname.startsWith("/assets/")) {
    return "public, max-age=86400, stale-while-revalidate=604800";
  }
  if (pathname === "/style.css" || pathname === "/favicon.svg" || pathname === "/sitemap.xml") {
    return "public, max-age=3600, stale-while-revalidate=86400";
  }
  return "public, max-age=0, must-revalidate";
}

export default {
  async fetch(request, env) {
    const response = await env.ASSETS.fetch(request);
    const headers = new Headers(response.headers);
    const pathname = new URL(request.url).pathname;

    for (const [name, value] of Object.entries(SECURITY_HEADERS)) {
      headers.set(name, value);
    }
    headers.set("Cache-Control", cacheControl(pathname));

    return new Response(response.body, {
      status: response.status,
      statusText: response.statusText,
      headers,
    });
  },
};
