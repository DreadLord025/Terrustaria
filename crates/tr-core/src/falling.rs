//! The falling blocks — `WorldGen.SpawnFallingBlockProjectile`
//! (`WorldGen.cs:86039-86090`), `BlockBelowMakesSandFall` (`:86585-86604`),
//! `AllowsSandfall` (`:86612-86624`) and `GetSandfallProjData`
//! (`:86514-86570`). The tile-fall flight is `Projectile.AI_010`'s
//! `ai[0] == 1f` mode (`velocity.Y += 0.41f` per tick).
//!
//! The `TileID.Sets.Falling` kinds: 53 sand, 234 ebonsand, 112 crimsand,
//! 116 sandstone, 224, 123 silt, 330-333 the snow variants, 495 — verbatim.
//!
//! The trigger runs from the tile-change path (the C# `TileFrame`'s
//! `:81380` check); the spawned projectile is the app pool's
//! `Behavior::FallingTile`.

use crate::tile::Tile;
use crate::world::World;

/// `TileID.Sets.Falling` — the kinds that fall when unsupported.
pub const FALLING: [u16; 11] = [53, 234, 112, 116, 224, 123, 330, 331, 332, 333, 495];

pub fn is_falling_kind(kind: u16) -> bool {
    FALLING.contains(&kind)
}

/// `GetSandfallProjData` (`WorldGen.cs:86514-86570`): the falling tile kind →
/// the projectile kind and the contact damage (the default 10; the snow
/// variants 0).
pub fn sandfall_proj_data(tile_kind: u16) -> (u16, i32) {
    match tile_kind {
        112 => (56, 10),
        59 => (39, 10),
        116 => (67, 10),
        123 => (71, 10),
        224 => (179, 10),
        234 => (241, 10),
        330 => (411, 0),
        331 => (412, 0),
        332 => (413, 0),
        333 => (414, 0),
        495 => (812, 10),
        // `default: projType = 31` — the sand ball.
        _ => (31, 10),
    }
}

/// `WorldGen.BlockBelowMakesSandFall` (`WorldGen.cs:86585-86604`): the tile
/// below is inactive; or the two below are inactive and the below is
/// non-solid; or the below is tile 165 (the hardened-sand blend).
fn block_below_makes_sand_fall(world: &World, x: i32, y: i32) -> bool {
    if y >= world.height - 1 {
        return false;
    }
    let Some(below) = world.tile(x, y + 1) else {
        return false;
    };
    // `tile.nactive()`: active and not actuator-hidden (`Tile.cs`).
    if !below.active() || below.flags & crate::tile::FLAG_INACTIVE != 0 {
        return true;
    }
    let two_below = world
        .tile(x, y + 2)
        .is_some_and(|t| t.active() && t.flags & crate::tile::FLAG_INACTIVE == 0);
    if !two_below && (!below.active() || !crate::tiledata::TILE_SOLID.has(below.kind)) {
        return true;
    }
    below.active() && below.kind == 165
}

/// `WorldGen.AllowsSandfall` (`WorldGen.cs:86612-86624`): the tile ABOVE the
/// falling block must not be a chest/sensor/statue-anchor (the kinds that
/// sit on sand and must not drop).
fn allows_sandfall(world: &World, x: i32, y: i32) -> bool {
    let Some(above) = world.tile(x, y - 1) else {
        return true;
    };
    if !above.active() {
        return true;
    }
    !matches!(
        above.kind,
        // `TileID.Sets.BasicChest/Fake + 323/88/80/77/26/475/470/597`
        21 | 34 | 72 | 323 | 88 | 80 | 77 | 26 | 475 | 470 | 597
    )
}

/// One falling-tile spawn request: the projectile kind, the contact damage
/// and the spawn pixel position (`i·16 + 8, j·16 + 8`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FallSpawn {
    pub proj_kind: u16,
    pub damage: i32,
    pub x: f32,
    pub y: f32,
    pub tile_kind: u16,
}

/// `WorldGen.SpawnFallingBlockProjectile` (`WorldGen.cs:86039-86090`): the
/// checks + `tileCache.ClearTile()` (the caller clears) + the spawn data.
pub fn check_falling(world: &World, x: i32, y: i32) -> Option<FallSpawn> {
    let tile = world.tile(x, y)?;
    let kind = tile.kind;
    if !is_falling_kind(kind) {
        return None;
    }
    if !block_below_makes_sand_fall(world, x, y) {
        return None;
    }
    if !allows_sandfall(world, x, y) {
        return None;
    }
    let (proj_kind, damage) = sandfall_proj_data(kind);
    Some(FallSpawn {
        proj_kind,
        damage,
        x: x as f32 * 16.0 + 8.0,
        y: y as f32 * 16.0 + 8.0,
        tile_kind: kind,
    })
}

