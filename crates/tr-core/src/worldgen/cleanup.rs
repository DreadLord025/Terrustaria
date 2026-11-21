//! The classic cleanup passes — W4-d106 lane 6:
//!
//! - [`tile_cleanup`] — `Tile Cleanup` (`WorldGen.cs:21520-21942`): the
//!   whole-world slope clear, the 40-margin tidy sweep (the half-brick
//!   conversions, the water-death kills, the drip placement under shelves
//!   and over pools, the wall-13/14/87 liquid strips) and the multi-tile
//!   object frame normalisation (the dart trap, the fragile ice, the shadow
//!   orb, the heart, the mana crystal, the chests, the pots, the demon
//!   altar, the Lihzahrd altar over spikes).
//! - [`remove_broken_traps`] — `Remove Broken Traps`
//!   (`WorldGen.cs:22121-22140`): the 50-margin red-wire sweep that clears
//!   every broken trap circuit (the two-list BFS, the working-circuit veto,
//!   the wire/actuator strip).
//! - [`final_cleanup`] — `Final Cleanup` (`WorldGen.cs:22141-22490`): the
//!   wall-hole fill, the surface-desert gate, the whole-world sweep (the
//!   sand-family fall columns, the boulder normalisation, the dungeon-wall
//!   liquid strip, the minecart liquid strip, the painting wall
//!   inheritance), the poo-668 scatter, the seed `Finish*` dispatch, the
//!   shimmer clean-up and the final `Liquid.LiquidCheck` merge sweep.
//!
//! Every pass draws from its own `pass_rng("<vanilla name>")` stream (the
//! per-pass re-seed model), so the insertion shifts no other step's ledger.
//!
//! Deviations (the `finish_seeds.rs` discipline — documented stand-ins, not
//! inventions):
//! - **No chest inventories at generation.** The `BasicChest` style override
//!   reads `Chest.FindChest(...).item[0].type` (`:21790-21811`); the port
//!   generates chests as tiles without a chest registry, so `FindChest`
//!   answers −1 and the frame-derived style stands (the same gap
//!   `finish_seeds.rs` records for its `Main.chest` loops).
//! - `Main.tileSolid[379]` is `false` for the whole Tile Cleanup pass
//!   (`:21524`, restored `:21940`); the local [`solid_tile`] reads the
//!   bubble as non-solid accordingly (the stock table has it solid,
//!   `Main.cs:9468`).
//! - Final Cleanup (`:22143`) pokes `Main.tileSolid[484] = false` (the
//!   rolling cactus non-solid for the sweep) — a load-order detail the
//!   port's tile data does not model (the `desert.rs` precedent); the
//!   sweep's boulder arm reads the stock `Sets.Boulders` membership.
//! - The Final Cleanup `TileFrame(i, j)` calls (`:22240`, `:22376`) ride
//!   the post-pipeline framing seam (the port carries no gen-time
//!   `TileFrame`; the R3/R4 passes document the same deferral).
//! - The seed `Finish*` dispatch (`:22423-22471`) keeps the M5-SEED-1
//!   seam: drunk/getgood/anniversary already run from `step_seed_finish`
//!   (`finish_seeds.rs`); this pass ports `FinishNoTraps` and
//!   `FinishRemixWorld` (flag-gated), the `NotTheBees` gate rides
//!   `liquids::not_the_bees` (the conversion body is the notTheBees
//!   lane's), and the vampire / ShimmerSpawnHalfOfWorld / RainbowSand /
//!   `FinalizeSecretSeeds` / NPC-angler tails are flag-absent or
//!   no-port-state stand-ins (documented at the dispatch site).

use crate::edit::kill_tile;
use crate::liquid::LiquidSystem;
use crate::seeds::{MicroSeedFlags, SeedFlags};
use crate::tile::{Tile, FLAG_ACTIVE, FLAG_HALF_BRICK, LiquidKind};
use crate::tiledata as td;
use crate::world::World;
use super::liquids::SettleAnchors;
use super::noise::Rng;
use super::pipeline::{GenVars, Streams};
use super::GenOptions;

/// `WallID.Sets.WallTypeToTerrainTileType` (`WallID.cs:101`):
/// `CreateUshortSet(0, 40,147, 71,161, 15,59, 86,225, 3,25, 83,203,
/// 178,367, 180,368)` — the default 0 plus the seven wall→tile pairs.
fn wall_to_terrain_tile(wall: u16) -> u16 {
    match wall {
        40 => 147,
        71 => 161,
        15 => 59,
        86 => 225,
        3 => 25,
        83 => 203,
        178 => 367,
        180 => 368,
        _ => 0,
    }
}

/// `WorldGen.SolidTile` with the pass's `tileSolid[379] = false` window
/// (`WorldGen.cs:21524`): the bubble is non-solid; everything else is the
/// stock nactive-solid test. An out-of-bounds tile reads solid (the C#
/// null→true rule, the `biome_detail::solid_tile` precedent).
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && t.kind != 379
        && td::TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && (t.flags & crate::tile::FLAG_INACTIVE) == 0
}

/// `Skyblock.denySomeGeneration` (`WorldGen.cs:3133-3147`): skyblock with
/// NONE of the nine surface/world secret seeds enabled.
fn deny_some_generation(opts: &GenOptions) -> bool {
    if !opts.flags.contains(SeedFlags::SKYBLOCK) {
        return false;
    }
    let surface_seeds = MicroSeedFlags::WORLD_IS_FROZEN
        | MicroSeedFlags::SURFACE_IS_DESERT
        | MicroSeedFlags::SURFACE_IS_MUSHROOMS
        | MicroSeedFlags::WORLD_IS_INFECTED
        | MicroSeedFlags::HALLOW_ON_THE_SURFACE
        | MicroSeedFlags::NO_INFECTION
        | MicroSeedFlags::EXTRA_FLOATING_ISLANDS
        | MicroSeedFlags::EXTRA_LIQUID
        | MicroSeedFlags::EXTRA_LIVING_TREES;
    !opts.micro_flags.intersects(surface_seeds)
}

/// `Tile Cleanup` (`GenPassNameID.TileCleanup`, `WorldGen.cs:21520-21942`)
/// — W4-d106 R3. Draws, in C# order on the pass's own stream, only inside
/// the drip block (`:21551-21626`): one `Next(3)` per candidate cell, then
/// (when it passes) `Next(15, 21)`, one `Next(l - num2)` per ≥128-liquid
/// candidate above, `Next(3, 11)` + one `Next((m - l) * 3)` per ≥200-liquid
/// candidate below when the upward scan placed nothing, and a closing
/// `Next(4)` for the sand-drip arm.
pub(crate) fn tile_cleanup(world: &mut World, rng: &mut Rng, opts: &GenOptions) {
    // `:21522` — `!Skyblock.denySomeGeneration`.
    if deny_some_generation(opts) {
        return;
    }
    // `:21524` — `Main.tileSolid[379] = false` for the pass (the local
    // `solid_tile` reads it; restored conceptually at `:21940`).
    // `:21526-21536` — the whole-world slope clear: every cell that is not
    // an active SaveSlopes kind loses its slope AND half-brick. The set is
    // the runtime-filled `tile_runner::save_slopes` (the generated bitset
    // is empty — `TileID.cs:468-471` fills it in PostSetupContent).
    for t in world.tiles.iter_mut() {
        if !t.active() || !super::tile_runner::save_slopes(t.kind as i32) {
            t.set_slope(0);
            t.flags &= !FLAG_HALF_BRICK;
        }
    }
    let (width, height) = (world.width, world.height);
    let drunk = opts.flags.contains(SeedFlags::DRUNK);
    let extra_liquid = opts.micro_flags.contains(MicroSeedFlags::EXTRA_LIQUID);
    // `:21537-21939` — the 40-margin tidy sweep, k-outer / l-inner.
    for k in 40..width - 40 {
        for l in 40..height - 40 {
            // `:21542-21546` — a top slope beside a half-brick becomes a
            // half-brick itself.
            let convert = world.tile(k, l).is_some_and(|t| {
                t.active()
                    && t.top_slope()
                    && ((t.left_slope()
                        && world.tile(k + 1, l).is_some_and(|n| n.active() && n.half_brick()))
                        || (t.right_slope()
                            && world
                                .tile(k - 1, l)
                                .is_some_and(|n| n.active() && n.half_brick())))
            });
            if convert {
                if let Some(t) = world.tile_mut(k, l) {
                    t.set_slope(0);
                    t.flags |= FLAG_HALF_BRICK;
                }
            }
            // `:21547-21550` — a slowly-dies-in-water object in liquid dies.
            let dies = world.tile(k, l).is_some_and(|t| {
                t.active() && t.liquid > 0 && td::SET_SLOWLY_DIES_IN_WATER.has(t.kind)
            });
            if dies {
                kill_tile(world, k, l, true);
            }
            // `:21551-21627` — the drip block: an air cell under a solid
            // shelf, 2/3 of the time.
            let drip_gate = world.tile(k, l).is_some_and(|t| !t.active() && t.liquid == 0)
                && rng.range(0, 3) != 0
                && solid_tile(world, k, l - 1);
            if drip_gate {
                // `:21553-21584` — the upward scan: a ≥128 liquid cell above
                // (not shimmer) drips down at 1-in-(distance).
                let num = rng.range(15, 21);
                let mut num2 = l - 2;
                while num2 >= l - num {
                    let cand = world.tile(k, num2).copied().unwrap_or_default();
                    if cand.liquid >= 128 && cand.liquid_kind() != LiquidKind::Shimmer {
                        let mut num3: u16 = 373;
                        if cand.liquid_kind() == LiquidKind::Lava {
                            num3 = 374;
                        } else if cand.liquid_kind() == LiquidKind::Honey {
                            num3 = 375;
                        } else if cand.liquid_kind() == LiquidKind::Shimmer {
                            num3 = 709;
                        }
                        if rng.range(0, l - num2) <= 1 {
                            // `:21573-21576` — the hive wall forces honey.
                            if world.tile(k, l).is_some_and(|t| t.wall == 86) {
                                num3 = 375;
                            }
                            if let Some(t) = world.tile_mut(k, l) {
                                t.kind = num3;
                                t.frame_x = 0;
                                t.frame_y = 0;
                                t.flags |= FLAG_ACTIVE;
                            }
                            break;
                        }
                    }
                    num2 -= 1;
                }
                // `:21585-21615` — the downward scan when nothing placed.
                if world.tile(k, l).is_some_and(|t| !t.active()) {
                    let num = rng.range(3, 11);
                    for m in (l + 1)..=(l + num) {
                        let cand = world.tile(k, m).copied().unwrap_or_default();
                        if cand.liquid >= 200 && cand.liquid_kind() != LiquidKind::Shimmer {
                            let mut num4: u16 = 373;
                            if cand.liquid_kind() == LiquidKind::Lava {
                                num4 = 374;
                            } else if cand.liquid_kind() == LiquidKind::Honey {
                                num4 = 375;
                            } else if cand.liquid_kind() == LiquidKind::Shimmer {
                                num4 = 709;
                            }
                            if rng.range(0, (m - l) * 3) <= 1 {
                                if let Some(t) = world.tile_mut(k, l) {
                                    t.kind = num4;
                                    t.frame_x = 0;
                                    t.frame_y = 0;
                                    t.flags |= FLAG_ACTIVE;
                                }
                                break;
                            }
                        }
                    }
                }
                // `:21616-21626` — the sand-drip arm: 1/4, over sandstone /
                // hardened sand.
                if world.tile(k, l).is_some_and(|t| !t.active()) && rng.range(0, 4) == 0 {
                    let above_kind = world.tile(k, l - 1).map_or(0, |t| t.kind);
                    if td::SET_CONVERSION_SANDSTONE.has(above_kind)
                        || td::SET_CONVERSION_HARDENED_SAND.has(above_kind)
                    {
                        if let Some(t) = world.tile_mut(k, l) {
                            t.kind = 461;
                            t.frame_x = 0;
                            t.frame_y = 0;
                            t.flags |= FLAG_ACTIVE;
                        }
                    }
                }
            }
            // `:21628-21631` — the hellstone/obsidian brick walls hold no
            // liquid.
            if world.tile(k, l).is_some_and(|t| matches!(t.wall, 13 | 14)) {
                if let Some(t) = world.tile_mut(k, l) {
                    t.liquid = 0;
                }
            }
            // `:21632-21649` — the hive wall: extraLiquid re-types to water
            // and the non-water drips to water drips; otherwise no liquid.
            if world.tile(k, l).is_some_and(|t| t.wall == 87) {
                if extra_liquid {
                    let t = world.tile_mut(k, l).unwrap();
                    if t.liquid > 0 {
                        t.set_liquid_kind(LiquidKind::Water);
                    }
                    if matches!(t.kind, 374 | 709 | 375) {
                        t.kind = 373;
                    }
                } else if let Some(t) = world.tile_mut(k, l) {
                    t.liquid = 0;
                }
            }
            // `:21650-21937` — the active-tile object family.
            if world.tile(k, l).is_some_and(|t| t.active()) {
                object_cleanup(world, k, l, opts, drunk);
            }
        }
    }
    // `:21940` — `Main.tileSolid[379] = true` (the stock table already has
    // it; the pass-local read was the local `solid_tile`).
}

