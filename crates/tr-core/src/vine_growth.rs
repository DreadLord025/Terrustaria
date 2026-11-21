//! The vine grow-tops + the jungle-thorn framing/growth residuals (W4-g83).
//!
//! * [`grow_overground_vines`] (`WorldGen.UpdateWorld_OvergroundTile`'s vine
//!   chain, `WorldGen.cs:72506-72802`): the seven overground grow arms — the
//!   plain/flower 2/52/382 (+ living-leaf 192) arm with the wall/remix flower
//!   pick, then the jungle 60/62/384/226, mushroom 70/528, hallow 109/115,
//!   corrupt 23/661/636, crimson 199/662/205 and ash 633/638 arms. Each grows
//!   the family vine one row below behind the free-below gate, the 10-row
//!   look-up through the family grass and [`grow_more_vines`]' density.
//! * [`too_many_jungle_vines_nearby`] (`WorldGen.cs:73189-73212`): the ±9×±5
//!   clamped box counting active 62, true strictly above `maxCount` (default
//!   6) — the temple-brick 226 arm's density gate.
//! * [`grow_underground_jungle_thorn`] (`WorldGen.cs:73658-73661`): the
//!   underground dispatcher's jungle-thorn arm — a 69 tile runs `GrowSpike`
//!   (69, 60) with NO roll. The port's overground 69 arm (`growth.rs:843`)
//!   sits inside `grass_pass`, which 69 is not a member of, so it never
//!   fires; the genuine arm is the underground one.
//! * [`thorn_down_merge_frame`] (`WorldGen.cs:82043-82066`): the thorn
//!   down-merge — cases 32/352/69/655 fold a matching grass neighbour BELOW
//!   into the thorn (`down = num`) before the generic block frameset runs.
//!   These are direct `down = num` assignments, NOT `TileMergeAttempt` calls,
//!   so `tools/GenTileData`'s merge parser never emits them and `framing.rs`
//!   lacks them. The helper computes the thorn's frame with the merge applied.
//!
//! WIRED (W4-g83 wire phase 2, `docs/research/123-vine-wire-contract.md`
//! §3/§4): `grow_overground_vines` rides the END of
//! `growth.rs::update_world_overground_tile` (after the `grass_pass` block —
//! the C# `:72504` vine chain follows the grass/moss arms), the jungle-thorn
//! arm sits right after `grow_vine_arm` in `update_world_underground_tile`
//! (the dead overground-69 arm inside `grass_pass` is DELETED in the same
//! step), and the thorn down-merge is the 4-line `frame_of_variant` insertion
//! after the own-slope match. Both growth fns take the caller's ACTIVE
//! `genRand` stream — the growth phase's `UpdateWorldGen` named stream, no
//! new stream, no reseed. `thorn_down_merge_frame` stays as the witness
//! fixture (contract §4 keep-one note: the witnesses assert frames).
//!
//! Deviations: the `Main.netMode == 2` `NetMessage.SendTileSquare` sides are
//! MP scope; `SquareTileFrame` rides [`crate::edit::square_tile_frame_default`]
//! on the threaded stream; `CopyPaintAndCoating` copies `meta` + the extras
//! byte (the `grow_spike` convention, `growth.rs:678-684`). The `num24` budget
//! in the plain arm (`:72508-72512`) is dead in the C# (never read) and is not
//! carried. The port-only `frame_important[vine]` write is dropped at wiring
//! (contract §2 — C# never marks the 8 vine kinds frame-important).

use crate::tiledata as td;
use crate::unified_random::UnifiedRandom;
use crate::world::World;

/// `TileID.Sets.IsVine` (`TileID.cs:531`):
/// `CreateBoolSet(52, 382, 62, 115, 205, 528, 636, 638)`.
const IS_VINE: [u16; 8] = [52, 382, 62, 115, 205, 528, 636, 638];

/// `WorldGen.InWorld(x, y, fluff)` (the `int` overload).
#[inline]
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `Collision.CanHitLine`'s tile-solid LOS walk — the port's `growth.rs`
/// `can_hit_line` (`:451-483`) verbatim: a Bresenham line between the two
/// 1×1 px boxes, blocked by any active solid (non-solidTop) cell.
fn can_hit_line(world: &World, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
    let steep = (y2 - y1).abs() > (x2 - x1).abs();
    let (mut ax, mut ay, mut bx, mut by) = (x1, y1, x2, y2);
    if steep {
        std::mem::swap(&mut ax, &mut ay);
        std::mem::swap(&mut bx, &mut by);
    }
    let dx = (bx - ax).abs();
    let dy = (by - ay).abs();
    let sx = if ax < bx { 1 } else { -1 };
    let sy = if ay < by { 1 } else { -1 };
    let mut err = dx / 2;
    let mut i = ax;
    loop {
        let (px, py) = if steep { (ay, i) } else { (i, ay) };
        if world.tile(px, py).is_some_and(|t| {
            t.active()
                && (t.kind as usize) < td::TILE_COUNT
                && td::TILE_SOLID.has(t.kind)
                && !td::TILE_SOLID_TOP.has(t.kind)
        }) {
            return false;
        }
        if i == bx {
            break;
        }
        err -= dy;
        if err < 0 {
            ay += sy;
            err += dx;
        }
        i += sx;
    }
    true
}

