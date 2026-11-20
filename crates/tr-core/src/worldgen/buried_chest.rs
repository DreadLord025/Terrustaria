//! `WorldGen.AddBuriedChest` (`WorldGen.cs:35871-37568`) — the placement
//! core the chest passes ride: the down-scan for the solid floor, the
//! shimmer/planter-bulb aborts, the 5×5 boulder/altar exclusion, the
//! `flag15` style default, the family style dispatch, `PlaceChest`
//! (`:57813-57846`) with the `NearOtherChests` exclusion (`Chest.cs:543`),
//! and the `trySlope` flatten/restore. Plus the `Surface Chests` pass
//! (`:17130-17210`).
//!
//! Deviations (the S1-A-4 seam, ANALYSIS "World chest loot"): the
//! stocking block (`:36218-37550`) lives in the tr-app hook
//! (`chest_stock::stock_placed_chest`) — the family MAIN-ITEM draws
//! (the desert/ice `SelectRandom` tables, the getGoodWorld `Next(num)`
//! overrides) ride the hook, not this core; the core resolves the
//! style/kind dispatch from the same world-state probes without
//! re-drawing them (the pass-granularity stand-in). The `errorWorld`
//! style shuffle (`:35986-36014`) and the `noTraps` gold→mahogany swap
//! (`:36201-36205`) DO draw here — they change the placed tile, not the
//! loot. The `dualDungeons` desert-loot alternation (`:36034-36038`) and
//! the `notTheBees` beach band (`:36050-36063`) are unported seed arms
//! (the port's beach-band probe reads false); the `Main.starGame` /
//! `vampireSeed` / `rainbowStuff` loot arms ride the hook's deferrals.
//! `PlaceChest`'s `TileObject.CanPlace` is the port's `place_chest`
//! 2×2 clearance (the structures.rs slice); the chest record rides the
//! stocking hook.

use super::noise::Rng;
use super::structures::place_chest;
use crate::tile::{FLAG_ACTIVE, LiquidKind};
use crate::tiledata::{SET_BASIC_CHEST, SET_BOULDERS, TILE_SOLID, TILE_SOLID_TOP};
use crate::world::World;

/// `WorldGen.beachDistance` (380 — the `evil.rs` slice).
const BEACH_DISTANCE: i32 = 380;

/// `Main.wallDungeon` (`Main.cs:10598-10606`) — the `evil.rs` slice.
const WALL_DUNGEON: [u16; 9] = [7, 8, 9, 94, 95, 96, 97, 98, 99];

/// The seed flags the dispatch reads — the step adapter resolves them
/// once (the `GenOptions`/micro-flag bundle).
#[derive(Clone, Copy, Default)]
pub(crate) struct ChestSeedFlags {
    pub remix: bool,
    pub get_good: bool,
    pub drunk: bool,
    pub anniversary: bool,
    pub not_the_bees: bool,
    pub dont_starve: bool,
    pub no_traps: bool,
    pub error_world: bool,
    pub crimson: bool,
    pub crimson_left: bool,
}

