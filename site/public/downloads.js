// Local OS hint for the downloads section. No network, storage, cookies, or
// tracking. If the API is unavailable, the static Windows link still works.
(() => {
  const grid = document.querySelector("[data-platform-grid]");
  if (!grid) return;

  const hint = (
    (navigator.userAgentData && navigator.userAgentData.platform) ||
    navigator.platform ||
    navigator.userAgent ||
    ""
  ).toLowerCase();

  let os = "linux";
  if (hint.includes("win")) os = "windows";
  else if (hint.includes("mac")) os = "macos";

  const card = grid.querySelector('[data-os="' + os + '"]');
  if (card) card.classList.add("is-current");

  const label = document.querySelector("[data-download-label]");
  const note = document.querySelector("[data-platform-note]");
  if (os === "windows") {
    if (label) label.textContent = "Download for Windows";
    if (note) note.textContent = "Windows x64 detected. Download and run droplet.exe; there is no installer.";
  } else if (os === "macos") {
    if (label) label.textContent = "Windows build";
    if (note) note.textContent = "This Mac is detected, but Droplet currently has no macOS build. The Windows exe is shown for reference.";
  } else {
    if (label) label.textContent = "Windows build";
    if (note) note.textContent = "Linux detected. Droplet currently has no Linux build; the Windows exe is shown for reference.";
  }
})();
