//! The micro-biomes — the shape-level ports of the `GameContent/Biomes`
//! micro classes: `CaveHouseBiome` (the underground cabins), `CampsiteBiome`
//! (the desert campsites), `SpikePitBiome` (the jungle spike pits) and
//! `EnchantedSwordBiome` (the sword shrines).
//!
//! Source map (line numbers into `decompiled/Terraria/GameContent/Biomes/`):
//! - `CaveHouseBiome.Place` (`:34-92`): the 25-tile wire/chest exclusion,
//!   the `HouseUtils.CreateBuilder` room — the port carves the wood-framed
//!   room with the platform, the chest and the torches in the underground
//!   dirt/stone band (the C#'s `HouseBuilder` variant walls = the
//!   shape-level approximation)
//! - `CampsiteBiome.Place` (`:9-55`): the disc `Next(6, 10)` radius, the
//!   solid-count gate, the tile 196 + `Next(4)` (the tent) replacing the
//!   sand family {53, 396, 397, 404}
//! - `SpikePitBiome.Place` (`:10-57`): the stone (1) pit — the 4+4+4
//!   shrinking circles offset down, the blob blotches; the port drops the
//!   spike tile placement (the jungle spikes = the follow-up)
//! - `EnchantedSwordBiome.Place` (`:19-113`): the 0/1 tile-scan gate (the
//!   1250 air/dirt cells in the 50×50 box), the 55-tile depth gate, the
//!   no-sand check, the 1-wide shaft down to the shrine chamber; the port
//!   places the shaft + the chamber with the sword item tile 197 (the C#'s
//!   sword sprite tile — the breakable item drop = the follow-up)
//!
//! Deviations (ANALYSIS "Micro-biomes (M1.4-WGEN-8)"): the C#'s
//! `HouseBuilder` room variants (the granite/marble/mushroom/ice/jungle
//! chest chances) are the shape-level approximation; the spike tiles and
//! the sword item drop are the follow-ups; the structure-map exclusions
//! are the per-pass neighbour scans.

use super::structures::place_chest;
use crate::tile::{FLAG_ACTIVE, Tile};
use crate::world::World;

/// `WorldGen.SolidTile` (`WorldGen.cs:70111-70131`) — the same fold the
/// `evil` pass's copy carries (out-of-bounds reads as solid, no
/// `tileSolidTop`, no half-brick, no slope).
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && crate::tiledata::TILE_SOLID.has(t.kind)
        && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && (t.flags & crate::tile::FLAG_INACTIVE) == 0
}

fn active_at(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| t.active())
}

fn clear(world: &mut World, x: i32, y: i32, wall: u16) {
    if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
        return;
    }
    let i = world.index(x, y);
    world.tiles[i] = Tile::default();
    world.tiles[i].wall = wall;
}

fn put(world: &mut World, x: i32, y: i32, kind: u16) {
    if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
        return;
    }
    let i = world.index(x, y);
    let t = &mut world.tiles[i];
    t.kind = kind;
    t.flags |= FLAG_ACTIVE;
    t.liquid = 0;
    t.set_slope(0);
}

/// M5-SEED-2 `SecretSeed.GenerateBiggerAbandonedHouses`
/// (`WorldGen.cs:431-447`): `biggerAbandonedHouses` makes every cave house a
/// big one; the vanilla fallback is a 1-in-3 roll and only under
/// `errorWorld`:
///
/// ```text
/// if (!SecretSeed.biggerAbandonedHouses.Enabled) {
///     if (SecretSeed.errorWorld.Enabled) return WorldGen.genRand.Next(3) == 0;
///     return false;
/// }
/// return true;
/// ```
///
/// The port models neither `biggerAbandonedHouses`'s sibling `errorWorld` (its
/// own M5-SEED-2 seed) nor the 1-in-3 fallback (an extra draw would shift the
/// classic stream), so the gate is the micro flag alone. Downstream the C#
/// takes the bigger path in `HouseUtils.CreateRooms` (`:88-91` → the
/// `CreateRooms_BigAbandonedHouses` chain, `:47-84`) plus the deeper origin
/// clamp in the pass body (`WorldGen.cs:17070-17081`).
fn bigger_abandoned_houses(world: &World) -> bool {
    world
        .micro_seeds
        .contains(crate::seeds::MicroSeedFlags::BIGGER_ABANDONED_HOUSES)
}

