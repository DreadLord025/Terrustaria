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

use crate::tile_light::FastRandom;
use crate::world::World;

/// `TileID.Sets.SpreadsCorruption` (`TileID.cs:580`).
pub const SPREADS_CORRUPTION: [u16; 10] = [23, 661, 25, 112, 398, 400, 163, 32, 636, 24];
/// `TileID.Sets.SpreadsCrimson` (`TileID.cs:589`).
pub const SPREADS_CRIMSON: [u16; 10] = [199, 662, 203, 234, 399, 401, 200, 352, 205, 201];

/// `Convert`'s tile map (`:55139-55205`): the source kind → the converted
/// kind per the evil flavour.
fn convert_tile(kind: u16, corruption: bool) -> Option<u16> {
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
fn convert_wall(wall: u16, corruption: bool) -> Option<u16> {
    Some(match (wall, corruption) {
        // The stone wall: the ebonstone 3 / the crimson 192.
        (1, c) => {
            if c {
                3
            } else {
                192
            }
        }
        // The dirt wall: the corrupt grass 69 / the crimson grass 193.
        (2, c) => {
            if c {
                69
            } else {
                193
            }
        }
        _ => return None,
    })
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
    rng: &mut FastRandom,
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
        if rng.next(3) != 0 {
            let (dx, dy) = match rng.next(4) {
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
        let nx = x + (rng.next(13) as i32 - 6);
        let ny = y + (rng.next(13) as i32 - 6);
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
    if downed_plant_boss && rng.next(2) != 0 {
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
    let mut tx = x + (rng.next(7) as i32 - 3);
    let mut ty = y + (rng.next(7) as i32 - 3);
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
        let reroll = rng.next(2) == 0;
        convert_cell(world, tx, ty, corruption, false);
        if !reroll {
            break;
        }
        tx += rng.next(7) as i32 - 3;
        ty += rng.next(7) as i32 - 3;
    }
}

/// `GERunner`'s hallow (good) map (`WorldGen.cs:76169-76290`): the hallowed
/// grass 109, the pearlstone 117, the hallowed sand 116, the hallowed ice
/// 164, the hallowed sandstone 403, the hallowed hardened sand 402 — and
/// the purify arm (the corrupt/crimson jungle grass 661/662 → the jungle
/// grass 60).
fn convert_tile_hallow(kind: u16) -> Option<u16> {
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

/// `GERunner`'s wall arms (`:76244-76260`): the sandstone wall 187 → the
/// hallowed sandstone wall 222; the ebonstone/crimstone walls 3/83 → the
/// hallowed stone wall 28.
fn convert_wall_hallow(wall: u16) -> Option<u16> {
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
    rng: &mut FastRandom,
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
pub fn initialize_hard_mode(world: &mut World, corruption: bool, rng: &mut FastRandom) {
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
        let mut rng = FastRandom::new(9);
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
        let mut rng = FastRandom::new(10);
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
        let mut rng = FastRandom::new(11);
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
        let mut rng = FastRandom::new(11);
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
            let mut rng = FastRandom::new(seed);
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
        // The walls: the stone 1 → 3 (corruption) / 192 (crimson), the
        // dirt 2 → 69/193.
        assert_eq!(convert_wall(1, true), Some(3));
        assert_eq!(convert_wall(1, false), Some(192));
        assert_eq!(convert_wall(2, true), Some(69));
        assert_eq!(convert_wall(2, false), Some(193));
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
        let mut rng = FastRandom::new(1);
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
        let mut rng = FastRandom::new(3);
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
        let mut rng = FastRandom::new(4);
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
        let mut rng = FastRandom::new(5);
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
        let mut rng = FastRandom::new(6);
        for _ in 0..100 {
            hard_update_world(&mut w, 200, 200, false, false, &mut rng);
        }
        assert_eq!(w.tile(201, 200).unwrap().kind, 2, "the spread frozen");
    }
}
