//! W4-d102 pass 6 — the Shimmer pass (`WorldGen.cs:15195-15245`) plus its
//! two biome builders: `ShimmerMakeBiome` (`:34853-35057`) and
//! `ShimmerMakeBiomeOpening` (`:35059-35102`).
//!
//! The pass samples a site on the dungeon-opposite side (the port's dungeon
//! side is fixed `DungeonSide.Right` — M5-BIOME decision #10, the
//! `temple.rs` precedent — so the `dungeonSide < Right` test is false and
//! the LEFT x band `Next(200, 0.11·width)` is the live arm), retries until
//! `ShimmerMakeBiome` accepts, then publishes `GenVars.shimmerPosition`
//! (`GenVars.cs:282`, `:15241`).
//!
//! Deviations (recorded in the lane ledger `docs/research/12a-d102-passes-2.md`):
//! - the RNG is the port's per-pass xorshift stream
//!   (`Streams::pass_rng("Shimmer")` — the `WorldGenerator.cs:497` per-pass
//!   re-seed model), not the shared `UnifiedRandom`; the draw ORDER is
//!   faithful, the byte stream is not (the port's worldgen contract).
//! - the gem-tree tail (`:35020-35055`) calls the growth port
//!   `growth::try_growing_tree_by_type`, whose stream is a `UnifiedRandom`
//!   (the C# `genRand` algorithm). Each accepted site seeds one from the
//!   pass stream (`UnifiedRandom::new(rng.next_u32() as i32)` — one extra
//!   word draw per site, the `micro_surface.rs` seed-feed precedent), so
//!   the tree draws stay isolated from every other pass.
//! - `GenVars.structures.AddProtectedStructure` (`:15243`) has no port
//!   state (no StructureMap — the `biomes.rs:1341` precedent); skipped.
//! - the C#'s `liquid = 0` writes keep the liquid-kind bits (the port's
//!   `desert.rs`/`dungeon.rs` precedent — the save model stores the kind in
//!   the flags); the pool's `shimmer(true)` is `set_liquid_kind(Shimmer)`.
//! - sub-vanilla valve (the `temple.rs::find_temple_site` precedent): on a
//!   world whose x band collapses (`0.2·width <= 200` — the widest retry
//!   band; every world under ~1001 wide, the 512 test worlds included) or
//!   whose y band cannot fit the smallest biome radius inside
//!   `InWorld(…, 50)`, the C# `Next` would throw / the retry loop would
//!   spin forever; the port skips the pass BEFORE any draw. Vanilla's
//!   smallest world (4200×1200) never hits it.
//! - retry cap: the C# loop has no attempt bound (a world with no valid
//!   site spins forever); the port gives up after 100,000 attempts and
//!   leaves `shimmerPosition` unset (`None`). A real world accepts within
//!   a handful of attempts, so the cap never fires on the vanilla sizes.