/// `CaveHouseBiome` (`:34-92`): the underground cabin — the wood frame, the
/// carved interior with the background walls, the wood platform, the chest
/// and the torches. The chest-chance variants (the ice/jungle/granite/
/// marble/mushroom/desert) resolve to the plain gold chest (the port's
/// chests hold no inventories yet).
///
/// M5-SEED-2 bigger houses (`WorldGen.cs:17070-17081`, `HouseUtils.cs:88-91`,
/// `:47-84`, `HouseBuilder.cs:637`): under `GenerateBiggerAbandonedHouses`
/// the pass always sizes up.
/// * `WorldGen.cs:17070-17081`: the origin's depth is clamped at least
///   `GetMaxPossibleRoomsInABigAbandonedHouse() * 10` = 70 tiles below the
///   band floor — the chained rooms need the headroom.
/// * `HouseUtils.cs:60-84` (`FindRoom_BigAbandonedHouses`): each room is a
///   `Next(15, 31)` wide × `Next(8, 13)` tall box, against `FindRoom`'s
///   `Clamp(…, 15, 30)` × `Clamp(…, 8, 12)`, and up to
///   `GetRandomizedRoomCountInABigAbandonedHouse()` = `7 - Next(4)` rooms
///   chain vertically against the vanilla one-plus-two-gated shape.
/// * `HouseBuilder.cs:637`: the vertical exits find solid ground over the full
///   50-tile drop (vanilla's `num7 < 50` short-circuit is disabled), so a
///   bigger house's rooms connect through the deeper stone.
///
/// The port carves one room per site, so the wider/deeper bounds map onto the
/// `hw`/`hh` rolls and the depth clamp onto the y band; a classic world takes
/// exactly the vanilla rolls.
pub(crate) fn pass_cave_houses(
    world: &mut World,
    rock_layer: f64,
    rng: &mut super::Rng,
) -> u32 {
    let count = 8 + rng.range(0, 5) as i32;
    let mut placed = 0u32;
    let bigger = bigger_abandoned_houses(world);
    let y_lo = world.world_surface as i32 + 40;
    let y_hi = (rock_layer as i32 + 260).min(world.height - 120);
    // `WorldGen.cs:17070-17081`: the big-house origin is pushed at least
    // `GetMaxPossibleRoomsInABigAbandonedHouse() * 10` = 70 tiles deeper —
    // the model's `y_lo` is the C#'s `worldSurfaceHigh + 20` band floor.
    let y_lo = if bigger { y_lo + 70 } else { y_lo };
    if y_hi <= y_lo {
        return placed;
    }
    for _ in 0..count {
        let x = rng.range(60, world.width - 60);
        // The spawn exclusion.
        if (x - world.width / 2).abs() < 120 {
            continue;
        }
        let y = rng.range(y_lo, y_hi);
        // The wider/deeper room bounds (`HouseUtils.cs:158-163`): the big
        // house's room is `Next(15, 31)` × `Next(8, 13)` against the vanilla
        // `Clamp(…, 15, 30)` × `Clamp(…, 8, 12)`, so the port's rolls grow by
        // one tile on each side. Classic worlds keep `range(5, 9)` /
        // `range(3, 5)` exactly.
        let (hw, hh) = if bigger {
            (rng.range(5, 10), rng.range(3, 6))
        } else {
            (rng.range(5, 9), rng.range(3, 5))
        };
        // `:38-52`: the 25-tile exclusion — the chests and the wires nearby
        // reject the spot.
        let mut blocked = false;
        for dx in -25..=25i32 {
            for dy in -25..=25i32 {
                if world
                    .tile(x + dx, y + dy)
                    .is_some_and(|t| t.active() && t.kind == 21)
                {
                    blocked = true;
                    break;
                }
            }
            if blocked {
                break;
            }
        }
        if blocked {
            continue;
        }
        // The ground must be the dirt/stone at the spot.
        if !world
            .tile(x, y)
            .is_some_and(|t| t.active() && matches!(t.kind, 0 | 1))
        {
            continue;
        }
        // The frame: the wood box.
        for px in (x - hw)..=(x + hw) {
            for py in (y - hh)..=(y + hh) {
                put(world, px, py, 27); // `TileID.WoodBlock`? — the wood frame tile
            }
        }
        // The interior: carved with the wood walls.
        for px in (x - hw + 1)..=(x + hw - 1) {
            for py in (y - hh + 1)..=(y + hh - 1) {
                clear(world, px, py, 9);
            }
        }
        // The floor: the wood platform row (the tile 19 — the platform).
        for px in (x - hw + 1)..=(x + hw - 1) {
            let py = y + hh / 2;
            put(world, px, py, 19);
        }
        // The torches on the interior walls.
        for tx in [x - hw + 2, x + hw - 2] {
            let ty = y - hh + 2;
            let i = world.index(tx, ty);
            if !world.tiles[i].active() {
                world.tiles[i] = Tile {
                    kind: 4,
                    ..Default::default()
                };
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        // The chest on the floor.
        let _ = place_chest(world, x - 1, y + hh / 2 - 1);
        placed += 1;
    }
    placed
}

/// `CampsiteBiome` (`:9-55`): the desert campsite — the disc of the radius
/// `Next(6, 10)` replacing the sand family {53, 396, 397, 404} with the
/// tent tiles (196 + `Next(4)`), gated by the solid-count check.
pub(crate) fn pass_campsites(world: &mut World, rng: &mut super::Rng) -> u32 {
    let count = 4 + rng.range(0, 4) as i32;
    let mut placed = 0u32;
    for _ in 0..count {
        let x = rng.range(100, world.width - 100);
        let y = rng.range(world.world_surface as i32, (world.world_surface as i32 + 60).min(world.height - 60));
        // The disc solid-count gate (`:11-17`): the solid cells must be
        // within 5 of the total.
        let mut solid = 0;
        let mut total = 0;
        for dx in -10..=10 {
            for dy in -10..=10 {
                total += 1;
                if active_at(world, x + dx, y + dy) {
                    solid += 1;
                }
            }
        }
        if solid < total - 5 {
            continue;
        }
        let radius = rng.range(6, 10);
        let tent = 196 + rng.range(0, 4) as u16;
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let px = x + dx;
                let py = y + dy;
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist > radius as f32 {
                    continue;
                }
                if let Some(t) = world.tile(px, py) {
                    if t.active() && matches!(t.kind, 53 | 396 | 397 | 404) {
                        put(world, px, py, tent);
                    }
                }
            }
        }
        placed += 1;
    }
    placed
}

