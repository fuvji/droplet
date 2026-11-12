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
    .sort((a, b) => a.getBoundingClientRect().top - b.getBoundingClientRect().top);

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
  paintScroll();

  const revealSelector = [
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

  const revealTargets = Array.from(document.querySelectorAll(revealSelector));
  let revealObserver = null;

  function revealAll() {
    for (const element of revealTargets) {
      element.classList.remove("will-reveal");
      element.classList.add("is-visible");
    }
    if (revealObserver) revealObserver.disconnect();
  }

  if (!reduceMotion() && "IntersectionObserver" in window) {
    revealObserver = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (!entry.isIntersecting) continue;
          entry.target.classList.add("is-visible");
          revealObserver.unobserve(entry.target);
        }
      },
      { rootMargin: "0px 0px -8% 0px", threshold: 0.06 }
    );

    const viewportHeight = window.innerHeight || 800;

    // Read all positions first. Interleaving rect reads with class/style
    // writes forces one layout per element and is the difference between a
    // smooth setup and a multi-frame stall on a throttled phone.
    const belowFold = revealTargets
      .map((element, index) => ({
        element,
        index,
        top: element.getBoundingClientRect().top,
      }))
      .filter((entry) => entry.top > viewportHeight * 0.92);

    for (const entry of belowFold) {
      entry.element.style.setProperty("--reveal-delay", `${(entry.index % 5) * 55}ms`);
      entry.element.classList.add("will-reveal");
      revealObserver.observe(entry.element);
    }
  }

  function syncReducedMotion() {
    if (reduceMotion()) revealAll();
  }

  if (reducedQuery.addEventListener) {
    reducedQuery.addEventListener("change", syncReducedMotion);
  } else if (reducedQuery.addListener) {
    reducedQuery.addListener(syncReducedMotion);
  }

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
