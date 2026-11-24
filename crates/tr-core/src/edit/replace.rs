//! The block-swap tile replacement — `WorldGen.ReplaceTile`
//! (`WorldGen.cs:62558-62780`) and its helper family. Two public API shapes
//! coexist: the split `replace_tile_prepare`/`replace_tile_commit` (net-17
//! relay, `net_apply::replace_tile_apply`) and the one-call
//! `replace_tile_write` (`world_sim/editing/block_swap.rs`).
//!
//! The kill-effects (dust/sound/drops/fossil shatter) are app-side; the core
//! records the anchor + the old cell so the caller runs them in C# order.

use crate::framing;
use crate::tile::Tile;
use crate::tiledata as td;
use crate::world::World;

use super::{can_pound_tile, ensure_frame_important, kill_tile, kill_tile_legacy, touch, ObjectDrop, TileKill, WALL_UNBREAKABLE};

/// `WorldGen.ReplaceTile(x, y, targetType, targetStyle)`
/// (`WorldGen.cs:62558-62580`): the two gates, the anchor move, then the
/// dust/sound/drop/fossil tails and the actual replacement. The dust count
/// is returned so the app can spawn the burst (`KillTile_GetTileDustAmount`
/// is the app's `collision::tile_dust_amount`); `sounds_kind`/`frame_x`
/// carry the ORIGINAL cell for `KillTile_PlaySounds`; `drops` collects the
/// `EliminateNaturalExtras` kills' cascade records (the C# `KillTile`s
/// there drop through `Item.NewItem` тАФ the app resolves them); `fossil`
/// latches the 404 pre-clear cell so the caller can run
/// `AttemptFossilShattering` BEFORE the replacement (the C# order,
/// `:62577`). The rigged-chest probe (`:62586-62590`) rides the returned
/// anchor тАФ the app checks `IsChestRigged` and fires the wiring + 59.
/// Returns `None` when a gate refused.
pub struct ReplaceOutcome {
    /// The moved top-left anchor (`MoveReplaceTileAnchor`).
    pub x: i32,
    pub y: i32,
    /// The ORIGINAL cell (the C# `tileSafely` reference тАФ kind/frames for
    /// the dust/sound/drop tails and the campfire frameY offset).
    pub old: Tile,
    /// The `EliminateNaturalExtras` kills' object-drop records тАФ always
    /// empty on the live path (the C# `_Single` discards them, `:62655`/
    /// `:62659`); kept for the caller's uniform resolution shape.
    pub drops: Vec<ObjectDrop>,
}

/// `Chest.IsLocked(Tile)` (`Chest.cs:301-316`): the locked chest frames тАФ
/// 21's six frameX bands, 467's `frameX / 36 == 13`. A null tile is locked
/// (the port's `Option` reads as locked at the call site).
fn chest_is_locked(t: &Tile) -> bool {
    if t.kind == 21 {
        let fx = i32::from(t.frame_x);
        return (72..=106).contains(&fx)
            || (144..=178).contains(&fx)
            || (828..=1006).contains(&fx)
            || (1296..=1330).contains(&fx)
            || (1368..=1402).contains(&fx)
            || (1440..=1474).contains(&fx);
    }
    if t.kind == 467 {
        return i32::from(t.frame_x) / 36 == 13;
    }
    false
}

/// `WorldGen.ReplaceTile_IsValidSolid` (`WorldGen.cs:62739-62750`): a
/// solid, non-solidTop tile тАФ ropes pass, everything else must NOT be
/// frame-important (`Main.tileFrameImportant` тАФ the port's boot-truth
/// static table, `tileobject::frame_important`).
fn is_valid_solid(kind: u16) -> bool {
    if td::TILE_SOLID.has(kind) && !td::TILE_SOLID_TOP.has(kind) {
        if !td::TILE_ROPE.has(kind) {
            return !crate::tileobject::frame_important(kind);
        }
        return true;
    }
    false
}

