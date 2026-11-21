# 374 — Liquid SEAL: the steps-6/7 merger hunks + the liquid-family bible

> Chain-banner: 71 → 374 per research-number-registry §10.2 (d54 tip
> `08d21666e`): `71-liquid-seal-hunks.md` → `374-liquid-seal-hunks.md` —
> 71 belongs to e20's wiring-device-reconcile; g46's 69 KEEP +
> 70-fork→252 unchanged. Cite THIS registry §10.

Date: 2026-11-21. Author: W4-g46 (seal phase, code-only — no cargo/tests run).
Branch `task/W4-g46-liquid-evaporation`, continuing the RE-FOLD union tip
`9bc722e5`. This note publishes the **exact mechanical hunks** the worldgen
master merge applies for research/70's steps 6 (the f25 gen-module dedupe)
and 7 (the f24/c30/g36 producer wiring), with the seam signatures they call,
plus the one-paragraph liquid-family bible. The runtime surfaces these hunks
call are ALL on this branch (the RE-FOLD carried them); f25's
`worldgen/settle_liquids.rs` itself stays on `task/W4-f25-worldgen-passes-8`
(tip `84b90360`) and lands at the worldgen merge — the hunks below are its
deduped form, ready to apply.

## The seam signatures (ON this branch, `crates/tr-core/src/liquid.rs`)

Every hunk below calls only these public surfaces — no new engine code:

