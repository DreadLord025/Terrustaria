//! The secret-seed tail helpers the Skyblock generator calls
//! (`WorldGen.cs:22803-22815`) — `DoPooEverywhere` (`:2966-2993`),
//! `DoErrorWorldShuffleBlocks` (`:1094-1278` + `DoErrorWorldGetRandomBlock`
//! `:1084-1092` + `ErrorWorldSwapTiles` `:1280-1295` +
//! `Variations.errorWorldAdjustment` `:329-336`), and
//! `DoAdditionalChangesAroundSpawnIfNeeded` (`:24936-24951` +
//! `PlaceTorchesAroundSpawn` `:24989-25087` + `RemoveSpecificTilesUnderSpawn`
//! `:24953-24987`).
//!
//! W4-f24 lane A3. These are shared helpers (the C# also calls them from the
//! Beaches/Life Crystals passes, `:14894`/`:16761` — those call sites belong
//! to the sibling lanes); this slice ports the bodies for the skyblock
//! generator's tail. Every draw rides the caller's pass stream
//! (`WorldGen.genRand`); the torch placement rides the port's `place_torch`
//! seam and the under-spawn removal the `kill_tile(no_item)` seam (the C#
//! `KillTile(fail:false, effectOnly:false, noItem:true)`).
//!
//! UNWIRED until A4: no pipeline step calls these yet — both classic
//! world-hash pins stay; `a == b` untouched.

use super::tile_runner::{tile_runner, TileRunnerOpts};
use crate::seeds::{active_secret_seed_count, MicroSeedFlags, SeedFlags};
use crate::tile::{FLAG_ACTIVE, FLAG_FULLBRIGHT};
use crate::tiledata::{SET_CLOUDS, SET_REMOVE_WHEN_UNDER_SPAWN, TILE_SOLID, TILE_SOLID_TOP};
use crate::world::World;

/// `WorldGen.InWorld(int, int, int fluff)` (`WorldGen.cs:9688-9695`).
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `WorldGen.SolidTile(int, int)` (`WorldGen.cs:70111-70132`) — null reads
/// as solid; active + tileSolid + !tileSolidTop + !halfBrick + slope 0 +
/// !inActive.
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && TILE_SOLID.has(t.kind)
        && !TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// `Tile.anyShimmer()` (`Tile.cs:428-435`).
fn any_shimmer(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Shimmer)
}

/// `WorldGen.IsTileNearby` (`WorldGen.cs:8862-8884`) — the 235 stride-3
/// special case kept.
fn is_tile_nearby(world: &World, x: i32, y: i32, kind: u16, distance: i32) -> bool {
    let stride = if kind == 235 { 3 } else { 1 };
    let mut i = x - distance;
    while i <= x + distance {
        for j in y - distance..=y + distance {
            if in_world(world, i, j, 0)
                && world.tile(i, j).is_some_and(|t| t.active() && t.kind == kind)
            {
                return true;
            }
        }
        i += stride;
    }
    false
}

/// `SecretSeed.Variations.errorWorldAdjustment(double)`
/// (`WorldGen.cs:329-336`) — `< 1` active seeds returns 4, else
/// `(int)(value * ((count + 3) / 4))` (integer division).
pub(crate) fn error_world_adjustment(value: f64, micro: MicroSeedFlags, classic: SeedFlags) -> i32 {
    let count = active_secret_seed_count(micro, classic);
    if count < 1 {
        return 4;
    }
    (value * ((count + 3) / 4) as f64) as i32
}

/// `SecretSeed.DoErrorWorldGetRandomBlock` (`WorldGen.cs:1084-1092`) — the
/// `Next(TileID.Count)` re-roll until a plain solid block (not solidTop, not
/// frameImportant, not dungeon, not 58/226/404).
pub(crate) fn do_error_world_get_random_block(
    world: &World,
    rng: &mut super::Rng,
) -> u16 {
    let mut num = rng.range(0, crate::tiledata::TILE_COUNT as i32);
    while !TILE_SOLID.has(num as u16)
        || TILE_SOLID_TOP.has(num as u16)
        || world.frame_important[num as usize]
        || super::dungeon_features::is_dungeon_tile(num as u16)
        || num == 58
        || num == 226
        || num == 404
    {
        num = rng.range(0, crate::tiledata::TILE_COUNT as i32);
    }
    num as u16
}

