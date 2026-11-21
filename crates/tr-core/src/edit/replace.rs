//! W4-g48 (net-17 kind 21) — `WorldGen.ReplaceTile` and its helper family
//! (`WorldGen.cs:62558-62775`): the smart-cursor block swap the
//! `17 TileManipulation` reader's kind 21 runs (`MessageBuffer.cs:1264`).
//! The former cited skip (`liquid.rs:709`) is closed.
//!
//! Seams left open (documented at the sites): the dust burst
//! (`KillTile_GetTileDustAmount`/`MakeTileDust`) is the app's visual lane;
//! `KillTile_PlaySounds` rides `kill_tile_sound` app-side; the
//! `KillTile_DropItems(includeLargeObjectDrops: true)` resolution and the
//! `AttemptFossilShattering` genRand chain + its 17 relays ride the caller
//! (`net_apply::replace_tile_apply`); the rigged-chest `HitSwitch` +
//! `SendData(59)` are app-side (`netglue::is_rigged` + `wiring.hit_switch`).

use crate::framing;
use crate::tile::Tile;
use crate::tiledata as td;
use crate::world::World;

use super::{can_pound_tile, ensure_frame_important, kill_tile, touch, ObjectDrop, WALL_UNBREAKABLE};

/// `WorldGen.ReplaceTile(x, y, targetType, targetStyle)`
/// (`WorldGen.cs:62558-62580`): the two gates, the anchor move, then the
/// dust/sound/drop/fossil tails and the actual replacement. The dust count
/// is returned so the app can spawn the burst (`KillTile_GetTileDustAmount`
/// is the app's `collision::tile_dust_amount`); `sounds_kind`/`frame_x`
/// carry the ORIGINAL cell for `KillTile_PlaySounds`; `drops` collects the
/// `EliminateNaturalExtras` kills' cascade records (the C# `KillTile`s
/// there drop through `Item.NewItem` — the app resolves them); `fossil`
/// latches the 404 pre-clear cell so the caller can run
/// `AttemptFossilShattering` BEFORE the replacement (the C# order,
/// `:62577`). The rigged-chest probe (`:62586-62590`) rides the returned
/// anchor — the app checks `IsChestRigged` and fires the wiring + 59.
/// Returns `None` when a gate refused.
pub struct ReplaceOutcome {
    /// The moved top-left anchor (`MoveReplaceTileAnchor`).
    pub x: i32,
    pub y: i32,
    /// The ORIGINAL cell (the C# `tileSafely` reference — kind/frames for
    /// the dust/sound/drop tails and the campfire frameY offset).
    pub old: Tile,
    /// The `EliminateNaturalExtras` kills' object-drop records.
    pub drops: Vec<ObjectDrop>,
}

/// `Chest.IsLocked(Tile)` (`Chest.cs:301-316`): the locked chest frames —
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
/// solid, non-solidTop tile — ropes pass, everything else must NOT be
/// frame-important (`Main.tileFrameImportant` — the port's boot-truth
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
/// exceptions, then the five validity families' AND chain — kept verbatim
/// incl. the C#'s bitwise `&` on bools (all operands side-effect free).
pub fn would_tile_replacement_work(world: &World, target: u16, x: i32, y: i32) -> bool {
    let Some(tile) = world.tile(x, y).copied() else {
        return false; // the C# null tile would throw; the reader gates InWorld
    };
    // `:62708-62711` — TileID.Count = 754 (`TileID.cs:2143`).
    if usize::from(target) >= td::TILE_COUNT {
        return false;
    }
    // `:62712-62715` — Conversion.Grass (2/23/199/109/477/492,
    // `TileID.cs:86`) + the ash grass 633.
    if matches!(target, 2 | 23 | 199 | 109 | 477 | 492 | 633) {
        return false;
    }
    // `:62716-62732` — the rope exceptions: a track/platform/planter target
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
    // `:62733-62736` — the validity chain.
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
                // `:62354-62361` — the tree-trunk branch frames.
                if td::SET_IS_A_TREE_TRUNK.has(above.kind) {
                    let (fx, fy) = (i32::from(above.frame_x), i32::from(above.frame_y));
                    return (fx == 66 && (0..=44).contains(&fy))
                        || (fx == 88 && (66..=110).contains(&fy))
                        || fy >= 198;
                }
                // `:62362-62369` — the palm's two refusal columns.
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
/// body — kept 1:1).
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
/// `KillTile` — the cascade records collect into `drops`.
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
        if let Some(kill) = kill_tile(world, x, y - 1, false) {
            drops.extend(kill.object_drops);
        }
    }
    let down = world.tile(x, y + 1).copied();
    if down.is_some_and(|t| {
        t.active()
            && (td::SET_REPLACE_TILE_BREAK_DOWN.has(t.kind)
                || (t.kind == 165 && matches!(t.frame_y, 0 | 18 | 72)))
    }) {
        if let Some(kill) = kill_tile(world, x, y + 1, false) {
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
            // `SquareTileFrame(x, y)` — the port's radius-1 reset superset
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
        // The C# `(int)(Math.Sin(...) * 10.0) - 5` — the cast truncates the
        // product BEFORE the −5; kept verbatim.
        num = x + (((y % 50) as f32 / 50.0 * std::f32::consts::TAU).sin() * 10.0) as i32 - 5;
        num2 = 0;
    }
    (13 + (num % 44 + num2 % 44) % 11) as u8
}

