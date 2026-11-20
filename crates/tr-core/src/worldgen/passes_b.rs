//! W4-g37 — the worldgen "missing-b" lane: the nine classic passes the d10x
//! census left unexecuted, remainder #10-18 in C# anchor order (`Statues` …
//! `Surface Ore and Stone`, `WorldGen.cs:16853-18447`). See
//! `docs/research/12h-g37-worldgen-missing-b.md` for the claim + census.
//!
//! Each pass draws from its own `pass_rng("<vanilla name>")` stream (the port's
//! model of the vanilla per-pass re-seed, `WorldGenerator.cs:497`), so inserting
//! a step shifts no other step's ledger. The three contract streams
//! (`rng`/`drng`/`srng`) are untouched.
//!
//! Shared placement helpers below are B-lane-local slices of the C#
//! `WorldGen.PlaceTile` / `Place*` family, ported verbatim for the kinds these
//! passes exercise. The general `Place1x2`/`Place1x1`/`Place3x2` wrappers also
//! land on the sibling `W4-g39` helper lane (`place_helpers.rs`); the merger
//! reconciles the duplicates (house style: local cited slices, cf. the
//! `solid_tile` copies in `evil.rs`/`temple.rs`/`spider_gem_moss.rs`).
//!
//! Deviations (recorded per pass + in the lane doc):
//! - the special-seed arms (skyblock / remix / drunk / not-the-bees /
//!   tenth-anniversary / star-game / dual-dungeons / no-traps) are deferred —
//!   a classic world reads none of them; the gates are cited inline.
//! - the C#'s pass-local `Main.tileSolid[...]` toggles (e.g. `:16981-16984`,
//!   `:17713-17715`, `:17885-17886`) are not modelled — the port's
//!   `td::TILE_SOLID` is the static boot table (the `spider_gem_moss.rs`
//!   `tileSolid[379]` precedent); the solidity reads use it unchanged.
//! - `PlaceTile`'s sound/dust tail (`mute: true` on every B call) is a no-op;
//!   the preamble arms that never fire for the B kinds (the grass/moss remaps
//!   `:59364-59430`, the drip/torch/liquid gates `:59450-59472`, the
//!   halfBrick-reset arm `:59477-59482`, the flower/vine family
//!   `:59483-59971`) are out of slice — the B kinds reach the entry gate, the
//!   Clear arm and their own dispatch case only.
//! - chest loot rides the `chest_stock` hook (S1-A-4); the placement half of
//!   `AddBuriedChest` is ported, the stocking block is the app seam.

use super::noise::Rng;
use crate::tile::{Tile, FLAG_ACTIVE};
use crate::tiledata as td;
use crate::world::World;

// --- shared geometry / solidity helpers -------------------------------------

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9695`).
pub(crate) fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `WorldGen.oceanDepths` (`WorldGen.cs:10963-10974`) — the ocean level
/// (`:4447`: `(worldSurface + rockLayer)/2 + 40`) and the ±`beachDistance`
/// (380, `:86914`) margins. No RNG.
pub(crate) fn ocean_depths(world: &World, x: i32, y: i32, world_surface: f64, rock_layer: f64) -> bool {
    let ocean_level = (world_surface + rock_layer) / 2.0 + 40.0;
    if y as f64 > ocean_level {
        return false;
    }
    const BEACH_DISTANCE: i32 = 380;
    x < BEACH_DISTANCE || x > world.width - BEACH_DISTANCE
}

/// `WorldGen.SolidTile2(Tile)` (`WorldGen.cs:70134-70151`) — the Tile overload:
/// null → true (the port's dense array has no nulls, so this evaluates the
/// predicate directly); active, `tileSolid`, slopeless, non-halfBrick, not
/// actuated. No `tileSolidTop` exclusion and no platform refinement (those ride
/// the indexed overload, [`crate::edit::solid_tile2`]).
fn solid_tile2_of(t: &Tile) -> bool {
    t.active()
        && td::TILE_SOLID.has(t.kind)
        && t.slope() == 0
        && !t.half_brick()
        && !t.inactive()
}

/// `Tile.anyShimmer()` (`Tile.cs:428-435`): a positive liquid amount whose kind
/// flag is shimmer.
fn any_shimmer(t: &Tile) -> bool {
    t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Shimmer
}

// --- the PlaceTile slice -----------------------------------------------------

/// `WorldGen.PlaceTile(i, j, Type, mute, forced, plr, style)`
/// (`WorldGen.cs:59342-60295`), the B-lane slice. The `plr` argument is −1 on
/// every B call (the 79/90 bed-direction arm `:60202-60212` reads it; no B
/// kind is a bed), so it is dropped from the signature. The preamble gates
/// that bind for the B kinds are ported verbatim (the 488 mannequin veto
/// `:59345`, the bounds `:59354`, the `forced || EmptyTile || !tileSolid[num]`
/// entry `:59373` — `Collision.EmptyTile` `Collision.cs:1359-1381` reduces to
/// `!tile.active()` during generation, no players/NPCs exist — the torch
/// liquid gate `:59454-59467`, the `Clear(Tile|TilePaint|Slope)` on an inactive
/// cell `:59473-59476` and the ResetsHalfBrick arm `:59477-59482`,
/// `Tile.cs:744-790`). The kind dispatch covers the B kinds; every other kind
/// falls to the C# default single-tile write (`:60234-60248`). The tail
/// (`:60255-60291`) re-frames and returns `result`; `mute: true` skips the
/// sound/dust block.
pub(crate) fn place_tile(
    world: &mut World,
    i: i32,
    j: i32,
    kind: u16,
    mute: bool,
    forced: bool,
    style: i32,
    rng: &mut Rng,
) -> bool {
    let num = kind;
    // `:59345-59348`: the mannequin 488 veto while generating.
    if world.tile(i, j).is_some_and(|t| t.active() && t.kind == 488) {
        return false;
    }
    // `:59349-59352`: `num >= TileID.Count`.
    if num as usize >= td::TILE_COUNT {
        return false;
    }
    let mut result = false;
    // `:59354`: the bounds gate.
    if i < 0 || j < 0 || i >= world.width || j >= world.height {
        return false;
    }
    let active_here = world.tile(i, j).is_some_and(|t| t.active());
    // `:59364-59372`: the grass-on-mud remaps (23/199 → 661/662) never fire
    // for the B kinds.
    // `:59373`: `forced || Collision.EmptyTile(i, j) || !Main.tileSolid[num] ||
    // …` — the grass-on-dirt / moss disjuncts never fire for the B kinds; the
    // generation-time EmptyTile is `!active` (no players/NPCs exist).
    let entry = forced || !active_here || !td::TILE_SOLID.has(num);
    if !entry {
        return false;
    }
    // `:59375-59430`: the grass/moss type guards — none of the B kinds is
    // 23/199/2/109/60/661/662/70/633 or a `tileMoss` kind, so they pass.
    // `:59431-59453`: the 81 coral floor arm and the IsADripTile veto — no B
    // kind is 81 or a drip tile {373, 374, 375, 461, 709}.
    // `:59454-59472`: the liquid gate. `checkingLiquid` is false during
    // generation; the torch arm binds only for kind 4 (the B5 house torches).
    if let Some(t) = world.tile(i, j) {
        if t.liquid > 0 {
            if num == 4 && style != 8 && style != 11 && style != 17 {
                return false;
            }
            // `:59468`: the non-torch liquid-veto list — no B kind is in it.
        }
    }
    // `:59473-59476`: an inactive cell is cleared (Tile | TilePaint | Slope) —
    // type/active/frames zeroed, paint + slope + halfBrick cleared; wall,
    // liquid, wiring and actuator survive (`Tile.Clear`, `Tile.cs:744-790`).
    if !active_here {
        if let Some(t) = world.tile_mut(i, j) {
            t.kind = 0;
            t.flags &= !FLAG_ACTIVE;
            t.frame_x = 0;
            t.frame_y = 0;
            t.set_paint(0);
            t.set_slope(0);
            t.flags &= !crate::tile::FLAG_HALF_BRICK;
        }
    } else if resets_half_brick(num)
        && !world
            .tile(i, j)
            .is_some_and(|t| crate::tileobject::frame_important(t.kind))
    {
        // `:59477-59482`: the ResetsHalfBrickPlacementAttempt arm — the grass
        // kinds clear halfBrick + frames on an active non-frame-important cell.
        if let Some(t) = world.tile_mut(i, j) {
            t.flags &= !crate::tile::FLAG_HALF_BRICK;
            t.frame_y = 0;
            t.frame_x = 0;
        }
    }
    // `:59483-59971`: the flower / vine / special-object arms — none of the B
    // kinds reaches them (the B kinds dispatch below).
    // The kind dispatch (the B arms of the `:59974-60249` switch).
    match num {
        // `:60065-60078`: the 2xX statue/fountain family.
        105 | 349 | 207 => {
            place_2xx(world, i, j, num, style);
            crate::edit::square_tile_frame_gen(world, i, j);
        }
        // `:60129-60140`: the 1x1 mechanism family (the statue trap plate 135).
        135 => {
            place_1x1(world, i, j, num as i32, style, rng);
            crate::edit::square_tile_frame_gen(world, i, j);
        }
        // `:59987-60003`: the 3x2 family (hellforge 77, piles 186/187, table 14).
        77 | 186 | 187 | 14 => {
            place_3x2(world, i, j, num, style);
            crate::edit::square_tile_frame_gen(world, i, j);
        }
        // `:60027-60041`: the 1x2 chair family (15).
        15 => {
            place_1x2(world, i, j, num, style, rng);
            crate::edit::square_tile_frame_gen(world, i, j);
        }
        // `:59843-59883`: the door 10 branch.
        10 => {
            place_door_branch(world, i, j, style, rng);
        }
        // `:59798-59842`: the torch 4 support arm (the B5 house torches).
        4 => {
            if torch_support(world, i, j) {
                if let Some(t) = world.tile_mut(i, j) {
                    t.flags |= FLAG_ACTIVE;
                    t.kind = 4;
                    t.frame_y = (22 * style) as i16;
                }
                crate::edit::square_tile_frame_gen(world, i, j);
            }
        }
        // `:60116-60119`: the banner 91.
        91 => {
            place_banner(world, i, j, num, style);
            crate::edit::square_tile_frame_gen(world, i, j);
        }
        // `:60234-60248`: the default single-tile write.
        _ => {
            if let Some(t) = world.tile_mut(i, j) {
                t.flags |= FLAG_ACTIVE;
                t.kind = num;
            }
        }
    }
    // `:60255-60262`: the tail re-frame + result. The `TruncatesWalls`
    // SquareWallFrame arm (`:60257-60260`) never fires for the B kinds — the
    // set is {54, 328, 459, 748} (`TileID.cs:595`), none of them placed here.
    if world.tile(i, j).is_some_and(|t| t.active()) {
        crate::edit::square_tile_frame_gen(world, i, j);
        result = true;
    }
    let _ = mute; // `:60263-60290`: the sound/dust block — every B call mutes.
    result
}