/// `SpikePitBiome` (`:10-57`): the jungle stone pit — the shrinking
/// circle blobs offset down (the C#'s 4+4+4 circle walk); the spike tile
/// placement is the follow-up.
pub(crate) fn pass_spike_pits(
    world: &mut World,
    spans: &super::biomes::BiomeSpans,
    rock_layer: f64,
    rng: &mut super::Rng,
) -> u32 {
    if spans.jungle_end <= spans.jungle_start {
        return 0;
    }
    let count = 3 + rng.range(0, 4) as i32;
    let mut placed = 0u32;
    let y_lo = rock_layer as i32;
    let y_hi = (rock_layer as i32 + 200).min(world.height - 120);
    if y_hi <= y_lo {
        return placed;
    }
    for _ in 0..count {
        let cx_lo = spans.jungle_start + 20;
        let cx_hi = (spans.jungle_end - 20).max(cx_lo + 1);
        let cx = rng.range(cx_lo, cx_hi);
        let cy = rng.range(y_lo, y_hi);
        // The 4 shrinking circles offset down (the pit walls).
        let mut r0 = rng.range(8, 10);
        for step in 0..4 {
            let r = r0 + step;
            let oy = 5 * step + 5;
            for dx in -r..=r {
                for dy in -r..=r {
                    let dist = ((dx * dx + dy * dy) as f32).sqrt();
                    if dist <= r as f32 {
                        put(world, cx + dx, cy + oy + dy, 1);
                    }
                }
            }
            r0 -= 1;
        }
        // The pit hollow: the inner circles carved.
        let mut r1 = rng.range(6, 7);
        for step in 0..4 {
            let r = r1 + step;
            let oy = 2 * step + 12;
            for dx in -r..=r {
                for dy in -r..=r {
                    let dist = ((dx * dx + dy * dy) as f32).sqrt();
                    if dist <= r as f32 {
                        clear(world, cx + dx, cy + oy + dy, super::biomes::WALL_JUNGLE);
                    }
                }
            }
            r1 -= 1;
        }
        // The spikes (48) on the inner rim touching the air (`:53-55`: the
        // IsTouchingAir cells, the checkerboard 2 spacing).
        for dx in -r1..=r1 {
            for dy in -r1..=r1 {
                let px = cx + dx;
                let py = cy + 12 + dy;
                if (dx + dy) % 2 != 0 {
                    continue;
                }
                if active_at(world, px, py) {
                    continue;
                }
                // The cell touches the pit's stone (the wall or the floor).
                let touching = [
                    (px - 1, py),
                    (px + 1, py),
                    (px, py - 1),
                    (px, py + 1),
                ]
                .iter()
                .any(|(nx, ny)| active_at(world, *nx, *ny));
                if touching {
                    put(world, px, py, 48);
                }
            }
        }
        placed += 1;
    }
    placed
}