/// `SecretSeed.ErrorWorldSwapTiles(Tile, Tile)` (`WorldGen.cs:1280-1295`) —
/// swap type + active + color(paint) + fullbrightBlock + invisibleBlock.
/// The invisible bit rides the world extras (the port's `invisible_block`
/// seam), the rest the tile fields.
fn error_world_swap_tiles(world: &mut World, x1: i32, y1: i32, x2: i32, y2: i32) {
    let (Some(a), Some(b)) = (world.tile(x1, y1).copied(), world.tile(x2, y2).copied()) else {
        return;
    };
    let inv_a = world.invisible_block(x1, y1);
    let inv_b = world.invisible_block(x2, y2);
    if let Some(t) = world.tile_mut(x1, y1) {
        t.kind = b.kind;
        set_flag(t, FLAG_ACTIVE, b.active());
        t.set_paint(b.paint());
        set_flag(t, FLAG_FULLBRIGHT, b.fullbright_block());
    }
    if let Some(t) = world.tile_mut(x2, y2) {
        t.kind = a.kind;
        set_flag(t, FLAG_ACTIVE, a.active());
        t.set_paint(a.paint());
        set_flag(t, FLAG_FULLBRIGHT, a.fullbright_block());
    }
    world.set_invisible_block(x1, y1, inv_b);
    world.set_invisible_block(x2, y2, inv_a);
}

fn set_flag(t: &mut crate::tile::Tile, flag: u8, on: bool) {
    if on {
        t.flags |= flag;
    } else {
        t.flags &= !flag;
    }
}

/// The error-world shuffle's cell gate — `!tileSolid || tileSolidTop ||
/// frameImportant` (the re-roll condition, `WorldGen.cs:1107`), plus the
/// per-loop extra terms the callers add.
fn shuffle_rejects(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    !TILE_SOLID.has(t.kind) || TILE_SOLID_TOP.has(t.kind) || world.frame_important[t.kind as usize]
}