/// `WorldGen.GrowMoreVines` (`WorldGen.cs:45586-45621`): the vine-density cap
/// around `(x, y)` — the ±4/±6/+10 box's active-vine count, the LOS-weighted
/// accumulation below the source (`(j-y)*2`, or `*20` for the mushroom vine
/// 528) and the 60 budget (÷5 → 12 when the SOURCE is 528). The port's
/// `growth.rs:494` carries the same; re-ported here so the module is
/// self-contained off the contested `growth.rs`.
fn grow_more_vines(world: &World, x: i32, y: i32) -> bool {
    // `if (!InWorld(x, y, 30)) return false;` (`:45588-45591`).
    if !in_world(world, x, y, 30) {
        return false;
    }
    let (lo_x, lo_y, hi_y) = (4, 6, 10);
    let mut budget = 60;
    if world.tile(x, y).is_some_and(|t| t.kind == 528) {
        budget /= 5;
    }
    let mut count = 0;
    for i in (x - lo_x)..=(x + lo_x) {
        for j in (y - lo_y)..=(y + hi_y) {
            if world
                .tile(i, j)
                .is_some_and(|t| t.active() && IS_VINE.contains(&t.kind))
            {
                count += 1;
                if j > y && can_hit_line(world, x, y, i, j) {
                    let weight = if world.tile(i, j).is_some_and(|t| t.kind == 528) {
                        20
                    } else {
                        2
                    };
                    count += (j - y) * weight;
                }
                if count > budget {
                    return false;
                }
            }
        }
    }
    true
}

/// `WorldGen.TooManyJungleVinesNearby` (`WorldGen.cs:73189-73212`): the ±9×±5
/// box clamped into `[10, maxTiles-1-10]`, counting active 62, true strictly
/// above `maxCount` (the C# default 6).
pub fn too_many_jungle_vines_nearby(world: &World, i: i32, j: i32, max_count: i32) -> bool {
    // `Utils.Clamp` tests the max first (`wind.rs::cs_clamp` convention); the
    // ranges here are well-ordered so a plain clamp matches.
    let clamp = |v: i32, lo: i32, hi: i32| v.max(lo).min(hi);
    let x0 = clamp(i - 9, 10, world.width - 1 - 10);
    let x1 = clamp(i + 9, 10, world.width - 1 - 10);
    let y0 = clamp(j - 5, 10, world.height - 1 - 10);
    let y1 = clamp(j + 5, 10, world.height - 1 - 10);
    let mut count = 0;
    for k in x0..=x1 {
        for l in y0..=y1 {
            if world.tile(k, l).is_some_and(|t| t.active() && t.kind == 62) {
                count += 1;
                if count > max_count {
                    return true;
                }
            }
        }
    }
    false
}

/// `Tile.anyLava()` (`Tile.cs`): liquid > 0 AND the lava flag.
#[inline]
fn any_lava(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Lava)
}

/// The free-below gate shared by every grow arm: the cell below is inactive
/// and has no lava (`!Main.tile[i, j+1].active() && !anyLava()`).
#[inline]
fn below_clear(world: &World, i: i32, j: i32) -> bool {
    world
        .tile(i, j + 1)
        .is_some_and(|t| !t.active())
        && !any_lava(world, i, j + 1)
}

/// The 10-row look-up through the family grass (`for (num = j; num > j-10;
/// num--)`): a bottom slope aborts (false); the first active `grass` cell
/// (no bottom slope) accepts (true). `grass2` is an optional second accepted
/// kind (the corrupt/crimson arms accept both the grass and its moss-stone).
fn lookup_grass(world: &World, i: i32, j: i32, grass: u16, grass2: Option<u16>) -> bool {
    let mut n = j;
    while n > j - 10 {
        let Some(t) = world.tile(i, n) else {
            n -= 1;
            continue;
        };
        if t.bottom_slope() {
            return false;
        }
        if t.active()
            && !t.bottom_slope()
            && (t.kind == grass || grass2.is_some_and(|g| t.kind == g))
        {
            return true;
        }
        n -= 1;
    }
    false
}