| Seam | Signature | C# home |
| --- | --- | --- |
| gen context | `LiquidSystem::set_gen_settle_context(&mut self, water_line: i32, lava_line: i32, remix: bool)` | arms `generating`+`generating_world`+`remix_world_gen`+`gen_water_line`+`gen_lava_line` (`Liquid.cs:178-192` context; `GenVars.cs:62-64`) |
| shimmer position | `LiquidSystem::set_gen_shimmer_position(&mut self, pos: Option<(i32, i32)>)` | `GenVars.shimmerPosition` (`GenVars.cs:282`, the g36-A1 `WorldGen.cs:15241` write); `None` = the `Vector2D.Zero` reset no-op |
| dungeon context | `LiquidSystem::set_gen_dungeon_context(&mut self, ctx: Option<GenDungeonContext>)` | `GenVars.dungeonGenVars` stand-in; `GenDungeonContext { outer_bounds: Vec<(i32,i32,i32,i32)>, extra_dungeon_walls: Vec<u16> }` (half-open `(left,top,right,bottom)`; `None` = the C# empty list) |
| gen ignore | `LiquidSystem::world_gen_tiles_ignore_water(&mut self, ignore: bool)` | `Liquid.worldGenTilesIgnoreWater` (`Liquid.cs:75-81`, the doors 10/192 + living-wood/leaf 191/190 `kind_solid` fold) |
| quick water | `LiquidSystem::quick_water(&mut self, world: &mut World, min_y: i32, max_y: i32)` | `Liquid.QuickWater` (`:104-148`); `-1` args resolve to 3 / `height-3`; the `ShimmerRemoveWater` arms run inside (`:108`/`:138`, `generating_world`-gated) |
| water check | `LiquidSystem::water_check(&mut self, world: &mut World)` | `WorldGen.WaterCheck` (`WorldGen.cs:80415-80494`) |
| gen loop | `LiquidSystem::gen_settle_loop(&mut self, world: &mut World)` | `WorldGen.cs:16158-16206` verbatim (Part-2 twin `:21265-21305` classic-identical): WaterCheck → quickSettle → 10 rounds × `num3*5` budget with live re-raise → WaterCheck between → ClearPendingLiquid → quickSettle down |
| kill drain | `LiquidSystem::drain_edit_kills(&mut self) -> Vec<crate::edit::TileKill>` | the gen-tail `edit_kills` drain (research/70 §9.3 — the records must not leak across the per-pass instance) |
| reaction check | `LiquidSystem::liquid_reaction_check(&mut self, world: &mut World, x: i32, y: i32, kind: u8)` | the `AttemptToMove*` tail's `LavaCheck`/`HoneyCheck`/`ShimmerCheck` dispatch (`Liquid.cs:284-286`/`:1445-1468`) — only needed if a gen-local `settle_water_at` is kept (the recommended unification drops it) |

The producer-side seams (on the f24/c30/g36 lanes, carried at the worldgen
merge): `GenVars.shimmer_position: Option<(i32,i32)>` (g36 `pipeline.rs:376`,
written `gv.shimmer_position = Some((x,y))` at the Shimmer step `:2093`);
`GenVars.dither_snakes: Vec<dither_snake::DitherSnake>` (c30 `pipeline.rs:377`);
f24 `dungeon_genvars::create_potential_dungeon_bounds(max_x, max_y,
world_surface, world_surface_high, underworld_layer, left_dungeon,
dungeon_entrance_is_underground) -> (DungeonBounds, DungeonBounds)` (inner,
outer) + `DualDungeonSetup { dungeons: Vec<DitherSnakeDungeon>, styles:
Vec<Vec<SnakeStyle>> }` where `DitherSnakeDungeon.outer: DungeonBounds`
(`.left()/.top()/.right()/.bottom()`, XNA Right/Bottom exclusive) and
`SnakeStyle.brick_wall: u16`; c30's duplicate producer
`dither_snake::create_potential_dungeon_bounds(max_tiles_x, max_tiles_y,
world_surface, left_dungeon) -> ((i32,i32,i32,i32), Option<(i32,i32,i32,i32)>)`
(bounds + degenerate-expanded hitbox) dedupes onto f24's at the master merge
(g36 research/88 owns that matrix).

## Step 6 — the f25 gen-module dedupe (DELETE the private copies)

f25's `worldgen/settle_liquids.rs` (`84b90360`, 611 lines) carries gen-local
copies of surfaces the RE-FOLD put on `LiquidSystem`. The merger KEEPS the
module (the pass body + the ocean retype sweep + `LiquidInteractionsCleanup`)
and DELETES the private copies, calling the base seams. The mechanical hunks:

### Hunk 6.1 — `mod.rs` registration (keep f25's)

`crates/tr-core/src/worldgen/mod.rs`, after `pub mod progress;` (line 40):
```rust
 mod settle_liquids;
```
(f25 `mod.rs:41`; the union's `mod.rs` lacks it — carry verbatim.)

### Hunk 6.2 — the pass body `settle_liquids` (replace the private loop)

f25 `settle_liquids.rs:56-123` — the ocean retype sweep (`:16137-16146`)
stays; the per-pass instance arming + the hand-rolled 10-round loop
(`:76-120`) DELETE, replaced by the base seams:
```rust
pub(crate) fn settle_liquids(world: &mut World, gv: &GenVars, remix: bool) {
    let (width, height) = (world.width, world.height);
    let ocean_level = (world.world_surface + world.rock_layer) / 2.0 + 40.0;
    // --- the ocean retype sweep (`WorldGen.cs:16137-16146`) — KEEP verbatim ---
    for i in 0..width {
        for j in 0..height {
            if ocean_depths(i, j, width, ocean_level) {
                let idx = world.index(i, j);
                let t = &mut world.tiles[idx];
                if t.liquid > 0 && t.liquid_kind() != LiquidKind::Water {
                    t.clear_liquid_kind();
                }
            }
        }
    }
    // The per-pass instance — the C# statics' stand-in.
    let mut sys = LiquidSystem::new(world);
    // f25's `set_generating(remix)` MAPS onto the fuller strict-flag context.
    sys.set_gen_settle_context(gv.water_line, gv.lava_line, remix);
    // Step-7 producer arming (the context setters are gen-driver-only).
    sys.set_gen_shimmer_position(gv.shimmer_position);
    sys.set_gen_dungeon_context(gen_dungeon_context(world, gv));
    // `Liquid.worldGenTilesIgnoreWater(true)` (`WorldGen.cs:16156`).
    sys.world_gen_tiles_ignore_water(true);
    // `Liquid.QuickWater(3)` (`:16157`) — the base method runs both
    // ShimmerRemoveWater arms + the bubble/ignore wrap internally.
    sys.quick_water(world, 3, world.height - 3);
    // The shared gen settle loop (`:16158-16206`) — WaterCheck + the 10-round
    // budget + ClearPendingLiquid + quickSettle down, all inside.
    sys.gen_settle_loop(world);
    // `Liquid.worldGenTilesIgnoreWater(false)` (`:16207`).
    sys.world_gen_tiles_ignore_water(false);
    // The gen-tail edit_kills drain (research/70 §9.3): DISCARD — the C#
    // KillTile forces noItem during generation (`WorldGen.cs:63546-63548`),
    // so the DelWater grass-ring/lily kills produce no loot at gen time; the
    // records are dropped, never resolved into items. RECORDED drain choice.
    let _ = sys.drain_edit_kills();
    // `Main.tileSolid[484] = false` (`:16208`) — the load-order poke the
    // port's static tables do not model (f25 module doc / desert.rs precedent).
}
```
DELETE from f25: the private `quick_water` (`:131-148`), `shimmer_remove_water`
(`:155-178`), `settle_water_at` (`:185-329`), `attempt_to_move` (`:336-355`),
`in_any_potential_dungeon_bounds` (`:446-461`), `clamp_bound` (`:465-467`),
`ocean_level` (`:470-472`), `at`/`kind_code`/`set_kind_code` (`:478-500`) — all
superseded by the base `quick_water`/`settle_water_at`/`shimmer_remove_water`/
`GenDungeonContext`/`scene::ocean_depths`. KEEP: `ocean_depths` (`:45-51`, or
dedupe onto `crate::scene::ocean_depths`), `liquid_interactions_cleanup`
(`:363-437`, the `WorldGen.cs:23672-23799` carry — call it from `quick_water`'s
gen tail; the base `quick_water` cites it as f25's, so the module keeps it and
the merger wires the call after the sweep). The `LiquidInteractionsCleanup`
call site: f25's private `quick_water` ran it at `:147`; with the base
`quick_water` the merger adds `liquid_interactions_cleanup(world)` after
`sys.quick_water(...)` (the `generatingWorld && !skyblock` gate is the gen
driver's — classic worlds run it).

### Hunk 6.3 — the ORIGIN-wall fix (already on the base)

f25's gen-local `settle_water_at` read the LANDING cell's wall
(`settle_liquids.rs:235-241`, `world.tiles[world.index(num, num2)].wall`); the
C# `tile` binds at the ORIGIN (`Liquid.cs:152`), so the base
`settle_water_at` reads `origin.wall` (`liquid.rs:1990`). The DELETE of the
private copy (6.2) carries the fix automatically — no separate hunk. The
dungeon predicate rides `GenDungeonContext::in_any_bounds` +
`is_dungeon_wall` (the `Main.wallDungeon` base ∪ the style extensions).

### Hunk 6.4 — the pipeline step (keep f25's slot)

`crates/tr-core/src/worldgen/pipeline.rs`, before
`step!("remove water", "Remove Water From Sand", step_remove_water)` (union
line 1368):
```rust
    // The vanilla Settle Liquids slot (`WorldGen.cs:16132`): after Jungle
    // Chests (`:15977`), before Remove Water From Sand (`:16211`). The port's
    // chain groups the later carve/decor family between them, so the step
    // sits immediately before `remove water` — every liquid-producing pass
    // (Lakes, the ocean family, the jungle/shrine water) has already applied.
    step!("settle liquids", "Settle Liquids", step_settle_liquids),
```
(f25 `pipeline.rs:1376-1382`.) The step adapter (f25 `pipeline.rs:2639-2652`):
```rust
fn step_settle_liquids(
    world: &mut World,
    gv: &mut GenVars,
    _streams: &mut Streams,
    opts: &GenOptions,
    _noise: &Noise,
    _pass_config: &ConfigSection,
) {
    let remix = opts.flags.contains(crate::seeds::SeedFlags::REMIX);
    settle_liquids::settle_liquids(world, gv, remix);
}
```
The `Settle Liquids Again` Part-2 twin (`WorldGen.cs:21246`, union
`pipeline.rs:1096` PassDecl) is classic-identical to `gen_settle_loop` — the
special-seed `extraLiquid`/`notTheBees` arms are d107/c30's; the classic
driver calls the same `gen_settle_loop`.

## Step 7 — the f24/c30/g36 producer wiring (the context arming, one place)

The gen driver arms the three context setters (6.2 calls them); the producer
conversion is the `gen_dungeon_context` helper the merger adds to
`settle_liquids.rs` (or `dungeon_genvars.rs`):

### Hunk 7.1 — the dungeon context producer

```rust
/// The `GenVars.dungeonGenVars` → [`GenDungeonContext`] conversion
/// (research/70 step 7): each `DualDungeonSetup.dungeons[i].outer` hitbox
/// becomes a half-open `(left, top, right, bottom)` row (XNA Right/Bottom
/// exclusive — `DungeonBounds.cs:33-39`), plus the union of the style quads'
/// `BrickWallType` + `ExtraWallTypes` BEYOND `Main.wallDungeon`
/// (`DungeonCrawler.cs:155-181`). On a classic single-dungeon world the
/// extension is empty (`isDungeonWall == Main.wallDungeon`) and the bounds
/// are the one `create_potential_dungeon_bounds` outer rect.
fn gen_dungeon_context(world: &World, gv: &GenVars) -> Option<GenDungeonContext> {
    let mut outer_bounds = Vec::new();
    let mut extra_dungeon_walls = Vec::new();
    if let Some(dual) = &gv.dual_dungeons {
        // Dual path (f24 `GenVars.dual_dungeons: Option<DualDungeonSetup>`,
        // `pipeline.rs:376`; c30's `dither_snakes` producer dedupes onto it):
        for d in &dual.dungeons {
            let o = &d.outer; // DungeonBounds — XNA Right/Bottom exclusive
            outer_bounds.push((o.left(), o.top(), o.right(), o.bottom()));
        }
        for quad in &dual.styles {
            for s in quad {
                if !dungeon_features::is_dungeon_wall(s.brick_wall) {
                    extra_dungeon_walls.push(s.brick_wall);
                }
                // + the style's ExtraWallTypes (the SnakeStyle paint slice
                //   carries brick_wall only; the ExtraWallTypes rows are
                //   c32's full table — the merger extends SnakeStyle or
                //   reads the DungeonGenerationStyles statics directly).
            }
        }
    } else {
        // Single-dungeon fallback (classic): the one outer potential rect.
        // The port's dungeon side rides `spans().dungeon_x` (M5-BIOME).
        let (w, h) = (world.width, world.height);
        let underworld_layer = h - 200; // Main.UnderworldLayer (Main.cs:2790)
        let (_inner, outer) = dungeon_genvars::create_potential_dungeon_bounds(
            w, h, world.world_surface, world.world_surface, underworld_layer,
            /*left_dungeon=*/ gv.spans().dungeon_x < w / 2,
            /*dungeon_entrance_is_underground=*/ false,
        );
        outer_bounds.push((outer.left(), outer.top(), outer.right(), outer.bottom()));
    }
    if outer_bounds.is_empty() {
        return None; // the C# empty dungeonGenVars list — the arm stays dead
    }
    Some(GenDungeonContext { outer_bounds, extra_dungeon_walls })
}
```
The style-wall extension table (`DungeonGenerationStyles.cs`, the
`BrickWallType`/`ExtraWallTypes` per style): Cavern 349 (no extra), Snow 71
(+40), Desert 187 (+216/223), Corruption 3 (+188/189/190/191), Crimson 83
(+192/193/194/195), Hallow 28 (+200/201/202/203), Jungle 64
(+204/205/206/207/15), Temple 87 (no extra), the colour styles 7/8/9 (base
`Main.wallDungeon`, no extension). The base set is
`dungeon_features::DUNGEON_WALLS = [7,8,9,94,95,96,97,98,99]`; only walls
OUTSIDE it enter `extra_dungeon_walls` (the `is_dungeon_wall` union dedupes).

### Hunk 7.2 — the shimmer producer (g36-A1, already wired on g36)

`gv.shimmer_position` is written by g36's Shimmer step
(`pipeline.rs:2093`, `gv.shimmer_position = Some((x, y))`, the
`WorldGen.cs:15241` write); 6.2's `sys.set_gen_shimmer_position(gv.shimmer_position)`
copies it. `None` (the sub-vanilla width guard returns early) = the C#
`Vector2D.Zero` reset — the `ShimmerRemoveWater` circle sits off-world, a
no-op. No new producer code; the seam is the carry.

### Hunk 7.3 — the edit_kills gen-tail drain choice (RECORDED)

6.2 drains `edit_kills` and DISCARDS (`let _ = sys.drain_edit_kills()`).
Rationale: the C# `WorldGen.KillTile` forces `noItem = true` while
`isGeneratingOrLoadingWorld` (`WorldGen.cs:63546-63548`), so the DelWater
grass-ring/lily/`Check*` kills produce NO loot during generation — resolving
them into items would be a deviation. The records are dropped at the pass
tail (the per-pass instance boundary, `WorldGenerator.cs:508`'s
`numLiquid == 0` invariant). This is the drain choice research/70 §11's f25
gen-hash RE-DERIVE row demands be picked + recorded: **DISCARD**. The
world-hash pins (`SEED42_512`/`DEFAULT_WORLD` + `the_pass_is_deterministic`)
RE-DERIVE at the gate — the real DelWater drain/tail + swap-remove buffer +
origin-wall fix change the gen settle vs f25's discard-only `end_pass`; do
not silence.

## The liquid-family bible (one paragraph)

The liquid family is ONE cellular mass-transfer engine — `Liquid.cs`'s 24
members ported whole into `crates/tr-core/src/liquid.rs` as `LiquidSystem`
(instance-per-world standing in for the C# statics, `ReInit` the swap
contract), with NO "pressure" constant and NO `LiquidEdge`/`QuickShimmer`
(both phantom symbols): a per-cell `Update` (`:449-964`) runs the underworld
water evaporation (`:466-474`, `min(2,liquid)` below `UnderworldLayer =
maxTilesY-200`, water-only), the per-kind reaction block (`LavaCheck`/
`HoneyCheck`/`ShimmerCheck` EVERY visit, the `delay` throttle gating only the
flow — lava 1-in-6, honey 1-in-11, water/shimmer ungated, `quickFall`
bypassing both), the fall/±1/±2/±3 row averaging (banker's rounding, the
sub-3 −1 bias, the LIVE post-evap/post-reaction `own_now` read), the Next(30)
254→255 surface nudge on the per-consumer `surface_rng`, and the 7/5-neighbour
extra wakes; the pass driver `UpdateLiquid` (`:983-1158`) walks the
`curMaxLiquid/cycles` stride with the `numLiquid` clamp, the `skipLiquid`
flag, the `kill>=8` retirement sweep + 254 promotion, the `LiquidBuffer`
swap-remove promotion, the stuck band + panic, and the `DelWater`
(`:1470-1599`) drain/keep-alive/grass-ring/alch-lily tail with the
`square_tile_frame_record` loot seam on `edit_kills`; `AddWater`
(`:1160-1207`) carries the 5-tile border + grate-546 carve-out + inline
`CheckWaterDeath`/`CheckLavaDeath` kill; the force-settle family
(`QuickWater`/`SettleWaterAt`/`AttemptToMove*`, `:104-433`) with the gen
re-type (waterLine/lavaLine/dungeon ORIGIN-wall) + `ShimmerRemoveWater`, the
WIRED world-load settle (`WorldFile.cs:733-764`), the runtime settle pump
(`WorldGen.cs:71526-71534` `skipCount` on the Liquid statics, `ReInit`
resets), the shared gen settle loop (`:16158-16206`) + the f25 export surface,
and the e21 pump/settle compose (`Wiring.cs:512/:516/:521` →
`pump_wakes` → `wake_around(r=1)`) are all ported; the reduced-liquids caps
(`Setting_UseReducedMaxLiquids`, `slowliquids=1`) ride settings/ReInit/gfx/
the pass-head OR-arm; the render side (`liquid_draw.rs`'s
`LiquidRenderer.InternalPrepareDraw` Pass-4 edges + the `HasFullWater`
DrawBlack gate + the wind-driven `_frameState` clock, `waterfall/`, the
`SetWaveMaskData` viscosity chain) is the draw half; the bucket/sponge use
arms (`Player.ItemCheck_UseBuckets`, `Player.cs:46318-46504`, the twelve kinds
+ `PutItemInInventoryFromItemUsage` + the five `PlaySound(19)` splashes) are
the player-facing half in `world_sim/editing/buckets.rs`; the only unported
members are MP (`NetSendLiquid`/the broadcast tails/the MP cycle scaling),
UI (`UpdateProgressDisplay`) and dev tooling
(`AddGameplaySnapshotComponents`), all cited out-of-scope — the sim gaps-list
is EMPTY (research/70 §10), the runtime-inert gen-fields contract holds
(`generating`/`generating_world`/`gen_ignore_water`/`quick_settle` default
false, the context setters gen-driver-only, `underworld_layer`
runtime-faithful, `surface_rng` advances only on a 254 average, `skip_count`
resets in `re_init`, `app.liquid_tick` stays DEAD), and every witness is
written-not-run until the gate command (`cargo test --workspace -- --skip
migrates_version_2_kinds_to_item_ids`, judged by the `test result: ok` lines).

## Seal status

- Steps 1-5 + the R-12-5 drivers: ON this branch (the RE-FOLD
  `e7626b72`/`f588d53c`/`16fd6ad7`/`d95911ce` + docs `9bc722e5`).
- The g47 closeout (`5c09a68b`) + the g45 doc-sweep (`6f5bd2a2`): carried by
  the seal (the per-liquid witnesses, the down-arm LIVE post-burn re-read,
  the fall-arm post-evap witness, the shimmer-flow pin, the bucket module,
  the render-gaps closure).
- Steps 6/7: the hunks above — the surfaces they call are ON this branch;
  f25's `settle_liquids.rs` lands at the worldgen master merge with these
  dedupes applied; the f25 gen-hash RE-DERIVE obligation stands (drain
  choice DISCARD, recorded §7.3).
- research/69 SUPERSEDED-BY-70; research/70 §8-12 carried + the SEAL status
  notes; this note (374, was 71 — registry §10) is the steps-6/7 hunk record
  + the bible.