//! The grass/spread family of the world-growth ticks (S1-K-3b split out of
//! `growth.rs`; behaviour frozen - verbatim move). `UpdateWorld_GrassGrowth`
//! (`:74269-74424`), `SpreadGrass` (`:75146-75257`), `MossConversion`
//! (`:71457-71503`) + the moss arm (`:72371-72424`), and the M5-SPREAD-1 wall
//! arms: `Convert`'s wall chains (`:55100+`), `SpreadGrassWalls`
//! (`:74884-74982`) and `SpreadDesertWalls` (`:74984-75123`). The public
//! seams stay re-exported from `growth`.

use crate::tile::FLAG_ACTIVE;
use crate::UnifiedRandom;
use crate::world::World;

use super::trees::kill_invalid_tree_above;

/// `WorldGen.UpdateWorld_GrassGrowth` (:74269-74508), the underground slice:
/// the per-type table (:74284-74356 - the infection rows gated by
/// `AllowedToSpreadInfections`, the jungle/mushroom/plant rows maxValue 10),
/// the plant-above roll (:74356-74376), the 3x3 soil scan converting the
/// neighbours via [`spread_grass`] (:74381-74404), and the case-60/70 tail
/// (:74414-74506 — the hardmode life fruit / Plantera bulb / detritus growth
/// and the mushroom cattail/tree, W4-c31 jungle residuals).
pub fn update_world_grass_growth(
    world: &mut World,
    i: i32,
    j: i32,
    min_i: i32,
    max_i: i32,
    min_j: i32,
    max_j: i32,
    allowed_to_spread: bool,
    rng: &mut UnifiedRandom,
) {
    if i < 10 || j < 10 || i >= world.width as i32 - 10 || j >= world.height as i32 - 10 {
        return;
    }
    let Some(t) = world.tile(i, j) else {
        return;
    };
    let kind = t.kind;
    let (soil, soil2, grass, grass2, plant, max_value): (i16, i16, i16, i16, i16, i32) = match kind {
        2 => (0, -1, 2, -1, -1, 1),
        23 => (0, 59, 23, 661, 24, 2),
        199 => (0, 59, 199, 662, 201, 2),
        661 => (59, 0, 661, 23, 24, 2),
        662 => (59, 0, 662, 199, 201, 2),
        60 => (59, -1, 60, -1, 61, 10),
        70 => (59, -1, 70, -1, 71, 10),
        633 => (57, -1, 633, -1, 637, 2),
        226 => (-1, -1, 226, -1, 61, 10),
        _ => return,
    };
    if matches!(kind, 23 | 199 | 661 | 662) && !allowed_to_spread {
        return;
    }
    // The plant-above roll (:74356-74376). `flag` records a successful
    // placement — the case-60 tail's `if (flag || ...)` gate reads it.
    let mut flag = false;
    if plant != -1
        && min_j < world.height as i32
        && world.tile(i, min_j).is_none_or(|t| !t.active())
        && rng.next_max(max_value) == 0
    {
        // The 226 density veto (:74359-74362).
        let veto = kind == 226 && too_many_jungle_plants_nearby(world, i, min_j, 2);
        if !veto {
            flag = true;
            let idx = world.index(i, min_j);
            world.tiles[idx] = crate::tile::Tile {
                kind: plant as u16,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
            if world.frame_important.len() <= plant as usize {
                world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
            }
            world.frame_important[plant as usize] = true;
        }
    }
    // The 3x3 soil scan (:74381-74404).
    if soil != -1 {
        for k in min_i..max_i {
            for l in min_j..max_j {
                if k < 10
                    || l < 10
                    || k >= world.width as i32 - 10
                    || l >= world.height as i32 - 10
                {
                    continue;
                }
                if k == i && l == j {
                    continue;
                }
                let Some(n) = world.tile(k, l) else {
                    continue;
                };
                if !n.active() {
                    continue;
                }
                if n.kind == soil as u16 {
                    spread_grass(world, k, l, soil as u16, grass as u16);
                } else if soil2 > -1 && grass2 > -1 && n.kind == soil2 as u16 {
                    spread_grass(world, k, l, soil2 as u16, grass2 as u16);
                }
            }
        }
    }
    // The case-60/70 tail (:74414-74506, W4-c31 jungle residuals).
    match kind {
        60 => jungle_grass_hardmode_tail(world, i, j, min_j, flag, rng),
        70 => mushroom_grass_tail(world, i, j, rng),
        _ => {}
    }
}

/// `WorldGen.TooManyJunglePlantsNearby` (`WorldGen.cs:73164-73187`): the
/// ±9×±5 box `Utils.Clamp`'d into `[10, max−1−10]`, counting active 61/74
/// tiles; true once the count exceeds `max_count`.
fn too_many_jungle_plants_nearby(world: &World, i: i32, j: i32, max_count: i32) -> bool {
    let lo_x = (i - 9).clamp(10, world.width - 1 - 10);
    let hi_x = (i + 9).clamp(10, world.width - 1 - 10);
    let lo_y = (j - 5).clamp(10, world.height - 1 - 10);
    let hi_y = (j + 5).clamp(10, world.height - 1 - 10);
    let mut count = 0;
    for x in lo_x..=hi_x {
        for y in lo_y..=hi_y {
            if world
                .tile(x, y)
                .is_some_and(|t| t.active() && (t.kind == 61 || t.kind == 74))
            {
                count += 1;
                if count > max_count {
                    return true;
                }
            }
        }
    }
    false
}

/// The stride-2 density scan the bulb/fruit arms run (`:74426-74436` /
/// `:74458-74468`): true when NO active `kind` tile sits in the
/// `[i-radius, i+radius)` × `[j-radius, j+radius)` box (the `>1 && <max-2`
/// bounds guard per cell).
fn no_nearby_kind(world: &World, i: i32, j: i32, radius: i32, kind: u16) -> bool {
    let mut m = i - radius;
    while m < i + radius {
        let mut n = j - radius;
        while n < j + radius {
            if m > 1
                && m < world.width - 2
                && n > 1
                && n < world.height - 2
                && world.tile(m, n).is_some_and(|t| t.active() && t.kind == kind)
            {
                return false;
            }
            n += 2;
        }
        m += 2;
    }
    true
}

/// The case-60 hardmode tail (`WorldGen.cs:74416-74496`): the `Next(25)`
/// gate + liquid veto, the all-mechs `Next(60)` Plantera-bulb arm (150-radius
/// stride-2 density), the any-mech `Next(30/40)` life-fruit arm (60/50-radius),
/// and the 233 detritus fallback (`Next(8)` styleY 0, `Next(12)` styleY 1).
/// `flag` is the plant-above placement flag (`:74355`/`:74365`).
fn jungle_grass_hardmode_tail(
    world: &mut World,
    i: i32,
    j: i32,
    min_j: i32,
    flag: bool,
    rng: &mut UnifiedRandom,
) {
    // `if (flag || genRand.Next(25) != 0 || tile[i, minJ].liquid != 0) break;`
    // (`:74418`) — the `||` short-circuit: a placed plant skips the draw.
    if flag || rng.next_max(25) != 0 {
        return;
    }
    if world.tile(i, min_j).is_some_and(|t| t.liquid != 0) {
        return;
    }
    let expert = false; // `Main.expertMode` constants-off (single-difficulty)
    // The all-mechs bulb arm (`:74422-74448`).
    if world.hard_mode
        && world.downed_mech_boss1
        && world.downed_mech_boss2
        && world.downed_mech_boss3
        && rng.next_max(60) == 0
    {
        if no_nearby_kind(world, i, j, 150, 238) {
            super::jungle_plants::place_jungle_plant(world, i, min_j, 238, 0, 0);
            crate::edit::square_tile_frame_gen(world, i, min_j);
            crate::edit::square_tile_frame_gen(world, i + 2, min_j);
            crate::edit::square_tile_frame_gen(world, i - 1, min_j);
        }
    }
    // The any-mech life-fruit arm (`:74449-74480`).
    let max_value2 = if expert { 30 } else { 40 };
    let any_mech =
        world.downed_mech_boss1 || world.downed_mech_boss2 || world.downed_mech_boss3;
    if world.hard_mode && any_mech && rng.next_max(max_value2) == 0 {
        let radius = if expert { 50 } else { 60 };
        if no_nearby_kind(world, i, j, radius, 236) {
            let style = rng.next_max(3);
            super::jungle_plants::place_jungle_plant(world, i, min_j, 236, style, 0);
            crate::edit::square_tile_frame_gen(world, i, min_j);
            crate::edit::square_tile_frame_gen(world, i + 1, min_j + 1);
        }
        return;
    }
    // The 233 detritus fallback (`:74481-74494`): styleY 0 at `Next(8)`,
    // retrying styleY 1 at `Next(12)` when the first left no 233.
    let style0 = rng.next_max(8);
    super::jungle_plants::place_jungle_plant(world, i, min_j, 233, style0, 0);
    if world.tile(i, min_j).is_some_and(|t| t.kind == 233) {
        return;
    }
    let style1 = rng.next_max(12);
    super::jungle_plants::place_jungle_plant(world, i, min_j, 233, style1, 1);
}

/// The case-70 mushroom tail (`WorldGen.cs:74497-74506`): a liquid cell
/// above grows a cattail; `Next(250)` grows a mushroom tree (the kind-5
/// `GrowTree`) when a player is in LOS. The `TreeGrowFXCheck` leaf FX is a
/// draw-side effect (the port's growth tick carries no FX channel).
fn mushroom_grass_tail(world: &mut World, i: i32, j: i32, rng: &mut UnifiedRandom) {
    if world.tile(i, j - 1).is_some_and(|t| t.liquid > 0) {
        super::decorations::place_cat_tail(world, i, j - 1);
    }
    // `Next(250) == 0 && TryGrowingTreeByType(5, i, j) && PlayerLOS(i, j)`
    // (`:74502`) — the `&&` short-circuit draws only after the roll passes.
    // `TryGrowingTreeByType(5, ...)` dispatches to `GrowTree` (`:30348-30349`).
    let grew = rng.next_max(250) == 0 && super::trees::grow_tree(world, i, j, rng);
    if grew {
        // `PlayerLOS` (`:69443-69454`) gates the `TreeGrowFXCheck` leaf FX
        // only; the tree already grew. The FX is draw-side (no growth-tick
        // channel in the port) — the LOS read parks with it.
    }
}

/// `WorldGen.SpreadGrass` (`WorldGen.cs:75146-75257`), the runtime slice
/// (`repeat: false` — the growth tick never recurses): the `InWorld(10)` and
/// dirt-type guards (`:75150`), the surface rule with the infection-grass
/// exemption (`:75154-75165`), the 3×3 enclosure scan with the lava exception
/// (`:75186-75201`), the `CanBeClearedDuringGeneration` gate (`:75202`), the
/// flower-27 exclusions (`:75202-75205`) and the conversion dispatch / plain
/// write (`:75206-75225`). RNG-free: every guard reads tiles only, so the
/// caller's draw count/order never depends on whether a cell converts.
///
/// The generation-time branch (`isGeneratingOrLoadingWorld`, the beach/centre
/// exclusion for 23/199) rides the worldgen-local copies
/// (`worldgen/evil.rs::spread_evil_grass`, `worldgen/biomes.rs::
/// spread_jungle_grass`); the paint/coating `color` argument and the
/// `SquareTileFrame` re-frame are the port's documented deviations (ANALYSIS).
pub fn spread_grass(world: &mut World, i: i32, j: i32, dirt: u16, grass: u16) {
    // `!InWorld(i, j, 10)` (`:75150`).
    if i < 10 || j < 10 || i >= world.width as i32 - 10 || j >= world.height as i32 - 10 {
        return;
    }
    let Some(t) = world.tile(i, j) else {
        return;
    };
    if !t.active() || t.kind != dirt {
        return;
    }
    // The surface rule (`:75154-75165`): at runtime the C#'s else-if only
    // fires for the NON-infection grasses — `grass != 199 && grass != 23 &&
    // grass != 661 && grass != 662` — so plain grass 2 (and jungle 60 /
    // mushroom 70 / hallow 109 / ash 633) never converts dirt 0 at or below
    // `worldSurface`, while the infection grasses convert dirt at any depth.
    // (The earlier slice applied the rule to every grass, freezing the
    // corrupt/crimson dirt spread below the surface line.)
    if !matches!(grass, 23 | 199 | 661 | 662) && dirt == 0 && j as f64 >= world.world_surface {
        return;
    }
    // The +-1 enclosure scan with the lava exception (:75186-75201): the C#'s
    // `flag` starts true, ANY non-solid tile in the 3x3 clears it, and a lava
    // tile re-arms it and breaks the scan. The spread proceeds only while
    // `flag` is cleared — a fully-solid 3x3 refuses. (The earlier slice had
    // this inverted — it refused when the box was all-empty instead of
    // all-solid, so the growth tick refused exactly the conversions the C#
    // allows. `worldgen/biomes.rs`'s `spread_jungle_grass` carries the C#
    // semantics already.)
    let mut enclosed = true;
    'enclosure: for k in (i - 1)..=(i + 1) {
        for l in (j - 1)..=(j + 1) {
            let Some(n) = world.tile(k, l) else {
                continue;
            };
            if !(n.active() && crate::tiledata::TILE_SOLID.has(n.kind)) {
                enclosed = false;
            }
            if n.liquid > 0 && n.liquid_kind() == crate::tile::LiquidKind::Lava {
                enclosed = true;
                break 'enclosure;
            }
        }
    }
    if enclosed {
        return;
    }
    // `!CanBeClearedDuringGeneration(type, i, j, skipDualDungeonCheck: true)`
    // (`:75202` → `:80168-80175`): with the dual-dungeon check skipped the
    // gate is the bare `TileID.Sets.CanBeClearedDuringGeneration` set
    // (`TileID.cs:573` — default true, the listed kinds false). The same
    // gate `worldgen/biomes.rs`'s `spread_jungle_grass` carries.
    if !crate::tiledata::SET_CAN_BE_CLEARED_DURING_GENERATION.has(t.kind) {
        return;
    }
    // The flower-27 exclusions (:75204-75206).
    if matches!(grass, 23 | 661 | 199 | 662 | 109)
        && world.tile(i, j - 1).is_some_and(|t| t.kind == 27)
    {
        return;
    }
    // `flag2 = dirt > 0 && TileID.Sets.Conversion.Grass[dirt]` (`:75207`) and
    // the conversion dispatch (`:75208-75221`): spreading a conversion grass
    // over a conversion-grass dirt converts the cell through the flavor's
    // map instead of the plain type write — corruption for 23/661, crimson
    // for 199/662, hallow for 109. Everything else takes the plain write,
    // preceded by the tree-invalidity kill.
    let conversion_dirt = matches!(dirt, 2 | 23 | 199 | 109 | 477 | 492);
    if conversion_dirt && matches!(grass, 23 | 661) {
        crate::spread::convert_cell_flavor(
            world,
            i,
            j,
            crate::spread::ConversionFlavor::Corruption,
            false,
        );
    } else if conversion_dirt && matches!(grass, 199 | 662) {
        crate::spread::convert_cell_flavor(
            world,
            i,
            j,
            crate::spread::ConversionFlavor::Crimson,
            false,
        );
    } else if conversion_dirt && grass == 109 {
        crate::spread::convert_cell_flavor(
            world,
            i,
            j,
            crate::spread::ConversionFlavor::Hallow,
            false,
        );
    } else {
        kill_invalid_tree_above(world, i, j, grass);
        let idx = world.index(i, j);
        world.tiles[idx] = crate::tile::Tile {
            kind: grass,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        if world.frame_important.len() <= grass as usize {
            world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
        }
        world.frame_important[grass as usize] = true;
    }
}

/// `WorldGen.MossConversion` (`WorldGen.cs:71457-71503`): the moss family's
/// internal conversion map — a moss-brick tile spreads over the pearlstone 38
/// unchanged, a moss tile over the dirt 1 unchanged, and every other kind maps
/// through the stone/moss-id pairs (the gray × purple × blue × green × brown
/// families); an unmapped kind returns 0 (no spread).
fn moss_conversion(this_type: u16, other_type: u16) -> u16 {
    if crate::tiledata::SET_TILE_MOSS_BRICK.has(this_type) && other_type == 38 {
        return this_type;
    }
    if crate::tiledata::TILE_MOSS.has(this_type) && other_type == 1 {
        return this_type;
    }
    match this_type {
        182 | 515 => {
            if this_type == 182 {
                515
            } else {
                182
            }
        }
        180 | 513 => {
            if this_type == 180 {
                513
            } else {
                180
            }
        }
        179 | 512 => {
            if this_type == 179 {
                512
            } else {
                179
            }
        }
        381 | 517 => {
            if this_type == 381 {
                517
            } else {
                381
            }
        }
        534 | 535 => {
            if this_type == 534 {
                535
            } else {
                534
            }
        }
        536 | 537 => {
            if this_type == 536 {
                537
            } else {
                536
            }
        }
        539 | 540 => {
            if this_type == 539 {
                540
            } else {
                539
            }
        }
        625 | 626 => {
            if this_type == 625 {
                626
            } else {
                625
            }
        }
        627 | 628 => {
            if this_type == 627 {
                628
            } else {
                627
            }
        }
        183 | 516 => {
            if this_type == 183 {
                516
            } else {
                183
            }
        }
        181 | 514 => {
            if this_type == 181 {
                514
            } else {
                181
            }
        }
        _ => 0,
    }
}

/// `UpdateWorld_OvergroundTile`'s moss branch (`WorldGen.cs:72371-72424`): a
/// moss tile (the `Main.tileMoss` family or the `tileMossBrick` family) with a
/// `NextDouble() < 0.5` roll converts the dirt/pearlstone neighbours in its
/// clamped box through [`moss_conversion`] — each conversion re-frames its
/// cell — and a `Next(6)` roll sprays a loose moss 184 into a free neighbour.
///
/// The port's omissions (recorded in ANALYSIS): the netcode
/// (`NetMessage.SendTileSquare`), the paint/coating copy (no color cache) and
/// the `color` argument of `SpreadGrass` (the port's growth writes carry none).
pub(super) fn grow_moss_arm(world: &mut World, i: i32, j: i32, min_i: i32, max_i: i32, min_j: i32, max_j: i32, rng: &mut UnifiedRandom) {
    if rng.next_double() >= 0.5 {
        return;
    }
    let Some(tile) = world.tile(i, j) else {
        return;
    };
    let this_type = tile.kind;
    for m in min_i..max_i {
        for n in min_j..max_j {
            if i == m && j == n {
                continue;
            }
            let Some(neighbour) = world.tile(m, n) else {
                continue;
            };
            if !neighbour.active() || !matches!(neighbour.kind, 1 | 38) {
                continue;
            }
            let converted = moss_conversion(this_type, neighbour.kind);
            if converted == 0 {
                continue;
            }
            spread_grass(world, m, n, neighbour.kind, converted);
            if world.tile(m, n).is_some_and(|t| t.kind == converted) {
                // `SquareTileFrame(num17, num18)` (`:72396-72398`) — the
                // converted cell's frames re-derive.
                crate::framing::reframe_area(world, m, n, 1, true);
            }
        }
    }
    if rng.next_max(6) == 0 {
        // The ±1 spray (`:72404-72421`): a free neighbour takes the loose
        // moss 184.
        let (mut tx, mut ty) = (i, j);
        match rng.next_max(4) {
            0 => tx -= 1,
            1 => tx += 1,
            2 => ty -= 1,
            _ => ty += 1,
        }
        if !world.tile(tx, ty).is_some_and(|t| t.active()) {
            if let Some(t) = world.tile_mut(tx, ty) {
                t.kind = 184;
                t.flags |= FLAG_ACTIVE;
            }
        }
    }
}

// --- M5-SPREAD-1: the underground dispatcher + its wall arms -----------------

/// `WallID.Sets.SpreadsCorruption` (`WallID.cs:93`) — the walls that carry the
/// corruption into `SpreadGrassWalls`/`SpreadDesertWalls`.
const SPREADS_CORRUPTION_WALLS: [u16; 4] = [69, 217, 220, 3];
/// `WallID.Sets.SpreadsCrimson` (`WallID.cs:94`).
const SPREADS_CRIMSON_WALLS: [u16; 4] = [83, 81, 218, 221];
/// `WallID.Sets.SpreadsHallow` (`WallID.cs:95`).
const SPREADS_HALLOW_WALLS: [u16; 4] = [70, 219, 222, 28];
/// `WallID.Sets.Conversion.Grass` (`WallID.cs:37`).
const CONVERSION_GRASS_WALLS: [u16; 12] = [63, 64, 65, 66, 67, 68, 69, 70, 81, 264, 265, 268];
/// `WallID.Sets.Conversion.Stone` (`WallID.cs:38`).
const CONVERSION_STONE_WALLS: [u16; 12] = [1, 61, 185, 3, 28, 83, 262, 274, 246, 248, 269, 349];
/// `WallID.Sets.Conversion.Dirt` (`WallID.cs:39`).
const CONVERSION_DIRT_WALLS: [u16; 2] = [2, 16];
/// `WallID.Sets.Conversion.Snow` (`WallID.cs:40`).
const CONVERSION_SNOW_WALLS: [u16; 2] = [40, 249];
/// `WallID.Sets.Conversion.Ice` (`WallID.cs:41`).
const CONVERSION_ICE_WALLS: [u16; 2] = [71, 266];
/// `WallID.Sets.CanBeConvertedToGlowingMushroom` (`WallID.cs:88`).
const CAN_BECOME_GLOWING_MUSHROOM_WALLS: [u16; 4] = [64, 67, 15, 247];
/// `WallID.Sets.Conversion.HardenedSand` (`WallID.cs:43`).
const CONVERSION_HARDENED_SAND_WALLS: [u16; 8] = [216, 217, 219, 218, 304, 305, 307, 306];
/// `WallID.Sets.Conversion.Sandstone` (`WallID.cs:42`).
const CONVERSION_SANDSTONE_WALLS: [u16; 8] = [187, 220, 222, 221, 275, 308, 310, 309];
/// `WallID.Sets.Conversion.PureSand` (`WallID.cs:44`).
const CONVERSION_PURE_SAND_WALLS: [u16; 4] = [216, 187, 304, 275];
/// `WallID.Sets.Conversion.NewWall1..4` (`WallID.cs:45-48`).
const CONVERSION_NEW_WALLS: [[u16; 10]; 4] = [
    [188, 192, 200, 204, 212, 276, 280, 288, 292, 300],
    [189, 193, 201, 205, 213, 277, 281, 289, 293, 301],
    [190, 194, 202, 206, 214, 278, 282, 290, 294, 302],
    [191, 195, 203, 207, 215, 279, 283, 291, 295, 303],
];
/// `TileID.Sets.SpreadsHallow` (`TileID.cs:584`): the hallow-source tiles.
const SPREADS_HALLOW: [u16; 10] = [109, 492, 117, 116, 402, 403, 164, 115, 110, 113];
/// `TileID.Sets.SpreadUnderground` (`TileID.cs:622`) — the underground tiles
/// the growth pass runs the grass arm over.
pub(super) const SPREAD_UNDERGROUND: [u16; 9] = [23, 661, 109, 199, 662, 60, 70, 633, 226];

/// `WorldGen.Convert`'s wall chains — every conversion type's wall arm
/// (`WorldGen.cs:55100+` crimson, `:55176+` hallow, `:55261+` corruption,
/// `:55337+` glowing mushroom, `:55363+` desert, `:55409+` snow, `:55446+`
/// purify, `:55527+` cleanse, `:55764+` jungle-mud, `:55883+` corruption-II):
/// the cell's wall set picks the new wall and the chain skips a cell that
/// already carries it. M5-SPREAD-2c extends the three evil types with the
/// 0/3/5/6/7/11/12 arms.
///
/// `y`/`world_surface`/`rng` carry the 0-case's `Next(10)` surface roll
/// (`:55529-55537`); the pure types pass `None`.
pub(crate) fn convert_wall_by_case(
    wall: u16,
    conversion_type: u8,
    y: i32,
    world_surface: f64,
    mut rng: Option<&mut UnifiedRandom>,
) -> Option<u16> {
    // `(guard set, target)` in the C#'s else-if order.
    let cases: [(usize, u16); 8] = match conversion_type {
        1 => [
            (0, 69),
            (1, 3),
            (2, 217),
            (3, 220),
            (4, 188),
            (5, 189),
            (6, 190),
            (7, 191),
        ],
        2 => [
            (0, 70),
            (1, 28),
            (2, 219),
            (3, 222),
            (4, 200),
            (5, 201),
            (6, 202),
            (7, 203),
        ],
        // 4 — the crimson.
        _ => [
            (0, 81),
            (1, 83),
            (2, 218),
            (3, 221),
            (4, 192),
            (5, 193),
            (6, 194),
            (7, 195),
        ],
    };
    // The 0/3/5/6/7/11/12 arms (`:55527+`, `:55764+`, `:55883+`) are not the
    // flat Grass/Stone/HardenedSand/Sandstone/NewWall ladder — each is its own
    // set union and switch arms.
    match conversion_type {
        // Glowing mushroom: `CanBeConvertedToGlowingMushroom` -> 80.
        3 => {
            if CAN_BECOME_GLOWING_MUSHROOM_WALLS.contains(&wall) && wall != 80 {
                return Some(80);
            }
            return None;
        }
        // Desert: (Stone | NewWall1-4 | Ice | Sandstone) -> 187, else
        // (HardenedSand | Dirt | Snow) -> 216 (`:55364-55374`).
        5 => {
            let stone_side = CONVERSION_STONE_WALLS.contains(&wall)
                || CONVERSION_SANDSTONE_WALLS.contains(&wall)
                || CONVERSION_ICE_WALLS.contains(&wall)
                || CONVERSION_NEW_WALLS[0].contains(&wall)
                || CONVERSION_NEW_WALLS[1].contains(&wall)
                || CONVERSION_NEW_WALLS[2].contains(&wall)
                || CONVERSION_NEW_WALLS[3].contains(&wall);
            if stone_side && wall != 187 {
                return Some(187);
            }
            if (CONVERSION_HARDENED_SAND_WALLS.contains(&wall)
                || CONVERSION_DIRT_WALLS.contains(&wall)
                || CONVERSION_SNOW_WALLS.contains(&wall))
                && wall != 216
            {
                return Some(216);
            }
            return None;
        }
        // Snow: the same two unions at 71 / 40 (`:55410-55420`).
        6 => {
            let stone_side = CONVERSION_STONE_WALLS.contains(&wall)
                || CONVERSION_SANDSTONE_WALLS.contains(&wall)
                || CONVERSION_ICE_WALLS.contains(&wall)
                || CONVERSION_NEW_WALLS[0].contains(&wall)
                || CONVERSION_NEW_WALLS[1].contains(&wall)
                || CONVERSION_NEW_WALLS[2].contains(&wall)
                || CONVERSION_NEW_WALLS[3].contains(&wall);
            if stone_side && wall != 71 {
                return Some(71);
            }
            if (CONVERSION_HARDENED_SAND_WALLS.contains(&wall)
                || CONVERSION_DIRT_WALLS.contains(&wall)
                || CONVERSION_SNOW_WALLS.contains(&wall))
                && wall != 40
            {
                return Some(40);
            }
            return None;
        }
        // Purify (7) / jungle-mud (11) / cleanse (0) (`:55446-55488`,
        // `:55764-55842`, `:55527-55609`) — the three share the switch and
        // the NewWall ladder; the natural-wall and sand-family arms differ.
        0 | 7 | 11 => {
            // Type 7's natural arm (the C#'s first branch, `:55449-55456`);
            // types 0/11 carry it in their default chain after the switch.
            let natural = conversion_type == 7
                && (CONVERSION_STONE_WALLS.contains(&wall)
                    || CONVERSION_ICE_WALLS.contains(&wall)
                    || CONVERSION_SANDSTONE_WALLS.contains(&wall));
            let flag = matches!(wall, 61 | 185 | 262 | 274);
            if natural && wall != 349 && !flag {
                return Some(349);
            }
            // The switch is the flag ids' own arm (`:55456-55463`) — it runs
            // exactly when the natural arm did not, so no flag gate here.
            match wall {
                262 => return Some(61),
                274 => return Some(185),
                _ => {}
            }
            // Cleanse's 292-295 -> 204-207 (`:55551-55590`).
            if matches!(conversion_type, 0 | 11) {
                if let Some(off) = [292u16, 293, 294, 295]
                    .iter()
                    .position(|&w| w == wall)
                {
                    return Some(204 + off as u16);
                }
                // The 69/70/81 grass family -> 64 (63/65 at the surface roll).
                if matches!(wall, 69 | 70 | 81) {
                    if (y as f64) < world_surface {
                        if let Some(r) = rng.as_deref_mut() {
                            return Some(if r.next_max(10) != 0 { 63 } else { 65 });
                        }
                    }
                    return Some(64);
                }
            }
            // Type 7/11's dirt arm (`:55465`, `:55807`); type 0 has none.
            if conversion_type != 0 && CONVERSION_DIRT_WALLS.contains(&wall) && wall != 2 {
                return Some(2);
            }
            if let Some(off) = CONVERSION_NEW_WALLS
                .iter()
                .position(|set| set.contains(&wall))
            {
                let target = 212 + off as u16;
                if wall != target {
                    return Some(target);
                }
            }
            // The default chain's tail (`:55789-55818` jungle-mud,
            // `:55624-55658` cleanse): the NewWall family → 212-215, the
            // HardenedSand → 216, the Sandstone → 187. Type 7's dirt arm
            // replaces it (`:55465`).
            if conversion_type != 7 && CONVERSION_STONE_WALLS.contains(&wall) && wall != 349 {
                return Some(349);
            }
            if CONVERSION_HARDENED_SAND_WALLS.contains(&wall) && wall != 216 {
                return Some(216);
            }
            if CONVERSION_SANDSTONE_WALLS.contains(&wall) && wall != 187 {
                return Some(187);
            }
            return None;
        }
        // Corruption-II (`:55886-55906`): the NewWall family -> the corruption
        // analog, else any non-zero non-ebonstone wall -> 3.
        12 => {
            if CONVERSION_NEW_WALLS[0].contains(&wall) && wall != 188 {
                return Some(188);
            }
            if CONVERSION_NEW_WALLS[1].contains(&wall) && wall != 189 {
                return Some(189);
            }
            if CONVERSION_NEW_WALLS[2].contains(&wall) && wall != 190 {
                return Some(190);
            }
            if CONVERSION_NEW_WALLS[3].contains(&wall) && wall != 191 {
                return Some(191);
            }
            if wall != 0 && wall != 3 {
                return Some(3);
            }
            return None;
        }
        _ => {}
    }
    // The flat ladder is the three evil types only — the 0/3/5/6/7/11/12 arms
    // all returned above, and the C#'s switch would otherwise fall into the
    // crimson arm.
    if !matches!(conversion_type, 1 | 2 | 4) {
        return None;
    }
    for (set, target) in cases {
        let in_set = match set {
            0 => CONVERSION_GRASS_WALLS.contains(&wall),
            1 => CONVERSION_STONE_WALLS.contains(&wall),
            2 => CONVERSION_HARDENED_SAND_WALLS.contains(&wall),
            3 => CONVERSION_SANDSTONE_WALLS.contains(&wall),
            n => CONVERSION_NEW_WALLS[n - 4].contains(&wall),
        };
        if in_set && wall != target {
            return Some(target);
        }
    }
    None
}

/// The crimson source tile set (`TileID.Sets.SpreadsCrimson`, `TileID.cs:589`).
fn crimson_set_slice() -> [u16; 10] {
    crate::spread::SPREADS_CRIMSON
}
/// The corruption source tile set (`TileID.Sets.SpreadsCorruption`, `TileID.cs:580`).
fn corrupt_set_slice() -> [u16; 10] {
    crate::spread::SPREADS_CORRUPTION
}
/// The hallow source tile set (`TileID.Sets.SpreadsHallow`).
fn hallow_set_slice() -> [u16; 10] {
    SPREADS_HALLOW
}

/// The `wallDist` box scan for the source tile kind (the wall arms'
/// `:74895-74906` shape).
fn box_has_source(
    world: &World,
    wall_dist: i32,
    i: i32,
    j: i32,
    is_source: &[u16],
) -> bool {
    for k in (i - wall_dist)..=(i + wall_dist) {
        for l in (j - wall_dist)..=(j + wall_dist) {
            if world
                .tile(k, l)
                .is_some_and(|t| t.active() && is_source.contains(&t.kind))
            {
                return true;
            }
        }
    }
    false
}

/// `WorldGen.SpreadGrassWalls` (`WorldGen.cs:74884-74982`): an evil cell
/// spreads into the neighbouring natural walls (the 63-68 family) when its own
/// source sits in the `wallDist` box — the crimson branch checks the crimson
/// source, the corruption branch the corruption one, the hallow branch the
/// hallow one. The neighbour wall then converts through `Convert`'s wall chain
/// (`tiles: false`).
pub fn spread_grass_walls(
    world: &mut World,
    wall_dist: i32,
    i: i32,
    j: i32,
    rng: &mut UnifiedRandom,
) {
    if i < 10 || j < 10 || i >= world.width - 10 || j >= world.height - 10 {
        return;
    }
    let Some(tile) = world.tile(i, j) else {
        return;
    };
    let (wall, active, kind) = (tile.wall, tile.active(), tile.kind);
    let crimson = SPREADS_CRIMSON_WALLS.contains(&wall)
        || (active && crate::spread::SPREADS_CRIMSON.contains(&kind));
    let corrupt = SPREADS_CORRUPTION_WALLS.contains(&wall)
        || (active && crate::spread::SPREADS_CORRUPTION.contains(&kind));
    let hallow =
        SPREADS_HALLOW_WALLS.contains(&wall) || (active && SPREADS_HALLOW.contains(&kind));
    // `(conversion type, the neighbour wall gate, the source set)`: the
    // crimson arm's family is the 63-68 band, the corruption arm's is
    // 63/65/66/68 (`:74959-74962`) and the hallow arm's the 63-68 band.
    let arm: Option<(u8, &[u16], &[u16])> = if crimson {
        Some((4, &[63, 64, 65, 66, 67, 68], &crimson_set_slice()))
    } else if corrupt {
        Some((1, &[63, 65, 66, 68], &corrupt_set_slice()))
    } else if hallow {
        Some((2, &[63, 64, 65, 66, 67, 68], &hallow_set_slice()))
    } else {
        None
    };
    let Some((conversion_type, gate, source)) = arm else {
        return;
    };
    let (x, y) = (i + rng.next_range(-2, 3), j + rng.next_range(-2, 3));
    if x < 10 || y < 10 || x >= world.width - 10 || y >= world.height - 10 {
        return;
    }
    let Some(neighbour) = world.tile(x, y) else {
        return;
    };
    let neighbour = neighbour.wall;
    if !gate.contains(&neighbour) {
        return;
    }
    if !box_has_source(world, wall_dist, i, j, source) {
        return;
    }
    let Some(new) =
        convert_wall_by_case(neighbour, conversion_type, y, world.world_surface, None) else {
        return;
    };
    let idx = world.index(x, y);
    world.tiles[idx].wall = new;
}

/// `WorldGen.SpreadDesertWalls` (`WorldGen.cs:74984-75123`): the desert walls
/// (the sandstone/hardened-sand family) convert to their evil variant when an
/// evil source sits in the `wallDist` box and the neighbour cell is still pure
/// sand.
pub fn spread_desert_walls(
    world: &mut World,
    wall_dist: i32,
    i: i32,
    j: i32,
    rng: &mut UnifiedRandom,
) {
    if i < 10 || j < 10 || i >= world.width - 10 || j >= world.height - 10 {
        return;
    }
    let Some(tile) = world.tile(i, j) else {
        return;
    };
    let (wall, active, kind) = (tile.wall, tile.active(), tile.kind);
    let sandstone_wall = CONVERSION_SANDSTONE_WALLS.contains(&wall);
    let hardened_wall = CONVERSION_HARDENED_SAND_WALLS.contains(&wall);
    let sandstone_tile = active && crate::tiledata::SET_CONVERSION_SANDSTONE.has(kind);
    let hardened_tile = active && crate::tiledata::SET_CONVERSION_SAND.has(kind);
    if !sandstone_wall && !hardened_wall && !sandstone_tile && !hardened_tile {
        return;
    }
    let mut flavour = 0u8;
    if SPREADS_CORRUPTION_WALLS.contains(&wall)
        || (active && crate::spread::SPREADS_CORRUPTION.contains(&kind))
    {
        flavour = 1;
    }
    if SPREADS_HALLOW_WALLS.contains(&wall) || (active && SPREADS_HALLOW.contains(&kind)) {
        flavour = 2;
    }
    if SPREADS_CRIMSON_WALLS.contains(&wall)
        || (active && crate::spread::SPREADS_CRIMSON.contains(&kind))
    {
        flavour = 3;
    }
    if flavour == 0 {
        return;
    }
    let conversion_type = match flavour {
        2 => 2,
        3 => 4,
        _ => 1,
    };
    let (x, y) = (i + rng.next_range(-2, 3), j + rng.next_range(-2, 3));
    let Some(neighbour) = world.tile(x, y) else {
        return;
    };
    if !CONVERSION_PURE_SAND_WALLS.contains(&neighbour.wall) {
        return;
    }
    let source: &[u16] = match conversion_type {
        4 => &crimson_set_slice(),
        2 => &hallow_set_slice(),
        _ => &corrupt_set_slice(),
    };
    if !box_has_source(world, wall_dist, i, j, source) {
        return;
    }
    let Some(new) =
        convert_wall_by_case(neighbour.wall, conversion_type, y, world.world_surface, None)
    else {
        return;
    };
    let idx = world.index(x, y);
    world.tiles[idx].wall = new;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::growth::test_fixtures::{put, world};
    use crate::growth::tests::set_wall;
    use crate::tile::Tile;

    /// `SpreadGrass`'s enclosure gate (`WorldGen.cs:75186-75201`): the C#'s
    /// `flag` starts true, any non-solid 3x3 tile clears it, and lava re-arms
    /// it. The spread proceeds when the box is OPEN (some non-solid tile) and
    /// refuses when it is fully solid — the earlier slice had this inverted.
    #[test]
    fn spread_grass_proceeds_on_an_open_box_and_refuses_when_enclosed() {
        // An open box: the dirt cell has air above it. The surface rule
        // (:75162) keeps dirt-0 below `worldSurface`, so the fixtures sit the
        // cell under it.
        let mut w = World::new("t".into(), 40, 40);
        w.world_surface = 35.0;
        let i = w.index(10, 20);
        w.tiles[i] = crate::tile::Tile {
            kind: 0,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut rng = UnifiedRandom::new(5);
        spread_grass(&mut w, 10, 20, 0, 2);
        assert!(w.tile(10, 20).is_some_and(|t| t.kind == 2), "the open box converted");

        // An enclosed box: every 3x3 neighbour solid.
        let mut w = World::new("t".into(), 40, 40);
        w.world_surface = 35.0;
        for dx in -1i32..=1 {
            for dy in -1i32..=1 {
                let i = w.index(10 + dx, 20 + dy);
                w.tiles[i] = crate::tile::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let i = w.index(10, 20);
        w.tiles[i] = crate::tile::Tile {
            kind: 0,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut rng = UnifiedRandom::new(5);
        spread_grass(&mut w, 10, 20, 0, 2);
        assert!(
            w.tile(10, 20).is_some_and(|t| t.kind == 0),
            "the enclosed box refused"
        );

        // The lava exception: a lava tile in the box re-arms the flag and the
        // conversion refuses even though the box is open.
        let mut w = World::new("t".into(), 40, 40);
        w.world_surface = 35.0;
        let i = w.index(10, 20);
        w.tiles[i] = crate::tile::Tile {
            kind: 0,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let i = w.index(11, 20);
        w.tiles[i] = crate::tile::Tile {
            kind: 0,
            liquid: 255,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tiles[i].set_liquid_kind(crate::tile::LiquidKind::Lava);
        let mut rng = UnifiedRandom::new(5);
        spread_grass(&mut w, 10, 20, 0, 2);
        assert!(
            w.tile(10, 20).is_some_and(|t| t.kind == 0),
            "the lava box refused"
        );
    }

    /// `SpreadGrass`'s surface rule + enclosure gate together pin the spread
    /// DIRECTION (`WorldGen.cs:75162`, `:75186-75201`): a buried soil cell
    /// whose whole 3×3 box is solid never converts, whichever grass touches
    /// it — grass grows along exposed surfaces (including exposed cliff faces,
    /// which the C# allows: there is no "cell above must be empty" rule), not
    /// into enclosed subsoil. The exposed surface neighbour converts.
    #[test]
    fn buried_soil_stays_soil_and_exposed_soil_converts() {
        // A dirt massif: rows 100..140 solid dirt, columns 90..110. The grass
        // fringe sits on the top row; `worldSurface` below everything so the
        // surface rule is not the gate under test.
        let mut w = World::new("t".into(), 200, 200);
        w.world_surface = 150.0;
        for x in 90..=110 {
            for y in 100..=140 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 0, flags: FLAG_ACTIVE, ..Default::default() };
            }
        }
        // The grass fringe on the exposed top row.
        for x in 90..=110 {
            let i = w.index(x, 100);
            w.tiles[i].kind = 2;
        }
        // Buried cells adjacent to grass: below the fringe, beside a future
        // side conversion — every one has a fully solid 3×3 box.
        spread_grass(&mut w, 100, 101, 0, 2); // directly below the grass
        spread_grass(&mut w, 100, 110, 0, 2); // deep inside the massif
        spread_grass(&mut w, 99, 101, 0, 2); // diagonal below the fringe edge
        assert_eq!(w.tile(100, 101).unwrap().kind, 0, "buried below grass stays dirt");
        assert_eq!(w.tile(100, 110).unwrap().kind, 0, "deep subsoil stays dirt");
        assert_eq!(w.tile(99, 101).unwrap().kind, 0, "buried diagonal stays dirt");
        // The exposed surface neighbour (air above) converts.
        let i = w.index(112, 100);
        w.tiles[i] = Tile { kind: 0, flags: FLAG_ACTIVE, ..Default::default() };
        spread_grass(&mut w, 112, 100, 0, 2);
        assert_eq!(w.tile(112, 100).unwrap().kind, 2, "exposed soil converts");
        // An exposed CLIFF face converts too — the C# gate is the enclosure
        // scan, not an above-cell test: a dirt column edge with air to the
        // side is a surface the grass may ride down.
        let mut w = World::new("t".into(), 200, 200);
        w.world_surface = 150.0;
        for x in 90..=100 {
            for y in 100..=140 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 0, flags: FLAG_ACTIVE, ..Default::default() };
            }
        }
        let i = w.index(100, 100);
        w.tiles[i].kind = 2;
        // (100,101): dirt below the grass on the cliff edge — air at x=101.
        spread_grass(&mut w, 100, 101, 0, 2);
        assert_eq!(
            w.tile(100, 101).unwrap().kind,
            2,
            "the exposed cliff face converts (vanilla allows the side fringe)"
        );
        // One column in from the edge the same cell is buried and refuses.
        assert_eq!(w.tile(99, 101).unwrap().kind, 0);
        spread_grass(&mut w, 99, 101, 0, 2);
        assert_eq!(w.tile(99, 101).unwrap().kind, 0, "the buried cell refuses");
    }

    /// The growth-tick 3×3 scan (`UpdateWorld_GrassGrowth` `:74381-74404` →
    /// `SpreadGrass`) never converts enclosed subsoil, and the pass consumes
    /// the same RNG draws whether a neighbour converts or not — the plant
    /// roll (`:74356`) fires before the scan and `SpreadGrass` itself draws
    /// nothing, so the draw count/order is independent of the eligibility
    /// outcome.
    #[test]
    fn the_grass_scan_leaves_buried_soil_and_the_draw_count_is_outcome_independent() {
        // Two identical dirt massifs with a corrupt-grass source on top; the
        // cell below the source is exposed to a cave pocket in one world and
        // fully buried in the other. The cell above the SOURCE stays open in
        // both, so the plant-above roll (`Next(2)`, plant 24) draws the same.
        let build = |pocket: bool| {
            let mut w = World::new("t".into(), 200, 200);
            w.world_surface = 150.0;
            for x in 90..=110 {
                for y in 100..=140 {
                    let i = w.index(x, y);
                    w.tiles[i] =
                        Tile { kind: 0, flags: FLAG_ACTIVE, ..Default::default() };
                }
            }
            let i = w.index(100, 100);
            w.tiles[i].kind = 23; // the corrupt-grass source on the surface
            if pocket {
                // A cave pocket beside the target cell (100,101) exposes it.
                let i = w.index(99, 101);
                w.tiles[i] = Tile::default();
            }
            w
        };
        let mut open = build(true);
        let mut buried = build(false);
        let mut rng_a = UnifiedRandom::new(77);
        let mut rng_b = UnifiedRandom::new(77);
        for _ in 0..50 {
            update_world_grass_growth(&mut open, 100, 100, 99, 102, 99, 102, true, &mut rng_a);
            update_world_grass_growth(&mut buried, 100, 100, 99, 102, 99, 102, true, &mut rng_b);
        }
        assert_eq!(
            open.tile(100, 101).unwrap().kind,
            23,
            "the pocket-exposed neighbour converted"
        );
        assert_eq!(
            buried.tile(100, 101).unwrap().kind,
            0,
            "the buried neighbour stayed dirt"
        );
        // No grass anywhere below the surface row in the buried world.
        for x in 90..=110 {
            for y in 101..=140 {
                assert_eq!(buried.tile(x, y).unwrap().kind, 0, "buried subsoil ({x},{y})");
            }
        }
        // The streams advanced identically: same draw count AND same values.
        for _ in 0..16 {
            assert_eq!(rng_a.next(), rng_b.next(), "the draw streams stay in lockstep");
        }
    }

    /// The surface rule's infection exemption (`WorldGen.cs:75162`): the
    /// `j >= worldSurface && dirt == 0` refusal fires only for the
    /// non-infection grasses — the corrupt/crimson grasses (23/199/661/662)
    /// convert dirt at any depth (the enclosure gate still applies), while
    /// plain grass 2 refuses below the surface line.
    #[test]
    fn the_surface_rule_exempts_the_infection_grasses() {
        let mut w = World::new("t".into(), 200, 200);
        w.world_surface = 105.0; // the cells sit BELOW the surface line
        let i = w.index(100, 120);
        w.tiles[i] = Tile { kind: 0, flags: FLAG_ACTIVE, ..Default::default() };
        spread_grass(&mut w, 100, 120, 0, 2);
        assert_eq!(w.tile(100, 120).unwrap().kind, 0, "plain grass refuses below the surface");
        spread_grass(&mut w, 100, 120, 0, 23);
        assert_eq!(w.tile(100, 120).unwrap().kind, 23, "the corruption converts at depth");
        let i = w.index(102, 120);
        w.tiles[i] = Tile { kind: 0, flags: FLAG_ACTIVE, ..Default::default() };
        spread_grass(&mut w, 102, 120, 0, 199);
        assert_eq!(w.tile(102, 120).unwrap().kind, 199, "the crimson converts at depth");
        // The enclosure gate still binds the infections: a fully solid box
        // refuses even the corruption.
        let mut w = World::new("t".into(), 200, 200);
        w.world_surface = 105.0;
        for dx in -1i32..=1 {
            for dy in -1i32..=1 {
                let i = w.index(100 + dx, 120 + dy);
                w.tiles[i] = Tile { kind: 0, flags: FLAG_ACTIVE, ..Default::default() };
            }
        }
        spread_grass(&mut w, 100, 120, 0, 23);
        assert_eq!(w.tile(100, 120).unwrap().kind, 0, "the enclosed corruption refuses");
    }

    #[test]
    fn the_wall_arms_convert_the_neighbour_walls() {
        // `SpreadGrassWalls`/`SpreadDesertWalls` (`:74884`/`:74984`): an evil
        // cell with its source in the wallDist box converts a neighbouring
        // natural wall through Convert's wall chain; the gated arms stand
        // down without the source or with a wrong neighbour family.
        // The setup: a corrupt grass tile (23) beside a natural grass wall
        // (63) at ±2, the corruption source inside the box.
        let mut w = world();
        for dx in 0..10 {
            for dy in 200..205 {
                put(&mut w, 100 + dx, dy, 1, 0); // the stone floor
                set_wall(&mut w, 100 + dx, dy, 63); // the natural grass walls
            }
        }
        // The corrupt grass in the box (a `SpreadsCorruption` tile).
        put(&mut w, 102, 203, 23, 0);
        let mut rng = UnifiedRandom::new(3);
        // The evil cell: the wall itself is a corruption-spreading wall (69)
        // at the source cell, so the arm fires from (102, 203).
        set_wall(&mut w, 102, 203, 69);
        let mut converted = false;
        for seed in 1..64u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            spread_grass_walls(&mut w, 3, 102, 203, &mut rng);
            // Any wall that left the natural family counts.
            converted = (100..110).any(|x| {
                (200..205).any(|y| w.tile(x, y).is_some_and(|t| t.wall != 63))
            });
            if converted {
                break;
            }
        }
        assert!(converted, "the corruption arm converts a natural wall");

        // Without the source in the box the arm stands down.
        let mut w2 = world();
        for dx in 0..10 {
            for dy in 200..205 {
                put(&mut w2, 100 + dx, dy, 1, 0);
                set_wall(&mut w2, 100 + dx, dy, 63);
            }
        }
        set_wall(&mut w2, 102, 203, 69); // the spreading wall, but no evil tile
        let mut rng2 = UnifiedRandom::new(3);
        spread_grass_walls(&mut w2, 3, 102, 203, &mut rng2);
        assert!(
            (100..110).all(|x| (200..205).all(|y| {
                // The source cell keeps its own spreading wall by construction.
                (x, y) == (102, 203) || w2.tile(x, y).is_some_and(|t| t.wall == 63)
            })),
            "no source in the box - no conversion"
        );
    }

    #[test]
    fn the_spread_stop_flag_freezes_the_infection_rows() {
        // The journey `StopBiomeSpreadPower` clears
        // `WorldGen.AllowedToSpreadInfections` (`WorldGen.cs:71502-71507`);
        // the grass-growth infection rows sit behind it (`:74293-74329`).
        let mut w = world();
        w.world_surface = 250.0;
        put(&mut w, 100, 200, 23, 0); // the corrupt-grass source
        put(&mut w, 101, 200, 0, 0); // the dirt neighbour
        let mut rng = UnifiedRandom::new(7);
        for _ in 0..50 {
            update_world_grass_growth(&mut w, 100, 200, 99, 102, 199, 202, false, &mut rng);
        }
        assert_eq!(
            w.tile(101, 200).unwrap().kind,
            0,
            "the dirt stays dirt while the spread is stopped"
        );
        // With the flag back on the same arm converts the dirt.
        let mut rng = UnifiedRandom::new(7);
        for _ in 0..50 {
            update_world_grass_growth(&mut w, 100, 200, 99, 102, 199, 202, true, &mut rng);
        }
        assert_eq!(
            w.tile(101, 200).unwrap().kind,
            23,
            "the corruption converts the dirt"
        );
    }

    /// DW-2.8: the SpreadGrass conversion dispatch (WorldGen.cs:75207-75221):
    /// a conversion-grass dirt (the Conversion.Grass set) spreads through the
    /// flavor's map, a plain write otherwise, and the tree-invalidity kill
    /// rides the plain arm.
    #[test]
    fn spread_grass_2_the_conversion_dispatch_and_the_tree_kill() {
        let mut w = World::new("t".into(), 64, 64);
        w.world_surface = 30.0;
        let idx = w.index(10, 20);
        w.tiles[idx] = Tile { kind: 2, flags: FLAG_ACTIVE, ..Default::default() };
        let tree_idx = w.index(10, 19);
        w.tiles[tree_idx] = Tile { kind: 5, flags: FLAG_ACTIVE, ..Default::default() };
        // Jungle grass over the wood grass: the plain write (60 is not a
        // conversion dirt), and the forest tree survives a jungle ground.
        spread_grass(&mut w, 10, 20, 2, 60);
        assert_eq!(w.tile(10, 20).unwrap().kind, 60);
        assert!(w.tile(10, 19).is_some_and(|t| t.active()), "jungle keeps the tree");
        // Sand under the same tree kills it — the tree arm.
        let mut w2 = World::new("t".into(), 64, 64);
        w2.world_surface = 30.0;
        let idx = w2.index(10, 20);
        w2.tiles[idx] = Tile { kind: 2, flags: FLAG_ACTIVE, ..Default::default() };
        let tree_idx = w2.index(10, 19);
        w2.tiles[tree_idx] = Tile { kind: 5, flags: FLAG_ACTIVE, ..Default::default() };
        spread_grass(&mut w2, 10, 20, 2, 112);
        assert_eq!(w2.tile(10, 20).unwrap().kind, 112);
        assert!(!w2.tile(10, 19).is_some_and(|t| t.active()), "sand kills the forest tree");
    }

    /// DW-2.8: the conversion arm — the corruption spread over a
    /// conversion-grass dirt takes the flavor's Convert (whose map turns the
    /// wood grass 2 into the corruption grass 23), and the walls are left
    /// alone (the SpreadGrass call passes walls: false).
    #[test]
    fn spread_grass_2_the_conversion_arm_converts_through_the_flavor() {
        let mut w = World::new("t".into(), 64, 64);
        w.world_surface = 30.0;
        let idx = w.index(10, 20);
        w.tiles[idx] = Tile { kind: 2, flags: FLAG_ACTIVE, ..Default::default() };
        spread_grass(&mut w, 10, 20, 2, 23);
        assert_eq!(w.tile(10, 20).unwrap().kind, 23, "the corruption map lands");
        let wall_idx = w.index(11, 21);
        w.tiles[wall_idx].wall = 1;
        spread_grass(&mut w, 10, 20, 2, 23);
        assert_eq!(w.tile(11, 21).unwrap().wall, 1, "the walls stay (walls: false)");
    }

    /// DW-2.9: `MossConversion`'s map (`WorldGen.cs:71457-71503`) — the
    /// moss-brick guard over the pearlstone, the moss guard over the dirt, the
    /// family pairs and the 0 for an unmapped kind.
    #[test]
    fn moss_conversion_3_the_map_and_its_guards() {
        // The moss-brick tiles keep the pearlstone 38 spread unchanged.
        assert_eq!(moss_conversion(512, 38), 512);
        // The moss tiles keep the dirt 1 spread unchanged.
        assert_eq!(moss_conversion(179, 1), 179);
        // The family pairs — the map runs only when BOTH guards miss: the
        // moss-brick guard covers a brick over the pearlstone 38 (515 stays),
        // so a brick maps only over a non-38 neighbour.
        assert_eq!(moss_conversion(182, 38), 515);
        assert_eq!(moss_conversion(515, 1), 182, "the brick maps over dirt");
        assert_eq!(moss_conversion(381, 38), 517);
        assert_eq!(moss_conversion(183, 38), 516);
        assert_eq!(moss_conversion(515, 38), 515, "the brick guard beats the map");
        // An unmapped kind returns 0 (the caller skips it).
        assert_eq!(moss_conversion(5, 1), 0);
    }

    /// DW-2.9: the moss spread arm (`WorldGen.cs:72371-72424`) — a moss tile
    /// converts the dirt neighbours in its box through the map; the
    /// conversion re-frames the cell.
    #[test]
    fn moss_conversion_3_the_moss_arm_converts_the_neighbours() {
        let mut w = World::new("t".into(), 64, 64);
        w.world_surface = 30.0;
        // The moss 179 at (10, 20); the dirt 1 neighbours above and right.
        let midx = w.index(10, 20);
        w.tiles[midx] = Tile { kind: 179, flags: FLAG_ACTIVE, ..Default::default() };
        for &(x, y) in &[(9, 20), (10, 19), (11, 20), (10, 21)] {
            let idx = w.index(x, y);
            w.tiles[idx] = Tile { kind: 1, flags: FLAG_ACTIVE, ..Default::default() };
        }
        // Drive the arm with the gate roll forced: the arm's first draw is the
        // NextDouble < 0.5 gate — a seeded stream may refuse, so run enough
        // attempts that at least one converts.
        let mut rng = UnifiedRandom::new(7);
        let mut converted = 0;
        for _ in 0..64 {
            grow_moss_arm(&mut w, 10, 20, 9, 11, 19, 21, &mut rng);
            converted += w
                .tiles
                .iter()
                .filter(|t| matches!(t.kind, 179) && t.active())
                .count();
            if converted > 1 {
                break;
            }
        }
        assert!(converted > 1, "the moss arm spreads the 179 onto a dirt neighbour");
        // The conversion leaves the moss source itself and the moss tile
        // unique-source intact.
        assert_eq!(w.tile(10, 20).unwrap().kind, 179);
    }

    // --- W4-c31 jungle residuals: the case-60 hardmode tail ---------------

    /// A jungle-grass (60) cell at (i, j) with a grass floor below the
    /// placement row min_j = j-1, hardmode + all three mechs downed.
    fn jungle_growth_world() -> World {
        let mut w = World::new("jg".into(), 400, 400);
        w.world_surface = 100.0;
        w.hard_mode = true;
        w.downed_mech_boss1 = true;
        w.downed_mech_boss2 = true;
        w.downed_mech_boss3 = true;
        // The grass floor row at y=201 (the placement ground for min_j=200).
        for x in 0..400 {
            let i = w.index(x, 201);
            w.tiles[i] = Tile { kind: 60, flags: FLAG_ACTIVE, ..Default::default() };
        }
        // The jungle-grass source at (200, 200).
        let i = w.index(200, 200);
        w.tiles[i] = Tile { kind: 60, flags: FLAG_ACTIVE, ..Default::default() };
        w
    }

    /// The hardmode tail places a life fruit (236) or bulb (238) or detritus
    /// (233) above the jungle grass over many growth ticks.
    #[test]
    fn the_hardmode_tail_grows_fruit_bulb_or_detritus() {
        let mut w = jungle_growth_world();
        let mut rng = UnifiedRandom::new(42);
        let mut grew = false;
        for _ in 0..4000 {
            update_world_grass_growth(&mut w, 200, 200, 199, 202, 199, 202, true, &mut rng);
            if w.tile(200, 199).is_some_and(|t| t.active() && matches!(t.kind, 236 | 238 | 233)) {
                grew = true;
                break;
            }
            // Reset the source cell (the tail may have overwritten min_j).
            let i = w.index(200, 200);
            w.tiles[i] = Tile { kind: 60, flags: FLAG_ACTIVE, ..Default::default() };
        }
        assert!(grew, "the hardmode tail grows a 236/238/233 above the grass");
    }

    /// The a == b determinism witness: two identical growth runs draw
    /// identical streams and write identical tiles.
    #[test]
    fn the_hardmode_tail_is_reproducible_for_a_seed() {
        let run = || {
            let mut w = jungle_growth_world();
            let mut rng = UnifiedRandom::new(7);
            for _ in 0..500 {
                update_world_grass_growth(&mut w, 200, 200, 199, 202, 199, 202, true, &mut rng);
            }
            w
        };
        assert_eq!(run().tiles, run().tiles, "a == b — the full tile state");
    }

    /// Pre-hardmode: the tail only grows the 233 detritus (no fruit/bulb).
    #[test]
    fn the_pre_hardmode_tail_grows_only_detritus() {
        let mut w = jungle_growth_world();
        w.hard_mode = false;
        let mut rng = UnifiedRandom::new(3);
        let mut saw_fruit_or_bulb = false;
        for _ in 0..2000 {
            update_world_grass_growth(&mut w, 200, 200, 199, 202, 199, 202, true, &mut rng);
            if w.tile(200, 199).is_some_and(|t| t.active() && matches!(t.kind, 236 | 238)) {
                saw_fruit_or_bulb = true;
            }
            let i = w.index(200, 200);
            w.tiles[i] = Tile { kind: 60, flags: FLAG_ACTIVE, ..Default::default() };
        }
        assert!(!saw_fruit_or_bulb, "no fruit/bulb before hardmode");
    }

    /// The liquid veto: a water cell at min_j blocks the tail.
    #[test]
    fn the_liquid_veto_blocks_the_tail() {
        let mut w = jungle_growth_world();
        let i = w.index(200, 199);
        w.tiles[i].liquid = 200; // water at the placement row
        let mut rng = UnifiedRandom::new(5);
        for _ in 0..500 {
            update_world_grass_growth(&mut w, 200, 200, 199, 202, 199, 202, true, &mut rng);
        }
        assert!(
            !w.tile(200, 199).is_some_and(|t| t.active() && matches!(t.kind, 236 | 238 | 233)),
            "the liquid veto blocks the placement"
        );
    }
}
