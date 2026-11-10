# AI and Vision Boundaries

Hydrogen ships a deterministic, local timing brain. There is no LLM, camera
reader, WebView, screen-video model, or network AI in the runtime. This is a
deliberate design choice, not a missing switch.

## What the runtime actually does

- `smart.rs` learns a per-module activation cadence with an EWMA and picks times
  inside the user's configured range.
- `tools/combat-lab.ps1` can turn human-verified hit timestamps into a compact
  `combat.profile`; the runtime uses that value only as a blended prior.
- A 1x1 GDI probe checks the crosshair pixel for the existing totem watcher.
- All input is injected through `SendInput`; the focus/master gates still stop
  every module when they are not satisfied.

There is no game-memory reading, aim assistance, target tracking, or public
server evasion.

## Research findings

- **Raw video is not a labelled policy.** OpenAI's Video PreTraining work
  (VPT) learns from unlabelled Minecraft video only after training an inverse
  dynamics model to infer actions, using large-scale data and compute:
  <https://cdn.openai.com/vpt/Paper.pdf>.
- **PvP research stacks are not drop-in runtime components.** `PVP ROBO`
  (MIT) is a Python/PaperMC training environment:
  <https://github.com/KyaKayKya/Minecraft-PVP-ROBO>. `rl-mc-pvp-bot` reads the
  screen and uses Python/NumPy inference:
  <https://github.com/robbiebusinessacc/rl-mc-pvp-bot>. `minecraft-rl` is a
  GPL-3.0 C++/LibTorch research simulator:
  <https://github.com/OlivierMor/minecraft-rl>.
- **The stock crosshair is not a reliable hit reward.** Vanilla 1.8 does not
  turn the crosshair red on every landed hit; that is a feature of client
  mods such as AdvancedCrosshair:
  <https://modrinth.com/mod/adv-crosshair>. A pixel-only reward would therefore
  be texture-pack and mod dependent.
- **Minecraft timing is tick-bound.** A game tick is 50 ms. The mace has a
  0.6 attack speed (33 ticks per full cooldown); sword/PvP timing and sprint
  resets need deliberate tick spacing. See:
  <https://minecraft.wiki/w/Attack_speed>,
  <https://mmcpvp.com/lessons/intermediate/wtaptiming>.

## Decision

Do not ship a fake "AI brain" or claim 0 ms vision. Keep the local timing brain,
optional offline profile, and short event-driven waits. A future vision or
external-model feature requires a new design, a measured latency/CPU/size
budget, a data-licensing review, and a separate opt-in surface. It must not
weaken the focus gate or turn Hydrogen into a public-server cheat.

## Practical next step

Collect a small, consented set of 20 TPS clips, label only confirmed hits, and
run `tools/combat-lab.ps1`. Compare the resulting profile against the default
timing in a controlled client. Only keep a profile when the measured hit rate
or consistency improves.
