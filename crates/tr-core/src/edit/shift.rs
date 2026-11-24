//! W4-c42 batch 2 — the wired trapdoor/tall-gate shifts:
//! `WorldGen.ShiftTrapdoor` (`WorldGen.cs:51447-51596`) and
//! `WorldGen.ShiftTallGate` (`:51723-51775`), the two `HitWireSingle`
//! device arms (`Wiring.cs:1440-1460`) call. The i34 seams are reused:
//! `validate_tile_square`, `kill_tile`, `can_kill_tile` (the wiring slice),
//! `framing::reframe_area` (the trailing `TileFrame` loops — the documented
//! radius-superset deviation, the `open_door` precedent).
//!
//! Seams left open (the claim doc): the `Collision.EmptyTile(ignoreTiles:
//! true)` entity half rides a `cell_blocked` closure (tr-core carries no
//! entity store — the wiring caller passes the always-free closure, the
//! `smart_cursor.rs:312` precedent); the `Wiring.SkipWire` calls
//! (`Wiring.running`-gated) are collected into `wire_skips` — the f11 §1
//! Option shape (research/95 union-exec): the caller passes `Some` only
//! while `Wiring.running`, `None` on the NPC/interaction stages; the
//! `PlaySound(8/9)` cues queue into `cues` as `(open, px, py)` — the
//! door-family cue vocabulary (`audio::play_door_sound`, ids 8/9).

use crate::framing;
use crate::tile::{Tile, FLAG_ACTIVE, FLAG_FULLBRIGHT, FLAG_HALF_BRICK, FLAG_INACTIVE};
use crate::world::World;

use super::{ensure_frame_important, kill_tile, kill_tile_legacy, touch, validate_tile_square, ObjectDrop};

/// `WorldGen.GetTopLeftAndStyles` (`WorldGen.cs:51777-51790`): the object's
/// top-left cell (mutating `x`/`y`) plus the `(styleX, styleY)` pair — the
/// frame divided by the whole-object span. `frame_x_inc`/`frame_y_inc` of 0
/// skip that axis's adjustment. `GetTileSafely` reads a default tile out of
/// bounds (frames 0). f11's pub(crate) placement + `.max(1)` divisor guard
/// is the reusable keeper (research/95 §2 / research/133 §3, fold-133 —
/// one copy; no current caller passes inc 0, the guard keeps a future one
/// from dividing by zero).
pub(crate) fn top_left_and_styles(
    world: &World,
    x: &mut i32,
    y: &mut i32,
    w: i32,
    h: i32,
    frame_x_inc: i32,
    frame_y_inc: i32,
) -> (i32, i32) {
    let t = world.tile(*x, *y).copied().unwrap_or_default();
    let (fx, fy) = (i32::from(t.frame_x), i32::from(t.frame_y));
    let result = (fx / (w * frame_x_inc.max(1)), fy / (h * frame_y_inc.max(1)));
    if frame_x_inc != 0 {
        *x -= fx / frame_x_inc % w;
    }
    if frame_y_inc != 0 {
        *y -= fy / frame_y_inc % h;
    }
    result
}

/// `Tile.ClearTile` (`Tile.cs:123-128`): `ClearSlope` (slope 0, halfBrick
/// off) + `active(false)` + `inActive(false)` — the kind/frames survive.
fn clear_tile(t: &mut Tile) {
    t.set_slope(0);
    t.flags &= !(FLAG_HALF_BRICK | FLAG_ACTIVE | FLAG_INACTIVE);
}

/// `Tile.BlockColorAndCoating` + `UseBlockColors` (`Tile.cs`): the port's
/// paint + Glow-coating pair, cached per column before the writes.
fn block_color_cache(t: Option<&Tile>) -> (u8, bool) {
    t.map(|t| (t.paint(), t.fullbright_block())).unwrap_or((0, false))
}

fn use_block_colors(t: &mut Tile, cache: (u8, bool)) {
    t.set_paint(cache.0);
    if cache.1 {
        t.flags |= FLAG_FULLBRIGHT;
    } else {
        t.flags &= !FLAG_FULLBRIGHT;
    }
}

