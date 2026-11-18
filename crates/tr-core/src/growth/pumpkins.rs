//! The pumpkin-growth family of the world-growth ticks (S1-K-3e split out of
//! `growth.rs`; behaviour frozen - verbatim move). M5-SEED-2's
//! `halloweenGen` / `endlessHalloween` pumpkins: `WorldGen.PlacePumpkin`
//! (`:52438-52488`), `WorldGen.GrowPumpkin` (`:52490-52552`) and the
//! `GrassPlantsEvilPlantsAndPumpkinsOnSurface` holiday arm (`:20067-20086`).
//! The public seams stay re-exported from `growth`.

use crate::tile::FLAG_ACTIVE;
use crate::UnifiedRandom;
use crate::world::World;

// --- M5-SEED-2: the `halloweenGen` / `endlessHalloween` pumpkins ------------

/// `WorldGen.PlacePumpkin` (`WorldGen.cs:52438-52488`): the 2x2 pumpkin
/// (tile 254) at `(x-1..x, superY-1..superY)`, the style row
/// `genRand.Next(6) * 36` in `frameY` and the 0/18 columns in `frameX`.
///
/// The style roll is drawn BEFORE the bounds guard (`:52440-52445`), so an
/// out-of-bounds call still spends it. The 2x2 must be clear (any active tile
/// except the 3/73/113/110 set — and 185 while its `frameY == 0` — blocks, as
/// does any liquid) and stand on solid grass/mushroom
/// ([`crate::edit::solid_tile`], types 2/109).
pub fn place_pumpkin(world: &mut World, x: i32, super_y: i32, rng: &mut UnifiedRandom) {
    // `:52440-52441`: `ushort type = 254; int num = WorldGen.genRand.Next(6) * 36;`
    let num = rng.next_max(6) * 36;
    // `:52442-52445`: the bounds guard — after the roll.
    if x < 5
        || x > world.width as i32 - 5
        || super_y < 5
        || super_y > world.height as i32 - 5
    {
        return;
    }
    let mut flag = true;
    for i in (x - 1)..(x + 1) {
        for j in (super_y - 1)..(super_y + 1) {
            if let Some(t) = world.tile(i, j) {
                if t.active()
                    && !matches!(t.kind, 3 | 73 | 113 | 110)
                    && !(t.kind == 185 && t.frame_y == 0)
                {
                    flag = false;
                }
                if t.liquid > 0 {
                    flag = false;
                }
            }
        }
        // `:52464-52467`: the ground under BOTH columns.
        let ground = world.tile(i, super_y + 1).map(|t| t.kind);
        if !crate::edit::solid_tile(world, i, super_y + 1)
            || !matches!(ground, Some(2 | 109))
        {
            flag = false;
        }
    }
    if !flag {
        return;
    }
    // `:52471-52486` — the write: the two columns (`frameX` 0 / 18) over the
    // two rows (`frameY` = the style row, `+18` on the bottom half), anchored
    // at `(x-1, superY-1)`.
    for (dx, dy, frame_x, frame_y) in [
        (0, 0, 0i16, num as i16),
        (1, 0, 18, num as i16),
        (0, 1, 0, num as i16 + 18),
        (1, 1, 18, num as i16 + 18),
    ] {
        let idx = world.index(x - 1 + dx, super_y - 1 + dy);
        world.tiles[idx].kind = 254;
        world.tiles[idx].frame_x = frame_x;
        world.tiles[idx].frame_y = frame_y;
        world.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
    }
}