/// `EnchantedSwordBiome` (`:19-113`): the sword shrine — the 1-wide shaft
/// down from the surface into the buried chamber with the sword tile (197,
/// the C#'s sword sprite tile; the breakable item drop = the follow-up).
pub(crate) fn pass_sword_shrines(
    world: &mut World,
    surface: &[i32],
    rock_layer: f64,
    desert: (i32, i32),
    rng: &mut super::Rng,
) -> u32 {
    let count = 2 + rng.range(0, 3) as i32;
    let mut placed = 0u32;
    for _ in 0..count {
        let x = rng.range(100, world.width - 100);
        // The desert band skips (the C#'s tile-scan gate `:24-30` requires
        // 1250 of the tiles 0/1 in the 50×50 box — the desert's sand 53
        // fails it, so the shrines never generate there).
        if x >= desert.0 && x < desert.1 {
            continue;
        }
        // The depth gate (`:32-38`): the origin 55+ tiles under the
        // surface.
        let ground = surface[x as usize];
        if ground < 55 {
            continue;
        }
        let shaft_top = ground - rng.range(10, 30);
        // The 1-wide shaft down to the chamber.
        let depth = rng.range(8, 14);
        let chamber_y = shaft_top + depth;
        if chamber_y as f64 > rock_layer + 200.0 {
            continue;
        }
        for py in shaft_top..chamber_y {
            clear(world, x, py, 0);
        }
        // The chamber: the 7×5 hollow with the stone frame.
        for px in (x - 3)..=(x + 3) {
            for py in chamber_y..(chamber_y + 5) {
                if px == x - 3 || px == x + 3 || py == chamber_y + 4 {
                    put(world, px, py, 1);
                } else {
                    clear(world, px, py, 0);
                }
            }
        }
        // The sword tile in the chamber's centre (`:100-108`): the
        // `ChanceOfRealSword` config value is 1.0 — the real sword tile 187
        // always (the break drops the Enchanted Sword item 989); the fake
        // tile 186 arm is dead with this config.
        put(world, x, chamber_y + 2, 187);
        placed += 1;
    }
    placed
}