/// `WorldGen.ShiftTrapdoor(x, y, playerAbove, onlyCloseOrOpen = -1)`
/// (`WorldGen.cs:51447-51596`): the trapdoor pair 386 (closed) / 387 (open).
/// The wired caller (`Wiring.cs:1443-1446`) tries `playerAbove: true` first,
/// then `false` on refusal, both with the C# default `onlyCloseOrOpen = -1`
/// (f11's param, merged per research/95 §2: −1 = both directions, 0 =
/// close-only, 1 = open-only — a future hammer/interact caller needs it).
/// `wire_skips` collects the `Wiring.running`-gated SkipWire cells (the §1
/// Option seam), `cues` the PlaySound(8) success / PlaySound(9) fallback
/// failure positions, `drops` the cut/drip sweep's KillTile cascades.
#[allow(clippy::too_many_arguments)]
pub fn shift_trapdoor(
    world: &mut World,
    x: i32,
    y: i32,
    player_above: bool,
    only_close_or_open: i32,
    cell_blocked: &dyn Fn(i32, i32) -> bool,
    can_kill: &dyn Fn(&World, i32, i32) -> bool,
    drops: &mut Vec<ObjectDrop>,
    mut wire_skips: Option<&mut Vec<(i32, i32)>>,
    cues: &mut Vec<(bool, i32, i32)>,
) -> bool {
    // `Framing.GetTileSafely(x, y)` (:51449) — the ORIGINAL cell's kind
    // drives both type tests (the C# `tileSafely` reference survives the
    // anchor adjustment below).
    let start_kind = world.tile(x, y).map_or(0, |t| t.kind);
    let (mut x, mut y) = (x, y);
    if start_kind == 386 && only_close_or_open != 1 {
        // :51450 — the open branch gates on `onlyCloseOrOpen != 1`.
        let (style_x, style_y) = top_left_and_styles(world, &mut x, &mut y, 2, 2, 18, 18);
        if style_x == 0 {
            // :51453-51494 — the top-row variant opens DOWNWARD.
            let mut _gen_rng = crate::UnifiedRandom::new(world.seed as i32);
            if !validate_tile_square(world, x, y, 2, 2, 386, drops, &mut _gen_rng) {
                return false;
            }
            if let Some(skips) = wire_skips.as_mut() {
                for cell in [(x, y), (x, y + 1), (x + 1, y), (x + 1, y + 1)] {
                    skips.push(cell); // `Wiring.SkipWire` (:51461-51464)
                }
            }
            if cell_blocked(x, y + 1) || cell_blocked(x + 1, y + 1) {
                return false; // `Collision.EmptyTile(ignoreTiles: true)` (:51466)
            }
            cues.push((true, x * 16 + 16, y * 16 + 16)); // PlaySound(8) (:51470)
            for i in 0..2 {
                // :51471-51478 — the top row clears (active 386 cells only).
                if let Some(t) = world.tile_mut(x + i, y) {
                    if t.active() && t.kind == 386 {
                        clear_tile(t);
                        touch(world, x + i, y);
                    }
                }
            }
            for j in 0..2 {
                // :51479-51485 — the bottom row becomes the open trapdoor.
                if let Some(t) = world.tile_mut(x + j, y + 1) {
                    t.kind = 387;
                    t.frame_x = (j * 18) as i16;
                    t.frame_y = (style_y * 18) as i16;
                    touch(world, x + j, y + 1);
                }
            }
            // The trailing TileFrame 4×3 loop (:51486-51492).
            ensure_frame_important(world, 387);
            framing::reframe_area(world, x, y + 1, 2, false);
            return true;
        }
        if style_x == 1 {
            // :51495-51532 — the bottom-row variant opens UPWARD.
            let mut _gen_rng = crate::UnifiedRandom::new(world.seed as i32);
            if !validate_tile_square(world, x, y, 2, 2, 386, drops, &mut _gen_rng) {
                return false;
            }
            if let Some(skips) = wire_skips.as_mut() {
                for cell in [(x, y - 1), (x, y), (x + 1, y - 1), (x + 1, y)] {
                    skips.push(cell); // :51503-51506
                }
            }
            if cell_blocked(x, y) || cell_blocked(x + 1, y) {
                return false; // :51508
            }
            cues.push((true, x * 16 + 16, y * 16)); // PlaySound(8) (:51512)
            for m in 0..2 {
                // :51513-51516 — the bottom row clears UNCONDITIONALLY.
                if let Some(t) = world.tile_mut(x + m, y + 1) {
                    clear_tile(t);
                    touch(world, x + m, y + 1);
                }
            }
            for n in 0..2 {
                // :51517-51523 — the top row becomes the open trapdoor.
                if let Some(t) = world.tile_mut(x + n, y) {
                    t.kind = 387;
                    t.frame_x = (n * 18) as i16;
                    t.frame_y = (style_y * 18) as i16;
                    touch(world, x + n, y);
                }
            }
            // The trailing TileFrame 4×3 loop (:51524-51530).
            ensure_frame_important(world, 387);
            framing::reframe_area(world, x, y, 2, false);
            return true;
        }
    }
    if start_kind == 387 && only_close_or_open != 0 {
        // :51534 — the close branch gates on `onlyCloseOrOpen != 0`.
        let _ = top_left_and_styles(world, &mut x, &mut y, 2, 1, 18, 18);
        // :51537-51542 — either open cell the kill rules refuse blocks the
        // close. The caller injects `WorldGen.CanKillTile` — the wiring
        // arms pass the flood's anchored-tile slice (`Wiring::can_kill_tile`,
        // the DeActive precedent), the i34 keep-one.
        let t2 = world.tile(x, y).copied();
        let t3 = world.tile(x + 1, y).copied();
        if t2.is_some_and(|t| t.active() && t.kind == 387 && !can_kill(world, x, y))
            || t3.is_some_and(|t| t.active() && t.kind == 387 && !can_kill(world, x + 1, y))
        {
            return false;
        }
        let mut _gen_rng = crate::UnifiedRandom::new(world.seed as i32);
        if !validate_tile_square(world, x, y, 2, 1, 387, drops, &mut _gen_rng) {
            return false; // :51543-51546
        }
        // `playerAbove.ToDirectionInt()` (`Utils.cs:2010-2017`): +1 / −1.
        let num3 = if player_above { 1 } else { -1 };
        for num4 in 0..2 {
            // :51548-51555 — the destination row must hold no real blocker.
            if let Some(t) = world.tile(x + num4, y + num3) {
                if t.active()
                    && !crate::tileobject::tile_cut(t.kind)
                    && !crate::tileobject::is_drip_tile(t.kind)
                {
                    return false;
                }
            }
        }
        if let Some(skips) = wire_skips.as_mut() {
            for cell in [(x, y), (x, y + num3), (x + 1, y), (x + 1, y + num3)] {
                skips.push(cell); // :51558-51561
            }
        }
        cues.push((true, x * 16 + 16, y * 16)); // PlaySound(8) (:51563)
        for num5 in 0..2 {
            // :51564-51571 — cut/drip tiles in the way die with a full
            // KillTile (default flags — the cascade runs).
            let cut = world.tile(x + num5, y + num3).is_some_and(|t| {
                t.active()
                    && (crate::tileobject::tile_cut(t.kind)
                        || crate::tileobject::is_drip_tile(t.kind))
            });
            if cut {
                if let Some(kill) = kill_tile_legacy(world, x + num5, y + num3, false) {
                    drops.extend(kill.object_drops);
                }
            }
        }
        for num6 in 0..2 {
            // :51572-51584 — the 2×2 closed body; the paint cache reads the
            // OPEN row cell of the same column before any write.
            let cache = block_color_cache(world.tile(x + num6, y));
            for num7 in 0..2 {
                let cy = y + num7 - i32::from(!player_above);
                if let Some(t) = world.tile_mut(x + num6, cy) {
                    t.kind = 386;
                    t.frame_x = (num6 * 18 + i32::from(player_above) * 36) as i16;
                    t.frame_y = (num7 * 18) as i16;
                    use_block_colors(t, cache);
                    t.flags |= FLAG_ACTIVE;
                    touch(world, x + num6, cy);
                }
            }
        }
        // The trailing TileFrame 4×4 loop (:51585-51591).
        ensure_frame_important(world, 386);
        framing::reframe_area(world, x, y + 1 - i32::from(!player_above) * 2, 2, false);
        return true;
    }
    // :51594-51595 — the fallback thud (the adjusted anchor, the C# locals).
    cues.push((false, x * 16, y * 16)); // PlaySound(9)
    false
}

