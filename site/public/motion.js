// Droplet interaction layer. No network, storage, cookies, or tracking.
(() => {
  "use strict";

  const root = document.documentElement;
  root.classList.add("js");

  const reducedQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
  const reduceMotion = () => reducedQuery.matches;

  const progress = document.createElement("div");
  progress.className = "read-progress";
  progress.setAttribute("aria-hidden", "true");
  document.body.appendChild(progress);

  const header = document.querySelector(".site-header");
  const navLinks = Array.from(document.querySelectorAll(".site-nav a")).filter((link) => {
    try {
      const url = new URL(link.href, window.location.href);
      return url.pathname === window.location.pathname && url.hash.length > 1;
    } catch {
      return false;
    }
  });
  const sections = navLinks
    .map((link) => document.querySelector(link.hash))
    .filter(Boolean)
    .sort((a, b) => {
      const relation = a.compareDocumentPosition(b);
      if (relation & Node.DOCUMENT_POSITION_FOLLOWING) return -1;
      if (relation & Node.DOCUMENT_POSITION_PRECEDING) return 1;
      return 0;
    });

  function updateActiveSection() {
    if (!sections.length) return;

    const marker = window.scrollY + window.innerHeight * 0.38;
    let current = null;

    for (const section of sections) {
      if (section.getBoundingClientRect().top + window.scrollY <= marker) {
        current = section;
      }
    }

    for (const link of navLinks) {
      const active = current && link.hash === `#${current.id}`;
      link.classList.toggle("is-active", Boolean(active));
      if (active) link.setAttribute("aria-current", "true");
      else link.removeAttribute("aria-current");
    }
  }

  let scrollQueued = false;

  function paintScroll() {
    const scrollTop = window.scrollY || window.pageYOffset || 0;
    const scrollable = Math.max(1, document.documentElement.scrollHeight - window.innerHeight);
    updateActiveSection();
    progress.style.setProperty("--read-progress", Math.min(1, scrollTop / scrollable).toFixed(4));
    if (header) header.classList.toggle("is-scrolled", scrollTop > 12);
    scrollQueued = false;
  }

  function queueScroll() {
    if (scrollQueued) return;
    scrollQueued = true;
    window.requestAnimationFrame(paintScroll);
  }

  window.addEventListener("scroll", queueScroll, { passive: true });
  window.addEventListener("resize", queueScroll, { passive: true });
  // Do not read section geometry during script execution. The first frame
  // paints after layout, when the browser can answer these reads without a
  // forced synchronous layout.
  queueScroll();

  // On phones, reveal each card group as one surface. Animating 40–50
  // individual cards costs more scroll frames than the grouping adds in
  // visual rhythm. Desktop keeps the finer per-card stagger.
  const narrowQuery = window.matchMedia("(max-width: 760px)");
  const groupedRevealSelector = [
    ".proof-rail",
    ".partner-rail",
    ".section-head",
    ".benchmark-card",
    ".performance-copy",
    ".measure-card",
    ".pipeline-card",
    ".media-frame",
    ".preview-lab",
    ".anatomy",
    ".interface-facts",
    ".capture-grid",
    ".module-grid",
    ".audit-layout",
    ".download-grid",
    ".release-timeline",
    ".faq-list",
    ".vault-shell",
    ".inventory-grid",
    ".policy-grid",
    ".inquiry-card",
  ].join(",");
  const individualRevealSelector = [
    ".proof-rail > div",
    ".partner-rail",
    ".section-head",
    ".benchmark-card",
    ".performance-copy",
    ".measure-card",
    ".pipeline-card",
    ".media-frame",
    ".preview-lab",
    ".anatomy li",
    ".interface-facts li",
    ".capture-frame",
    ".capture-copy",
    ".module-card",
    ".audit-layout > *",
    ".download-card",
    ".release-block",
    ".release-timeline article",
    ".faq-list details",
    ".vault-shell",
    ".inventory-card",
    ".policy-grid > div",
    ".inquiry-card",
  ].join(",");

  let revealTargets = [];
  let revealObserver = null;
  let activeNarrow = null;
  let activeOrientation = null;
  let revealRefreshTimer = 0;

  function currentOrientation() {
    if (window.screen && window.screen.orientation &&
        typeof window.screen.orientation.angle === "number") {
      return window.screen.orientation.angle;
    }
    return typeof window.orientation === "number" ? window.orientation : null;
  }

  function clearRevealTargets() {
    for (const element of revealTargets) {
      element.classList.remove("will-reveal", "is-visible");
      element.style.removeProperty("--reveal-delay");
    }
    revealTargets = [];
  }

  function setupRevealTargets() {
    if (revealObserver) {
      revealObserver.disconnect();
      revealObserver = null;
    }

    clearRevealTargets();

    const narrow = narrowQuery.matches;
    activeNarrow = narrow;
    activeOrientation = currentOrientation();
    revealTargets = Array.from(document.querySelectorAll(
      narrow ? groupedRevealSelector : individualRevealSelector
    ));

    if (reduceMotion() || !("IntersectionObserver" in window)) return;

    const revealIndex = new Map(revealTargets.map((element, index) => [element, index]));

    // Use the observer's entry geometry instead of calling
    // getBoundingClientRect() during setup. The observer's initial callback
    // gives us the same above/below-fold decision without forcing layout in
    // the middle of script execution.
    const observer = new IntersectionObserver(
      (entries) => {
        const viewportHeight = window.innerHeight || 800;
        for (const entry of entries) {
          const element = entry.target;
          const top = entry.boundingClientRect.top;
          if (entry.isIntersecting || top <= viewportHeight * 0.92) {
            element.classList.remove("will-reveal");
            element.classList.add("is-visible");
            observer.unobserve(element);
            continue;
          }
          if (!element.classList.contains("will-reveal")) {
            const index = revealIndex.get(element) || 0;
            element.style.setProperty("--reveal-delay", `${(index % 5) * 55}ms`);
            element.classList.add("will-reveal");
          }
        }
      },
      { rootMargin: "0px 0px -8% 0px", threshold: 0.06 }
    );
    revealObserver = observer;

    for (const element of revealTargets) {
      observer.observe(element);
    }
  }

  function scheduleRevealRefresh(force) {
    const narrow = narrowQuery.matches;
    const orientation = currentOrientation();
    if (!force && narrow === activeNarrow && orientation === activeOrientation) return;
    if (revealRefreshTimer) return;

    revealRefreshTimer = window.setTimeout(() => {
      revealRefreshTimer = 0;
      setupRevealTargets();
    }, 120);
  }

  function syncReducedMotion() {
    setupRevealTargets();
  }

  setupRevealTargets();

  if (narrowQuery.addEventListener) {
    narrowQuery.addEventListener("change", () => scheduleRevealRefresh(true));
  } else if (narrowQuery.addListener) {
    narrowQuery.addListener(() => scheduleRevealRefresh(true));
  }

  if (reducedQuery.addEventListener) {
    reducedQuery.addEventListener("change", syncReducedMotion);
  } else if (reducedQuery.addListener) {
    reducedQuery.addListener(syncReducedMotion);
  }

  window.addEventListener("orientationchange", () => scheduleRevealRefresh(true), { passive: true });
  window.addEventListener("resize", () => scheduleRevealRefresh(false), { passive: true });

  const finePointer = window.matchMedia("(hover: hover) and (pointer: fine)").matches;

  if (!reduceMotion() && finePointer) {
    const spotlightTargets = document.querySelectorAll(
      [
        ".console-card",
        ".benchmark-card",
        ".performance-copy",
        ".measure-card",
        ".pipeline-card",
        ".media-frame",
        ".preview-lab",
        ".capture-frame",
        ".module-card",
        ".audit-panel",
        ".download-card",
        ".vault-shell",
        ".inventory-card",
        ".policy-grid > div",
        ".inquiry-card",
      ].join(",")
    );

    for (const target of spotlightTargets) {
      target.classList.add("spotlight");
      const layer = document.createElement("span");
      layer.className = "spotlight-layer";
      layer.setAttribute("aria-hidden", "true");
      target.appendChild(layer);

      let frame = 0;
      target.addEventListener(
        "pointermove",
        (event) => {
          if (frame) return;
          frame = window.requestAnimationFrame(() => {
            const bounds = target.getBoundingClientRect();
            const x = ((event.clientX - bounds.left) / Math.max(1, bounds.width)) * 100;
            const y = ((event.clientY - bounds.top) / Math.max(1, bounds.height)) * 100;
            target.style.setProperty("--spot-x", `${x.toFixed(1)}%`);
            target.style.setProperty("--spot-y", `${y.toFixed(1)}%`);
            frame = 0;
          });
        },
        { passive: true }
      );
    }

    const hero = document.querySelector(".hero");
    if (hero) {
      let heroFrame = 0;
      let nextX = 0;
      let nextY = 0;

      hero.addEventListener(
        "pointermove",
        (event) => {
          const bounds = hero.getBoundingClientRect();
          nextX = ((event.clientX - bounds.left) / Math.max(1, bounds.width) - 0.5) * 12;
          nextY = ((event.clientY - bounds.top) / Math.max(1, bounds.height) - 0.5) * 9;

          if (heroFrame) return;
          heroFrame = window.requestAnimationFrame(() => {
            hero.style.setProperty("--hero-x", `${nextX.toFixed(2)}px`);
            hero.style.setProperty("--hero-y", `${nextY.toFixed(2)}px`);
            heroFrame = 0;
          });
        },
        { passive: true }
      );

      hero.addEventListener("pointerleave", () => {
        hero.style.setProperty("--hero-x", "0px");
        hero.style.setProperty("--hero-y", "0px");
      });
    }
  }
})();
