//! The `Water Chests` pass (`GenPassNameID.UnderwaterChests`,
//! `WorldGen.cs:17245-17355`): the ocean-cave treasure chests around the
//! `GenVars.oceanCaveTreasure` anchors, then the water-chest scatter (the
//! 186/4404/277/187 main-item cycle with the 1/10 water-breathing 863
//! override) over the world's water cells — all through the faithful
//! `AddBuriedChest` core (`buried_chest.rs`), style 17 (the water chest).
//!
//! Deviations: the `extraLiquid` arms (the 268 flipper main, the ×3 count,
//! the 6-item treasure list) ride the micro flag; the `Main.starGame` /
//! vampire loot arms ride the S1-A-4 hook's deferrals. The two scatter
//! re-draw loops are UNBOUNDED in the C# (`:17323-17327`/`:17341-17345` —
//! they spin until a water cell appears); the port keeps them verbatim
//! (the generated worlds always carry ocean water; a waterless fixture
//! must not call the scatter — the size guard covers the sub-vanilla
//! bands only).

use super::buried_chest::{add_buried_chest, ChestSeedFlags};
use super::noise::Rng;
use crate::tile::LiquidKind;
use crate::world::World;

/// `WorldGen.beachDistance` (380 — the `evil.rs` slice).
const BEACH_DISTANCE: i32 = 380;

/// `Main.UnderworldLayer` (`Main.cs:2790`: maxTilesY − 200).
fn underworld_layer(world: &World) -> i32 {
    world.height - 200
}

/// `Utils.NextFromList` (`Utils.cs:1213-1216`): one `Next(len)` pick.
fn next_from_list(rng: &mut Rng, list: &[u16]) -> u16 {
    list[rng.range(0, list.len() as i32) as usize]
}

