//! The block-swap tile replacement — `WorldGen.ReplaceTile`
//! (`WorldGen.cs:62558-62580`) + its validity helpers
//! (`IsTileReplaceable` :62328-62375, `WouldTileReplacementWork`
//! :62705-62737, `MoveReplaceTileAnchor` :62686-62703) + the replacement
//! writes (`ReplaceTile_DoActualReplacement` :62582-62684). The
//! kill-effects (dust/sound/drops/fossil shatter) are app-side
//! (`world_sim/editing/block_swap.rs`); the core records the anchor + the
//! old cell so the caller can run them in the C# order.

use crate::tile::Tile;
use crate::tiledata as td;
use crate::world::World;

use super::{can_pound_tile, kill_tile, TileKill};

/// `WorldGen.IsTileReplaceable` (`WorldGen.cs:62328-62375`): the cell above
/// may forbid the replacement — the cactus 80 / fallen log 488 always, the
/// `PreventsTileReplaceIfOnTopOfIt` set when the kinds differ, with the
/// tree-trunk frame windows and the palm 323's two forbidden columns.
pub fn is_tile_replaceable(world: &World, x: i32, y: i32) -> bool {
    if y < 1 {
        // `:62331` — the `y >= 1` guard SKIPS the above-cell checks and
        // falls through to the trailing `return true` (:62374).
        return true;
    }
    let (Some(tile), Some(above)) = (world.tile(x, y), world.tile(x, y - 1)) else {
        return false;
    };
    if tile.wall == 350 {
        return false;
    }
    if above.active() {
        if above.kind == 80 || above.kind == 488 {
            return false;
        }
        if td::SET_PREVENTS_TILE_REPLACE_IF_ON_TOP_OF_IT.has(above.kind)
            && tile.kind != above.kind
        {
            if td::SET_IS_A_TREE_TRUNK.has(above.kind) {
                let (fx, fy) = (above.frame_x, above.frame_y);
                return (fx == 66 && (0..=44).contains(&fy))
                    || (fx == 88 && (66..=110).contains(&fy))
                    || fy >= 198;
            }
            if above.kind == 323 {
                return above.frame_x != 66 && above.frame_x != 220;
            }
            return false;
        }
    }
    true
}

/// `WorldGen.ReplaceTile_IsValidSolid` (`WorldGen.cs:62739-62750`).
fn valid_solid(kind: u16) -> bool {
    if td::TILE_SOLID.has(kind) && !td::TILE_SOLID_TOP.has(kind) {
        if !td::TILE_ROPE.has(kind) {
            return !crate::tileobject::frame_important(kind);
        }
        return true;
    }
    false
}

/// `WorldGen.WouldTileReplacementWork` (`WorldGen.cs:62705-62737`). The
/// chest-lock term reads `Chest.IsLocked(tile)` — the locked-frame decode
/// (`Chest.cs:301-316`), inlined here (the core has no chest store; the
/// decode is pure tile state).
pub fn would_tile_replacement_work(world: &World, target: u16, x: i32, y: i32) -> bool {
    let Some(tile) = world.tile(x, y) else {
        return false;
    };
    if target as usize >= td::TILE_COUNT {
        return false;
    }
    // `:62712-62715` — the grass family + the ash grass 633 never replace.
    if td::SET_GRASS.has(target) || target == 633 {
        return false;
    }
    // `:62716-62732` — the rope exceptions: a track/platform/planter-box
    // replaces a rope when a same-kind neighbour already holds.
    if td::TILE_ROPE.has(tile.kind)
        && x >= 2
        && x < world.width - 2
        && y >= 2
        && y < world.height - 2
    {
        let left = world.tile(x - 1, y);
        let right = world.tile(x + 1, y);
        if let (Some(l), Some(r)) = (left, right) {
            if target == 314
                && ((l.active() && l.kind == 314) || (r.active() && r.kind == 314))
            {
                return true;
            }
            if td::SET_PLATFORMS.has(target)
                && ((l.active() && td::SET_PLATFORMS.has(l.kind))
                    || (r.active() && td::SET_PLATFORMS.has(r.kind)))
            {
                return true;
            }
            if target == 380
                && ((l.active() && l.kind == 380) || (r.active() && r.kind == 380))
            {
                return true;
            }
        }
    }
    // `:62733-62736` — the validity matrix. The C#'s `&` on bools is the
    // non-short-circuit AND (every operand evaluates; all are side-effect
    // free, so `&` on the computed bools is the faithful shape).
    let solid_bad = !valid_solid(target) || !valid_solid(tile.kind);
    let torch_bad = !td::SET_TORCHES.has(target) || !td::SET_TORCHES.has(tile.kind);
    let campfire_bad = !td::SET_CAMPFIRES.has(target) || !td::SET_CAMPFIRES.has(tile.kind);
    let platform_bad = !td::SET_PLATFORMS.has(target) || !td::SET_PLATFORMS.has(tile.kind);
    let chest_bad = !td::SET_BASIC_CHEST.has(target)
        || !td::SET_BASIC_CHEST.has(tile.kind)
        || chest_is_locked(tile);
    let dresser_bad =
        !td::SET_BASIC_DRESSER.has(target) || !td::SET_BASIC_DRESSER.has(tile.kind);
    let solid_pair_bad = (!valid_solid(target) && !td::SET_PLATFORMS.has(target))
        || (!valid_solid(tile.kind) && !td::SET_PLATFORMS.has(tile.kind));
    !(platform_bad & torch_bad & campfire_bad & solid_bad & solid_pair_bad & chest_bad
        & dresser_bad)
}

