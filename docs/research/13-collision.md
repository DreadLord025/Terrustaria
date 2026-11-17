# 13 — Collision / HitTile / scans

> Wave 2 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

C#: `decompiled/Terraria/Collision.cs` (4 598 lines, the static `Collision` class) plus its
callers in `Player.cs` (`DryCollision`/`WetCollision`/`SlopingCollision`,
`Player.cs:23436-23541`, `:28168-28195`, `:28210`) and `Projectile.cs`/`NPC.cs`
(acquisition scans). Responsibility: every tile-relative geometry test in the game —
entity-vs-tile movement collision (axis AABB + slope slides + step up/down), liquid
overlap (wet flags, water line), tile-hit scanning for swung weapons, hurting-tile
damage, sticky-tile adhesion, and the line-of-sight walks the AI uses to decide
"can this entity see/hit that one".

Rust: `crates/tr-app/src/collision.rs` (the ported subset, module doc :1-5 lists it as
`TileCollision`/`SlopeCollision`/`StepUp`/`StepDown` only) consumed by
`crates/tr-app/src/player.rs::move_vanilla` (`:5584-5687`) and the minecart resize
(`:1666`, `:4628`); the AI's line-of-sight is the faithful `CanHit`/`CanHitLine`/
`SolidCollision` set in `crates/tr-app/src/projectile.rs::can_hit` (`:8669-8718`)
and `::can_hit_line` (`:8745+`) plus `player.rs::solid_collision` (`:1476-1501`,
all three `pub(crate)` and called by the NPC AI). There is no `world_sim/player/collision.rs`
— module 03's note pointed at `crates/tr-app/src/collision.rs`, which is where the code
actually is.

## Key entities

C# (`Collision.cs`):
- `TileCollision` (`:2292-2453`) — the axis-resolving core: scan window `(x/16)−1 .. (x+w)/16+2`, `maxTilesY−40` clamp, halfBrick shrink (`:2342-2347`), the slope `flag2/flag3` pass-through gates (`:2352-2380`), platform one-way (`tileSolidTop` excluded from the X/Y arms, `fallThrough` gate on the floor arm with the `Velocity.Y <= 1 | fall2` test, `:2384`), the four directional arms (`:2381-2449`) with the `num/num2/num3/num4` last-tile bookkeeping and the `gravDir` ±0.01 landing nudge (`:2394`, `:2444`).
- `SlopeCollision` (`:1791-2031`) — slope/stairs slide; resets `Collision.stair`/`stairFall`, the platform one-way rules (`:1846-1860`: `Velocity.Y < 0` blocked, the `−1−|Velocity.X|` penetration window, the slope-1/2 up/down-rise exclusion), halfBrick offset, the bitsByte[1..4] diagonal push-back after a recursive `TileCollision` call (`:1996-2029`).
- `StepUp` (`:3626-3774`) / `StepDown` (`:3564-3624`) — the 1-tile ledge lifts (≤ 16.1 px, `stepSpeed` 1/2, `gfxOffY` absorbs the pop), `IgnoredByNpcStepUp` via `specialChecksMode`, `holdsMatching`, `bottomWorld/16 − 42` floor row.
- `WetCollision` (`:1572-1651`) / `LavaCollision` (`:1653-1684`) — box scan with the liquid surface offset `(256 − liquid)/32 * 2` (`:1610-1612`) and the sloped-tile-above-liquid support arm (`:1628-1646`); `WetCollision` sets the static `Collision.honey`/`Collision.shimmer` flags (`:49`, `:51`).
- `HitTiles` (`:3063-3108`) / `HitTilesInACircle` (`:3016-3061`) — weapon hitbox → `WorldGen.KillTile(i, j, fail: true, effectOnly: true)`; the circle variant adds the centre-distance `< (Width+Height)/4` test (`:3054`).
- `StickyTiles` (`:3362-3435`) — tiles 51 / 229 (slope 0), ±1 px window, the dust-30 trail at `|vel| > 0.7 && rand.Next(30) == 0`.
- `AnyHurtingTiles` (`:3110`) / `CanTileHurt` (`:3206`) / `SwitchTiles` (`:3227`) / `BuildTileContacts` (`:2135-2290`) / `noSlopeCollision` (`:2033-2133`) / `AdvancedTileCollision` (`:4422`) / `DrownCollision` (`:1383`) / `GetWaterLine` (`:1493-1560`).
- LOS: `CanHit` (`:371-423`), `CanHitWithCheck` (`:480-586`), `CanHitLine` (`:588-778`), `HitLine` (`:780`), `SolidTiles*` (`:3437-3561`), `EmptyTile` (`:1359-1382`).

