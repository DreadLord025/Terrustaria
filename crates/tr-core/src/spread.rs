//! The biome spread — `WorldGen.hardUpdateWorld`'s infection arms
//! (`WorldGen.cs:69645-69860`) and `Convert` (`:55085-55673`).
//!
//! Source map (line numbers into `decompiled/Terraria/`):
//! - `TileID.Sets.SpreadsCorruption` = {23, 661, 25, 112, 398, 400, 163, 32,
//!   636, 24} (`TileID.cs:580`), `SpreadsCrimson` = {199, 662, 203, 234,
//!   399, 401, 200, 352, 205, 201} (`:589`)
//! - `Convert`'s tile map (`:55139-55205`): the Conversion.Stone set → the
//!   crimstone 203 / the ebonstone 25, the jungle grass → 662/23, the grass
//!   → 199/23, the ice → 200/163, the sand → 234/112, the hardened sand →
//!   399/398, the sandstone → 401/400; the walls: the stone → 192/3, the
//!   dirt → 193/69 (the natural-wall slice of `WallID.Sets.Conversion`)
//! - the spread loop (`:69696-69860`): the ±3 random target, the
//!   chlorophyte defence (`nearbyChlorophyte` `:69502+` →
//!   `ChlorophyteDefense`), the sunflower (27) suppression
//!   (`CountNearBlocksTypes(i, j, 2, 1, 27)`), the conversions with the
//!   `Next(2) == 0` re-roll chain
//! - the chlorophyte arms (`:69646-69685`): the 211/346 sources grow into
//!   the mud/the jungle grass via `CanChlorophyteGrow` (`:69464-69500` —
//!   the density gates 40 in the radius-35 box / 130 in the radius-85 box,
//!   halved above the rock layer) and the ±6 random re-scan (`Convert(...,
//!   9)`)
//!
//! Deviations (ANALYSIS "Biome spread (M5-SPREAD-2)"): the wall conversion
//! covers the natural-wall slice (1/2 → 3/69 or 192/193) — the C#'s full
//! `WallID.Sets.Conversion` tables span the brick walls too; the thorn
//! bushes (32/352) spread by the set membership but grow no bushes (the
//! plant placement = the follow-up); the post-PlantBoss rate halves
//! (`Next(2) != 0`) verbatim; the creative spread-stop rides the
//! `allowed_to_spread` input.

use crate::UnifiedRandom;
use crate::world::World;

/// `TileID.Sets.SpreadsCorruption` (`TileID.cs:580`).
pub const SPREADS_CORRUPTION: [u16; 10] = [23, 661, 25, 112, 398, 400, 163, 32, 636, 24];
/// `TileID.Sets.SpreadsCrimson` (`TileID.cs:589`).
pub const SPREADS_CRIMSON: [u16; 10] = [199, 662, 203, 234, 399, 401, 200, 352, 205, 201];

/// `Convert`'s tile map (`:55139-55205`): the source kind → the converted
/// kind per the evil flavour.
pub(crate) fn convert_tile(kind: u16, corruption: bool) -> Option<u16> {
    Some(match (kind, corruption) {
        // The grass family (the Conversion.Grass set: 2, 477, 60, 69...):
        // the corrupt grass 23 / the crimson grass 199.
        (2, c) => {
            if c {
                23
            } else {
                199
            }
        }
        // The jungle grass (the Conversion.JungleGrass set): 23/662.
        (60, c) => {
            if c {
                23
            } else {
                662
            }
        }
        // The stone + the moss family (the Conversion.Stone set): the
        // ebonstone 25 / the crimstone 203.
        (1, c) => {
            if c {
                25
            } else {
                203
            }
        }
        // The ice (the Conversion.Ice set): the purple 163 / the red 200.
        (161, c) => {
            if c {
                163
            } else {
                200
            }
        }
        // The sand: the ebonsand 112 / the crimsand 234.
        (53, c) => {
            if c {
                112
            } else {
                234
            }
        }
        // The hardened sand: 398/399.
        (397, c) => {
            if c {
                398
            } else {
                399
            }
        }
        // The sandstone: 400/401.
        (396, c) => {
            if c {
                400
            } else {
                401
            }
        }
        _ => return None,
    })
}

/// The natural-wall slice of `WallID.Sets.Conversion` (`:55090-55135`): the
/// stone wall 1 → 192/3, the dirt wall 2 → 193/69.
/// M5-SPREAD-2c: the natural-wall slice is the C#'s full wall chains, so the
/// fn routes through `growth::convert_wall_by_case` (every conversion type's
/// `Convert_ActuallyConvertWall` ladder, `WorldGen.cs:55100+`). The bool
/// keeps the two evil types (1 corruption / 4 crimson) the evil callers use.
pub(crate) fn convert_wall(wall: u16, corruption: bool) -> Option<u16> {
    crate::growth::convert_wall_by_case(wall, if corruption { 1 } else { 4 }, 0, 0.0, None)
}

/// `Convert`'s single-tile overload (`:55085-55673`). `chlorophyte` = the
/// conversion types 8/9 (`:55665-55673`: the mud 59 and the jungle grass 60
/// become the chlorophyte 211).
pub fn convert_cell(world: &mut World, x: i32, y: i32, corruption: bool, chlorophyte: bool) {
    let (kind, wall) = match world.tile(x, y) {
        Some(t) => (t.kind, t.wall),
        None => return,
    };
    if !world.tile(x, y).is_some_and(|t| t.active()) && wall == 0 {
        return;
    }
    if chlorophyte {
        if matches!(kind, 59 | 60) {
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = 211;
            }
        }
        return;
    }
    if let Some(new_kind) = convert_tile(kind, corruption) {
        if let Some(t) = world.tile_mut(x, y) {
            t.kind = new_kind;
            t.frame_x = 0;
            t.frame_y = 0;
        }
    }
    if let Some(new_wall) = convert_wall(wall, corruption) {
        if let Some(t) = world.tile_mut(x, y) {
            t.wall = new_wall;
        }
    }
}

/// `CanChlorophyteGrow` (`:69464-69500`): the density gates — the
/// chlorophyte count in the radius-35 box ≤ 40 (the box ×1.5 and the count
/// /2 above the rock layer) and ≤ 130 in the radius-85 box.
/// The three conversion flavors the conversion tables key on — the C#'s
/// `WorldGen.Convert`'s conversion types (1 = corruption, 4 = crimson,
/// 2 = hallow).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConversionFlavor {
    Corruption,
    Crimson,
    Hallow,
}

/// `WorldGen.Convert(i, j, conversionType, tiles: true, walls: false)`'s
/// single-cell arm (`:55085-55140`): the flavor's tile/wall maps over the
/// cell. The SpreadGrass arms call it with `walls: false`.
pub fn convert_cell_flavor(world: &mut World, x: i32, y: i32, flavor: ConversionFlavor, walls: bool) {
    let (kind, wall) = match world.tile(x, y) {
        Some(t) => (t.kind, t.wall),
        None => return,
    };
    if !world.tile(x, y).is_some_and(|t| t.active()) && wall == 0 {
        return;
    }
    match flavor {
        ConversionFlavor::Corruption | ConversionFlavor::Crimson => {
            let corruption = flavor == ConversionFlavor::Corruption;
            if let Some(new_kind) = convert_tile(kind, corruption) {
                if let Some(t) = world.tile_mut(x, y) {
                    t.kind = new_kind;
                    t.frame_x = 0;
                    t.frame_y = 0;
                }
            }
            if walls {
                if let Some(new_wall) = convert_wall(wall, corruption) {
                    if let Some(t) = world.tile_mut(x, y) {
                        t.wall = new_wall;
                    }
                }
            }
        }
        ConversionFlavor::Hallow => {
            if let Some(new_kind) = convert_tile_hallow(kind) {
                if let Some(t) = world.tile_mut(x, y) {
                    t.kind = new_kind;
                    t.frame_x = 0;
                    t.frame_y = 0;
                }
            }
            if walls {
                if let Some(new_wall) = convert_wall_hallow(wall) {
                    if let Some(t) = world.tile_mut(x, y) {
                        t.wall = new_wall;
                    }
                }
            }
        }
    }
}

pub fn can_chlorophyte_grow(world: &World, x: i32, y: i32) -> bool {
    let (mut r_small, mut cap_big) = (35i32, 130i32);
    if (y as f64) < world.rock_layer {
        r_small = (r_small as f64 * 1.5) as i32;
        cap_big /= 2;
    }
    let count = |r: i32| -> i32 {
        let mut n = 0;
        for k in (x - r)..(x + r) {
            for l in (y - r)..(y + r) {
                if world
                    .tile(k, l)
                    .is_some_and(|t| t.active() && t.kind == 211)
                {
                    n += 1;
                }
            }
        }
        n
    };
    if count(r_small) > 40 {
        return false;
    }
    count(85) <= cap_big
}

/// `nearbyChlorophyte` (`:69502+`): the chlorophyte within 5 tiles (the
/// defence radius).
fn nearby_chlorophyte(world: &World, x: i32, y: i32) -> bool {
    for dx in -5..=5 {
        for dy in -5..=5 {
            if world
                .tile(x + dx, y + dy)
                .is_some_and(|t| t.active() && t.kind == 211)
            {
                return true;
            }
        }
    }
    false
}

/// `CountNearBlocksTypes(i, j, 2, 1, 27)`: the sunflower (27) suppression
/// within 2 tiles.
fn sunflower_nearby(world: &World, x: i32, y: i32) -> bool {
    for dx in -2..=2 {
        for dy in -2..=2 {
            if world
                .tile(x + dx, y + dy)
                .is_some_and(|t| t.active() && t.kind == 27)
            {
                return true;
            }
        }
    }
    false
}