/// `SecretSeed.DoErrorWorldShuffleBlocks` (`WorldGen.cs:1094-1278`) — the
/// four loops: random-block replacement (`maxTilesX*10/adjustment`, halved
/// on skyblock), the pair swaps (`maxTilesX/2/adjustment`, the spawn-band
/// veto), the rect swaps (same count, the ±50 band veto), and the line
/// extensions (`maxTilesX*10/adjustment`, the direction walk copying the
/// source tile into inactive cells). `rng` is the pass stream.
pub(crate) fn do_error_world_shuffle_blocks(
    world: &mut World,
    rng: &mut super::Rng,
    flags: SeedFlags,
    micro: MicroSeedFlags,
) {
    let num = 30;
    let num2 = 50;
    let max_x = world.width;
    let max_y = world.height;
    let skyblock = flags.contains(SeedFlags::SKYBLOCK);
    let adj = error_world_adjustment(1.0, micro, flags);
    // `:1098-1117`: the random-block replacement loop.
    let mut num3 = max_x * 10 / adj;
    if skyblock {
        num3 /= 2;
    }
    for _ in 0..num3 {
        let mut x = rng.range(num, max_x - num);
        let mut y = rng.range(num + num2, max_y - num);
        while shuffle_rejects(world, x, y) {
            x = rng.range(num, max_x - num);
            y = rng.range(num + num2, max_y - num);
        }
        let kind = do_error_world_get_random_block(world, rng);
        if let Some(t) = world.tile_mut(x, y) {
            t.kind = kind;
            if skyblock {
                t.flags |= FLAG_ACTIVE;
            }
        }
    }
    // `:1118-1147`: the pair swaps (the anyShimmer + neighbour-frame gates).
    num3 = max_x / 2 / adj;
    for _ in 0..num3 {
        let mut x1 = rng.range(num, max_x - num);
        let mut y1 = rng.range(num + num2, max_y - num);
        while any_shimmer(world, x1, y1)
            || shuffle_rejects(world, x1, y1)
            || world.frame_important[world.tile(x1, y1 - 1).map(|t| t.kind).unwrap_or(0) as usize]
            || world.frame_important[world.tile(x1, y1 + 1).map(|t| t.kind).unwrap_or(0) as usize]
        {
            x1 = rng.range(num, max_x - num);
            y1 = rng.range(num + num2, max_y - num);
        }
        let mut x2 = rng.range(num, max_x - num);
        let mut y2 = rng.range(num + num2, max_y - num);
        while any_shimmer(world, x2, y2)
            || shuffle_rejects(world, x2, y2)
            || world.frame_important[world.tile(x2, y2 - 1).map(|t| t.kind).unwrap_or(0) as usize]
            || world.frame_important[world.tile(x2, y2 + 1).map(|t| t.kind).unwrap_or(0) as usize]
        {
            x2 = rng.range(num, max_x - num);
            y2 = rng.range(num + num2, max_y - num);
        }
        // `:1135-1145`: the spawn-band veto (remix: the underworld centre;
        // else: the surface centre).
        if flags.contains(SeedFlags::REMIX) {
            if (x1 > max_x / 2 - 30 && x1 < max_x / 2 + 30 && y1 > max_y - 200)
                || (x2 > max_x / 2 - 30 && x2 < max_x / 2 + 30 && y2 > max_y - 200)
            {
                continue;
            }
        } else if (x1 > max_x / 2 - 30
            && x1 < max_x / 2 + 30
            && (y1 as f64) < world.world_surface)
            || (x2 > max_x / 2 - 30 && x2 < max_x / 2 + 30 && (y2 as f64) < world.world_surface)
        {
            continue;
        }
        error_world_swap_tiles(world, x1, y1, x2, y2);
    }
    // `:1148-1225`: the rect swaps.
    num3 = max_x / 2 / adj;
    for _ in 0..num3 {
        let (mut w, mut h);
        if rng.range(0, 2) == 0 {
            w = rng.range(5, 21);
            h = rng.range(1, 4);
        } else {
            w = rng.range(1, 4);
            h = rng.range(5, 21);
        }
        let x1 = rng.range(num, max_x - num);
        let y1 = rng.range(num + num2, max_y - num);
        let mut bad = false;
        for l in x1..x1 + w {
            for m in y1..y1 + h {
                if any_shimmer(world, l, m)
                    || shuffle_rejects(world, l, m)
                    || world.frame_important[world.tile(l, m - 1).map(|t| t.kind).unwrap_or(0) as usize]
                    || world.frame_important[world.tile(l, m + 1).map(|t| t.kind).unwrap_or(0) as usize]
                {
                    bad = true;
                    break;
                }
            }
            if bad {
                break;
            }
        }
        if bad {
            continue;
        }
        let x2 = rng.range(num, max_x - num);
        let y2 = rng.range(num + num2, max_y - num);
        bad = false;
        for n in x2..x2 + w {
            for o in y2..y2 + h {
                if any_shimmer(world, n, o)
                    || shuffle_rejects(world, n, o)
                    || world.frame_important[world.tile(n, o - 1).map(|t| t.kind).unwrap_or(0) as usize]
                    || world.frame_important[world.tile(n, o + 1).map(|t| t.kind).unwrap_or(0) as usize]
                {
                    bad = true;
                    break;
                }
            }
            if bad {
                break;
            }
        }
        if bad {
            continue;
        }
        // `:1207-1217`: the ±50 spawn-band veto.
        if flags.contains(SeedFlags::REMIX) {
            if (x1 + w / 2 > max_x / 2 - 50 && x1 < max_x / 2 + 50 && y1 > max_y - 200)
                || (x2 + w / 2 > max_x / 2 - 50 && x2 < max_x / 2 + 50 && y2 > max_y - 200)
            {
                continue;
            }
        } else if (x1 + w / 2 > max_x / 2 - 50
            && x1 < max_x / 2 + 50
            && (y1 as f64) < world.world_surface)
            || (x2 + w / 2 > max_x / 2 - 50 && x2 < max_x / 2 + 50 && (y2 as f64) < world.world_surface)
        {
            continue;
        }
        for dx in 0..w {
            for dy in 0..h {
                error_world_swap_tiles(world, x1 + dx, y1 + dy, x2 + dx, y2 + dy);
            }
        }
    }
    // `:1226-1277`: the line extensions.
    num3 = max_x * 10 / adj;
    for _ in 0..num3 {
        let mut x = rng.range(num, max_x - num);
        let mut y = rng.range(num + num2, max_y - num);
        while shuffle_rejects(world, x, y)
            || super::dungeon_features::is_dungeon_tile(world.tile(x, y).map(|t| t.kind).unwrap_or(0))
        {
            x = rng.range(num, max_x - num);
            y = rng.range(num + num2, max_y - num);
        }
        // `:1236-1255`: the axis direction (two Next(2) rolls).
        let (mut dx, mut dy) = (0.0f64, 0.0f64);
        if rng.range(0, 2) == 0 {
            if rng.range(0, 2) == 0 {
                dx = -1.0;
            } else {
                dx = 1.0;
            }
        } else if rng.range(0, 2) == 0 {
            dy = -1.0;
        } else {
            dy = 1.0;
        }
        let steps = rng.range(5, 21);
        let (mut cx, mut cy) = (x, y);
        for _ in 0..steps {
            cx += dx as i32;
            cy += dy as i32;
            if world.tile(cx, cy).is_some_and(|t| t.active()) {
                break;
            }
            // `:1269-1275`: the source loses slope/halfBrick; the target
            // copies active/type/color/fullbright/invisible.
            if let Some(t) = world.tile_mut(x, y) {
                t.set_slope(0);
                t.flags &= !crate::tile::FLAG_HALF_BRICK;
            }
            let src = world.tile(x, y).copied();
            let inv = world.invisible_block(x, y);
            if let (Some(src), Some(t)) = (src, world.tile_mut(cx, cy)) {
                set_flag(t, FLAG_ACTIVE, src.active());
                t.kind = src.kind;
                t.set_paint(src.paint());
                set_flag(t, FLAG_FULLBRIGHT, src.fullbright_block());
            }
            world.set_invisible_block(cx, cy, inv);
        }
    }
}

