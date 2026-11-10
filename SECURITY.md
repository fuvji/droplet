# Security policy

Droplet is a local Windows macro overlay. It sends synthetic keyboard and mouse
input through Win32 `SendInput`, keeps settings in
`%LOCALAPPDATA%\Droplet`, and does not read game memory, contact the network, or
install a background service.

## Supported versions

| Version | Supported |
| --- | --- |
| 2.6.x | Yes |
| 2.5.x and older | No |

Only builds from the [project site](https://fuvji.github.io/droplet/)
or [GitHub](https://github.com/fuvji/droplet) are supported. Verify the release
hash before running a binary from anywhere else.

## Reporting a vulnerability

Use a [private security advisory](https://github.com/fuvji/droplet/security/advisories/new).
Do not open a public issue for an unpatched vulnerability.

Include the Droplet version, Windows version, the smallest reproduction, and
whether the build is the portable executable or the MSIX package. A proof of
concept helps; a live exploit against someone else's account does not.

Expect an acknowledgement within three days. Fixes ship as a patch release,
and the advisory is published once a build is available.

## Scope

In scope: the overlay, input injection and safety gates, configuration and
migration, packaging, and the static site.

Out of scope: Minecraft itself, third-party clients or mods, anti-cheat
behavior, and reports that require an already-compromised machine.

## Deliberate boundaries

Droplet has no streamproof or capture-concealment mode, no process hiding, no
scan evasion, no anti-cheat bypass, and no game-memory access. The overlay is
ordinary visible desktop composition. Those are product boundaries, not
vulnerabilities.