use super::noise::Rng;
use super::spider_gem_moss::place_tight;
use crate::edit::solid_tile;
use crate::growth::try_growing_tree_by_type;
use crate::seeds::SeedFlags;
use crate::tile::{LiquidKind, FLAG_ACTIVE, FLAG_HALF_BRICK};
use crate::unified_random::UnifiedRandom;
use crate::world::World;

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9695`).
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// The site's x draw (`:15210`, `:15228`, `:15233`, `:15238`): the
/// `dungeonSide < DungeonSide.Right` arm draws `(0.89·width, width - 200)`,
/// the other `(200, 0.11·width)`. The port's dungeon side is fixed Right
/// (M5-BIOME decision #10), so the second band is the live one.
fn draw_x(rng: &mut Rng, width: i32) -> i32 {
    rng.range(200, (width as f64 * 0.11) as i32)
}

/// The Shimmer pass (`WorldGen.cs:15195-15245`). Returns the placed site
/// (`GenVars.shimmerPosition`, `:15241`) or `None` when the pass is skipped
/// (skyblock / the sub-vanilla valve).
pub(crate) fn pass_shimmer(
    world: &mut World,
    world_surface: f64,
    rock_layer: f64,
    rng: &mut Rng,
) -> Option<(i32, i32)> {
    // `Skyblock.denyAllGeneration` (`:15197`) is `skyblockWorldGen`
    // (`WorldGen.cs:3131`) — the port's `SeedFlags::SKYBLOCK` (the
    // `islands.rs` precedent).
    if world.seeds.contains(SeedFlags::SKYBLOCK) {
        return None;
    }
    // `:15199-15208` — the y band. The C# casts bind `(int)(a + b) / 2`
    // (integer division of the truncated sum), kept verbatim.
    let num = (world_surface + rock_layer) as i32 / 2 + 50;
    let mut num2 = (((world.height - 250) * 2) as f64 + rock_layer) as i32 / 3;
    if num2 > world.height - 330 - 100 - 30 {
        num2 = world.height - 330 - 100 - 30;
    }
    if num2 <= num {
        num2 = num + 50;
    }
    // The sub-vanilla valve (module doc): BOTH x bands must exist (the
    // `> 20000` retry arm widens to `0.2·width`) and the smallest biome
    // (radius 105) must fit inside InWorld(…, 50) at the band's floor —
    // checked BEFORE the first draw.
    if (world.width as f64 * 0.2) as i32 <= 200 || num + 105 >= world.height - 50 {
        return None;
    }
    let tenth = world.seeds.contains(SeedFlags::TENTH_ANNIVERSARY);
    let mut y = rng.range(num, num2);
    let mut x = draw_x(rng, world.width);
    // `:15211-15216` — the anniversary band.
    let num5 = world_surface as i32 + 150;
    let mut num6 = (rock_layer + world_surface + 200.0) as i32 / 2;
    if num6 <= num5 {
        num6 = num5 + 50;
    }
    if tenth {
        y = rng.range(num5, num6);
    }
    // `:15221-15240` — the retry loop (the port's 100,000-attempt cap is
    // the module-doc valve; a real world accepts long before it).
    let mut attempts = 0;
    while !shimmer_make_biome(world, rng, x, y) {
        attempts += 1;
        if attempts > 100_000 {
            return None;
        }
        if tenth && attempts < 10000 {
            y = rng.range(num5, num6);
            x = draw_x(rng, world.width);
        } else if attempts > 20000 {
            y = rng.range(world_surface as i32 + 100 + 20, num2);
            x = rng.range(200, (world.width as f64 * 0.2) as i32);
        } else {
            y = rng.range((world_surface + rock_layer) as i32 / 2 + 20, num2);
            x = draw_x(rng, world.width);
        }
    }
    // `:15241` — the publication (the adapter writes GenVars). The
    // `AddProtectedStructure` 200×200 box (`:15242-15243`) has no port
    // state (module doc).
    Some((x, y))
}

/// `WorldGen.ShimmerMakeBiome` (`WorldGen.cs:34853-35057`): the ellipse
/// shell + pool carve, the num==0 stone shelf with its stalactites, the two
/// side openings and the 500-roll gem-tree tail. Returns false (before any
/// tile write) when the box leaves `InWorld(…, 50)` or touches crimstone
/// 203 / ebonstone 25 (`:34875-34888` — the C# reads `type` without an
/// active gate, kept verbatim).
fn shimmer_make_biome(world: &mut World, rng: &mut Rng, x: i32, y: i32) -> bool {
    // `:34855-34874` — the shape rolls.
    let num = rng.range(0, 2);
    let mut num2 = 0.6;
    let mut num3 = 1.3;
    let mut num4 = 0.3;
    if num == 0 {
        num2 = 0.55;
        num3 = 2.0;
    }
    num2 *= 1.05 - rng.next_f64() * 0.1;
    num3 *= 1.05 - rng.next_f64() * 0.1;
    num4 *= 1.0 - rng.next_f64() * 0.1;
    let num5 = rng.range(105, 125);
    let num6 = (num5 as f64 * num4) as i32;
    let num7 = (num5 as f64 * num2) as i32;
    let num8 = rng.range(9, 13);
    let num9 = x - num5;
    let num10 = x + num5;
    let num11 = y - num5;
    let num12 = y + num5;
    // The acceptance scan (`:34875-34888`).
    for i in num11..=num12 {
        for j in num9..=num10 {
            if !in_world(world, j, i, 50) {
                return false;
            }
            if world.tile(j, i).is_some_and(|t| t.kind == 203 || t.kind == 25) {
                return false;
            }
        }
    }
    // `:34889-34898` — the pool's jittered top/bottom rows.
    let mut num13 = y;
    if rng.range(0, 4) == 0 {
        num13 = y - rng.range(0, 2);
    }
    let mut num14 = y - num8;
    if rng.range(0, 4) == 0 {
        num14 = y - num8 - rng.range(0, 2);
    }
    // The cell sweep (`:34899-34960`).
    for k in num11..=num12 {
        for l in num9..=num10 {
            shimmer_cell(
                world, rng, x, y, l, k, num9, num5, num6, num7, num3, &mut num13, &mut num14, num8,
            );
        }
    }
    // The num==0 stone shelf (`:34961-35017`).
    if num == 0 {
        shimmer_stone_shelf(world, rng, x, y, num5, num4);
    }
    // The two side openings (`:35018-35019`).
    shimmer_make_biome_opening(world, rng, -1, x - num5, y, num8);
    shimmer_make_biome_opening(world, rng, 1, x + num5, y, num8);
    // The gem-tree tail (`:35020-35055`): 500 rolls, the kind switch
    // 0..6 → 583..589, both side grounds solid, then the growth port.
    let num20 = 70;
    for _ in 0..500 {
        let tx = rng.range(x - num20, x + num20);
        let ty = rng.range(y - 2, y + 3);
        let tree_tile_type = 583 + rng.range(0, 7) as u16;
        if solid_tile(world, tx - 1, ty) && solid_tile(world, tx + 1, ty) {
            // The growth port rides a UnifiedRandom (module doc): one word
            // of the pass stream seeds it, per accepted site.
            let mut tree_rng = UnifiedRandom::new(rng.next_u32() as i32);
            try_growing_tree_by_type(world, tree_tile_type, tx, ty, &mut tree_rng);
        }
    }
    true
}

/// One cell of the biome sweep (`WorldGen.cs:34903-34958`): the liquid
/// wipe, the two row re-jitters, the three ellipse distances (the 1.4/1.2
/// top/bottom split, the num3 shell, the ×2 pool ellipse) and the
/// shell/pool writes. The C#'s `&&`/`||` short-circuits gate the fraction
/// draws — kept verbatim.
#[allow(clippy::too_many_arguments)]
fn shimmer_cell(
    world: &mut World,
    rng: &mut Rng,
    x: i32,
    y: i32,
    l: i32,
    k: i32,
    num9: i32,
    num5: i32,
    num6: i32,
    num7: i32,
    num3: f64,
    num13: &mut i32,
    num14: &mut i32,
    num8: i32,
) {
    // `:34903` — the liquid wipe (the kind bits stay, module doc).
    if let Some(t) = world.tile_mut(l, k) {
        t.liquid = 0;
    }
    if rng.range(0, 4) == 0 {
        *num13 = y - rng.range(0, 2);
    }
    if rng.range(0, 4) == 0 {
        *num14 = y - num8 + rng.range(0, 2);
    }
    // `:34912` — the shell distance, the 1.4 (above) / 1.2 (below) split.
    let y_scale = if k <= y { 1.4 } else { 1.2 };
    let mut num15 = (((l - x).abs() as f64 * (1.0 + rng.next_f64() * 0.02)).powi(2)
        + ((k - y).abs() as f64 * y_scale * (1.0 + rng.next_f64() * 0.02)).powi(2))
    .sqrt() as i32;
    if num15 < num5 {
        // `:34915-34919` — the colour/slope/half-brick wipe + stone retype.
        if let Some(t) = world.tile_mut(l, k) {
            t.set_paint(0);
            t.set_slope(0);
            t.flags &= !FLAG_HALF_BRICK;
            t.kind = 1;
        }
        world.set_wall_paint(l, k, 0);
        // `:34920-34923` — the jittered active margin (the second Next(2)
        // only draws when the first test passes — the C# `&&`).
        if l > num9 + 5 + rng.range(0, 2) && l < x + num5 - 5 - rng.range(0, 2) {
            if let Some(t) = world.tile_mut(l, k) {
                t.flags |= FLAG_ACTIVE;
            }
        }
        if (num15 as f64) < num5 as f64 * 0.9 {
            if let Some(t) = world.tile_mut(l, k) {
                t.wall = 0;
            }
        }
    }
    // `:34929-34937` — the num3 shell + the two deactivation bands.
    num15 = (((l - x).abs() as f64 * (1.0 + rng.next_f64() * 0.02)).powi(2)
        + ((k - y).abs() as f64 * num3 * (1.0 + rng.next_f64() * 0.02)).powi(2))
    .sqrt() as i32;
    if k > *num14 && k < *num13 {
        if let Some(t) = world.tile_mut(l, k) {
            t.flags &= !FLAG_ACTIVE;
        }
    }
    if k < *num13 && num15 < (num7 as f64 * (1.0 + rng.next_f64() * 0.02)) as i32 {
        if let Some(t) = world.tile_mut(l, k) {
            t.flags &= !FLAG_ACTIVE;
        }
    }
    // `:34938-34958` — the ×2 pool ellipse.
    num15 = (((l - x).abs() as f64 * (1.0 + rng.next_f64() * 0.02)).powi(2)
        + (((k - y).abs() * 2) as f64 * (1.0 + rng.next_f64() * 0.02)).powi(2))
    .sqrt() as i32;
    if k < y - 1 || num15 >= (num6 as f64 * (1.0 + rng.next_f64() * 0.025)) as i32 {
        return;
    }
    if k <= y + 2 || num15 != num6 - 1 || rng.range(0, 2) != 0 {
        if let Some(t) = world.tile_mut(l, k) {
            t.flags &= !FLAG_ACTIVE;
        }
    }
    if k >= y {
        if let Some(t) = world.tile_mut(l, k) {
            t.liquid = if k == y { 127 } else { u8::MAX };
            t.set_liquid_kind(LiquidKind::Shimmer);
        }
    }
}

/// The num==0 stone shelf (`WorldGen.cs:34961-35017`): the jittered column
/// walk, the climb to the first active tile, the shrinking stone rows with
/// the `Next(3) < counter` shrink roll and the `num19` budget, then the
/// trailing stone stub + the `PlaceTight` stalactite roll.
fn shimmer_stone_shelf(world: &mut World, rng: &mut Rng, x: i32, y: i32, num5: i32, num4: f64) {
    let mut m = (x as f64 - num5 as f64 * num4) as i32 - rng.range(-15, 1) - 5;
    let num10 = (x as f64 + num5 as f64 * num4) as i32 + rng.range(0, 16);
    let mut num16 = 0;
    while m < num10 {
        // The climb (`:34969-34973`): up to the first active tile. The
        // `num17 > 0` floor is the port's safety valve — the C# would throw
        // on a null tile above the world; the biome's stone shell always
        // stops the climb first.
        let mut num17 = y - 3;
        while num17 > 0 && !world.tile(m, num17).is_some_and(|t| t.active()) {
            num17 -= 1;
        }
        num17 -= 4;
        let mut num18 = rng.range(5, 10);
        let mut num19 = rng.range(15, 21);
        let mut n = m - num18;
        while num18 > 0 {
            n = m - num18;
            while n < m + num18 {
                if let Some(t) = world.tile_mut(n, num17) {
                    t.flags |= FLAG_ACTIVE;
                    t.kind = 1;
                }
                n += 1;
            }
            num16 += 1;
            if rng.range(0, 3) < num16 {
                num16 = 0;
                num18 -= 1;
                m += rng.range(-1, 2);
            }
            if num19 <= 0 {
                num18 -= 1;
            }
            num19 -= 1;
            num17 += 1;
        }
        // The trailing stub (`:34999-35015`).
        n -= rng.range(1, 3);
        for dy in [-2i32, -1, 0] {
            if let Some(t) = world.tile_mut(n, num17 + dy) {
                t.flags |= FLAG_ACTIVE;
                t.kind = 1;
            }
        }
        if rng.range(0, 2) == 0 {
            if let Some(t) = world.tile_mut(n, num17 + 1) {
                t.flags |= FLAG_ACTIVE;
                t.kind = 1;
            }
            place_tight(world, rng, n, num17 + 2, false);
        } else {
            place_tight(world, rng, n, num17 + 1, false);
        }
        // The for-loop increment (`:34967`) draws even on the exit check.
        m += rng.range(9, 14);
    }
}

/// `WorldGen.ShimmerMakeBiomeOpening` (`WorldGen.cs:35059-35102`): the
/// do-while tunnel walk — per step the three-column solid probe, the
/// deactivation sweeps, the two wall wipes and the 1-in-2 y jitter; the
/// walk stops on a clear step or outside the ±100 / 50-tile margins.
fn shimmer_make_biome_opening(
    world: &mut World,
    rng: &mut Rng,
    direction: i32,
    x: i32,
    mut y: i32,
    cave_opening_size: i32,
) {
    let mut num = x;
    let size = cave_opening_size - 1;
    loop {
        num += direction;
        let mut flag = true;
        for i in (y - size + 1)..(y - 1) {
            if solid_tile(world, num, i) {
                flag = false;
            }
            if solid_tile(world, num + direction, i) {
                flag = false;
            }
            if solid_tile(world, num + direction * 2, i) {
                flag = false;
            }
            if let Some(t) = world.tile_mut(num, i) {
                t.flags &= !FLAG_ACTIVE;
            }
        }
        for j in (y - size)..y {
            if let Some(t) = world.tile_mut(num - direction, j) {
                t.flags &= !FLAG_ACTIVE;
            }
        }
        if solid_tile(world, num - direction, y - size - 1) {
            if let Some(t) = world.tile_mut(num - direction, y - size - 1) {
                t.wall = 0;
            }
        }
        if solid_tile(world, num - direction, y) {
            if let Some(t) = world.tile_mut(num - direction, y) {
                t.wall = 0;
            }
        }
        if rng.range(0, 2) == 0 {
            y += rng.range(-1, 2);
        }
        // The do-while condition (`:35101`), negated into the break.
        if flag || num < 50 || num > world.width - 50 || (num - x).abs() > 100 {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stone world below row 200 — the shimmer's canvas. Wide enough for
    /// the live x bands (`0.2·width > 200`) and tall enough for the largest
    /// biome (radius 124) inside InWorld(…, 50) at the band's floor.
    fn stone_world(width: i32, height: i32) -> World {
        let mut w = World::new("shimmer".into(), width, height);
        for x in 0..width {
            for y in 200..height {
                let i = w.index(x, y);
                w.tiles[i].kind = 1;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.world_surface = 200.0;
        w.rock_layer = 240.0;
        w
    }

    fn shimmer_cells(w: &World) -> usize {
        w.tiles
            .iter()
            .filter(|t| t.liquid > 0 && t.liquid_kind() == LiquidKind::Shimmer)
            .count()
    }

    #[test]
    fn the_biome_lands_and_replays() {
        let mut a = stone_world(2000, 900);
        let mut b = stone_world(2000, 900);
        let pa = pass_shimmer(&mut a, 200.0, 240.0, &mut Rng(0x5413));
        let pb = pass_shimmer(&mut b, 200.0, 240.0, &mut Rng(0x5413));
        assert_eq!(a.tiles, b.tiles, "a == b");
        assert_eq!(pa, pb);
        let (x, y) = pa.expect("the site places");
        assert!(shimmer_cells(&a) > 0, "the pool holds shimmer");
        // The pool's surface row carries the half-fill 127 at the centre.
        assert_eq!(a.tile(x, y).unwrap().liquid, 127);
        assert_eq!(a.tile(x, y).unwrap().liquid_kind(), LiquidKind::Shimmer);
        // The site sits in the live band: x < 0.11·width, y below the
        // surface + rock midpoint + 50 floor.
        assert!((200..(2000.0 * 0.11) as i32).contains(&x));
        assert!(y >= (200 + 240) / 2 + 50);
    }

    #[test]
    fn the_sub_vanilla_world_skips_before_any_draw() {
        // A 512-wide world: the x band (200, 56) collapses — the valve
        // skips the pass without touching a tile or the stream.
        let mut w = stone_world(512, 512);
        let before = w.tiles.clone();
        let mut rng = Rng(0x5413);
        let state = rng.0;
        assert_eq!(pass_shimmer(&mut w, 150.0, 170.0, &mut rng), None);
        assert_eq!(w.tiles, before);
        assert_eq!(rng.0, state, "no draws on the skipped pass");
    }

    #[test]
    fn the_skyblock_seed_skips_the_pass() {
        let mut w = stone_world(2000, 900);
        w.seeds |= SeedFlags::SKYBLOCK;
        let before = w.tiles.clone();
        assert_eq!(pass_shimmer(&mut w, 200.0, 240.0, &mut Rng(0x5413)), None);
        assert_eq!(w.tiles, before);
    }

    #[test]
    fn an_evil_box_is_rejected_without_a_write() {
        // `ShimmerMakeBiome`'s acceptance scan (`:34883-34886`) bails on
        // crimstone 203 / ebonstone 25 BEFORE any tile write. A single
        // direct call on an evil site returns false and leaves the world
        // untouched (the retry loop's job is to redraw, tested by the
        // clean-world placement above).
        let mut w = stone_world(2000, 900);
        for x in 0..2000 {
            for y in 200..900 {
                let i = w.index(x, y);
                w.tiles[i].kind = 203;
            }
        }
        let before = w.tiles.clone();
        assert!(!shimmer_make_biome(&mut w, &mut Rng(0x5413), 300, 400));
        assert_eq!(w.tiles, before, "the rejection writes nothing");
    }

    #[test]
    fn an_out_of_world_box_is_rejected() {
        // The InWorld(…, 50) arm (`:34879-34882`): a site too close to the
        // left edge rejects before any write.
        let mut w = stone_world(2000, 900);
        let before = w.tiles.clone();
        assert!(!shimmer_make_biome(&mut w, &mut Rng(0x5413), 60, 400));
        assert_eq!(w.tiles, before);
    }
}