/// `WorldGen.WouldTileReplacementWork(attemptingToReplaceWith, x, y)`
/// (`WorldGen.cs:62705-62737`): the target-kind gates, the rope-neighbour
/// exceptions, then the five validity families' AND chain тАФ kept verbatim
/// incl. the C#'s bitwise `&` on bools (all operands side-effect free).
pub fn would_tile_replacement_work(world: &World, target: u16, x: i32, y: i32) -> bool {
    let Some(tile) = world.tile(x, y).copied() else {
        return false; // the C# null tile would throw; the reader gates InWorld
    };
    // `:62708-62711` тАФ TileID.Count = 754 (`TileID.cs:2143`).
    if usize::from(target) >= td::TILE_COUNT {
        return false;
    }
    // `:62712-62715` тАФ Conversion.Grass (2/23/199/109/477/492,
    // `TileID.cs:86`) + the ash grass 633.
    if matches!(target, 2 | 23 | 199 | 109 | 477 | 492 | 633) {
        return false;
    }
    // `:62716-62732` тАФ the rope exceptions: a track/platform/planter target
    // beside a matching neighbour converts a rope.
    if td::TILE_ROPE.has(tile.kind)
        && x >= 2
        && x < world.width - 2
        && y >= 2
        && y < world.height - 2
    {
        let left = world.tile(x - 1, y).copied();
        let right = world.tile(x + 1, y).copied();
        let beside = |pred: &dyn Fn(u16) -> bool| {
            left.is_some_and(|t| t.active() && pred(t.kind))
                || right.is_some_and(|t| t.active() && pred(t.kind))
        };
        if target == 314 && beside(&|k| k == 314) {
            return true;
        }
        if td::SET_PLATFORMS.has(target) && beside(&|k| td::SET_PLATFORMS.has(k)) {
            return true;
        }
        if target == 380 && beside(&|k| k == 380) {
            return true;
        }
    }
    // `:62733-62736` тАФ the validity chain.
    let flag = !is_valid_solid(target) || !is_valid_solid(tile.kind);
    let flag2 = !td::SET_TORCHES.has(target) || !td::SET_TORCHES.has(tile.kind);
    let flag3 = !td::SET_CAMPFIRES.has(target) || !td::SET_CAMPFIRES.has(tile.kind);
    let platform = !td::SET_PLATFORMS.has(target) || !td::SET_PLATFORMS.has(tile.kind);
    let solid_side = (!is_valid_solid(target) && !td::SET_PLATFORMS.has(target))
        || (!is_valid_solid(tile.kind) && !td::SET_PLATFORMS.has(tile.kind));
    let chest = !td::SET_BASIC_CHEST.has(target)
        || !td::SET_BASIC_CHEST.has(tile.kind)
        || chest_is_locked(&tile);
    let dresser = !td::SET_BASIC_DRESSER.has(target) || !td::SET_BASIC_DRESSER.has(tile.kind);
    !(platform & flag2 & flag3 & flag & solid_side & chest & dresser)
}

/// `WorldGen.IsTileReplaceable(x, y)` (`WorldGen.cs:62328-62374`): the
/// unbreakable-wall refusal and the tile-above gates (cactus 80 / fallen
/// log 488 / the PreventsTileReplaceIfOnTopOfIt set with the tree-trunk and
/// palm exceptions).
pub fn is_tile_replaceable(world: &World, x: i32, y: i32) -> bool {
    let Some(tile) = world.tile(x, y).copied() else {
        return false;
    };
    if y >= 1 {
        let Some(above) = world.tile(x, y - 1).copied() else {
            return false;
        };
        if tile.wall == WALL_UNBREAKABLE {
            return false; // `:62338-62341`
        }
        if above.active() {
            if above.kind == 80 || above.kind == 488 {
                return false; // `:62344-62351`
            }
            if td::SET_PREVENTS_TILE_REPLACE_IF_ON_TOP_OF_IT.has(above.kind)
                && tile.kind != above.kind
            {
                // `:62354-62361` тАФ the tree-trunk branch frames.
                if td::SET_IS_A_TREE_TRUNK.has(above.kind) {
                    let (fx, fy) = (i32::from(above.frame_x), i32::from(above.frame_y));
                    return (fx == 66 && (0..=44).contains(&fy))
                        || (fx == 88 && (66..=110).contains(&fy))
                        || fy >= 198;
                }
                // `:62362-62369` тАФ the palm's two refusal columns.
                if above.kind == 323 {
                    return !(above.frame_x == 66 || above.frame_x == 220);
                }
                return false;
            }
        }
    }
    true
}

