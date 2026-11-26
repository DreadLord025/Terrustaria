//! The `Life Crystals` pass (`WorldGen.cs:16755-16852`) + `AddLifeCrystal`
//! (`:31912-31979`) + `AddManaCrystal` (`:31981-32048`): the 2×2 heart
//! crystals (12) on cave floors below the surface band, plus the mana
//! crystals (639) of the errorWorld/noSurface arm.
//!
//! Deviations: the `errorWorld` shuffle (`DoErrorWorldShuffleBlocks`,
//! `:16761`) and the `NotTheBees()` transform (`:16769`) are unported seed
//! transforms (the port's other pass heads skip them the same way); the
//! `Main.starGame` multiplier (`:16779`) is an unported mode; the
//! `tileSolid[56] = false` (getGoodWorld, `:16765`) and `tileSolid[225] =
//! false` (`:16850`) boot-table toggles are not modelled — the port's boot
//! table stands (the `mini_biomes.rs::solid_tile2` deferral precedent); the
//! skyblock `IsTileNearby` spacing gates (`:31935`/`:32004`) are moot — the
//! step adapter gates plain skyblock out (`denySomeGeneration`, `:16757`).

use super::biome_detail::empty_tile_check;
use super::noise::Rng;
use crate::tile::{FLAG_ACTIVE, FLAG_HALF_BRICK, LiquidKind};
use crate::tiledata::TILE_SOLID;
use crate::world::World;

/// `Main.offLimitBorderTiles` (`Main.cs:66581`: 40).
const OFF_LIMIT_BORDER_TILES: i32 = 40;

/// `Main.wallDungeon` (`Main.cs:10598-10606`) — the nine dungeon walls
/// (the `evil.rs::WALL_DUNGEON` slice).
const WALL_DUNGEON: [u16; 9] = [7, 8, 9, 94, 95, 96, 97, 98, 99];

/// `Tile.nactive()` (`Tile.cs:240-247`) — active and not wired-inactive.
fn nactive(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| {
        t.active() && (t.flags & crate::tile::FLAG_INACTIVE) == 0
    })
}

/// `Tile.anyLava()` / `Tile.anyShimmer()` (`Tile.cs:410-435`) — liquid > 0
/// of the kind.
fn any_lava(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| t.liquid > 0 && t.liquid_kind() == LiquidKind::Lava)
}

fn any_shimmer(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| t.liquid > 0 && t.liquid_kind() == LiquidKind::Shimmer)
}

/// `WorldGen.AddLifeCrystal` (`WorldGen.cs:31912-31979`) /
/// `AddManaCrystal` (`:31981-32048`) — identical bodies bar the placed kind
/// (12 vs 639) and the moot skyblock gate. Scan down for the first active
/// `tileSolid` cell (the RAW table — solid-top platforms count), then the
/// lava/shimmer vetoes over the crystal's top row, the 2×2 empty check, the
/// dungeon-wall veto at the floor, the two-solid support cells below (which
/// flatten their slope/half-brick), and the four frame writes (18 px grid).
fn add_crystal(world: &mut World, i: i32, j: i32, kind: u16) -> bool {
    for k in j..world.height {
        let floor_solid = world
            .tile(i, k)
            .is_some_and(|t| t.active() && TILE_SOLID.has(t.kind));
        if !floor_solid {
            continue;
        }
        let num = k - 1;
        // `:31919-31926` — the lava/shimmer vetoes over the top row.
        if any_lava(world, i, num - 1) || any_lava(world, i - 1, num - 1) {
            return false;
        }
        if any_shimmer(world, i, num - 1) {
            return false;
        }
        // `:31927-31930` — the 2×2 box must be empty (ignoreID −1).
        if !empty_tile_check(world, i - 1, i, num - 1, num, -1) {
            return false;
        }
        // `:31931-31934` — no dungeon wall behind the lower-right cell.
        if world
            .tile(i, num)
            .is_some_and(|t| WALL_DUNGEON.contains(&t.wall))
        {
            return false;
        }
        // `:31939-31948` — the two support cells below must be nactive and
        // raw-solid.
        for sx in [i - 1, i] {
            let supported = world
                .tile(sx, num + 1)
                .is_some_and(|t| {
                    t.active()
                        && (t.flags & crate::tile::FLAG_INACTIVE) == 0
                        && TILE_SOLID.has(t.kind)
                });
            if !supported {
                return false;
            }
        }
        // `:31949-31958` — flatten the supports' slopes/half-bricks
        // (`blockType() != 0` ⟺ half-brick or slope > 0, `Tile.cs:202-214`).
        for sx in [i - 1, i] {
            if let Some(t) = world.tile_mut(sx, num + 1) {
                if t.half_brick() || t.slope() != 0 {
                    t.set_slope(0);
                    t.flags &= !FLAG_HALF_BRICK;
                }
            }
        }
        // `:31959-31974` — the four 2×2 frame writes.
        for (dx, dy) in [(-1i32, -1i32), (0, -1), (-1, 0), (0, 0)] {
            if let Some(t) = world.tile_mut(i + dx, num + dy) {
                t.flags |= FLAG_ACTIVE;
                t.kind = kind;
                t.frame_x = if dx == 0 { 18 } else { 0 };
                t.frame_y = if dy == 0 { 18 } else { 0 };
            }
        }
        return true;
    }
    false
}