/// Plant the family vine one row below the source: `type = vine`,
/// `active(true)`, `CopyPaintAndCoating(source)`, `SquareTileFrame(i, j+1)`
/// (the C# arms' shared write, e.g. `:72551-72555`). The paint/coating copy
/// rides `meta` + the extras byte (the `grow_spike` convention).
///
/// `rng` is the growth tick's ACTIVE `genRand` stream (contract §2/§3): the
/// write's `SquareTileFrame` runs the vine/seaweed checks on the SAME stream
/// the caller handed `update_world_*`. The port-only
/// `frame_important[vine] = true` write is DROPPED at wiring (contract §2 —
/// no C# `Main.tileFrameImportant` write exists for the 8 vine kinds; the
/// flag mis-routed vines away from `reframe_area`'s cosmetic pass).
fn plant_vine(
    world: &mut World,
    i: i32,
    j: i32,
    vine: u16,
    paint_src_y: i32,
    rng: &mut UnifiedRandom,
) {
    let dy = j + 1;
    let (src_meta, src_extras) = {
        let si = world.index(i, paint_src_y);
        (world.tiles[si].meta, world.extras_at(si))
    };
    if let Some(t) = world.tile_mut(i, dy) {
        t.kind = vine;
        t.flags |= crate::tile::FLAG_ACTIVE;
    }
    let di = world.index(i, dy);
    world.tiles[di].meta = src_meta;
    world.set_extras_at(di, src_extras);
    crate::edit::square_tile_frame_default(world, i, dy, rng);
}