/// `TileID.Sets.ResetsHalfBrickPlacementAttempt` (`TileID.cs:497`,
/// `CreateBoolSet(true, …)` — default true, the listed kinds false): the grass
/// / moss / ash-grass family that clears halfBrick + frames on re-placement.
fn resets_half_brick(kind: u16) -> bool {
    !matches!(
        kind,
        2 | 23
            | 661
            | 60
            | 70
            | 199
            | 662
            | 109
            | 477
            | 492
            | 179
            | 512
            | 180
            | 513
            | 181
            | 514
            | 182
            | 515
            | 183
            | 516
            | 381
            | 517
            | 534
            | 535
            | 536
            | 537
            | 539
            | 540
            | 625
            | 626
            | 627
            | 628
            | 633
    )
}

/// The torch-4 support gate of `PlaceTile` (`WorldGen.cs:59835`): a wall at the
/// cell, or a solid side (the slope parity arms), or a solid floor (the
/// platform / 380-planter refinement). `IsTreeType` rides the port's
/// `torch::is_tree_type_pub`; `TopEdgeCanBeAttachedTo` the torch module's.
fn torch_support(world: &World, i: i32, j: i32) -> bool {
    let t = |dx: i32, dy: i32| world.tile(i + dx, j + dy).copied().unwrap_or_default();
    let here = t(0, 0);
    if here.wall > 0 {
        return true;
    }
    let side_solid = |c: Tile| {
        c.active()
            && ((td::TILE_SOLID.has(c.kind)
                && !td::TILE_SOLID_TOP.has(c.kind)
                && !td::SET_NOT_REALLY_SOLID.has(c.kind))
                || td::SET_IS_BEAM.has(c.kind))
    };
    let (l, r, below) = (t(-1, 0), t(1, 0), t(0, 1));
    let (ld, rd) = (t(-1, 1), t(1, 1));
    let (lu, ru) = (t(-1, -1), t(1, -1));
    let tree = |c: Tile| crate::torch::is_tree_type_pub(c.kind as i32);
    let left_ok = l.active()
        && (l.slope() == 0 || l.slope() % 2 != 1)
        && (side_solid(l) || (tree(l) && tree(lu) && tree(ld)));
    let right_ok = r.active()
        && (r.slope() == 0 || r.slope() % 2 != 0)
        && (side_solid(r) || (tree(r) && tree(ru) && tree(rd)));
    let floor_ok = below.active()
        && td::TILE_SOLID.has(below.kind)
        && ((td::SET_PLATFORMS.has(below.kind)
            && crate::torch::top_edge_can_be_attached_to(world, i, j + 1))
            || ((!td::TILE_SOLID_TOP.has(below.kind)
                || (below.kind == 380 && below.slope() == 0))
                && !td::SET_NOT_REALLY_SOLID.has(below.kind)
                && !below.half_brick()
                && below.slope() == 0));
    left_ok || right_ok || floor_ok
}

/// `WorldGen.Place2xX(x, y, type, style)` (`WorldGen.cs:39663-39722`): the
/// 2-wide × `num3`-tall object. The 105-style-34 → 349 remap, the 105 sheet
/// wrap (`style/55`), the 104 (5-tall) / 207 (4-tall) heights, the empty-body +
/// `SolidTile2` floor gate, then the column-major frame writes
/// (`frameX = num + col*18`, `frameY = num2 + row*18`).
pub(crate) fn place_2xx(world: &mut World, x: i32, y: i32, mut kind: u16, mut style: i32) {
    let mut num = style * 36;
    let mut num2 = 0;
    let mut num3 = 3;
    if kind == 105 && style == 34 {
        kind = 349;
        style = 0;
        num = 0;
    }
    if kind == 105 {
        let num4 = style / 55;
        num -= 1980 * num4;
        num2 += 54 * num4;
    }
    if kind == 104 {
        num3 = 5;
    }
    if kind == 207 {
        num3 = 4;
    }
    let mut flag = true;
    for row in (y - num3 + 1)..(y + 1) {
        for col in [x, x + 1] {
            if world.tile(col, row).is_some_and(|t| t.active()) {
                flag = false;
            }
        }
    }
    if flag
        && crate::edit::solid_tile2(world, x, y + 1)
        && crate::edit::solid_tile2(world, x + 1, y + 1)
    {
        for row in 0..num3 {
            for col in 0..2 {
                let (px, py) = (x + col, y - num3 + 1 + row);
                if let Some(t) = world.tile_mut(px, py) {
                    t.flags |= FLAG_ACTIVE;
                    t.frame_y = (num2 + row * 18) as i16;
                    t.frame_x = (num + col * 18) as i16;
                    t.kind = kind;
                }
            }
        }
    }
}

/// `WorldGen.Place1x1(x, y, type, style)` (`WorldGen.cs:45188-45235`): the
/// `InWorld(2)` gate, the 324 seashell arm (not a B kind), the
/// mechanism-on-boulder veto + `SolidTile2` floor + inactive-cell gate, then the
/// frame writes (the 36/144/239 `frameX = style*18` arm, else
/// `frameY = style*18`).
pub(crate) fn place_1x1(world: &mut World, x: i32, y: i32, kind: i32, style: i32, rng: &mut Rng) {
    if !in_world(world, x, y, 2) {
        return;
    }
    let below_solid = crate::edit::solid_tile2(world, x, y + 1);
    let active_here = world.tile(x, y).is_some_and(|t| t.active());
    if kind == 324 {
        // `:45204-45213`: the seashell plain arm (the B trap is 135, not 324).
        let below_table = world
            .tile(x, y + 1)
            .is_some_and(|t| t.active() && crate::tileobject::is_tile_table(t.kind));
        if below_solid || below_table {
            if let Some(t) = world.tile_mut(x, y) {
                t.flags |= FLAG_ACTIVE;
                t.kind = kind as u16;
                t.frame_x = (22 * rng.range(0, 3)) as i16;
                t.frame_y = (22 * style) as i16;
            }
        }
        return;
    }
    // `:45214`: the mechanism-on-boulder veto.
    let below_boulder = world
        .tile(x, y + 1)
        .is_some_and(|t| td::SET_BOULDERS.has(t.kind));
    let veto = td::SET_IS_A_MECHANISM.has(kind as u16) && below_boulder;
    if !veto && below_solid && !active_here {
        if let Some(t) = world.tile_mut(x, y) {
            t.flags |= FLAG_ACTIVE;
            t.kind = kind as u16;
            match kind {
                36 | 144 | 239 => {
                    t.frame_x = (style * 18) as i16;
                    t.frame_y = 0;
                }
                _ => {
                    t.frame_y = (style * 18) as i16;
                }
            }
        }
    }
}

