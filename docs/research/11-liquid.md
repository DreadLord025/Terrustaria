# 11 — Liquid / LiquidBuffer / modules

> Wave 1 · researched 2026-11-12 · sources: decompiled (Liquid.cs, LiquidBuffer.cs, Tile.cs, WorldGen.cs, WorldFile.cs, Main.cs, Rain.cs) + crates (tr-core/src/liquid.rs, liquid_draw.rs, worldgen/liquids.rs, wld/sections/tiles.rs, waterfall/; tr-app/src/world_sim/liquids.rs, render/liquids.rs) + ROADMAP §3

## Scope

Deck-side numbers first, so the rest is anchored: the C# side is one struct file —
`decompiled/Terraria/Liquid.cs` (~1680 lines,
`public struct Liquid` at `Liquid.cs:13`) plus its overflow queue `LiquidBuffer.cs` and the
per-tile accessors in `Tile.cs` (`liquid`, `liquidType`, `lava/honey/shimmer/anyWater`,
`skipLiquid`, `checkingLiquid` — `Tile.cs:235-379`). It owns: the active/buffer lists, the
mass-transfer cellular update (fall-then-level), the type-reaction rules (lava↔water→obsidian,
honey↔water→honey block/crisp honey, everything→shimmer), quick-settle (world load/worldgen),
stuck-detection, panic mode, and the net change set. Rendering (`GameContent/Liquid/*`,
`WaterfallManager.cs`) and the worldgen fills (`WorldGen.PlaceLiquid`,
`WorldGen.WaterCheck`, `WorldGen.cs:80415`) are adjacent modules.

Rust side: `crates/tr-core/src/liquid.rs` (582 lines, `LiquidSystem`) is the cellular core,
`liquid_draw.rs` the draw-instance builder, `worldgen/liquids.rs` the lake/ocean fill pass,
`wld/sections/tiles.rs` the `.wld` tile writer, `crates/tr-app/src/world_sim/liquids.rs` (44
lines) the 60 Hz stepper, `render/liquids.rs` the draw, `crates/tr-core/src/waterfall/` the
waterfall scan.

## Key entities

- `decompiled/Terraria/Liquid.cs::Liquid` (`Liquid.cs:13`) — struct; statics `maxLiquid` 25000,
  `maxLiquidBuffer` 50000, `cycles` 10 (`Liquid.cs:1669-1677`), `curMaxLiquid`, `numLiquid`,
  `skipCount`, `stuckCount/stuckAmount/stuck`, `quickFall`, `quickSettle`, `wetCounter`,
  `panicCounter/panicMode/panicY`; instance `x`, `y`, `kill`, `delay`.
- `Liquid.Update` (`Liquid.cs:449-964`) — the per-cell update: kill on solid, underworld
  evaporation, lava/honey/shimmer gating + `LavaCheck`/`HoneyCheck`/`ShimmerCheck`,
  fall-into-the-cell-below (`:567-597`), then the neighbour averaging cascade (±1/±2/±3,
  `:598-939`), then the kill bookkeeping (`:940-963`).
- `Liquid.UpdateLiquid` (`Liquid.cs:983-1158`) — pass driver: `tilesIgnoreWater(true)`,
  MP cycle scaling, panic/quick-settle branch, the `wetCounter` window
  `[curMaxLiquid/cycles·(w-1), …·w)`, the skipLiquid gate, the `kill >= 8` (MP
  `10 + players/3`) retirement sweep, buffer promotion, stuck detection, net broadcast.
- `Liquid.AddWater` (`Liquid.cs:1160-1207`) — queue one cell (5-tile border guard, `checkingLiquid`
  dedupe, `curMaxLiquid` cap → `LiquidBuffer.AddBuffer`, and `TileObjectData.CheckWaterDeath/
  CheckLavaDeath` → `WorldGen.KillTile` for tiles that die in liquid).
- `Liquid.QuickWater`/`SettleWaterAt`/`AttemptToMoveHoney|Lava|Shimmer`
  (`Liquid.cs:104-435`) — the bulk "settle this rectangle" used by worldgen and the
  world-load settle (`IO/WorldFile.cs:735-760`).
- `Liquid.LiquidCheck`/`GetLiquidMergeTypes`/`CreateLiquidMergeTile`/`LiquidOverwriteStrip`
  (`Liquid.cs:1225-1443`) — type reactions: merge tiles 56 (obsidian), 229 (honey block),
  230 (crisp honey), 659 (shimmer-side), the `num >= 24` mass threshold, `tileCut` kills.
