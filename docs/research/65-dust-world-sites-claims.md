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
# 65 — World-sim dust-site batches (W4-d54 / W4-e4 / W4-f6): claims + census

Status: DELIVERED + WIRED (W4-f6, batch 3 of the world lane). All 40 sites
ported across 4 commits — `885a6bfa` (batch 1, Mount movement 10),
`d6f555b8` (batch 2, Mount effects 10), `9dec91b0` (batch 3, Mount
spawn/dismount 7), `680ac9d8` (batch 4, small world files 13) — then WIRED
to the live machines across 6 wiring commits (`bebd0c55` spawn/dismount,
`ac0c7edc` movement, `65ec64a2` effects, `bcd63051` chest+poof, `7dfcd098`
pylon+void-lens, `7c9165da` lava bubbles). The wiring seam census + the e4
keep-one decision live in research/198 (renamed 67→198, round-1). Code-only: WRITTEN NOT RUN, the 32
draw-mirror + 6 wiring witnesses are owed a compile window before merge.
The claim below was written early per the lane's coordination rule — the
branch list is the claim record; this doc pins the file split so the world
batches never overlap the entity lanes or the g11-g15 Main/WorldGen/WorldItem
lanes.

WIRING NOTE (research/198, renamed from 67): the chest unlock rides **UpdatePlayers**
(ItemCheck→TileInteractionsUse), NOT the bare global the Contract section
below guessed; the pylon/void-lens/lava sites are draw-side (private
`tile_draw_rand`/`liquid_rand` + bare global). The popup flash (g57), rain
(d130) and nebula net-102 (f12/MP-only) stay staged — their machines are on
other branches / unported in SP. f6's 27 Mount sites SUPERSEDE e4's 19
(keep-one: f6 canonical, e4 wires orch 41-61 only).

## Lane boundary (what "world" means here)

The dust-site waves split by C# caller file:

- **Player.cs** — d51 / e1 / f3 / g66 (player lanes).
- **NPC.cs** — d52 / e2 / f4 / g59-g62 (NPC lanes).
- **Projectile.cs** — d53 / e3 / f5 / g63-g65 (projectile lanes).
- **Main.cs (23 sites)** — g11 / g12 / g13, per the pushed
  `64-dust-main-sites-claims.md` thirds split. CLAIMED, untouched here.
- **WorldGen.cs (29 sites)** — g14 (`task/W4-g14-dust-worldgen-sites`).
  CLAIMED by branch, untouched here.
- **WorldItem.cs (17 sites)** — g15 (`task/W4-g15-dust-item-sites`).
  CLAIMED by branch, untouched here.
- **TileDrawing.cs (53)** — d56-d60 tile-draw lanes. Untouched here.

The **world lane** (d54 first / e4 second / f6 third) is everything else that
spawns dust from the world simulation: the mount machine, the tile-edit /
chest / wiring helpers, the liquid renderer, the floating-text / pylon /
void-lens helpers, and the net / rain riders. d54 and e4 pushed **zero**
commits (dead lanes), so every site below is unclaimed; f6 takes the next
~40 in line order, 10 per commit.

## Census (f6's ~40, line-ordered per file)

`Mount.cs` — **27** sites (the minecart-213 wheel sparks are already
`minecart.rs`; these are the OTHER mount dusts):