/// `SecretSeed.DoPooEverywhere` (`WorldGen.cs:2966-2993`) — the poo-666
/// TileRunner scatter: `area * 0.0002` sites divided by
/// `(2 + activeSecretSeedCount) / 3` (min 1), the 30-step downward solid
/// find, the dungeon/cloud/wall-73 veto (skyblock bypasses it), then
/// `TileRunner(x, y, Next(5,12), Next(10,31), 666, addTile:true)`.
pub(crate) fn do_poo_everywhere(
    world: &mut World,
    rng: &mut super::Rng,
    flags: SeedFlags,
    micro: MicroSeedFlags,
) {
    let num = 40;
    let mut count = ((world.width * world.height) as f64 * 0.0002) as i32;
    let mut div = (2 + active_secret_seed_count(micro, flags)) as i32 / 3;
    if div < 1 {
        div = 1;
    }
    count /= div;
    let skyblock = flags.contains(SeedFlags::SKYBLOCK);
    let runner_opts = TileRunnerOpts {
        add_tile: true,
        ..TileRunnerOpts::for_world(world)
    };
    let world_surface = world.world_surface;
    for _ in 0..count {
        let x = rng.range(num, world.width - num);
        let mut y = rng.range(num, world.height - num);
        for _ in 0..30 {
            if solid_tile(world, x, y) {
                break;
            }
            y += 1;
        }
        let gate = world.tile(x, y).is_some_and(|t| {
            skyblock
                || !t.active()
                || (!super::dungeon_features::is_dungeon_tile(t.kind)
                    && !super::dungeon_features::is_dungeon_wall(t.wall)
                    && !SET_CLOUDS.has(t.kind)
                    && t.wall != 73)
        });
        if gate && solid_tile(world, x, y) {
            let strength = rng.range(5, 12);
            let steps = rng.range(10, 31);
            tile_runner(world, rng, x, y, strength as f64, steps, 666, world_surface, &runner_opts);
        }
    }
}