- `Liquid.DelWater` (`Liquid.cs:1470-1600`), `LiquidBuffer` queue.
- `crates/tr-core/src/liquid.rs::LiquidSystem` (`liquid.rs:32-46`) — `active: Vec<Entry>`,
  `checking` bitvec (= `Tile.checkingLiquid`), `buffer` (LiquidBuffer), `cursor`/`pass_len`
  (the pass window), `dirty` (world revision bump).
- `crates/tr-core/src/liquid.rs::update_tile` (`liquid.rs:181-325`) — the port of
  `Liquid.Update` for one cell; helpers `set_level` (`:329`), `round_level` (`:354`,
  banker's rounding `round_ties_even`), `kind_code`/`set_kind` (`:359-376`), `is_solid`
  (`:381-388`).
- `crates/tr-core/src/liquid.rs::update`/`end_pass`/`refill_from_buffer` (`liquid.rs:123-178`)
  — the pass driver; `MAX_ACTIVE` 25000, `MAX_BUFFER` 50000, `CYCLES` 10, `KILL_PASSES` 8.
- `crates/tr-app/src/world_sim/liquids.rs::update_liquids` (`world_sim/liquids.rs:12-23`) —
  the 60 Hz `FixedStepper` that calls `LiquidSystem::update` on every other tick (30 Hz).
- `crates/tr-core/src/wld/sections/tiles.rs` (`tiles.rs:85-100`, `:237-242`) — liquid amount +
  kind bytes per tile in the `.wld` tile block, and the RLE key that folds liquid into the
  tile-header comparison.

Adjacent, deliberately outside this module: the player's wet/breath flags and movement
multipliers (`Player.cs:27728-27910`, ported in `world_sim/player.rs`, ROADMAP M2-PHYS-3),
the `SceneMetrics` liquid counts that drive the shimmer ramps (`SceneState.cs:283-309`,
ported with M1.5-LIQUID-3), the liquid sound loops (`LegacySoundPlayer`, M1.4-BG-20), the
liquid death rules for tiles and NPCs (`WorldGen.cs:72203`, `TileObjectData`), and the
`.wld` tile section as a whole (module 42). Pumps, actuator liquid control and buckets are
still open items in ROADMAP §3.

## Dependencies

- Calls into: `Tile` accessors (`nactive`, `liquidType`, `lava/honey/shimmer/any*`,
  `skipLiquid`, `checkingLiquid`), `Main.tileSolid`/`tileSolidTop`/`tileCut`/
  `tileObsidianKill`, `WorldGen` (`InWorld`, `SolidTile`, `KillTile`, `PlaceTile`,
  `ReplaceTile`, `SquareTileFrame`, `genRand`, `GetLiquidChangeType`,
  `PlayLiquidChangeSound`), `TileObjectData.CheckWaterDeath/CheckLavaDeath`,
  `Main.UnderworldLayer`, `NetLiquidModule`, `Language`.
- Called by: `Main.Update`-side liquid step (`Main.cs`), `WorldGen` (fills,
  `WaterCheck`, `AddWater` at `WorldGen.cs:4573`, `:80415`), `Collision`/`Player` (wet flags,
  out of module), `IO/WorldFile` (post-load settle), `DelegateMethods.cs:510` (a scan that
  wakes liquid).
- Shared data: the tile grid itself (`Tile.liquid` byte + kind bits), `Main.liquid[]`/
  `Main.liquidBuffer[]` (Rust: `LiquidSystem.active`/`buffer`), `Liquid.skipCount`
  (cadence), `GenVars.waterLine` (panic).

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| Statics & constants (`maxLiquid`, `maxLiquidBuffer`, `cycles`) | PORTED | `liquid.rs:15-19` | 25000 / 50000 / 10 — matches `Liquid.cs:1671-1673` exactly |
| `AddWater` queue + `checkingLiquid` dedupe | PORTED (W4-g45 round 2 → union) | `liquid.rs::add_water` | Dedupe + cap → buffer + the `:1163` 5-tile border guard + `type != 546` carve-out + the `:1183-1206` inline `CheckWaterDeath/CheckLavaDeath` kill (`daa1cdb0`, carried onto the g47 base by the W4-g46 RE-FOLD `16fd6ad7` — the `TileKill` queues on `edit_kills`, the loot upgrade; `&mut World` signature; fixtures (+6,+6)-widened; `liquid_addwater_guards` ×10) |
| Pass driver (`UpdateLiquid`) | PORTED (S1-A-8 + W4-g47 + R-12-5) | `liquid.rs::update`/`update_world_tick` | the C# `curMaxLiquid/cycles` stride + `numLiquid` clamp (`:1069-1106`), `tilesIgnoreWater` (`:986`/`:1157`), stuck/panic/quickSettle, the `skipCount` runtime pump (`WorldGen.cs:71526-71534`, R-12-5 driver 1 — the app `% 2` divider removed); the net broadcast stays out of scope (`Liquid.cs:983-1158`) |
| Per-cell update (`Liquid.Update`) | PORTED (union) | `liquid.rs::update_tile` | Fall + ±1/±2/±3 averaging 1:1 incl. banker's rounding; the `delay` gate (R-11-5), underworld evaporation (W4-g46), `liquidType` reactions (R-11-5), the `genRand` nudge on `surface_rng` (W4-g46) and the 7/5-neighbour extra wakes (W4-g45) all landed — W4-g46 RE-FOLD union `e7626b72` |
| `skipLiquid` skip flag | PORTED | `liquid.rs` `set_skip`/`is_skip` | `Liquid.cs:586-587` post-fall writes, `:1096-1104` per-entry gate, `:1173` AddWater clear (anchors `set_skip`) |
| `delay` (lava 5 / honey 10 / quickFall 10) | PORTED (R-11-5) | `liquid.rs` `Entry.delay`/`flow_delay` | `Liquid.cs:51`, `:485-491`, `:516-522`, `:1083-1091`; the cadence pins ride `liquid_merge_cadence` (g45 carry, union-widened) |
| `quickSettle` / `quickFall` | PORTED (W4-g47) | `liquid.rs` `quick_settle`/`quick_fall` | `Liquid.cs:35/33`, `:588-591`, `:944-948`, `:1056-1068` — phase 1 `047f37f3`; the gfx-quality override `Main.cs:17105-17113` rides `gfx_quality.rs::apply` |
| Liquid reactions (`Lava/Honey/ShimmerCheck` → `LiquidCheck` → merge tiles) | PORTED (R-11-5) | `liquid.rs::lava_check/honey_check/shimmer_check/liquid_check/create_liquid_merge_tile` | obsidian 56 / honey block 229 / crisp honey 230 / shimmer 659, the `num >= 24` pooling, the `tileCut` burn (`Liquid.cs:1225-1443`); merge family ×11 witnessed |
| Underworld water evaporation | PORTED (W4-g46 → union) | `liquid.rs::update_tile` (the `y > underworld_layer && kind == 0` arm) | `Liquid.cs:466-474` — `min(2, liquid)` shed per update below `UnderworldLayer` (`Main.cs:2790`, `maxTilesY - 200`), water-only (the type bits); an emptied cell retires (`kill = 999`, `:475-478`). `underworld_layer` seeded from the world height in `new`; the fall arm reads the live post-evap level (`own_now`); every flow witness pins the band out of reach (`new_flow_fixture`, the `tool.rs` `i32::MAX` convention). Witnesses: `liquid_evaporation` ×4 + the re-pinned layer-row boundary (`liquid_merge_cadence`) — union `e7626b72` |
| Stuck detection + panic mode + `QuickWater` | PORTED (W4-g47) | `liquid.rs` `end_pass`/`start_panic`/`quick_water`/`settle_water_at`/`water_check`/`settle_loaded_world` | stuck band `:1131-1149` + panic `:966-1054` + `QuickWater`/`SettleWaterAt`/`AttemptToMove*` `:104-433` + `WaterCheck` `WorldGen.cs:80415-80494` — phases 2/3 `e43fa0bc`/`dab3df3d`; the load hook (`WorldFile.cs:735-760`) is WIRED at `App::new` (settle wiring `551c1e7b`, caller-pinned `frame_rng`, dry-world byte/RNG no-op so the round-trip pins hold; `TR_NO_SETTLE` + staging opt-out) |
| Kill threshold 8 / MP `10+players/3` | PARTIAL | `liquid.rs:21` | 8 hardcoded (correct for single player; MP variant absent) |
| `DelWater` retirement tail + `LiquidBuffer` counts | PORTED (W4-g47, g46 loot seam adopted) | `liquid.rs` `del_water`/`end_pass`/`add_water` | `Liquid.cs:1470-1599` drain rules/keep-alive/swap-remove/lava grass ring/`CheckAlch`/`CheckLilyPad` tail + the `kill>=8` sweep's 254-promotion `:1109-1119` + `LiquidBuffer.cs:11-27` AddBuffer/DelBuffer (checkingLiquid bit, maxLiquidBuffer-2 cap, swap-remove promotion) — phase 2 `e43fa0bc`; the `Check*` drops AND the grass-ring/lily `square_tile_frame_record` cascades ride `drain_edit_kills` (g46's loot seam adopted at `d3a30ebf`, keep-one g47 canonical); the g45 sharper witnesses (delay-inheritance/mud-60 ring RECORDS/dry-pad/buffer-delay-0/kill-0 stuck) unioned in `liquid_delwater` |
| `.wld` liquid bytes | PORTED | `wld/sections/tiles.rs:85-100`, `:237-242` | Liquid amount then kind byte, per the cited `WorldFile.cs:1480-1491`; the liquid byte participates in the RLE key exactly as `bTileHeader` does in the writer |
| Render side (`liquid_draw`, `render/liquids`, waterfall) | PORTED | `liquid_draw.rs:88`, `world_sim/liquids.rs:27-44`, `waterfall/` | Water styles + shimmer draw are ROADMAP-M1.5-LIQUID-2/3 ✅ (re-verified: `liquid_draw.rs::build` builds the per-tile liquid instances, `liquid_type` maps the 4 kinds) |
| Worldgen fills | PARTIAL | `worldgen/liquids.rs:21` (`place_liquids`) | Lake bowl + ocean shores + lava drips; the `SettleWaterAt` gen re-type arm (`Liquid.cs:178-192`) is ported + gated on `generating` (`set_gen_settle_context`, W4-g47 `551c1e7b`) INCLUDING the dungeon arm (`GenDungeonContext`, final phase `d3a30ebf` — f24/c30 bounds carry, ORIGIN wall read) and `QuickWater`'s `ShimmerRemoveWater` (g36-A1 `shimmer_position` seam); the gen settle DRIVER (the two `Settle Liquids` passes, `WorldGen.cs:16132`/`:21246`) is f25's `settle_liquids.rs` — graft per docs/research/70 |
| `Setting_UseReducedMaxLiquids` (`slowliquids=1`) | PORTED (W4-g47 final) | `settings.rs` `reduced_max_liquids` + `apply_launch_args`, `liquid.rs` `set_reduced_max_liquids`/`re_init`, `gfx_quality.rs::apply` | `Main.cs:1665`/`:5079-5083`/`:67008` launch-only flag; `ReInit` 5000 cap (`Liquid.cs:98-101`), gfx `2500+2500q` (`Main.cs:17080-17083`) + flat-5000 quickSettle override (`:17108-17111`), pass-head `numLiquid>2000` quickFall OR-arm (`Liquid.cs:1057-1060`) — `d3a30ebf`; the MP `:1000-1003` arm stays out of scope (deviation 9) |