/// The tile-fall flight: `AI_010`'s `ai[0] == 1f` mode — `velocity.Y +=
/// 0.41f` per tick, the landing when the tile below is solid.
pub const TILE_FALL_GRAVITY: f32 = 0.41;

/// 's SetDefaults value ().
pub const TIME_LEFT_DEFAULT: i32 = 3600;

/// Whether the falling block at the pixel position has landed (the tile
/// below its bottom edge is solid).
pub fn landed(world: &World, x: f32, y: f32) -> bool {
    // The 10×10 sand ball box; the bottom edge's tile row.
    let bottom = y + 10.0;
    let tx = (x / 16.0) as i32;
    let ty = (bottom / 16.0) as i32;
    world
        .tile(tx, ty)
        .is_some_and(|t| t.active() && crate::tiledata::TILE_SOLID.has(t.kind))
}

/// Whether the tile can be placed back at the landing position (the C#
/// `WorldGen.PlaceTile`'s occupied check).
pub fn can_place_back(world: &World, x: f32, y: f32, tile_kind: u16) -> bool {
    let tx = (x / 16.0) as i32;
    let ty = (y / 16.0) as i32;
    if !world.in_bounds(tx, ty) {
        return false;
    }
    let Some(t) = world.tile(tx, ty) else {
        return false;
    };
    // The landing spot must be free (or hold the liquid the block sinks
    // into) and not solid.
    !t.active() || (!crate::tiledata::TILE_SOLID.has(t.kind) && t.kind != tile_kind)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{FLAG_ACTIVE, Tile};
    use crate::tiledata::TILE_SOLID;

    fn world() -> World {
        World::new("t".into(), 64, 64)
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
    fn the_sand_with_air_below_spawns_the_sandball() {
        let mut w = world();
        put(&mut w, 10, 20, 53); // sand, air below
        let spawn = check_falling(&w, 10, 20).unwrap();
        assert_eq!(spawn.proj_kind, 31, "the default sand ball (`:86570`)");
        assert_eq!(spawn.damage, 10);
        assert_eq!(spawn.x, 10.0 * 16.0 + 8.0);
        assert_eq!(spawn.tile_kind, 53);
    }

    #[test]
    fn the_sand_on_solid_ground_does_not_fall() {
        let mut w = world();
        put(&mut w, 10, 20, 53);
        put(&mut w, 10, 21, 1); // stone below
        assert!(check_falling(&w, 10, 20).is_none());
    }

    #[test]
    fn the_hardened_sand_blend_below_makes_sand_fall() {
        let mut w = world();
        put(&mut w, 10, 20, 53);
        put(&mut w, 10, 21, 165); // the blend below
        assert!(check_falling(&w, 10, 20).is_some(), "tile 165 below");
    }

    #[test]
    fn the_sandfall_data_map_is_verbatim() {
        assert_eq!(sandfall_proj_data(112), (56, 10));
        assert_eq!(sandfall_proj_data(59), (39, 10));
        assert_eq!(sandfall_proj_data(234), (241, 10));
        assert_eq!(sandfall_proj_data(330), (411, 0), "the snow variants dmg 0");
        assert_eq!(sandfall_proj_data(333), (414, 0));
        assert_eq!(sandfall_proj_data(495), (812, 10));
        assert_eq!(sandfall_proj_data(53), (31, 10), "the default sand ball");
    }

    #[test]
    fn the_sand_under_a_chest_does_not_fall() {
        let mut w = world();
        put(&mut w, 10, 20, 53);
        put(&mut w, 10, 19, 21); // a chest above blocks the fall
        assert!(check_falling(&w, 10, 20).is_none(), "`AllowsSandfall`");
    }

    #[test]
    fn the_landing_check_uses_the_solid_below() {
        let mut w = world();
        put(&mut w, 10, 24, 1);
        // The falling block's bottom edge at y px 394 sits inside the solid
        // tile row 24 → landed.
        assert!(landed(&w, 160.0, 384.0));
        // Higher up: not landed.
        assert!(!landed(&w, 160.0, 300.0));
    }

    #[test]
    fn the_placement_needs_a_free_spot() {
        let mut w = world();
        put(&mut w, 10, 25, 1);
        assert!(!can_place_back(&w, 160.0, 400.0, 53), "the solid spot");
        assert!(can_place_back(&w, 160.0, 300.0, 53), "the air spot");
    }

    #[test]
    fn the_falling_set_is_verbatim() {
        for kind in FALLING {
            assert!(is_falling_kind(kind));
        }
        assert!(!is_falling_kind(1));
        assert!(TILE_SOLID.has(53), "the sand is solid");
    }
}