/// `WorldGen.RemoveSpecificTilesUnderSpawn` (`WorldGen.cs:24953-24987`) —
/// the 50-row × 5-column `RemoveWhenUnderSpawn` kill sweep under the spawn
/// (`KillTile(fail:false, effectOnly:false, noItem:true)` — the port's
/// `kill_tile(no_item)` seam).
pub(crate) fn remove_specific_tiles_under_spawn(world: &mut World, spawn_x: i32, spawn_y: i32) {
    for i in 0..50 {
        let y = spawn_y + i;
        if !in_world(world, spawn_x, y, 5) {
            continue;
        }
        for dx in [0, -1, -2, 1, 2] {
            let x = spawn_x + dx;
            if world
                .tile(x, y)
                .is_some_and(|t| t.active() && SET_REMOVE_WHEN_UNDER_SPAWN.has(t.kind))
            {
                crate::edit::kill_tile_legacy(world, x, y, true);
            }
        }
    }
}

/// `WorldGen.PlaceTorchesAroundSpawn` (`WorldGen.cs:24989-25087`) — the
/// 2000-attempt torch scatter: the growing ±num box (num caps at 120), the
/// 200-retry InWorld(50) find, the ±1 horizontal walk to the first solid
/// (maxTilesX/2 budget), the step-back, the torch-4 proximity gate
/// (distance 20, skyblock 15) and the biome style ladder
/// (60→21, 70→22, 161/147→9, 53→16, 25→18, 203→19). The placement rides the
/// port's `place_torch` seam (the C# `PlaceTile(…, 4, …, style)`).
pub(crate) fn place_torches_around_spawn(
    world: &mut World,
    rng: &mut super::Rng,
    flags: SeedFlags,
    spawn_x: i32,
    spawn_y: i32,
) {
    let mut num = 1;
    let distance = if flags.contains(SeedFlags::SKYBLOCK) { 15 } else { 20 };
    let mut i = 0;
    while i < 2000 {
        if num < 120 {
            num += 1;
        }
        let (min_x, max_x) = (spawn_x - num, spawn_x + num);
        let (min_y, max_y) = (spawn_y - num, spawn_y + num);
        let mut x = rng.range(min_x, max_x);
        let mut y = rng.range(min_y, max_y);
        let mut budget = 200;
        while budget > 0 && !in_world(world, x, y, 50) {
            budget -= 1;
            x = rng.range(min_x, max_x);
            y = rng.range(min_y, max_y);
        }
        if budget <= 0 {
            break;
        }
        let mut dir = 1;
        if rng.range(0, 2) == 0 {
            dir *= -1;
        }
        if world.tile(x, y).is_some_and(|t| !t.active() && t.liquid == 0) {
            let mut walk = world.width / 2;
            while !world.tile(x, y).is_some_and(|t| t.active()) {
                walk -= 1;
                if walk <= 0 {
                    break;
                }
                x += dir;
                if !in_world(world, x, y, 50) {
                    x -= dir;
                    break;
                }
            }
            if walk <= 0 {
                i += 1;
                continue;
            }
            x -= dir;
            if !is_tile_nearby(world, x, y, 4, distance) {
                let style = if is_tile_nearby(world, x, y, 60, 25) {
                    21
                } else if is_tile_nearby(world, x, y, 70, 25) {
                    22
                } else if is_tile_nearby(world, x, y, 161, 25) {
                    9
                } else if is_tile_nearby(world, x, y, 147, 25) {
                    9
                } else if is_tile_nearby(world, x, y, 53, 25) {
                    16
                } else if is_tile_nearby(world, x, y, 25, 25) {
                    18
                } else if is_tile_nearby(world, x, y, 203, 25) {
                    19
                } else {
                    0
                };
                crate::edit::place_torch(world, x, y, style);
            }
        } else {
            // `:25084`: the occupied-cell retry does not consume the attempt.
            i -= 1;
        }
        i += 1;
    }
}