/// `WorldGen.ShiftTallGate(x, y, closing, forced = false)`
/// (`WorldGen.cs:51723-51775`): the tall-gate pair 388 (closed) / 389
/// (open). The wired caller (`Wiring.cs:1454-1459`) passes
/// `closing = (type == 389)` and ignores the result. The gate's def is
/// ALWAYS `GetTileData(388, frameX / 18)` (:51733) — the closed kind's
/// registry entry, 2-arg → alternate 0, even when opening (the research/95
/// §3 canonical decode; g4's alternate-1 and f11's no-lookup both dropped).
///
/// **The NPC-bash drain re-point contract (research/95 §3, g4 `53063dd1`):**
/// the `wants_door_bash` gate-388 arm in `world_sim/npcs.rs` calls THIS body
/// (its own `edit::shift_tall_gate` copy is dropped) with
/// `closing: false, forced: false`, a REAL player/NPC box-overlap
/// `cell_blocked` closure (the `close_door` convention — the reference
/// implementation for the §8.1 entity-half seam), `wire_skips: None` (no
/// flood runs on the NPC stage — the C# `Wiring.running` gate is false
/// there), and a local `cues` vec drained on the UpdateNpcs stream
/// (`play_door_sound(open, px, py, …)` per cue — NOT the drain's old
/// hit-cell `play_door_sound(true, tx*16, ty*16)` call: this body queues
/// the cue at the ADJUSTED top `(x*16+16, y*16+16)`, `:51762`). Refusal →
/// `ai[3] = 60.0` + `net_update`; success → `world_key = None`; drops
/// spawn on UpdateNpcs. The reframe here is radius 4 (the safe superset);
/// g4's radius-2 copy MISSED rows y−1/y+5 of the C# walk — never revive it.
#[allow(clippy::too_many_arguments)]
pub fn shift_tall_gate(
    world: &mut World,
    x: i32,
    y: i32,
    closing: bool,
    forced: bool,
    cell_blocked: &dyn Fn(i32, i32) -> bool,
    drops: &mut Vec<ObjectDrop>,
    wire_skips: Option<&mut Vec<(i32, i32)>>,
    cues: &mut Vec<(bool, i32, i32)>,
) -> bool {
    let kind: u16 = if closing { 388 } else { 389 };
    let expected: u16 = if closing { 389 } else { 388 };
    let Some(tile) = world.tile(x, y).copied() else {
        return false;
    };
    if tile.kind != expected {
        return false; // :51728-51731
    }
    // :51732-51743 — the def decode + the row walk. The C# subtracts only
    // `CoordinateHeights[i]` (NOT the padding) in the walk — kept verbatim.
    let style = i32::from(tile.frame_x) / 18;
    let Some(def) = crate::tileobject::def_for_tile(388) else {
        return false;
    };
    let Some(def) = crate::tileobject::get_tile_data(def, style, 0) else {
        return false;
    };
    let full_height = def.coordinate_full_height();
    let (width, height) = (def.width, def.height);
    let mut rem = i32::from(tile.frame_y) % full_height;
    let mut row = 0i32;
    while row < height && rem - def.coordinate_heights[row as usize] >= 0 {
        rem -= def.coordinate_heights[row as usize];
        row += 1;
    }
    let y = y - row;
    let mut _gen_rng = crate::UnifiedRandom::new(world.seed as i32);
    if !validate_tile_square(world, x, y, width, height, expected, drops, &mut _gen_rng) {
        return false; // :51744-51747
    }
    if let Some(skips) = wire_skips {
        for j in 0..height {
            skips.push((x, y + j)); // `Wiring.SkipWire` (:51750-51753)
        }
    }
    for k in 0..height {
        // :51755-51761 — the entity occupancy gate (`forced` skips it; the
        // wired caller passes false).
        if !forced && cell_blocked(x, y + k) {
            return false;
        }
    }
    cues.push((true, x * 16 + 16, y * 16 + 16)); // PlaySound(8) (:51762)
    for l in 0..height {
        // :51763-51766 — the kind swaps, the frames survive.
        if let Some(t) = world.tile_mut(x, y + l) {
            t.kind = kind;
            touch(world, x, y + l);
        }
    }
    // The trailing TileFrame 3×(height+2) loop (:51767-51773) — rows
    // y−1..y+height (7 rows for the 5-tall gate); centred at y+height/2 the
    // farthest row is 3 away, so radius 4 keeps the documented superset.
    ensure_frame_important(world, kind);
    framing::reframe_area(world, x, y + height / 2, 4, false);
    true
}