/// `WorldGen.UpdateWorld_OvergroundTile`'s vine chain
/// (`WorldGen.cs:72506-72802`): the seven overground grow arms, in source
/// order. The first arm (2/52/382/192) is a standalone `if`; the lily-61
/// conversion (`:72563`) is its `else if` (NOT ported here — it is the
/// jungle-plant lane); the remaining six arms are a SECOND `if`/`else if`
/// chain (`:72575-72802`). `rng` is the C# `WorldGen.genRand`.
///
/// The C# reads the cell type fresh per arm (a prior arm may have changed
/// it); the port re-reads `world.tile(i, j)` at each chain head.
pub fn grow_overground_vines(world: &mut World, i: i32, j: i32, rng: &mut UnifiedRandom) {
    // ---- First chain: the plain/flower arm (`:72506-72562`) ----
    let kind = world.tile(i, j).map(|t| t.kind).unwrap_or(0);
    // `type == 2 || type == 52 || type == 382 || (type == 192 && Next(10)==0)`
    // (`:72506`). The 192 roll draws ONLY when the type is 192 (the `||`
    // short-circuits).
    let plain_arm = kind == 2
        || kind == 52
        || kind == 382
        || (kind == 192 && rng.next_max(10) == 0);
    if plain_arm {
        // `num24 = 60 (20 on 52/382)` (`:72508-72512`) is DEAD in the C# —
        // never read; not carried.
        // `Next(1)==0 && GrowMoreVines && !below.active() && !below.anyLava()`
        // (`:72513`). `Next(1)` always returns 0 but still draws.
        if rng.next_max(1) == 0 && grow_more_vines(world, i, j) && below_clear(world, i, j) {
            // `type3 = 52; if (type==382) type3=382; else if (type!=52) {
            // wall 68/65/66/63 → 382; below-wall → 382; remix Next(5)==0 → 382 }`
            // (`:72516-72535`).
            let mut vine: u16 = 52;
            if kind == 382 {
                vine = 382;
            } else if kind != 52 {
                let wall = world.tile(i, j).map(|t| t.wall).unwrap_or(0);
                let below_wall = world.tile(i, j + 1).map(|t| t.wall).unwrap_or(0);
                let flower_wall = |w: u16| w == 68 || w == 65 || w == 66 || w == 63;
                if flower_wall(wall) {
                    vine = 382;
                } else if flower_wall(below_wall) {
                    vine = 382;
                }
                // `if (Main.remixWorld && genRand.Next(5)==0) type3=382;`
                // (`:72531-72534`) — the roll draws only in a remix world.
                if world.seeds.contains(crate::seeds::SeedFlags::REMIX) && rng.next_max(5) == 0 {
                    vine = 382;
                }
            }
            // The 10-row look-up through dirt 2 (`:72536-72548`).
            if lookup_grass(world, i, j, 2, None) {
                // `CopyPaintAndCoating(Main.tile[i, j])` (`:72554`) — the
                // SOURCE cell, not the cell above.
                plant_vine(world, i, j, vine, j, rng);
            }
        }
    }
    // NOTE: the lily-61 `else if` (`:72563-72574`) is the jungle-plant lane
    // (ported in `growth.rs`); it is not a vine arm and stays out.

    // ---- Second chain: the six family arms (`:72575-72802`) ----
    // Re-read the type (the first chain does not change the source cell).
    let kind = world.tile(i, j).map(|t| t.kind).unwrap_or(0);
    // Jungle: 60/62/384/226 (`:72575-72617`).
    if (kind == 60 || kind == 62 || kind == 384 || kind == 226) && grow_more_vines(world, i, j) {
        let mut flag = true;
        let mut odds = 30;
        if kind == 62 {
            odds = 10;
        }
        if kind == 226 {
            flag = !too_many_jungle_vines_nearby(world, i, j, 6);
            odds = 20;
        }
        if flag && rng.next_max(odds) == 0 && below_clear(world, i, j) {
            // Look-up through 60/384/226 in ONE walk (`:72591-72603`).
            if lookup_jungle_grass(world, i, j) {
                plant_vine(world, i, j, 62, j, rng);
            }
        }
    } else if (kind == 70 || kind == 528) && grow_more_vines(world, i, j) {
        // Mushroom (`:72618-72654`).
        let odds = if kind == 528 { 7 } else { 70 };
        if rng.next_max(odds) == 0 && below_clear(world, i, j) && lookup_grass(world, i, j, 70, None)
        {
            plant_vine(world, i, j, 528, j, rng);
        }
    } else if (kind == 109 || kind == 115) && grow_more_vines(world, i, j) {
        // Hallow (`:72655-72691`).
        let odds = if kind == 115 { 20 } else { 60 };
        if rng.next_max(odds) == 0 && below_clear(world, i, j) && lookup_grass(world, i, j, 109, None)
        {
            plant_vine(world, i, j, 115, j, rng);
        }
    } else if (kind == 23 || kind == 661 || kind == 636) && grow_more_vines(world, i, j) {
        // Corrupt (`:72692-72728`).
        let odds = if kind == 636 { 20 } else { 60 };
        if rng.next_max(odds) == 0
            && below_clear(world, i, j)
            && lookup_grass(world, i, j, 23, Some(661))
        {
            plant_vine(world, i, j, 636, j, rng);
        }
    } else if (kind == 199 || kind == 662 || kind == 205) && grow_more_vines(world, i, j) {
        // Crimson (`:72729-72765`).
        let odds = if kind == 205 { 20 } else { 60 };
        if rng.next_max(odds) == 0
            && below_clear(world, i, j)
            && lookup_grass(world, i, j, 199, Some(662))
        {
            plant_vine(world, i, j, 205, j, rng);
        }
    } else if (kind == 633 || kind == 638) && grow_more_vines(world, i, j) {
        // Ash (`:72766-72802`).
        let odds = if kind == 638 { 7 } else { 70 };
        if rng.next_max(odds) == 0 && below_clear(world, i, j) && lookup_grass(world, i, j, 633, None)
        {
            plant_vine(world, i, j, 638, j, rng);
        }
    }
}

/// The jungle look-up (`WorldGen.cs:72591-72603`): ONE walk up from `j`
/// accepting `type == 60 || type == 384 || type == 226` (no bottom slope);
/// a bottom slope aborts (false). The three kinds share the single walk so
/// the row order matches the C# (a 226 cell above a 60 cell accepts at the
/// 226 row, and a bottom slope anywhere aborts the whole look-up).
fn lookup_jungle_grass(world: &World, i: i32, j: i32) -> bool {
    let mut n = j;
    while n > j - 10 {
        let Some(t) = world.tile(i, n) else {
            n -= 1;
            continue;
        };
        if t.bottom_slope() {
            return false;
        }
        if t.active() && !t.bottom_slope() && (t.kind == 60 || t.kind == 384 || t.kind == 226) {
            return true;
        }
        n -= 1;
    }
    false
}

/// `WorldGen.UpdateWorld_UndergroundTile`'s jungle-thorn arm
/// (`WorldGen.cs:73658-73661`): a 69 tile runs `GrowSpike(i, j, 69, 60)` with
/// NO roll. Returns true when the arm fired (the cell was a 69).
pub fn grow_underground_jungle_thorn(
    world: &mut World,
    i: i32,
    j: i32,
    rng: &mut UnifiedRandom,
) -> bool {
    if world.tile(i, j).is_some_and(|t| t.kind == 69) {
        crate::growth::grow_spike(world, i, j, 69, 60, rng);
        true
    } else {
        false
    }
}