/// `WorldGen.MoveReplaceTileAnchor(ref x, ref y, targetType, t)`
/// (`WorldGen.cs:62686-62703`): the CURRENT cell's object family moves the
/// anchor to its top-left (the `targetType` parameter is unread in the C#
/// body тАФ kept 1:1).
#[allow(unused_variables)]
fn move_replace_tile_anchor(x: &mut i32, y: &mut i32, target_type: u16, t: &Tile) {
    let (fx, fy) = (i32::from(t.frame_x), i32::from(t.frame_y));
    if td::SET_BASIC_CHEST.has(t.kind) {
        *x -= fx % 36 / 18;
        *y -= fy % 36 / 18;
    }
    if td::SET_BASIC_DRESSER.has(t.kind) {
        *x -= fx % 54 / 18;
        *y -= fy % 36 / 18;
    }
    if td::SET_CAMPFIRES.has(t.kind) {
        *x -= fx % 54 / 18;
        *y -= fy % 36 / 18;
    }
}

/// `Tile.ClearBlockPaintAndCoating` (`Tile.cs:904-909`): paint 0 + the
/// Glow bit off + the Echo bit off (the world's sparse extras byte).
fn clear_block_paint_and_coating(world: &mut World, x: i32, y: i32) {
    if let Some(t) = world.tile_mut(x, y) {
        t.set_paint(0);
        t.flags &= !crate::tile::FLAG_FULLBRIGHT;
    }
    world.set_invisible_block(x, y, false);
}

/// `WorldGen.ReplaceTile_EliminateNaturalExtras(x, y)`
/// (`WorldGen.cs:62649-62662`): the ReplaceTileBreakUp/Down sets (plus the
/// cobweb 165's frame rows) above/below the anchor die with a full
/// `KillTile` тАФ the cascade records collect into `drops`.
fn eliminate_natural_extras(world: &mut World, x: i32, y: i32, drops: &mut Vec<ObjectDrop>) {
    if !(x >= 2 && x < world.width - 2 && y >= 2 && y < world.height - 2) {
        return; // InWorld(x, y, 2)
    }
    let up = world.tile(x, y - 1).copied();
    if up.is_some_and(|t| {
        t.active()
            && (td::SET_REPLACE_TILE_BREAK_UP.has(t.kind)
                || (t.kind == 165 && matches!(t.frame_y, 36 | 54 | 90)))
    }) {
        if let Some(kill) = kill_tile_legacy(world, x, y - 1, false) {
            drops.extend(kill.object_drops);
        }
    }
    let down = world.tile(x, y + 1).copied();
    if down.is_some_and(|t| {
        t.active()
            && (td::SET_REPLACE_TILE_BREAK_DOWN.has(t.kind)
                || (t.kind == 165 && matches!(t.frame_y, 0 | 18 | 72)))
    }) {
        if let Some(kill) = kill_tile_legacy(world, x, y + 1, false) {
            drops.extend(kill.object_drops);
        }
    }
}

/// `WorldGen.ReplaceTile_DoActualReplacement_Area`
/// (`WorldGen.cs:62664-62684`): the chest 2x2 / dresser+campfire 3x2 grid
/// write (kind + style frames + paint clear per cell), then the
/// `SquareTileFrame` walk over every cell.
fn do_actual_replacement_area(
    world: &mut World,
    target: u16,
    style: i32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    frame_offset_x: i32,
    frame_offset_y: i32,
) {
    for i in 0..w {
        for j in 0..h {
            if let Some(t) = world.tile_mut(x + i, y + j) {
                t.kind = target;
                t.frame_x = (style * (w * 18) + i * 18 + frame_offset_x) as i16;
                t.frame_y = (j * 18 + frame_offset_y) as i16;
            }
            clear_block_paint_and_coating(world, x + i, y + j);
            touch(world, x + i, y + j);
        }
    }
    ensure_frame_important(world, target);
    for k in 0..w {
        for l in 0..h {
            // `SquareTileFrame(x, y)` тАФ the port's radius-1 reset superset
            // (the documented reframe deviation, the wiring precedent).
            framing::reframe_area(world, x + k, y + l, 1, true);
        }
    }
}

/// `WorldGen.GetRainbowPaintIDForPosition(x, y, wiggly)`
/// (`WorldGen.cs:24463-24473`): the rainbow micro-seed's sand paint.
fn rainbow_paint_id(x: i32, y: i32, wiggly: bool) -> u8 {
    let mut num = x;
    let mut num2 = y;
    if wiggly {
        // The C# `(int)(Math.Sin(...) * 10.0) - 5` тАФ the cast truncates the
        // product BEFORE the тИТ5; kept verbatim.
        num = x + (((y % 50) as f32 / 50.0 * std::f32::consts::TAU).sin() * 10.0) as i32 - 5;
        num2 = 0;
    }
    (13 + (num % 44 + num2 % 44) % 11) as u8
}