## Tests

- `crates/tr-core/src/liquid.rs::tests` (`liquid.rs:390-581`):
  `water_falls_and_fills_the_pit_bottom` (`:430`), `pool_levels_out_and_settles` (`:453`),
  `stone_walls_block_the_spread` (`:475`),
  `a_gap_in_the_dam_drains_into_the_lower_chamber` (`:499`),
  `add_water_deduplicates_tiles` (`:553`),
  `the_actuated_floor_lets_water_through` (`:566`).
  Coverage: gravity/fall, levelling, solid and dam blocking, dedupe, the `nactive()`
  (actuated) read, mass conservation within ±3.
- `crates/tr-core/src/liquid_draw.rs` — 3 `#[test]`s at `:478`, `:504`, `:524` (instance
  building; names not extracted in this pass).
- `crates/tr-core/src/worldgen/liquids.rs` — `lake_keeps_grass_on_the_dry_banks` (`:193`),
  `lake_bowl_floor_is_solid_below_the_surface` (`:226`).
- No tests exist for: lava/honey/shimmer kinds (no kind is ever set outside tests), the
  `delay` cadence, evaporation, merge tiles, stuck/panic, kill thresholds, or a `.wld`
  liquid-byte round-trip (`chest.rs:3393`'s DW-3.4 covers the whole-file cycle, not the
  liquid bytes specifically).