/// The active-tile object family of the Tile Cleanup sweep
/// (`WorldGen.cs:21650-21937`): the per-kind frame normalisation. Each arm
/// re-reads the cell (the C# re-reads `Main.tile[k, l]` per `if`).
fn object_cleanup(world: &mut World, k: i32, l: i32, opts: &GenOptions, drunk: bool) {
    let kind = world.tile(k, l).map_or(0, |t| t.kind);
    // `:21652-21667` — the dart trap: a half-brick/sloped seat cell dies.
    if kind == 137 {
        let (fx, fy) = world.tile(k, l).map_or((0, 0), |t| (t.frame_x, t.frame_y));
        let num5 = fy as i32 / 18;
        if num5 <= 2 || num5 == 5 {
            let num6 = if fx >= 18 { 1 } else { -1 };
            let seat_unstable = world
                .tile(k + num6, l)
                .is_some_and(|t| t.half_brick() || t.slope() != 0);
            if seat_unstable {
                if let Some(t) = world.tile_mut(k + num6, l) {
                    t.flags &= !FLAG_ACTIVE;
                }
            }
        }
    } else if kind == 162 {
        // `:21668-21676` — the unsupported fragile ice dies.
        let above = world.tile(k, l - 1).copied().unwrap_or_default();
        let below = world.tile(k, l + 1).copied().unwrap_or_default();
        // `CanKillTile` — the port has no chest registry, so the container
        // veto closure answers false (the `finish_seeds.rs` gap).
        if !above.active()
            && !below.active()
            && below.liquid == 0
            && crate::edit::can_kill_tile(world, k, l, |_, _| false)
        {
            if let Some(t) = world.tile_mut(k, l) {
                t.flags &= !FLAG_ACTIVE;
            }
        }
    }
    let kind = world.tile(k, l).map_or(0, |t| t.kind);
    // `:21677-21704` — the shadow orb: normalise the 2×2 frame, the evil
    // half picked by the drunk wall rule or the world's crimson.
    if kind == 31 {
        let (fx, fy, wall) = world
            .tile(k, l)
            .map_or((0, 0, 0), |t| (t.frame_x, t.frame_y, t.wall));
        let num7 = fx as i32 / 18;
        let num8 = if drunk {
            if wall == 83 { 1 } else { 0 }
        } else if opts.crimson {
            1
        } else {
            0
        };
        let num9 = k - num7 % 2;
        let num10 = fy as i32 / 18;
        // `:21685-21689` — num11 = num10 / 2 (the style row), num12 the
        // block's top row.
        let num11 = num10 / 2;
        let num12 = l - num10 % 2;
        for n in 0..2 {
            for num13 in 0..2 {
                if let Some(t) = world.tile_mut(num9 + n, num12 + num13) {
                    t.flags |= FLAG_ACTIVE;
                    t.set_slope(0);
                    t.flags &= !FLAG_HALF_BRICK;
                    t.kind = 31;
                    t.frame_x = (n * 18 + 36 * num8) as i16;
                    t.frame_y = (num13 * 18 + 36 * num11) as i16;
                }
            }
        }
    }
    // `:21705-21738` — the heart: normalise the 2×2 frame (style rows ride
    // frameX/36 and frameY/36) and seat it on a terrain floor.
    if kind == 12 {
        normalize_2x2_with_floor(world, k, l, 12);
    }
    let kind = world.tile(k, l).map_or(0, |t| t.kind);
    // `:21739-21772` — the mana crystal: the same shape as the heart.
    if kind == 639 {
        normalize_2x2_with_floor(world, k, l, 639);
    }
    // `:21773-21833` — the basic chests: normalise the 2×2 frame; the style
    // override reads the chest's first item (`Chest.FindChest`) — the port
    // has no chest registry at generation, so the frame-derived style
    // stands (documented stand-in).
    let kind = world.tile(k, l).map_or(0, |t| t.kind);
    if td::SET_BASIC_CHEST.has(kind) {
        let (fx, fy) = world.tile(k, l).map_or((0, 0), |t| (t.frame_x, t.frame_y));
        let mut num36 = fx as i32 / 18;
        let mut num37 = 0;
        let chest_kind: u16 = if kind == 467 { 467 } else { 21 };
        let mut num38 = k;
        let num39 = l - fy as i32 / 18;
        while num36 >= 2 {
            num37 += 1;
            num36 -= 2;
        }
        num38 -= num36;
        // `:21790-21811` — FindChest == −1 in the port (no chest registry):
        // the 1156/1571/1569/1260/1572 → style 23..27 overrides never fire.
        for num41 in 0..2 {
            for num42 in 0..2 {
                if let Some(t) = world.tile_mut(num38 + num41, num39 + num42) {
                    t.flags |= FLAG_ACTIVE;
                    t.set_slope(0);
                    t.flags &= !FLAG_HALF_BRICK;
                    t.kind = chest_kind;
                    t.frame_x = (num41 * 18 + 36 * num37) as i16;
                    t.frame_y = (num42 * 18) as i16;
                }
            }
            seat_floor(world, num38 + num41, num39 + 2, false);
        }
    }
    // `:21834-21875` — the pots: normalise the 2×2 frame with the style
    // rows, seat on a terrain floor.
    let kind = world.tile(k, l).map_or(0, |t| t.kind);
    if kind == 28 {
        let (fx, fy) = world.tile(k, l).map_or((0, 0), |t| (t.frame_x, t.frame_y));
        let mut num45 = fx as i32 / 18;
        let mut num46 = 0;
        let mut num47 = k;
        while num45 >= 2 {
            num46 += 1;
            num45 -= 2;
        }
        num47 -= num45;
        let mut num48 = fy as i32 / 18;
        let mut num49 = 0;
        let mut num50 = l;
        while num48 >= 2 {
            num49 += 1;
            num48 -= 2;
        }
        num50 -= num48;
        for num51 in 0..2 {
            for num52 in 0..2 {
                if let Some(t) = world.tile_mut(num47 + num51, num50 + num52) {
                    t.flags |= FLAG_ACTIVE;
                    t.set_slope(0);
                    t.flags &= !FLAG_HALF_BRICK;
                    t.kind = 28;
                    t.frame_x = (num51 * 18 + 36 * num46) as i16;
                    t.frame_y = (num52 * 18 + 36 * num49) as i16;
                }
            }
            seat_floor(world, num47 + num51, num50 + 2, false);
        }
    }
    // `:21876-21932` — the demon altar: normalise the 3×2 frame, seat on a
    // terrain floor (re-typing non-solid seats), clear the pot cell below
    // and the half-object cells beside it.
    let kind = world.tile(k, l).map_or(0, |t| t.kind);
    if kind == 26 {
        let (fx, fy, wall) = world
            .tile(k, l)
            .map_or((0, 0, 0), |t| (t.frame_x, t.frame_y, t.wall));
        let mut num55 = fx as i32 / 18;
        let mut num57 = k;
        let num58 = l - fy as i32 / 18;
        // `:21882-21887` — the while loop also counts style columns into
        // num56, but `:21888` overwrites num56 with the drunk-wall / crimson
        // rule, so the count is dead; only the column remainder matters.
        while num55 >= 3 {
            num55 -= 3;
        }
        num57 -= num55;
        let num56 = if drunk {
            if wall == 83 { 1 } else { 0 }
        } else if opts.crimson {
            1
        } else {
            0
        };
        for num59 in 0..3 {
            for num60 in 0..2 {
                if let Some(t) = world.tile_mut(num57 + num59, num58 + num60) {
                    t.flags |= FLAG_ACTIVE;
                    t.set_slope(0);
                    t.flags &= !FLAG_HALF_BRICK;
                    t.kind = 26;
                    t.frame_x = (num59 * 18 + 54 * num56) as i16;
                    t.frame_y = (num60 * 18) as i16;
                }
            }
            seat_floor(world, num57 + num59, num58 + 2, true);
            // `:21913-21917` — a pot's bottom half under the altar dies.
            let below_pot = world
                .tile(num57 + num59, num58 + 3)
                .is_some_and(|t| t.kind == 28 && t.frame_y % 36 >= 18);
            if below_pot {
                if let Some(t) = world.tile_mut(num57 + num59, num58 + 3) {
                    t.kind = 0;
                    t.flags &= !FLAG_ACTIVE;
                }
            }
        }
        // `:21919-21931` — the side columns: the left half of a 2×2 object
        // on the left, the right half on the right, die. The C#'s right
        // arm reads the LEFT cell for its 639 test (`:21926`'s
        // `Main.tile[num57 - 1, ...]`) — ported verbatim.
        for num63 in 0..3 {
            let left_half = world.tile(num57 - 1, num58 + num63).is_some_and(|t| {
                matches!(t.kind, 28 | 12 | 639) && t.frame_x % 36 < 18
            });
            if left_half {
                if let Some(t) = world.tile_mut(num57 - 1, num58 + num63) {
                    t.kind = 0;
                    t.flags &= !FLAG_ACTIVE;
                }
            }
            let right_cell = world.tile(num57 + 3, num58 + num63).copied().unwrap_or_default();
            let left_639 = world
                .tile(num57 - 1, num58 + num63)
                .is_some_and(|t| t.kind == 639);
            if (matches!(right_cell.kind, 28 | 12) || left_639) && right_cell.frame_x % 36 >= 18 {
                if let Some(t) = world.tile_mut(num57 + 3, num58 + num63) {
                    t.kind = 0;
                    t.flags &= !FLAG_ACTIVE;
                }
            }
        }
    }
    // `:21933-21936` — the Lihzahrd altar over wooden spikes: the spikes
    // become Lihzahrd brick.
    let kind = world.tile(k, l).map_or(0, |t| t.kind);
    if kind == 237 && world.tile(k, l + 1).is_some_and(|t| t.kind == 232) {
        if let Some(t) = world.tile_mut(k, l + 1) {
            t.kind = 226;
        }
    }
}

/// The heart / mana-crystal normalisation (`WorldGen.cs:21705-21772`): the
/// 2×2 frame with the style rows in frameX/36 (`num17`) and frameY/36
/// (`num20`), plus the terrain floor row.
fn normalize_2x2_with_floor(world: &mut World, k: i32, l: i32, kind: u16) {
    let (fx, fy) = world.tile(k, l).map_or((0, 0), |t| (t.frame_x, t.frame_y));
    let num16 = fx as i32 / 18;
    let num17 = num16 / 2;
    let num18 = k - num16 % 2;
    let num19 = fy as i32 / 18;
    let num20 = num19 / 2;
    let num21 = l - num19 % 2;
    for num22 in 0..2 {
        for num23 in 0..2 {
            if let Some(t) = world.tile_mut(num18 + num22, num21 + num23) {
                t.flags |= FLAG_ACTIVE;
                t.set_slope(0);
                t.flags &= !FLAG_HALF_BRICK;
                t.kind = kind;
                t.frame_x = (num22 * 18 + 36 * num17) as i16;
                t.frame_y = (num23 * 18 + 36 * num20) as i16;
            }
        }
        seat_floor(world, num18 + num22, num21 + 2, false);
    }
}

/// The object seat floor (`WorldGen.cs:21730-21736` and the altar variant
/// `:21902-21912`): an empty seat activates as the wall's terrain tile; the
/// altar variant also re-types a non-solid (and non-platform) seat. Either
/// way the seat loses its slope and half-brick.
fn seat_floor(world: &mut World, x: i32, y: i32, altar: bool) {
    let Some(t) = world.tile_mut(x, y) else {
        return;
    };
    if !t.active() {
        t.flags |= FLAG_ACTIVE;
        t.kind = wall_to_terrain_tile(t.wall);
    }
    if altar {
        // `:21907-21910` — `(!tileSolid[type] || tileSolidTop[type]) &&
        // !Platforms[type]` re-types the seat to the wall's terrain tile.
        let solid_seat = td::TILE_SOLID.has(t.kind) && !td::TILE_SOLID_TOP.has(t.kind);
        if !solid_seat && !td::SET_PLATFORMS.has(t.kind) {
            t.kind = wall_to_terrain_tile(t.wall);
        }
    }
    t.set_slope(0);
    t.flags &= !FLAG_HALF_BRICK;
}

// --- W4-d106 R5: Remove Broken Traps ------------------------------------------

/// `WorldGen.IsItATrap` (`WorldGen.cs:26788-26803`): an active actuator, or
/// a mechanism that is not on the ignore-when-validating list.
fn is_it_a_trap(t: &Tile) -> bool {
    if !t.active() {
        return false;
    }
    if t.actuator() {
        return true;
    }
    if td::SET_IS_A_MECHANISM.has(t.kind) {
        return !td::SET_IGNORE_WHEN_VALIDATING_TRAPS.has(t.kind);
    }
    false
}

/// `WorldGen.IsItATrigger` (`WorldGen.cs:26805-26823`): an active trigger
/// kind, the golden-chest style-4 pressure plate (467 frameX/36 == 4), or a
/// minecart-track pressure plate (314 frameX 20/21, `Minecart.IsPressurePlate`
/// `Minecart.cs:549-560`).
fn is_it_a_trigger(t: &Tile) -> bool {
    if !t.active() {
        return false;
    }
    if td::SET_IS_A_TRIGGER.has(t.kind) {
        return true;
    }
    if t.kind == 467 && t.frame_x as i32 / 36 == 4 {
        return true;
    }
    if t.kind == 314 && matches!(t.frame_x, 20 | 21) {
        return true;
    }
    false
}

/// `Tile.ResetToType` (`Tile.cs:249-259`): liquid 0, `sTileHeader = 32`
/// (active, no half-brick/actuator/inactive), the other headers 0, frames 0,
/// the new kind. The wall is kept (the C# does not touch it).
fn reset_to_type(t: &mut Tile, kind: u16) {
    t.liquid = 0;
    t.flags = FLAG_ACTIVE;
    t.meta = 0;
    t.frame_x = 0;
    t.frame_y = 0;
    t.kind = kind;
}

/// `WorldGen.ClearBrokenTraps` (`WorldGen.cs:26840-26958`): flood-fill the
/// red-wire network from `start`, bounded by the two 2000 counters; if the
/// network carries BOTH a trap and a trigger it is a working circuit and is
/// left alone, otherwise every visited cell loses its wire (and actuator —
/// with the boulder-above reset) and its trap/trigger tile dies (the 314
/// track keeps its frame, set to 1). `visited` is the pass-wide
/// already-went-over set (the C# `pointsWeAlreadyWentOver`).
fn clear_broken_traps(world: &mut World, start: (i32, i32), visited: &mut Vec<(i32, i32)>) {
    // `:26848`/`:26857` — both C# counters are 2000.
    clear_broken_traps_capped(world, start, visited, 2000, 2000);
}

/// [`clear_broken_traps`] with the two BFS caps exposed — a test-only seam
/// (the vanilla call always passes 2000/2000): the >cap leftover-carry edge
/// (e20 research-148 §3b) is unreachable on a runnable fixture at the real
/// 2000-pop cap, so the witness drives small caps through this body.
fn clear_broken_traps_capped(
    world: &mut World,
    start: (i32, i32),
    visited: &mut Vec<(i32, i32)>,
    outer_waves: i32,
    inner_pops: i32,
) {
    // The C#'s two-list swap BFS (`t`/`t2`), each bounded by 2000.
    let mut t: Vec<(i32, i32)> = Vec::new();
    let mut t2: Vec<(i32, i32)> = vec![start];
    let mut list: Vec<(i32, i32)> = Vec::new();
    let mut has_trap = false;
    let mut has_trigger = false;
    let mut num = outer_waves;
    while !t2.is_empty() {
        num -= 1;
        if num <= 0 {
            break;
        }
        // `:26856` — `Utils.Swap(ref t, ref t2)` with NO clear (e20
        // research-148 §3b, VERIFIED against the C#): when the inner loop
        // breaks on the 2000-pop cap with `t` still non-empty, the leftover
        // cells survive in the swapped-in `t2` and are processed BEFORE the
        // next wave's appended neighbours. The previous `t2.clear()` here
        // dropped them (the >2000-inner-pop edge); removed for the 1:1 swap.
        // The working-circuit break path also carries leftovers into `t2`,
        // but the outer `flag2 & flag` break exits before another swap —
        // identical to the C#.
        std::mem::swap(&mut t, &mut t2);
        let mut num2 = inner_pops;
        while !t.is_empty() {
            num2 -= 1;
            if num2 <= 0 {
                break;
            }
            let item = t.remove(0);
            // `:26867` — InWorld(item, 5).
            if item.0 < 5
                || item.1 < 5
                || item.0 >= world.width - 5
                || item.1 >= world.height - 5
            {
                continue;
            }
            if !world.wires.wire(item.0, item.1) {
                continue;
            }
            // `:26874-26875` — record the cell in both lists.
            visited.push(item);
            list.push(item);
            let tile = world.tile(item.0, item.1).copied().unwrap_or_default();
            if is_it_a_trap(&tile) {
                has_trap = true;
            }
            if is_it_a_trigger(&tile) {
                has_trigger = true;
            }
            if has_trigger && has_trap {
                break;
            }
            // `:26888-26907` — queue the four neighbours not already in list.
            for (nx, ny) in [
                (item.0 - 1, item.1),
                (item.0 + 1, item.1),
                (item.0, item.1 - 1),
                (item.0, item.1 + 1),
            ] {
                if !list.contains(&(nx, ny)) {
                    t2.push((nx, ny));
                }
            }
        }
        if has_trigger && has_trap {
            break;
        }
    }
    // `:26915-26918` — a working circuit is left alone.
    if has_trigger && has_trap {
        return;
    }
    // `:26919-26957` — strip the broken network.
    for &(x, y) in &list {
        world.wires.set(x, y, crate::wiring::WireColor::Red, false);
        let tile = world.tile(x, y).copied().unwrap_or_default();
        if tile.actuator() {
            if let Some(t) = world.tile_mut(x, y) {
                t.flags &= !crate::tile::FLAG_ACTUATOR;
            }
            // `:26926-26940` — a boulder above with frameX % 16 == 0 resets
            // the 2×2 boulder block above to stone.
            let above = world.tile(x, y - 1).copied().unwrap_or_default();
            if td::SET_BOULDERS.has(above.kind) && above.frame_x % 16 == 0 {
                for i in 0..2 {
                    for j in -2..0 {
                        let boulder = world
                            .tile(x + i, y + j)
                            .is_some_and(|t| t.active() && td::SET_BOULDERS.has(t.kind));
                        if boulder {
                            if let Some(t) = world.tile_mut(x + i, y + j) {
                                reset_to_type(t, 1);
                            }
                        }
                    }
                }
            }
        }
        // `:26942-26956` — the trap tile dies (except the statue 105); the
        // trigger tile dies too, except the minecart track 314 (frameX = 1).
        // The C# calls the 2-arg `KillTile(x, y)` (`noItem = false` default,
        // `:63517`), but `KillTile` itself forces `noItem = true` whenever
        // `WorldGen.isGeneratingOrLoadingWorld` (`:63546-63549`, set at
        // `:6612` for the whole generation) — at gen time the `true` here IS
        // the vanilla behaviour, no items spawn (e20 research-148 §3a,
        // VERIFIED; receipt in research-312 §3).
        let tile = world.tile(x, y).copied().unwrap_or_default();
        if is_it_a_trap(&tile) && tile.kind != 105 {
            kill_tile(world, x, y, true);
        } else if is_it_a_trigger(&tile) {
            if tile.kind == 314 {
                if let Some(t) = world.tile_mut(x, y) {
                    t.frame_x = 1;
                }
            } else {
                kill_tile(world, x, y, true);
            }
        }
    }
}