/// `WorldGen.ReplaceTile_DoActualReplacement_Single`
/// (`WorldGen.cs:62607-62647`): the extras clear, the kind swap, the
/// platform/planter/torch style rows, the paint clear, the slope/half
/// flatten (`CanPoundTile` gate + the platformтЖТplatform exemption), the
/// sand seed paints and the truncates-walls / tile reframes.
fn do_actual_replacement_single(
    world: &mut World,
    target: u16,
    style: i32,
    x: i32,
    y: i32,
    chest_holds_items: impl Fn(i32, i32) -> bool + Copy,
    rainbow_sand: bool,
    tenth_anniversary_plain: bool,
) {
    // `:62609` тАФ `EliminateNaturalExtras` runs in `replace_tile_commit`
    // ahead of this call (the records collect into the plan there).
    let Some(old) = world.tile(x, y).copied() else {
        return;
    };
    let old_kind = old.kind;
    if let Some(t) = world.tile_mut(x, y) {
        t.kind = target;
        // `:62612-62619` тАФ the style rows.
        if td::SET_PLATFORMS.has(target) || target == 380 {
            t.frame_y = (style * 18) as i16;
        }
        if td::SET_TORCHES.has(target) {
            t.frame_y = (style * 22) as i16;
        }
    }
    clear_block_paint_and_coating(world, x, y);
    // `:62621-62630` тАФ the flatten gate.
    let mut flag = !can_pound_tile(world, x, y, false, chest_holds_items);
    if td::SET_PLATFORMS.has(old_kind) && td::SET_PLATFORMS.has(target) {
        flag = false;
    }
    if flag {
        if let Some(t) = world.tile_mut(x, y) {
            t.set_slope(0);
            t.flags &= !crate::tile::FLAG_HALF_BRICK;
        }
    }
    // `:62631-62641` тАФ the sand paints. `RainbowSandAndBlackSandWalls` =
    // `Main.onlyShimmerOceanWorlds` (`SpecialSeedFeatures.cs:93`, the
    // drunk+tenth !remix !zenith !bees combo тАФ `Main.cs:2674-2684`) and
    // `tenthAnniversaryWorld && !remix && !notTheBees && !dontStarve` ride
    // the caller's precomputed flags (the world's seed tables).
    if matches!(target, 53 | 396 | 397) {
        if rainbow_sand {
            let paint = rainbow_paint_id(x, y, true);
            if let Some(t) = world.tile_mut(x, y) {
                t.set_paint(paint);
            }
        } else if tenth_anniversary_plain {
            if let Some(t) = world.tile_mut(x, y) {
                t.set_paint(7);
            }
        }
    }
    touch(world, x, y);
    // `:62642-62645` тАФ the truncates-walls membership change re-frames the
    // wall square; the port's `reframe_area` covers walls too (the
    // documented superset тАФ the wall-only pass is inside it).
    if td::SET_TRUNCATES_WALLS.has(target) != td::SET_TRUNCATES_WALLS.has(old_kind) {
        framing::reframe_area(world, x, y, 1, true);
    }
    // `:62646` тАФ `SquareTileFrame(x, y)`.
    framing::reframe_area(world, x, y, 1, true);
}

/// `WorldGen.ReplaceTile(x, y, targetType, targetStyle)`
/// (`WorldGen.cs:62558-62580`) тАФ the world-state half, split into the
/// gate/anchor phase ([`replace_tile_prepare`]) and the write phase
/// ([`replace_tile_commit`]) so the caller can run the C# tails (dust тЖТ
/// sound тЖТ drops тЖТ fossil тЖТ rigged-switch) between them in the source
/// order; the tails all read the pre-swap `tileSafely` the plan carries.
/// The one-call form runs both phases back-to-back (no tails тАФ the
/// worldgen/liquid callers' shape).
///
/// `chest_holds_items` is the `CanPoundTile` chest scan (the app injects
/// its store; sources without one pass `|_, _| false`), `rainbow_sand` /
/// `tenth_anniversary_plain` are the seed-flag reads (see
/// [`do_actual_replacement_single`]).
pub fn replace_tile_prepare(
    world: &World,
    x: i32,
    y: i32,
    target: u16,
) -> Option<ReplaceOutcome> {
    let old = world.tile(x, y).copied()?;
    if !would_tile_replacement_work(world, target, x, y) {
        return None; // `:62561-62564`
    }
    if !is_tile_replaceable(world, x, y) {
        return None; // `:62565-62568`
    }
    let (mut ax, mut ay) = (x, y);
    move_replace_tile_anchor(&mut ax, &mut ay, target, &old);
    Some(ReplaceOutcome { x: ax, y: ay, old, drops: Vec::new() })
}