Rust:
- `collision.rs::tile_collision` (`:70-198`) — 1:1 with `TileCollision`, gravDir 1 / hoik true, `ignoreDoors`/`ignoreAetheriumPlatforms` not exposed.
- `collision.rs::slope_collision` (`:209-400`) — 1:1 with `SlopeCollision`, `fall` param only.
- `collision.rs::step_up` (`:405-499`), `::step_down` (`:611-661`), `::tile_collision_in_steps_of_16` (`:505-545`), `::try_changing_size_from_bottom_center` (`:552-606`).
- Helpers `solid` (`:15-26`, includes the `WorldGen.cs:15715` leaf-192 carve-out), `support_solid` (`:37-39`), `scan_bottom` (`:54-60`).
- `player.rs::move_vanilla` (`:5584-5687`) — DryCollision/WetCollision pipeline; wet flags from the mid tile (`:2484-2493`); the liquid displacement (`:5643-5660`); `liquid_move_speed` (`:755-758`).
- `player.rs::solid_collision` (`:1476-1501`, `pub(crate)`) — a 1:1 `Collision.SolidCollision`, the diving flyer's crash gate (`NPC.cs:65679`).
- `projectile.rs::can_hit` (`:8669`) / `::can_hit_line` (`:8745`) — faithful ports, cited to the C# line ranges; both `pub(crate)` and called by the NPC AI sites (S1-D-7).

## Dependencies

- Calls into: `tr-core::tile::Tile`, `tr_core::tiledata` (`TILE_SOLID`, `TILE_SOLID_TOP`,
  `SET_PLATFORMS`), `tr_core::World::tile`, `tr_core::TILE_PX`.
- Called by: `player.rs::move_vanilla` (the fixed-step player movement),
  `player.rs::try_landing` (`:3611`, `Mount.TryLanding`), the minecart resize/dismount
  seam (`:1666`, `:4628`) → minecart riding (M2-PHYS-7), `tile_collision_in_steps_of_16`
  ← `try_changing_size_from_bottom_center`.
- Shared data: `Tile.kind/flags(FLAG_ACTIVE, FLAG_INACTIVE)/slope/frame_y/half_brick/liquid/liquid_kind`
  (`tr-core/tile.rs`); `TILE_SOLID*`/`SET_PLATFORMS` generated in `tr-core/src/tiledata/`.
- The liquid damage/slow side is split: flags in `player.rs` (`:2488-2493`,
  `movement_speed_for` `:169-176`), gravity in `liquid_gravity` (M2-PHYS-3), tile
  damage (`AnyHurtingTiles`/`CanTileHurt`) has no home yet.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `TileCollision` (`:2292-2453`) | PORTED | `collision.rs::tile_collision` `:70-198` | Order, constants (num6 `(cy1+3)*16`, the ±0.01 landing nudge at `:186`), halfBrick, slope flag2/flag3 gates, the four arms verbatim. `gravDir` fixed to 1, `hoik` fixed true, `ignoreDoors`/`ignoreAetheriumPlatforms` unexposed |