/// `Chest.IsLocked(Tile)` (`Chest.cs:301-316`) — the locked-frame decode,
/// the app `chest::is_locked`'s twin (the core cannot reach the app crate).
fn chest_is_locked(t: &Tile) -> bool {
    let fx = t.frame_x;
    if t.kind == 21 {
        return (72..=106).contains(&fx)
            || (144..=178).contains(&fx)
            || (828..=1006).contains(&fx)
            || (1296..=1330).contains(&fx)
            || (1368..=1402).contains(&fx)
            || (1440..=1474).contains(&fx);
    }
    if t.kind == 467 {
        return fx / 36 == 13;
    }
    false
}

/// `WorldGen.MoveReplaceTileAnchor` (`WorldGen.cs:62686-62703`): the
/// chest/dresser/campfire anchor walk-back from the clicked cell.
pub fn move_replace_anchor(world: &World, x: i32, y: i32) -> (i32, i32) {
    let (mut x, mut y) = (x, y);
    if let Some(t) = world.tile(x, y) {
        if td::SET_BASIC_CHEST.has(t.kind) {
            x -= i32::from(t.frame_x) % 36 / 18;
            y -= i32::from(t.frame_y) % 36 / 18;
        }
        if td::SET_BASIC_DRESSER.has(t.kind) {
            x -= i32::from(t.frame_x) % 54 / 18;
            y -= i32::from(t.frame_y) % 36 / 18;
        }
        if td::SET_CAMPFIRES.has(t.kind) {
            x -= i32::from(t.frame_x) % 54 / 18;
            y -= i32::from(t.frame_y) % 36 / 18;
        }
    }
    (x, y)
}

/// The result of a successful `ReplaceTile`: the anchor the write landed
/// on, the OLD cell (the dust/drops read it — the C# `tileSafely` is a
/// LIVE reference, so its dust/drops see the pre-replacement state; the
/// port snapshots), and the natural-extras kills the replacement caused.
pub struct Replaced {
    pub anchor_x: i32,
    pub anchor_y: i32,
    pub old: Tile,
    /// `ReplaceTile_EliminateNaturalExtras`' `KillTile` results (the
    /// above/below vine-adjacent cells) — the app resolves their drops.
    pub extras_kills: Vec<(i32, i32, TileKill)>,
}