/// The write phase тАФ `ReplaceTile_DoActualReplacement`
/// (`WorldGen.cs:62582-62605`) over a prepared plan. The rigged-chest
/// `HitSwitch` + `SendData(59)` (`:62586-62590`) is app-side: the caller
/// probes `IsChestRigged(plan.x, plan.y)` BEFORE this call (the frame
/// check reads the OLD tiles). The `EliminateNaturalExtras` kills'
/// drops are DISCARDED (the C# `_Single` calls plain `KillTile` with no
/// collection тАФ `:62655`/`:62659`); `plan.drops` stays empty, kept on the
/// struct for the caller's uniform resolution shape.
pub fn replace_tile_commit(
    world: &mut World,
    plan: &mut ReplaceOutcome,
    target: u16,
    style: i32,
    chest_holds_items: impl Fn(i32, i32) -> bool + Copy,
    rainbow_sand: bool,
    tenth_anniversary_plain: bool,
) {
    let (ax, ay) = (plan.x, plan.y);
    if td::SET_BASIC_CHEST.has(target) {
        do_actual_replacement_area(world, target, style, ax, ay, 2, 2, 0, 0);
    } else if td::SET_BASIC_DRESSER.has(target) {
        do_actual_replacement_area(world, target, style, ax, ay, 3, 2, 0, 0);
    } else if td::SET_CAMPFIRES.has(target) {
        let offset_y = if plan.old.frame_y >= 36 { 36 } else { 0 };
        do_actual_replacement_area(world, target, style, ax, ay, 3, 2, 0, offset_y);
    } else {
        // The extras clear runs FIRST inside `_Single` (`:62609`) тАФ the
        // C# `KillTile(x, y┬▒1)` there discards its drops (no collection at
        // the call site, `:62655`/`:62659`); the port mirrors that (the
        // records drop on the floor тАФ the plan's `drops` stays empty).
        let mut discarded = Vec::new();
        eliminate_natural_extras(world, ax, ay, &mut discarded);
        do_actual_replacement_single(
            world,
            target,
            style,
            ax,
            ay,
            chest_holds_items,
            rainbow_sand,
            tenth_anniversary_plain,
        );
    }
}

/// The one-call form: prepare + commit, no tails (the dust/sound/drop/
/// fossil/rigged seams run only through the split phases' caller).
pub fn replace_tile(
    world: &mut World,
    x: i32,
    y: i32,
    target: u16,
    style: i32,
    chest_holds_items: impl Fn(i32, i32) -> bool + Copy,
    rainbow_sand: bool,
    tenth_anniversary_plain: bool,
) -> Option<ReplaceOutcome> {
    let mut plan = replace_tile_prepare(world, x, y, target)?;
    replace_tile_commit(
        world,
        &mut plan,
        target,
        style,
        chest_holds_items,
        rainbow_sand,
        tenth_anniversary_plain,
    );
    Some(plan)
}

