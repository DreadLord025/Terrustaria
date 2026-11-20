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
| `AddWater` queue + `checkingLiquid` dedupe | PARTIAL | `liquid.rs:93-109` | Dedupe + cap → buffer ported; the 5-tile border guard, `type != 546` carve-out and the `CheckWaterDeath/CheckLavaDeath` kill are gone (`Liquid.cs:1163`, `:1188-1206`) |
| Pass driver (`UpdateLiquid`) | PARTIAL | `liquid.rs::update`/`end_pass` | Fixed `curMaxLiquid/cycles` stride + the `numLiquid` clamp (S1-A-8); `tilesIgnoreWater`/Bubble-379 (S1-L-14); W4-g45 landed the `DelWater` retirement sweep + 254→255 promotion (`Liquid.cs:1109-1119`), the buffer `checkingLiquid` promotion (`:1125-1130`) and the stuck window (`:1131-1149`). No panic/quick-settle, no net broadcast (`Liquid.cs:983-1158`) |
| Per-cell update (`Liquid.Update`) | PORTED | `liquid.rs::update_tile` | Fall + ±1/±2/±3 averaging ported 1:1 including banker's rounding; `kill` bookkeeping matches; R-11-5 landed the reaction block + `delay` gate; W4-c34 landed the underworld evaporation (`Liquid.cs:466-474`); W4-g45 landed the `genRand` 254→255 nudge (`:892-895`), the extra wake calls (`:748-771`/`:821-836`) and the `DelWater` retirement lifecycle. `quickSettle` (`:588-591`/`:944-948`) rides W4-g47 |
| `skipLiquid` skip flag | PORTED | `liquid.rs` (`skip` bitvec, `set_skip`/`is_skip`) | The two writes after a down-transfer (`Liquid.cs:586-587`) + the pass-loop gate (`:1096-1104`) + the quickFall clear (`:1089`); anchors.txt row `set_skip` |
| `delay` (lava 5 / honey 10 / quickFall 10) | PORTED | `liquid.rs` (`Entry.delay`, the reaction block) | R-11-5: the gate throttles only the flow, reactions run every visit (`Liquid.cs:484-490`/`:513-522`); `quickFall` forces 10 in the pass loop (`:1086-1089`); W4-c34 pins the cadence (`liquid_merge_cadence` witnesses, written not run) |
| `quickSettle` / `quickFall` | PARTIAL | `liquid.rs` (`quick_fall` field + the pass-loop arm) | `quickFall` ported (`Liquid.cs:1083-1090`); the `quickSettle` writers (`:35`, `:588-591`, `:944-948`, `:1063-1068`) ride the unported force-settle family (R-11-7) |
| Liquid reactions (`Lava/Honey/ShimmerCheck` → `LiquidCheck` → merge tiles) | PORTED | `liquid.rs` (`lava_check`/`honey_check`/`shimmer_check`/`liquid_check`/`create_liquid_merge_tile`/`get_liquid_merge_types`/`liquid_overwrite_strip`) | R-11-5 (`Liquid.cs:1225-1468`): merge tiles 56/229/230/659 at `num >= 24`, the `tileCut` burns, the container override, the generation strip branch; W4-c34 adds the shimmer+lava/honey 659 and sub-24 down-arm drain witnesses (written not run). Deviations in ANALYSIS "The liquid reactions" |
| Underworld water evaporation | PORTED | `liquid.rs::update_tile` (the `underworld_layer` block) | W4-c34: `Liquid.cs:466-474` — `min(2, liquid)` per visit below `UnderworldLayer` (`Main.cs:2790`, `maxTilesY - 200`), water only, before the zero kill (`:475`); the `underworld_layer >= 0` guard keeps sub-200 fixture worlds out of the band (ANALYSIS W4-c34 note); witnesses written not run |
| `Liquid.DelWater` retirement (`Liquid.cs:1470-1599`) | PORTED | `liquid.rs::del_water` + `retire_sweep` | W4-g45: the sub-2 dry, sub-20 drain, >=20 keep-alive, the lava grass-burn ring, the alch/lily-pad tail (`edit::check_alch_tile`/`check_lily_pad`), the swap-remove that copies x/y/kill but NOT delay (`:1577-1579`); witnesses written not run |
| Stuck detection + panic mode + `QuickWater` | PARTIAL | `liquid.rs` (`stuck`/`stuck_count`/`stuck_amount`) | W4-g45 landed the stuck window (`Liquid.cs:1131-1149`); panic mode + `QuickWater`/`SettleWaterAt` (`:966-1054`, `:104-435`) and the post-load settle `WorldFile.cs:735-760` stay R-11-7 |
| Kill threshold 8 / MP `10+players/3` | PARTIAL | `liquid.rs:21` | 8 hardcoded (correct for single player; MP variant absent) |
| `.wld` liquid bytes | PORTED | `wld/sections/tiles.rs:85-100`, `:237-242` | Liquid amount then kind byte, per the cited `WorldFile.cs:1480-1491`; the liquid byte participates in the RLE key exactly as `bTileHeader` does in the writer |
| Render side (`liquid_draw`, `render/liquids`, waterfall) | PORTED | `liquid_draw.rs:88`, `world_sim/liquids.rs:27-44`, `waterfall/` | Water styles + shimmer draw are ROADMAP-M1.5-LIQUID-2/3 ✅ (re-verified: `liquid_draw.rs::build` builds the per-tile liquid instances, `liquid_type` maps the 4 kinds) |
| Worldgen fills | PORTED | `worldgen/liquids.rs:21` (`place_liquids`) | Lake bowl + ocean shores + lava drips; the `Liquid.QuickWater` *settle* half is absent |

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
- No tests exist for: stuck/panic, kill thresholds, or a `.wld` liquid-byte
  round-trip (`chest.rs:3393`'s DW-3.4 covers the whole-file cycle, not the
  liquid bytes specifically).
