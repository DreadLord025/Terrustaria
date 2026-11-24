# 183 — WorldGen.cs dust-site census + claims (W4-g14, adopting d54/e4/f6)

Status: CLAIM + census doc (W4-g14 `task/W4-g14-dust-worldgen-sites`).
In-flight probe (2026-11-20/21): `origin/task/W4-d54-dust-sites-world`,
`origin/task/W4-e4-dust-sites-world-2` and local `task/W4-f6-dust-world-3`
all sit at base `a725acb0` with ZERO pushed commits — the whole
WorldGen.cs dust lane is unclaimed. g14 adopts ALL sites, line-ordered,
10 sites per commit.

## Census

`decompiled/Terraria/WorldGen.cs` carries **29** `Dust.NewDust*` call
sites (research/15's per-file table: 29). Line-ordered, with the
enclosing method and the RNG each site's draws ride:

| # | Line | Family | Enclosing method | RNG notes |
| - | ---- | ------ | ---------------- | --------- |
| 1 | 29649 | NewDustDirect | `TreeGrowFX_AshShake` (`:29639-29672`) | Main.rand: RandomVector2 + internals + NextVector2Circular |
| 2 | 29657 | NewDustDirect | `TreeGrowFX_AshShake` | Main.rand: RandomVector2 + NextFloat scale + internals + Circular + Next(2) |
| 3 | 37624 | NewDust | `UnlockDoor` (`:37599-37628`) | Main.rand internals only (4 per door cell × 3) |
| 4 | 44074 | NewDust | `paintEffect` (`:44064-44085`) | internals Main.rand; post-spawn gate genRand.Next(2) |
| 5 | 44097 | NewDust | `paintCoatEffect` (`:44087-44108`) | old-color pick Main.rand.Next(count); gate genRand.Next(2) |
| 6 | 58034 | NewDust | `KillActuator` (`:58022-58039`) | 5 × type 50, internals only |
| 7 | 58134 | NewDust | `KillWire` (`:58122-58139`) | 5 × type 50 |
| 8 | 58164 | NewDust | `KillWire2` (`:58152-58169`) | 5 × type 50 |
| 9 | 58194 | NewDust | `KillWire3` (`:58182-58199`) | 5 × type 50 |
| 10 | 58224 | NewDust | `KillWire4` (`:58212-58229`) | 5 × type 50 |
| 11 | 60287 | NewDust | `PlaceTile` (`:59342-...`, the `!mute` kind-22/140 arm `:60283-60289`) | 3 × type 14 |
| 12 | 62140 | NewDust | `KillWall_MakeWallDust` (`:61345-62158`) | wall-348 arm: velocity ×0.4 |
| 13 | 62144 | NewDust | `KillWall_MakeWallDust` | wall-44/346 disco arm: type 66, alpha 100, scale 0.75, noGravity |
| 14 | 62151 | NewDust | `KillWall_MakeWallDust` | wall-76 arm: alpha 75, Color(0,80,255,100), scale 0.75 |
| 15 | 62155 | NewDust | `KillWall_MakeWallDust` | generic arm (num >= 0, wall-range gate) |
| 16 | 69089 | NewDust | `KillTile_MakeTileDust` (`:66687-69169`) | 627/628/184-col-10 disco arm, noGravity + noLightEmittance |
| 17 | 69101 | NewDust | `KillTile_MakeTileDust` | 518 liquid-lift arm (SolidTile(i,j-1) cap 8) |
| 18 | 69105 | NewDust | `KillTile_MakeTileDust` | 352 num==5 arm: scale 1.5, noGravity, vel ×1.65, fadeIn 1.6 |
| 19 | 69114 | NewDust | `KillTile_MakeTileDust` | 160/692 disco arm: type 66, alpha 100, scale 0.75, noGravity |
| 20 | 69120 | NewDust | `KillTile_MakeTileDust` | 323 palm arm: position.x += frameY |
| 21 | 69124 | NewDust | `KillTile_MakeTileDust` | 314 minecart-track arm: type 213, speeds genRand.Next(-2,3) ×2, fadeIn = scale+1+0.01*genRand.Next(0,51) |
| 22 | 69132 | NewDust | `KillTile_MakeTileDust` | flag2 (gemsparkle family) arm: alpha 75, scale 0.75, noLight |
| 23 | 69138 | NewDust | `KillTile_MakeTileDust` | 193/18-num4/19-num4 arm: Color(0,80,255,100) |
| 24 | 69142 | NewDust | `KillTile_MakeTileDust` | 197 arm: Color(97,200,255,100) |
| 25 | 69146 | NewDust | `KillTile_MakeTileDust` | 185 num 86..91 arm: alpha 75, scale 0.75, noLight |
| 26 | 69152 | NewDust | `KillTile_MakeTileDust` | 4 num==66 torch-disco arm: float-ctor colour truncation, noGravity |
| 27 | 69158 | NewDust | `KillTile_MakeTileDust` | num==139 arm: Type = num + Main.rand.Next(4) |
| 28 | 69162 | NewDust | `KillTile_MakeTileDust` | 722 arm: velocity ×0.4 |
| 29 | 69166 | NewDust | `KillTile_MakeTileDust` | generic tail |

NOT a spawn site: `:7179-7180` (`clearWorld`'s `Main.dust[n] = new Dust();
dustIndex = n` pool reset) — the port's `DustPool::new` already writes
`dust_index` per slot (`particles.rs:242-246`); the reset itself is the
clear-world lane's, cited here so no batch re-ports it.

## The two type ladders are IN scope (RNG-bearing)

Sites 12-15 and 16-29 sit behind the `KillWall_MakeWallDust` /
`KillTile_MakeTileDust` dust-TYPE ladders. Those ladders carry ~11 and
~35 `WorldGen.genRand` draws respectively (plus one `Main.rand.Next(3)`
at `:67402` and the `Main.rand.Next(4)` argument at `:69158`) — dropping
them would break the RNG-order contract, so the ladders port verbatim
with the spawn arms. `KillTile_GetTileDustAmount` (the COUNT table) is
already ported (`tr-app::collision::tile_dust_amount`,
`projectile_ai_misc::kill_tile_dust_amount_fail`) and stays theirs.

## RNG world-pass slots (the S1-A-10 rule)

Two distinct streams per method, exactly as the C#:

- **genRand** — every `WorldGen.genRand.Next` in the ladders and the
  paint post-spawn gates. The port's world-pass slot is the caller's
  `UnifiedRandom` handed in as `gen` (the framing precedent:
  `world.frame_rng` is the port's genRand stand-in,
  `framing.rs:1263`); the worldgen-pipeline wiring wave may instead hand
  the pass's xorshift slot — the functions are stream-agnostic.
- **Main.rand** — the `NewDust` internals (`DustPool::new_dust`, d50
  contract) plus the explicit argument draws (`RandomVector2`,
  `NextVector2Circular`, `NextFloat`, the 139 `Next(4)`, the 19-row-30
  `Next(3)`, the paintCoat old-color pick). Handed in as `rng`.

The pool's generating guard (`SpawnCtx.generating`, `Dust.cs:89-92`)
means the worldgen passes themselves emit nothing at runtime — these
helpers fire from the runtime edit paths (KillTile/KillWall/paint/wire/
actuator/door/tree-growth). The emitters are UNWIRED pure functions (the
d50/e2 precedent); wiring belongs to the edit-path lanes.

## Batch split (≤10 sites per commit, line order; function atomicity wins)

- **Batch 1 (sites 1-10):** TreeGrowFX_AshShake pair, UnlockDoor,
  paintEffect, paintCoatEffect, KillActuator, KillWire×4.
- **Batch 2 (sites 11-15):** PlaceTile 22/140 arm + the whole
  KillWall_MakeWallDust (ladder + its 4 spawn arms — one function, lands
  whole).
- **Batch 3 (sites 16-29):** the whole KillTile_MakeTileDust (ladder +
  all 14 spawn arms — one function; splitting it across commits would
  leave a temporarily unfaithful body, so the 14-site function lands in
  one commit, the documented exception to the 10-site cap).

Pool contract: `tr_core::particles::DustPool` (d50,
`origin/task/W4-d50-dust-type-table`). Code-only mode: witnesses WRITTEN
NOT RUN; no cargo/rustc/rustfmt/tests/graphify invoked (user directive).