/// One placement loop of the pass (`:16786-16811` / `:16823-16848`): `count`
/// crystals, each retrying the y/x draw pair until placement or the
/// 10000-failure budget. The classic y band is
/// `((int)(worldSurface * 2 + rockLayer)) / 3 .. height − 300`; remix
/// REDRAWS y over `worldSurface .. height − 400` (the classic draw is still
/// consumed, `:16793-16797`).
fn crystal_loop(
    world: &mut World,
    count: i32,
    world_surface: f64,
    rock_layer: f64,
    remix: bool,
    kind: u16,
    rng: &mut Rng,
) {
    let band_lo = ((world_surface * 2.0 + rock_layer) as i32) / 3;
    for _ in 0..count {
        let mut flag = false;
        let mut num3 = 0;
        while !flag {
            let mut j = rng.range(band_lo, world.height - 300);
            if remix {
                j = rng.range(world_surface as i32, world.height - 400);
            }
            let x = rng.range(OFF_LIMIT_BORDER_TILES, world.width - OFF_LIMIT_BORDER_TILES);
            if add_crystal(world, x, j, kind) {
                flag = true;
            } else {
                num3 += 1;
                if num3 >= 10000 {
                    flag = true;
                }
            }
        }
    }
}

/// The `Life Crystals` pass (`WorldGen.cs:16755-16852`). The skyblock gate
/// and the unported seed transforms live in the step adapter / module note.
/// Sub-vanilla size guard (the mining-explosives convention): the x band
/// needs > 80 wide, the classic y band needs height − 300 above its floor,
/// and a remix world's redraw band needs height − 400 above the surface —
/// the C# `Next` throws otherwise; the port skips before any draw.
pub(crate) fn pass_life_crystals(
    world: &mut World,
    world_surface: f64,
    rock_layer: f64,
    remix: bool,
    tenth_anniversary: bool,
    mana_arm: bool,
    rng: &mut Rng,
) {
    let band_lo = ((world_surface * 2.0 + rock_layer) as i32) / 3;
    if world.width <= OFF_LIMIT_BORDER_TILES * 2
        || world.height - 300 <= band_lo
        || (remix && world.height - 400 <= world_surface as i32)
    {
        return;
    }
    // `:16772-16776` — the count and the anniversary multiplier.
    let area = (world.width * world.height) as f64;
    let mut num = area * 2E-05;
    if tenth_anniversary {
        num *= 1.2;
    }
    crystal_loop(world, num as i32, world_surface, rock_layer, remix, 12, rng);
    // `:16812-16849` — the errorWorld/noSurface mana-crystal arm.
    if mana_arm {
        let mut num = area * 1E-05;
        if tenth_anniversary {
            num *= 1.2;
        }
        crystal_loop(world, num as i32, world_surface, rock_layer, remix, 639, rng);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stone world with an air pocket at rows 60-63 over a solid floor
    /// row 64, columns 48-53 — a crystal at x=50 finds the floor at k=64
    /// (num=63), the 2×2 box rows 62-63 is empty, the supports row 64 solid.
    fn pocket_world() -> World {
        let mut world = World::new("lc".into(), 200, 500);
        for y in 0..500 {
            for x in 0..200 {
                let i = world.index(x, y);
                world.tiles[i].kind = 1;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for y in 60..64 {
            for x in 48..54 {
                let i = world.index(x, y);
                world.tiles[i].flags &= !FLAG_ACTIVE;
            }
        }
        world
    }

    #[test]
    fn add_life_crystal_places_the_2x2_on_the_floor() {
        let mut world = pocket_world();
        assert!(add_crystal(&mut world, 50, 60, 12));
        for (dx, dy, fx, fy) in [(-1, -1, 0, 0), (0, -1, 18, 0), (-1, 0, 0, 18), (0, 0, 18, 18)] {
            let t = world.tile(50 + dx, 63 + dy).unwrap();
            assert!(t.active() && t.kind == 12);
            assert_eq!((t.frame_x, t.frame_y), (fx, fy));
        }
    }

    #[test]
    fn add_life_crystal_vetoes_lava_over_the_box() {
        let mut world = pocket_world();
        // The veto cell (i, num − 1) = (50, 62) — lava in the box's top row.
        let i = world.index(50, 62);
        world.tiles[i].liquid = 255;
        world.tiles[i].flags |= crate::tile::FLAG_LIQUID_LAVA;
        assert!(!add_crystal(&mut world, 50, 60, 12));
    }

    #[test]
    fn add_life_crystal_vetoes_the_dungeon_wall() {
        let mut world = pocket_world();
        let __bi = world.index(50, 63);
        world.tiles[__bi].wall = 7;
        assert!(!add_crystal(&mut world, 50, 60, 12));
    }

    #[test]
    fn add_life_crystal_flattens_a_sloped_support() {
        let mut world = pocket_world();
        let s = world.index(49, 64);
        world.tiles[s].flags |= FLAG_HALF_BRICK;
        assert!(add_crystal(&mut world, 50, 60, 12));
        assert!(!world.tiles[s].half_brick(), "the support flattens");
    }

    #[test]
    fn life_crystals_pass_is_deterministic_for_a_seed() {
        let base = pocket_world();
        let mut a = base.clone();
        let mut b = base.clone();
        // width 200 > 80, height 500 − 300 = 200 > band ((20*2+100)/3=46).
        pass_life_crystals(&mut a, 20.0, 100.0, false, false, false, &mut Rng(42));
        pass_life_crystals(&mut b, 20.0, 100.0, false, false, false, &mut Rng(42));
        assert_eq!(a.tiles, b.tiles, "a == b");
    }

    #[test]
    fn life_crystals_size_guard_draws_nothing() {
        let mut world = World::new("tiny".into(), 60, 100);
        let mut rng = Rng(0x5EED);
        pass_life_crystals(&mut world, 20.0, 50.0, false, false, false, &mut rng);
        assert_eq!(rng.0, 0x5EED, "the size guard draws nothing");
    }

    #[test]
    fn mana_arm_places_the_639_family() {
        let mut world = pocket_world();
        // area 200*500 = 100000 → 2 hearts + 1 mana crystal.
        pass_life_crystals(&mut world, 20.0, 100.0, false, false, true, &mut Rng(7));
        let mana = world.tiles.iter().filter(|t| t.active() && t.kind == 639).count();
        assert!(mana >= 4, "the mana arm places a 2×2 ({mana})");
    }
}