## Deviations & risks

1. **Liquid type reactions absent (behavioural, high).** `LiquidUpdate` never calls
   `LavaCheck/HoneyCheck/ShimmerCheck`, so water meeting lava never becomes obsidian
   (`Liquid.cs:1261-1265`, tile 56), honey+water never becomes honey block/crisp honey
   (229/230, `:1399-1420`), and nothing becomes 659. The port's `update_tile` only compares
   kind codes and blocks cross-kind flow, so pools of different liquids simply refuse to
   mix. Not admitted anywhere in ROADMAP §3 (which lists only the render leftovers).
2. **`delay` cadence missing (behavioural, med-high).** Lava moves every 6th visit
   (`delay < 5`) and honey every 11th (`delay < 10`) — C# `Liquid.cs:485-491/516-522`.
   Without `delay`, lava/honey propagate ~6-11× faster than vanilla and settle differently.
   Not admitted.
3. **Underworld evaporation — PORTED (W4-g46, union `e7626b72`).** `Liquid.cs:466-474`
   removes `min(2, liquid)` units per update from water below `UnderworldLayer`; the
   union's `update_tile` does the same (unguarded `y > underworld_layer`, seeded
   `height - 200` in `new` — runtime-faithful). The mass-conservation witnesses exclude
   the band via `new_flow_fixture` (the `i32::MAX` sentinel, the `tool.rs` convention);
   the evaporation witnesses use a sealed pocket inside the band. The fall arm reads
   the live post-evaporation level. ANALYSIS "The liquid skip/settle surface" re-mark.