/// `hardUpdateWorld`'s spread arms (`:69645-69860`): the chlorophyte growth
/// (the 211/346 sources) and the evil/crimson infection spread. `x`/`y` are
/// the random picks the caller drew (the same `UpdateWorld` underground
/// picks the crystal shards ride).
pub fn hard_update_world(
    world: &mut World,
    x: i32,
    y: i32,
    downed_plant_boss: bool,
    allowed_to_spread: bool,
    rng: &mut UnifiedRandom,
) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    if !t.active() {
        return;
    }
    let kind = t.kind;
    // `:69604-69644`: the crystal-shard arm runs FIRST in `hardUpdateWorld`
    // (the shard stones below the rock layer, their own gates inside).
    crate::growth::grow_crystal_shards(world, x, y, rng);
    // `:69646-69685`: the chlorophyte arms (the 211 chlorophyte + the 346
    // chlorophyte jungle-grass source).
    if matches!(kind, 211 | 346) {
        // `Next(3) != 0`: the 4-dir neighbour growth into the mud/the jungle
        // grass, gated by `CanChlorophyteGrow`.
        if rng.next_max(3) != 0 {
            let (dx, dy) = match rng.next_max(4) {
                0 => (1, 0),
                1 => (-1, 0),
                2 => (0, 1),
                _ => (0, -1),
            };
            let (nx, ny) = (x + dx, y + dy);
            if world
                .tile(nx, ny)
                .is_some_and(|t| t.active() && matches!(t.kind, 59 | 60))
                && can_chlorophyte_grow(world, nx, ny)
            {
                convert_cell(world, nx, ny, false, true);
            }
        }
        // The ±6 random re-scan (`Convert(..., 9)` — the mud/jungle cells
        // near the source convert too).
        let nx = x + (rng.next_max(13) as i32 - 6);
        let ny = y + (rng.next_max(13) as i32 - 6);
        if world
            .tile(nx, ny)
            .is_some_and(|t| t.active() && matches!(t.kind, 59 | 60))
            && can_chlorophyte_grow(world, nx, ny)
        {
            convert_cell(world, nx, ny, false, true);
        }
        return;
    }
    // `:69688-69691`: the post-PlantBoss worlds halve the spread; the
    // creative spread-stop gate.
    if downed_plant_boss && rng.next_max(2) != 0 {
        return;
    }
    if !allowed_to_spread {
        return;
    }
    let corruption = SPREADS_CORRUPTION.contains(&kind);
    let crimson = SPREADS_CRIMSON.contains(&kind);
    if !corruption && !crimson {
        return;
    }
    // The spread loop (`:69696-69860`): the ±3 target, the chlorophyte
    // defence, the sunflower suppression, the conversions with the
    // `Next(2) == 0` re-roll chain.
    let mut tx = x + (rng.next_max(7) as i32 - 3);
    let mut ty = y + (rng.next_max(7) as i32 - 3);
    loop {
        if !(10..world.width - 10).contains(&tx) || !(10..world.height - 10).contains(&ty) {
            break;
        }
        if nearby_chlorophyte(world, tx, ty) {
            // `ChlorophyteDefense` (`:69456-69460`): the conversion runs in
            // the remix worlds only — the normal worlds just break the
            // spread loop here.
            break;
        }
        if sunflower_nearby(world, tx, ty) {
            break;
        }
        let convertible = world.tile(tx, ty).is_some_and(|t| {
            matches!(t.kind, 2 | 477 | 1 | 60 | 69 | 161 | 53 | 396 | 397)
        });
        if !convertible {
            break;
        }
        let reroll = rng.next_max(2) == 0;
        convert_cell(world, tx, ty, corruption, false);
        if !reroll {
            break;
        }
        tx += rng.next_max(7) as i32 - 3;
        ty += rng.next_max(7) as i32 - 3;
    }
}