/// `WorldGen.Place3x2(x, y, type, style)` (`WorldGen.cs:52066-52196`): the
/// 3-wide × 2-tall object. The 14-style-25 picnic one-row arm, the campfire
/// liquid veto, the 186/187 pile + 26 altar floor vetoes, the 285-family
/// solidTop-frameY-0 floor arm, the default `SolidTile2` floor, then the
/// column-major frame writes (`frameX = 54*style + col*18`). The 88 chest
/// ledger arm (`Chest.CreateChest`) is the chest-lane seam (documented no-op).
pub(crate) fn place_3x2(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
        return;
    }
    let picnic = kind == 14 && style == 25;
    let top = if picnic { y } else { y - 1 };
    let mut flag2 = true;
    for col in (x - 1)..(x + 2) {
        for row in top..(y + 1) {
            let Some(t) = world.tile(col, row) else {
                continue;
            };
            if t.active() {
                flag2 = false;
            }
            // `:52096-52099`: the campfire liquid veto (no B kind is a campfire).
            if td::SET_CAMPFIRES.has(kind) && t.liquid > 0 {
                flag2 = false;
            }
        }
        let below = world.tile(col, y + 1);
        match kind {
            // `:52125-52138`: the 285-family solidTop-frameY-0 floor arm.
            285 | 286 | 298 | 299 | 310 | 361 | 362 | 363 | 364 | 582 | 619 => {
                let ok = crate::edit::solid_tile2(world, col, y + 1)
                    || below.is_some_and(|t| {
                        t.active()
                            && !t.inactive()
                            && td::TILE_SOLID_TOP.has(t.kind)
                            && t.frame_y == 0
                    });
                if !ok {
                    flag2 = false;
                }
            }
            // `:52107-52123`: the default arm.
            _ => {
                if (kind == 186 || kind == 187)
                    && invalid_tile_for_piles(world, col, y + 1)
                {
                    flag2 = false;
                }
                if kind == 26 && below.is_some_and(|t| td::SET_BOULDERS.has(t.kind)) {
                    flag2 = false;
                }
                if !crate::edit::solid_tile2(world, col, y + 1) {
                    flag2 = false;
                }
            }
        }
    }
    // `:52141-52151`: the 88 chest ledger — the chest-lane seam (no-op here).
    if flag2 {
        let num2 = (54 * style) as i16;
        if picnic {
            for col in 0..3 {
                if let Some(t) = world.tile_mut(x - 1 + col, y) {
                    t.flags |= FLAG_ACTIVE;
                    t.frame_y = 0;
                    t.frame_x = num2 + (col * 18) as i16;
                    t.kind = kind;
                }
            }
            return;
        }
        for row in 0..2 {
            for col in 0..3 {
                if let Some(t) = world.tile_mut(x - 1 + col, y - 1 + row) {
                    t.flags |= FLAG_ACTIVE;
                    t.frame_y = (row * 18) as i16;
                    t.frame_x = num2 + (col * 18) as i16;
                    t.kind = kind;
                }
            }
        }
    }
}

/// `WorldGen.InvalidTileForPilesOrSpeleothems(x, y)`
/// (`WorldGen.cs:38979-38994`): an active boulder under the pile veto.
fn invalid_tile_for_piles(world: &World, x: i32, y: i32) -> bool {
    if !in_world(world, x, y, 2) {
        return false;
    }
    world
        .tile(x, y)
        .is_some_and(|t| t.active() && td::SET_BOULDERS.has(t.kind))
}

/// `WorldGen.Place1x2(x, y, type, style)` (`WorldGen.cs:46652-46679`): the
/// pre-gate sapling `Next(3)` (type 20 only), the `SolidTile2` floor +
/// inactive-above gate, then the two-row write (`frameY = style*40 + row*18`).
pub(crate) fn place_1x2(world: &mut World, x: i32, y: i32, kind: u16, style: i32, rng: &mut Rng) {
    let frame_x = if kind == 20 { (rng.range(0, 3) * 18) as i16 } else { 0 };
    if crate::edit::solid_tile2(world, x, y + 1)
        && !world.tile(x, y - 1).is_some_and(|t| t.active())
    {
        let num = (style * 40) as i16;
        for row in 0..2 {
            if let Some(t) = world.tile_mut(x, y - 1 + row) {
                t.flags |= FLAG_ACTIVE;
                t.frame_y = num + (row * 18) as i16;
                t.frame_x = frame_x;
                t.kind = kind;
            }
        }
    }
}

/// `WorldGen.PlaceBanner(x, y, type, style)` (`WorldGen.cs:46258-46300`): the
/// 111-style sheet wrap, the solid-non-top anchor above + three clear rows,
/// then the three-row write (`frameY = num2 + row*18`, `frameX = num`).
pub(crate) fn place_banner(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    let mut num = style * 18;
    let mut num2 = 0;
    let mut num3 = style;
    while num3 >= 111 {
        num3 -= 111;
        num -= 1998;
        num2 += 54;
    }
    let anchor = world.tile(x, y - 1).is_some_and(|t| {
        t.active() && !t.inactive() && td::TILE_SOLID.has(t.kind) && !td::TILE_SOLID_TOP.has(t.kind)
    });
    let clear = (0..3).all(|d| !world.tile(x, y + d).is_some_and(|t| t.active()));
    if anchor && clear {
        for row in 0..3 {
            if let Some(t) = world.tile_mut(x, y + row) {
                t.flags |= FLAG_ACTIVE;
                t.frame_y = (num2 + row * 18) as i16;
                t.frame_x = num as i16;
                t.kind = kind;
            }
        }
    }
}

/// The door-10 branch of `PlaceTile` (`WorldGen.cs:59843-59883`): the up arm
/// (j-1/j-2 clear, j-3 solid) places at j-1, else the down arm (j+1/j+2 clear,
/// j-3 solid) places at j+1, else refuse. Both call `PlaceDoor`.
fn place_door_branch(world: &mut World, x: i32, y: i32, style: i32, rng: &mut Rng) {
    let solid_at = |w: &World, px: i32, py: i32| {
        w.tile(px, py).is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind))
    };
    let clear = |w: &World, px: i32, py: i32| !w.tile(px, py).is_some_and(|t| t.active());
    if clear(world, x, y - 1) && clear(world, x, y - 2) && solid_at(world, x, y - 3) {
        place_door(world, x, y - 1, style, rng);
        crate::edit::square_tile_frame_gen(world, x, y);
    } else {
        if clear(world, x, y + 1)
            && clear(world, x, y + 2)
            && solid_at(world, x, y + 3)
        {
            place_door(world, x, y + 1, style, rng);
            crate::edit::square_tile_frame_gen(world, x, y);
        }
    }
}

/// `WorldGen.PlaceDoor(i, j, type, style)` (`WorldGen.cs:31773-31803`): the
/// solid anchor two rows above + `SolidTile` floor two rows below, then the
/// three-row closed door (tile 10) with `frameY = 54*(style%36) + row*18` and
/// `frameX = 54*(style/36) + Next(3)*18` (the swing variant per row).
fn place_door(world: &mut World, x: i32, y: i32, style: i32, rng: &mut Rng) -> bool {
    let num = style / 36;
    let num2 = style % 36;
    let num3 = 54 * num;
    let num4 = 54 * num2;
    let anchored = world
        .tile(x, y - 2)
        .is_some_and(|t| t.active() && !t.inactive() && td::TILE_SOLID.has(t.kind))
        && crate::edit::solid_tile(world, x, y + 2);
    if !anchored {
        return false;
    }
    for row in 0..3 {
        if let Some(t) = world.tile_mut(x, y - 1 + row) {
            t.flags |= FLAG_ACTIVE;
            t.kind = 10;
            t.frame_y = (num4 + row * 18) as i16;
            t.frame_x = (num3 + rng.range(0, 3) * 18) as i16;
        }
    }
    true
}

// --- B1: Statues -------------------------------------------------------------

/// `GenVars.StatuesWithTraps` (`GenVars.cs:474`): the statue-list indices that
/// wire a dart trap — {4, 7, 10, 18}.
const STATUES_WITH_TRAPS: [usize; 4] = [4, 7, 10, 18];

/// `WorldGen.SetupStatueList` (`WorldGen.cs:4449-4488`): the 73-entry statue
/// table — `(tile type, style)` pairs. The first 44 are statue 105 styles 0-43
/// with the 34 → 349 and 43 → 50 overrides, then the 29 appended styles.
fn statue_list() -> Vec<(u16, i32)> {
    let mut list: Vec<(u16, i32)> = (0..44).map(|i| (105u16, i)).collect();
    list[34] = (349, 0);
    list[43] = (105, 50);
    for style in [
        63, 64, 65, 66, 68, 69, 70, 71, 72, 73, 75, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61,
        62, 77, 78, 67, 74, 37, 2,
    ] {
        list.push((105, style));
    }
    list
}

/// `WorldGen.PlaceStatueTrap(x, y)` (`WorldGen.cs:4490-4513`): the
/// `actuallyNoTrapsForRealIMeanIt` + `InWorld(20)` gate, then the 21×21 scan for
/// an inactive cell over a `SolidTile2(Tile)` floor (not 162); place a pressure
/// plate 135 there and wire it back to the statue, returning on the first.
fn place_statue_trap(world: &mut World, x: i32, y: i32, no_traps_for_real: bool, rng: &mut Rng) {
    if no_traps_for_real || !in_world(world, x, y, 20) {
        return;
    }
    for i in -10..=10 {
        for j in -10..=10 {
            let below = world.tile(x + i, y + j + 1).copied();
            let here_active = world.tile(x + i, y + j).is_some_and(|t| t.active());
            let floor_ok = below.is_some_and(|t| solid_tile2_of(&t) && t.kind != 162);
            if !here_active && floor_ok {
                place_tile(world, x + i, y + j, 135, true, false, 0, rng);
                let placed = world
                    .tile(x + i, y + j)
                    .is_some_and(|t| t.active() && t.kind == 135);
                if placed {
                    wire_line(world, (x, y), (x + i, y + j));
                    return;
                }
            }
        }
    }
}