4. **Skip/window mechanics differ (determinism, med).** (a) `skipLiquid` is not ported at
   all, so tiles the original would skip a pass for are updated; (b) the C# window is a
   fixed 2500-slot stride with a clamp that *re-walks the entire active list every update
   when `numLiquid <= 2500`* (`Liquid.cs:1070-1082`), while the Rust walks
   `ceil(len/10)` entries per update (`liquid.rs:135`). For small pools the C# simulates
   10× more often per wall-clock second and retires `kill`-ed entries immediately.
   Not admitted (ROADMAP's "Hidden" line mentions `skipCount > 1`, which is a *different*
   mechanism and is ported as `% 2 == 0` in `world_sim/liquids.rs:18`).
   **(b) FIXED 2026-11-13 (S1-A-8):** the port now walks the fixed `curMaxLiquid / cycles`
   stride with the `numLiquid` clamp (`liquid.rs::LiquidSystem::update`), matching
   `Liquid.cs:1069-1106`; (a) `skipLiquid` is still unported.
5. **`genRand` nudge — PORTED, per-consumer stream (determinism, low-med; W4-g46 union).**
   `Liquid.cs:892-895` (`num == 254 && WorldGen.genRand.Next(30) == 0 → 255`) is ported
   in the 3-cell surface arm. It is the ONLY `genRand` draw in `Liquid.cs` (RNG census).
   Vanilla shares one `genRand` (= `Main.rand`, `WorldGen.cs:4445`) across all consumers;
   the union keeps a per-consumer `UnifiedRandom` (`LiquidSystem::surface_rng`, seeded
   from the world seed in `new`, the `place_rng` precedent) so the draw never interleaves
   with other systems — the recorded deviation (research/70 step 3: g46's stream pick
   wins over g45's `frame_rng`). The roll runs once per 254 average, BEFORE any neighbour
   write. Witnesses: `surface_rng_is_the_world_seeded_unified_stream` (draw-for-draw pin)
   + the g45 nudge ×3 RE-PINNED onto `surface_rng` (`liquid_nudge_wakes`).
6. **Extra wake calls — PORTED (W4-g45 carry, union `e7626b72`).** The 7- and
   5-neighbour arms re-queue ±1/±2/±3 whenever the neighbour *or the centre* changed
   (`Liquid.cs:748-771`, `:821-836`) — the centre-diff reduction before the conditional
   centre write. Witness: `the_extra_wake_requeues_an_unqueued_neighbour_on_a_centre_change`
   (re-derived onto the 7-arm fixed-point fixture — the C# 3-cell arm has no extra wakes).
7. **`AddWater` guards — FIXED (W4-g45 round 2 `daa1cdb0`/`8e81814e`, carried onto
   the union by the RE-FOLD `16fd6ad7`).** The 5-tile world-border exclusion
   (`Liquid.cs:1163`), the `tile.type != 546` carve-out, and the
   `CheckWaterDeath/CheckLavaDeath` → `KillTile` of plants/breakables submerged in liquid
   (`:1183-1206`) are on `add_water(&mut World, …)`; the inline kill's `TileKill` queues
   on `edit_kills` (the loot upgrade — g45b discarded it), the `:1183` gate reads the
   port's `generating` (= `isGeneratingOrLoadingWorld`, armed on load too). The ~40
   fixtures are (+6,+6)-embedded past the border with containment walls;
   `liquid_addwater_guards` ×10 (ANALYSIS "The g46c RE-FOLD landing record").
8. **`tilesIgnoreWater` (behavioural, low).** `Liquid.cs:69-73/986/1157` flips
   `Main.tileSolid[546]` (+ boulder solidity) for the whole pass; `Liquid.Update:451` also
   forces `tileSolid[379] = true` per cell. The port has a static table, so liquid vs those
   tiles is constant-time-consistent but edge-case different.
9. **MP kill threshold (behavioural, N/A for SP).** `kill >= 10 + players/3`
   (`Liquid.cs:985`, `:1111`) vs the ported 8; only matters once netplay lands.
10. **Net change set (out of scope).** `_netChangeSet` + `NetLiquidModule.CreateAndBroadcastByChunk`
    (`Liquid.cs:53-67`, `:1151-1156`) — single-player target, no port needed today.
11. **`quickSettle`/panic/stuck (behavioural, med).** World-load settle
    (`WorldFile.cs:735-760`), panic mode and the stuck self-clear are absent: a pathological
    pool never force-settles and never empties itself. Not admitted.
12. **ROADMAP DONE re-verification.** §3 rows "✅ core (M1.5-LIQUID-2 done)" and
    M1.5-LIQUID-2/3 (render/shimmer) re-verify as **PASS** for the draw side
    (`liquid_draw.rs`, `render/liquids.rs`, `waterfall/`). The "core" claim covers only the
    cellular water flow, so the honest status for the simulation is **PARTIAL** per rows 1-11
    above; the remaining-items list in §3 is accurate for what it lists but omits the whole
    reaction/evaporation/skip/stuck surface.

## Task seeds

- **S — DONE (R-11-5)** — the `delay` field + the lava 5 / honey 10 gate + the
  `quickFall`-sets-10 pre-pass flag; the cadence pins ride `liquid_merge_cadence`.
- **S — DONE (W4-g46, union)** — underworld water evaporation (`Liquid.cs:466-474`),
  gated on `LiquidSystem::underworld_layer` (`height - 200`); the mass-conservation
  witnesses exclude the band via `new_flow_fixture`. See the port-status row + deviation 3.
- **S — DONE (W4-g46, union)** — the `genRand` 254→255 nudge (`Liquid.cs:892-895`) on the
  per-consumer `surface_rng` seam; the consumed draw is recorded in deviation 5 and pinned
  by `surface_rng_is_the_world_seeded_unified_stream`.
- **M** — Port `skipLiquid` (tile flag + the gate in the pass loop + the two writes after a
  fall, `Liquid.cs:586-587/1096-1103`) and switch the pass window to the C# 2500-slot stride
  with the `numLiquid` clamp, so a small active list re-walks every update. Includes a
  byte-level settle comparison test against a known `.wld` liquid layout.
- **M** — Port the liquid reactions: `LiquidCheck` + `GetLiquidMergeTypes` +
  `CreateLiquidMergeTile` + `LiquidOverwriteStrip` (merge tiles 56/229/230/659, the
  `num >= 24` threshold, `tileCut` kills, the `tileSolid[546]` swap) and wire the three
  `*Check` calls into `update_tile`. Depends on `edit.rs`'s `KillTile`/`PlaceTile`/
  `SquareTileFrame` being callable from tr-core (new seam or an out-param).
- **M — DONE (W4-g45 round 2 → union)** — `AddWater`'s guards: 5-tile border,
  `type != 546`, and `CheckWaterDeath/CheckLavaDeath` → kill (the `edit_kills` loot
  upgrade; deviation 7).
- **L** — Port the force-settle family: `QuickWater`/`SettleWaterAt`/
  `AttemptToMoveHoney|Lava|Shimmer`, the world-load settle (`WorldFile.cs:735-760`,
  `quickSettle`), stuck detection and panic mode. Depends on M above (merge tiles share
  `CreateLiquidMergeTile`) and on the save pipeline (task 42) for the load hook.
- **L** — MP parity: `curMaxLiquid`/`cycles` from the active-player count and the
  `kill >= 10 + players/3` threshold, plus `_netChangeSet` → the net module. Blocked on the
  netcode module (17).