/// `WorldGen.DoAdditionalChangesAroundSpawnIfNeeded`
/// (`WorldGen.cs:24936-24951`) — the torch scatter + the noTraps under-spawn
/// removal. The default spawn coords read `Main.spawnTileX/Y` (the port's
/// `world.spawn_x/y`).
pub(crate) fn do_additional_changes_around_spawn_if_needed(
    world: &mut World,
    rng: &mut super::Rng,
    flags: SeedFlags,
) {
    let spawn_x = world.spawn_x;
    let spawn_y = world.spawn_y;
    place_torches_around_spawn(world, rng, flags, spawn_x, spawn_y);
    if flags.contains(SeedFlags::NO_TRAPS) {
        remove_specific_tiles_under_spawn(world, spawn_x, spawn_y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `errorWorldAdjustment`: < 1 active seed returns 4; else the
    /// (count+3)/4 integer scale (`WorldGen.cs:329-336`).
    #[test]
    fn w4_f24_error_world_adjustment_ladder() {
        let none = MicroSeedFlags::none();
        let classic = SeedFlags::none();
        assert_eq!(error_world_adjustment(1.0, none, classic), 4);
        // One micro-seed: (1+3)/4 = 1.
        let one = MicroSeedFlags::from(MicroSeedFlags::POO_EVERYWHERE);
        assert_eq!(error_world_adjustment(1.0, one, classic), 1);
        // Five micro-seeds: (5+3)/4 = 2.
        let five = MicroSeedFlags::from(
            MicroSeedFlags::POO_EVERYWHERE
                | MicroSeedFlags::ERROR_WORLD
                | MicroSeedFlags::NO_SURFACE
                | MicroSeedFlags::EXTRA_LIQUID
                | MicroSeedFlags::RANDOM_SPAWN,
        );
        assert_eq!(error_world_adjustment(1.0, five, classic), 2);
    }

    /// `DoErrorWorldGetRandomBlock` never returns a solidTop/frameImportant/
    /// dungeon/58/226/404 kind (`WorldGen.cs:1084-1092`); a == b replay.
    #[test]
    fn w4_f24_error_world_random_block_is_plain_solid() {
        let mut w = crate::worldgen::generate(&crate::worldgen::GenOptions {
            width: 512,
            height: 512,
            seed: 42,
            ..Default::default()
        });
        let pick = |w: &World| {
            let mut rng = super::super::Rng::from_seed(9);
            (0..20).map(|_| do_error_world_get_random_block(w, &mut rng)).collect::<Vec<_>>()
        };
        let a = pick(&w);
        let b = pick(&w);
        assert_eq!(a, b);
        for k in a {
            assert!(TILE_SOLID.has(k));
            assert!(!TILE_SOLID_TOP.has(k));
            assert!(!w.frame_important[k as usize]);
            assert!(!super::super::dungeon_features::is_dungeon_tile(k));
            assert!(!matches!(k, 58 | 226 | 404));
        }
        let _ = &mut w;
    }

    /// `ErrorWorldSwapTiles` swaps type/active/paint/fullbright/invisible
    /// (`WorldGen.cs:1280-1295`).
    #[test]
    fn w4_f24_error_world_swap_is_symmetric() {
        let mut w = World::new("t".into(), 64, 64);
        if let Some(t) = w.tile_mut(1, 1) {
            t.kind = 1;
            t.flags |= FLAG_ACTIVE;
            t.set_paint(3);
        }
        if let Some(t) = w.tile_mut(2, 2) {
            t.kind = 40;
            t.set_paint(7);
            t.flags |= FLAG_FULLBRIGHT;
        }
        w.set_invisible_block(1, 1, true);
        error_world_swap_tiles(&mut w, 1, 1, 2, 2);
        let a = w.tile(1, 1).unwrap();
        let b = w.tile(2, 2).unwrap();
        assert_eq!(a.kind, 40);
        assert!(!a.active());
        assert_eq!(a.paint(), 7);
        assert!(a.fullbright_block());
        assert_eq!(b.kind, 1);
        assert!(b.active());
        assert_eq!(b.paint(), 3);
        assert!(!b.fullbright_block());
        assert!(!w.invisible_block(1, 1));
        assert!(w.invisible_block(2, 2));
        // Swapping back restores the original.
        error_world_swap_tiles(&mut w, 1, 1, 2, 2);
        assert_eq!(w.tile(1, 1).unwrap().kind, 1);
        assert_eq!(w.tile(2, 2).unwrap().kind, 40);
    }
}