/// The result of a successful `ReplaceTile`: the anchor the write landed
/// on, the OLD cell (the dust/drops read it тАФ the C# `tileSafely` is a
/// LIVE reference, so its dust/drops see the pre-replacement state; the
/// port snapshots), and the natural-extras kills the replacement caused.
pub struct Replaced {
    pub anchor_x: i32,
    pub anchor_y: i32,
    pub old: Tile,
    /// `ReplaceTile_EliminateNaturalExtras`' `KillTile` results (the
    /// above/below vine-adjacent cells) тАФ the app resolves their drops.
    pub extras_kills: Vec<(i32, i32, TileKill)>,
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

/// `WorldGen.ReplaceTile` (`WorldGen.cs:62558-62580`) minus the
/// kill-effects: the two validity gates, the anchor move, then the
/// replacement write. The caller runs the dust (`KillTile_GetTileDustAmount`
/// + `KillTile_MakeTileDust`), the sounds (`KillTile_PlaySounds`), the drops
/// (`KillTile_DropItems(includeLargeObjectDrops: true)`) and the fossil
/// shatter BETWEEN the gates and the write тАФ the C# order (`:62570-62578`);
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
        // `:62586-62590` тАФ the rigged-chest trip is app-side (the wiring
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
        // single arm (`WorldGen.cs:62609`), BEFORE the kind write тАФ the
        // kills' cascades see the OLD cell still standing. The chest/
        // dresser/campfire area writes never call it.
        let extras_kills = eliminate_natural_extras_kills(world, ax, ay);
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
/// `KillTile(x, y┬▒1)` тАФ the full cascade + sounds + drops, app-resolved
/// through `Replaced::extras_kills`.
fn eliminate_natural_extras_kills(world: &mut World, x: i32, y: i32) -> Vec<(i32, i32, TileKill)> {
    let mut kills = Vec::new();
    if x < 2 || x >= world.width - 2 || y < 2 || y >= world.height - 2 {
        return kills;
    }
    if let Some(up) = world.tile(x, y - 1) {
        if up.active()
            && (td::SET_REPLACE_TILE_BREAK_UP.has(up.kind)
                || (up.kind == 165 && matches!(up.frame_y, 36 | 54 | 90)))
        {
            if let Some(k) = kill_tile_legacy(world, x, y - 1, false) {
                kills.push((x, y - 1, k));
            }
        }
    }
    if let Some(down) = world.tile(x, y + 1) {
        if down.active()
            && (td::SET_REPLACE_TILE_BREAK_DOWN.has(down.kind)
                || (down.kind == 165 && matches!(down.frame_y, 0 | 18 | 72)))
        {
            if let Some(k) = kill_tile_legacy(world, x, y + 1, false) {
                kills.push((x, y + 1, k));
            }
        }
    }
    kills
}

/// `WorldGen.ReplaceTile_DoActualReplacement_Area`
/// (`WorldGen.cs:62664-62684`): the multi-cell write, then the per-cell
/// `SquareTileFrame` walk (the port's `square_tile_frame_default` тАФ the
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
            let _ = super::square_tile_frame_default_legacy(world, ax + i, ay + j);
        }
    }
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
    // `:62612-62619` тАФ the platform/planter-box and torch style frames.
    if td::SET_PLATFORMS.has(t.kind) || t.kind == 380 {
        t.frame_y = (style * 18) as i16;
    }
    if td::SET_TORCHES.has(t.kind) {
        t.frame_y = (style * 22) as i16;
    }
    clear_block_paint_and_coating(world, ax, ay);
    // `:62621-62630` тАФ the slope/half-brick flatten, the platform-pair
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
    // `:62631-62641` тАФ the sand paint arms. The rainbow-sand seed
    // (`SpecialSeedFeatures.RainbowSandAndBlackSandWalls` =
    // `Main.onlyShimmerOceanWorlds`) has no port seed flag тАФ skipped, the
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
    // `:62642-62645` тАФ the wall-truncation change re-frames the wall
    // (`SquareWallFrame` тАФ the port's radius-1 reset-false superset).
    if td::SET_TRUNCATES_WALLS.has(target) != td::SET_TRUNCATES_WALLS.has(old_kind) {
        crate::framing::reframe_area(world, ax, ay, 1, false);
    }
    // `:62646` тАФ `SquareTileFrame(topLeftX, topLeftY)`.
    let _ = super::square_tile_frame_default_legacy(world, ax, ay);
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

    /// A stoneтЖТdirt swap lands: the kind writes, the paint clears, the
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
    /// bottom-right cell of a 2├Ч2 chest moves to the top-left.
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
    /// `Chest.IsLocked` term тАФ the gold chest frameX 72..106).
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
    /// (`WorldGen.cs:62331`/`:62374` тАФ the guard falls through to true).
    #[test]
    fn w4_g80_the_top_row_replacement_is_replaceable() {
        let w = world();
        assert!(is_tile_replaceable(&w, 10, 0), "y < 1 falls through");
    }

    /// The platform-pair keeps its slope (`WorldGen.cs:62622-62625`); a
    /// solidтЖТplatform swap flattens (CanPoundTile refuses the mechanism
    /// kinds only тАФ a plain platform pounds).
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