/// `WorldGen.ReplaceTile` (`WorldGen.cs:62558-62580`) minus the
/// kill-effects: the two validity gates, the anchor move, then the
/// replacement write. The caller runs the dust (`KillTile_GetTileDustAmount`
/// + `KillTile_MakeTileDust`), the sounds (`KillTile_PlaySounds`), the drops
/// (`KillTile_DropItems(includeLargeObjectDrops: true)`) and the fossil
/// shatter BETWEEN the gates and the write — the C# order (`:62570-62578`);
/// the port splits the write out so the app can keep that order with its
/// item/dust pools. `chest_holds` feeds `CanPoundTile`'s refusal.
pub fn replace_tile_write(
    world: &mut World,
    x: i32,
    y: i32,
    target: u16,
    style: i32,
    chest_holds: impl Fn(i32, i32) -> bool + Copy,
) -> Option<Replaced> {
    let old = world.tile(x, y).copied()?;
    if !would_tile_replacement_work(world, target, x, y) {
        return None;
    }
    if !is_tile_replaceable(world, x, y) {
        return None;
    }
    let (ax, ay) = move_replace_anchor(world, x, y);
    if td::SET_BASIC_CHEST.has(target) {
        // `:62586-62590` — the rigged-chest trip is app-side (the wiring
        // pool lives there); the caller checks `is_rigged` BEFORE this
        // write (the C# trips on the OLD tile).
        replace_area(world, target, style, ax, ay, 2, 2, 0, 0);
    } else if td::SET_BASIC_DRESSER.has(target) {
        replace_area(world, target, style, ax, ay, 3, 2, 0, 0);
    } else if td::SET_CAMPFIRES.has(target) {
        let off_y = if old.frame_y >= 36 { 36 } else { 0 };
        replace_area(world, target, style, ax, ay, 3, 2, 0, off_y);
    } else {
        // `ReplaceTile_EliminateNaturalExtras` runs at the HEAD of the
        // single arm (`WorldGen.cs:62609`), BEFORE the kind write — the
        // kills' cascades see the OLD cell still standing. The chest/
        // dresser/campfire area writes never call it.
        let extras_kills = eliminate_natural_extras(world, ax, ay);
        replace_single(world, target, style, ax, ay, &old, chest_holds);
        return Some(Replaced {
            anchor_x: ax,
            anchor_y: ay,
            old,
            extras_kills,
        });
    }
    Some(Replaced {
        anchor_x: ax,
        anchor_y: ay,
        old,
        extras_kills: Vec::new(),
    })
}

/// `WorldGen.ReplaceTile_EliminateNaturalExtras`
/// (`WorldGen.cs:62649-62662`): the vine-adjacent cells above/below die
/// when the replacement lands (the `ReplaceTileBreakUp`/`BreakDown` sets +
/// the stalactite/stalagmite 165 frame rows). The kills are plain
/// `KillTile(x, y±1)` — the full cascade + sounds + drops, app-resolved
/// through `Replaced::extras_kills`.
fn eliminate_natural_extras(world: &mut World, x: i32, y: i32) -> Vec<(i32, i32, TileKill)> {
    let mut kills = Vec::new();
    if x < 2 || x >= world.width - 2 || y < 2 || y >= world.height - 2 {
        return kills;
    }
    if let Some(up) = world.tile(x, y - 1) {
        if up.active()
            && (td::SET_REPLACE_TILE_BREAK_UP.has(up.kind)
                || (up.kind == 165 && matches!(up.frame_y, 36 | 54 | 90)))
        {
            if let Some(k) = kill_tile(world, x, y - 1, false) {
                kills.push((x, y - 1, k));
            }
        }
    }
    if let Some(down) = world.tile(x, y + 1) {
        if down.active()
            && (td::SET_REPLACE_TILE_BREAK_DOWN.has(down.kind)
                || (down.kind == 165 && matches!(down.frame_y, 0 | 18 | 72)))
        {
            if let Some(k) = kill_tile(world, x, y + 1, false) {
                kills.push((x, y + 1, k));
            }
        }
    }
    kills
}