/// `Remove Broken Traps` (`GenPassNameID.BrokenTrapCleanup`,
/// `WorldGen.cs:22121-22140`) — W4-d106 R5. The 50-margin sweep finds every
/// red-wired cell not already visited and clears its broken network. The
/// gate `!noTrapsWorldGen || tenthAnniversaryWorldGen || notTheBees` runs the
/// sweep on a classic world (noTraps off). Draws no RNG.
pub(crate) fn remove_broken_traps(world: &mut World, opts: &GenOptions) {
    // `:22123` — the gate.
    let no_traps = opts.flags.contains(SeedFlags::NO_TRAPS);
    let anniversary = opts.flags.contains(SeedFlags::TENTH_ANNIVERSARY);
    let not_the_bees = opts.flags.contains(SeedFlags::NOT_THE_BEES);
    if no_traps && !anniversary && !not_the_bees {
        return;
    }
    let mut visited: Vec<(i32, i32)> = Vec::new();
    let num = 50;
    for i in num..world.width - num {
        for j in 50..world.height - 50 {
            if world.wires.wire(i, j) && !visited.contains(&(i, j)) {
                clear_broken_traps(world, (i, j), &mut visited);
            }
        }
    }
}

// --- W4-d106 R6: Final Cleanup ----------------------------------------------

/// `WorldGen.beachDistance` (`WorldGen.cs:4449`) — the ±380 ocean margins the
/// surface partial-liquid clear and the remix conversion band read.
const BEACH_DISTANCE: i32 = 380;

/// `Main.wallDungeon` (`Main.cs:10598-10606`) — the nine dungeon walls.
const WALL_DUNGEON: [u16; 9] = [7, 8, 9, 94, 95, 96, 97, 98, 99];

/// `Main.tileDungeon` (`Main.cs:7992-7997`) — the six dungeon bricks.
const TILE_DUNGEON: [u16; 6] = [41, 43, 44, 677, 678, 679];

/// `TileID.DirtiestBlock` (`TileID.cs:1965`) — the poo scatter's target.
const TILE_DIRTIEST: u16 = 668;