- `crates/tr-core/src/liquid.rs::liquid_delwater` + `liquid_nudge_wakes`
  (W4-g45, written NOT run — code-only mode): the delay-inheriting swap-remove
  (`Liquid.cs:1577-1579`), the 254→255 sweep promotion (`:1113-1116`), the
  keep-alive rescue (`:1506-1510`), the lava grass-burn ring (`:1532-1560`),
  the alch/lily-pad tail (`:1580-1598`), the buffer `checkingLiquid` promotion
  (`:1125-1130`), the stuck force-clear (`:1131-1149`), the `genRand` nudge
  promote/keep/stream-advance pins (`:892-895`) and the extra-wake re-queue
  (`:748-771`/`:821-836`).
- `crates/tr-core/src/liquid.rs::liquid_merge_cadence` (W4-c34, written NOT
  run — code-only mode): the lava 1-in-6 / honey 1-in-11 shaft cadences
  (including the skipLiquid visit between falls), the `quickFall` bypass, the
  merge-before-delay ordering, the underworld evaporation (−2/visit, the dry
  retire, the layer-row exemption, the lava/honey/shimmer exemption), the
  down-arm sub-24 drain + water re-type, the water-cell no-burn gate, and the
  shimmer+lava/honey → 659 merges. The R-11-5 `tests` module already pins the
  water+lava/honey/shimmer merges, the 24-unit gate, the down-arm merge, the
  container override, the `tileCut` burn and the merge-type/change-type tables.

## Deviations & risks

1. **Liquid type reactions absent (behavioural, high).** — **FIXED (R-11-5,
   re-verified W4-c34).** `liquid.rs::liquid_check` + `create_liquid_merge_tile`
   port `Liquid.cs:1225-1468`; the merge tiles 56/229/230/659 form at `num >= 24`
   and the `*Check` calls ride every `update_tile` visit. W4-c34 adds the
   shimmer+lava/honey and sub-24 down-arm witnesses.
2. **`delay` cadence missing (behavioural, med-high).** — **FIXED (R-11-5,
   re-verified W4-c34).** `Entry.delay` gates the flow only (lava `delay < 5`,
   honey `delay < 10`, `Liquid.cs:484-490`/`:513-522`); the reactions run every
   visit; `quickFall` forces `delay = 10` in the pass loop (`:1086-1089`).
   W4-c34 pins the 1-in-6 / 1-in-11 cadence with shaft witnesses.