// (`grow_spike`'s own `SquareTileFrame` seam threads the same `rng` — the
// spike write's framing dispatch runs on the caller's active stream.)

/// `WorldGen.TileFrameCosmetic`'s thorn down-merge (`WorldGen.cs:82043-82066`):
/// cases 32/352/69/655 fold a matching grass neighbour BELOW into the thorn
/// (`down = num`) before the generic block frameset. Returns the thorn's
/// `(frameX, frameY)` for the cell at `(x, y)` with the merge applied, or
/// `None` when the cell is not an active thorn.
///
/// The merge pairs: 32↔23 (`:82043-82048`), 352↔199 (`:82049-82054`),
/// 69↔60 (`:82055-82060`), 655↔60 (`:82061-82066`). The down slot becomes
/// the thorn's own type when it holds the paired grass; the other seven
/// slots keep their framing types. The generic frameset (`framing.rs::RULES`)
/// then resolves the mask. `variant` is the cell's stored `frameNumber`.
pub fn thorn_down_merge_frame(world: &World, x: i32, y: i32, variant: usize) -> Option<(i16, i16)> {
    let tile = world.tile(x, y)?;
    if !tile.active() {
        return None;
    }
    let me = tile.kind;
    let grass_below: u16 = match me {
        32 => 23,
        352 => 199,
        69 | 655 => 60,
        _ => return None,
    };
    let num = me as i32;
    // The eight neighbour slots, mirroring `framing.rs::frame_of_variant`'s
    // active-type reads (the thorn kinds are not stone/aliases, so the
    // framing type is the kind itself).
    let slot = |dx: i32, dy: i32| -> i32 {
        world
            .tile(x + dx, y + dy)
            .map(|t| if t.active() { t.kind as i32 } else { -1 })
            .unwrap_or(-1)
    };
    let mut up = slot(0, -1);
    let mut down = slot(0, 1);
    let mut left = slot(-1, 0);
    let mut right = slot(1, 0);
    let ul = slot(-1, -1);
    let ur = slot(1, -1);
    let dl = slot(-1, 1);
    let dr = slot(1, 1);
    // Sloped neighbours hide their open side (`framing.rs:840-859`,
    // `WorldGen.cs:81440-81507`); the thorn's own slope hides two sides
    // (`:81493-81507`). These run BEFORE the thorn down-merge in the C#
    // (`:81632-81700` precedes `:82043-82066`).
    if let Some(t) = world.tile(x - 1, y) {
        if t.slope() == 1 || t.slope() == 3 {
            left = -1;
        }
    }
    if let Some(t) = world.tile(x + 1, y) {
        if t.slope() == 2 || t.slope() == 4 {
            right = -1;
        }
    }
    if let Some(t) = world.tile(x, y - 1) {
        if t.slope() == 3 || t.slope() == 4 {
            up = -1;
        }
    }
    if let Some(t) = world.tile(x, y + 1) {
        if t.slope() == 1 || t.slope() == 2 {
            down = -1;
        }
    }
    match tile.slope() {
        1 => {
            up = -1;
            right = -1;
        }
        2 => {
            up = -1;
            left = -1;
        }
        3 => {
            down = -1;
            right = -1;
        }
        4 => {
            down = -1;
            left = -1;
        }
        _ => {}
    }
    // The thorn down-merge (`:82043-82066`): `if (down == grass) down = num;`
    // — AFTER the slope adjustments, so a sloped grass-below cell (down == -1)
    // does not merge.
    if down == grass_below as i32 {
        down = num;
    }
    // The generic block frameset mask (`framing.rs:1009-1042`), in RULES
    // order. The bits match `framing.rs`'s UP/DOWN/LEFT/RIGHT/corners.
    const UP: u8 = 1;
    const DOWN: u8 = 2;
    const LEFT: u8 = 4;
    const RIGHT: u8 = 8;
    const UP_LEFT: u8 = 16;
    const UP_RIGHT: u8 = 32;
    const DOWN_LEFT: u8 = 64;
    const DOWN_RIGHT: u8 = 128;
    let mut mask = 0u8;
    for (v, bit) in [
        (up, UP),
        (down, DOWN),
        (left, LEFT),
        (right, RIGHT),
        (ul, UP_LEFT),
        (ur, UP_RIGHT),
        (dl, DOWN_LEFT),
        (dr, DOWN_RIGHT),
    ] {
        if v == num {
            mask |= bit;
        }
    }
    let variant = variant.min(2);
    Some(resolve_block_frame(mask, variant))
}

