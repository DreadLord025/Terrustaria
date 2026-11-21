//! The classic cleanup passes — W4-d106 lane 6:
//!
//! - [`tile_cleanup`] — `Tile Cleanup` (`WorldGen.cs:21520-21942`): the
//!   whole-world slope clear, the 40-margin tidy sweep (the half-brick
//!   conversions, the water-death kills, the drip placement under shelves
//!   and over pools, the wall-13/14/87 liquid strips) and the multi-tile
//!   object frame normalisation (the dart trap, the fragile ice, the shadow
//!   orb, the heart, the mana crystal, the chests, the pots, the demon
//!   altar, the Lihzahrd altar over spikes).
//! - (R5 `Remove Broken Traps` and R6 `Final Cleanup` land here next.)
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

use crate::edit::kill_tile;
use crate::seeds::{MicroSeedFlags, SeedFlags};
use crate::tile::{FLAG_ACTIVE, FLAG_HALF_BRICK, LiquidKind};
use crate::tiledata as td;
use crate::world::World;
use super::noise::Rng;
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
    // The C#'s two-list swap BFS (`t`/`t2`), each bounded by 2000.
    let mut t: Vec<(i32, i32)> = Vec::new();
    let mut t2: Vec<(i32, i32)> = vec![start];
    let mut list: Vec<(i32, i32)> = Vec::new();
    let mut has_trap = false;
    let mut has_trigger = false;
    let mut num = 2000;
    while !t2.is_empty() {
        num -= 1;
        if num <= 0 {
            break;
        }
        std::mem::swap(&mut t, &mut t2);
        t2.clear();
        let mut num2 = 2000;
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
}