3. **Underworld evaporation missing (behavioural, med).** — **FIXED (W4-c34).**
   `update_tile` removes `min(2, liquid)` per visit from water below
   `UnderworldLayer` (`Liquid.cs:466-474`), before the zero kill (`:475`). The
   existing mass-conservation witnesses run in sub-200 fixture worlds where the
   port's `underworld_layer >= 0` guard keeps the band empty (the C# layer would
   sit below row 0 there — vanilla worlds are ≥ 1200 tall, so the guard never
   fires at runtime); the deviation is recorded in ANALYSIS "The liquid
   reactions" (W4-c34 note).
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
5. **Missing `genRand` nudge (determinism, low-med).** — **FIXED (W4-g45).**
   `Liquid.cs:892-895` (`num == 254 && WorldGen.genRand.Next(30) == 0 → 255`)
   is ported in the 3-neighbour arm; the draw rides `World.frame_rng` (the
   port's `genRand` stand-in) and advances exactly when the C# does (only on
   a 254). RNG census: seed 35 first `Next(30)`=0, seed 0 first=21, verified
   against the `UnifiedRandom` golden peek/next. ANALYSIS "The liquid
   reactions" W4-g45 addendum.
6. **Missing extra wake calls (determinism, low).** — **FIXED (W4-g45).** In
   the 7- and 5-neighbour arms the original re-queues ±1/±2/±3 whenever the
   neighbour *or the centre* changed (`Liquid.cs:748-771`, `:821-836`); the
   port now runs the same centre-diff wake pass before the conditional centre
   write, re-queueing an already-settled neighbour `set_level` left unqueued.
7. **`AddWater` guards (behavioural, low-med).** The 5-tile world-border exclusion
   (`Liquid.cs:1163`), the `tile.type != 546` carve-out, and the
   `CheckWaterDeath/CheckLavaDeath` → `KillTile` of plants/breakables submerged in liquid
   (`:1183-1206`) are all absent; Rust's `is_solid` reads the static `TILE_SOLID` table only.
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

- **S** — Add the `delay` field to `Entry` + the lava 5 / honey 10 gate and the
  `quickFall`-sets-10 pre-pass flag (`Liquid.cs:51/485/516/1087`). Needs a seeded test that a
  lava column falls exactly on the C# cadence.
- **S** — Port underworld water evaporation (`Liquid.cs:466-474`), gated on the port's
  `underworld_layer` constant; update the mass-conservation test to exclude the band.
- **S** — Port the `genRand` 254→255 nudge (`Liquid.cs:892-895`) on the ported RNG seam and
  record the consumed draw in the determinism notes.
- **M** — Port `skipLiquid` (tile flag + the gate in the pass loop + the two writes after a
  fall, `Liquid.cs:586-587/1096-1103`) and switch the pass window to the C# 2500-slot stride
  with the `numLiquid` clamp, so a small active list re-walks every update. Includes a
  byte-level settle comparison test against a known `.wld` liquid layout.
- **M** — Port the liquid reactions: `LiquidCheck` + `GetLiquidMergeTypes` +
  `CreateLiquidMergeTile` + `LiquidOverwriteStrip` (merge tiles 56/229/230/659, the
  `num >= 24` threshold, `tileCut` kills, the `tileSolid[546]` swap) and wire the three
  `*Check` calls into `update_tile`. Depends on `edit.rs`'s `KillTile`/`PlaceTile`/
  `SquareTileFrame` being callable from tr-core (new seam or an out-param).
- **M** — Port `AddWater`'s missing guards: 5-tile border, `type != 546`, and
  `CheckWaterDeath/CheckLavaDeath` → kill.
- **L** — Port the force-settle family: `QuickWater`/`SettleWaterAt`/
  `AttemptToMoveHoney|Lava|Shimmer`, the world-load settle (`WorldFile.cs:735-760`,
  `quickSettle`), stuck detection and panic mode. Depends on M above (merge tiles share
  `CreateLiquidMergeTile`) and on the save pipeline (task 42) for the load hook.
- **L** — MP parity: `curMaxLiquid`/`cycles` from the active-player count and the
  `kill >= 10 + players/3` threshold, plus `_netChangeSet` → the net module. Blocked on the
  netcode module (17).