/// `WorldGen.ReplaceTile_DoActualReplacement_Single`
/// (`WorldGen.cs:62607-62647`): the extras clear, the kind swap, the
/// platform/planter/torch style rows, the paint clear, the slope/half
/// flatten (`CanPoundTile` gate + the platform→platform exemption), the
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
    // `:62609` — `EliminateNaturalExtras` runs in `replace_tile_commit`
    // ahead of this call (the records collect into the plan there).
    let Some(old) = world.tile(x, y).copied() else {
        return;
    };
    let old_kind = old.kind;
    if let Some(t) = world.tile_mut(x, y) {
        t.kind = target;
        // `:62612-62619` — the style rows.
        if td::SET_PLATFORMS.has(target) || target == 380 {
            t.frame_y = (style * 18) as i16;
        }
        if td::SET_TORCHES.has(target) {
            t.frame_y = (style * 22) as i16;
        }
    }
    clear_block_paint_and_coating(world, x, y);
    // `:62621-62630` — the flatten gate.
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
    // `:62631-62641` — the sand paints. `RainbowSandAndBlackSandWalls` =
    // `Main.onlyShimmerOceanWorlds` (`SpecialSeedFeatures.cs:93`, the
    // drunk+tenth !remix !zenith !bees combo — `Main.cs:2674-2684`) and
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
    // `:62642-62645` — the truncates-walls membership change re-frames the
    // wall square; the port's `reframe_area` covers walls too (the
    // documented superset — the wall-only pass is inside it).
    if td::SET_TRUNCATES_WALLS.has(target) != td::SET_TRUNCATES_WALLS.has(old_kind) {
        framing::reframe_area(world, x, y, 1, true);
    }
    // `:62646` — `SquareTileFrame(x, y)`.
    framing::reframe_area(world, x, y, 1, true);
}

/// `WorldGen.ReplaceTile(x, y, targetType, targetStyle)`
/// (`WorldGen.cs:62558-62580`) — the world-state half, split into the
/// gate/anchor phase ([`replace_tile_prepare`]) and the write phase
/// ([`replace_tile_commit`]) so the caller can run the C# tails (dust →
/// sound → drops → fossil → rigged-switch) between them in the source
/// order; the tails all read the pre-swap `tileSafely` the plan carries.
/// The one-call form runs both phases back-to-back (no tails — the
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

/// The write phase — `ReplaceTile_DoActualReplacement`
/// (`WorldGen.cs:62582-62605`) over a prepared plan. The rigged-chest
/// `HitSwitch` + `SendData(59)` (`:62586-62590`) is app-side: the caller
/// probes `IsChestRigged(plan.x, plan.y)` BEFORE this call (the frame
/// check reads the OLD tiles). The `EliminateNaturalExtras` kills'
/// cascade records append to `plan.drops` (the single arm only — the C#
/// calls it inside `_Single`).
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
        // The extras clear runs FIRST inside `_Single` (`:62609`) — its
        // records collect into the plan (the caller resolves the drops).
        eliminate_natural_extras(world, ax, ay, &mut plan.drops);
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