/// `WorldUtils.WireLine(start, end)` (`WorldBuilding/WorldUtils.cs:109-129`):
/// the original start/end captured before the coordinate swaps, the horizontal
/// run at the original start.Y then the vertical run at the original end.X,
/// each cell red-wired (`WorldGen.PlaceWire`, `WorldGen.cs:58111-58120`).
fn wire_line(world: &mut World, start: (i32, i32), end: (i32, i32)) {
    let point = start;
    let point2 = end;
    let (mut sx, mut ex) = (start.0, end.0);
    let (mut sy, mut ey) = (start.1, end.1);
    if ex < sx {
        std::mem::swap(&mut ex, &mut sx);
    }
    if ey < sy {
        std::mem::swap(&mut ey, &mut sy);
    }
    for i in sx..=ex {
        world.wires.set(i, point.1, crate::wiring::WireColor::Red, true);
    }
    for j in sy..=ey {
        world.wires.set(point2.0, j, crate::wiring::WireColor::Red, true);
    }
}

// --- the buried-chest placement core (B2/B3/B5) ------------------------------

/// `Chest.NearOtherChests(x, y)` (`Chest.cs:543-558`): a `BasicChest` tile
/// ({21, 467}) within the ±25 × ±8 window.
fn near_other_chests(world: &World, x: i32, y: i32) -> bool {
    for i in (x - 25)..(x + 25) {
        for j in (y - 8)..(y + 8) {
            if world
                .tile(i, j)
                .is_some_and(|t| t.active() && (t.kind == 21 || t.kind == 467))
            {
                return true;
            }
        }
    }
    false
}

/// `WorldGen.PlaceChest(x, y, type, notNearOtherChests, style)`
/// (`WorldGen.cs:57813-57846`): the boulder-floor veto, the near-chest
/// exclusion, the `TileObject.CanPlace`/`Place` route, then `Chest.CreateChest`
/// (the port's chest record rides the `chest_stock` hook, S1-A-4). Returns the
/// placed anchor on success.
fn place_chest_at(
    world: &mut World,
    x: i32,
    y: i32,
    kind: u16,
    not_near: bool,
    style: i32,
) -> Option<(i32, i32)> {
    // `:57816-57819`: the boulder-floor veto.
    let boulder_below = |px: i32| {
        world
            .tile(px, y + 1)
            .is_some_and(|t| td::SET_BOULDERS.has(t.kind))
    };
    if boulder_below(x) || boulder_below(x + 1) {
        return None;
    }
    if not_near && near_other_chests(world, x - 1, y - 1) {
        return None;
    }
    let Some(def) = crate::tileobject::def_for_tile(kind) else {
        return None;
    };
    // The placement stream is taken out for the read-only `can_place` call and
    // restored after (chest 21/467 have RandomStyleRange 0, so this draws
    // nothing — the pass stream stays aligned; the temple.rs:1988 precedent).
    let mut rand = std::mem::take(&mut world.place_rng);
    let instance = crate::tileobject::can_place(
        world,
        x,
        y,
        def,
        style,
        1,
        None,
        &mut rand,
        world.tile_object_random_cache,
    );
    world.place_rng = rand;
    let instance = instance?;
    if crate::tileobject::place(world, &instance) {
        Some((instance.x_coord, instance.y_coord))
    } else {
        None
    }
}

/// `WorldGen.IsUndergroundDesert(x, y)` (`WorldGen.cs:35755-35781`): the
/// surface/15-85 % band gates, then the ±15 wall-187/216 scan.
fn is_underground_desert(world: &World, x: i32, y: i32, world_surface: f64) -> bool {
    if (y as f64) < world_surface {
        return false;
    }
    if (x as f64) < world.width as f64 * 0.15 || (x as f64) > world.width as f64 * 0.85 {
        return false;
    }
    for i in (x - 15)..=(x + 15) {
        for j in (y - 15)..=(y + 15) {
            if world.tile(i, j).is_some_and(|t| t.wall == 187 || t.wall == 216) {
                return true;
            }
        }
    }
    false
}

/// `WorldGen.IsDungeon(x, y)` (`WorldGen.cs:35783-35798`): below the surface,
/// in bounds, and a `Main.wallDungeon` wall.
fn is_dungeon_at(world: &World, x: i32, y: i32, world_surface: f64) -> bool {
    if (y as f64) < world_surface || y >= world.height {
        return false;
    }
    if x < 0 || x >= world.width {
        return false;
    }
    world
        .tile(x, y)
        .is_some_and(|t| crate::scene::is_dungeon_wall(t.wall))
}

/// `WorldGen.IsLockedDungeonBiomeChest(chestType, chestStyle)`
/// (`WorldGen.cs:35850-35869`): the 21 styles 23-27 and the 467 style 13.
fn is_locked_dungeon_biome_chest(kind: u16, style: i32) -> bool {
    match kind {
        21 => (23..=27).contains(&style),
        467 => style == 13,
        _ => false,
    }
}