/// `HoneyPatchBiome.Place` (`HoneyPatchBiome.cs:9-58`): the honey pocket
/// beside a hive — the 15-disc scan gate (75 % of the solids are mud/hive
/// and 2+ hive tiles), the 8-disc solid gate (20+), the important-location
/// scan, then the radial-dithered honey-block fill, the 3-tall honey
/// pocket, the mud rim and the honey cap.
///
/// The `structures` map's `CanPlace`/`AddProtectedStructure` pair rides the
/// caller's `protected` rect list (the port's structure-map stand-in, the
/// same neighbour-rect idiom the other micro passes document).
pub(crate) fn place_honey_patch(
    world: &mut World,
    ox: i32,
    oy: i32,
    protected: &mut Vec<(i32, i32, i32, i32)>,
    rng: &mut super::Rng,
) -> bool {
    // `:11-13`: the origin must not be an active solid tile.
    if world.tile(ox, oy).is_some_and(|t| t.active()) && solid_tile(world, ox, oy) {
        return false;
    }
    // `:14-18`: the down-80 IsSolid find, the result 2 tiles lower.
    let mut py = oy;
    let mut found = None;
    for _ in 0..80 {
        py += 1;
        if solid_tile(world, ox, py) {
            found = Some(py);
            break;
        }
    }
    let (mut px, mut py) = match found {
        Some(y) => (ox, y + 2),
        None => return false,
    };
    // `:19-25`: the 15-disc scan — the solids, the mud/hive count and the
    // hive count; the gates are 75 % and 2+.
    let (mut solid, mut mud_hive, mut hive) = (0i32, 0i32, 0i32);
    for dx in -15..=15 {
        for dy in -15..=15 {
            if (dx * dx + dy * dy) as f32 > 225.0 {
                continue;
            }
            if !active_at(world, px + dx, py + dy) {
                continue;
            }
            solid += 1;
            match world.tile(px + dx, py + dy).map(|t| t.kind) {
                Some(59) | Some(60) => mud_hive += 1,
                _ => {}
            }
            if world.tile(px + dx, py + dy).map(|t| t.kind) == Some(60) {
                hive += 1;
            }
        }
    }
    // The C# gate is `obj2 / obj >= 0.75 && obj3 >= 2` — the hive count
    // (the 60 tiles) must clear 2. A solid-mud disc without hive blocks
    // would pass the ratio but fail the count, so the test's world seeds a
    // 60-tile core.
    if (mud_hive * 4) < (solid * 3) || hive < 2 {
        return false;
    }
    // `:26-31`: the 8-disc solid gate (20+).
    let mut solid8 = 0i32;
    for dx in -8..=8 {
        for dy in -8..=8 {
            if (dx * dx + dy * dy) as f32 > 64.0 {
                continue;
            }
            if active_at(world, px + dx, py + dy) {
                solid8 += 1;
            }
        }
    }
    if solid8 < 20 {
        return false;
    }
    // `:32-35`: the structure map's 16×16 box.
    for &(rx, ry, rw, rh) in protected.iter() {
        if px < rx + rw && px + 16 > rx && py < ry + rh && py + 16 > ry {
            return false;
        }
    }
    // `:60-83`'s TooCloseToImportantLocations: the underworld band
    // (`Main.UnderworldLayer` = `maxTilesY - 200`, `Main.cs:2790`) and the
    // 150-tile 10-step scan for the hive tile 226 and the walls 83/3/87.
    if py >= world.height - 200 - 30 {
        return false;
    }
    'scan: for sx in (px - 150..px + 150).step_by(10) {
        if sx <= 0 || sx > world.width - 1 {
            continue;
        }
        for sy in (py - 150..py + 150).step_by(10) {
            if sy <= 0 || sy > world.height - 1 {
                continue;
            }
            if let Some(t) = world.tile(sx, sy) {
                if (t.active() && t.kind == 226) || t.wall == 83 || t.wall == 3 || t.wall == 87 {
                    return false;
                }
            }
        }
    }
    // `:37`: the radial-dither honey fill over the 15-disc (the inner-
    // weighted pass — `NextDouble > dist/10` keeps the rim sparse).
    for dx in -15..=15 {
        for dy in -15..=15 {
            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            if dist > 15.0 {
                continue;
            }
            if !solid_tile(world, px + dx, py + dy) {
                continue;
            }
            let dither = (dist as f64 / 10.0).clamp(0.0, 1.0);
            if rng.next_f64() <= dither {
                continue;
            }
            put(world, px + dx, py + dy, 229);
        }
    }
    // `:38-41`: the inner pocket — the blotched 3×4 clear with the honey
    // liquid, captured as the shape the rim and the cap read.
    let mut pocket: Vec<(i32, i32)> = Vec::new();
    for dx in -6..=6 {
        for dy in 0..=3 {
            let dist = ((dx * dx + dy * dy * 9) as f32).sqrt();
            if dist > 4.0 {
                continue;
            }
            if rng.chance(3) {
                continue;
            }
            pocket.push((px + dx, py + dy));
            clear(world, px + dx, py + dy, 0);
            let i = world.index(px + dx, py + dy);
            world.tiles[i].liquid = 255;
            world.tiles[i].set_liquid_kind(crate::tile::LiquidKind::Honey);
        }
    }
    // `:42-44`: the mud rim — the pocket's inner outline one tile below.
    for &(cx, cy) in &pocket {
        let outline = [
            (cx - 1, cy),
            (cx + 1, cy),
            (cx, cy - 1),
            (cx, cy + 1),
        ]
        .iter()
        .any(|(nx, ny)| world.tile(*nx, *ny).is_some_and(|t| !t.active()));
        if !outline {
            continue;
        }
        let py2 = cy + 1;
        if !(1..=3).contains(&(py2 - py)) {
            continue;
        }
        if world.tile(cx, py2).is_some_and(|t| !t.active()) {
            put(world, cx, py2, 59);
        }
    }
    // `:45-47`: the honey cap — the pocket expanded by 1, below the patch
    // height, not solid and not holding honey.
    for &(cx, cy) in &pocket {
        for ex in -1..=1 {
            for ey in -1..=1 {
                let (tx, ty) = (cx + ex, cy + ey);
                if ty > py {
                    continue;
                }
                if world.tile(tx, ty).is_some_and(|t| t.active()) {
                    continue;
                }
                if world.tile(tx, ty).map(|t| t.liquid) == Some(255)
                    && world.tile(tx, ty).map(|t| t.liquid_kind()) == Some(crate::tile::LiquidKind::Honey)
                {
                    continue;
                }
                put(world, tx, ty, 229);
            }
        }
    }
    protected.push((px - 8, py - 8, 16, 16));
    true
}