/// `WorldGen.GrowPumpkin` (`WorldGen.cs:52490-52552`): widen the pumpkin the
/// tile `(i, j)` belongs to by one 2-tile patch — every one of its 4 tiles
/// shifts `frameX += 36`, so the object moves one style column right in the
/// tilesheet. Only the leftmost 2x2 patch of a ≤4-patch pumpkin grows: the
/// frames must already spell a consistent 2x2 patch (`frameX ==
/// (k - num3) * 18 + num4 * 36`) standing on solid grass (types 2/477/492/109),
/// and a pumpkin already 4 patches wide returns.
///
/// `destroyObject` (`:52492-52495`) gates the KillTile-driven call, which the
/// port never routes here (the worldgen sweep is its only caller), so the port
/// omits the flag.
pub fn grow_pumpkin(world: &mut World, i: i32, j: i32) {
    let Some(t) = world.tile(i, j) else {
        return;
    };
    // `:52497-52502`: the patch's top row — `num2` (the tile's `frameY / 18`
    // folded down to 0/1 by the `-2` walk) is subtracted from `j`, so both a
    // top-row and a bottom-row tile resolve to the pumpkin's top row.
    let mut num2 = (t.frame_y / 18) as i32;
    while num2 > 1 {
        num2 -= 2;
    }
    let num = j - num2;
    // `:52503-52510`: the same fold across the columns gives the left edge
    // and the patch index (each patch is 36 px = 2 tiles wide).
    let mut num3 = (t.frame_x / 18) as i32;
    let mut num4 = 0i32;
    while num3 > 1 {
        num3 -= 2;
        num4 += 1;
    }
    let num3 = i - num3;
    let num5 = num4 * 36;
    // `:52512-52515`: the four-patch cap.
    if num4 >= 4 {
        return;
    }
    // `:52516-52533`: the 2x2 must be this patch of the pumpkin and the row
    // below it must be solid soil.
    let mut flag = false;
    for k in num3..(num3 + 2) {
        for l in num..(num + 2) {
            // `:52524`: the cell must be this patch of the pumpkin — the
            // `frameX` the patch index predicts (`num5` = 36 px per patch).
            let want_x = ((k - num3) * 18 + num5) as i16;
            let matches_patch = world
                .tile(k, l)
                .is_some_and(|t| t.active() && t.kind == 254 && t.frame_x == want_x);
            if !matches_patch {
                flag = true;
            }
        }
        let soil = world.tile(k, num + 2).map(|t| t.kind);
        // `:52529`: the row below must be solid soil.
        let solid = crate::edit::solid_tile(world, k, num + 2);
        if !solid || !matches!(soil, Some(2 | 477 | 492 | 109)) {
            flag = true;
        }
    }
    if flag {
        return;
    }
    // `:52534-52550`: the widen.
    for m in num3..(num3 + 2) {
        for n in num..(num + 2) {
            if let Some(t) = world.tile_mut(m, n) {
                if t.active() && t.kind == 254 {
                    t.frame_x += 36;
                }
            }
        }
    }
}