/// `LiquidKind` → the C# `liquidType()` code (0 water / 1 lava / 2 honey /
/// 3 shimmer) — the final `LiquidCheck` sweep's type argument.
fn liquid_code(k: LiquidKind) -> u8 {
    match k {
        LiquidKind::Water => 0,
        LiquidKind::Lava => 1,
        LiquidKind::Honey => 2,
        LiquidKind::Shimmer => 3,
    }
}

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9695`).
fn in_world_f(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// A tile's kind, or 0 outside the map (the `micro_surface::kind_at`
/// convention — the C# sweeps read `Main.tile[x, y].type` as 0 there).
fn kind_at(world: &World, x: i32, y: i32) -> u16 {
    world.tile(x, y).map_or(0, |t| t.kind)
}

/// A tile's wall, or 0 outside the map.
fn wall_at(world: &World, x: i32, y: i32) -> u16 {
    world.tile(x, y).map_or(0, |t| t.wall)
}

/// A tile's liquid amount, or 0 outside the map.
fn liquid_at(world: &World, x: i32, y: i32) -> u8 {
    world.tile(x, y).map_or(0, |t| t.liquid)
}

/// `Main.tileSolid[type] && !Main.tileSolidTop[type]` — the raw table test
/// (no active/slope gate), the fall column's support checks read it.
fn solid_not_top(kind: u16) -> bool {
    td::TILE_SOLID.has(kind) && !td::TILE_SOLID_TOP.has(kind)
}

/// `Tile.ClearTile` (`Tile.cs:123-128`): ClearSlope + active(false) +
/// inActive(false) — the kind, wall, frames and liquid survive.
fn clear_tile(t: &mut Tile) {
    t.set_slope(0);
    t.flags &= !(FLAG_ACTIVE | crate::tile::FLAG_INACTIVE | FLAG_HALF_BRICK);
}

/// `Tile.ClearEverything` (`Tile.cs:110-121`): every header + the wall zero.
fn clear_everything(t: &mut Tile) {
    t.kind = 0;
    t.wall = 0;
    t.liquid = 0;
    t.flags = 0;
    t.meta = 0;
    t.frame_x = 0;
    t.frame_y = 0;
}

/// `WorldGen.FillWallHolesInArea` (`WorldGen.cs:9886-9898`): the column scan
/// over `[max(left,0), min(right,width)]` (the inclusive bound is guarded by
/// the `i < width` test), each column's wall-hole fill summed (the total is
/// unused by the caller).
fn fill_wall_holes_in_area(world: &mut World, left: i32, top: i32, right: i32, bottom: i32) {
    let num = left.max(0);
    let num2 = right.min(world.width);
    for i in num..=num2 {
        if i >= 0 && i < world.width {
            fill_wall_holes_in_column(world, i, top, bottom);
        }
    }
}

/// `WorldGen.FillWallHolesInColumn` (`WorldGen.cs:9900-9926`): walk the column
/// top→bottom; a wall-0 cell that follows a walled cell starts a hole fill.
fn fill_wall_holes_in_column(world: &mut World, x: i32, start_y: i32, end_y: i32) -> i32 {
    let mut num = 0;
    let x = x.clamp(2, world.width - 1 - 2);
    let start_y = start_y.max(2);
    let end_y = end_y.min(world.height - 2);
    let mut flag = false;
    for i in start_y..end_y {
        if wall_at(world, x, i) == 0 {
            if flag {
                flag = false;
                if fill_wall_holes_in_spot(world, x, i, 150) {
                    num += 1;
                }
            }
        } else {
            flag = true;
        }
    }
    num
}

/// `WorldGen.FillWallHolesInSpot` (`WorldGen.cs:9928-10053`): the 150-cap BFS
/// over the wall-0 region reachable through open (inactive-neighbour) cells,
/// then the majority-wall fill (default wall 2 when no wall borders the hole).
/// The wall-count ledger keeps insertion order so the strict-`>` majority pick
/// resolves ties to the first-seen wall (the C# `Dictionary` foreach order);
/// the final fill writes one wall to every hole cell, so its iteration order
/// does not affect the output.
fn fill_wall_holes_in_spot(world: &mut World, origin_x: i32, origin_y: i32, max_walls_threshold: usize) -> bool {
    if !in_world_f(world, origin_x, origin_y, 2) {
        return false;
    }
    let mut frontier: Vec<(i32, i32)> = vec![(origin_x, origin_y)]; // list2
    let mut visited: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new(); // hashSet
    let mut wall_counts: Vec<(u16, i32)> = Vec::new(); // dictionary (insertion order)
    while !frontier.is_empty() {
        let mut current: Vec<(i32, i32)> = frontier.clone(); // list = list2
        frontier.clear(); // list2.Clear()
        while !current.is_empty() {
            if visited.len() >= max_walls_threshold {
                return false;
            }
            let item = current[0];
            if visited.contains(&item) {
                current.remove(0);
                continue;
            }
            if !in_world_f(world, item.0, item.1, 1) {
                current.remove(0);
                continue;
            }
            visited.insert(item);
            current.remove(0);
            let wall = wall_at(world, item.0, item.1);
            if wall != 0 {
                if let Some(e) = wall_counts.iter_mut().find(|(w, _)| *w == wall) {
                    e.1 += 1;
                } else {
                    wall_counts.push((wall, 1));
                }
                continue;
            }
            // `:9976-10000` — the open-cell test: an inactive neighbour in the
            // horizontal or vertical cross lets the hole spread.
            let mut flag = false;
            for nx in (item.0 - 1)..=(item.0 + 1) {
                if !world.tile(nx, item.1).is_some_and(|t| t.active()) {
                    flag = true;
                    break;
                }
            }
            if !flag {
                for ny in (item.1 - 1)..=(item.1 + 1) {
                    if !world.tile(item.0, ny).is_some_and(|t| t.active()) {
                        flag = true;
                        break;
                    }
                }
            }
            if flag {
                // `:10003-10022` — left, right, up, down (the C# order).
                for nb in [
                    (item.0 - 1, item.1),
                    (item.0 + 1, item.1),
                    (item.0, item.1 - 1),
                    (item.0, item.1 + 1),
                ] {
                    if !visited.contains(&nb) {
                        frontier.push(nb);
                    }
                }
            }
        }
    }
    if visited.len() == 1 {
        return false;
    }
    // `:10030-10043` — the majority wall (strict `>` → first-seen wins ties).
    let mut wall: u16 = 0;
    let mut num = -1i32;
    for &(w, c) in &wall_counts {
        if c > num {
            wall = w;
            num = c;
        }
    }
    if num == -1 {
        wall = 2;
    }
    // `:10044-10051` — fill every hole cell that still has no wall.
    for &(x, y) in &visited {
        if let Some(t) = world.tile_mut(x, y) {
            if t.wall == 0 {
                t.wall = wall;
            }
        }
    }
    true
}

/// The sand-family hardened conversion (`WorldGen.cs:22184-22204` /
/// `:22221-22236`): sand 53 → 397, ebonsand 112 → 398, crimsand 234 → 399,
/// slush 224 → snow 147, silt 123 → stone 1.
fn harden_sand(kind: u16) -> u16 {
    match kind {
        53 => 397,
        112 => 398,
        234 => 399,
        224 => 147,
        123 => 1,
        _ => kind,
    }
}

/// The Final Cleanup whole-world sweep (`WorldGen.cs:22150-22378`), i-outer /
/// j-inner over the full map. Draws no RNG (every arm is a tile-state test).
/// `ws` is `Main.worldSurface` (the real layer anchor — the port's
/// `world.world_surface` is only written at the world tail, so the pass reads
/// `gv.layers()`; see [`final_cleanup`]). The per-cell `TileFrame(i, j)`
/// (`:22376`) and the palm re-frame (`:22240`) ride the post-pipeline framing
/// seam (the port carries no gen-time `TileFrame`).
fn final_cleanup_sweep(world: &mut World, ws: f64, rock_layer: f64, opts: &GenOptions) {
    let (width, height) = (world.width, world.height);
    let remix = opts.flags.contains(SeedFlags::REMIX);
    let dual_dungeons = opts.micro_flags.contains(MicroSeedFlags::DUAL_DUNGEONS);
    let extra_liquid = opts.micro_flags.contains(MicroSeedFlags::EXTRA_LIQUID);
    let ocean_level = (ws + rock_layer) / 2.0 + 40.0;
    for i in 0..width {
        for j in 0..height {
            // `:22155-22242` — the sand-family fall column.
            let sand_kind = kind_at(world, i, j);
            let is_sand = matches!(sand_kind, 53 | 112 | 234 | 224 | 123);
            if world.tile(i, j).is_some_and(|t| t.active())
                && !super::biome_detail::solid_tile(world, i, j + 1)
                && is_sand
            {
                let below_active = world.tile(i, j + 1).is_some_and(|t| t.active());
                let below_cut = crate::tileobject::tile_cut(kind_at(world, i, j + 1));
                // `:22157` — the surface fall gate.
                if (j as f64) < ws + 10.0
                    && (!below_active || below_cut)
                    && wall_at(world, i, j + 1) != 191
                    && !super::spider_gem_moss::ocean_depths(world, i, j, ocean_level)
                {
                    // `:22159-22168` — the 314-track veto within 10 below.
                    let mut num = 10;
                    let mut num2 = j + 1;
                    for k in num2..(num2 + 10) {
                        if world.tile(i, k).is_some_and(|t| t.active() && t.kind == 314) {
                            num = 0;
                            break;
                        }
                    }
                    // `:22169-22179` — drop the sand down the open column.
                    while (!world.tile(i, num2).is_some_and(|t| t.active())
                        || crate::tileobject::tile_cut(kind_at(world, i, num2)))
                        && num > 0
                        && num2 < height - 50
                    {
                        if let Some(t) = world.tile_mut(i, num2 - 1) {
                            t.set_slope(0);
                            t.flags &= !FLAG_HALF_BRICK;
                        }
                        if let Some(t) = world.tile_mut(i, num2) {
                            t.flags |= FLAG_ACTIVE;
                            t.kind = sand_kind;
                            t.set_slope(0);
                            t.flags &= !FLAG_HALF_BRICK;
                        }
                        num2 += 1;
                        num -= 1;
                    }
                    // `:22180-22205` — a full 10-drop onto air hardens the base.
                    if num == 0 && !world.tile(i, num2).is_some_and(|t| t.active()) {
                        if let Some(t) = world.tile_mut(i, num2) {
                            t.kind = harden_sand(sand_kind);
                            t.flags |= FLAG_ACTIVE;
                        }
                    } else if world
                        .tile(i, num2)
                        .is_some_and(|t| t.active() && solid_not_top(t.kind))
                    {
                        // `:22206-22210` — flatten the solid support.
                        if let Some(t) = world.tile_mut(i, num2) {
                            t.set_slope(0);
                            t.flags &= !FLAG_HALF_BRICK;
                        }
                    }
                } else if world
                    .tile(i, j + 1)
                    .is_some_and(|t| solid_not_top(t.kind) && (t.top_slope() || t.half_brick()))
                {
                    // `:22212-22216` — flatten a sloped/half support below.
                    if let Some(t) = world.tile_mut(i, j + 1) {
                        t.set_slope(0);
                        t.flags &= !FLAG_HALF_BRICK;
                    }
                } else {
                    // `:22217-22237` — no fall: harden the sand in place.
                    if let Some(t) = world.tile_mut(i, j) {
                        t.kind = harden_sand(sand_kind);
                    }
                }
                // `:22238-22241` — the palm above re-frames (framing seam).
            }
            // `:22243-22247` — sandstone/hardened-sand walls hold lava.
            let wall = wall_at(world, i, j);
            if (wall == 187 || wall == 216) && liquid_at(world, i, j) > 0 && !remix && !dual_dungeons {
                if let Some(t) = world.tile_mut(i, j) {
                    t.liquid = u8::MAX;
                    t.set_liquid_kind(LiquidKind::Lava);
                }
            }
            // `:22248-22252` — the dart trap 137 loses its slope/half-brick.
            if kind_at(world, i, j) == 137 {
                if let Some(t) = world.tile_mut(i, j) {
                    t.set_slope(0);
                    t.flags &= !FLAG_HALF_BRICK;
                }
            }
            // `:22253-22300` — the boulder 2x2 normalisation + altar-26 veto.
            if world.tile(i, j).is_some_and(|t| t.active() && td::SET_BOULDERS.has(t.kind)) {
                let (bkind, fx, fy) = {
                    let t = world.tile(i, j).unwrap();
                    (t.kind, t.frame_x, t.frame_y)
                };
                let num3 = i - fx as i32 / 18;
                let num4 = j - fy as i32 / 18;
                let mut flag = false;
                for l in 0..2 {
                    if world.tile(num3 + l, num4 - 1).is_some_and(|t| t.active() && t.kind == 26) {
                        flag = true;
                        break;
                    }
                    for m in 0..2 {
                        if let Some(t) = world.tile_mut(num3 + l, num4 + m) {
                            t.flags |= FLAG_ACTIVE;
                            t.set_slope(0);
                            t.flags &= !FLAG_HALF_BRICK;
                            t.kind = bkind;
                            t.frame_x = (l * 18) as i16;
                            t.frame_y = (m * 18) as i16;
                        }
                    }
                }
                if flag {
                    // `:22278-22299` — an altar above: the rolling cactus 484
                    // becomes hardened sand 397, every other boulder dirt 0.
                    let ty: u16 = if bkind == 484 { 397 } else { 0 };
                    for n in 0..2 {
                        for num7 in 0..2 {
                            if let Some(t) = world.tile_mut(num3 + n, num4 + num7) {
                                t.flags |= FLAG_ACTIVE;
                                t.set_slope(0);
                                t.flags &= !FLAG_HALF_BRICK;
                                t.kind = ty;
                                t.frame_x = 0;
                                t.frame_y = 0;
                            }
                        }
                    }
                }
            }
            // `:22301-22304` — a palm 323 in liquid dies.
            if kind_at(world, i, j) == 323 && liquid_at(world, i, j) > 0 {
                kill_tile(world, i, j, true);
            }
            // `:22305-22318` — the dungeon-wall liquid strip.
            if WALL_DUNGEON.contains(&wall) {
                if let Some(t) = world.tile_mut(i, j) {
                    t.set_liquid_kind(LiquidKind::Water);
                }
                if kind_at(world, i, j) == 374 {
                    if let Some(t) = world.tile_mut(i, j) {
                        t.kind = 373;
                    }
                }
                if world.tile(i, j).is_some_and(|t| t.active() && t.kind == 56) {
                    kill_tile(world, i, j, true);
                    if let Some(t) = world.tile_mut(i, j) {
                        t.set_liquid_kind(LiquidKind::Water);
                        t.liquid = u8::MAX;
                    }
                }
            }
            // `:22319-22333` — the minecart-track 314 liquid strip (15 up, 1 down).
            if world.tile(i, j).is_some_and(|t| t.active() && t.kind == 314) && !extra_liquid {
                let num10 = 15;
                let num11 = 1;
                let mut num12 = j;
                while j - num12 < num10 {
                    if let Some(t) = world.tile_mut(i, num12) {
                        t.liquid = 0;
                    }
                    num12 -= 1;
                }
                let mut num12 = j;
                while num12 - j < num11 {
                    if let Some(t) = world.tile_mut(i, num12) {
                        t.liquid = 0;
                    }
                    num12 += 1;
                }
            }
            // `:22334-22339` — the gold-coin pile 332 extends onto air below.
            if world.tile(i, j).is_some_and(|t| t.active() && t.kind == 332)
                && !world.tile(i, j + 1).is_some_and(|t| t.active())
            {
                if let Some(t) = world.tile_mut(i, j + 1) {
                    clear_everything(t);
                    t.flags |= FLAG_ACTIVE;
                    t.kind = 332;
                }
            }
            // `:22340-22343` — the surface partial-liquid clear (Clouds veto).
            if i > BEACH_DISTANCE
                && i < width - BEACH_DISTANCE
                && (j as f64) < ws
                && liquid_at(world, i, j) > 0
                && liquid_at(world, i, j) < u8::MAX
                && liquid_at(world, i - 1, j) < u8::MAX
                && liquid_at(world, i + 1, j) < u8::MAX
                && liquid_at(world, i, j + 1) < u8::MAX
                && !world.tile(i - 1, j).is_some_and(|t| t.active() && td::SET_CLOUDS.has(t.kind))
                && !world.tile(i + 1, j).is_some_and(|t| t.active() && td::SET_CLOUDS.has(t.kind))
                && !world.tile(i, j + 1).is_some_and(|t| t.active() && td::SET_CLOUDS.has(t.kind))
            {
                if let Some(t) = world.tile_mut(i, j) {
                    t.liquid = 0;
                }
            }
            // `:22344-22375` — the painting wall inheritance.
            if world.tile(i, j).is_some_and(|t| t.active() && td::SET_PAINTINGS.has(t.kind)) {
                let no_wall = world.tile(i, j).is_some_and(|t| t.wall == 0);
                if no_wall && in_world_f(world, i, j, 2) {
                    let mut num13: u16 = 0;
                    for (nx, ny) in [(i - 1, j), (i + 1, j), (i, j - 1), (i, j + 1)] {
                        if num13 == 0 {
                            let w = wall_at(world, nx, ny);
                            if w > 0 {
                                num13 = w;
                            }
                        }
                    }
                    if num13 != 0 {
                        if let Some(t) = world.tile_mut(i, j) {
                            t.wall = num13;
                        }
                    }
                }
            }
            // `:22376` — TileFrame(i, j) rides the post-pipeline framing seam.
        }
    }
}

/// `WorldGen.GetWorldSize` (`WorldGen.cs:6559-6570`): ≤4200 → 0, ≤6400 → 1,
/// else 2.
fn get_world_size(width: i32) -> i32 {
    if width <= 4200 {
        0
    } else if width <= 6400 {
        1
    } else {
        2
    }
}

/// The poo-668 scatter (`WorldGen.cs:22379-22422`): count 3/6/9 by world size
/// (anniversary ×5), the 3000-attempt valve, `Next(50, w-50)` ×
/// `Next(50, h-200)` picks, a Dirt-set tile (Mud too under notTheBees) becomes
/// the Dirtiest Block 668 via ClearTile + active.
fn poo_scatter(world: &mut World, rng: &mut Rng, opts: &GenOptions) {
    let (width, height) = (world.width, world.height);
    let anniversary = opts.flags.contains(SeedFlags::TENTH_ANNIVERSARY);
    let not_the_bees = opts.flags.contains(SeedFlags::NOT_THE_BEES);
    let mut num14 = 0;
    let mut num15 = match get_world_size(width) {
        1 => 6,
        2 => 9,
        _ => 3,
    };
    if anniversary {
        num15 *= 5;
    }
    let num16 = 50;
    let (min_value, min_value2) = (num16, num16);
    let (max_value, max_value2) = (width - num16, height - 200);
    let mut num17 = 3000;
    while num14 < num15 {
        num17 -= 1;
        if num17 <= 0 {
            break;
        }
        // The C# throws on a collapsed range (a sub-vanilla test world); the
        // port's width guard skips the scatter before any draw (the R1/R2
        // sub-vanilla convention).
        if max_value <= min_value || max_value2 <= min_value2 {
            break;
        }
        let num18 = rng.range(min_value, max_value);
        let num19 = rng.range(min_value2, max_value2);
        let dirt = world.tile(num18, num19).is_some_and(|t| {
            t.active()
                && (td::SET_DIRT.has(t.kind) || (not_the_bees && td::SET_MUD.has(t.kind)))
        });
        if dirt {
            num14 += 1;
            if let Some(t) = world.tile_mut(num18, num19) {
                clear_tile(t);
                t.flags |= FLAG_ACTIVE;
                t.kind = TILE_DIRTIEST;
            }
        }
    }
}

// --- the seed Finish* arms (W4-d106 R6, flag-gated) -------------------------

/// `WorldGen.SolidTile2` (`WorldGen.cs:70492-70509`): active, `tileSolid`,
/// slope 0, no half-brick, not inactive; out-of-bounds reads as solid (the
/// `mini_biomes::solid_tile2` precedent — the platform half-brick/top-slope
/// alternative is moot under the half-brick veto).
fn solid_tile2(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && td::TILE_SOLID.has(t.kind)
        && t.slope() == 0
        && !t.half_brick()
        && (t.flags & crate::tile::FLAG_INACTIVE) == 0
}

/// The L-shaped red-wire run (`WorldGen.cs:26039-26065` and its three twins):
/// wire the origin, walk x to the target column, then y to the target row.
fn wire_run(world: &mut World, mut x: i32, mut y: i32, tx: i32, ty: i32) {
    world.wires.set(x, y, crate::wiring::WireColor::Red, true);
    while x != tx {
        if x < tx {
            x += 1;
        }
        if x > tx {
            x -= 1;
        }
        world.wires.set(x, y, crate::wiring::WireColor::Red, true);
    }
    while y != ty {
        if y < ty {
            y += 1;
        }
        if y > ty {
            y -= 1;
        }
        world.wires.set(x, y, crate::wiring::WireColor::Red, true);
    }
}

/// `WorldGen.PlaceTile(num2, num3, 135, mute: true, forced: true, -1, style)`
/// (`WorldGen.cs:26075`) → `Place1x1`'s default branch (`:45214-45233`):
/// `InWorld(x, y, 2)`, a `SolidTile2` floor, an empty cell, then
/// `frameY = 18 * style` (the `frameX` keeps its value — the C# does not write
/// it on this branch; a fresh cell's 0 stands). 135 is not a mechanism
/// (`TileID.Sets.Wiring.IsAMechanism`), so the boulder-floor veto never fires.
fn place_plate(world: &mut World, x: i32, y: i32, style: i32) {
    if !in_world_f(world, x, y, 2) {
        return;
    }
    if !solid_tile2(world, x, y + 1) {
        return;
    }
    if world.tile(x, y).is_some_and(|t| t.active()) {
        return;
    }
    if let Some(t) = world.tile_mut(x, y) {
        // `PlaceTile`'s `tile.Clear(Tile | TilePaint | Slope)` on the inactive
        // cell (`:59473-59476`) before `Place1x1` writes the plate.
        t.kind = 0;
        t.frame_x = 0;
        t.frame_y = 0;
        t.set_paint(0);
        t.set_slope(0);
        t.flags &= !FLAG_HALF_BRICK;
        t.flags |= FLAG_ACTIVE;
        t.kind = 135;
        t.frame_y = (style * 18) as i16;
    }
}

/// `WorldGen.FinishNoTraps` (`WorldGen.cs:25986-26224`): the noTraps seed's
/// heart→life-crystal-boulder conversion, the statue/golden-chest wiring (the
/// 25×25 wire probe, the 15×15 pressure-plate scatter, the 8×8 explosives
/// placement) and the unwired golden-chest→chest revert. Draws on the pass's
/// own stream. `SetBoulderSolidity(false/true)` (`:25988`/`:26186`) is a
/// load-order table poke the port's tile data does not model (the R2
/// precedent — no solidity read happens between the two calls). The
/// `Main.chest` 5346 loop (`:26204-26223`) is the no-chest-inventory stand-in
/// (no records → no draws, the `finish_seeds.rs` discipline).
fn finish_no_traps(world: &mut World, rng: &mut Rng, opts: &GenOptions) {
    let (width, height) = (world.width, world.height);
    let get_good = opts.flags.contains(SeedFlags::FOR_THE_WORTHY);
    let anniversary = opts.flags.contains(SeedFlags::TENTH_ANNIVERSARY);
    let remix = opts.flags.contains(SeedFlags::REMIX);
    // `:25989-25993` — the heart conversion's 1-in-N gate.
    let mut max_value = 5;
    if get_good && !anniversary {
        max_value = 2;
    }
    for i in 50..width - 50 {
        let mut j = 50;
        while j < height - 50 {
            // `:25999-26005` — the && chain draws Next(maxValue) first.
            let heart = rng.range(0, max_value) == 0
                && world.tile(i, j).is_some_and(|t| {
                    t.active() && t.kind == 12 && t.frame_x == 0 && t.frame_y == 0
                });
            if heart {
                for &(dx, dy) in &[(0i32, 0i32), (0, 1), (1, 0), (1, 1)] {
                    if let Some(t) = world.tile_mut(i + dx, j + dy) {
                        t.kind = 665;
                    }
                }
            }
            // `:26006-26009` — the statue/golden-chest gate (even cells only).
            let gate = i % 2 == 0
                && j % 2 == 0
                && world.tile(i, j).is_some_and(|t| {
                    t.active()
                        && (t.kind == 105
                            || (t.kind == 467 && (t.frame_x == 144 || t.frame_x == 162)))
                });
            if !gate {
                j += 1;
                continue;
            }
            // `:26010-26020` — the 3x3 wire probe.
            let mut flag = false;
            for k in (i - 1)..=(i + 1) {
                for l in (j - 1)..=(j + 1) {
                    if world.wires.wire(k, l) {
                        flag = true;
                    }
                }
            }
            if !flag {
                // `:26022-26036` — the 25x25 wire probe (625 draws, break on a hit).
                let mut flag2 = false;
                let num = 25;
                let mut num2 = -1;
                let mut num3 = -1;
                for _ in 0..(num * num) {
                    num2 = rng.range(i - num, i + num + 1);
                    num3 = rng.range(j - num, j + num + 1);
                    if world.wires.wire(num2, num3) {
                        flag2 = true;
                        break;
                    }
                }
                if flag2 {
                    // `:26037-26066` — wire the statue to the found wire.
                    wire_run(world, i, j, num2, num3);
                } else if world.tile(i, j).is_some_and(|t| t.kind == 105) {
                    // `:26067-26112` — the 15x15 pressure-plate scatter.
                    let num = 15;
                    let mut flag3 = false;
                    for _ in 0..(num * num) {
                        num2 = i + rng.range(-num, num + 1);
                        num3 = j + rng.range(-num, num + 1);
                        let style = rng.range(2, 4);
                        place_plate(world, num2, num3, style);
                        if world.tile(num2, num3).is_some_and(|t| t.kind == 135) {
                            flag3 = true;
                            break;
                        }
                    }
                    if flag3 {
                        wire_run(world, i, j, num2, num3);
                    }
                }
            }
            // `:26114-26183` — the golden-chest explosives arm (or the j++ skip).
            if world.tile(i, j).is_some_and(|t| t.kind == 467) && (!anniversary || remix) {
                let num8 = 8;
                for _ in 0..(num8 * num8) {
                    let num10 = i + rng.range(-num8, num8 + 1);
                    let num11 = j + rng.range(-num8, num8 + 1);
                    let nk = kind_at(world, num10, num11);
                    if nk != 0 && nk != 1 && !td::SET_ORE.has(nk) && nk != 59 && nk != 151 {
                        continue;
                    }
                    // `:26125-26139` — the 3x3 all-solid probe.
                    let mut flag4 = true;
                    for num12 in (num10 - 1)..=(num10 + 1) {
                        for num13 in (num11 - 1)..=(num11 + 1) {
                            if !super::biome_detail::solid_tile(world, num12, num13) {
                                flag4 = false;
                            }
                        }
                    }
                    if !flag4 {
                        continue;
                    }
                    // `:26140-26145` — the explosives write (TileFrame deferred).
                    if let Some(t) = world.tile_mut(num10, num11) {
                        t.kind = 141;
                        t.frame_x = 0;
                        t.frame_y = 0;
                        t.set_slope(0);
                        t.flags &= !FLAG_HALF_BRICK;
                    }
                    if !world.tile(num10, num11).is_some_and(|t| t.kind == 141) {
                        continue;
                    }
                    wire_run(world, i, j, num10, num11);
                    break;
                }
            } else {
                j += 1;
            }
            j += 1;
        }
    }
    // `:26187-26203` — the unwired golden-chest → chest revert (20-margin).
    for num16 in 20..width - 20 {
        for num17 in 20..height - 20 {
            let revert = world.tile(num16, num17).is_some_and(|t| {
                t.active() && t.kind == 467 && t.frame_x == 144 && t.frame_y == 0
            }) && !world.wires.wire(num16, num17)
                && !world.wires.wire(num16 + 1, num17)
                && !world.wires.wire(num16, num17 + 1)
                && !world.wires.wire(num16 + 1, num17 + 1);
            if revert {
                for &(dx, fx) in &[(0i32, 36i16), (1, 54)] {
                    for dy in 0..2 {
                        if let Some(t) = world.tile_mut(num16 + dx, num17 + dy) {
                            t.kind = 21;
                            t.frame_x = fx;
                        }
                    }
                }
            }
        }
    }
    // `:26204-26223` — the Main.chest 5346 loop: no chest records → no draws.
}

/// `WorldGen.CheckLilyPad` (`WorldGen.cs:59203-59310`): the lily-pad 518's
/// support check — a non-water liquid kills it; otherwise the first solid
/// (non-top) cell below picks the frame band (grass 0 / hallow 18 / jungle 36)
/// and the pad floats up/down onto the liquid surface, or dies on an invalid
/// floor. The `netMode` sends are no-ops (generation is offline). The
/// `SquareTileFrame` re-frames ride the framing seam.
fn check_lily_pad(world: &mut World, x: i32, y: i32) {
    // `:59209-59217` — a non-water liquid kills the pad.
    if world.tile(x, y).is_some_and(|t| t.liquid_kind() != LiquidKind::Water) {
        kill_tile(world, x, y, true);
        return;
    }
    // `:59218-59226` — walk down to the first solid (non-top) cell.
    let mut num = y;
    while (!world.tile(x, num).is_some_and(|t| t.active())
        || !td::TILE_SOLID.has(kind_at(world, x, num))
        || td::TILE_SOLID_TOP.has(kind_at(world, x, num)))
        && num < world.height - 50
    {
        num += 1;
        if world.tile(x, num).is_none() {
            return;
        }
    }
    let ty = kind_at(world, x, num);
    let mut num2 = -1i32;
    if ty == 2 || ty == 477 {
        num2 = 0;
    }
    if ty == 109 || ty == 116 {
        num2 = 18;
    }
    if ty == 60 {
        num2 = 36;
    }
    if num2 >= 0 {
        // `:59243-59250` — re-frame to the band.
        if world.tile(x, y).is_some_and(|t| t.frame_y as i32 != num2) {
            if let Some(t) = world.tile_mut(x, y) {
                t.frame_y = num2 as i16;
            }
        }
        // `:59251-59267` — float up onto liquid above.
        if world.tile(x, y - 1).is_some_and(|t| t.liquid > 0 && !t.active()) {
            let (fx, fy, paint) = {
                let t = world.tile(x, y).unwrap();
                (t.frame_x, t.frame_y, t.paint())
            };
            if let Some(t) = world.tile_mut(x, y - 1) {
                t.flags |= FLAG_ACTIVE;
                t.kind = 518;
                t.frame_x = fx;
                t.frame_y = fy;
                t.flags &= !FLAG_HALF_BRICK;
                t.set_slope(0);
                t.set_paint(paint);
            }
            if let Some(t) = world.tile_mut(x, y) {
                t.flags &= !FLAG_ACTIVE;
                t.kind = 0;
            }
        } else {
            // `:59270-59273` — a wet pad stays.
            if world.tile(x, y).is_some_and(|t| t.liquid != 0) {
                return;
            }
            let below_active = world.tile(x, y + 1).is_some_and(|t| t.active());
            if !below_active {
                // `:59276-59286` — float down onto air below.
                let (fx, fy, paint) = {
                    let t = world.tile(x, y).unwrap();
                    (t.frame_x, t.frame_y, t.paint())
                };
                if let Some(t) = world.tile_mut(x, y + 1) {
                    t.flags |= FLAG_ACTIVE;
                    t.kind = 518;
                    t.frame_x = fx;
                    t.frame_y = fy;
                    t.flags &= !FLAG_HALF_BRICK;
                    t.set_slope(0);
                    t.set_paint(paint);
                }
                if let Some(t) = world.tile_mut(x, y) {
                    t.flags &= !FLAG_ACTIVE;
                    t.kind = 0;
                }
            } else if world.tile(x, y + 1).is_some_and(|t| {
                t.active()
                    && !td::SET_PLATFORMS.has(t.kind)
                    && (!td::TILE_SOLID.has(t.kind) || td::TILE_SOLID_TOP.has(t.kind))
            }) {
                // `:59292-59299` — an invalid floor kills the pad.
                kill_tile(world, x, y, true);
            }
        }
    } else {
        // `:59302-59309` — no valid support band kills the pad.
        kill_tile(world, x, y, true);
    }
}

/// `WorldGen.AddTrees(undergroundOnly: true)` (`WorldGen.cs:31365-31451`), the
/// remix-only underground arm (`:31423-31450`): the surface loop is skipped
/// (`undergroundOnly`), and the underground band `worldSurface..maxTilesY-300`
/// grows vanity willow 616 / sakura 596 on grass (1-in-7, anniversary always)
/// or a forest tree 5 otherwise. The kind-5 arm rides `growth::grow_tree` (the
/// port's `try_growing_tree_by_type` dispatches only the settings profiles).
fn add_trees_underground(world: &mut World, rng: &mut crate::UnifiedRandom, ws: f64, opts: &GenOptions) {
    let anniversary = opts.flags.contains(SeedFlags::TENTH_ANNIVERSARY);
    // `:31367-31368` — the two head NextDouble draws run even for
    // undergroundOnly (the surface probabilities are unused, but the draws
    // advance the stream before the underground loop).
    let _num = 0.1 + rng.next_double() * 0.35;
    let _num2 = 0.1 + rng.next_double() * 0.35;
    let num3 = ws as i32;
    let num4 = world.height - 300;
    for k in 1..world.width - 1 {
        for l in num3..num4 {
            let grass = world.tile(k, l).is_some_and(|t| t.active() && t.kind == 2);
            if grass && (anniversary || rng.next_max(7) == 0) {
                if rng.next_max(2) == 0 {
                    crate::growth::try_growing_tree_by_type(world, 616, k, l, rng);
                } else {
                    crate::growth::try_growing_tree_by_type(world, 596, k, l, rng);
                }
            } else {
                crate::growth::grow_tree(world, k, l, rng);
            }
        }
    }
}

/// `WorldGen.FinishRemixWorld` (`WorldGen.cs:26350-26770`): the remix seed's
/// cloud→flesh/lesion repaint, the surface conversion band, the spawn-liquid
/// clear, the underground trees, the dungeon repaint, the ash-plant scatter
/// and the lily-pad check. Draws on one name-seeded `UnifiedRandom` (the
/// `AddTrees` grower shares vanilla's genRand). The `Convert` calls ride the
/// port's `spread::convert_cell_flavor` model (tiles + walls, the C# single-cell
/// overload's defaults); the anniversary-remix 661/662 arm reads the
/// post-Convert type (the port's hallow table maps 661/662 → 60, documented).
/// `QuickWater(-2)` rides `liquids::quick_water` (the verbose arg is
/// progress-only). The spawn-clear loop reads the port's interim spawn (the
/// real spawn rides the world tail).
fn finish_remix_world(
    world: &mut World,
    gv: &GenVars,
    rng: &mut crate::UnifiedRandom,
    opts: &GenOptions,
) {
    let (width, height) = (world.width, world.height);
    let ws = gv.layers().world_surface;
    let rock_layer = gv.layers().rock_layer;
    let anniversary = opts.flags.contains(SeedFlags::TENTH_ANNIVERSARY);
    let get_good = opts.flags.contains(SeedFlags::FOR_THE_WORTHY);
    let drunk = opts.flags.contains(SeedFlags::DRUNK);
    let not_the_bees = opts.flags.contains(SeedFlags::NOT_THE_BEES);
    let no_traps = opts.flags.contains(SeedFlags::NO_TRAPS);
    let crimson_left = gv.crimson_left;
    let error_world = opts.micro_flags.contains(MicroSeedFlags::ERROR_WORLD);
    let surface_in_space = opts.micro_flags.contains(MicroSeedFlags::SURFACE_IS_IN_SPACE);
    let no_surface = opts.micro_flags.contains(MicroSeedFlags::NO_SURFACE);
    let anchors = SettleAnchors {
        water_line: gv.water_line,
        lava_line: gv.lava_line,
        rock_layer,
        world_surface: ws,
    };
    let mut remix_sys = LiquidSystem::new(world);
    remix_sys.set_generating(true);
    remix_sys.set_remix_world_gen(true);

    // `:26352-26432` — the cloud/sunplate repaint + the cloud-wall conversion.
    let mut num = 0;
    for i in 25..width - 25 {
        for j in 25..height - 25 {
            let cloud = world.tile(i, j).is_some_and(|t| {
                t.active() && (td::SET_CLOUDS.has(t.kind) || t.kind == 202)
            });
            if cloud {
                if j > num {
                    num = j;
                }
                if !anniversary || get_good {
                    let ty = if drunk {
                        // `:26367-26385` — one Next(-2,3) side draw; crimsonLeft
                        // flips the flesh/lesion side.
                        let left = i < width / 2 + rng.next_range(-2, 3);
                        if crimson_left {
                            if left { 195 } else { 474 }
                        } else if left {
                            474
                        } else {
                            195
                        }
                    } else if opts.crimson {
                        195
                    } else {
                        474
                    };
                    if let Some(t) = world.tile_mut(i, j) {
                        t.kind = ty;
                    }
                }
            }
            let wall = wall_at(world, i, j);
            if (wall != 73 && wall != 82) || (anniversary && !get_good) {
                continue;
            }
            let new_wall = if drunk {
                let left = i < width / 2 + rng.next_range(-2, 3);
                if crimson_left {
                    if left { 77 } else { 233 }
                } else if left {
                    233
                } else {
                    77
                }
            } else if opts.crimson {
                77
            } else {
                233
            };
            if let Some(t) = world.tile_mut(i, j) {
                t.wall = new_wall;
            }
        }
    }
    // `:26433-26585` — the surface conversion band.
    for k in 25..width - 25 {
        for l in 25..height - 25 {
            let conversion_flavor = if opts.crimson {
                crate::spread::ConversionFlavor::Crimson
            } else {
                crate::spread::ConversionFlavor::Corruption
            };
            // `:26442-26516` — the deep-band liquid arms (wall-87 / getgood /
            // notTheBees / drunk+anniversary).
            if (l > num || error_world || surface_in_space)
                && (l as f64) < ws
                && k > BEACH_DISTANCE
                && k < width - BEACH_DISTANCE
            {
                if wall_at(world, k, l) == 87 {
                    if liquid_at(world, k, l) > 0 {
                        if let Some(t) = world.tile_mut(k, l) {
                            t.set_liquid_kind(LiquidKind::Water);
                        }
                    }
                } else if get_good {
                    remix_liquid_arm(world, k, l, LiquidKind::Lava, 374, &[375, 373, 709]);
                } else if not_the_bees {
                    remix_liquid_arm(world, k, l, LiquidKind::Honey, 375, &[374, 373, 709]);
                } else if drunk && anniversary {
                    remix_liquid_arm(world, k, l, LiquidKind::Shimmer, 709, &[374, 373, 375]);
                }
            }
            // `:26517-26571` — the surface conversion band.
            if (l as f64) < ws + rng.next_max(3) as f64 {
                if anniversary && !get_good {
                    if wall_at(world, k, l) != 3 && wall_at(world, k, l) != 83 {
                        crate::spread::convert_cell_flavor(
                            world, k, l, crate::spread::ConversionFlavor::Hallow, true,
                        );
                        if kind_at(world, k, l) == 661 || kind_at(world, k, l) == 662 {
                            if let Some(t) = world.tile_mut(k, l) {
                                t.kind = 109;
                            }
                            if super::biome_detail::solid_tile(world, k, l)
                                && !world.tile(k, l - 1).is_some_and(|t| t.active())
                            {
                                place_hallow_plant(world, k, l);
                            }
                            for m in (k - 3)..=(k + 3) {
                                for n in (l - 3)..=(l + 3) {
                                    if kind_at(world, m, n) == 59 {
                                        if let Some(t) = world.tile_mut(m, n) {
                                            t.kind = 0;
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if drunk {
                    let flavor = if crimson_left {
                        if k < width / 2 + rng.next_range(-2, 3) {
                            crate::spread::ConversionFlavor::Crimson
                        } else {
                            crate::spread::ConversionFlavor::Corruption
                        }
                    } else if k < width / 2 + rng.next_range(-2, 3) {
                        crate::spread::ConversionFlavor::Corruption
                    } else {
                        crate::spread::ConversionFlavor::Crimson
                    };
                    crate::spread::convert_cell_flavor(world, k, l, flavor, true);
                } else {
                    crate::spread::convert_cell_flavor(world, k, l, conversion_flavor, true);
                }
            }
            // `:26572-26575` — the marble/granite wall clear above the surface.
            if (l as f64) < ws - rng.next_range(19, 22) as f64
                && (wall_at(world, k, l) == 178 || wall_at(world, k, l) == 180)
            {
                if let Some(t) = world.tile_mut(k, l) {
                    t.wall = 0;
                }
            }
            // `:26576-26579` — the obsidian kill.
            if world.tile(k, l).is_some_and(|t| t.active() && t.kind == 56) {
                kill_tile(world, k, l, true);
            }
            // `:26580-26583` — the exposed dirt → grass below the rock layer.
            if (l as f64) > rock_layer
                && l < height - 350
                && world.tile(k, l).is_some_and(|t| t.kind == 0 && t.active())
                && (!world.tile(k - 1, l - 1).is_some_and(|t| t.active())
                    || !super::biome_detail::solid_tile(world, k, l - 1)
                    || !world.tile(k + 1, l - 1).is_some_and(|t| t.active())
                    || !world.tile(k - 1, l).is_some_and(|t| t.active())
                    || !world.tile(k + 1, l).is_some_and(|t| t.active())
                    || !world.tile(k - 1, l + 1).is_some_and(|t| t.active())
                    || !world.tile(k, l + 1).is_some_and(|t| t.active())
                    || !world.tile(k + 1, l + 1).is_some_and(|t| t.active()))
            {
                if let Some(t) = world.tile_mut(k, l) {
                    t.kind = 2;
                }
            }
        }
    }
    // `:26586` — QuickWater(-2).
    super::liquids::quick_water(world, &mut remix_sys, &anchors, gv.shimmer_position, no_traps);
    // `:26587-26632` — the spawn-liquid clear loop (randomSpawn veto).
    let mut flag = !opts.micro_flags.contains(MicroSeedFlags::RANDOM_SPAWN);
    while flag {
        let num2 = (width as f64 * 0.38) as i32;
        let num3 = (width as f64 * 0.62) as i32;
        let num4 = height - 135;
        for num5 in num2..(num3 + 15) {
            for num6 in (height - 200)..(num4 + 10) {
                if let Some(t) = world.tile_mut(num5, num6) {
                    t.liquid = 0;
                    if t.kind == 58 {
                        t.kind = 57;
                    }
                    if t.kind == 230 || t.kind == 56 || t.kind == 659 {
                        t.flags &= !FLAG_ACTIVE;
                    }
                }
            }
        }
        super::liquids::quick_water(world, &mut remix_sys, &anchors, gv.shimmer_position, no_traps);
        flag = false;
        for num7 in -10..15 {
            if world
                .tile(world.spawn_x, world.spawn_y - num7)
                .is_some_and(|t| t.liquid > 0)
            {
                flag = true;
                break;
            }
        }
    }
    // `:26633` — AddTrees(undergroundOnly: true).
    add_trees_underground(world, rng, ws, opts);
    // `:26634-26717` — the dungeon repaint (!anniversary || getgood).
    if !anniversary || get_good {
        for num8 in 10..width - 10 {
            let mut color: u8 = 22;
            let mut color2: u8 = 22;
            if drunk {
                if (crimson_left && num8 < width / 2) || (!crimson_left && num8 > width / 2) {
                    color2 = 13;
                    color = 13;
                }
            } else if opts.crimson {
                color2 = 13;
                color = 13;
            }
            for num9 in 10..height - 10 {
                let (kind, active, wall, frame_y) = {
                    let t = world.tile(num8, num9).copied().unwrap_or_default();
                    (t.kind, t.active(), t.wall, t.frame_y)
                };
                if active && (TILE_DUNGEON.contains(&kind) || td::SET_CRACKED_BRICKS.has(kind)) {
                    if let Some(t) = world.tile_mut(num8, num9) {
                        t.set_paint(color);
                    }
                }
                if WALL_DUNGEON.contains(&wall) {
                    world.set_wall_paint(num8, num9, 25);
                    if kind == 19 && frame_y != 180 {
                        if let Some(t) = world.tile_mut(num8, num9) {
                            t.set_paint(color);
                        }
                    }
                }
                if active {
                    let mut flag2 = false;
                    if anniversary {
                        if !no_surface && kind == 191 {
                            flag2 = true;
                        }
                        if kind == 151 {
                            if let Some(t) = world.tile_mut(num8, num9) {
                                t.set_paint(color);
                            }
                        }
                    }
                    if kind == 226 {
                        flag2 = true;
                    }
                    if kind == 61 && kind_at(world, num8, num9 + 1) == 226 {
                        flag2 = true;
                    }
                    if kind == 137 {
                        let num10 = frame_y as i32 / 18;
                        if (1..=4).contains(&num10) {
                            flag2 = true;
                        }
                    }
                    if flag2 {
                        if let Some(t) = world.tile_mut(num8, num9) {
                            t.set_paint(color2);
                        }
                    }
                }
                if !no_surface && wall == 244 {
                    world.set_wall_paint(num8, num9, 25);
                }
                if wall == 34 {
                    world.set_wall_paint(num8, num9, 25);
                }
                if wall == 87 {
                    world.set_wall_paint(num8, num9, 25);
                    if let Some(t) = world.tile_mut(num8, num9) {
                        t.set_paint(color2);
                    }
                }
            }
        }
    }
    // `:26718-26733` — the ash-plant frame scatter (Main.rand == genRand).
    let num11 = width as f64 / 4200.0 * rng.next_range(2, 5) as f64;
    let mut num12 = 0.0;
    while num12 < num11 {
        let num13 = rng.next_range((width as f64 * 0.39) as i32, (width as f64 * 0.61) as i32);
        let num14 = rng.next_range(10, 31);
        for num15 in (num13 - num14)..=(num13 + num14) {
            for num16 in (height - 250)..(height - 25) {
                if kind_at(world, num15, num16) == 637 {
                    if let Some(t) = world.tile_mut(num15, num16) {
                        t.frame_x = (18 * rng.next_range(6, 11)) as i16;
                    }
                }
            }
        }
        num12 += 1.0;
    }
    // `:26734-26758` — the notTheBees wall-86 surface strip.
    if not_the_bees {
        for num17 in 3..width - 3 {
            let mut flag3 = true;
            let mut num18 = 0.0;
            while num18 < ws {
                let y = num18 as i32;
                if flag3 {
                    if wall_at(world, num17, y) == 86 {
                        if let Some(t) = world.tile_mut(num17, y) {
                            t.wall = 0;
                        }
                    }
                    if world.tile(num17, y).is_some_and(|t| t.active()) {
                        flag3 = false;
                    }
                } else if wall_at(world, num17, y) == 0
                    && wall_at(world, num17, y + 1) == 0
                    && wall_at(world, num17, y + 2) == 0
                    && wall_at(world, num17, y + 3) == 0
                    && wall_at(world, num17, y + 4) == 0
                    && wall_at(world, num17 - 1, y) == 0
                    && wall_at(world, num17 + 1, y) == 0
                    && wall_at(world, num17 - 2, y) == 0
                    && wall_at(world, num17 + 2, y) == 0
                    && !world.tile(num17, y).is_some_and(|t| t.active())
                    && !world.tile(num17, y + 1).is_some_and(|t| t.active())
                    && !world.tile(num17, y + 2).is_some_and(|t| t.active())
                    && !world.tile(num17, y + 3).is_some_and(|t| t.active())
                {
                    flag3 = true;
                }
                num18 += 1.0;
            }
        }
    }
    // `:26759` — QuickWater(-2).
    super::liquids::quick_water(world, &mut remix_sys, &anchors, gv.shimmer_position, no_traps);
    // `:26760-26769` — the lily-pad check sweep.
    for num19 in 0..width {
        for num20 in 0..height {
            if kind_at(world, num19, num20) == 518 {
                check_lily_pad(world, num19, num20);
            }
        }
    }
}

/// The remix deep-band liquid arm (`WorldGen.cs:26451-26515`): re-type the
/// liquid (killing lava-doomed tiles for lava/shimmer), convert the drip kind,
/// kill the honey/obsidian/shimmer blocks, and re-frame the herbs (the
/// `TileFrame` rides the framing seam). `drip` is the arm's target drip kind;
/// `drip_sources` the three source kinds it converts from.
fn remix_liquid_arm(world: &mut World, k: i32, l: i32, kind: LiquidKind, drip: u16, drip_sources: &[u16]) {
    if liquid_at(world, k, l) > 0 {
        if kind != LiquidKind::Honey
            && world.tile(k, l).is_some_and(|t| td::TILE_LAVA_DEATH.has(t.kind))
        {
            kill_tile(world, k, l, true);
        }
        if let Some(t) = world.tile_mut(k, l) {
            t.set_liquid_kind(kind);
        }
    }
    let tk = kind_at(world, k, l);
    if drip_sources.contains(&tk) {
        if let Some(t) = world.tile_mut(k, l) {
            t.kind = drip;
        }
    }
    if tk == 230 || tk == 229 || tk == 659 || tk == 56 {
        kill_tile(world, k, l, true);
    }
    // `:26469-26472` — the herb 82/83/84 TileFrame rides the framing seam.
}

/// `WorldGen.PlaceTile(k, l, 110)` (`WorldGen.cs:26530`) → the flower arm
/// (`:59507-59580`): `IsFitToPlaceFlowerIn` (a flat hallowed-grass floor), then
/// the wall-gated plant placement. The port places the hallow plant 110 on a
/// valid floor with the default frame (the `NextFromList` frame roll and the
/// `AllowsPlantsToGrow` wall gate ride the plant-placement seam — the remix
/// anniversary arm's floor is the converted 109, so the simple write stands).
fn place_hallow_plant(world: &mut World, k: i32, l: i32) {
    // `IsFitToPlaceFlowerIn` (`:58231-58251`) for type 110: the floor must be
    // active, flat, and hallowed grass 109 / golf grass 492.
    let floor_ok = world.tile(k, l + 1).is_some_and(|t| {
        t.active() && t.slope() == 0 && !t.half_brick() && (t.kind == 109 || t.kind == 492)
    });
    if !floor_ok {
        return;
    }
    if world.tile(k, l).is_some_and(|t| t.active()) {
        return;
    }
    if let Some(t) = world.tile_mut(k, l) {
        t.flags |= FLAG_ACTIVE;
        t.kind = 110;
    }
}

// --- the shimmer clean-up (W4-d106 R6, None-guarded) ------------------------

/// `WorldGen.Shimmerator` (`WorldGen.cs:35118-35127`) + `ShimmeratorNext`
/// (`:35129-35147`) + `Shimminate` (`:35104-35116`): flood-fill the connected
/// non-solid liquid pocket (or obsidian 56) from `(x, y)` under the
/// `maxTileCount` cap, then shimmer-ify every counted cell (liquid 255 +
/// shimmer type, killing trees 5 / obsidian 56 / cut tiles except pots 28).
/// The C# recursion walks an explicit stack here (the fill carries no RNG, so
/// the traversal order is free — the `count_tiles` precedent). A border touch
/// bails the whole pocket (`numTileCount = maxTileCount`).
fn shimmerator(world: &mut World, x: i32, y: i32, max_tile_count: i32) {
    let mut counted: Vec<(i32, i32)> = Vec::new();
    let mut seen: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
    let mut stack = vec![(x, y)];
    let mut num_tile_count = 0;
    let mut bailed = false;
    while let Some((cx, cy)) = stack.pop() {
        if num_tile_count >= max_tile_count {
            break;
        }
        if cx <= 1 || cx >= world.width - 1 || cy <= 1 || cy >= world.height - 1 {
            num_tile_count = max_tile_count;
            bailed = true;
            break;
        }
        if !seen.insert((cx, cy)) {
            continue;
        }
        let Some(t) = world.tile(cx, cy).copied() else {
            continue;
        };
        // `:35137` — the pocket gate: a non-solid liquid cell (not already
        // shimmer) or an active obsidian 56.
        let pocket = (!super::biome_detail::solid_tile(world, cx, cy)
            && t.liquid_kind() != LiquidKind::Shimmer
            && t.liquid > 0)
            || (t.active() && t.kind == 56);
        if !pocket {
            continue;
        }
        counted.push((cx, cy));
        num_tile_count += 1;
        stack.push((cx - 1, cy));
        stack.push((cx + 1, cy));
        stack.push((cx, cy - 1));
        stack.push((cx, cy + 1));
    }
    // `:35123-35126` — Shimminate only on a non-empty, under-cap pocket.
    if !bailed && num_tile_count > 0 && num_tile_count < max_tile_count {
        for &(px, py) in &counted {
            if let Some(t) = world.tile_mut(px, py) {
                t.set_liquid_kind(LiquidKind::Shimmer);
                t.liquid = u8::MAX;
            }
            let tk = kind_at(world, px, py);
            if tk == 5 || tk == 56 || (crate::tileobject::tile_cut(tk) && tk != 28) {
                kill_tile(world, px, py, true);
            }
        }
    }
}

/// `WorldGen.ShimmerCleanUp` (`WorldGen.cs:34716-34821`): the shimmer disc's
/// ore→stone / block-kill / moss→stone sweep, the jittered cleanse ellipse,
/// the shimmer-fill walk, the anniversary shimmer band and the tree-in-shimmer
/// kill. `None`-guarded on `gv.shimmer_position` (the C# guards on `(0,0)`;
/// the producer at the gate is g36 A1's published position — d102's Shimmer
/// copy DROPs per research-88 §2.1). The disc's two `NextDouble` jitter
/// draws ride the THREADED Final Cleanup stream (the scatter's continuation,
/// `final_cleanup`'s note). Gate keep-one: this local arm DROPs onto g36's
/// `shimmer.rs::shimmer_clean_up` (research-88 §6.4, research-312 §2).
fn shimmer_clean_up(world: &mut World, rng: &mut Rng, gv: &GenVars, opts: &GenOptions) {
    let Some((sx, sy)) = gv.shimmer_position else {
        return;
    };
    let anniversary = opts.flags.contains(SeedFlags::TENTH_ANNIVERSARY);
    super::liquids::shimmer_remove_water(world, gv.shimmer_position);
    let num = 120;
    let num2 = 90;
    let (num3, num4) = (sx - num, sx + num);
    let (num5, num6) = (sy - num, sy + num);
    // `:34730-34763` — the disc sweep (i = y outer, j = x inner).
    for i in num5..=num6 {
        for j in num3..=num4 {
            let dx = (j - sx).abs() as f64;
            let dy = (i - sy).abs() as f64;
            // `:34734` — the inner disc (radius 120).
            if (dx * dx + dy * dy).sqrt() as i32 < num {
                let tk = kind_at(world, j, i);
                if tk == 22 || tk == 204 {
                    if let Some(t) = world.tile_mut(j, i) {
                        t.kind = 1;
                    }
                }
                if tk == 51 || tk == 56 || tk == 229 || tk == 230 || tk == 659 {
                    if let Some(t) = world.tile_mut(j, i) {
                        t.flags &= !FLAG_ACTIVE;
                    }
                }
                if td::SET_CONVERSION_MOSS.has(tk) {
                    if let Some(t) = world.tile_mut(j, i) {
                        t.kind = 1;
                    }
                }
                if tk == 184 {
                    if let Some(t) = world.tile_mut(j, i) {
                        t.flags &= !FLAG_ACTIVE;
                    }
                }
            }
            // `:34753` — the jittered cleanse ellipse (radius 90, y-stretch
            // 1.4 above / 1.2 below). The two NextDouble draws run per cell.
            let stretch = if (i as f64) > sy as f64 { 1.2 } else { 1.4 };
            let jx = dx * (1.0 + rng.next_f64() * 0.02);
            let jy = dy * stretch * (1.0 + rng.next_f64() * 0.02);
            if (jx * jx + jy * jy).sqrt() as i32 < num2 {
                if kind_at(world, j, i) == 162 {
                    if let Some(t) = world.tile_mut(j, i) {
                        t.flags &= !FLAG_ACTIVE;
                    }
                }
                if let Some(t) = world.tile_mut(j, i) {
                    t.wall = 0;
                }
                crate::spread::convert_cleanse(world, j, i, 3);
            }
        }
    }
    // `:34764-34788` — the shimmer-fill walk from the disc centre.
    let mut num7 = sx;
    let mut num8 = sy;
    let mut b: u8 = 127;
    // `:34767` — `Liquid.tilesIgnoreWater(true)`. The port's shimmer-fill walk
    // and `shimmerator` read `biome_detail::solid_tile` (the stock table), not
    // the LiquidSystem solidity, so the toggle is a documented no-op stand-in
    // here (the boulder/grate family it lifts does not appear in the shimmer
    // disc's fill path). Armed for the 1:1 shape; restored at `:34810`.
    let mut sys = LiquidSystem::new(world);
    sys.set_ignore_solids(true);
    // `:34768` — while liquid <= b OR not shimmer.
    while !world
        .tile(num7, num8)
        .is_some_and(|t| t.liquid > b && t.liquid_kind() == LiquidKind::Shimmer)
    {
        while !world.tile(num7, num8).is_some_and(|t| t.active()) {
            if let Some(t) = world.tile_mut(num7, num8) {
                t.liquid = b;
                t.set_liquid_kind(LiquidKind::Shimmer);
            }
            num7 -= 1;
        }
        num7 = sx;
        while !world.tile(num7, num8).is_some_and(|t| t.active()) {
            if let Some(t) = world.tile_mut(num7, num8) {
                t.liquid = b;
                t.set_liquid_kind(LiquidKind::Shimmer);
            }
            num7 += 1;
        }
        num7 = sx;
        num8 += 1;
        b = u8::MAX;
        if world.tile(num7, num8).is_some_and(|t| t.active()) {
            break;
        }
    }
    // `:34789-34809` — the anniversary shimmer band (drip→709 + Shimmerator).
    if anniversary {
        let num9 = 170;
        // `WorldGen.maxTileCount` — the last classic writer is Grass Wall's
        // 3500 (`:19593`, R1); the port carries no later reset before this.
        let max_tile_count = 3500;
        for k in (sx - num9)..=(sx + num9) {
            for l in (sy + 40)..(world.height - 330 - 100) {
                if in_world_f(world, k, l, 0) && wall_at(world, k, l) != 86 {
                    let tk = kind_at(world, k, l);
                    if tk == 375 || tk == 374 || tk == 373 {
                        if let Some(t) = world.tile_mut(k, l) {
                            t.kind = 709;
                        }
                    }
                    if world.tile(k, l).is_some_and(|t| {
                        t.liquid > 0
                            && t.liquid_kind() != LiquidKind::Shimmer
                            && t.liquid_kind() != LiquidKind::Honey
                    }) {
                        shimmerator(world, k, l, max_tile_count);
                    }
                }
            }
        }
    }
    sys.set_ignore_solids(false);
    // `:34811-34820` — the tree-in-shimmer kill (10-margin).
    for m in 10..world.width - 10 {
        for n in 10..world.height - 10 {
            if world.tile(m, n).is_some_and(|t| {
                t.liquid > 0 && t.liquid_kind() == LiquidKind::Shimmer && t.kind == 5
            }) {
                kill_tile(world, m, n, true);
            }
        }
    }
}

/// `Final Cleanup` (`GenPassNameID.FinalCleanup`, `WorldGen.cs:22141-22490`)
/// — W4-d106 R6. The wall-hole fill, the surface-desert gate, the whole-world
/// sweep, the poo-668 scatter, the seed `Finish*` dispatch, the shimmer
/// clean-up and the final `Liquid.LiquidCheck` merge sweep. The classic body
/// draws only the poo scatter's `pass_rng("Final Cleanup")` stream; the seed
/// arms draw their own name-seeded streams. The layer anchors ride
/// `gv.layers()` (the port's `world.world_surface`/`rock_layer` are only
/// written at the world tail, after this pass).
///
/// The seed dispatch (`:22423-22471`) keeps the M5-SEED-1 seam: drunk /
/// getgood / anniversary already run from `step_seed_finish`
/// (`finish_seeds.rs`) — re-running them here would double-draw, so this pass
/// ports only `FinishNoTraps` and `FinishRemixWorld` (flag-gated) plus the
/// `NotTheBees` gate (`liquids::not_the_bees`). The vampire /
/// ShimmerSpawnHalfOfWorld / RainbowSand arms read flags the port's classic
/// path never sets (documented stand-ins); `FinalizeSecretSeeds` + the
/// anniversary paint-trees tail ride `step_seed_finish` (`micro_finish` /
/// `micro_surface`); the NPC / angler / extra-spawn tails are no-port-state
/// no-ops. `notTheBees = false` / `getGoodWorldGen = false` (`:22486-22487`)
/// are static resets the port's immutable `opts` does not model.
pub(crate) fn final_cleanup(world: &mut World, gv: &mut GenVars, streams: &mut Streams, opts: &GenOptions) {
    let ws = gv.layers().world_surface;
    let rock_layer = gv.layers().rock_layer;
    // `:22143` — `Main.tileSolid[484] = false` is a load-order poke the port's
    // tile data does not model (the `desert.rs` precedent, module doc).
    // `:22144` — the wall-hole fill over the surface band.
    fill_wall_holes_in_area(world, 0, 0, world.width, ws as i32);
    // `:22146-22149` — the surface-desert gate
    // (`surfaceIsDesertNormalFunction` = surfaceIsDesert && !noSurface).
    if opts.micro_flags.contains(MicroSeedFlags::SURFACE_IS_DESERT)
        && !opts.micro_flags.contains(MicroSeedFlags::NO_SURFACE)
    {
        let active = crate::seeds::active_secret_seed_count(opts.micro_flags, opts.flags);
        super::micro_surface::surface_is_desert(
            world,
            &mut streams.pass_rng("DoSurfaceIsDesert"),
            ws,
            opts.micro_flags,
            active,
        );
    }
    // `:22150-22378` — the whole-world sweep.
    final_cleanup_sweep(world, ws, rock_layer, opts);
    // `:22379-22422` — the poo-668 scatter. ONE `pass_rng("Final Cleanup")`
    // instance threads through the whole pass tail (research-88 §6.4 stream
    // note): the C# `genRand` is a single per-pass UnifiedRandom, so the
    // ShimmerCleanUp ellipse jitter (`:34753`) CONTINUES the scatter's
    // sequence — a fresh `pass_rng` at the cleanup call would replay the
    // scatter's draws instead. (The flag-gated Finish* hosts draw their own
    // name-seeded streams — the recorded M5-SEED-1 deviation; on a classic
    // world nothing draws between the scatter and the cleanup.)
    let mut fc_rng = streams.pass_rng("Final Cleanup");
    poo_scatter(world, &mut fc_rng, opts);
    // `:22423-22426` — FinishNoTraps (flag-gated).
    if opts.flags.contains(SeedFlags::NO_TRAPS) {
        finish_no_traps(world, &mut streams.pass_rng("FinishNoTraps"), opts);
    }
    // `:22427-22434` — the anniversary / drunk Finish* tails ride
    // `step_seed_finish` (the M5-SEED-1 seam, module doc).
    // `:22435-22439` — NotTheBees + FinishNotTheBees. The conversion gate rides
    // `liquids::not_the_bees` (the body is the notTheBees lane's); the
    // `FinishNotTheBees` tail (`:25388-25797`) is the same lane's stand-in.
    super::liquids::not_the_bees(world, opts);
    // `:22440-22443` — FinishGetGoodWorld rides `step_seed_finish`.
    // `:22444-22447` — FinishRemixWorld (flag-gated).
    if opts.flags.contains(SeedFlags::REMIX) {
        finish_remix_world(world, gv, &mut streams.pass_unified("FinishRemixWorld"), opts);
    }
    // `:22448-22456` — ShimmerSpawnHalfOfWorld / RainbowSandAndBlackSandWalls
    // read `Main.onlyShimmerOceanWorlds` (no shimmer-ocean mode in the port's
    // classic path) — both gates read false (documented stand-ins).
    // `:22457-22466` — the vampire spawn/vanity tail reads `Main.vampireSeed`
    // (the port carries no vampire world state) — a no-op stand-in.
    // `:22467` — FinalizeSecretSeeds rides `step_seed_finish`.
    // `:22468-22471` — the anniversary paint-trees/mushrooms tail rides
    // `step_seed_finish` (the M5-SEED-1 seam).
    // `:22472` — ShimmerCleanUp (None-guarded), on the THREADED scatter
    // stream (the note above). KEEP-ONE (research-88 §2.1/§6.4/§7): at the
    // gate g36's `shimmer.rs::shimmer_clean_up` is the surviving arm and
    // this local copy (plus `shimmerator` below) DROPs; the compose-hunk
    // moves the call to the tail of g41's `step_seed_finish` with the SAME
    // threaded Rng stashed through GenVars (research-312 §2 publishes the
    // d106-side drop hunks). The merge sweep below rides the g47/g46-union
    // `LiquidSystem` surface at the graft (`set_gen_settle_context` +
    // `set_gen_shimmer_position` + a pub `gen_merge_sweep` wrapper).
    shimmer_clean_up(world, &mut fc_rng, gv, opts);
    // `:22473-22483` — the final whole-world LiquidCheck merge sweep
    // (y-outer / x-inner, createMergeTilesDuringGen: true).
    let mut sys = LiquidSystem::new(world);
    sys.set_generating(true);
    sys.set_remix_world_gen(opts.flags.contains(SeedFlags::REMIX));
    for num21 in 0..world.height {
        for num22 in 0..world.width {
            if let Some(t) = world.tile(num22, num21).copied() {
                if t.liquid > 0 {
                    sys.liquid_check(world, num22, num21, liquid_code(t.liquid_kind()), true);
                }
            }
        }
    }
    // `:22484-22488` — ExtraSpawnPointManager / TownNPCPositionsCleanup /
    // AnglerQuestSwap are no-port-state no-ops; the `notTheBees = false` /
    // `getGoodWorldGen = false` static resets do not map onto immutable opts.
}

// --- W4-d106 R3 witnesses (written NOT run — code-only branch) ---------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::Tile;

    fn opts() -> GenOptions {
        GenOptions { width: 200, height: 200, ..Default::default() }
    }

    /// The whole-world slope clear (`:21526-21536`): a non-SaveSlopes active
    /// tile loses its slope and half-brick; a solid (SaveSlopes) tile keeps
    /// them.
    #[test]
    fn r3_the_slope_clear_spares_save_slopes_kinds() {
        let mut w = World::new("cleanup".into(), 200, 200);
        // A plant (kind 3, non-solid, not SaveSlopes) with a slope.
        let i = w.index(100, 100);
        w.tiles[i] = Tile { kind: 3, flags: FLAG_ACTIVE, ..Default::default() };
        w.tiles[i].set_slope(2);
        w.tiles[i].flags |= FLAG_HALF_BRICK;
        // A stone block (solid → SaveSlopes) with a slope.
        let j = w.index(101, 100);
        w.tiles[j] = Tile { kind: 1, flags: FLAG_ACTIVE, ..Default::default() };
        w.tiles[j].set_slope(2);
        tile_cleanup(&mut w, &mut Rng(42), &opts());
        assert_eq!(w.tile(100, 100).unwrap().slope(), 0, "the plant lost its slope");
        assert!(!w.tile(100, 100).unwrap().half_brick());
        assert_eq!(w.tile(101, 100).unwrap().slope(), 2, "the stone kept its slope");
    }

    /// The wall-13/14 liquid strip (`:21628-21631`) runs inside the 40-margin
    /// sweep.
    #[test]
    fn r3_the_hellstone_wall_holds_no_liquid() {
        let mut w = World::new("cleanup".into(), 200, 200);
        let i = w.index(100, 100);
        w.tiles[i] = Tile { wall: 13, liquid: 200, ..Default::default() };
        tile_cleanup(&mut w, &mut Rng(42), &opts());
        assert_eq!(w.tile(100, 100).unwrap().liquid, 0);
    }

    /// The pass is deterministic (a == b on the same seed) and the skyblock
    /// `denySomeGeneration` gate (`:21522`) skips it tile-for-tile.
    #[test]
    fn r3_the_pass_is_deterministic_and_skyblock_skips_it() {
        let mut a = World::new("cleanup".into(), 200, 200);
        let mut b = World::new("cleanup".into(), 200, 200);
        for x in 50..150 {
            let i = a.index(x, 120);
            a.tiles[i] = Tile { kind: 1, flags: FLAG_ACTIVE, ..Default::default() };
            b.tiles[i] = Tile { kind: 1, flags: FLAG_ACTIVE, ..Default::default() };
            let j = a.index(x, 119);
            a.tiles[j].liquid = 255;
            b.tiles[j].liquid = 255;
        }
        tile_cleanup(&mut a, &mut Rng(42), &opts());
        tile_cleanup(&mut b, &mut Rng(42), &opts());
        assert_eq!(a.tiles, b.tiles, "a == b");
        // Skyblock with no surface seeds → denySomeGeneration skips.
        let mut sky = World::new("cleanup".into(), 200, 200);
        let i = sky.index(100, 100);
        sky.tiles[i] = Tile { wall: 13, liquid: 200, ..Default::default() };
        let before = sky.tiles.clone();
        let sky_opts = GenOptions {
            flags: SeedFlags::from(SeedFlags::SKYBLOCK),
            ..opts()
        };
        tile_cleanup(&mut sky, &mut Rng(42), &sky_opts);
        assert_eq!(sky.tiles, before, "skyblock skips the pass");
    }

    // --- W4-d106 R5 witnesses (written NOT run — code-only branch) ----------

    /// A wired dart trap (137) with NO trigger is a broken circuit: the
    /// sweep strips its wire and kills the trap.
    #[test]
    fn r5_a_broken_trap_circuit_is_stripped() {
        let mut w = World::new("traps".into(), 200, 200);
        // A dart trap at (100,100), wired, no trigger anywhere.
        let i = w.index(100, 100);
        w.tiles[i] = Tile { kind: 137, flags: FLAG_ACTIVE, ..Default::default() };
        w.wires.set(100, 100, crate::wiring::WireColor::Red, true);
        w.wires.set(101, 100, crate::wiring::WireColor::Red, true);
        remove_broken_traps(&mut w, &opts());
        assert!(!w.wires.wire(100, 100), "the broken wire was stripped");
        assert!(!w.wires.wire(101, 100), "the whole network was stripped");
        assert!(
            !w.tile(100, 100).is_some_and(|t| t.active() && t.kind == 137),
            "the trap died"
        );
    }

    /// A wired trap WITH a trigger (a pressure plate 135) is a working
    /// circuit: the sweep leaves it alone.
    #[test]
    fn r5_a_working_circuit_is_left_alone() {
        let mut w = World::new("traps".into(), 200, 200);
        let i = w.index(100, 100);
        w.tiles[i] = Tile { kind: 137, flags: FLAG_ACTIVE, ..Default::default() };
        let j = w.index(102, 100);
        w.tiles[j] = Tile { kind: 135, flags: FLAG_ACTIVE, ..Default::default() };
        for x in 100..=102 {
            w.wires.set(x, 100, crate::wiring::WireColor::Red, true);
        }
        remove_broken_traps(&mut w, &opts());
        assert!(w.wires.wire(100, 100), "the working wire survived");
        assert!(
            w.tile(100, 100).is_some_and(|t| t.active() && t.kind == 137),
            "the working trap survived"
        );
    }

    /// The pass is deterministic (a == b) and the noTraps gate skips it.
    #[test]
    fn r5_the_pass_is_deterministic_and_no_traps_skips_it() {
        let build = || {
            let mut w = World::new("traps".into(), 200, 200);
            let i = w.index(100, 100);
            w.tiles[i] = Tile { kind: 137, flags: FLAG_ACTIVE, ..Default::default() };
            w.wires.set(100, 100, crate::wiring::WireColor::Red, true);
            w
        };
        let mut a = build();
        let mut b = build();
        remove_broken_traps(&mut a, &opts());
        remove_broken_traps(&mut b, &opts());
        assert_eq!(a.tiles, b.tiles, "a == b");
        assert_eq!(a.wires.wire(100, 100), b.wires.wire(100, 100));
        // noTraps (without anniversary/notTheBees) skips the sweep.
        let mut nt = build();
        let nt_opts = GenOptions {
            flags: SeedFlags::from(SeedFlags::NO_TRAPS),
            ..opts()
        };
        remove_broken_traps(&mut nt, &nt_opts);
        assert!(nt.wires.wire(100, 100), "noTraps skips the sweep");
    }

    /// e20 research-148 §3b (FIXED, retire phase): the C# `Utils.Swap` does
    /// NOT clear the swapped-in `t2` — leftover cells from a capped inner
    /// wave survive and are processed in later outer waves. A wired plus at
    /// (100,100) (all FIVE cells wired) with a 2-pop inner cap (1 effective
    /// pop/wave): wave 2 pops the wired (99,100) — its neighbour queue keeps
    /// `t2` non-empty so the outer loop continues — and breaks with the
    /// three other wired cells still in `t`; the old `t2.clear()` dropped
    /// them (their wires survived), the 1:1 swap carries them in `t2` into
    /// a later wave (before that wave's own appended neighbours — the C#
    /// order). Small caps via the test seam — the
    /// vanilla 2000-pop edge is the same code path.
    #[test]
    fn r5_the_bfs_carries_capped_leftovers_like_the_cs_swap() {
        let mut w = World::new("traps".into(), 200, 200);
        let plus = [(100, 100), (99, 100), (101, 100), (100, 99), (100, 101)];
        for (x, y) in plus {
            w.wires.set(x, y, crate::wiring::WireColor::Red, true);
        }
        let mut visited = Vec::new();
        clear_broken_traps_capped(&mut w, (100, 100), &mut visited, 2000, 2);
        for (x, y) in plus {
            assert!(!w.wires.wire(x, y), "the leftover cell ({x},{y}) was stripped");
            assert!(visited.contains(&(x, y)), "({x},{y}) was visited");
        }
    }

    /// The outer-cap exhaustion path reads the same way: with 2 outer waves
    /// (1 effective swap) only the start cell is processed; the queued
    /// neighbours stay unvisited in `t2` — the C# breaks before the next
    /// swap, so their wires survive (no strip of unvisited cells).
    #[test]
    fn r5_the_outer_cap_leaves_the_queue_unprocessed() {
        let mut w = World::new("traps".into(), 200, 200);
        for (x, y) in [(100, 100), (101, 100), (100, 99), (100, 101)] {
            w.wires.set(x, y, crate::wiring::WireColor::Red, true);
        }
        let mut visited = Vec::new();
        clear_broken_traps_capped(&mut w, (100, 100), &mut visited, 2, 2000);
        assert!(!w.wires.wire(100, 100), "the start was stripped");
        assert!(w.wires.wire(101, 100), "the queued neighbour kept its wire");
        assert_eq!(visited, vec![(100, 100)]);
    }

    // --- W4-d106 R6 witnesses (written NOT run — code-only branch) -----------

    /// A GenVars with the layer anchors the Final Cleanup sweep reads.
    fn gv_with_layers(ws: f64, rock: f64) -> GenVars {
        let mut gv = GenVars::default();
        gv.set_layers_for_test(super::super::terrain::Layers {
            avg_surface: ws,
            world_surface: ws,
            rock_layer: rock,
            hell_top: 480,
            surface_low: ws - 20.0,
            surface_high: ws + 20.0,
            rock_low: rock - 20.0,
            rock_high: rock + 20.0,
        });
        gv
    }

    /// The sand-fall column (`:22155-22205`): a sand tile over air near the
    /// surface drops down the open column; a full 10-drop onto air hardens the
    /// base to 397. The world is 1024 wide so the column sits outside the ±380
    /// `oceanDepths` beach margins (a 512-wide world is all beach).
    #[test]
    fn r6_the_sand_falls_and_hardens() {
        let mut w = World::new("fc".into(), 1024, 512);
        // A sand tile at (512, 100) with air below (ws = 120, oceanLevel 240).
        let i = w.index(512, 100);
        w.tiles[i] = Tile { kind: 53, flags: FLAG_ACTIVE, ..Default::default() };
        final_cleanup_sweep(&mut w, 120.0, 280.0, &opts());
        // The sand dropped 10 cells (101..110); the base at 111 hardened to 397.
        assert_eq!(w.tile(512, 111).unwrap().kind, 397, "the base hardened to 397");
        assert!(w.tile(512, 111).unwrap().active());
        // The fall column between the origin and the base is sand.
        assert_eq!(w.tile(512, 105).unwrap().kind, 53, "the fall column is sand");
        // The origin tile keeps its sand kind (the C# does not clear it).
        assert_eq!(w.tile(512, 100).unwrap().kind, 53, "the origin stays sand");
    }

    /// The poo-668 scatter (`:22379-22422`) is deterministic (a == b) and
    /// converts Dirt-set tiles to the Dirtiest Block 668.
    #[test]
    fn r6_the_poo_scatter_is_deterministic() {
        let build = || {
            let mut w = World::new("fc".into(), 512, 512);
            // Fill the scatter band with dirt so the picks always land.
            for y in 50..312 {
                for x in 50..462 {
                    let i = w.index(x, y);
                    w.tiles[i] = Tile { kind: 0, flags: FLAG_ACTIVE, ..Default::default() };
                }
            }
            w
        };
        let mut a = build();
        let mut b = build();
        poo_scatter(&mut a, &mut Rng(42), &opts());
        poo_scatter(&mut b, &mut Rng(42), &opts());
        assert_eq!(a.tiles, b.tiles, "a == b");
        let poo = a.tiles.iter().filter(|t| t.active() && t.kind == TILE_DIRTIEST).count();
        // 3 successful picks; 668 is itself in Sets.Dirt, so a re-pick of a
        // converted cell counts again — the distinct count is 1..=3.
        assert!((1..=3).contains(&poo), "the poo blocks landed: {poo}");
    }

    /// The wall-hole fill (`:9886-10053`): a wall-0 pocket enclosed by walls
    /// fills with the majority wall; the 150-cap and the single-cell veto hold.
    #[test]
    fn r6_the_wall_hole_fill_uses_the_majority_wall() {
        let mut w = World::new("fc".into(), 64, 64);
        // A 3x3 wall-0 pocket at (10..12, 10..12) ringed by wall 4.
        for y in 9..=13 {
            for x in 9..=13 {
                let i = w.index(x, y);
                let ring = x == 9 || x == 13 || y == 9 || y == 13;
                w.tiles[i] = Tile { wall: if ring { 4 } else { 0 }, ..Default::default() };
            }
        }
        let filled = fill_wall_holes_in_spot(&mut w, 11, 11, 150);
        assert!(filled, "the pocket filled");
        assert_eq!(w.tile(11, 11).unwrap().wall, 4, "the majority wall 4 filled the hole");
    }

    /// The whole pass is deterministic (a == b) on a classic world.
    #[test]
    fn r6_the_pass_is_deterministic() {
        let build = || {
            let mut w = World::new("fc".into(), 512, 512);
            let i = w.index(256, 100);
            w.tiles[i] = Tile { kind: 53, flags: FLAG_ACTIVE, ..Default::default() };
            let j = w.index(200, 200);
            w.tiles[j] = Tile { kind: 0, flags: FLAG_ACTIVE, ..Default::default() };
            w
        };
        let mut a = build();
        let mut b = build();
        let mut gv_a = gv_with_layers(120.0, 280.0);
        let mut gv_b = gv_with_layers(120.0, 280.0);
        let mut sa = Streams::new(42);
        let mut sb = Streams::new(42);
        final_cleanup(&mut a, &mut gv_a, &mut sa, &opts());
        final_cleanup(&mut b, &mut gv_b, &mut sb, &opts());
        assert_eq!(a.tiles, b.tiles, "a == b");
    }

    // --- W4-d106 retire witnesses (written NOT run — code-only branch) -------

    /// research-88 §6.4 stream note (FIXED): the ShimmerCleanUp ellipse
    /// jitter CONTINUES the scatter's `pass_rng("Final Cleanup")` sequence —
    /// the C# genRand is ONE per-pass instance. The witness replays both
    /// candidate streams over the full 241×241 disc loop, picks the first
    /// cell where the two ellipse gates DIFFER (below the dirt band, off
    /// the fill-walk row), plants a wall-2 marker there, and asserts the
    /// pass zeroed it iff the CONTINUED gate is inside. A fresh
    /// `pass_rng("ShimmerCleanUp")` (the pre-fix form) replays the scatter's
    /// draws and fails. A shimmer position is REQUIRED — on a classic world
    /// the None guard returns before any draw and the trap cannot bite.
    #[test]
    fn r6_the_shimmer_cleanup_continues_the_scatter_stream() {
        let (sx, sy) = (256i32, 350i32);
        // Replay one stream's ellipse gates over the whole disc loop,
        // recording (cell, inside) — 2 NextDouble per cell, C# order.
        let gates = |rng: &mut Rng| -> Vec<((i32, i32), bool)> {
            let mut out = Vec::new();
            for i in (sy - 120)..=(sy + 120) {
                for j in (sx - 120)..=(sx + 120) {
                    let dx = (j - sx).abs() as f64;
                    let dy = (i - sy).abs() as f64;
                    let stretch = if (i as f64) > sy as f64 { 1.2 } else { 1.4 };
                    let jx = dx * (1.0 + rng.next_f64() * 0.02);
                    let jy = dy * stretch * (1.0 + rng.next_f64() * 0.02);
                    out.push(((j, i), ((jx * jx + jy * jy).sqrt() as i32) < 90));
                }
            }
            out
        };
        let streams = Streams::new(42);
        let mut continued = streams.pass_rng("Final Cleanup");
        // The all-dirt scatter band makes the scatter draw exactly 3×2
        // range rolls (3 successful picks, no retries).
        for _ in 0..3 {
            continued.range(50, 462);
            continued.range(50, 312);
        }
        let cont = gates(&mut continued);
        let fresh = gates(&mut streams.pass_rng("Final Cleanup"));
        // The marker cell: gates differ, below the dirt band (y > 311), off
        // the fill-walk row (350) and away from the fill-walk stone seats.
        let pick = cont
            .iter()
            .zip(fresh.iter())
            .find(|((c, a), (f, b))| {
                c == f && a != b && c.1 > 320 && c.1 != 350
            })
            .map(|((c, a), _)| (*c, *a));
        let ((mx, my), in_continued) =
            pick.expect("seed 42 separates the streams somewhere in the lower disc");
        let mut w = World::new("fc".into(), 512, 512);
        for y in 50..312 {
            for x in 50..462 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 0, flags: FLAG_ACTIVE, ..Default::default() };
            }
        }
        // The shimmer-fill walk (`:34764-34788`) terminates on active seats:
        // stone left/right of the centre on row 350 and below it.
        for (x, y) in [(200, 350), (300, 350), (256, 351)] {
            let i = w.index(x, y);
            w.tiles[i] = Tile { kind: 1, flags: FLAG_ACTIVE, ..Default::default() };
        }
        let m = w.index(mx, my);
        w.tiles[m].wall = 2;
        let mut gv = gv_with_layers(120.0, 280.0);
        gv.shimmer_position = Some((sx, sy));
        let mut streams = Streams::new(42);
        final_cleanup(&mut w, &mut gv, &mut streams, &opts());
        assert_eq!(
            w.tile(mx, my).unwrap().wall == 0,
            in_continued,
            "the cleanse ellipse at ({mx},{my}) followed the CONTINUED scatter stream"
        );
    }

    /// The final LiquidCheck merge sweep (`:22473-22483`) visits y-OUTER /
    /// x-INNER: a lava cell with a water neighbour to its LEFT merges to
    /// obsidian 56 only when the left cell was already zeroed by the earlier
    /// visit in the same row — a x-outer sweep would see the water still
    /// there at the lava cell's own check and produce the same merge, but
    /// the y-order matters across rows: the fixture pins the sweep result
    /// against a manual y-outer/x-inner liquid_check replay.
    #[test]
    fn r6_the_merge_sweep_matches_the_y_outer_replay() {
        let build = || {
            let mut w = World::new("fc".into(), 512, 512);
            // Two adjacent liquid rows below the scatter band: water at
            // (250, 400), lava at (251, 400); honey at (250, 401) under the
            // water. 30+30 >= 24 so the side arm merges.
            let a = w.index(250, 400);
            w.tiles[a].liquid = 30;
            w.tiles[a].set_liquid_kind(LiquidKind::Water);
            let b = w.index(251, 400);
            w.tiles[b].liquid = 30;
            w.tiles[b].set_liquid_kind(LiquidKind::Lava);
            let c = w.index(250, 401);
            w.tiles[c].liquid = 30;
            w.tiles[c].set_liquid_kind(LiquidKind::Honey);
            w
        };
        let mut swept = build();
        let mut gv = gv_with_layers(120.0, 280.0);
        let mut streams = Streams::new(42);
        final_cleanup(&mut swept, &mut gv, &mut streams, &opts());
        // The manual replay over the same fixture (the sweep's own body).
        let mut replay = build();
        let mut sys = LiquidSystem::new(&replay);
        sys.set_generating(true);
        for y in 0..replay.height {
            for x in 0..replay.width {
                if let Some(t) = replay.tile(x, y).copied() {
                    if t.liquid > 0 {
                        sys.liquid_check(&mut replay, x, y, liquid_code(t.liquid_kind()), true);
                    }
                }
            }
        }
        assert_eq!(swept.tiles, replay.tiles, "the sweep == the y-outer/x-inner replay");
    }
}