/// `WorldGen.ReplaceTile_DoActualReplacement_Area`
/// (`WorldGen.cs:62664-62684`): the multi-cell write, then the per-cell
/// `SquareTileFrame` walk (the port's `square_tile_frame_default` — the
/// object drops the support checks produce are discarded like the wiring
/// callers'; a replacement's own cells were just written, so the checks
/// find them intact).
fn replace_area(
    world: &mut World,
    target: u16,
    style: i32,
    ax: i32,
    ay: i32,
    w: i32,
    h: i32,
    off_x: i32,
    off_y: i32,
) {
    for i in 0..w {
        for j in 0..h {
            if let Some(t) = world.tile_mut(ax + i, ay + j) {
                t.kind = target;
                t.frame_x = (style * (w * 18) + i * 18 + off_x) as i16;
                t.frame_y = (j * 18 + off_y) as i16;
                clear_block_paint_and_coating(world, ax + i, ay + j);
            }
        }
    }
    for i in 0..w {
        for j in 0..h {
            let _ = super::square_tile_frame_default(world, ax + i, ay + j);
        }
    }
}

/// `Tile.ClearBlockPaintAndCoating` (`Tile.cs:904-909`).
fn clear_block_paint_and_coating(world: &mut World, x: i32, y: i32) {
    if let Some(t) = world.tile_mut(x, y) {
        t.set_paint(0);
        t.flags &= !crate::tile::FLAG_FULLBRIGHT;
    }
    world.set_invisible_block(x, y, false);
}