| Line | Family | Enclosing method |
| ---- | ------ | ---------------- |
| 3113 | NewDust | `UseDrill` (`:3059`) block beam |
| 3170 | NewDust | `UseDrill` wall beam |
| 3878 | NewDust | `UpdateFrame` (`:3581`) case 10/47 dash ring |
| 3896 | NewDust | `UpdateFrame` case 10 dash body |
| 3939 | NewDust | `UpdateFrame` case 44 dash ring |
| 3970 | NewDust | `UpdateFrame` case 44 run puff |
| 4000 | NewDust | `UpdateFrame` case 45 dash ring |
| 4023 | NewDust | `UpdateFrame` case 45 dash body |
| 4103 | NewDust | `UpdateFrame` case 14 basilisk run |
| 4123 | NewDust | `UpdateFrame` case 14 basilisk charge |
| 4626 | NewDustDirect | `EmitGolfCartSmoke` (`:4623`) |
| 4703 | NewDustPerfect | `DoGemMinecartEffect` (`:4697`) |
| 4719 | NewDustPerfect | `DoSteamMinecartEffect` (`:4712`) |
| 4750 | NewDustPerfect | `DoExhaustMinecartEffect` (`:4729`) |
| 4782 | NewDustPerfect | `DoConfettiMinecartEffect` (`:4763`) |
| 4824 | NewDustPerfect | `UpdateEffects` (`:4795`) case 23 broom sparkle |
| 4849 | NewDustPerfect | `UpdateEffects` case 23 broom trail |
| 5061 | NewDust | `UpdateEffects` case 11 drill-mount trail (lerp) |
| 5071 | NewDust | `UpdateEffects` case 11 drill-mount trail (single) |
| 5099 | NewDustPerfect | `UpdateEffects` case 22 lava-cart trail |
| 5149 | NewDustDirect | `UpdateEffects` case 61 pixie tail |
| 6339 | NewDust | `DoFailedDismountDust` (`:6320`) |
| 6382 | NewDustDirect | `DoSpawnDust` (`:6371`) wolf 52 |
| 6391 | CloneDust | `DoSpawnDust` wolf 52 clone |
| 6404 | NewDustDirect | `DoSpawnDust` chillet 62-65 |
| 6413 | CloneDust | `DoSpawnDust` chillet clone |
| 6493 | NewDust | `DoSpawnDust` generic tail |

Small world files — **13** sites:

| File:Line | Family | Enclosing method |
| --------- | ------ | ---------------- |
| `Chest.cs:420` | NewDust | `Unlock` (`:346`) 4-per-tile |
| `Utils.cs:1089` | NewDust | `PoofOfSmoke` (`:1078`) 10-dust tail |
| `GameContent/TeleportPylonsSystem.cs:390` | NewDust | `SpawnInWorldDust` (`:327`) |
| `GameContent/VoidLensHelper.cs:48` | NewDustDirect | `SpawnVoidLensDust` (`:40`) arm A |
| `GameContent/VoidLensHelper.cs:63` | NewDustDirect | `SpawnVoidLensDust` arm B |
| `PopupText.cs:461` | NewDustPerfect | `EmitFancyFlashDust` (`:442`) loop 1 |
| `PopupText.cs:470` | CloneDust | `EmitFancyFlashDust` loop 1 clone |
| `PopupText.cs:479` | NewDustPerfect | `EmitFancyFlashDust` loop 2 |
| `PopupText.cs:488` | CloneDust | `EmitFancyFlashDust` loop 2 clone |
| `GameContent/Liquid/LiquidRenderer.cs:560` | NewDust | `InternalPrepareDraw` (`:161`) lava bubble |
| `GameContent/Liquid/LiquidRenderer.cs:564` | NewDust | `InternalPrepareDraw` lava bubble big |
| `Rain.cs:122` | NewDust | `Rain.Update` (`:96`) splash |
| `MessageBuffer.cs:3697` | NewDust | `GetData` case 102 nebula level-up |

**Already ported / dead (not re-ported):** `Collision.cs:3403` (the cobweb
puff rides `collision.rs::sticky_tiles`' `web_puff` → `player.rs`
`pending_sticky_dust`), `Collision.cs:3427` (dead — `type == 51` inside the
honey-block else-arm, never true), `WaterfallManager.cs:1061`
(`waterfall/draw.rs::try_sparkling` → `rain.rs` sparkle pool).

## Batch split (10 sites per commit)

- **f6 batch 1** — Mount movement: 3113/3170 (UseDrill) + 3878/3896/3939/
  3970/4000/4023/4103/4123 (UpdateFrame dash/gallop). 10 sites.
- **f6 batch 2** — Mount effects: 4626/4703/4719/4750/4782/4824/4849/5061/
  5071/5099 (EmitGolfCartSmoke + the minecart effects + UpdateEffects
  broom/drill/lava). 10 sites.
