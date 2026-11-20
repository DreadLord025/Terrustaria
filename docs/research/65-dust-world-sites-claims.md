# 65 — WorldGen/Main dust-site batches (W4-d54): claims + census

Status: CLAIM doc (W4-d54, the world lane's first batch). The branch list is
the claim record; this doc pins the site split so the world lane's follow-ups
(W4-e4 `dust-sites-world-2`, W4-f6 `dust-world-3`) and the g-lane census
(research/64) never overlap.

## Census

`decompiled/Terraria/WorldGen.cs` carries **29** `Dust.NewDust*`-family call
sites (research/15's per-file table); `Main.cs` carries **23**. W4-d54 takes
**WorldGen's 29 + Main's first 11** (line order) = **40 sites**. Main's
remaining 12 (22744-57793, the `DrawNPCDirect`/`DrawNPCExtras`/late-file
sites) stay with the g-lane's thirds split (research/64: g12 sites 12-16,
g13 sites 17-23) — d54 does NOT touch them.

### WorldGen.cs (29 sites, line-ordered)

| # | Line | Family | Enclosing method |
| - | ---- | ------ | ---------------- |
| 1 | 29649 | NewDustDirect | `TreeGrowFX_AshShake` (`:29639`) — 31 ash puff |
| 2 | 29657 | NewDustDirect | `TreeGrowFX_AshShake` — 6 ember |
| 3 | 37624 | NewDust | `UnlockDoor` (`:37599`) — 11 gold-door puff |
| 4 | 44074 | NewDust | `paintEffect` (`:44064`) — 143 paint |
| 5 | 44097 | NewDust | `paintCoatEffect` (`:44087`) — 143 coating |
| 6 | 58034 | NewDust | `KillActuator` (`:58022`) — 50 actuator |
| 7 | 58134 | NewDust | `KillWire` (`:58122`) — 50 wire |
| 8 | 58164 | NewDust | `KillWire2` (`:58152`) — 50 wire |
| 9 | 58194 | NewDust | `KillWire3` (`:58182`) — 50 wire |
| 10 | 58224 | NewDust | `KillWire4` (`:58212`) — 50 wire |
| 11 | 60287 | NewDust | `PlaceTile` (`:59342`) — 14 torch/campfire |
| 12 | 62140 | NewDust | `KillWall_MakeWallDust` (`:61345`) — wall 348 |
| 13 | 62144 | NewDust | `KillWall_MakeWallDust` — 66 disco |
| 14 | 62151 | NewDust | `KillWall_MakeWallDust` — wall 76 blue |
| 15 | 62155 | NewDust | `KillWall_MakeWallDust` — default |
| 16 | 69089 | NewDust | `KillTile_MakeTileDust` (`:66687`) — disco tile |
| 17 | 69101 | NewDust | `KillTile_MakeTileDust` — 518 fountain |
| 18 | 69105 | NewDust | `KillTile_MakeTileDust` — 352/5 campfire |
| 19 | 69114 | NewDust | `KillTile_MakeTileDust` — 66 disco |
| 20 | 69120 | NewDust | `KillTile_MakeTileDust` — 323 frameY offset |
| 21 | 69124 | NewDust | `KillTile_MakeTileDust` — 314/213 minecart |
| 22 | 69132 | NewDust | `KillTile_MakeTileDust` — flag2 noLight |
| 23 | 69138 | NewDust | `KillTile_MakeTileDust` — 193/18/19 blue |
| 24 | 69142 | NewDust | `KillTile_MakeTileDust` — 197 blue |
| 25 | 69146 | NewDust | `KillTile_MakeTileDust` — 185 noLight |
| 26 | 69152 | NewDust | `KillTile_MakeTileDust` — 4/66 torch disco |
| 27 | 69158 | NewDust | `KillTile_MakeTileDust` — 139 random type |
| 28 | 69162 | NewDust | `KillTile_MakeTileDust` — 722 velocity |
| 29 | 69166 | NewDust | `KillTile_MakeTileDust` — default |

### Main.cs (first 11 sites, line-ordered)

| # | Line | Family | Enclosing method |
| - | ---- | ------ | ---------------- |
| 30 | 13088 | NewDust | `snowing()` (`:13032`) — 76 snowflake |
| 31 | 13243 | NewDust | `TeleportEffect` (`:13233`) style 0 — 159 |
| 32 | 13262 | NewDust | `TeleportEffect` style 1 — 164 |
| 33 | 13280 | NewDust | `TeleportEffect` style 2 — 58 GhostWhite |
| 34 | 13290 | NewDust | `TeleportEffect` style 3 — 180 |
| 35 | 13324 | NewDust | `TeleportEffect` style 4 — 263 portal |
| 36 | 13340 | NewDustDirect | `TeleportEffect` style 5 — dustWater |
| 37 | 13358 | NewDustDirect | `TeleportEffect` style 7 — 35 |
| 38 | 13390 | NewDustDirect | `TeleportEffect` style 10 — 267 hsl + CloneDust |
| 39 | 13438 | NewDustPerfect | `TeleportEffect` style 11 — 267 |
| 40 | 13452 | NewDust | `TeleportEffect` style 13 — 27 |

`TeleportEffect` style 9 (`:13376`) routes to `TeleportPylonsSystem.
SpawnInWorldDust` — not a `Dust.NewDust*` site in Main.cs, so it is out of
this census (the pylon lane's).

## Batch split (10 sites per commit)

- **Batch 1 (sites 1-10)**: ash-shake ×2, unlock-door, paint ×2,
  kill-actuator, kill-wire ×4.
- **Batch 2 (sites 11-20)**: PlaceTile torch + `KillWall_MakeWallDust`
  (full wall→dust table + 4 spawn arms) + `KillTile_MakeTileDust` head
  (disco/fountain/campfire/disco-66/323 arms).
- **Batch 3 (sites 21-29)**: `KillTile_MakeTileDust` tail (minecart/flag2/
  blue/noLight/torch-disco/random/722/default arms) + the full tile→dust
  type table.
- **Batch 4 (sites 30-40)**: `snowing()` + `TeleportEffect` styles
  0/1/2/3/4/5/7/10/11/13.

## RNG contract (world-pass slots)

Each ported site drains on the stream of the `Main.SwapRandom` pass that
executes it in the C# (never a new stream, S1-A-10), per the d51/d52
convention:

- The editing sites (ash-shake, unlock-door, paint, actuator/wire cutter,
  PlaceTile, KillTile/KillWall/ReplaceWall) run from `Player.ItemCheck` /
  `Player.PickTile` / `Player.PickWall` under the `UpdatePlayers` swap
  (`Main.cs:17754`) → `RngKey::UpdatePlayers`.
- `WorldGen.genRand` draws (the wall/tile type-table rolls, paint's
  `Next(2)`, minecart's `Next(-2,3)`) ride `World::frame_rng` — the port's
  runtime `genRand` stand-in (`world.rs:53-60`, the `SquareTileFrame`
  precedent). The worldgen *pipeline* streams (`worldgen/pipeline.rs`
  `pass_rng`) are untouched: these are runtime editing draws, not
  generation passes.
- `snowing()` runs in `Main.DoUpdate` (`Main.cs:17238`) OUTSIDE every
  `SwapRandom` block → the bare `Main.rand` → `NamedRngs::global`.
- `TeleportEffect` runs from `Player.Teleport` (`Player.cs:37444`/`:37489`)
  under `UpdatePlayers` → `RngKey::UpdatePlayers`.

Pool contract: `tr_core::particles::DustPool::new_dust` /
`new_dust_direct` / `new_dust_perfect` / `clone_dust` (the d50 chain,
`Dust.cs:75-256` verbatim). The `generating` guard returns the sentinel
during worldgen, so the generation-time callers emit nothing at runtime
(research/15's "N/A (latent)" verdict) — the ported bodies are the
runtime-editing halves.

## Live seams vs latent bodies

- **Live**: kill-actuator/kill-wire (`world_sim/editing/wiring.rs` cutter),
  PlaceTile (`editing.rs::place_tile`), KillTile (`editing.rs::break_tile`),
  KillWall (`editing.rs::break_wall`), ReplaceWall (`place_wall_route`).
- **Latent** (no live port caller yet — bodies ported verbatim + cited, the
  d52 unported-arm precedent): ash-shake (`TreeGrowFX`/`ShakeTree` not
  ported), unlock-door (`UnlockDoor` not ported), paint/coat
  (`paintTile`/`paintWall` not ported), snowing (the port's `rain.rs`
  snowfall is the documented M1.4-WEATHER-2 stand-in), TeleportEffect
  (`Player.Teleport` is the recall_teleport follow-up).

Witnesses: draw-mirror tests (the d52 `new_dust_draws` oracle pattern),
WRITTEN NOT RUN (code-only mode).