/// `WorldGen.ReplaceTile_DoActualReplacement_Single`
/// (`WorldGen.cs:62607-62647`).
fn replace_single(
    world: &mut World,
    target: u16,
    style: i32,
    ax: i32,
    ay: i32,
    old: &Tile,
    chest_holds: impl Fn(i32, i32) -> bool,
) {
    let old_kind = old.kind;
    let Some(t) = world.tile_mut(ax, ay) else {
        return;
    };
    t.kind = target;
    // `:62612-62619` — the platform/planter-box and torch style frames.
    if td::SET_PLATFORMS.has(t.kind) || t.kind == 380 {
        t.frame_y = (style * 18) as i16;
    }
    if td::SET_TORCHES.has(t.kind) {
        t.frame_y = (style * 22) as i16;
    }
    clear_block_paint_and_coating(world, ax, ay);
    // `:62621-62630` — the slope/half-brick flatten, the platform-pair
    // exception (`CanPoundTile` on the NEW cell).
    let mut flatten = !can_pound_tile(world, ax, ay, false, chest_holds);
    if td::SET_PLATFORMS.has(old_kind) && td::SET_PLATFORMS.has(target) {
        flatten = false;
    }
    if flatten {
        if let Some(t) = world.tile_mut(ax, ay) {
            t.set_slope(0);
            t.flags &= !crate::tile::FLAG_HALF_BRICK;
        }
    }
    // `:62631-62641` — the sand paint arms. The rainbow-sand seed
    // (`SpecialSeedFeatures.RainbowSandAndBlackSandWalls` =
    // `Main.onlyShimmerOceanWorlds`) has no port seed flag — skipped, the
    // M5-SEED stand-in convention; the anniversary arm reads the seed.
    if matches!(target, 53 | 396 | 397) {
        let seeds = world.seeds;
        if seeds.contains(crate::seeds::SeedFlags::TENTH_ANNIVERSARY)
            && !seeds.contains(crate::seeds::SeedFlags::REMIX)
            && !seeds.contains(crate::seeds::SeedFlags::NOT_THE_BEES)
            && !seeds.contains(crate::seeds::SeedFlags::DONT_STARVE)
        {
            if let Some(t) = world.tile_mut(ax, ay) {
                t.set_paint(7);
            }
        }
    }
    // `:62642-62645` — the wall-truncation change re-frames the wall
    // (`SquareWallFrame` — the port's radius-1 reset-false superset).
    if td::SET_TRUNCATES_WALLS.has(target) != td::SET_TRUNCATES_WALLS.has(old_kind) {
        crate::framing::reframe_area(world, ax, ay, 1, false);
    }
    // `:62646` — `SquareTileFrame(topLeftX, topLeftY)`.
    let _ = super::square_tile_frame_default(world, ax, ay);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    fn world() -> World {
        let mut w = World::new("t".into(), 40, 40);
        w.frame_important = vec![false; td::TILE_COUNT];
        for x in 0..40 {
            let i = w.index(x, 20);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w
    }

    /// A stone→dirt swap lands: the kind writes, the paint clears, the
    /// anchor stays (`WorldGen.cs:62607-62646`).
    #[test]
    fn w4_g80_the_single_replacement_writes_the_kind_and_clears_paint() {
        let mut w = world();
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tiles[i].set_paint(5);
        let r = replace_tile_write(&mut w, 10, 19, 0, 0, |_, _| false).unwrap();
        assert_eq!((r.anchor_x, r.anchor_y), (10, 19));
        assert_eq!(r.old.kind, 1);
        let t = w.tile(10, 19).unwrap();
        assert_eq!(t.kind, 0);
        assert_eq!(t.paint(), 0, "the paint cleared");
    }

    /// The grass family never replaces (`WorldGen.cs:62712-62715`); the
    /// same-kind solid pair does.
    #[test]
    fn w4_g80_the_grass_target_refuses_and_the_solid_pair_works() {
        let w = world();
        assert!(!would_tile_replacement_work(&w, 2, 10, 19), "grass 2");
        assert!(!would_tile_replacement_work(&w, 633, 10, 19), "ash grass");
        assert!(would_tile_replacement_work(&w, 0, 10, 19), "dirt over stone");
        assert!(!would_tile_replacement_work(&w, 14, 10, 19), "a table");
    }

    /// The chest anchor walk-back (`WorldGen.cs:62688-62692`): the clicked
    /// bottom-right cell of a 2×2 chest moves to the top-left.
    #[test]
    fn w4_g80_the_chest_anchor_walks_back() {
        let mut w = world();
        let i = w.index(11, 19);
        w.tiles[i] = Tile {
            kind: 21,
            frame_x: 18,
            frame_y: 18,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        assert_eq!(move_replace_anchor(&w, 11, 19), (10, 18));
    }

    /// The locked chest refuses the swap (`WorldGen.cs:62736`'s
    /// `Chest.IsLocked` term — the gold chest frameX 72..106).
    #[test]
    fn w4_g80_the_locked_chest_refuses() {
        let mut w = world();
        for (kind, fx) in [(21u16, 72i16), (467, 13 * 36)] {
            let i = w.index(10, 19);
            w.tiles[i] = Tile {
                kind,
                frame_x: fx,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
            assert!(
                !would_tile_replacement_work(&w, 21, 10, 19),
                "locked {kind} refuses"
            );
        }
    }

    /// The cactus above forbids the replacement; the tree-trunk window
    /// allows it (`WorldGen.cs:62344-62361`).
    #[test]
    fn w4_g80_the_above_cell_gates() {
        let mut w = world();
        let i = w.index(10, 18);
        w.tiles[i] = Tile {
            kind: 80,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        assert!(!is_tile_replaceable(&w, 10, 19), "the cactus above");
        w.tiles[i].kind = 5;
        w.tiles[i].frame_x = 66;
        w.tiles[i].frame_y = 22;
        assert!(is_tile_replaceable(&w, 10, 19), "the trunk window");
        w.tiles[i].frame_y = 132;
        assert!(!is_tile_replaceable(&w, 10, 19), "outside the window");
    }

    /// The top row skips the above-cell checks and passes
    /// (`WorldGen.cs:62331`/`:62374` — the guard falls through to true).
    #[test]
    fn w4_g80_the_top_row_replacement_is_replaceable() {
        let w = world();
        assert!(is_tile_replaceable(&w, 10, 0), "y < 1 falls through");
    }

    /// The platform-pair keeps its slope (`WorldGen.cs:62622-62625`); a
    /// solid→platform swap flattens (CanPoundTile refuses the mechanism
    /// kinds only — a plain platform pounds).
    #[test]
    fn w4_g80_the_platform_pair_keeps_the_slope() {
        let mut w = world();
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 19,
            frame_y: 0,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tiles[i].set_slope(1);
        let _ = replace_tile_write(&mut w, 10, 19, 19, 1, |_, _| false).unwrap();
        let t = w.tile(10, 19).unwrap();
        assert_eq!(t.slope(), 1, "the platform pair keeps the slope");
        assert_eq!(t.frame_y, 18, "the style frame wrote");
    }
}