/// The generic block frameset (`framing.rs::RULES`, `WorldGen.cs:84403-84765`),
/// first-match-wins. Re-ported here (the `RULES` table is private to the
/// contested `framing.rs`); the frames sit on an 18 px grid.
fn resolve_block_frame(mask: u8, variant: usize) -> (i16, i16) {
    const UP: u8 = 1;
    const DOWN: u8 = 2;
    const LEFT: u8 = 4;
    const RIGHT: u8 = 8;
    const UL: u8 = 16;
    const UR: u8 = 32;
    const DL: u8 = 64;
    const DR: u8 = 128;
    // (need, want, frames[3]) — the `framing.rs::RULES` rows verbatim.
    let rules: [(u8, u8, [(i16, i16); 3]); 20] = [
        (UP | DOWN | LEFT | RIGHT | UL | UR, UP | DOWN | LEFT | RIGHT, [(108, 18), (126, 18), (144, 18)]),
        (UP | DOWN | LEFT | RIGHT | DL | DR, UP | DOWN | LEFT | RIGHT, [(108, 36), (126, 36), (144, 36)]),
        (UP | DOWN | LEFT | RIGHT | UL | DL, UP | DOWN | LEFT | RIGHT, [(180, 0), (180, 18), (180, 36)]),
        (UP | DOWN | LEFT | RIGHT | UR | DR, UP | DOWN | LEFT | RIGHT, [(198, 0), (198, 18), (198, 36)]),
        (UP | DOWN | LEFT | RIGHT, UP | DOWN | LEFT | RIGHT, [(18, 18), (36, 18), (54, 18)]),
        (DOWN | LEFT | RIGHT, DOWN | LEFT | RIGHT, [(18, 0), (36, 0), (54, 0)]),
        (UP | LEFT | RIGHT, UP | LEFT | RIGHT, [(18, 36), (36, 36), (54, 36)]),
        (UP | DOWN | RIGHT, UP | DOWN | RIGHT, [(0, 0), (0, 18), (0, 36)]),
        (UP | DOWN | LEFT, UP | DOWN | LEFT, [(72, 0), (72, 18), (72, 36)]),
        (DOWN | RIGHT, DOWN | RIGHT, [(0, 54), (36, 54), (72, 54)]),
        (DOWN | LEFT, DOWN | LEFT, [(18, 54), (54, 54), (90, 54)]),
        (UP | RIGHT, UP | RIGHT, [(0, 72), (36, 72), (72, 72)]),
        (UP | LEFT, UP | LEFT, [(18, 72), (54, 72), (90, 72)]),
        (LEFT | RIGHT, LEFT | RIGHT, [(108, 72), (126, 72), (144, 72)]),
        (UP | DOWN, UP | DOWN, [(90, 0), (90, 18), (90, 36)]),
        (UP, UP, [(108, 54), (126, 54), (144, 54)]),
        (DOWN, DOWN, [(108, 0), (126, 0), (144, 0)]),
        (RIGHT, RIGHT, [(162, 0), (162, 18), (162, 36)]),
        (LEFT, LEFT, [(216, 0), (216, 18), (216, 36)]),
        (0, 0, [(162, 54), (180, 54), (198, 54)]),
    ];
    for (need, want, frames) in rules {
        if mask & need == want {
            return frames[variant];
        }
    }
    ([18, 36, 54][variant], 18)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{Tile, FLAG_ACTIVE};

    fn world() -> World {
        World::new("t".into(), 200, 200)
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    // ---- TooManyJungleVinesNearby ----

    // The ±9×±5 box counts active 62; six is the cap (not too many), seven is.
    #[test]
    fn w4_g83_too_many_jungle_vines_counts_the_box() {
        let mut w = world();
        // Six 62 cells inside the box around (100, 100).
        for (dx, dy) in [(-2, 0), (-1, 0), (0, 0), (1, 0), (2, 0), (3, 0)] {
            put(&mut w, 100 + dx, 100 + dy, 62);
        }
        assert!(!too_many_jungle_vines_nearby(&w, 100, 100, 6), "six == cap");
        put(&mut w, 104, 100, 62); // seventh
        assert!(too_many_jungle_vines_nearby(&w, 100, 100, 6), "seven > cap");
        // A 62 outside the box (±10 columns) does not count.
        let mut w2 = world();
        for dy in 0..7 {
            put(&mut w2, 110, 100 + dy, 62);
        }
        assert!(!too_many_jungle_vines_nearby(&w2, 100, 100, 6));
    }

    // The clamp: an anchor at the world edge still scans a valid box.
    #[test]
    fn w4_g83_too_many_jungle_vines_clamps_the_edge() {
        let w = world();
        assert!(!too_many_jungle_vines_nearby(&w, 0, 0, 6));
        assert!(!too_many_jungle_vines_nearby(&w, 199, 199, 6));
    }

    // ---- GrowMoreVines density ----

    // An empty interior passes; the mushroom source's budget is 12 not 60.
    #[test]
    fn w4_g83_grow_more_vines_the_mushroom_budget() {
        let mut w = world();
        assert!(grow_more_vines(&w, 100, 100), "empty box passes");
        // A 528 source divides the budget by 5. Fill the box with 13 vine
        // cells (no LOS weight, all at/above the source row): > 12 fails.
        put(&mut w, 100, 100, 528);
        let mut placed = 0;
        for dx in -4..=4 {
            for dy in -6..=0 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                if placed >= 13 {
                    break;
                }
                put(&mut w, 100 + dx, 100 + dy, 52);
                placed += 1;
            }
        }
        assert!(!grow_more_vines(&w, 100, 100), "13 > 12 mushroom budget");
        // A plain 52 source keeps the 60 budget: the same 13 cells pass.
        put(&mut w, 100, 100, 52);
        assert!(grow_more_vines(&w, 100, 100), "13 < 60 plain budget");
    }

    // The 30-tile border guard refuses an edge anchor.
    #[test]
    fn w4_g83_grow_more_vines_the_border_guard() {
        let w = world();
        assert!(!grow_more_vines(&w, 10, 100));
        assert!(!grow_more_vines(&w, 100, 10));
    }

    // ---- The overground grow arms ----

    // The plain arm: dirt 2 with a free cell below grows a 52 vine (the
    // Next(1) always fires, the look-up finds the dirt at the source row).
    #[test]
    fn w4_g83_the_plain_arm_grows_a_vine_below_dirt() {
        let mut w = world();
        put(&mut w, 100, 100, 2); // dirt
        let mut grew = 0;
        for seed in 1u64..200 {
            let mut w = w.clone();
            let mut rng = UnifiedRandom::new(seed as i32);
            grow_overground_vines(&mut w, 100, 100, &mut rng);
            if w.tile(100, 101).is_some_and(|t| t.active() && t.kind == 52) {
                grew += 1;
            }
        }
        assert!(grew > 0, "the plain arm grows 52 ({grew}/199)");
    }

    // The flower-wall pick: dirt with wall 65 grows a 382 flower vine.
    #[test]
    fn w4_g83_the_flower_wall_pick_grows_382() {
        let mut w = world();
        put(&mut w, 100, 100, 2);
        w.tiles[w.index(100, 100)].wall = 65; // living-wood wall
        let mut grew382 = 0;
        for seed in 1u64..200 {
            let mut w = w.clone();
            let mut rng = UnifiedRandom::new(seed as i32);
            grow_overground_vines(&mut w, 100, 100, &mut rng);
            if w.tile(100, 101).is_some_and(|t| t.active() && t.kind == 382) {
                grew382 += 1;
            }
        }
        assert!(grew382 > 0, "wall 65 picks the flower 382 ({grew382}/199)");
    }

    // A 52 vine source keeps the 52 type (no flower pick) and grows below.
    #[test]
    fn w4_g83_a_vine_source_grows_the_same_family() {
        let mut w = world();
        put(&mut w, 100, 99, 2); // dirt support above
        put(&mut w, 100, 100, 52); // the vine source
        let mut grew = 0;
        for seed in 1u64..200 {
            let mut w = w.clone();
            let mut rng = UnifiedRandom::new(seed as i32);
            grow_overground_vines(&mut w, 100, 100, &mut rng);
            if w.tile(100, 101).is_some_and(|t| t.active() && t.kind == 52) {
                grew += 1;
            }
        }
        assert!(grew > 0, "a 52 source grows 52 ({grew}/199)");
    }

    // The mushroom arm: 70 grass grows 528 at the 1/70 odds.
    #[test]
    fn w4_g83_the_mushroom_arm_grows_528() {
        let mut w = world();
        put(&mut w, 100, 100, 70);
        let mut grew = 0;
        for seed in 1u64..2000 {
            let mut w = w.clone();
            let mut rng = UnifiedRandom::new(seed as i32);
            grow_overground_vines(&mut w, 100, 100, &mut rng);
            if w.tile(100, 101).is_some_and(|t| t.active() && t.kind == 528) {
                grew += 1;
            }
        }
        assert!(grew > 0, "the mushroom arm grows 528 ({grew}/1999)");
    }

    // The jungle arm: 60 grass grows 62 at the 1/30 odds.
    #[test]
    fn w4_g83_the_jungle_arm_grows_62() {
        let mut w = world();
        put(&mut w, 100, 100, 60);
        let mut grew = 0;
        for seed in 1u64..2000 {
            let mut w = w.clone();
            let mut rng = UnifiedRandom::new(seed as i32);
            grow_overground_vines(&mut w, 100, 100, &mut rng);
            if w.tile(100, 101).is_some_and(|t| t.active() && t.kind == 62) {
                grew += 1;
            }
        }
        assert!(grew > 0, "the jungle arm grows 62 ({grew}/1999)");
    }

    // The free-below gate: an active cell below blocks every arm.
    #[test]
    fn w4_g83_an_occupied_cell_below_blocks_growth() {
        let mut w = world();
        put(&mut w, 100, 100, 2);
        put(&mut w, 100, 101, 1); // stone below
        for seed in 1u64..100 {
            let mut w = w.clone();
            let mut rng = UnifiedRandom::new(seed as i32);
            grow_overground_vines(&mut w, 100, 100, &mut rng);
            assert_eq!(w.tile(100, 101).unwrap().kind, 1, "the stone stays");
        }
    }

    // ---- The underground jungle-thorn arm ----

    // A 69 tile fires GrowSpike(69, 60); a non-69 does not.
    #[test]
    fn w4_g83_the_underground_thorn_arm_fires_on_69() {
        let mut w = world();
        put(&mut w, 100, 100, 69);
        let mut rng = UnifiedRandom::new(1);
        assert!(grow_underground_jungle_thorn(&mut w, 100, 100, &mut rng));
        // A dirt cell does not fire.
        put(&mut w, 100, 100, 2);
        assert!(!grow_underground_jungle_thorn(&mut w, 100, 100, &mut rng));
    }

    // ---- The thorn down-merge frame ----

    // A lone thorn 69 (no neighbours) frames to the isolated block frame.
    #[test]
    fn w4_g83_a_lone_thorn_uses_the_isolated_frame() {
        let mut w = world();
        put(&mut w, 100, 100, 69);
        for v in 0..3 {
            let f = thorn_down_merge_frame(&w, 100, 100, v).unwrap();
            assert_eq!(f, ([(162, 54), (180, 54), (198, 54)][v]), "variant {v}");
        }
    }

    // The down-merge: a 69 thorn with jungle grass 60 BELOW folds the down
    // slot into the thorn, so the frame is the "single open side UP" row
    // (down merged) — (108,0)/(126,0)/(144,0), NOT the isolated frame.
    #[test]
    fn w4_g83_the_thorn_down_merge_folds_the_grass() {
        let mut w = world();
        put(&mut w, 100, 100, 69);
        put(&mut w, 100, 101, 60); // jungle grass below
        for v in 0..3 {
            let f = thorn_down_merge_frame(&w, 100, 100, v).unwrap();
            assert_eq!(f, ([(108, 0), (126, 0), (144, 0)][v]), "down merged, variant {v}");
        }
        // Without the merge (a stone below instead) the down slot stays open
        // → the isolated frame.
        put(&mut w, 100, 101, 1);
        let f = thorn_down_merge_frame(&w, 100, 100, 0).unwrap();
        assert_eq!(f, (162, 54), "stone below does not merge");
    }

    // The four thorn kinds each merge their paired grass: 32↔23, 352↔199,
    // 69↔60, 655↔60.
    #[test]
    fn w4_g83_the_four_thorn_merge_pairs() {
        for (thorn, grass) in [(32u16, 23u16), (352, 199), (69, 60), (655, 60)] {
            let mut w = world();
            put(&mut w, 100, 100, thorn);
            put(&mut w, 100, 101, grass);
            let f = thorn_down_merge_frame(&w, 100, 100, 0).unwrap();
            assert_eq!(f, (108, 0), "thorn {thorn} merges grass {grass}");
        }
    }

    // A non-thorn cell returns None.
    #[test]
    fn w4_g83_a_non_thorn_has_no_merge_frame() {
        let mut w = world();
        put(&mut w, 100, 100, 2);
        assert!(thorn_down_merge_frame(&w, 100, 100, 0).is_none());
    }

    // The slope adjustments run BEFORE the down-merge (`:81632-81700` precedes
    // `:82043`): a top-sloped grass-below cell has its down slot hidden (-1),
    // so it does NOT merge — the thorn keeps the isolated frame.
    #[test]
    fn w4_g83_a_sloped_grass_below_does_not_merge() {
        let mut w = world();
        put(&mut w, 100, 100, 69);
        put(&mut w, 100, 101, 60);
        w.tiles[w.index(100, 101)].set_slope(1); // top slope hides the down slot
        let f = thorn_down_merge_frame(&w, 100, 100, 0).unwrap();
        assert_eq!(f, (162, 54), "sloped grass below: down hidden, no merge");
    }
}