/// `GrassPlantsEvilPlantsAndPumpkinsOnSurface`'s holiday arm
/// (`WorldGen.cs:20067-20086`): the `SecretSeed.halloweenGen.Enabled ||
/// SecretSeed.endlessHalloween.Enabled` sweep over the surface grass
/// (`Main.tile[i, j].type == 2`, `j` from 50 while `j < Main.worldSurface`,
/// `i` from 40 to `maxTilesX - 40`). Each active grass cell rolls
/// `genRand.Next(15)`; a hit calls [`place_pumpkin`] at `(i, j - 1)` and then
/// [`grow_pumpkin`] `genRand.Next(5)` times.
///
/// Seam: the port's worldgen runs no `GrassPlantsEvilPlantsAndPumpkinsOnSurface`
/// step yet — the M5-SEED-2 pipeline hooks landed the Grass slot
/// (`worldgen/micro_grass.rs`, the `DoDigExtraHoles` host) and the
/// `FinalizeSecretSeeds` tail, not this pass — so this is the pass body the
/// step will call when it lands, flag-gated so a classic world draws nothing.
/// `rng` is the pass's `genRand` stream; the C# reads the worldgen's global
/// `WorldGen.genRand`, so the step supplies the pipeline's stream for this
/// pass name.
pub fn halloween_pumpkins(world: &mut World, rng: &mut UnifiedRandom) {
    // `:20069`: `halloweenGen || endlessHalloween` (the two micro seeds share
    // the arm — `endlessHalloween` also sets `Main.forceHalloweenForever`,
    // `WorldGen.cs:572-574`).
    if !(world
        .micro_seeds
        .contains(crate::seeds::MicroSeedFlags::HALLOWEEN_GEN)
        || world
            .micro_seeds
            .contains(crate::seeds::MicroSeedFlags::ENDLESS_HALLOWEEN))
    {
        return;
    }
    let world_surface = world.world_surface;
    for i in 40..(world.width as i32 - 40) {
        // `:20075`: `for (int j = 50; (double)j < Main.worldSurface; j++)`.
        let mut j = 50;
        while (j as f64) < world_surface {
            // `:20076-20077`: the active grass gate, then the 1-in-15 roll.
            if world
                .tile(i, j)
                .is_some_and(|t| t.active() && t.kind == 2)
                && rng.next_max(15) == 0
            {
                // `:20078-20082`: the pumpkin plus `Next(5)` growth patches.
                place_pumpkin(world, i, j - 1, rng);
                let num = rng.next_max(5);
                for _ in 0..num {
                    grow_pumpkin(world, i, j - 1);
                }
            }
            j += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::Tile;
    use crate::seeds::MicroSeedFlags;
    /// A grass world for the pumpkin sweep (`WorldGen.cs:20067-20086`): one
    /// grass row (type 2, solid) at y 55 with air above it and
    /// `world_surface = 60`, so the sweep's `j in 50..worldSurface` band
    /// covers the row.
    fn pumpkin_world() -> World {
        let mut w = World::new("p".into(), 200, 120);
        for x in 0..200 {
            let i = w.index(x, 55);
            w.tiles[i] = Tile {
                kind: 2,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.world_surface = 60.0;
        w
    }

    // M5-SEED-2: `halloweenGen` / `endlessHalloween`
    // (`WorldGen.cs:20069-20084`) — the surface pumpkin sweep and its two
    // helpers (`PlacePumpkin` `:52438-52488`, `GrowPumpkin` `:52490-52552`).
    #[test]
    fn place_pumpkin_lays_the_two_by_two_over_the_grass_row() {
        // `WorldGen.cs:52471-52486`: the 2x2 of tile 254 at
        // `(x-1..x, superY-1..superY)`, `frameX` 0/18 across the columns and
        // `frameY` = the `Next(6) * 36` style row (+18 on the bottom half).
        let mut w = pumpkin_world();
        let mut rng = UnifiedRandom::new(3);
        place_pumpkin(&mut w, 60, 54, &mut rng);
        let top = w.tile(59, 53).unwrap();
        let right = w.tile(60, 53).unwrap();
        let bot_left = w.tile(59, 54).unwrap();
        let bot_right = w.tile(60, 54).unwrap();
        assert_eq!((top.kind, top.frame_x, top.frame_y), (254, 0, top.frame_y));
        assert_eq!(right.frame_x, 18);
        assert_eq!(bot_left.frame_x, 0);
        assert_eq!(bot_right.frame_x, 18);
        // The style row is one of the six 36-px rows, the bottom half 18 lower.
        assert_eq!(top.frame_y % 36, 0, "a 36-px style row");
        assert_eq!(bot_left.frame_y, top.frame_y + 18);
        assert_eq!(bot_right.frame_y, top.frame_y + 18);
        assert!(top.frame_y < 216, "six style rows: 0..180");
        // The roll is drawn even when the placement is refused (the C# draws
        // `Next(6)` before its bounds guard, `:52441-52445`).
        let mut r1 = UnifiedRandom::new(9);
        place_pumpkin(&mut w, 1, 2, &mut r1); // out of bounds: the guard fires
        // The draw-count observable: a twin stream that drew exactly the one
        // style roll matches the refused call's stream state.
        let mut twin = UnifiedRandom::new(9);
        twin.next_max(6);
        assert_eq!(r1, twin, "the style roll still spends its draw");
    }

    #[test]
    fn grow_pumpkin_widens_one_patch_and_caps_at_four() {
        // `WorldGen.cs:52534-52550`: each accepted call shifts the pumpkin's
        // four tiles `frameX += 36` (one 2-tile patch right in the tilesheet),
        // and `:52512-52515` refuses the fifth patch.
        let mut w = pumpkin_world();
        let mut rng = UnifiedRandom::new(11);
        place_pumpkin(&mut w, 60, 54, &mut rng);
        // The call passes the grass column's own cell (the pumpkin's
        // bottom-right tile), like the C#'s `GrowPumpkin(i, j - 1, 254)`.
        grow_pumpkin(&mut w, 60, 54);
        assert_eq!(w.tile(59, 53).unwrap().frame_x, 36);
        assert_eq!(w.tile(60, 53).unwrap().frame_x, 54);
        assert_eq!(w.tile(59, 54).unwrap().frame_x, 36);
        assert_eq!(w.tile(60, 54).unwrap().frame_x, 54);
        // Three more accepted patches, then the cap: the frames stay put and
        // the tiles are still the pumpkin.
        for _ in 0..2 {
            grow_pumpkin(&mut w, 60, 54);
        }
        assert_eq!(w.tile(59, 53).unwrap().frame_x, 108);
        grow_pumpkin(&mut w, 60, 54);
        assert_eq!(w.tile(59, 53).unwrap().frame_x, 144, "the fourth patch");
        grow_pumpkin(&mut w, 60, 54);
        assert_eq!(w.tile(59, 53).unwrap().frame_x, 144, "the cap: no fifth");
        // The soil gate: dig the row out from under the pumpkin and the widen
        // refuses (`:52529`).
        let mut w = pumpkin_world();
        let mut rng = UnifiedRandom::new(11);
        place_pumpkin(&mut w, 60, 54, &mut rng);
        let i = w.index(59, 55);
        w.tiles[i].flags &= !FLAG_ACTIVE;
        grow_pumpkin(&mut w, 60, 54);
        assert_eq!(w.tile(59, 53).unwrap().frame_x, 0, "no soil, no widen");
    }

    #[test]
    fn the_pumpkin_sweep_runs_only_under_the_holiday_flags() {
        // `WorldGen.cs:20069`: `halloweenGen || endlessHalloween` — the sweep
        // is the ONLY writer of tile 254 here, so its census is the flag's
        // observable, and a classic world draws nothing at all (the stream is
        // untouched).
        let mut classic = pumpkin_world();
        let mut rng = UnifiedRandom::new(5);
        halloween_pumpkins(&mut classic, &mut rng);
        // The draw-count observable: the stream state still equals a pristine
        // seed-5 stream — no draw was spent.
        assert_eq!(rng, UnifiedRandom::new(5), "a classic world spends no draw");
        assert!(classic.tiles.iter().all(|t| t.kind != 254), "no pumpkins");
        // The two seeds share the arm (`:20069`).
        for bit in [
            MicroSeedFlags::HALLOWEEN_GEN,
            MicroSeedFlags::ENDLESS_HALLOWEEN,
        ] {
            let mut w = pumpkin_world();
            w.micro_seeds.insert(bit);
            halloween_pumpkins(&mut w, &mut UnifiedRandom::new(5));
            let planted = w.tiles.iter().filter(|t| t.kind == 254).count();
            assert!(planted >= 4, "the flag plants pumpkins: {planted}");
            // Every pumpkin stands directly on the grass row it was rolled
            // from: the top row's soil is 2 below it, the bottom row's 1.
            let mut checked = 0;
            for x in 40..160 {
                for y in 53..55 {
                    if let Some(t) = w.tile(x, y) {
                        if t.kind == 254 {
                            // The top row's `frameY / 18` is even (0/2/4/…),
                            // the bottom row's odd (1/3/5/…) — the fold
                            // `GrowPumpkin` runs (`:52499-52502`).
                            let soil_row = if (t.frame_y / 18) % 2 == 0 { y + 2 } else { y + 1 };
                            assert!(
                                w.tile(x, soil_row)
                                    .is_some_and(|s| s.active() && s.kind == 2),
                                "the pumpkin at ({x}, {y}) stands on grass"
                            );
                            checked += 1;
                        }
                    }
                }
            }
            assert!(checked > 0, "a pumpkin cell was inspected");
        }
    }

}