/// `WorldGen.AddBuriedChest(i, j, mainItemInChest, notNearOtherChests,
/// chestStyle, trySlope, chestTileType)` (`WorldGen.cs:35877-37568`), the
/// placement half. The downward floor scan with the per-row rejections
/// (shimmer / larva 231 / the ±2 boulder-altar box), the `trySlope` top-slope
/// flatten + restore, the style resolution (`num9`) and the family dispatch,
/// then `PlaceChest(i-1, k-1)`. The stocking block (`:36218-37551`) rides the
/// `chest_stock` hook (S1-A-4). Returns the placed anchor.
///
/// The B callers pass `chestTileType 0` (→ 21) and styles {-1, 12, 17, 13, 2};
/// the desert/ice/hell kind-overrides require depth bands + tile probes a
/// surface/water/sky chest never reaches in a classic world, so the kind stays
/// 21 and the style is the caller's (the `flag15` default for −1). The
/// special-seed arms (errorWorld, getGood, notTheBees, remix, drunk, tenth,
/// dualDungeons, noTraps) are deferred — a classic world reads none.
#[allow(clippy::too_many_arguments)]
fn add_buried_chest(
    world: &mut World,
    i: i32,
    j: i32,
    main_item: u16,
    not_near: bool,
    chest_style: i32,
    try_slope: bool,
    world_surface: f64,
    rock_layer: f64,
    rng: &mut Rng,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
) -> Option<(i32, i32)> {
    let mut chest_kind: u16 = 21; // `:35880-35883`: chestTileType 0 → 21.
    for k in j..(world.height - 10) {
        // `:35906-35913`: a shimmer cell or the larva 231 aborts the scan.
        if world.tile(i, k).is_some_and(|t| any_shimmer(t)) {
            return None;
        }
        if world.tile(i, k).is_some_and(|t| t.active() && t.kind == 231) {
            return None;
        }
        // `:35914-35944`: the trySlope top-slope flatten (the style-17 ±30
        // chest-exclusion box runs first).
        let mut slope_left: i32 = -1;
        let mut slope_here: i32 = -1;
        if try_slope
            && world.tile(i, k).is_some_and(|t| {
                t.active() && td::TILE_SOLID.has(t.kind) && !td::TILE_SOLID_TOP.has(t.kind)
            })
        {
            if chest_style == 17 {
                for l in (i - 30)..=(i + 30) {
                    for m in (k - 30)..=(k + 30) {
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
                slope_left = world.tile(i - 1, k).map(|t| t.slope() as i32).unwrap_or(0);
                if let Some(t) = world.tile_mut(i - 1, k) {
                    t.set_slope(0);
                }
            }
            if world.tile(i, k).is_some_and(|t| t.top_slope()) {
                slope_here = world.tile(i, k).map(|t| t.slope() as i32).unwrap_or(0);
                if let Some(t) = world.tile_mut(i, k) {
                    t.set_slope(0);
                }
            }
        }
        // `:35945-35948`: the remix centre-band abort (special seed — deferred).
        // `:35949-35959`: the ±2 boulder/altar box (InWorld fluff 100).
        for n in (i - 2)..=(i + 2) {
            for m in (k - 2)..=(k + 2) {
                if in_world(world, n, m, 100)
                    && world.tile(n, m).is_some_and(|t| {
                        t.active()
                            && (td::SET_BOULDERS.has(t.kind) || t.kind == 26 || t.kind == 237)
                    })
                {
                    return None;
                }
            }
        }
        // `:35960-35963`: the floor gate.
        if !crate::edit::solid_tile(world, i, k) {
            continue;
        }
        // `:35964-35985`: the style resolution. `flag15` = below
        // worldSurface+25 → num9 = 1; the caller's chestStyle overrides.
        let flag15 = (k as f64) >= world_surface + 25.0;
        let mut num9 = if flag15 { 1 } else { 0 };
        if chest_style >= 0 {
            num9 = chest_style;
        }
        let mut num10: u16 = main_item;
        // `:36023-36048`: flag3 — the underground-desert arm (the num10 == 0 +
        // depth + wall probe). The desert main roll rides the stocking hook.
        if num10 == 0
            && k <= world.height - 205
            && is_underground_desert(world, i, k, world_surface)
        {
            num9 = 10;
            chest_kind = 467;
        }
        // `:36064-36088`: flag2 — the ice arm (the 147/161/162/197 tile probe
        // below worldSurface+25, above maxTilesY-205).
        let ice_floor = world
            .tile(i, k)
            .is_some_and(|t| matches!(t.kind, 147 | 161 | 162 | 197));
        if num10 == 0
            && (k as f64) >= world_surface + 25.0
            && k <= world.height - 205
            && ice_floor
        {
            num9 = 11;
        }
        // `:36115-36125`: flag8 — the hell arm (below maxTilesY-205, num10 == 0).
        if k > world.height - 205 && num10 == 0 {
            num9 = 4;
        }
        // `:36134-36150`: the style-12 living-tree wall gate.
        if chest_kind == 21 && num9 == 12 {
            if world.tile(i - 1, k - 1).map(|t| t.wall).unwrap_or(0) != 244 {
                num9 = 0;
                num10 = 0;
            }
        }
        // `:36163-36170`: the dungeon / locked-biome flags (stocking-side).
        let _dungeon = num9 != 0 && is_dungeon_at(world, i, k, world_surface);
        let _locked = is_locked_dungeon_biome_chest(chest_kind, num9);
        // `:36206`: PlaceChest(i-1, k-1, chestTileType, notNearOtherChests, num9).
        if let Some(anchor) = place_chest_at(world, i - 1, k - 1, chest_kind, not_near, num9) {
            // The stocking block (`:36218-37551`) — the chest_stock hook. The
            // band/context reads ride the pass's layer anchors (world_surface
            // is only published by the world tail, after this step).
            let saved_surface = world.world_surface;
            let saved_rock = world.rock_layer;
            world.world_surface = world_surface;
            world.rock_layer = rock_layer;
            super::chest_stock::stock_placed_chest(
                world,
                stock.as_deref_mut(),
                rng,
                anchor.0,
                anchor.1,
                num9,
                chest_kind,
                num10,
                not_near,
            );
            world.world_surface = saved_surface;
            world.rock_layer = saved_rock;
            return Some(anchor);
        }
        // `:37554-37564`: the trySlope restore on a failed place.
        if try_slope {
            if slope_left > -1 {
                if let Some(t) = world.tile_mut(i - 1, k) {
                    t.set_slope(slope_left as u8);
                }
            }
            if slope_here > -1 {
                if let Some(t) = world.tile_mut(i, k) {
                    t.set_slope(slope_here as u8);
                }
            }
        }
        return None;
    }
    None
}

/// The `Statues` pass (`WorldGen.cs:16853-16970`): the width-scaled statue
/// count, the per-statue rejection search (the ocean-depths re-roll budget, the
/// fall-to-floor, the frame-important / shimmer veto), the forced `PlaceTile`,
/// and the trap wire on the four trap indices. Returns the number placed.
pub(crate) fn pass_statues(
    world: &mut World,
    world_surface: f64,
    rock_layer: f64,
    no_traps_for_real: bool,
    rng: &mut Rng,
) -> u32 {
    let statues = statue_list();
    let len = statues.len() as i32;
    // `:16859-16860`: `num3 = (statueList.Length * 2) * (maxTilesX / 4200.0)`.
    // The noTraps ×15 / starGame / skyblock arms (`:16861-16878`) are special
    // seeds — a classic world reads none.
    let num3 = ((len * 2) as f64 * (world.width as f64 / 4200.0)) as i32;
    let num4 = 10000;
    // `:16893`: the y band `((int)(worldSurface*2 + rockLayer)) / 3 .. maxY-300`.
    let y_lo = ((world_surface * 2.0 + rock_layer) as i32) / 3;
    let y_hi = world.height - 300;
    // The sub-vanilla guard: the C# `Next(lo, hi)` throws on a collapsed band;
    // the port's tiny test worlds skip (the d10x idiom).
    if y_hi <= y_lo || world.width <= 40 || num3 <= 0 {
        return 0;
    }
    let mut num = 0i32; // the statue-list cursor
    let mut placed = 0u32;
    for _i in 0..num3 {
        if num >= len {
            num = 0;
        }
        let (x_type, y_style) = statues[num as usize];
        let mut flag = false;
        let mut num5 = 0;
        while !flag {
            let mut num6 = rng.range(20, world.width - 20);
            let mut num7 = rng.range(y_lo, y_hi);
            // `:16898-16912`: the ocean-depths re-roll budget (the dual-dungeons
            // bounds disjunct is a special seed — deferred).
            let mut num8 = 100;
            while ocean_depths(world, num6, num7, world_surface, rock_layer) {
                num8 -= 1;
                if num8 <= 0 {
                    break;
                }
                num6 = rng.range(20, world.width - 20);
                num7 = rng.range(y_lo, y_hi);
            }
            if num8 <= 0 {
                flag = true;
            } else {
                // `:16919-16926`: fall to the first active tile.
                while num7 < world.height && !world.tile(num6, num7).is_some_and(|t| t.active()) {
                    num7 += 1;
                }
                if num7 >= world.height {
                    flag = true;
                } else {
                    num7 -= 1;
                    // `:16934-16943`: the frame-important / shimmer veto.
                    let mut flag2 = true;
                    if let Some(t) = world.tile(num6, num7) {
                        if t.active()
                            && crate::tileobject::frame_important(t.kind)
                            && !crate::tileobject::tile_cut(t.kind)
                        {
                            flag2 = false;
                        }
                        if any_shimmer(t) {
                            flag2 = false;
                        }
                    }
                    if flag2 {
                        place_tile(world, num6, num7, x_type, true, true, y_style, rng);
                    }
                    // `:16948-16964`: the success / retry arm.
                    let ok = world
                        .tile(num6, num7)
                        .is_some_and(|t| t.active() && t.kind == x_type);
                    if ok {
                        flag = true;
                        if STATUES_WITH_TRAPS.contains(&(num as usize)) {
                            place_statue_trap(world, num6, num7, no_traps_for_real, rng);
                        }
                        num += 1;
                        placed += 1;
                    } else {
                        num5 += 1;
                        if num5 >= num4 {
                            flag = true;
                        }
                    }
                }
            }
        }
    }
    placed
}

// --- B2: Surface Chests ------------------------------------------------------

/// The `Surface Chests` pass (`WorldGen.cs:17130-17210`): the width-scaled
/// surface-chest count, the per-chest rejection search (the ocean-depths
/// re-roll, the wall-2/59/244 surface gate, the ±50 wall-244 living-tree scan),
/// then `AddBuriedChest` with the living-tree style 12 / the default −1.
/// Returns the number placed.
pub(crate) fn pass_surface_chests(
    world: &mut World,
    world_surface: f64,
    surface_low: f64,
    rock_layer: f64,
    no_surface: bool,
    rng: &mut Rng,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
) -> u32 {
    // `:17132`: the noSurface gate (a classic world reads false).
    if no_surface {
        return 0;
    }
    // `:17140`: the count `(int)(maxTilesX * 0.005)`.
    let count = (world.width as f64 * 0.005) as i32;
    let num = 2000; // `:17135`: the give-up budget.
    // `:17148`: the y band `(int)worldSurfaceLow .. (int)worldSurface`.
    let y_lo = surface_low as i32;
    let y_hi = world_surface as i32;
    // The sub-vanilla guard: the C# Next(lo, hi) throws on a collapsed band.
    if y_hi <= y_lo || world.width <= 400 || count <= 0 {
        return 0;
    }
    let mut placed = 0u32;
    for _i in 0..count {
        let mut flag = false;
        let mut num2 = 0;
        while !flag {
            let mut num3 = rng.range(200, world.width - 200);
            let mut num4 = rng.range(y_lo, y_hi);
            // `:17155-17159`: the ocean-depths re-roll (the remix arm deferred).
            while ocean_depths(world, num3, num4, world_surface, rock_layer) {
                num3 = rng.range(300, world.width - 300);
                num4 = rng.range(y_lo, y_hi);
            }
            // `:17161-17194`: the surface gate. flag2 = the living-tree style
            // (wall 244), flag3 = a valid surface site.
            let mut flag2 = false;
            let mut flag3 = false;
            let cell = world.tile(num3, num4).copied().unwrap_or_default();
            if !cell.active() {
                // `:17165-17172`: wall 2 / 59 / 244 (the remix disjunct deferred).
                if cell.wall == 2 || cell.wall == 59 || cell.wall == 244 {
                    if cell.wall == 244 {
                        flag2 = true;
                    }
                    flag3 = true;
                }
            } else {
                // `:17176-17193`: the ±50 wall-244 scan (the `j += 2` / `k += 2`
                // stride from num6-50 / num7-50, the Next(num8) == 0 acceptance
                // with the num8++ tightening).
                let mut num8 = 1;
                let mut j = num3 - 50;
                while j <= num3 + 50 {
                    let mut k = num4 - 50;
                    while k <= num4 + 50 {
                        if (k as f64) < world_surface
                            && world
                                .tile(j, k)
                                .is_some_and(|t| !t.active() && t.wall == 244)
                            && rng.range(0, num8) == 0
                        {
                            flag2 = true;
                            num8 += 1;
                            flag3 = true;
                            num3 = j;
                            num4 = k;
                        }
                        k += 2;
                    }
                    j += 2;
                }
            }
            // `:17195-17206`: the AddBuriedChest call (style 12 for the
            // living-tree site, else −1) + the give-up budget.
            let style = if flag2 { 12 } else { -1 };
            if flag3
                && add_buried_chest(
                    world,
                    num3,
                    num4,
                    0,
                    true,
                    style,
                    false,
                    world_surface,
                    rock_layer,
                    rng,
                    stock.as_deref_mut(),
                )
                .is_some()
            {
                flag = true;
                placed += 1;
            } else {
                num2 += 1;
                if num2 >= num {
                    flag = true;
                }
            }
        }
    }
    placed
}

// --- B3: Water Chests --------------------------------------------------------

/// `WorldGen.UnderwaterChests` (`WorldGen.cs:17245-17355`): the ocean-cave
/// treasure chests (the widening ±num search around each `oceanCaveTreasure`
/// anchor), then the width-scaled scatter (the 1-in-10 water-breathing 863 /
/// the 186/4404/277/187 cycle, the two liquid-search loops). Every chest is
/// `AddBuriedChest(…, style 17, trySlope: beach-band, 0)`. Returns the number
/// placed.
#[allow(clippy::too_many_arguments)]
pub(crate) fn pass_water_chests(
    world: &mut World,
    gv_num_treasure: i32,
    gv_treasure: [(i32, i32); 2],
    world_surface: f64,
    rock_layer: f64,
    rng: &mut Rng,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
) -> u32 {
    let mut placed = 0u32;
    let underworld_layer = world.height - 200; // `Main.UnderworldLayer` (Main.cs:2790).
    const BEACH_DISTANCE: i32 = 380;
    // The sub-vanilla guard: the C# `Next(50, maxX-50)` / liquid-search bands
    // collapse on a tiny world; skip (the d10x idiom).
    if world.width <= 100 || underworld_layer <= 1 {
        return 0;
    }
    // `:17250-17270`: the ocean-cave treasure chests.
    for i in 0..gv_num_treasure {
        // `:17252`: NextFromList{863, 186, 277, 187, 4404} (the extraLiquid 268
        // arm is a special seed — deferred).
        let main_item = [863u16, 186, 277, 187, 4404][rng.range(0, 5) as usize];
        let mut flag = false;
        let mut num = 2.0f64;
        while !flag && num < 50.0 {
            num += 0.1;
            let (tx, ty) = gv_treasure[i as usize];
            let mut num2 = rng.range(tx - num as i32, tx + num as i32 + 1);
            let num3 = rng.range(ty - (num as i32) / 2, ty + (num as i32) / 2 + 1);
            // `:17264`: the edge fold.
            num2 = if num2 >= world.width {
                (num2 as f64 + num / 2.0) as i32
            } else {
                (num2 as f64 - num / 2.0) as i32
            };
            // `:17265`: InWorld + liquid > 250 + water (the notTheBees/remix
            // disjuncts are special seeds — deferred).
            let water = world.tile(num2, num3).is_some_and(|t| {
                t.liquid > 250 && t.liquid_kind() == crate::tile::LiquidKind::Water
            });
            if in_world(world, num2, num3, 0) && water {
                if add_buried_chest(
                    world, num2, num3, main_item, false, 17, true,
                    world_surface, rock_layer, rng, stock.as_deref_mut(),
                )
                .is_some()
                {
                    flag = true;
                    placed += 1;
                }
            }
        }
    }
    // `:17271-17353`: the width-scaled scatter.
    let num5 = world.width as f64 / 4200.0; // `:17272` (extraLiquid ×3 deferred).
    let scatter_count = (9.0 * num5) as i32;
    let mut num4 = 0i32;
    for _j in 0..scatter_count {
        // `:17280-17316`: the main-item pick.
        let num6: u16;
        num4 += 1;
        if rng.range(0, 10) == 0 {
            num6 = 863;
        } else {
            match num4 {
                1 => num6 = 186,
                2 => num6 = 4404,
                3 => num6 = 277,
                _ => {
                    num6 = 187;
                    num4 = 0;
                }
            }
        }
        // `:17317-17334`: the first liquid-search loop (the full-depth band).
        let mut flag2 = false;
        let mut num7 = 0;
        while !flag2 {
            let mut num8 = rng.range(50, world.width - 50);
            let mut num9 = rng.range(1, underworld_layer);
            let mut rerolls = 0;
            while !liquid_water(world, num8, num9) {
                num8 = rng.range(50, world.width - 50);
                num9 = rng.range(50, underworld_layer);
                rerolls += 1;
                // The C# inner loop is unbounded (it hangs on a world with no
                // water); the port caps it (a degenerate-world guard).
                if rerolls > 10000 {
                    break;
                }
            }
            if rerolls > 10000 {
                break;
            }
            let try_slope = num8 < BEACH_DISTANCE || num8 > world.width - BEACH_DISTANCE;
            if add_buried_chest(
                world, num8, num9, num6, false, 17, try_slope,
                world_surface, rock_layer, rng, stock.as_deref_mut(),
            )
            .is_some()
            {
                flag2 = true;
                placed += 1;
            }
            num7 += 1;
            if num7 > 10000 {
                break;
            }
        }
        // `:17335-17352`: the second liquid-search loop (the surface band).
        let mut flag2 = false;
        let mut num7 = 0;
        while !flag2 {
            let mut num10 = rng.range(50, world.width - 50);
            let mut num11 = rng.range(world_surface as i32, underworld_layer);
            let mut rerolls = 0;
            while !liquid_water(world, num10, num11) {
                num10 = rng.range(50, world.width - 50);
                num11 = rng.range(world_surface as i32, underworld_layer);
                rerolls += 1;
                if rerolls > 10000 {
                    break;
                }
            }
            if rerolls > 10000 {
                break;
            }
            let try_slope = num10 < BEACH_DISTANCE || num10 > world.width - BEACH_DISTANCE;
            if add_buried_chest(
                world, num10, num11, num6, false, 17, try_slope,
                world_surface, rock_layer, rng, stock.as_deref_mut(),
            )
            .is_some()
            {
                flag2 = true;
                placed += 1;
            }
            num7 += 1;
            if num7 > 10000 {
                break;
            }
        }
    }
    placed
}

/// The water-chest liquid gate (`WorldGen.cs:17323`/`:17341`): `liquid >= 250`
/// and water kind (the notTheBees/remix disjuncts are special seeds — deferred).
fn liquid_water(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| {
        t.liquid >= 250 && t.liquid_kind() == crate::tile::LiquidKind::Water
    })
}

// --- B4: Cave Walls ----------------------------------------------------------

/// The port's pre-laid baseline walls (`terrain.rs:344-350` writes dirt 2 /
/// stone 1 / sandstone 187 under every solid row; the C# leaves them 0 until
/// this pass). The `spider_gem_moss.rs::BASELINE_WALLS` adaptation: a
/// `wall != 0` test reads these as the C#'s 0, so only a wall the port actually
/// painted (jungle/mud/hive/gem/moss/cave) counts.
const BASELINE_WALLS: [u16; 4] = [1, 2, 187, 216];

/// `WorldGen.countTiles`'s scratch counters (`WorldGen.cs:9546-9557`).
#[derive(Default, Clone, Copy)]
struct CaveCounts {
    tiles: i32,
    shroom: i32,
    lava: i32,
    ice: i32,
    sand: i32,
    rock: i32,
}

/// `WorldGen.countTiles(x, y, jungle, lavaOk)` + `nextCount`
/// (`WorldGen.cs:9546-9635`) over an explicit stack (the C# recursion depth is
/// the pocket size; the fill carries no RNG, so the traversal order is free).
/// The `max` is `WorldGen.maxTileCount` (1500 for this pass). The wall-244 /
/// shimmer / border bail set `tiles = max`; the non-jungle wall-0 / lava gates
/// bail; the jungle mode skips them. Counts the 70/1/147-161/53-396-397 tiles.
fn count_tiles(world: &World, x: i32, y: i32, jungle: bool, lava_ok: bool, max: i32) -> CaveCounts {
    let mut c = CaveCounts::default();
    let mut seen: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
    let mut stack = vec![(x, y)];
    while let Some((x, y)) = stack.pop() {
        if c.tiles >= max {
            return c;
        }
        // `:9565-9568`: the border bail.
        if x <= 1 || x >= world.width - 1 || y <= 1 || y >= world.height - 1 {
            c.tiles = max;
            return c;
        }
        if !seen.insert((x, y)) {
            continue;
        }
        let Some(t) = world.tile(x, y) else {
            continue;
        };
        // `:9575-9584`: the living-wood wall + shimmer bails.
        if t.wall == 244 {
            c.tiles = max;
            return c;
        }
        if t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Shimmer {
            c.tiles = max;
            return c;
        }
        let lava_here = t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Lava;
        // `:9585-9605`: the non-jungle wall/lava gates. The `wall != 0` test
        // reads the port's pre-laid baseline walls as the C#'s 0 (the
        // spider_gem_moss.rs adaptation — the C# runs this pass before any
        // wall exists under the solid rows).
        if !jungle {
            if !BASELINE_WALLS.contains(&t.wall) {
                c.tiles = max;
                return c;
            }
            if lava_here {
                c.lava += 1;
                if !lava_ok {
                    c.tiles = max;
                    return c;
                }
            }
        }
        // `:9606-9624`: the tile counters.
        if t.active() {
            match t.kind {
                70 => c.shroom += 1,
                1 => c.rock += 1,
                147 | 161 => c.ice += 1,
                53 | 396 | 397 => c.sand += 1,
                _ => {}
            }
        }
        // `:9625-9633`: the open-region expansion.
        if !crate::edit::solid_tile(world, x, y) {
            c.tiles += 1;
            stack.push((x - 1, y));
            stack.push((x + 1, y));
            stack.push((x, y - 1));
            stack.push((x, y + 1));
        }
    }
    c
}

/// `Spread.Wall(x, y, wallType)` (`WorldGen.cs:3356-3414`): the wave BFS that
/// paints the wall over the open region; a solid-or-walled cell only takes the
/// wall when it is active and unwalled, then stops the spread.
fn spread_wall(world: &mut World, x: i32, y: i32, wall: u16) {
    if !in_world(world, x, y, 0) {
        return;
    }
    let mut seen: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
    let mut next: Vec<(i32, i32)> = vec![(x, y)];
    while !next.is_empty() {
        let cur = std::mem::take(&mut next);
        for item in cur {
            if !in_world(world, item.0, item.1, 1) {
                continue;
            }
            seen.insert(item);
            let solid = crate::edit::solid_tile(world, item.0, item.1);
            let t = world.tile(item.0, item.1).copied().unwrap_or_default();
            // The `tile.wall != 0` / `== 0` tests read the baseline walls as
            // the C#'s 0 (the spider_gem_moss.rs:630-637 adaptation).
            let real_wall = !BASELINE_WALLS.contains(&t.wall);
            if solid || real_wall {
                if t.active() && !real_wall {
                    if let Some(tt) = world.tile_mut(item.0, item.1) {
                        tt.wall = wall;
                    }
                }
                continue;
            }
            if let Some(tt) = world.tile_mut(item.0, item.1) {
                tt.wall = wall;
            }
            for n in [(item.0 - 1, item.1), (item.0 + 1, item.1), (item.0, item.1 - 1), (item.0, item.1 + 1)] {
                if !seen.contains(&n) {
                    next.push(n);
                }
            }
        }
    }
}

/// `Spread.Wall2(x, y, wallType)` (`WorldGen.cs:3416-3524`): the budgeted
/// (`maxWallOut2` = 5000, `:86922`) wave BFS. The `CannotBeReplacedByWallSpread`
/// walls {4,40,3,83,87,244,34} (`WallID.cs:98`) and the same-wall cells stop;
/// the `WallSpreadStopsAtAir` walls {63,62} (`:99`) refuse air cells and spread
/// to the 8-neighbourhood + the ±2 horizontal; a solid active cell takes the
/// wall without spreading.
fn spread_wall2(world: &mut World, x: i32, y: i32, wall: u16) {
    if !in_world(world, x, y, 0) {
        return;
    }
    const MAX_WALL_OUT: i32 = 5000;
    let stops_at_air = wall == 63 || wall == 62;
    let mut num2 = 0;
    let mut seen: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
    let mut next: Vec<(i32, i32)> = vec![(x, y)];
    while !next.is_empty() {
        let cur = std::mem::take(&mut next);
        for item in cur {
            if !in_world(world, item.0, item.1, 1) {
                continue;
            }
            seen.insert(item);
            let t = world.tile(item.0, item.1).copied().unwrap_or_default();
            // `:3445-3448`: the same-wall / cannot-be-replaced bail.
            let cannot_replace = matches!(t.wall, 4 | 40 | 3 | 83 | 87 | 244 | 34);
            if t.wall == wall || cannot_replace {
                continue;
            }
            if !crate::edit::solid_tile(world, item.0, item.1) {
                // `:3451-3456`: the stops-at-air refusal.
                if stops_at_air && t.wall == 0 {
                    continue;
                }
                num2 += 1;
                if num2 >= MAX_WALL_OUT {
                    continue;
                }
                if let Some(tt) = world.tile_mut(item.0, item.1) {
                    tt.wall = wall;
                }
                let mut nbrs = vec![
                    (item.0 - 1, item.1), (item.0 + 1, item.1),
                    (item.0, item.1 - 1), (item.0, item.1 + 1),
                ];
                if stops_at_air {
                    nbrs.extend([
                        (item.0 - 1, item.1 - 1), (item.0 + 1, item.1 - 1),
                        (item.0 - 1, item.1 + 1), (item.0 + 1, item.1 + 1),
                        (item.0 - 2, item.1), (item.0 + 2, item.1),
                    ]);
                }
                for n in nbrs {
                    if !seen.contains(&n) {
                        next.push(n);
                    }
                }
            } else if t.active() {
                // `:3518-3521`: the solid active cell takes the wall.
                if let Some(tt) = world.tile_mut(item.0, item.1) {
                    tt.wall = wall;
                }
            }
        }
    }
}

/// The `Cave Walls` pass (`WorldGen.cs:17711-17839`,
/// `GenPassNameID.CaveWallsInEnclosedSpaces`): the 4 %-width enclosed-space
/// wall fills (the shroom/ice/lava/stone wall pick), then the 2 %-width jungle
/// wall-64 → 15 fills. The remix second loop (`:17787-17809`) is a special
/// seed — deferred. The `tileSolid[162/226/232]` toggles (`:17713-17715`) are
/// unmodelled (the static `td::TILE_SOLID`, module-header deviation).
pub(crate) fn pass_cave_walls(
    world: &mut World,
    world_surface: f64,
    rock_layer: f64,
    lava_line: i32,
    rng: &mut Rng,
) {
    let max_tile_count = 1500;
    // `:17720-17786`: the enclosed-space fills.
    let count = (world.width as f64 * 0.04) as i32;
    let y_lo = ((world_surface + rock_layer) / 2.0) as i32;
    let y_hi = world.height - 220;
    if y_hi > y_lo && world.width > 400 {
        for _i in 0..count {
            let mut num = 0;
            let mut x = rng.range(200, world.width - 200);
            let mut y = rng.range(y_lo, y_hi);
            let mut c = count_tiles(world, x, y, false, true, max_tile_count);
            while (c.tiles >= max_tile_count || c.tiles < 10) && num < 500 {
                num += 1;
                x = rng.range(200, world.width - 200);
                y = rng.range(y_lo, y_hi);
                c = count_tiles(world, x, y, false, true, max_tile_count);
            }
            if num < 500 {
                // `:17744-17783`: the wall pick.
                let mut num3 = rng.range(0, 2);
                let wall: u16;
                if (c.shroom as f64) > (c.rock as f64) * 0.75 {
                    wall = 80;
                } else if c.ice > 0 {
                    wall = if num3 == 0 { 40 } else { 71 };
                } else if c.lava > 0 {
                    wall = 79;
                } else {
                    num3 = rng.range(0, 4);
                    wall = match num3 {
                        0 => 59,
                        1 => 61,
                        2 => 170,
                        _ => 171,
                    };
                }
                spread_wall(world, x, y, wall);
            }
        }
    }
    // `:17811-17837`: the jungle wall-64 → 15 fills.
    let count2 = (world.width as f64 * 0.02) as i32;
    let jy_lo = world_surface as i32;
    let jy_hi = lava_line;
    if jy_hi > jy_lo && world.width > 400 {
        for _k in 0..count2 {
            let mut num7 = 0;
            let mut num8 = rng.range(200, world.width - 200);
            let mut num9 = rng.range(jy_lo, jy_hi);
            // `:17818-17822`: the first count only when wall == 64.
            let mut num10 = if world.tile(num8, num9).map(|t| t.wall).unwrap_or(0) == 64 {
                count_tiles(world, num8, num9, true, false, max_tile_count).tiles
            } else {
                0
            };
            while (num10 >= max_tile_count || num10 < 10) && num7 < 1000 {
                num7 += 1;
                num8 = rng.range(200, world.width - 200);
                num9 = rng.range(jy_lo, jy_hi);
                let w = world.tile(num8, num9).map(|t| t.wall).unwrap_or(0);
                // `:17828-17831`: the wallHouse / 244 gate keeps num10 stale.
                if !crate::edit::wall_house(w) && w != 244 {
                    num10 = if w == 64 {
                        count_tiles(world, num8, num9, true, false, max_tile_count).tiles
                    } else {
                        0
                    };
                }
            }
            if num7 < 1000 {
                spread_wall2(world, num8, num9, 15);
            }
        }
    }
}

// --- B5: Floating Island Houses ----------------------------------------------

/// `WorldGen.IslandHouse(i, j, islandStyle)` (`WorldGen.cs:79203-79484`): the
/// skyware house on the island's surface — the ±30 dungeon/evil-wall veto, the
/// width/height rolls, the surface-anchor scan, the sunplate-202 shell + the
/// skyware-82 interior, the door, the chest (the `skyIslandHouseCount` cycle),
/// the fountain (islandStyle > 0), the window walls, the table + chairs, the
/// banners, and the palm trees (islandStyle == 1). The classic islandStyle is
/// 0, so the fountain/palm arms are no-ops; the remix/drunk/getGood arms are
/// deferred.
#[allow(clippy::too_many_arguments)]
fn island_house(
    world: &mut World,
    i: i32,
    j: i32,
    island_style: i32,
    world_surface: f64,
    rock_layer: f64,
    sky_count: &mut u32,
    rng: &mut Rng,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
) {
    // `:79206-79216`: the ±30 dungeon/evil-wall veto.
    for k in (i - 30)..=(i + 30) {
        for l in (j - 30)..=(j + 30) {
            if in_world(world, k, l, 0) {
                let w = world.tile(k, l).map(|t| t.wall).unwrap_or(0);
                if crate::scene::is_dungeon_wall(w) || w == 87 || w == 3 || w == 83 {
                    return;
                }
            }
        }
    }
    // `:79221-79222`: the sunplate 202 shell + the skyware 82 interior.
    let kind: u16 = 202;
    let wall: u16 = 82;
    // `:79224-79230`: the direction + width/height rolls.
    let num2 = if rng.range(0, 2) == 0 { -1 } else { 1 };
    let num3 = rng.range(7, 12);
    let num4 = rng.range(5, 7);
    // `:79231-79247`: the surface anchor.
    let anchor_x = i + (num3 + 2) * num2;
    // The C# `vector2D.Y` defaults to 0 (the `default(Vector2D)` at :79223) and
    // is only set by the surface scan; the port keeps the island row as the
    // fallback (the scan always finds the cloud disc on a real island).
    let mut vec_y = j as f64;
    if (j as f64) > world_surface {
        vec_y = (j - 5) as f64;
    } else {
        for m in (j - 15)..(j + 30) {
            let ok = world.tile(anchor_x, m).is_some_and(|t| {
                t.wall > 0
                    && t.active()
                    && crate::edit::solid_tile(world, anchor_x, m)
                    && t.kind != 192
                    && t.kind != 191
                    && t.kind != 384
            });
            if ok {
                vec_y = (m - 1) as f64;
                break;
            }
        }
    }
    let vec_x = i as f64;
    // `:79249-79268`: the shell bounds.
    let mut num5 = (vec_x - num3 as f64 - 1.0) as i32;
    let mut num6 = (vec_x + num3 as f64 + 1.0) as i32;
    let mut num7 = (vec_y - num4 as f64 - 1.0) as i32;
    let mut num8 = (vec_y + 2.0) as i32;
    num5 = num5.max(0);
    num6 = num6.min(world.width);
    num7 = num7.max(0);
    num8 = num8.min(world.height);
    // `:79269-79283`: the sunplate shell (the top-corner skip).
    for n in num5..=num6 {
        for num9 in (num7 - 1)..(num8 + 1) {
            if num9 != num7 - 1 || (n != num5 && n != num6) {
                if let Some(t) = world.tile_mut(n, num9) {
                    t.flags |= FLAG_ACTIVE;
                    t.liquid = 0;
                    t.kind = kind;
                    t.wall = 0;
                    t.flags &= !crate::tile::FLAG_HALF_BRICK;
                    t.set_slope(0);
                }
            }
        }
    }
    // `:79284-79303`: the interior bounds.
    let mut num5 = (vec_x - num3 as f64) as i32;
    let mut num6 = (vec_x + num3 as f64) as i32;
    let mut num7 = (vec_y - num4 as f64) as i32;
    let mut num8 = (vec_y + 1.0) as i32;
    num5 = num5.max(0);
    num6 = num6.min(world.width);
    num7 = num7.max(0);
    num8 = num8.min(world.height);
    // `:79304-79314`: the skyware interior (the top-corner skip, wall-0 only).
    for num10 in num5..=num6 {
        for num11 in num7..num8 {
            if (num11 != num7 || (num10 != num5 && num10 != num6))
                && world.tile(num10, num11).map(|t| t.wall).unwrap_or(0) == 0
            {
                if let Some(t) = world.tile_mut(num10, num11) {
                    t.flags &= !FLAG_ACTIVE;
                    t.wall = wall;
                }
            }
        }
    }
    // `:79315-79322`: the door opening.
    let mut num12 = i + (num3 + 1) * num2;
    let num13 = vec_y as i32;
    for num14 in (num12 - 2)..=(num12 + 2) {
        for dy in 0..3 {
            if let Some(t) = world.tile_mut(num14, num13 - dy) {
                t.flags &= !FLAG_ACTIVE;
            }
        }
    }
    // `:79334-79337`: the classic door (style 9; the remix 5/38 arms deferred).
    place_tile(world, num12, num13, 10, true, false, 9, rng);
    // `:79338-79347`: the far wall column.
    num12 = i + (num3 + 1) * -num2 - num2;
    for num15 in num7..=(num8 + 1) {
        if let Some(t) = world.tile_mut(num12, num15) {
            t.flags |= FLAG_ACTIVE;
            t.liquid = 0;
            t.kind = kind;
            t.wall = 0;
            t.flags &= !crate::tile::FLAG_HALF_BRICK;
            t.set_slope(0);
        }
    }
    // `:79348-79368`: the chest main-item cycle.
    let mut num16 = *sky_count;
    if num16 > 3 {
        num16 = rng.range(0, 4) as u32;
    }
    let main_item: u16 = match num16 {
        0 => 159,
        1 => 65,
        2 => 158,
        _ => 2219,
    };
    // `:79373-79376`: the classic chest (style 13; getGood 2 deferred).
    add_buried_chest(
        world, i, num13 - 3, main_item, false, 13, false,
        world_surface, rock_layer, rng, stock.as_deref_mut(),
    );
    // `:79377-79393`: the fountain (islandStyle > 0 — classic 0, no-op).
    if island_style > 0 {
        for num17 in 0..100000 {
            let num18 = i + rng.range(-50, 51);
            let num19 = num13 + rng.range(0, 21);
            let not_sunplate = world.tile(num18, num19).map(|t| t.kind).unwrap_or(0) != 202;
            if (num17 >= 50000 || not_sunplate)
                && !world.tile(num18, num19).is_some_and(|t| t.active())
            {
                place_2xx(world, num18, num19, 207, island_style);
                if world.tile(num18, num19).is_some_and(|t| t.active()) {
                    switch_fountain(world, num18, num19);
                    break;
                }
            }
        }
    }
    // `:79394`: the counter advance.
    *sky_count += 1;
    // `:79395-79419`: the window walls (classic — the remix gate is false).
    {
        let num20 = i - num3 / 2 + 1;
        let num21 = i + num3 / 2 - 1;
        let num22 = if num3 > 10 { 2 } else { 1 };
        let num23 = (num7 + num8) / 2 - 1;
        for num24 in (num20 - num22)..=(num20 + num22) {
            for num25 in (num23 - 1)..=(num23 + 1) {
                if let Some(t) = world.tile_mut(num24, num25) {
                    t.wall = 21;
                }
            }
        }
        for num26 in (num21 - num22)..=(num21 + num22) {
            for num27 in (num23 - 1)..=(num23 + 1) {
                if let Some(t) = world.tile_mut(num26, num27) {
                    t.wall = 21;
                }
            }
        }
    }
    // `:79420-79441`: the table + chairs (classic 14 style 7 / 15 style 10).
    let num28 = i + (num3 / 2 + 1) * -num2;
    place_tile(world, num28, num8 - 1, 14, true, false, 7, rng);
    place_tile(world, num28 - 2, num8 - 1, 15, true, false, 10, rng);
    place_tile(world, num28 + 2, num8 - 1, 15, true, false, 10, rng);
    // `:79442-79443`: the left chair's frame shift.
    if let Some(t) = world.tile_mut(num28 - 2, num8 - 1) {
        t.frame_x += 18;
    }
    if let Some(t) = world.tile_mut(num28 - 2, num8 - 2) {
        t.frame_x += 18;
    }
    // `:79444-79461`: the banners (classic).
    place_tile(world, num5 + 1, num7, 91, true, false, rng.range(7, 10), rng);
    place_tile(world, num6 - 1, num7, 91, true, false, rng.range(7, 10), rng);
    let (i2, j2) = if num2 > 0 { (num5, num7 + 1) } else { (num6, num7 + 1) };
    place_tile(world, i2, j2, 91, true, false, rng.range(7, 10), rng);
    // `:79462-79483`: the palm trees (islandStyle == 1 — classic 0, no-op).
    if island_style != 1 {
        return;
    }
    let mut num29 = rng.range(3, 6);
    for _num30 in 0..100000 {
        let num31 = i + rng.range(-50, 51);
        let num32 = num13 + rng.range(-10, 21);
        if !world.tile(num31, num32).is_some_and(|t| t.active()) {
            super::ocean::grow_palm_tree(world, num31, num32 + 1, 0, false, rng);
            if world.tile(num31, num32).is_some_and(|t| t.active()) {
                num29 -= 1;
            }
        }
        if num29 <= 0 {
            break;
        }
    }
}

/// `WorldGen.SwitchFountain(i, j)` (`WorldGen.cs:51150-51185`): the 2×4
/// fountain's frameY ±72 toggle (the on/off state). The `Wiring.running`
/// SkipWire tail is a runtime-wiring concern — no-op during generation.
fn switch_fountain(world: &mut World, i: i32, j: i32) {
    let (fx, fy) = world
        .tile(i, j)
        .map(|t| (t.frame_x as i32, t.frame_y as i32))
        .unwrap_or((0, 0));
    let mut num3 = fx / 18;
    while num3 >= 2 {
        num3 -= 2;
    }
    let mut num4 = fy / 18;
    if num4 >= 4 {
        num4 -= 4;
    }
    let num = i - num3;
    let num2 = j - num4;
    for k in num..(num + 2) {
        for l in num2..(num2 + 4) {
            if let Some(t) = world.tile_mut(k, l) {
                if t.active() && t.kind == 207 {
                    if t.frame_y < 72 {
                        t.frame_y += 72;
                    } else {
                        t.frame_y -= 72;
                    }
                }
            }
        }
    }
}

/// The `Floating Island Houses` pass (`WorldGen.cs:17859-17874`): walk the
/// recorded house sites (`GenVars.numIslandHouses`), skipping the sky lakes,
/// and build each `IslandHouse`. Returns the number built.
pub(crate) fn pass_floating_island_houses(
    world: &mut World,
    sites: &[(i32, i32, i32, bool)],
    world_surface: f64,
    rock_layer: f64,
    sky_count: &mut u32,
    rng: &mut Rng,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
) -> u32 {
    let mut built = 0u32;
    for &(x, y, style, lake) in sites {
        if !lake {
            island_house(
                world, x, y, style, world_surface, rock_layer, sky_count, rng,
                stock.as_deref_mut(),
            );
            built += 1;
        }
    }
    built
}
