# Combat Lab

Hydrogen's combat lab is an offline, zero-dependency way to turn timestamped
hit observations from combat footage into a small **timing prior** for the
runtime. It is not a vision model and it does not watch video or use the
network.

## What it learns

The lab measures the spacing between confirmed hits per module and writes the
median interval to a compact profile:

```text
# Hydrogen combat profile v1
v6=50
v5=51
wtap=105
```

At startup, Hydrogen reads `%LOCALAPPDATA%\Hydrogen\combat.profile` once. The
learned value is blended with the existing local cadence bias, then clamped to
the module's configured Min/Max and tick floor. Delete the file to return to
the original timing behavior. Smart timing must be enabled for the profile to
affect a wait.

This is intentionally only a **timing prior**. It does not aim, move, select a
target, or learn a visual policy.

## Annotate a local clip

Open `tools/combat-annotator.html` directly in a browser. Load a local video,
choose the module, and mark each confirmed hit with `H` (or the button). The
50 ms step buttons align marks to game ticks; the page keeps the file in the
browser and never uploads it. Export the CSV, then run the lab below.

The annotator is a convenience, not a hit detector. You are still responsible
for confirming that a hit landed. Do not use footage you do not have permission
to use, and do not annotate public-server cheating.

## Input format

Use a CSV with one row per candidate event. Only rows marked as a confirmed hit
are used:

```csv
clip,module,time_ms,hit
duel-01,v6,1000,1
duel-01,v6,1053,1
duel-01,v6,1111,1
duel-02,v5,420,1
duel-02,v5,472,1
```

Required columns:

| Column | Meaning |
| --- | --- |
| `clip` | Clip or recording identifier. Intervals never cross clips. |
| `module` | Runtime module key: `v5`, `v6`, `v2`, `v3`, `v8`, `vA`, `vC`/`wtap`, etc. |
| `time_ms` | Hit timestamp in milliseconds from the clip start. |
| `hit` | `1`, `true`, `yes`, or `hit` for a confirmed hit; anything else is ignored. |

Use 20-game-tick footage when possible: one tick is 50 ms. Record only hits you
can verify from the footage; a miss or a guessed label will teach the wrong
tempo. The lab ignores intervals longer than 2 seconds and uses a minimum of
six intervals per module by default.

## Run

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools/combat-lab.ps1 -Csv .\combat.csv
```

Optional arguments:

```powershell
-MinSamples 10
-OutFile .\combat.profile
```

The default output is `%LOCALAPPDATA%\Hydrogen\combat.profile`. Restart
Hydrogen after generating it.

## Data and safety rules

- Use your own recordings or datasets whose licenses allow this use. Do not
  commit video, images, or datasets to this repository.
- Annotate private, consenting test servers only. Do not use the lab or
  Hydrogen to cheat on public servers.
- The runtime never opens the CSV, video, or combat profile over the network.
- Profile values are clamped to the selected UI ranges and the 50 ms tick floor;
  the lab cannot bypass the focus, master, or binding safety gates.
- The lab is a research aid, not proof that a timing is optimal. Validate any
  profile in a controlled client before trusting it.

## Why not train directly on raw internet video?

Unlabelled video has no action or hit label. Video-pretraining research such as
OpenAI VPT uses an inverse-dynamics model and very large compute budgets to
infer actions, and PvP projects such as PVP ROBO or rl-mc-pvp-bot ship Python
training/inference stacks. None of that fits Hydrogen's zero-dependency,
offline, small-binary runtime. The lab therefore starts from the smallest
honest signal: human-verified hit timestamps. See [AI boundaries](AI.md) for
the research summary.