/// The `Water Chests` pass (`WorldGen.cs:17245-17355`). The skyblock gate
/// (`denyAllGeneration`, `:17247`) lives in the step adapter. Sub-vanilla
/// size guard: the scatter bands need > 100 wide, UnderworldLayer > 50
/// and the surface below it — the C# `Next` throws otherwise.
#[allow(clippy::too_many_arguments)]
pub(crate) fn pass_water_chests(
    world: &mut World,
    gv: &mut super::pipeline::GenVars,
    world_surface: f64,
    rock_layer: f64,
    flags: ChestSeedFlags,
    extra_liquid: bool,
    rng: &mut Rng,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
) {
    let uw = underworld_layer(world);
    if world.width <= 100 || uw <= 50 || (world_surface as i32) >= uw {
        return;
    }
    // `:17250-17270` — the ocean-cave treasure anchors.
    for i in 0..gv.num_ocean_cave_treasure as usize {
        // `:17252-17256` — the main pick (the extraLiquid 6-item list).
        let main = if extra_liquid {
            next_from_list(rng, &[863, 186, 277, 187, 4404, 268])
        } else {
            next_from_list(rng, &[863, 186, 277, 187, 4404])
        };
        let (tx, ty) = gv.ocean_cave_treasure[i];
        let mut flag = false;
        let mut num = 2.0f64;
        while !flag && num < 50.0 {
            num += 0.1;
            // `:17262-17264` — the widening box draws and the x shift.
            let mut num2 = rng.range(tx - num as i32, tx + num as i32 + 1);
            let num3 = rng.range(ty - (num as i32) / 2, ty + (num as i32) / 2 + 1);
            num2 = if num2 >= world.width {
                (num2 as f64 + num / 2.0) as i32
            } else {
                (num2 as f64 - num / 2.0) as i32
            };
            // `:17265-17268` — the deep-water gate, then the style-17
            // trySlope place.
            let water = world.tile(num2, num3).is_some_and(|t| {
                t.liquid > 250
                    && (t.liquid_kind() == LiquidKind::Water || flags.not_the_bees || flags.remix)
            });
            if num2 >= 0 && num2 < world.width && num3 >= 0 && num3 < world.height && water {
                if let Some(p) = add_buried_chest(
                    world, num2, num3, main, false, 17, true, 0,
                    world_surface, rock_layer, flags,
                    gv.desert_hive_high, gv.desert_hive_low,
                    &mut gv.hell_chest, gv.hell_chest_item.len(), rng,
                ) {
                    super::chest_stock::stock_placed_chest(
                        world, stock.as_deref_mut(), rng, p.x, p.y, p.style, p.kind, main, false,
                    );
                    flag = true;
                }
            }
        }
    }
    // `:17271-17277` — the scatter count (the extraLiquid ×3).
    let mut num4 = 0;
    let mut num5 = world.width as f64 / 4200.0;
    if extra_liquid {
        num5 *= 3.0;
    }
    let mut j = 0;
    while (j as f64) < 9.0 * num5 {
        j += 1;
        // `:17280-17316` — the main-item cycle with the 1/10 override.
        let mut num6: u16;
        num4 += 1;
        let max_value = if flags.anniversary { 7 } else { 10 };
        if rng.range(0, max_value) == 0 {
            num6 = 863;
        } else {
            match num4 {
                1 => num6 = 186,
                2 => num6 = 4404,
                3 => num6 = 277,
                _ => {
                    if extra_liquid && num4 == 4 {
                        num6 = 268;
                    } else {
                        num6 = 187;
                        num4 = 0;
                    }
                }
            }
        }
        // `:17317-17334` — scatter arm 1: any water above the underworld
        // (the re-draw band's y floor moves 1 → 50 after the first draw).
        let mut flag2 = false;
        let mut num7 = 0;
        while !flag2 {
            let mut num8 = rng.range(50, world.width - 50);
            let mut num9 = rng.range(1, uw);
            while !world.tile(num8, num9).is_some_and(|t| {
                t.liquid >= 250
                    && (t.liquid_kind() == LiquidKind::Water || flags.not_the_bees || flags.remix)
            }) {
                num8 = rng.range(50, world.width - 50);
                num9 = rng.range(50, uw);
            }
            let try_slope = num8 < BEACH_DISTANCE || num8 > world.width - BEACH_DISTANCE;
            if let Some(p) = add_buried_chest(
                world, num8, num9, num6, false, 17, try_slope, 0,
                world_surface, rock_layer, flags,
                gv.desert_hive_high, gv.desert_hive_low,
                &mut gv.hell_chest, gv.hell_chest_item.len(), rng,
            ) {
                super::chest_stock::stock_placed_chest(
                    world, stock.as_deref_mut(), rng, p.x, p.y, p.style, p.kind, num6, false,
                );
                flag2 = true;
            }
            num7 += 1;
            if num7 > 10000 {
                break;
            }
        }
        // `:17335-17352` — scatter arm 2: water below the surface (the
        // remix bypass is NOT in this arm's gate, verbatim).
        let mut flag2 = false;
        let mut num7 = 0;
        while !flag2 {
            let mut num10 = rng.range(50, world.width - 50);
            let mut num11 = rng.range(world_surface as i32, uw);
            while !world.tile(num10, num11).is_some_and(|t| {
                t.liquid >= 250 && (t.liquid_kind() == LiquidKind::Water || flags.not_the_bees)
            }) {
                num10 = rng.range(50, world.width - 50);
                num11 = rng.range(world_surface as i32, uw);
            }
            let try_slope = num10 < BEACH_DISTANCE || num10 > world.width - BEACH_DISTANCE;
            if let Some(p) = add_buried_chest(
                world, num10, num11, num6, false, 17, try_slope, 0,
                world_surface, rock_layer, flags,
                gv.desert_hive_high, gv.desert_hive_low,
                &mut gv.hell_chest, gv.hell_chest_item.len(), rng,
            ) {
                super::chest_stock::stock_placed_chest(
                    world, stock.as_deref_mut(), rng, p.x, p.y, p.style, p.kind, num6, false,
                );
                flag2 = true;
            }
            num7 += 1;
            if num7 > 10000 {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    /// A stone world with a water column at x=100 (rows 60-63, liquid 255)
    /// over the floor row 64 — the scatter's water gate finds it.
    fn water_world() -> World {
        let mut world = World::new("wc".into(), 800, 500);
        for y in 0..500 {
            for x in 0..800 {
                let i = world.index(x, y);
                world.tiles[i].kind = 1;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for y in 60..64 {
            for x in 98..104 {
                let i = world.index(x, y);
                world.tiles[i].flags &= !FLAG_ACTIVE;
                world.tiles[i].liquid = 255;
            }
        }
        world
    }

    #[test]
    fn water_chests_size_guard_draws_nothing() {
        let mut world = World::new("tiny".into(), 80, 100);
        let mut gv = super::super::pipeline::GenVars::default();
        let mut rng = Rng(0x5EED);
        pass_water_chests(
            &mut world, &mut gv, 20.0, 50.0, ChestSeedFlags::default(), false, &mut rng, None,
        );
        assert_eq!(rng.0, 0x5EED, "the size guard draws nothing");
    }

    #[test]
    fn treasure_anchor_places_a_style_17_chest() {
        let mut world = water_world();
        let mut gv = super::super::pipeline::GenVars::default();
        gv.num_ocean_cave_treasure = 1;
        gv.ocean_cave_treasure[0] = (100, 62);
        // width 800 → the scatter count is (int)(9 * 800/4200) = 1; the
        // scatter's unbounded water re-draw finds the column. Run the
        // treasure arm only by zeroing the scatter via the width guard?
        // No — the full pass runs; the water column satisfies both arms.
        pass_water_chests(
            &mut world, &mut gv, 20.0, 200.0, ChestSeedFlags::default(), false, &mut Rng(42), None,
        );
        let chests = world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 21 && t.frame_x >= 17 * 18)
            .count();
        assert!(chests >= 4, "a style-17 2×2 lands ({chests})");
    }

    #[test]
    fn water_chests_pass_is_deterministic_for_a_seed() {
        let base = water_world();
        let mut a = base.clone();
        let mut b = base.clone();
        let mut gva = super::super::pipeline::GenVars::default();
        let mut gvb = super::super::pipeline::GenVars::default();
        gva.num_ocean_cave_treasure = 1;
        gvb.num_ocean_cave_treasure = 1;
        gva.ocean_cave_treasure[0] = (100, 62);
        gvb.ocean_cave_treasure[0] = (100, 62);
        pass_water_chests(&mut a, &mut gva, 20.0, 200.0, ChestSeedFlags::default(), false, &mut Rng(42), None);
        pass_water_chests(&mut b, &mut gvb, 20.0, 200.0, ChestSeedFlags::default(), false, &mut Rng(42), None);
        assert_eq!(a.tiles, b.tiles, "a == b");
    }
}