| `SlopeCollision` (`:1791-2031`) | PORTED | `collision.rs::slope_collision` `:209-400` | Platform one-way rules `:255-272`, stair/stairFall `:331-366`, bitsByte push-back `:373-392`. No `ignoreAetheriumPlatforms`; `gravity` param (unused in C#) dropped |
| `StepUp` / `StepDown` (`:3564-3774`) | PORTED | `collision.rs::step_up` `:405-499`, `::step_down` `:611-661` | 16.1 px cap, `stepSpeed` 1/2/2.5, `gfxOffY`, `bottomWorld/16−42` row. `specialChecksMode` (`IgnoredByNpcStepUp`) not plumbed; caller passes `holds_matching = false` |
| `DryCollision` pipeline (`Player.cs:23436-23541`) | PORTED | `player.rs::move_vanilla` `:5584-5687` | StepDown → StepUp → TileCollision → wet displacement → SlopeCollision → `stair` gfx (`:5675-5678`) matches the C# order, incl. the cart `height−10` body (`:5631`) |
| `WetCollision` / `LavaCollision` (`:1572-1684`) | PARTIAL | `player.rs:2484-2493` | Wet flags come from a **single mid-tile** sample, not the C# box scan: no `(256−liquid)/32*2` surface offset, no sloped-tile support arm (`:1628-1646`); `honey`/`shimmer` → separate bools. Shallow-liquid standing and slope-over-liquid wet detection diverge |
| Liquid slow/damage flags (`Player.cs:27008-27017`, `:27728-27910`, `:28168-28195`) | PORTED (flags+speed) | `player.rs:169-176`, `:755-758`, `:5643-5660` | `movement_speed_for` (honey 0.25, shimmer 0.375, water/lava 0.5), `liquid_move_speed` applied to free axes only. ROADMAP M2-PHYS-3 claim verified PASS |
| `HitTiles` / `HitTilesInACircle` (`:3016-3108`) | PORTED | `collision.rs::hit_tiles` / `hit_tiles_in_a_circle` (S1-E-13d) | The scan is geometry-only: `KillTile(effectOnly: true)` returns at `WorldGen.cs:63704-63707` before any removal, so nothing is destroyed — the dust half is `collision.rs::tile_dust_amount`. The ~30 `Projectile.cs` call sites have no generic projectile-to-tile seam yet |
| `StickyTiles` (`:3362-3435`) | PORTED | `collision.rs::sticky_tiles` (S1-E-13c) + `player.rs::sticky_movement` | The `Player.sticky` field (`player.rs:750`, `FloorVisuals`) stays the floor-flag, not the C# adhesion scan (tiles 51/229 with the dust-30 trail) |
| `AnyHurtingTiles` / `CanTileHurt` (`:3110-3226`) | PORTED | `touch_damage.rs::hurt_tiles` / `can_tile_hurt` (S1-E-13a) | |
| `SwitchTiles` (`:3227-3360`) | PORTED | `collision.rs::switch_tiles` | |
| `BuildTileContacts` (`:2135-2290`) | PORTED | `collision.rs::build_tile_contacts` (S1-E-13e) | Landed with its caller, `apply_conveyor_belt_movement_to_velocity` (`:3918-4188`) |
| `noSlopeCollision` (`:2033-2133`) | PORTED | `collision.rs::no_slope_collision` (S1-E-13d) | Only caller is `NPC.Collision_MoveSnailOnSlopes` (`NPC.cs:92782`); the snail family is unported, so it is `#[allow(dead_code)]` |
| `AdvancedTileCollision` (`:4422-4566`) | PORTED | `collision.rs::advanced_tile_collision` (S1-E-13d) | Only caller is `NPC.Collision_MoveSandshark` (`NPC.cs:92732`); the sandshark family is unported, so it is `#[allow(dead_code)]` |
| `DrownCollision` / `GetWaterLine` (`:1383-1569`) | PORTED | `collision.rs::drown_collision` / `get_water_line` / `get_water_line_iterate` (S1-E-13b) + `player.rs::check_drowning` | |
| `CanHit` (`:371-423`) | PORTED | `projectile.rs::can_hit` `:8669-8718` | Anchor clamps, dominant-axis walk, full-height flank test, do/while exit — cited to `:388-423`, `:476-477`. `pub(crate)`; the 15 `1,1`-anchor and box-form NPC AI sites call it (S1-D-7) |
| `CanHitLine` (`:588-778`) | PORTED | `projectile.rs::can_hit_line` `:8745+` | Faithful; called by the Vortex Rifleman arm (`NPC.cs:58587`) and the Ancient Cultist's ritual gate (`NPC.cs:39275`) — the `#[allow(dead_code)]` is gone (S1-D-7) |
| AI line-of-sight | PORTED (S1-D-7) | `projectile.rs::can_hit` / `can_hit_line` + `player.rs::solid_collision` (S1-D-7) | The old `npc_ai.rs::line_clear` half-tile sampler is deleted; the sites now run the faithful walks (centres `1,1` for `CanHit`, `0,0` for `CanHitLine`, `position/width/height` boxes where the C# boxes) |
| `EmptyTile` (`:1359-1382`) | PORTED | `growth.rs::empty_tile_check` `:366` (+ `biome_detail.rs:116`) | PORT_MAP `:50` marks done (M7-SMARTCURSOR-1) — the `ignoreTiles: false` arm only |
| `TileCollisionInStepsOf16` (`:2455-2483`) | PORTED | `collision.rs::tile_collision_in_steps_of_16` `:505-545` | `Width−1` slope probes, `vector5.Length()` return |
| `TryChangingSizeFromBottomCenter` (`:2486-2533`) | PORTED | `collision.rs:552-606` | Split arithmetic kept verbatim incl. the no-op second add |

PORT_MAP `:167` ("Collision.cs / Physics/* → collision.rs — partial") was **confirmed
PARTIAL** at the wave-2 audit; after S1-E-13a/b/c/d the collision family (the movement
routines, the hurting/sticky/drowning families, the hit scans, the contact enumeration
and the two alternate collision variants) is 1:1. What stays absent is the rest of the
4 598-line file: the line-of-sight walks beyond `CanHit`/`CanHitLine`, the laser/mass
scans, and the wet-flags box scan (line above, PARTIAL).

## Tests

- `crates/tr-app/src/collision.rs::tests` (25 tests). S1-E-13d's six, one per function:
  `the_slope_free_collision_ignores_the_ramp_gate`,
  `the_sandshark_collision_skips_its_set_and_nudges_by_gravity`,
  `the_hit_scan_reports_the_solid_cells_under_the_swept_box`,
  `the_circle_hit_scan_drops_the_box_corner`,
  `the_contacts_name_the_side_and_the_overlap`,
  `the_tile_dust_amount_table_matches_the_source`. The earlier rows:
  `the_actuated_block_is_walk_through`,
  `dismount_space_check_blocks_growth_under_a_two_tile_ceiling`, the drowning family's
  four and the sticky family's two.
- `crates/tr-app/src/player.rs` movement tests: `walks_down_a_slope_smoothly` (`:7858`),
  `ceiling_slopes_block_by_their_triangle` (`:8099`), `standing_on_a_ground_slope`
  (`:8122`), plus the collides-filtered companions around `:5757`.
- `wiring.rs::solid_tiles` (`:1086`) covers the actuation-aware solid test.
- `npc.rs::tests` (S1-D-7): `the_stardust_cell_drops_its_chase_when_the_tile_walk_sees_a_wall`
  (`CanHit`'s both-flanks-solid block) and `the_vortex_rifleman_does_not_arm_through_the_tile_walk`
  (`CanHitLine`'s any-of-three block).
- No test covers: `StickyTiles`, `HitTiles`, `AnyHurtingTiles`, `WetCollision`'s liquid
  surface offset.

## Deviations & risks

1. **Wet flags read one tile, not a box** (behavioural, medium). C# `WetCollision` scans
   the hitbox against every liquid tile using the surface offset and accepts a sloped
   tile above the liquid (`Collision.cs:1610-1612`, `:1628-1646`); the port samples the
   mid tile only (`player.rs:2487`). Consequence: the feet-in-shallow-water case and the
   "standing on a slope over water" case do not wet the player, and honey/shimmer
   detection inherits the same error. Not admitted in ROADMAP (M2-PHYS-3 claims the four
   flags are done — they are done in *kind*, not in *shape*).
2. **No `StickyTiles`** (behavioural, medium). The C# adhesion (tiles 51/229, the
   `rand.Next(30)` dust-30 trail) has no Rust counterpart; `Player.sticky` exists but is
   a different mechanism (`FloorVisuals`). Not admitted anywhere.
3. **No `HitTiles`** (behavioural, medium-high for combat feel). Weapons/projectiles
   never call `WorldGen.KillTile(..., effectOnly)`, so tile destruction on swing and the
   M4-PROJ-3b note "deferred HitTiles" remain open. Admitted only inside that row.
4. **No hurting-tile damage** (behavioural, medium). `AnyHurtingTiles`/`CanTileHurt`
   absent → spikes, geysers and lava-tile contact damage missing; the Obsidian Skull
   `fireWalk` note in M3-EQUIP-3 already mentions the hot-tile touch is unported.
5. **`tile_collision` is gravDir-1-only** (behavioural, medium). `gravDir = −1` bodies
   (reversed-gravity mounts/NPCs) cannot use the port; the `−0.01` landing branch
   (`Collision.cs:2394`) is unreachable. Same for the missing `ignoreDoors` /
   `ignoreAetheriumPlatforms` parameters.
6. **`scan_bottom` heuristic** (determinism, low). C# always clamps to `maxTilesY − 40`;
   the port only applies the margin when `world.height > 400` (`collision.rs:54-60`),
   otherwise it scans to the floor. Tiny worlds (the 64×64 test worlds) scan 40 rows the
   vanilla would have skipped.
7. **Hoik edge at `i == 0`** (cosmetic/behavioural edge, low). C# skips the left arm when
   `i < 1` (`:2401-2405`); the port reads `world.tile(i − 1, j)` which returns `None` →
   slope 0 → the collision still applies.
8. **`step_up` ignores `specialChecksMode`** (behavioural, low today). The
   `IgnoredByNpcStepUp` set is not plumbed; only the player calls `step_up`, so no live
   divergence, but the NPC reuse path inherits it.
9. **AI LOS (RESOLVED, S1-D-7)**. The old `npc_ai.rs::line_clear` half-tile sampler is
   deleted; the 18 `npc.rs` AI sites plus the Destroyer's segment laser
   (`world_sim/npcs.rs:921`) now call the faithful `projectile.rs::can_hit` /
   `::can_hit_line` / `player.rs::solid_collision` with the C# argument forms. The
   sample-vs-walk divergence (NPCs seeing/aggroing through geometry the vanilla blocks)
   is gone.
10. **`line_clear` used `TILE_SOLID` only (RESOLVED, S1-D-7)**. The replacements inherit
    `projectile.rs::full_solid`'s `!inActive`/`active`/`tileSolidTop`/slope/halfBrick
    conjunction, so actuated and platform tiles are treated as the C# does.

## Task seeds

- **S — Port `WetCollision`'s box scan** (fixes deviation 1): move the wet-flag
  computation out of `player.rs::update` into a `collision::wet_collision(world, pos, w, h)
  -> (wet, lava_wet, honey_wet, shimmer_wet)` porting `Collision.cs:1572-1651` verbatim
  (surface offset + slope support arm), then feed the four flags from it. Depends on
  nothing; re-verify M2-PHYS-3 afterwards.
- **M — Port `HitTiles`/`HitTilesInACircle` + tile-hit resolution** (deviation 3): a
  `collision::hit_tiles` calling the existing kill/drop path; wire the player swing and
  the projectile `tileCollide` sites. Depends on the edit/kill events (M7-FALL-1's
  `check_falling` shows the seam).
- **M — Port `StickyTiles`, `AnyHurtingTiles`/`CanTileHurt`, `SwitchTiles`**
  (deviations 2/4): three small self-contained scans; `CanTileHurt` needs the per-id
  damage table (`TileID.Sets` hurting ids) which lives in `tiledata`.
- **S — Switch the AI to the faithful LOS walks (DONE, S1-D-7)**: the `npc_ai::line_clear`
  call sites now use `projectile.rs::can_hit_line`/`can_hit` and `player.rs::solid_collision`
  (the `allow(dead_code)` is gone, `line_clear` deleted). The full workspace suite is
  green; the change is determinism-sensitive (aggro outcomes shift), so a before/after
  screenshot sweep is still owed before landing.
- **S — Generalise `tile_collision`/`slope_collision` signatures**: expose `gravDir`,
  `ignore_doors`, `ignore_aetherium_platforms`, `special_checks_mode` (deviations 5/8),
  with the current call sites passing today's defaults so behaviour is frozen.
- **M — Port `BuildTileContacts`, `noSlopeCollision`, `AdvancedTileCollision`,
  `DrownCollision`/`GetWaterLine`** (the remaining PARTIAL→PORTED rows): needed by conveyor
  belts/hoik/AI; larger because `AdvancedTileCollision`/`noSlopeCollision` have no Rust
  caller yet — port together with the task that first needs them.
- **M — LOS/HitTile unit tests**: build a small synthetic world and compare the port
  against the C# line ranges for `CanHitLine` (both axis orders), `HitTiles` (the circle
  distance test), and `StepUp`/`StepDown` thresholds (7/9/17 px, 16.1 px).