/// `GERunner`'s hallow (good) map (`WorldGen.cs:76169-76290`): the hallowed
/// grass 109, the pearlstone 117, the hallowed sand 116, the hallowed ice
/// 164, the hallowed sandstone 403, the hallowed hardened sand 402 — and
/// the purify arm (the corrupt/crimson jungle grass 661/662 → the jungle
/// grass 60).
pub(crate) fn convert_tile_hallow(kind: u16) -> Option<u16> {
    Some(match kind {
        2 => 109,
        1 | 25 | 203 => 117,
        53 | 123 | 112 | 234 => 116,
        161 | 163 | 200 => 164,
        396 => 403,
        397 => 402,
        23 | 199 => 109,
        661 | 662 => 60,
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// `WorldGen.Convert` — the spray/cleanse entry (S1-D-4 unarmed styles, the
// aiStyle-31 Clentaminator arm's `WorldGen.Convert(point.X, point.Y, 0, size)`
// call, `Projectile.cs:26834`). Only conversion type 0 (the pure spray 145's
// `num245 = 0` default arm, `:26793-26795`) is ported here — the other rowed
// spray kinds (146-149, 1015-1017) have no `proj_data` rows yet.

/// `TileID.Sets.Conversion.Grass` (`TileID.cs:86`).
const CONV_GRASS: [u16; 6] = [2, 23, 199, 109, 477, 492];
/// `TileID.Sets.Conversion.JungleGrass` (`TileID.cs:84`).
const CONV_JUNGLE_GRASS: [u16; 3] = [60, 661, 662];
/// `TileID.Sets.Conversion.MushroomGrass` (`TileID.cs:85`).
const CONV_MUSHROOM_GRASS: [u16; 1] = [70];
/// `TileID.Sets.Conversion.Snow` (`TileID.cs:89`).
const CONV_SNOW: [u16; 1] = [147];
/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9695`).
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `WorldGen.TileIsExposedToAir(x, y)` (`WorldGen.cs:54782-54800`, W4-f25
/// helper C1): the 3x3 box around the cell holds an inactive tile, a
/// non-solid tile or a platform — any of the three reads "exposed to air".
/// The `InWorld(2)` gate refuses the border band. UNWIRED (the Convert
/// 7/15/18/19 dirt arms `:55512`/`:56031`/`:56171`/`:56222` and the c32
/// biome-dungeon rooms are its consumers; no executed step calls it).
pub fn tile_is_exposed_to_air(world: &World, x: i32, y: i32) -> bool {
    // `:54784-54787`.
    if !in_world(world, x, y, 2) {
        return false;
    }
    // `:54788-54798`: `tile != null && (!tile.active() ||
    // !Main.tileSolid[tile.type] || TileID.Sets.Platforms[tile.type])`.
    for i in (x - 1)..=(x + 1) {
        for j in (y - 1)..=(y + 1) {
            if let Some(t) = world.tile(i, j) {
                if !t.active()
                    || !crate::tiledata::TILE_SOLID.has(t.kind)
                    || crate::tiledata::SET_PLATFORMS.has(t.kind)
                {
                    return true;
                }
            }
        }
    }
    false
}

/// `WorldGen.BlockBelowMakesSandConvertIntoHardenedSand(i, j)`
/// (`WorldGen.cs:86562-86583`, W4-f25 helper C2): the cell BELOW is inactive
/// or a non-solid kind — sand converting over such a floor hardens (397)
/// instead of staying loose (53). The bottom row refuses (no cell below).
/// UNWIRED (the Convert 5/16 sand arms `:55386`/`:56064` and
/// `BiomeDungeonRoom.cs:264/:280` are its consumers).
pub fn block_below_makes_sand_convert_into_hardened_sand(
    world: &World,
    i: i32,
    j: i32,
) -> bool {
    // `:86565-86568`: `j >= Main.maxTilesY - 1` → false.
    if j >= world.height - 1 {
        return false;
    }
    // `:86569-86573`: the null tile → false.
    let Some(tile) = world.tile(i, j + 1) else {
        return false;
    };
    // `:86574-86581`: `!tile.nactive()` → true; else the in-range non-solid
    // kind → true (the generated TILE_SOLID set carries the range gate).
    !tile.active() || !crate::tiledata::TILE_SOLID.has(tile.kind)
}

/// `WorldGen.Convert(i, j, conversionType, size)` (`WorldGen.cs:55067-55083`):
/// the square walk with the Manhattan `< 6` gate. The port carries the type-0
/// (cleanse) cell only — the spray's conversion type.
pub fn convert_cleanse(world: &mut World, i: i32, j: i32, size: i32) {
    for k in (i - size)..=(i + size) {
        for l in (j - size)..=(j + size) {
            if in_world(world, k, l, 1) && (k - i).abs() + (l - j).abs() < 6 {
                convert_cleanse_cell(world, k, l);
            }
        }
    }
}

/// `Convert`'s type-0 arm (`WorldGen.cs:55527-55664`): the wall chain, then
/// the torch/grass/stone/sand/ice tile ladder, the mushroom grass and the
/// thorn kill. `Convert_ActuallyConvertTile`/`Wall` (`:56312-56343`) write
/// only when the cell differs, and re-frame through `SquareTileFrame` /
/// `SquareWallFrame` — the port's [`crate::framing::reframe_area`] superset
/// (its documented deviation).
///
/// Deviations (ANALYSIS "The unarmed projectile styles (S1-D-4)"): the wall
/// 69/70/81 surface roll and the wall-80 depth rolls draw `WorldGen.genRand`
/// (`:55535`, `:55584`) — the port has no runtime genRand stream (the
/// `edit.rs` precedent), so the surface arm takes the no-rng 64 (the shared
/// `convert_wall_by_case(wall, 0, .., None)` behaviour) and the wall-80 arm
/// compares against the unrolled band edges. The tree cross-conversion
/// (`ConvertTreeAndGround`, `:54902+`) is unported — an invalid tree above a
/// converted cell is killed, never re-typed.
fn convert_cleanse_cell(world: &mut World, x: i32, y: i32) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    let (kind, wall, active) = (t.kind, t.wall, t.active());
    if !active && wall == 0 {
        return;
    }
    // The wall half (`:55528-55611`). The 80 arm (`:55582-55592`) sits in the
    // C#'s default chain between the NewWall ladder and the HardenedSand arm;
    // the shared `convert_wall_by_case` has no 80 row, so it is handled here
    // first (the two arms are wall-disjoint). The switch's no-op cases
    // 61/185/204-207 (`:55602-55608`) break WITHOUT converting — the shared
    // ladder's Stone/NewWall sets would otherwise catch them, so they are
    // gated out here.
    let new_wall = if matches!(wall, 61 | 185 | 204 | 205 | 206 | 207) {
        None
    } else if wall == 80 {
        // `j2 < worldSurface + 4 + Next(3) || j2 > (maxTilesY + rockLayer)/2
        // - 3 + Next(3)` — the unrolled band edges (the genRand skip).
        let mid = (world.height as f64 + world.rock_layer) / 2.0 - 3.0;
        Some(if (y as f64) < world.world_surface + 4.0 || (y as f64) > mid {
            15u16
        } else {
            64
        })
    } else {
        crate::growth::convert_wall_by_case(wall, 0, y, world.world_surface, None)
    };
    if let Some(new_wall) = new_wall {
        // `Convert_ActuallyConvertWall` (`:56329-56343`): non-zero, different.
        if wall != 0 && wall != new_wall {
            if let Some(t) = world.tile_mut(x, y) {
                t.wall = new_wall;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
    }
    // The tile half (`:55616-55663`) — the else-if ladder in source order.
    let torch_style = if active && kind == 4 {
        // `TileID.Sets.Conversion.Torch(tile, 0)` (`TileID.cs:51-56`): the
        // corrupt/crimson/hallow rows 18-20 and the mushroom row 22.
        let num = world.tile(x, y).map_or(0, |t| t.frame_y as i32 / 22);
        if (18..=20).contains(&num) || num == 22 {
            // `Convert_ActuallyConvertTorch` (`:56287-56310`): the type-0
            // mushroom quirk — style 22 lands on 21, not 0.
            Some(if num == 22 { 21i16 } else { 0 })
        } else {
            None
        }
    } else {
        None
    };
    if let Some(style) = torch_style {
        let cur = world.tile(x, y).map_or(0, |t| t.frame_y as i32 / 22);
        if cur != style as i32 {
            if let Some(t) = world.tile_mut(x, y) {
                t.frame_y = style * 22;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
        return;
    }
    let new_kind: Option<u16> = if kind == 492 {
        Some(477)
    } else if CONV_JUNGLE_GRASS.contains(&kind) && kind != 60 {
        Some(60)
    } else if CONV_GRASS.contains(&kind) && kind != 2 && kind != 477 {
        Some(2)
    } else if crate::tiledata::SET_CONVERSION_STONE.has(kind) && kind != 1 {
        Some(1)
    } else if crate::tiledata::SET_CONVERSION_SAND.has(kind) && kind != 53 {
        Some(53)
    } else if crate::tiledata::SET_CONVERSION_HARDENED_SAND.has(kind) && kind != 397 {
        Some(397)
    } else if crate::tiledata::SET_CONVERSION_SANDSTONE.has(kind) && kind != 396 {
        Some(396)
    } else if crate::tiledata::SET_ICE.has(kind) && kind != 161 {
        // `TileID.Sets.Conversion.Ice` = {161, 163, 164, 200} (`TileID.cs:91`)
        // — the generated Ice set carries the same four convertible ids.
        Some(161)
    } else if CONV_MUSHROOM_GRASS.contains(&kind) {
        Some(60)
    } else {
        None
    };
    if let Some(new_kind) = new_kind {
        // `Convert_ActuallyConvertTile` (`:56312-56327`): active, different,
        // the tree-above validity check, then the write + re-frame.
        if active && kind != new_kind {
            try_kill_invalid_tree_above(world, x, y, new_kind);
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = new_kind;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
    } else if kind == 32 || kind == 352 {
        // `:55656-55662` — the thorn bushes die (the netMode arm is MP-only).
        let _ = crate::edit::kill_tile(world, x, y, false);
    }
}

/// `TryConvertingOrKillingTreesAboveIfTheyWouldBecomeInvalid`
/// (`WorldGen.cs:54802-54899`): the tree trunk (or palm 323) directly above
/// the converted cell dies when the new floor no longer fits its ground
/// test. The gem/vanity/ash profiles (`GrowTreeSettings.Profiles.
/// TryGetFromTreeId`, `:3890-3928`) override the vanilla test; the
/// `ConvertTreeAndGround` cross-conversion half is the documented skip.
fn try_kill_invalid_tree_above(world: &mut World, x: i32, y: i32, new_floor: u16) {
    if !in_world(world, x, y, 2) {
        return;
    }
    let Some(above) = world.tile(x, y - 1) else {
        return;
    };
    if !above.active() {
        return;
    }
    let tree = above.kind;
    let is_trunk = crate::torch::TREE_TRUNKS.contains(&tree);
    if !is_trunk && tree != 323 {
        return;
    }
    let grass_side = CONV_GRASS.contains(&new_floor)
        || CONV_JUNGLE_GRASS.contains(&new_floor)
        || CONV_SNOW.contains(&new_floor)
        || CONV_MUSHROOM_GRASS.contains(&new_floor);
    let mut fits = true;
    match tree {
        5 => fits = grass_side,
        323 => fits = crate::tiledata::SET_CONVERSION_SAND.has(new_floor),
        _ => {}
    }
    // The profile override (`:54840-54843`): gem trees 583-589 take
    // `GemTreeGroundTest` (Stone | Moss, `:30304-30315`), the vanity trees
    // 596/616 `VanityTreeGroundTest` (Grass minus 23/199, `:30317-30328`),
    // the ash tree 634 `AshTreeGroundTest` (633, `:30330-30341`).
    if (583..=589).contains(&tree) {
        fits = crate::tiledata::SET_CONVERSION_STONE.has(new_floor)
            || crate::tiledata::SET_CONVERSION_MOSS.has(new_floor);
    } else if tree == 596 || tree == 616 {
        fits = CONV_GRASS.contains(&new_floor) && new_floor != 23 && new_floor != 199;
    } else if tree == 634 {
        fits = new_floor == 633;
    }
    if !fits {
        let _ = crate::edit::kill_tile(world, x, y - 1, false);
    }
}

// ---------------------------------------------------------------------------
// `WorldGen.Convert` — the hallow cell (W4-f25 helper C3, conversion type 2).
// The consumers: SpreadGrass's 109 dispatch (`:75218`), DoHallowOnSurface
// (`:1967-2011`), ConvertSkyIslands (`:24701`), initializeHardMode's hallow
// runners (`:31630-31651`), the 146 Blue Solution spray / ChlorophyteBolt
// death (`Projectile.cs:26803`/`:77635`). UNWIRED — the ported
// `convert_cell_flavor(Hallow)` / `gerunner` maps stay as they are.

/// `TileID.Sets.Conversion.Thorn` (`TileID.cs:95`). (d1's powder port carries
/// the same constant on its branch; the merger dedupes.)
const CONV_THORN: [u16; 4] = [32, 352, 69, 655];

/// `TileID.Sets.Conversion.Torch(tile, conversionType)`'s default arm
/// (`TileID.cs:63-77`): the styles every non-0/non-11 conversion type accepts.
fn torch_convertible_default(style: i32) -> bool {
    matches!(style, 0 | 9 | 16 | 18 | 19 | 20 | 21 | 22)
}

/// `WorldGen.Convert(i, j, 2, size)` (`WorldGen.cs:55067-55083` walk +
/// `:55176-55260` cell): the square walk with the Manhattan `< 6` gate, each
/// cell through [`convert_hallow_cell`]. The C# overload's `tiles`/`walls`
/// halves both run (the callers that pass `tiles: false`/`walls: false` are
/// the wall/tile sweeps — they ride the cell fn's halves directly when their
/// consumer lands).
pub fn convert_sized_hallow(world: &mut World, i: i32, j: i32, size: i32) {
    for k in (i - size)..=(i + size) {
        for l in (j - size)..=(j + size) {
            if in_world(world, k, l, 1) && (k - i).abs() + (l - j).abs() < 6 {
                convert_hallow_cell(world, k, l);
            }
        }
    }
}

/// `Convert`'s type-2 cell arm (`WorldGen.cs:55176-55260`): the wall chain
/// (the flat Grass/Stone/HardenedSand/Sandstone/NewWall ladder → 70/28/219/
/// 222/200-203, `:55179-55210`), then the torch → style 20 / moss-or-stone →
/// 117 / golf grass → 492 / grass → 109 / ice → 164 / sand → 116 / hardened
/// sand → 402 / sandstone → 403 / thorn-kill ladder in source order, then the
/// INDEPENDENT mud arm (`:55256-55259`): a mud 59 cell with a hallow-grass
/// 109 4-neighbour converts to dirt 0.
/// `Convert_ActuallyConvertTile`/`Wall`/`Torch` (`:56287-56343`) write only
/// when the cell differs and re-frame — the port's `reframe_area` superset
/// (the `convert_cleanse_cell` deviation). The genRand-free wall ladder takes
/// `None` (type 2 draws nothing); the `RainbowSandAndBlackSandWalls` paint
/// arms are the d1-documented skip.
pub fn convert_hallow_cell(world: &mut World, x: i32, y: i32) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    let (kind, wall, active) = (t.kind, t.wall, t.active());
    // `:55087-55097` — the empty cell converts nothing.
    if !active && wall == 0 {
        return;
    }
    // The wall half (`:55177-55211`): the shared type-2 flat ladder.
    if let Some(new_wall) =
        crate::growth::convert_wall_by_case(wall, 2, y, world.world_surface, None)
    {
        // `Convert_ActuallyConvertWall` (`:56329-56343`): non-zero, different.
        if wall != 0 && wall != new_wall {
            if let Some(t) = world.tile_mut(x, y) {
                t.wall = new_wall;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
    }
    // The tile half (`:55216-55255`) — the else-if ladder in source order.
    if active && kind == 4 && torch_convertible_default(t.frame_y as i32 / 22) {
        // `Convert_ActuallyConvertTorch(.., 20)` (`:55218`).
        let cur = t.frame_y as i32 / 22;
        if cur != 20 {
            if let Some(t) = world.tile_mut(x, y) {
                t.frame_y = 20 * 22;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
    } else {
        // `Main.tileMoss[type] || Conversion.Stone[type]` (`:55220`) — the
        // generated TILE_MOSS set is Main.cs:7007-7047's eleven ids.
        let new_kind: Option<u16> = if (crate::tiledata::TILE_MOSS.has(kind)
            || crate::tiledata::SET_CONVERSION_STONE.has(kind))
            && kind != 117
        {
            Some(117)
        } else if crate::tiledata::SET_GOLF_GRASS.has(kind) && kind != 492 {
            Some(492)
        } else if CONV_GRASS.contains(&kind) && kind != 109 && kind != 492 {
            Some(109)
        } else if crate::tiledata::SET_ICE.has(kind) && kind != 164 {
            Some(164)
        } else if crate::tiledata::SET_CONVERSION_SAND.has(kind) && kind != 116 {
            Some(116)
        } else if crate::tiledata::SET_CONVERSION_HARDENED_SAND.has(kind) && kind != 402 {
            Some(402)
        } else if crate::tiledata::SET_CONVERSION_SANDSTONE.has(kind) && kind != 403 {
            Some(403)
        } else {
            None
        };
        if let Some(new_kind) = new_kind {
            // `Convert_ActuallyConvertTile` (`:56312-56327`).
            if active && kind != new_kind {
                try_kill_invalid_tree_above(world, x, y, new_kind);
                if let Some(t) = world.tile_mut(x, y) {
                    t.kind = new_kind;
                }
                crate::framing::reframe_area(world, x, y, 1, false);
            }
        } else if CONV_THORN.contains(&kind) {
            // `:55248-55255` — the thorn bushes die (the netMode arm is MP).
            let _ = crate::edit::kill_tile(world, x, y, false);
        }
    }
    // The independent mud arm (`:55256-55259`): the SNAPSHOT type 59 with a
    // live 109 4-neighbour → dirt 0 through the same actually-convert shape.
    if kind == 59
        && [
            world.tile(x - 1, y),
            world.tile(x + 1, y),
            world.tile(x, y - 1),
            world.tile(x, y + 1),
        ]
        .iter()
        .any(|t| t.is_some_and(|t| t.kind == 109))
    {
        if active && kind != 0 {
            try_kill_invalid_tree_above(world, x, y, 0);
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = 0;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
    }
}

// ---------------------------------------------------------------------------
// `WorldGen.Convert` — the spray cells (W4-f25 helper C4, conversion types
// 5/6/7: desertify / snow / purify). The consumers: the aiStyle-31
// Clentaminator's 1015 Sand / 1016 Snow / 1017 Purple Solution sprays
// (`Projectile.cs:26813-26823`, the d4 dispatch). UNWIRED.

/// `WorldGen.Convert(i, j, conversionType, size)` for the spray types 5/6/7
/// (`WorldGen.cs:55067-55083` walk): the square walk with the Manhattan
/// `< 6` gate, each cell through [`convert_spray_cell`].
pub fn convert_sized_spray(world: &mut World, i: i32, j: i32, conversion_type: u8, size: i32) {
    if !matches!(conversion_type, 5 | 6 | 7) {
        return;
    }
    for k in (i - size)..=(i + size) {
        for l in (j - size)..=(j + size) {
            if in_world(world, k, l, 1) && (k - i).abs() + (l - j).abs() < 6 {
                convert_spray_cell(world, k, l, conversion_type);
            }
        }
    }
}

/// `Convert`'s type-5/6/7 cell arms (`WorldGen.cs:55363-55408` desertify,
/// `:55409-55445` snow, `:55446-55526` purify): the wall chain, then the
/// torch / tile ladder in source order.
///
/// Deviations (the `convert_cleanse_cell` / d1 precedent): the wall halves
/// ride the shared `convert_wall_by_case` (types 5/6 verbatim; type 7's
/// `:55465` dirt arm there carries only the Dirt set — the HardenedSand/Snow
/// union half is pre-gated here, order-equivalent: the union is disjoint from
/// the natural Stone/Ice/Sandstone arm, the 262/274 switch and the NewWall
/// sets). The `RainbowSandAndBlackSandWalls` paint arms are the documented
/// skip; the re-frame is the `reframe_area` superset.
pub fn convert_spray_cell(world: &mut World, x: i32, y: i32, conversion_type: u8) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    let (kind, wall, active) = (t.kind, t.wall, t.active());
    // `:55087-55097` — the empty cell converts nothing.
    if !active && wall == 0 {
        return;
    }
    // The wall half. Type 7's `:55465` union pre-gate (see the fn doc).
    let new_wall = if conversion_type == 7
        && (crate::growth::conversion_hardened_sand_walls_contains(wall)
            || crate::growth::conversion_snow_walls_contains(wall))
        && wall != 2
    {
        Some(2u16)
    } else {
        crate::growth::convert_wall_by_case(wall, conversion_type, y, world.world_surface, None)
    };
    if let Some(new_wall) = new_wall {
        // `Convert_ActuallyConvertWall` (`:56329-56343`): non-zero, different.
        if wall != 0 && wall != new_wall {
            if let Some(t) = world.tile_mut(x, y) {
                t.wall = new_wall;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
    }
    // The torch arm (`:55379`/`:55425`/`:55493`): the default style set,
    // styles 16 / 9 / 0 per type.
    if active && kind == 4 && torch_convertible_default(t.frame_y as i32 / 22) {
        let style = match conversion_type {
            5 => 16i16,
            6 => 9,
            _ => 0,
        };
        let cur = t.frame_y as i32 / 22;
        if cur != style as i32 {
            if let Some(t) = world.tile_mut(x, y) {
                t.frame_y = style * 22;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
        return;
    }
    // The tile ladders in source order. `Conversion.Dirt` = {0}
    // (`TileID.cs:88`), `Conversion.Snow` = {147} (`:89`).
    let dirt = kind == 0;
    let snow = CONV_SNOW.contains(&kind);
    let moss_or = |k: u16| crate::tiledata::TILE_MOSS.has(k);
    let new_kind: Option<u16> = match conversion_type {
        // Desertify (`:55383-55399`): grass/sand/snow/dirt → 53 (397 over a
        // hardening floor), hardened sand → 397, moss/stone/ice/sandstone →
        // 396.
        5 => {
            if (CONV_GRASS.contains(&kind)
                || crate::tiledata::SET_CONVERSION_SAND.has(kind)
                || snow
                || dirt)
                && kind != 53
            {
                Some(if block_below_makes_sand_convert_into_hardened_sand(world, x, y) {
                    397
                } else {
                    53
                })
            } else if crate::tiledata::SET_CONVERSION_HARDENED_SAND.has(kind) && kind != 397 {
                Some(397)
            } else if (moss_or(kind)
                || crate::tiledata::SET_CONVERSION_STONE.has(kind)
                || crate::tiledata::SET_ICE.has(kind)
                || crate::tiledata::SET_CONVERSION_SANDSTONE.has(kind))
                && kind != 396
            {
                Some(396)
            } else {
                None
            }
        }
        // Snow (`:55429-55436`): grass/sand/hardened/snow/dirt → 147,
        // moss/stone/ice/sandstone → 161.
        6 => {
            if (CONV_GRASS.contains(&kind)
                || crate::tiledata::SET_CONVERSION_SAND.has(kind)
                || crate::tiledata::SET_CONVERSION_HARDENED_SAND.has(kind)
                || snow
                || dirt)
                && kind != 147
            {
                Some(147)
            } else if (moss_or(kind)
                || crate::tiledata::SET_CONVERSION_STONE.has(kind)
                || crate::tiledata::SET_ICE.has(kind)
                || crate::tiledata::SET_CONVERSION_SANDSTONE.has(kind))
                && kind != 161
            {
                Some(161)
            } else {
                None
            }
        }
        // Purify (`:55497-55517`): stone/ice/sandstone → 1 (NO moss arm),
        // golf grass → 477, grass → 2, sand/hardened/snow/dirt → 0 (2 when
        // exposed to air).
        _ => {
            if (crate::tiledata::SET_CONVERSION_STONE.has(kind)
                || crate::tiledata::SET_ICE.has(kind)
                || crate::tiledata::SET_CONVERSION_SANDSTONE.has(kind))
                && kind != 1
            {
                Some(1)
            } else if crate::tiledata::SET_GOLF_GRASS.has(kind) && kind != 477 {
                Some(477)
            } else if CONV_GRASS.contains(&kind) && kind != 2 && kind != 477 {
                Some(2)
            } else if (crate::tiledata::SET_CONVERSION_SAND.has(kind)
                || crate::tiledata::SET_CONVERSION_HARDENED_SAND.has(kind)
                || snow
                || dirt)
                && kind != 0
            {
                Some(if tile_is_exposed_to_air(world, x, y) { 2 } else { 0 })
            } else {
                None
            }
        }
    };
    if let Some(new_kind) = new_kind {
        // `Convert_ActuallyConvertTile` (`:56312-56327`).
        if active && kind != new_kind {
            try_kill_invalid_tree_above(world, x, y, new_kind);
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = new_kind;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
    } else if CONV_THORN.contains(&kind) && kind != 69 {
        // `:55400-55407`/`:55437-55444`/`:55518-55525` — the thorn kill
        // (the 69 corrupt-thorn exemption is the C#'s own; the netMode arm
        // is MP-only).
        let _ = crate::edit::kill_tile(world, x, y, false);
    }
}

// ---------------------------------------------------------------------------
// `WorldGen.Convert` — the chlorophyte cells (W4-f25 helper C5, conversion
// types 8/9/10). The consumers: hardUpdateWorld's chlorophyte arms
// (`:69658`/`:69686` type 8, `:69697` type 9 — the ported `convert_cell(..,
// chlorophyte: true)` approximation stays, unwired) and ChlorophyteDefense's
// remix-only type 10 (`:69460`). UNWIRED.

/// `WorldGen.Convert(i, j, conversionType, size)` for the chlorophyte types
/// 8/9/10 (`WorldGen.cs:55067-55083` walk): the square walk with the
/// Manhattan `< 6` gate, each cell through [`convert_chlorophyte_cell`].
/// The C# call sites all use the single-cell overload (size 0).
pub fn convert_sized_chlorophyte(
    world: &mut World,
    i: i32,
    j: i32,
    conversion_type: u8,
    size: i32,
) {
    if !matches!(conversion_type, 8 | 9 | 10) {
        return;
    }
    for k in (i - size)..=(i + size) {
        for l in (j - size)..=(j + size) {
            if in_world(world, k, l, 1) && (k - i).abs() + (l - j).abs() < 6 {
                convert_chlorophyte_cell(world, k, l, conversion_type);
            }
        }
    }
}

/// `Convert`'s type-8/9/10 cell arms (`WorldGen.cs:55665-55670` /
/// `:55671-55720` / `:55721-55763`): tiles-only (no wall half in any of the
/// three), no torch arm.
/// - 8: the mud 59 / jungle grass 60 → the chlorophyte 211.
/// - 9: the FULL grass set (2/23/109/199/477/492/661/662) → 60, dirt 0 → 59,
///   the evil stone/sand/hardened/sandstone pairs → their pure kinds, the
///   six thorn/evil kinds (24/32/201/205/352/636) die.
/// - 10: the same WITHOUT the pure-grass/dirt rows (only the four infection
///   grasses 23/199/661/662 → 60) — the ChlorophyteDefense remix conversion.
/// `Convert_ActuallyConvertTile` (`:56312-56327`) writes only when the cell
/// differs; the re-frame is the `reframe_area` superset (the cleanse/d1
/// precedent).
pub fn convert_chlorophyte_cell(world: &mut World, x: i32, y: i32, conversion_type: u8) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    let (kind, wall, active) = (t.kind, t.wall, t.active());
    // `:55087-55097` — the empty cell converts nothing.
    if !active && wall == 0 {
        return;
    }
    // Type 8 (`:55665-55670`): `tiles && (type == 59 || type == 60)`.
    if conversion_type == 8 {
        if matches!(kind, 59 | 60) && active && kind != 211 {
            try_kill_invalid_tree_above(world, x, y, 211);
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = 211;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
        return;
    }
    // Types 9/10 (`:55676-55719` / `:55726-55762`): the switch on type.
    let new_kind: Option<u16> = match kind {
        // The type-9-only pure rows (`:55678-55690`): the plain/golf/hallow
        // grasses and the dirt.
        2 | 109 | 477 | 492 if conversion_type == 9 => Some(60),
        0 if conversion_type == 9 => Some(59),
        // The shared infection-grass row (`:55679`/`:55684-55685` vs
        // `:55728-55731`).
        23 | 199 | 661 | 662 => Some(60),
        25 | 203 => Some(1),
        112 | 234 => Some(53),
        398 | 399 => Some(397),
        400 | 401 => Some(396),
        _ => None,
    };
    if let Some(new_kind) = new_kind {
        if active && kind != new_kind {
            try_kill_invalid_tree_above(world, x, y, new_kind);
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = new_kind;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
    } else if matches!(kind, 24 | 32 | 201 | 205 | 352 | 636) {
        // `:55707-55718` / `:55750-55761` — the thorn/evil kinds die (the
        // netMode arm is MP-only).
        let _ = crate::edit::kill_tile(world, x, y, false);
    }
}

/// `GERunner`'s wall arms (`:76244-76260`): the sandstone wall 187 → the
/// hallowed sandstone wall 222; the ebonstone/crimstone walls 3/83 → the
/// hallowed stone wall 28.
pub(crate) fn convert_wall_hallow(wall: u16) -> Option<u16> {
    Some(match wall {
        187 => 222,
        3 | 83 => 28,
        _ => return None,
    })
}

/// `WorldGen.GERunner` (`WorldGen.cs:76169-76330`): the V-shaped strip
/// walker — the position starts at the sky `(i, 0)`, moves with the
/// velocity `(speedX, 5f)` per step, converting the Manhattan disc of the
/// radius `num2 · 0.5` (`num2 = Next(200, 250) · maxTilesX/4200`, the rim
/// wobble `Next(-10, 11) · 0.015`) around it. `good` runs the hallow map,
/// the evil side runs `Convert`'s map by the world's flavour.
pub fn gerunner(
    world: &mut World,
    x: i32,
    speed_x: f64,
    good: bool,
    corruption: bool,
    rng: &mut UnifiedRandom,
) {
    let scale = world.width as f64 / 4200.0;
    let num2 = (rng.next_range(200, 250) as f64) * scale;
    let mut px = x as f64;
    let mut py = 0.0;
    let mut vx = speed_x;
    let vy = 5.0;
    let bottom = world.height - 10;
    let mut steps = 0;
    while py < bottom as f64 && steps < world.height * 2 {
        steps += 1;
        let radius = num2 * 0.5;
        for m in (px - radius) as i32..=(px + radius) as i32 {
            for n in (py - radius) as i32..=(py + radius) as i32 {
                if m < 2 || n < 2 || m >= world.width - 2 || n >= world.height - 5 {
                    continue;
                }
                let manhattan = ((m as f64 - px).abs() + (n as f64 - py).abs()) as f64;
                let wobble = 1.0 + (rng.next_range(-10, 11) as f64) * 0.015;
                if manhattan >= radius * wobble {
                    continue;
                }
                let (kind, wall) = match world.tile(m, n) {
                    Some(t) => (t.kind, t.wall),
                    None => continue,
                };
                if good {
                    if let Some(new_kind) = convert_tile_hallow(kind) {
                        if let Some(t) = world.tile_mut(m, n) {
                            t.kind = new_kind;
                        }
                    }
                    if let Some(new_wall) = convert_wall_hallow(wall) {
                        if let Some(t) = world.tile_mut(m, n) {
                            t.wall = new_wall;
                        }
                    }
                } else {
                    if let Some(new_kind) = convert_tile(kind, corruption) {
                        if let Some(t) = world.tile_mut(m, n) {
                            t.kind = new_kind;
                            t.frame_x = 0;
                            t.frame_y = 0;
                        }
                    }
                    if let Some(new_wall) = convert_wall(wall, corruption) {
                        if let Some(t) = world.tile_mut(m, n) {
                            t.wall = new_wall;
                        }
                    }
                }
            }
        }
        px += vx;
        py += vy;
    }
}

/// `WorldGen.initializeHardMode`'s runner half (`WorldGen.cs:31660-31666`):
/// the hallow V on one side and the evil V on the other (the C# `GERunner(
/// num3, 0, 3·num5, 5f)` good + `GERunner(num4, 0, −3·num5, 5f, good:
/// false)`); the start xs sit ±`maxTilesX/7` off the middle.
pub fn initialize_hard_mode(world: &mut World, corruption: bool, rng: &mut UnifiedRandom) {
    let offset = world.width / 7;
    // `num5 = ±1` (`WorldGen.cs:31585-31590`) — the velocity `3·num5` gives
    // the 3:5 V slope verbatim.
    let speed = 3.0;
    // The hallow runner: the east side heading further east.
    gerunner(world, world.width / 2 + offset, speed, true, corruption, rng);
    // The evil runner: the west side heading further west.
    gerunner(world, world.width / 2 - offset, -speed, false, corruption, rng);
}

#[cfg(test)]
mod runner_tests {
    use super::*;
    use crate::tile::{FLAG_ACTIVE, Tile};

    fn world() -> World {
        let mut w = World::new("t".into(), 800, 600);
        w.rock_layer = 300.0;
        // The ground band: the grass over the stone.
        for x in 0..800 {
            for y in 300..340 {
                put(&mut w, x, y, if y == 300 { 2 } else { 1 });
            }
            for y in 340..600 {
                put(&mut w, x, y, 1);
            }
        }
        w
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    #[test]
    fn the_hallow_map_is_verbatim() {
        // `GERunner`'s good arms (`WorldGen.cs:76262-76290`): the hallowed
        // grass 109, the pearlstone 117, the hallowed sand 116, the
        // hallowed ice 164, 403, 402 — and the purify arm (661/662 → 60).
        assert_eq!(convert_tile_hallow(2), Some(109));
        assert_eq!(convert_tile_hallow(1), Some(117));
        assert_eq!(convert_tile_hallow(25), Some(117));
        assert_eq!(convert_tile_hallow(203), Some(117));
        assert_eq!(convert_tile_hallow(53), Some(116));
        assert_eq!(convert_tile_hallow(112), Some(116));
        assert_eq!(convert_tile_hallow(234), Some(116));
        assert_eq!(convert_tile_hallow(161), Some(164));
        assert_eq!(convert_tile_hallow(163), Some(164));
        assert_eq!(convert_tile_hallow(396), Some(403));
        assert_eq!(convert_tile_hallow(397), Some(402));
        assert_eq!(convert_tile_hallow(23), Some(109));
        assert_eq!(convert_tile_hallow(199), Some(109));
        assert_eq!(convert_tile_hallow(661), Some(60), "the purify arm");
        assert_eq!(convert_tile_hallow(662), Some(60));
        assert_eq!(convert_tile_hallow(211), None);
        // The walls: 187 → 222, 3/83 → 28.
        assert_eq!(convert_wall_hallow(187), Some(222));
        assert_eq!(convert_wall_hallow(3), Some(28));
        assert_eq!(convert_wall_hallow(83), Some(28));
    }

    #[test]
    fn the_gerunner_carves_the_v_strip() {
        let mut w = world();
        let mut rng = UnifiedRandom::new(9);
        gerunner(&mut w, 400, 2.0, true, false, &mut rng);
        // The strip: the hallowed grass/pearlstone along the V from the
        // sky to the bottom.
        let hallowed = w
            .tiles
            .iter()
            .filter(|t| t.active() && matches!(t.kind, 109 | 117))
            .count();
        assert!(hallowed > 300, "the hallowed tiles ({hallowed})");
        // The V spans the ground band's full depth: the hallowed tiles at
        // the top (300-340) and at the bottom (560-599) of the band.
        let top = (300..340).any(|y| {
            (0..800).any(|x| {
                w.tile(x, y)
                    .is_some_and(|t| t.active() && matches!(t.kind, 109 | 117))
            })
        });
        let bottom = (560..600).any(|y| {
            (0..800).any(|x| {
                w.tile(x, y)
                    .is_some_and(|t| t.active() && matches!(t.kind, 109 | 117))
            })
        });
        assert!(top, "the V top");
        assert!(bottom, "the V bottom");
    }

    #[test]
    fn the_evil_runner_uses_the_flavour() {
        // The evil runner with the corruption: the grass → the corrupt
        // grass 23, the stone → the ebonstone 25.
        let mut w = world();
        let mut rng = UnifiedRandom::new(10);
        gerunner(&mut w, 400, -2.0, false, true, &mut rng);
        let corrupt = w
            .tiles
            .iter()
            .filter(|t| t.active() && matches!(t.kind, 23 | 25))
            .count();
        assert!(corrupt > 300, "the corrupted tiles ({corrupt})");
    }

    #[test]
    fn initialize_hard_mode_runs_both_runners() {
        // The corruption world (the crimson world swaps the evil kinds).
        let mut w = world();
        let mut rng = UnifiedRandom::new(11);
        initialize_hard_mode(&mut w, true, &mut rng);
        let hallowed = w
            .tiles
            .iter()
            .filter(|t| t.active() && matches!(t.kind, 109 | 117 | 116 | 164))
            .count();
        let corrupt = w
            .tiles
            .iter()
            .filter(|t| t.active() && matches!(t.kind, 23 | 25 | 112))
            .count();
        assert!(hallowed > 200, "the hallow V ({hallowed})");
        assert!(corrupt > 200, "the evil V ({corrupt})");
        // The crimson world: the evil kinds swap (199/203/234).
        let mut w2 = world();
        let mut rng = UnifiedRandom::new(11);
        initialize_hard_mode(&mut w2, false, &mut rng);
        let crimson = w2
            .tiles
            .iter()
            .filter(|t| t.active() && matches!(t.kind, 199 | 203 | 234))
            .count();
        assert!(crimson > 200, "the crimson V ({crimson})");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `hardUpdateWorld`'s first arm is the crystal shards (`:69604-69644`):
    /// a shard-capable stone below the rock layer grows tile 129 through
    /// `hard_update_world`, at the vanilla gates (the `Next(5)` roll, the
    /// radius-6 density).
    #[test]
    fn hard_update_world_grows_the_crystal_shards_first() {
        // 117 = Pearlstone (`CAN_GROW_CRYSTAL_SHARDS`), below the rock layer.
        let mut w = World::new("t".into(), 60, 120);
        w.rock_layer = 40.0;
        for x in 0..60 {
            for y in 20..120 {
                let i = w.index(x, y);
                w.tiles[i] = crate::tile::Tile {
                    kind: 117,
                    ..Default::default()
                };
                w.tiles[i].flags |= crate::tile::FLAG_ACTIVE;
            }
        }
        // Air around the source so the 4-dir pick has somewhere to plant.
        for (dx, dy) in [(-1, 0), (1, 0), (0, 1)] {
            let i = w.index(30 + dx, 80 + dy);
            w.tiles[i] = crate::tile::Tile::default();
        }
        // Sweep seeds: over enough picks the 1/5 gate and the 4-dir roll land.
        let mut grew = 0;
        let mut runs = 0;
        for seed in 1u64..400 {
            let mut w = w.clone();
            let mut rng = UnifiedRandom::new(seed as i32);
            hard_update_world(&mut w, 30, 80, false, true, &mut rng);
            runs += 1;
            grew += w
                .tiles
                .iter()
                .filter(|t| t.active() && t.kind == 129)
                .count();
        }
        assert!(grew > 0, "the shard arm plants 129 ({grew} over {runs} picks)");
    }

    use super::*;
    use crate::tile::{FLAG_ACTIVE, Tile};

    fn world() -> World {
        let mut w = World::new("t".into(), 400, 400);
        w.rock_layer = 200.0;
        w
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    #[test]
    fn the_convert_map_is_verbatim() {
        // `:55139-55205`: the corruption targets (the ebonstone 25, the
        // corrupt grass 23, the ebonsand 112, the purple ice 163, 398, 400)
        // and the crimson twins (203, 199, 234, 200, 399, 401).
        // The corruption (type 1): the ebonstone 25, the corrupt grass 23,
        // the ebonsand 112, the purple ice 163, 398, 400. The crimson
        // (type 4): the crimstone 203, the crimson grass 199, the jungle
        // grass 662, the crimsand 234, the red ice 200, 399, 401.
        assert_eq!(convert_tile(1, true), Some(25));
        assert_eq!(convert_tile(2, true), Some(23));
        assert_eq!(convert_tile(60, true), Some(23));
        assert_eq!(convert_tile(53, true), Some(112));
        assert_eq!(convert_tile(161, true), Some(163));
        assert_eq!(convert_tile(397, true), Some(398));
        assert_eq!(convert_tile(396, true), Some(400));
        assert_eq!(convert_tile(1, false), Some(203));
        assert_eq!(convert_tile(2, false), Some(199));
        assert_eq!(convert_tile(60, false), Some(662));
        assert_eq!(convert_tile(53, false), Some(234));
        assert_eq!(convert_tile(161, false), Some(200));
        assert_eq!(convert_tile(396, false), Some(401));
        assert_eq!(convert_tile(399, false), None, "the already-corrupt tile");
        // The walls (`:55261-55296` corruption / `:55100-55135` crimson): the
        // stone 1 → 3 / 83 — the C#'s chain has no dirt-wall row, so the old
        // natural-slice 2 → 69/193 pair was the deviation M5-SPREAD-2c
        // replaces (the dirt-wall arm lands on conversion types 0/7/11).
        assert_eq!(convert_wall(1, true), Some(3));
        assert_eq!(convert_wall(1, false), Some(83));
        assert_eq!(convert_wall(2, true), None);
        assert_eq!(convert_wall(2, false), None);
    }

    /// M5-SPREAD-2c: the extended conversion types' wall chains are the C#
    /// verbatim (`:55446+` purify, `:55409+` snow, `:55363+` desert,
    /// `:55337+` glowing mushroom, `:55883+` corruption-II).
    #[test]
    fn m5_spread_2c_the_wall_chains_are_verbatim_across_the_types() {
        // Purify (7): the natural walls → 349, the sand/dirt family → 2, the
        // NewWall family → 212-215, the 262/274 switch.
        assert_eq!(growth_wall(1, 7), Some(349));
        // Wall 2 is the Dirt set's own target (`wall != 2` guard, `:55465`) —
        // the second dirt wall 16 converts.
        assert_eq!(growth_wall(2, 7), None);
        assert_eq!(growth_wall(16, 7), Some(2));
        assert_eq!(growth_wall(188, 7), Some(212));
        assert_eq!(growth_wall(262, 7), Some(61));
        assert_eq!(growth_wall(274, 7), Some(185));
        // Snow (6): the stone side → 71, the sand/dirt side → 40.
        assert_eq!(growth_wall(1, 6), Some(71));
        assert_eq!(growth_wall(2, 6), Some(40));
        // Desert (5): 187 / 216.
        assert_eq!(growth_wall(1, 5), Some(187));
        assert_eq!(growth_wall(2, 5), Some(216));
        // Glowing mushroom (3): the four mushroom-safe walls → 80.
        assert_eq!(growth_wall(15, 3), Some(80));
        assert_eq!(growth_wall(2, 3), None);
        // Corruption-II (12): the NewWall family → the corruption analog, else
        // any live non-ebonstone wall → 3.
        // The guards make the set's own members fall through to the ebonstone
        // arm (188 ∈ NewWall1, so `wall != 188` fails and 3 wins).
        assert_eq!(growth_wall(192, 12), Some(188));
        assert_eq!(growth_wall(188, 12), Some(3));
        assert_eq!(growth_wall(187, 12), Some(3));
        // Cleanse (0): the grass family → 64, the 262/274/292-295 switch.
        assert_eq!(growth_wall(69, 0), Some(64));
        assert_eq!(growth_wall(292, 0), Some(204));
        assert_eq!(growth_wall(2, 0), None, "the dirt wall has no type-0 arm");
    }

    /// The growth module's wall-chain entry, straight from the test.
    fn growth_wall(wall: u16, conversion_type: u8) -> Option<u16> {
        crate::growth::convert_wall_by_case(wall, conversion_type, 0, 1.0e9, None)
    }

    #[test]
    fn the_spread_converts_the_neighbours() {
        // The ebonstone source: the grass neighbours convert to the corrupt
        // grass, the stone to the ebonstone.
        let mut w = world();
        put(&mut w, 200, 200, 25); // the ebonstone source
        put(&mut w, 201, 200, 2); // the grass
        put(&mut w, 199, 200, 1); // the stone
        put(&mut w, 200, 199, 53); // the sand
        put(&mut w, 200, 201, 161); // the ice
        // The stone's wall set before the spread (the wall converts with
        // the tile — the already-corrupt tiles leave the Convert map).
        let i = w.index(199, 200);
        w.tiles[i].wall = 1;
        let mut rng = UnifiedRandom::new(1);
        for _ in 0..200 {
            hard_update_world(&mut w, 200, 200, false, true, &mut rng);
        }
        assert_eq!(w.tile(201, 200).unwrap().kind, 23, "the grass corrupted");
        assert_eq!(w.tile(199, 200).unwrap().kind, 25, "the stone corrupted");
        assert_eq!(w.tile(200, 199).unwrap().kind, 112, "the sand corrupted");
        assert_eq!(w.tile(200, 201).unwrap().kind, 163, "the ice corrupted");
        assert_eq!(w.tile(199, 200).unwrap().wall, 3, "the wall corrupted");
    }

    #[test]
    fn the_sunflower_and_chlorophyte_defend() {
        // The sunflower (27) within 2 tiles stops the spread.
        let mut w = world();
        put(&mut w, 200, 200, 25);
        put(&mut w, 201, 200, 2);
        put(&mut w, 204, 200, 27); // the sunflower two tiles out
        put(&mut w, 203, 200, 2);
        let mut rng = UnifiedRandom::new(3);
        for _ in 0..100 {
            hard_update_world(&mut w, 200, 200, false, true, &mut rng);
        }
        assert_eq!(w.tile(203, 200).unwrap().kind, 2, "the sunflower blocks");
        assert_eq!(w.tile(201, 200).unwrap().kind, 23, "the far side corrupts");
        // The chlorophyte defence (`ChlorophyteDefense` is the remix-only
        // conversion — the normal worlds just stop the spread): the
        // corrupt grass next to the chlorophyte stays put.
        let mut w = world();
        put(&mut w, 200, 200, 211); // the chlorophyte
        put(&mut w, 202, 200, 23); // the corrupt grass next to it
        put(&mut w, 205, 200, 2); // the grass beyond the defence radius
        let mut rng = UnifiedRandom::new(4);
        for _ in 0..100 {
            hard_update_world(&mut w, 202, 200, false, true, &mut rng);
        }
        assert_eq!(w.tile(202, 200).unwrap().kind, 23, "the source intact");
        assert_eq!(
            w.tile(201, 200).unwrap().kind,
            0,
            "the defence radius holds the cells"
        );
    }

    #[test]
    fn the_chlorophyte_grows_into_the_mud() {
        // The 211 source: the mud neighbours convert, gated by
        // `CanChlorophyteGrow` (the local density ≤ 40).
        let mut w = world();
        for x in 190..210 {
            for y in 190..210 {
                put(&mut w, x, y, 59); // the mud field
            }
        }
        put(&mut w, 200, 200, 211);
        let mut rng = UnifiedRandom::new(5);
        let mut grew = false;
        for _ in 0..200 {
            hard_update_world(&mut w, 200, 200, false, true, &mut rng);
            if w.tile(201, 200).unwrap().kind == 211 {
                grew = true;
                break;
            }
        }
        assert!(grew, "the chlorophyte spreads into the mud");
    }

    #[test]
    fn the_spread_gates_respect_the_inputs() {
        // `allowed_to_spread = false` freezes the spread.
        let mut w = world();
        put(&mut w, 200, 200, 25);
        put(&mut w, 201, 200, 2);
        let mut rng = UnifiedRandom::new(6);
        for _ in 0..100 {
            hard_update_world(&mut w, 200, 200, false, false, &mut rng);
        }
        assert_eq!(w.tile(201, 200).unwrap().kind, 2, "the spread frozen");
    }

    // W4-f25 helper C1 — `TileIsExposedToAir` (`WorldGen.cs:54782-54800`).
    // WRITTEN NOT RUN (code-only round).

    #[test]
    fn w4_f25_c1_exposure_reads_the_3x3_box() {
        let mut w = World::new("t".into(), 60, 60);
        // A fully-solid stone 3x3: no exposure.
        for x in 29..=31 {
            for y in 29..=31 {
                let i = w.index(x, y);
                w.tiles[i] = crate::tile::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= crate::tile::FLAG_ACTIVE;
            }
        }
        assert!(!tile_is_exposed_to_air(&w, 30, 30), "the solid box seals");
        // An inactive centre opens it (`!tile.active()`).
        let i = w.index(30, 30);
        w.tiles[i].flags &= !crate::tile::FLAG_ACTIVE;
        assert!(tile_is_exposed_to_air(&w, 30, 30), "the inactive centre");
        // Restore; a non-solid active neighbour (a torch 4) opens it too.
        w.tiles[i].flags |= crate::tile::FLAG_ACTIVE;
        let j = w.index(31, 30);
        w.tiles[j].kind = 4;
        assert!(tile_is_exposed_to_air(&w, 30, 30), "the non-solid neighbour");
        // A platform 19 neighbour: tileSolid true but Platforms — exposed.
        w.tiles[j].kind = 19;
        assert!(tile_is_exposed_to_air(&w, 30, 30), "the platform neighbour");
    }

    #[test]
    fn w4_f25_c1_the_border_band_refuses() {
        let w = World::new("t".into(), 60, 60);
        // `InWorld(x, y, 2)`: x/y < 2 or >= width-2 refuses before the scan.
        assert!(!tile_is_exposed_to_air(&w, 1, 30));
        assert!(!tile_is_exposed_to_air(&w, 58, 30));
        assert!(!tile_is_exposed_to_air(&w, 30, 1));
        assert!(!tile_is_exposed_to_air(&w, 30, 58));
        // Inside the band, open air reads exposed.
        assert!(tile_is_exposed_to_air(&w, 2, 30));
    }

    // W4-f25 helper C2 — `BlockBelowMakesSandConvertIntoHardenedSand`
    // (`WorldGen.cs:86562-86583`). WRITTEN NOT RUN (code-only round).

    #[test]
    fn w4_f25_c2_the_floor_below_decides_the_hardening() {
        let mut w = World::new("t".into(), 60, 60);
        // Open air below (the default inactive cell): hardens.
        assert!(block_below_makes_sand_convert_into_hardened_sand(&w, 30, 30));
        // A solid stone below: stays loose.
        let i = w.index(30, 31);
        w.tiles[i] = crate::tile::Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= crate::tile::FLAG_ACTIVE;
        assert!(!block_below_makes_sand_convert_into_hardened_sand(&w, 30, 30));
        // A non-solid active kind below (a torch 4): hardens.
        w.tiles[i].kind = 4;
        assert!(block_below_makes_sand_convert_into_hardened_sand(&w, 30, 30));
        // An INACTIVE stone below: `!nactive()` wins over the kind.
        w.tiles[i].kind = 1;
        w.tiles[i].flags &= !crate::tile::FLAG_ACTIVE;
        assert!(block_below_makes_sand_convert_into_hardened_sand(&w, 30, 30));
        // The bottom row refuses (no cell below).
        assert!(!block_below_makes_sand_convert_into_hardened_sand(&w, 30, 59));
    }

    // W4-f25 helper C3 — the Convert type-2 hallow cell
    // (`WorldGen.cs:55176-55260`). WRITTEN NOT RUN (code-only round).

    #[test]
    fn w4_f25_c3_the_hallow_ladder_is_verbatim() {
        let mut w = World::new("t".into(), 600, 400);
        let rows = [
            (2u16, 109u16),
            (23, 109),
            (199, 109),
            (477, 492), // the golf-grass arm BEFORE grass
            (492, 492), // already golf — stays
            (1, 117),
            (25, 117),
            (203, 117),
            (179, 117), // the moss family rides the stone arm
            (161, 164),
            (163, 164),
            (200, 164),
            (53, 116),
            (112, 116),
            (234, 116),
            (397, 402),
            (398, 402),
            (399, 402),
            (396, 403),
            (400, 403),
            (401, 403),
        ];
        for (i, (src, _)) in rows.iter().enumerate() {
            let x = 100 + i as i32;
            let idx = w.index(x, 200);
            w.tiles[idx] = crate::tile::Tile {
                kind: *src,
                ..Default::default()
            };
            w.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
        }
        for (i, _) in rows.iter().enumerate() {
            convert_hallow_cell(&mut w, 100 + i as i32, 200);
        }
        for (i, (src, dst)) in rows.iter().enumerate() {
            assert_eq!(
                w.tile(100 + i as i32, 200).unwrap().kind,
                *dst,
                "{src} hallowing"
            );
        }
        // The already-hallow cells stay (the `!= target` gates).
        for (x, kind) in [(200, 109u16), (201, 117), (202, 164), (203, 116)] {
            let idx = w.index(x, 200);
            w.tiles[idx] = crate::tile::Tile {
                kind,
                ..Default::default()
            };
            w.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
            convert_hallow_cell(&mut w, x, 200);
            assert_eq!(w.tile(x, 200).unwrap().kind, kind);
        }
    }

    #[test]
    fn w4_f25_c3_the_mud_arm_reads_the_live_neighbours() {
        let mut w = World::new("t".into(), 600, 400);
        let put = |w: &mut World, x: i32, y: i32, kind: u16| {
            let idx = w.index(x, y);
            w.tiles[idx] = crate::tile::Tile {
                kind,
                ..Default::default()
            };
            w.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
        };
        // A mud cell with NO 109 neighbour stays mud.
        put(&mut w, 100, 200, 59);
        convert_hallow_cell(&mut w, 100, 200);
        assert_eq!(w.tile(100, 200).unwrap().kind, 59);
        // The left neighbour converts to 109 in the SAME cell pass order —
        // the mud arm reads the live tile (`Main.tile[i2-1, j2].type`).
        put(&mut w, 110, 200, 59);
        put(&mut w, 109, 200, 2);
        convert_hallow_cell(&mut w, 109, 200); // the grass → 109 first
        convert_hallow_cell(&mut w, 110, 200); // then the mud sees it
        assert_eq!(w.tile(109, 200).unwrap().kind, 109);
        assert_eq!(w.tile(110, 200).unwrap().kind, 0, "the mud → dirt");
        // The thorn kill arm (`:55248-55255`).
        put(&mut w, 120, 200, 32);
        convert_hallow_cell(&mut w, 120, 200);
        assert!(
            !w.tile(120, 200).unwrap().active(),
            "the thorn dies (no 109 neighbour, no mud arm)"
        );
    }

    #[test]
    fn w4_f25_c3_the_wall_chain_and_torch() {
        let mut w = World::new("t".into(), 600, 400);
        // The flat wall ladder (`:55179-55210` via convert_wall_by_case).
        for (x, wall, want) in [
            (100i32, 63u16, 70u16),
            (101, 1, 28),
            (102, 216, 219),
            (103, 187, 222),
            (104, 188, 200),
            (105, 189, 201),
            (106, 190, 202),
            (107, 191, 203),
        ] {
            let idx = w.index(x, 200);
            w.tiles[idx] = crate::tile::Tile {
                kind: 1,
                wall,
                ..Default::default()
            };
            w.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
            convert_hallow_cell(&mut w, x, 200);
            assert_eq!(w.tile(x, 200).unwrap().wall, want, "wall {wall}");
            // The stone tile hallowed alongside (kind 1 → 117).
            assert_eq!(w.tile(x, 200).unwrap().kind, 117);
        }
        // The torch → style 20 (`:55216-55218`), the default style set.
        let idx = w.index(110, 200);
        w.tiles[idx] = crate::tile::Tile {
            kind: 4,
            frame_y: 9 * 22,
            ..Default::default()
        };
        w.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
        convert_hallow_cell(&mut w, 110, 200);
        assert_eq!(w.tile(110, 200).unwrap().frame_y, 20 * 22);
        // A non-convertible style (e.g. 1) stays.
        let idx = w.index(111, 200);
        w.tiles[idx] = crate::tile::Tile {
            kind: 4,
            frame_y: 22,
            ..Default::default()
        };
        w.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
        convert_hallow_cell(&mut w, 111, 200);
        assert_eq!(w.tile(111, 200).unwrap().frame_y, 22);
    }

    #[test]
    fn w4_f25_c3_the_sized_walk_carries_the_manhattan_gate() {
        let mut w = World::new("t".into(), 600, 400);
        for x in 97..=103 {
            for y in 197..=203 {
                let idx = w.index(x, y);
                w.tiles[idx] = crate::tile::Tile {
                    kind: 2,
                    ..Default::default()
                };
                w.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
            }
        }
        convert_sized_hallow(&mut w, 100, 200, 3);
        assert_eq!(w.tile(100, 200).unwrap().kind, 109, "the centre");
        assert_eq!(w.tile(100, 197).unwrap().kind, 109, "Manhattan 3");
        assert_eq!(w.tile(97, 197).unwrap().kind, 2, "Manhattan 6 skips");
        assert_eq!(w.tile(103, 203).unwrap().kind, 2, "the far corner skips");
    }

    // W4-f25 helper C4 — the Convert spray cells, types 5/6/7
    // (`WorldGen.cs:55363-55526`). WRITTEN NOT RUN (code-only round).

    #[test]
    fn w4_f25_c4_the_desertify_ladder_and_the_hardening_floor() {
        let mut w = World::new("t".into(), 600, 400);
        let put = |w: &mut World, x: i32, y: i32, kind: u16| {
            let idx = w.index(x, y);
            w.tiles[idx] = crate::tile::Tile {
                kind,
                ..Default::default()
            };
            w.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
        };
        // Grass over OPEN AIR below → hardened 397 (`:55386`).
        put(&mut w, 100, 200, 2);
        convert_spray_cell(&mut w, 100, 200, 5);
        assert_eq!(w.tile(100, 200).unwrap().kind, 397);
        // Grass over solid stone below → loose sand 53.
        put(&mut w, 101, 200, 2);
        put(&mut w, 101, 201, 1);
        convert_spray_cell(&mut w, 101, 200, 5);
        assert_eq!(w.tile(101, 200).unwrap().kind, 53);
        // The already-sand cell stays (`type != 53` gate); hardened → 397
        // stays; stone/ice/sandstone/moss → 396.
        for (x, src, dst) in [
            (102i32, 53u16, 53u16),
            (103, 398, 397),
            (104, 1, 396),
            (105, 161, 396),
            (106, 396, 396),
            (107, 179, 396),
            (108, 147, 397), // snow over air hardens too (the union arm)
        ] {
            put(&mut w, x, 200, src);
            convert_spray_cell(&mut w, x, 200, 5);
            assert_eq!(w.tile(x, 200).unwrap().kind, dst, "{src} desertify");
        }
        // The wall ladder (`:55366-55373`): stone-side → 187, dirt-side → 216.
        for (x, wall, want) in [(120i32, 1u16, 187u16), (121, 2, 216)] {
            put(&mut w, x, 200, 0); // an inactive-kind tile: dirt 0 active
            w.tile_mut(x, 200).unwrap().wall = wall;
            convert_spray_cell(&mut w, x, 200, 5);
            assert_eq!(w.tile(x, 200).unwrap().wall, want, "wall {wall}");
        }
    }

    #[test]
    fn w4_f25_c4_the_snow_ladder() {
        let mut w = World::new("t".into(), 600, 400);
        let put = |w: &mut World, x: i32, y: i32, kind: u16| {
            let idx = w.index(x, y);
            w.tiles[idx] = crate::tile::Tile {
                kind,
                ..Default::default()
            };
            w.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
        };
        for (x, src, dst) in [
            (100i32, 2u16, 147u16),
            (101, 53, 147),
            (102, 397, 147), // hardened sand rides the FIRST union
            (103, 147, 147), // already snow stays
            (104, 0, 147),   // dirt 0 is in the union
            (105, 1, 161),
            (106, 161, 161),
            (107, 396, 161),
            (108, 182, 161), // moss → ice
        ] {
            put(&mut w, x, 200, src);
            convert_spray_cell(&mut w, x, 200, 6);
            assert_eq!(w.tile(x, 200).unwrap().kind, dst, "{src} snow");
        }
        // The wall ladder (`:55412-55419`): stone-side → 71, dirt-side → 40.
        put(&mut w, 120, 200, 1);
        w.tile_mut(120, 200).unwrap().wall = 1;
        convert_spray_cell(&mut w, 120, 200, 6);
        assert_eq!(w.tile(120, 200).unwrap().wall, 71);
        assert_eq!(w.tile(120, 200).unwrap().kind, 161);
    }

    #[test]
    fn w4_f25_c4_the_purify_ladder_and_the_air_exposure() {
        let mut w = World::new("t".into(), 600, 400);
        let put = |w: &mut World, x: i32, y: i32, kind: u16| {
            let idx = w.index(x, y);
            w.tiles[idx] = crate::tile::Tile {
                kind,
                ..Default::default()
            };
            w.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
        };
        // Stone/ice/sandstone → 1; NO moss arm (the moss 179 stays).
        for (x, src, dst) in [
            (100i32, 25u16, 1u16),
            (101, 163, 1),
            (102, 400, 1),
            (103, 179, 179),
            (104, 492, 477), // golf grass first
            (105, 23, 2),
            (106, 199, 2),
        ] {
            put(&mut w, x, 200, src);
            convert_spray_cell(&mut w, x, 200, 7);
            assert_eq!(w.tile(x, 200).unwrap().kind, dst, "{src} purify");
        }
        // Sand in a sealed solid box → dirt 0 (`TileIsExposedToAir` false).
        for x in 109..=111 {
            for y in 199..=201 {
                put(&mut w, x, y, 1);
            }
        }
        put(&mut w, 110, 200, 112);
        convert_spray_cell(&mut w, 110, 200, 7);
        assert_eq!(w.tile(110, 200).unwrap().kind, 0, "the sealed sand → dirt");
        // Sand with an open neighbour → grass 2.
        put(&mut w, 120, 200, 112);
        convert_spray_cell(&mut w, 120, 200, 7);
        assert_eq!(w.tile(120, 200).unwrap().kind, 2, "the exposed sand → grass");
        // The thorn kill with the 69 exemption (`:55518`).
        put(&mut w, 130, 200, 32);
        convert_spray_cell(&mut w, 130, 200, 7);
        assert!(!w.tile(130, 200).unwrap().active(), "the thorn dies");
        put(&mut w, 131, 200, 69);
        convert_spray_cell(&mut w, 131, 200, 7);
        assert!(w.tile(131, 200).unwrap().active(), "the 69 corrupt thorn stays");
        // The `:55465` union pre-gate: a hardened-sand wall 216 → 2.
        put(&mut w, 140, 200, 1);
        w.tile_mut(140, 200).unwrap().wall = 216;
        convert_spray_cell(&mut w, 140, 200, 7);
        assert_eq!(w.tile(140, 200).unwrap().wall, 2, "the hardened wall purifies");
        // The natural arm: a stone wall 1 → 349 (`:55450`).
        put(&mut w, 141, 200, 1);
        w.tile_mut(141, 200).unwrap().wall = 1;
        convert_spray_cell(&mut w, 141, 200, 7);
        assert_eq!(w.tile(141, 200).unwrap().wall, 349);
    }

    #[test]
    fn w4_f25_c4_the_spray_torch_styles_and_the_walk_gate() {
        let mut w = World::new("t".into(), 600, 400);
        for (x, ty, want) in [(100i32, 5u8, 16i16), (101, 6, 9), (102, 7, 0)] {
            let idx = w.index(x, 200);
            w.tiles[idx] = crate::tile::Tile {
                kind: 4,
                frame_y: 18 * 22,
                ..Default::default()
            };
            w.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
            convert_spray_cell(&mut w, x, 200, ty);
            assert_eq!(w.tile(x, 200).unwrap().frame_y, want * 22, "type {ty}");
        }
        // The sized walk's Manhattan gate + the type filter.
        for x in 197..=203 {
            for y in 197..=203 {
                let idx = w.index(x, y);
                w.tiles[idx] = crate::tile::Tile {
                    kind: 2,
                    ..Default::default()
                };
                w.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
            }
        }
        convert_sized_spray(&mut w, 200, 200, 6, 3);
        assert_eq!(w.tile(200, 197).unwrap().kind, 147, "Manhattan 3");
        assert_eq!(w.tile(197, 197).unwrap().kind, 2, "Manhattan 6 skips");
        // An unported type is a no-op, not a mis-conversion.
        convert_sized_spray(&mut w, 200, 200, 9, 0);
        assert_eq!(w.tile(200, 200).unwrap().kind, 147);
    }

    // W4-f25 helper C5 — the Convert chlorophyte cells, types 8/9/10
    // (`WorldGen.cs:55665-55763`). WRITTEN NOT RUN (code-only round).

    #[test]
    fn w4_f25_c5_type_8_converts_only_mud_and_jungle_grass() {
        let mut w = World::new("t".into(), 600, 400);
        let put = |w: &mut World, x: i32, y: i32, kind: u16| {
            let idx = w.index(x, y);
            w.tiles[idx] = crate::tile::Tile {
                kind,
                ..Default::default()
            };
            w.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
        };
        for (x, src, dst) in [
            (100i32, 59u16, 211u16),
            (101, 60, 211),
            (102, 2, 2), // the plain grass is NOT in the type-8 arm
            (103, 1, 1),
        ] {
            put(&mut w, x, 200, src);
            convert_chlorophyte_cell(&mut w, x, 200, 8);
            assert_eq!(w.tile(x, 200).unwrap().kind, dst, "{src} type 8");
        }
    }

    #[test]
    fn w4_f25_c5_types_9_and_10_differ_on_the_pure_rows() {
        let mut w = World::new("t".into(), 600, 400);
        let put = |w: &mut World, x: i32, y: i32, kind: u16| {
            let idx = w.index(x, y);
            w.tiles[idx] = crate::tile::Tile {
                kind,
                ..Default::default()
            };
            w.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
        };
        // Type 9's full grass set + dirt (`:55678-55690`).
        for (x, src, dst) in [
            (100i32, 2u16, 60u16),
            (101, 109, 60),
            (102, 477, 60),
            (103, 492, 60),
            (104, 0, 59),
            (105, 23, 60),
            (106, 199, 60),
            (107, 661, 60),
            (108, 662, 60),
            (109, 25, 1),
            (110, 203, 1),
            (111, 112, 53),
            (112, 234, 53),
            (113, 398, 397),
            (114, 399, 397),
            (115, 400, 396),
            (116, 401, 396),
        ] {
            put(&mut w, x, 200, src);
            convert_chlorophyte_cell(&mut w, x, 200, 9);
            assert_eq!(w.tile(x, 200).unwrap().kind, dst, "{src} type 9");
        }
        // Type 10: the pure rows STAY, the infection rows convert
        // (`:55726-55749` — the ChlorophyteDefense remix arm).
        for (x, src, dst) in [
            (100i32, 2u16, 2u16),
            (101, 109, 109),
            (102, 477, 477),
            (103, 492, 492),
            (104, 0, 0),
            (105, 23, 60),
            (106, 199, 60),
            (107, 661, 60),
            (108, 662, 60),
            (109, 25, 1),
            (110, 203, 1),
        ] {
            put(&mut w, x, 210, src);
            convert_chlorophyte_cell(&mut w, x, 210, 10);
            assert_eq!(w.tile(x, 210).unwrap().kind, dst, "{src} type 10");
        }
        // The thorn/evil kill row, both types (`:55707-55718`/`:55750-55761`).
        for (i, kind) in [24u16, 32, 201, 205, 352, 636].iter().enumerate() {
            put(&mut w, 130 + i as i32, 200, *kind);
            convert_chlorophyte_cell(&mut w, 130 + i as i32, 200, 9);
            assert!(
                !w.tile(130 + i as i32, 200).unwrap().active(),
                "{kind} dies"
            );
        }
    }
}