/// `WorldGen.SolidTile(int, int)` (`WorldGen.cs:70111-70131`) — the
/// `edit/kill.rs::solid_tile` slice (OOB reads false via the try/catch).
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| {
        t.active()
            && !t.inactive()
            && TILE_SOLID.has(t.kind)
            && !TILE_SOLID_TOP.has(t.kind)
            && !t.half_brick()
            && t.slope() == 0
    })
}

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9695`).
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `WorldGen.IsUndergroundDesert` (`WorldGen.cs:35755-35781`): the
/// surface/15-85 % band gates, then the ±15 wall-187/216 probe.
fn is_underground_desert(world: &World, x: i32, y: i32, world_surface: f64, rock_layer: f64, remix: bool) -> bool {
    if (y as f64) < world_surface {
        return false;
    }
    if (x as f64) < world.width as f64 * 0.15 || (x as f64) > world.width as f64 * 0.85 {
        return false;
    }
    if remix && (y as f64) > rock_layer {
        return false;
    }
    for i in x - 15..=x + 15 {
        for j in y - 15..=y + 15 {
            if world.tile(i, j).is_some_and(|t| t.wall == 187 || t.wall == 216) {
                return true;
            }
        }
    }
    false
}

/// `WorldGen.IsDungeon` (`WorldGen.cs:35783-35798`).
fn is_dungeon(world: &World, x: i32, y: i32, world_surface: f64) -> bool {
    if (y as f64) < world_surface || y >= world.height || x < 0 || x >= world.width {
        return false;
    }
    world.tile(x, y).is_some_and(|t| WALL_DUNGEON.contains(&t.wall))
}

/// `WorldGen.IsLockedDungeonBiomeChest` (`WorldGen.cs:35850-35869`).
fn is_locked_dungeon_biome_chest(chest_type: u16, style: i32) -> bool {
    match chest_type {
        21 => (23..=27).contains(&style),
        467 => style == 13,
        _ => false,
    }
}

/// `Chest.NearOtherChests` (`Chest.cs:543-557`): the 50×16 BasicChest
/// scan around `(x, y)`.
fn near_other_chests(world: &World, x: i32, y: i32) -> bool {
    for i in x - 25..x + 25 {
        for j in y - 8..y + 8 {
            if world
                .tile(i, j)
                .is_some_and(|t| t.active() && SET_BASIC_CHEST.has(t.kind))
            {
                return true;
            }
        }
    }
    false
}

/// The resolved placement — the core's output for the stocking seam.
pub(crate) struct BuriedChestPlace {
    /// The 2×2 top-left anchor (`PlaceChest(i-1, num7-1)`'s bottom-left
    /// origin lifted one row — the `mini_biomes.rs` precedent).
    pub x: i32,
    pub y: i32,
    pub style: i32,
    pub kind: u16,
    /// `flag14` — the hell arm advanced the counter.
    pub hell: bool,
}

/// `WorldGen.AddBuriedChest` (`WorldGen.cs:35877-37568`), the placement
/// core. `gv_hell` is the `GenVars.hellChest` counter (advanced on the
/// hell arm's success, `:36210-36217`); `hive_high`/`hive_low` are the
/// `GenVars.desertHiveHigh/Low` pair the desert `flag20` reads (`:36041`).
/// The main-item dispatch draws ride the stocking hook (the module note).
#[allow(clippy::too_many_arguments)]
pub(crate) fn add_buried_chest(
    world: &mut World,
    i: i32,
    j: i32,
    main_item: u16,
    not_near_other_chests: bool,
    chest_style: i32,
    try_slope: bool,
    chest_tile_type: u16,
    world_surface: f64,
    rock_layer: f64,
    flags: ChestSeedFlags,
    hive_high: i32,
    hive_low: i32,
    gv_hell: &mut usize,
    hell_item_len: usize,
    rng: &mut Rng,
) -> Option<BuriedChestPlace> {
    let mut chest_tile_type = if chest_tile_type == 0 { 21 } else { chest_tile_type };
    // `:35884-35888` — the getGoodWorld rarity denominator (the loot
    // overrides ride the hook; the core keeps the value for the dispatch).
    let mut num = 15;
    if flags.anniversary {
        num *= 3;
    }
    for k in j..world.height - 10 {
        let mut num2: i32 = -1;
        let mut num3: i32 = -1;
        // `:35906-35913` — the shimmer / planter's-bulb aborts.
        if world.tile(i, k).is_some_and(|t| {
            (t.liquid > 0 && t.liquid_kind() == LiquidKind::Shimmer)
                || (t.active() && t.kind == 231)
        }) {
            return None;
        }
        // `:35914-35944` — the trySlope arm: the style-17 61×61 chest
        // exclusion, then the two top-slope flattens (restored on failure).
        if try_slope
            && world.tile(i, k).is_some_and(|t| {
                t.active() && TILE_SOLID.has(t.kind) && !TILE_SOLID_TOP.has(t.kind)
            })
        {
            if chest_style == 17 {
                for l in i - 30..=i + 30 {
                    for m in k - 30..=k + 30 {
                        if !in_world(world, l, m, 5) {
                            return None;
                        }
                        if world
                            .tile(l, m)
                            .is_some_and(|t| t.active() && (t.kind == 21 || t.kind == 467))
                        {
                            return None;
                        }
                    }
                }
            }
            if world.tile(i - 1, k).is_some_and(|t| t.top_slope()) {
                num2 = world.tile(i - 1, k).map_or(0, |t| t.slope() as i32);
                if let Some(t) = world.tile_mut(i - 1, k) {
                    t.set_slope(0);
                }
            }
            if world.tile(i, k).is_some_and(|t| t.top_slope()) {
                num3 = world.tile(i, k).map_or(0, |t| t.slope() as i32);
                if let Some(t) = world.tile_mut(i, k) {
                    t.set_slope(0);
                }
            }
        }
        // `:35945-35948` — the remix centre-bottom abort.
        if flags.remix
            && (i as f64) > world.width as f64 * 0.37
            && (i as f64) < world.width as f64 * 0.63
            && k > world.height - 250
        {
            return None;
        }
        // `:35949-35959` — the 5×5 boulder/altar exclusion (fluff 100).
        for n in i - 2..=i + 2 {
            for m in k - 2..=k + 2 {
                if in_world(world, n, m, 100)
                    && world.tile(n, m).is_some_and(|t| {
                        t.active() && (SET_BOULDERS.has(t.kind) || t.kind == 26 || t.kind == 237)
                    })
                {
                    return None;
                }
            }
        }
        // `:35960-35963` — the floor gate.
        if !solid_tile(world, i, k) {
            continue;
        }
        let num7 = k;
        // `:35966-35985` — the style default (flag15) and the overrides.
        let mut num9;
        let mut num10: i32 = 0;
        let flag15 = if flags.remix {
            num7 < world.height - 400
        } else {
            (num7 as f64) >= world_surface + 25.0
        };
        num9 = if flag15 || num10 > 0 { 1 } else { 0 };
        if chest_style >= 0 {
            num9 = chest_style;
        }
        // `:35982-35985` — mainItemInChest >= 0 always (the default 0).
        num10 = main_item as i32;
        // `:35986-36014` — the errorWorld style shuffle (a placement draw).
        if flags.error_world && chest_tile_type == 21 && rng.range(0, 2) == 0 {
            let mut num11: i32 = -1;
            match num9 {
                1 | 11 | 12 | 13 | 15 => num11 = 6,
                0 => num11 = 5,
                _ => {}
            }
            if num11 > -1 {
                if rng.range(0, 3) == 0 {
                    num11 = rng.range(0, 52);
                    while (18..=27).contains(&num11) {
                        num11 = rng.range(0, 52);
                    }
                }
                num9 = num11;
            }
        }
        // `:36015-36018` — the seed combo flags.
        let flag16 = flags.remix && flags.get_good;
        let flag17 = flags.drunk && flags.anniversary;
        let flag18 = flags.not_the_bees && flags.anniversary;
        let flag19 = chest_tile_type == 21;
        let mut flag14 = false;
        // `:36023-36048` — the desert arm (flag3): the style/kind swap
        // fires on the world-state probe; the main pick rides the hook.
        if (chest_tile_type == 467 && chest_style == 10)
            || ((chest_tile_type == 21 && chest_style == 42) & flag16)
            || (num10 == 0
                && num7 <= world.height - 205
                && is_underground_desert(world, i, k, world_surface, rock_layer, flags.remix))
        {
            num9 = 10;
            chest_tile_type = 467;
            if (flag16 && chest_style == 42) & flag19 {
                num9 = 42;
                chest_tile_type = 21;
            }
            // flag20 (`:36041`) — the hive-depth test; the dualDungeons
            // alternation is unported (the module note). The main pick
            // (SelectRandom + the getGoodWorld override) rides the hook.
            let _flag20 = num7 > (hive_high * 3 + hive_low * 4) / 7;
        }
        // `:36050-36063` — the notTheBees beach band (flag21): unported
        // (the module note) — the band probe reads false.
        let flag21 = false;
        // `:36064-36088` — the ice arm (flag2).
        let floor_kind = world.tile(i, k).map_or(0, |t| t.kind);
        if (chest_tile_type == 21 && chest_style == 11)
            || (chest_tile_type == 467 && chest_style == 24)
            || (num10 == 0
                && (num7 as f64) >= world_surface + 25.0
                && num7 <= world.height - 205
                && ((matches!(floor_kind, 147 | 161 | 162 | 197)) | flag21))
        {
            if chest_tile_type == 21 {
                num9 = 11;
            }
            if flag17 && chest_tile_type == 21 && chest_style == 34 {
                num9 = 34;
            }
            // The main pick + the remix swap + the 1/20 override ride the hook.
        }
        // `:36089-36114` — the jungle arm (flag4).
        if (chest_tile_type == 21
            && (chest_style == 10
                || ((chest_style == 8 || chest_style == 29) && false /* dualDungeons */)
                || ((chest_style == 29) & flag18)))
            || ((chest_tile_type == 467 && chest_style == 11) & flag18)
            || num10 == 211
            || num10 == 212
            || num10 == 213
            || num10 == 753
        {
            // GeneratingDungeon is false outside the dungeon pass.
            num9 = 10;
            chest_tile_type = 21;
            if flag18 {
                if (chest_style == 29) & flag19 {
                    num9 = 29;
                    chest_tile_type = 21;
                } else if chest_style == 11 && !flag19 {
                    num9 = 11;
                    chest_tile_type = 467;
                }
            }
        }
        // `:36115-36125` — the hell arm (flag8/flag14): the main item is
        // the shuffled table's current entry (the counter advances on
        // success below); the getGoodWorld override rides the hook.
        if chest_tile_type == 21 && (chest_style == 4 || (num7 > world.height - 205 && num10 == 0))
        {
            num9 = 4;
            flag14 = true;
        }
        // `:36134-36150` — the living-tree arm (flag6): the wall-244 probe
        // at the anchor's upper-left decides style 12 vs the wood reset.
        if chest_tile_type == 21 && num9 == 12 {
            if world.tile(i - 1, num7 - 1).map_or(0, |t| t.wall) != 244 {
                num9 = 0;
                num10 = 0;
            }
        }
        // `:36163-36171` — the dungeon / locked-biome flags (flag9/flag10)
        // gate loot only (the hook's ctx.dungeon probe).
        let _flag9 = (chest_tile_type == 21 && num9 != 0 && is_dungeon(world, i, k, world_surface))
            || is_locked_dungeon_biome_chest(chest_tile_type, num9);
        // `:36176-36196` — the water arm (flag12): the remix evil swap.
        if chest_tile_type == 21 && (num9 == 13 || num10 == 159 || num10 == 65 || num10 == 158 || num10 == 2219)
        {
            if flags.remix && !flags.get_good && !flags.anniversary {
                let mut flag22 = flags.crimson;
                if flags.drunk {
                    flag22 = (flags.crimson_left && i < world.width / 2)
                        || (!flags.crimson_left && i > world.width / 2);
                }
                if flag22 {
                    num9 = 43;
                } else {
                    chest_tile_type = 467;
                    num9 = 3;
                }
            }
        }
        // `:36197-36200` — the 939 water override (num10 == 0 only).
        if (num10 == 939 || (chest_tile_type == 21 && num9 == 15) || (chest_tile_type == 467 && num9 == 2))
            && num10 == 0
        {
            num10 = 939;
        }
        // `:36201-36205` — the noTraps gold→mahogany swap (a placement draw).
        if flags.no_traps && num9 == 1 && chest_tile_type == 21 && (!flags.remix || rng.range(0, 3) == 0)
        {
            num9 = 4;
            chest_tile_type = 467;
        }
        let _ = num; // the getGoodWorld denominator — the hook's overrides
        // `:36206` — PlaceChest(i-1, num7-1): the boulder-under veto, the
        // NearOtherChests exclusion, the 2×2 place. The port's place_chest
        // takes the TOP-left (the C# origin is the bottom-left, one row up).
        let (ax, ay) = (i - 1, num7 - 2);
        let boulder_under = [ay + 2].iter().any(|&yy| {
            [ax, ax + 1].iter().any(|&xx| {
                world.tile(xx, yy).is_some_and(|t| SET_BOULDERS.has(t.kind))
            })
        });
        let mut placed = false;
        if !boulder_under {
            let near = not_near_other_chests && near_other_chests(world, ax - 1, ay);
            if !near {
                placed = place_chest(world, ax, ay, num9);
            }
        }
        if placed {
            // `:36210-36217` — the hell counter advance.
            if flag14 {
                *gv_hell += 1;
                if *gv_hell >= hell_item_len {
                    *gv_hell = 0;
                }
            }
            return Some(BuriedChestPlace { x: ax, y: ay, style: num9, kind: chest_tile_type, hell: flag14 });
        }
        // `:37554-37564` — the trySlope restore on failure.
        if try_slope {
            if num2 > -1 {
                if let Some(t) = world.tile_mut(i - 1, k) {
                    t.set_slope(num2 as u8);
                }
            }
            if num3 > -1 {
                if let Some(t) = world.tile_mut(i, k) {
                    t.set_slope(num3 as u8);
                }
            }
        }
        return None;
    }
    None
}

/// `WorldGen.oceanDepths` (`WorldGen.cs:10963-10973`) — the `evil.rs`
/// slice.
fn ocean_depths(world: &World, x: i32, y: i32, ocean_level: f64) -> bool {
    if y as f64 > ocean_level {
        return false;
    }
    x < BEACH_DISTANCE || x > world.width - BEACH_DISTANCE
}

/// The `Surface Chests` pass (`WorldGen.cs:17130-17210`): `width * 0.005`
/// chests in the surface band, the wall-2/59/244 gate (remix bypasses),
/// the ±50 living-tree search over active origins, the 2000-failure
/// budget. The skyblock/noSurface gates live in the step adapter.
/// Sub-vanilla size guard: the x bands need > 600 wide and the y band
/// needs surfaceLow < surface — the C# `Next` throws otherwise.
#[allow(clippy::too_many_arguments)]
pub(crate) fn pass_surface_chests(
    world: &mut World,
    gv: &mut super::pipeline::GenVars,
    world_surface_low: f64,
    world_surface: f64,
    rock_layer: f64,
    flags: ChestSeedFlags,
    rng: &mut Rng,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
) {
    if world.width <= 600 || (world_surface_low as i32) >= world_surface as i32 {
        return;
    }
    let ocean_level = (world_surface + rock_layer) / 2.0 + 40.0;
    let count = (world.width as f64 * 0.005) as i32;
    for _ in 0..count {
        let mut flag = false;
        let mut num2 = 0;
        while !flag {
            // `:17147-17160` — the point pair; remix redraws the y over
            // the deep band, classic re-draws both out of the oceans.
            let mut num3 = rng.range(200, world.width - 200);
            let mut num4 = rng.range(world_surface_low as i32, world_surface as i32);
            if flags.remix {
                num4 = rng.range(world.height - 400, world.height - 150);
            } else {
                while ocean_depths(world, num3, num4, ocean_level) {
                    num3 = rng.range(300, world.width - 300);
                    num4 = rng.range(world_surface_low as i32, world_surface as i32);
                }
            }
            // `:17161-17194` — the wall gate / the living-tree search.
            let mut flag2 = false;
            let mut flag3 = false;
            let cell = world.tile(num3, num4);
            if !cell.is_some_and(|t| t.active()) {
                let wall = cell.map_or(0, |t| t.wall);
                if wall == 2 || wall == 59 || wall == 244 || flags.remix {
                    if wall == 244 {
                        flag2 = true;
                    }
                    flag3 = true;
                }
            } else {
                let mut num8 = 1;
                for j in (num3 - 50..=num3 + 50).step_by(2) {
                    for k in (num4 - 50..=num4 + 50).step_by(2) {
                        if (k as f64) < world_surface
                            && world.tile(j, k).is_some_and(|t| !t.active() && t.wall == 244)
                            && rng.range(0, num8) == 0
                        {
                            flag2 = true;
                            num8 += 1;
                            flag3 = true;
                            num3 = j;
                            num4 = k;
                        }
                    }
                }
            }
            // `:17195-17206` — the place + the budget.
            let placed = if flag3 {
                add_buried_chest(
                    world, num3, num4, 0, true, if flag2 { 12 } else { -1 }, false, 0,
                    world_surface, rock_layer, flags,
                    gv.desert_hive_high, gv.desert_hive_low,
                    &mut gv.hell_chest, gv.hell_chest_item.len(), rng,
                )
            } else {
                None
            };
            if let Some(p) = placed {
                super::chest_stock::stock_placed_chest(
                    world, stock.as_deref_mut(), rng, p.x, p.y, p.style, p.kind, 0, true,
                );
                flag = true;
            } else {
                num2 += 1;
                if num2 >= 2000 {
                    flag = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stone world with an air pocket at rows 60-63 over the floor row
    /// 64 — a chest at x=50 scans down to k=64 and anchors at (49, 62).
    fn pocket_world() -> World {
        let mut world = World::new("bc".into(), 800, 500);
        for y in 0..500 {
            for x in 0..800 {
                let i = world.index(x, y);
                world.tiles[i].kind = 1;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for y in 60..64 {
            for x in 47..54 {
                let i = world.index(x, y);
                world.tiles[i].flags &= !FLAG_ACTIVE;
            }
        }
        world
    }

    fn flags() -> ChestSeedFlags {
        ChestSeedFlags::default()
    }

    #[test]
    fn buried_chest_places_on_the_floor() {
        let mut world = pocket_world();
        let mut hell = 0usize;
        let p = add_buried_chest(
            &mut world, 50, 60, 0, false, -1, false, 0,
            100.0, 200.0, flags(), 500, 0, &mut hell, 5, &mut Rng(42),
        );
        let p = p.expect("the pocket floor takes the chest");
        assert_eq!((p.x, p.y), (49, 62));
        assert_eq!(p.style, 0, "above worldSurface+25 → the wood default");
        assert!(world.tile(49, 62).is_some_and(|t| t.active() && t.kind == 21));
    }

    #[test]
    fn buried_chest_gold_style_below_the_surface_pad() {
        let mut world = pocket_world();
        let mut hell = 0usize;
        // worldSurface 20 → the floor row 64 is below 20+25 → style 1.
        let p = add_buried_chest(
            &mut world, 50, 60, 0, false, -1, false, 0,
            20.0, 200.0, flags(), 500, 0, &mut hell, 5, &mut Rng(42),
        );
        assert_eq!(p.unwrap().style, 1);
    }

    #[test]
    fn buried_chest_shimmer_column_aborts() {
        let mut world = pocket_world();
        let i = world.index(50, 61);
        world.tiles[i].liquid = 255;
        world.tiles[i].flags |= crate::tile::FLAG_LIQUID_SHIMMER;
        let mut hell = 0usize;
        let p = add_buried_chest(
            &mut world, 50, 60, 0, false, -1, false, 0,
            20.0, 200.0, flags(), 500, 0, &mut hell, 5, &mut Rng(42),
        );
        assert!(p.is_none(), "the shimmer aborts the scan");
    }

    #[test]
    fn buried_chest_boulder_exclusion_aborts() {
        let mut world = pocket_world();
        // A boulder (138) inside the 5×5 box — but the fluff-100 gate
        // needs the cell ≥100 from every edge; move the pocket deep.
        let mut world2 = World::new("bc2".into(), 800, 500);
        for y in 0..500 {
            for x in 0..800 {
                let i = world2.index(x, y);
                world2.tiles[i].kind = 1;
                world2.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for y in 260..264 {
            for x in 347..354 {
                let i = world2.index(x, y);
                world2.tiles[i].flags &= !FLAG_ACTIVE;
            }
        }
        let b = world2.index(350, 262);
        world2.tiles[b].kind = 138;
        world2.tiles[b].flags |= FLAG_ACTIVE;
        let mut hell = 0usize;
        let p = add_buried_chest(
            &mut world2, 350, 260, 0, false, -1, false, 0,
            20.0, 200.0, flags(), 500, 0, &mut hell, 5, &mut Rng(42),
        );
        assert!(p.is_none(), "the boulder aborts the scan");
        let _ = world;
    }

    #[test]
    fn near_other_chests_blocks_the_second() {
        let mut world = pocket_world();
        let mut hell = 0usize;
        let p1 = add_buried_chest(
            &mut world, 50, 60, 0, true, -1, false, 0,
            20.0, 200.0, flags(), 500, 0, &mut hell, 5, &mut Rng(42),
        );
        assert!(p1.is_some());
        // A second chest 4 columns away — inside the 50×16 exclusion.
        for y in 60..64 {
            for x in 54..58 {
                let i = world.index(x, y);
                world.tiles[i].flags &= !FLAG_ACTIVE;
            }
        }
        let p2 = add_buried_chest(
            &mut world, 55, 60, 0, true, -1, false, 0,
            20.0, 200.0, flags(), 500, 0, &mut hell, 5, &mut Rng(42),
        );
        assert!(p2.is_none(), "NearOtherChests refuses the neighbour");
    }

    #[test]
    fn surface_chests_pass_is_deterministic_for_a_seed() {
        let base = pocket_world();
        let mut a = base.clone();
        let mut b = base.clone();
        let mut gva = super::super::pipeline::GenVars::default();
        let mut gvb = super::super::pipeline::GenVars::default();
        pass_surface_chests(&mut a, &mut gva, 10.0, 20.0, 200.0, flags(), &mut Rng(42), None);
        pass_surface_chests(&mut b, &mut gvb, 10.0, 20.0, 200.0, flags(), &mut Rng(42), None);
        assert_eq!(a.tiles, b.tiles, "a == b");
    }

    #[test]
    fn surface_chests_size_guard_draws_nothing() {
        let mut world = World::new("tiny".into(), 100, 100);
        let mut gv = super::super::pipeline::GenVars::default();
        let mut rng = Rng(0x5EED);
        pass_surface_chests(&mut world, &mut gv, 10.0, 20.0, 50.0, flags(), &mut rng, None);
        assert_eq!(rng.0, 0x5EED, "the size guard draws nothing");
    }
}