- **f6 batch 3** — Mount spawn/dismount: 5149/6339/6382/6391/6404/6413/6493
  (pixie tail + DoFailedDismountDust + DoSpawnDust). 7 sites.
- **f6 batch 4** — small world files: Chest 420, Utils 1089, Pylons 390,
  VoidLens 48/63, PopupText 461/470/479/488, LiquidRenderer 560/564, Rain
  122, MessageBuffer 3697. 13 sites.

## Contract

Pool: `origin/task/W4-d50-dust-type-table`
(`tr_core::particles::DustPool::new_dust` / `new_dust_direct` /
`new_dust_perfect` / `clone_dust`, `Dust.cs:75-286` verbatim).

RNG: one slot per C# pass — each ported site drains on the stream of the
`Main.SwapRandom` pass that executes it in the C# (never a new stream,
S1-A-10). The mount / chest / poof sites ride `UpdatePlayers`
(`Player.Update`, `Main.cs:17754`); the pylon `SpawnInWorldDust` rides the
caller's pass (`Main.TeleportEffect` = g11/g12's, `Player.UpdateTeleportVisuals`
= UpdatePlayers); the void-lens rides `UpdateProjectiles`
(`Projectile.cs:52195`); the floating-text rides `UpdateFloatingText`
(`Main.cs:17791`); the lava bubbles ride the renderer's own `_random`
(`LiquidRenderer.cs:132`, a private stream — documented); the rain splash
rides the draw's `Main.rand` (`Main.DrawItems` `:38821`, no swap — the bare
global); the nebula level-up rides the net dispatch's pass.

Code-only mode: WRITTEN NOT RUN, no cargo/rustc/rustfmt/tests/graphify
invoked (user directive). The witnesses are owed a compile window before
merge. Emitters whose enclosing machine is unported are staged (the e2
Skeletron/Brain precedent) with draw-mirror witnesses; the live ones (Chest
Unlock, Utils.PoofOfSmoke, the mount arms present in `mount.rs` /
`player.rs::update_mount`) are wired.

## Delivery (W4-f6)

Two new modules, both staged-emitter (the enclosing machines are unported
M2-MOUNT-3/4 or draw-side; the wiring wave calls them at the C# positions):

- `crates/tr-app/src/mount_dust.rs` — all 27 Mount.cs sites (batches 1-3),
  plus the shared `GetFairyQueenWeaponsColorFull`
  (`Projectile.cs:21665-21889`, the ~40-arm dev-name switch + the
  Cenx/Artyrian/Bame/Unit One early returns + the `hslToRgb` generic path)
  and the `MathHelper.Lerp/SmoothStep`, `Turn01ToCyclic010`,
  `PingPongFrom01To010`, `NextVector2Square/Circular`, `DirectionTo`,
  float-`Color` ctor/lerp/scale helpers. 24 draw-mirror witnesses.
- `crates/tr-app/src/world_dust.rs` — the 13 small world-file sites
  (batch 4) + the `dustWater` map. 8 draw-mirror witnesses.

Deliberate drops (each cited at its site): the `dust.shader` /
`dust.customData` writes (the `particles.rs` field drops), the mount
`Lighting.AddLight` heads (the light lane's), the case-14 `cMount == 0`
FloorVisuals hop (`Mount.cs:4110-4115`, the caller's), the
`netMode == 2` head guards (single-player), and the `Gore.NewGore` bodies
(W4-d55's lane — their ARGUMENT draws are consumed in order, the e2 spin
hand-off split). `EmitGolfCartSmoke` (`Mount.cs:4623`) is DEAD code (zero
callers) — ported verbatim for the census, staged unwired.

RNG: every site drains on its C# pass's stream (S1-A-10) — the mount /
chest / poof / nebula sites on `UpdatePlayers` or the bare global
(`NamedRngs::global()`, the clicks/netglue convention), the void-lens on
`UpdateProjectiles`, the fancy flash on `UpdateFloatingText`, the lava
bubbles on the renderer's PRIVATE `_random` (`LiquidRenderer.cs:132`, a
distinct stream — NOT a swap key), the rain splash on the draw's bare
global. No new streams.