/// The hive pass's honey patches (`WorldGen.cs:15946-15968`): each placed
/// hive rolls `Next(5)` patches on the angle/radius ring around its centre
/// (the 30-90 px ring, the bounds gate, the `extraLiquid` seed's zero).
pub(crate) fn place_honey_patches_around(
    world: &mut World,
    cx: i32,
    cy: i32,
    protected: &mut Vec<(i32, i32, i32, i32)>,
    rng: &mut super::Rng,
) -> u32 {
    let want = if world
        .micro_seeds
        .contains(crate::seeds::MicroSeedFlags::EXTRA_LIQUID)
    {
        0
    } else {
        rng.range(0, 5)
    };
    let mut placed = 0u32;
    let mut guard = 10000;
    let mut done = 0;
    while done < want && guard > 0 {
        guard -= 1;
        let dist = rng.next_f64() * 60.0 + 30.0;
        let ang = rng.next_f64() * std::f64::consts::TAU;
        let px = cx + (ang.cos() * dist) as i32;
        let py = cy + (ang.sin() * dist) as i32;
        if px <= 50 || px >= world.width - 50 {
            continue;
        }
        if place_honey_patch(world, px, py, protected, rng) {
            placed += 1;
            done += 1;
        }
    }
    placed
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::seeds::MicroSeedFlags;
    use crate::worldgen::noise::Rng;

    /// A stone world with the band the cave houses draw in, deep enough for
    /// the bigger arm's +70 floor shift.
    fn cave_world(width: i32, height: i32) -> World {
        let mut w = World::new("mb".into(), width, height);
        for x in 0..width {
            for y in 100..height {
                let i = w.index(x, y);
                w.tiles[i].kind = 1;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.world_surface = 130.0;
        w.rock_layer = 400.0;
        w
    }

    /// The wood-frame tiles (27) a placed house leaves behind.
    fn frame_tiles(world: &World) -> usize {
        world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 27)
            .count()
    }

    #[test]
    fn the_bigger_house_flag_always_sizes_up() {
        // `WorldGen.cs:433-447`: `biggerAbandonedHouses` returns true
        // unconditionally; without it (and without errorWorld, which the port
        // does not model) the C# returns false.
        let classic = World::new("mb".into(), 600, 600);
        let mut flagged = World::new("mb".into(), 600, 600);
        flagged.micro_seeds |= MicroSeedFlags::BIGGER_ABANDONED_HOUSES;
        assert!(!bigger_abandoned_houses(&classic));
        assert!(bigger_abandoned_houses(&flagged));
    }

    #[test]
    fn the_bigger_houses_are_wider_and_deeper_than_the_vanilla_ones() {
        // The pass-level effect: the flagged pass draws the same number of
        // attempted houses but its room rolls are `range(5, 10)` ×
        // `range(3, 6)` (the C#'s `Next(15, 31)` × `Next(8, 13)` chained
        // rooms) against the classic `range(5, 9)` × `range(3, 5)`, so a
        // seed sweep leaves strictly more wood-frame behind.
        let width = 900;
        let mut classic_total = 0usize;
        let mut bigger_total = 0usize;
        for seed in 1..=6u64 {
            let mut classic = cave_world(width, 900);
            pass_cave_houses(&mut classic, 400.0, &mut Rng(seed));
            classic_total += frame_tiles(&classic);

            let mut bigger = cave_world(width, 900);
            bigger.micro_seeds |= MicroSeedFlags::BIGGER_ABANDONED_HOUSES;
            pass_cave_houses(&mut bigger, 400.0, &mut Rng(seed));
            bigger_total += frame_tiles(&bigger);
        }
        assert!(
            bigger_total > classic_total,
            "the bigger houses cover more: bigger {bigger_total} vs classic {classic_total}"
        );
    }

    #[test]
    fn the_classic_house_rolls_are_untouched() {
        // The byte-identity half: with no micro flag the pass places houses at
        // exactly the vanilla geometry — a 900-wide world over a seed sweep
        // reproduces the same frame-tile total as a run that never consults
        // the flag (the two are the same code path, so this pins the arm's
        // gate rather than a second implementation).
        let a = {
            let mut w = cave_world(900, 900);
            pass_cave_houses(&mut w, 400.0, &mut Rng(99));
            frame_tiles(&w)
        };
        let b = {
            let mut w = cave_world(900, 900);
            w.micro_seeds = MicroSeedFlags::none();
            pass_cave_houses(&mut w, 400.0, &mut Rng(99));
            frame_tiles(&w)
        };
        assert_eq!(a, b);
        assert!(a > 0, "the classic pass places houses");
    }

    #[test]
    fn the_honey_patch_fills_the_pocket_and_registers_the_rect() {
        // `HoneyPatchBiome.Place` (`HoneyPatchBiome.cs:9-58`): the pocket's
        // cells carry the honey liquid (kind 2) and the honey-block tile
        // 229, and the 16×16 rect joins the protected list.
        // A 600-tall world: the underworld band (`height - 200 - 30` =
        // 370) must sit below the patch at y = 100 (`Main.cs:60`'s gate).
        let mut w = World::new("hp".into(), 400, 600);
        for x in 0..400 {
            for y in 0..600 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 59,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        // The hive core (tile 60) the `obj3 >= 2` gate counts.
        for x in 90..110 {
            for y in 90..110 {
                let i = w.index(x, y);
                w.tiles[i].kind = 60;
            }
        }
        let mut protected = Vec::new();
        let mut rng = crate::worldgen::noise::Rng::from_seed(7);
        let (mut solid, mut mud_hive, mut hive) = (0i32, 0i32, 0i32);
        for dx in -15..=15 {
            for dy in -15..=15 {
                if (dx * dx + dy * dy) as f32 > 225.0 {
                    continue;
                }
                if let Some(t) = w.tile(100 + dx, 100 + dy) {
                    if t.active() {
                        solid += 1;
                        if matches!(t.kind, 59 | 60) {
                            mud_hive += 1;
                        }
                        if t.kind == 60 {
                            hive += 1;
                        }
                    }
                }
            }
        }
        // The origin must not be solid (`:11-13`): the ring roll lands in
        // the open, so the patch seeds its own air cell.
        let i = w.index(100, 100);
        w.tiles[i] = Tile::default();
        let ok = super::place_honey_patch(&mut w, 100, 100, &mut protected, &mut rng);
        assert!(
            ok,
            "the mud disc passes the 75 % gate (protected={protected:?} solid={solid} mud_hive={mud_hive} hive={hive})"
        );
        // The honey pockets hold the liquid kind 2 (`LiquidKind::Honey`).
        let honey = w
            .tiles
            .iter()
            .filter(|t| t.liquid_kind() == crate::tile::LiquidKind::Honey)
            .count();
        assert!(honey > 0, "the pocket holds honey: {honey}");
        let blocks = w.tiles.iter().filter(|t| t.active() && t.kind == 229).count();
        assert!(blocks > 0, "the patch fills with the honey block 229: {blocks}");
        assert_eq!(protected.len(), 1, "the 16x16 rect is registered");
    }

    #[test]
    fn the_honey_patch_rejects_the_small_disc_and_the_underworld() {
        // The 8-disc solid gate (20+) refuses a sparse world, and the
        // underworld band (`Main.UnderworldLayer - 30`) refuses a deep
        // origin.
        let mut w = World::new("hp2".into(), 200, 200);
        let mut protected = Vec::new();
        let mut rng = crate::worldgen::noise::Rng::from_seed(9);
        assert!(
            !super::place_honey_patch(&mut w, 100, 100, &mut protected, &mut rng),
            "the empty air fails the solid gate"
        );
        // The underworld: a full disc 30 tiles above the band's bottom.
        let mut w2 = World::new("hp3".into(), 200, 200);
        for x in 0..200 {
            for y in 0..200 {
                let i = w2.index(x, y);
                w2.tiles[i] = Tile {
                    kind: 59,
                    ..Default::default()
                };
                w2.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut rng2 = crate::worldgen::noise::Rng::from_seed(3);
        let deep = w2.height - 200 - 40;
        assert!(
            !super::place_honey_patch(&mut w2, 100, deep, &mut protected, &mut rng2),
            "the underworld band refuses"
        );
    }

    #[test]
    fn the_hive_places_its_five_patches_on_the_ring() {
        // `WorldGen.cs:15946-15968`: each placed hive rolls `Next(5)`
        // patches at the 30-90 px ring; the `extraLiquid` seed zeroes the
        // count. The jungle span lets the hive place.
        let mut w = World::new("hp4".into(), 400, 300);
        for x in 0..400 {
            for y in 0..300 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 59,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.world_surface = 75.0;
        w.rock_layer = 150.0;
        let spans = super::super::biomes::BiomeSpans {
            jungle_start: 60,
            jungle_end: 340,
            dungeon_x: 40,
        };
        let mut rng = crate::worldgen::noise::Rng::from_seed(11);
        // One hive with the full world disc for its patches.
        let before = w.tiles.iter().filter(|t| t.liquid_kind() == crate::tile::LiquidKind::Honey).count();
        let placed = super::super::structures::pass_beehives(&mut w, &spans, 150.0, &mut rng);
        let after = w.tiles.iter().filter(|t| t.liquid_kind() == crate::tile::LiquidKind::Honey).count();
        assert!(placed >= 1, "the hive placed: {placed}");
        assert!(after > before, "the hive's patches hold honey: {before} -> {after}");
    }

}