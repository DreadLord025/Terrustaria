//! The biome spread — `WorldGen.hardUpdateWorld`'s infection arms
//! (`WorldGen.cs:69645-69860`) and `Convert` (`:55085-55673`).
//!
//! Source map (line numbers into `decompiled/Terraria/`):
//! - `TileID.Sets.SpreadsCorruption` = {23, 661, 25, 112, 398, 400, 163, 32,
//!   636, 24} (`TileID.cs:580`), `SpreadsCrimson` = {199, 662, 203, 234,
//!   399, 401, 200, 352, 205, 201} (`:589`)
//! - `Convert`'s tile map (`:55139-55205`): the Conversion.Stone set → the
//!   crimstone 203 / the ebonstone 25, the jungle grass → 662/23, the grass
//!   → 199/23, the ice → 200/163, the sand → 234/112, the hardened sand →
//!   399/398, the sandstone → 401/400; the walls: the stone → 192/3, the
//!   dirt → 193/69 (the natural-wall slice of `WallID.Sets.Conversion`)
//! - the spread loop (`:69696-69860`): the ±3 random target, the
//!   chlorophyte defence (`nearbyChlorophyte` `:69502+` →
//!   `ChlorophyteDefense`), the sunflower (27) suppression
//!   (`CountNearBlocksTypes(i, j, 2, 1, 27)`), the conversions with the
//!   `Next(2) == 0` re-roll chain
//! - the chlorophyte arms (`:69646-69685`): the 211/346 sources grow into
//!   the mud/the jungle grass via `CanChlorophyteGrow` (`:69464-69500` —
//!   the density gates 40 in the radius-35 box / 130 in the radius-85 box,
//!   halved above the rock layer) and the ±6 random re-scan (`Convert(...,
//!   9)`)
//!
//! Deviations (ANALYSIS "Biome spread (M5-SPREAD-2)"): the wall conversion
//! covers the natural-wall slice (1/2 → 3/69 or 192/193) — the C#'s full
//! `WallID.Sets.Conversion` tables span the brick walls too; the thorn
//! bushes (32/352) spread by the set membership but grow no bushes (the
//! plant placement = the follow-up); the post-PlantBoss rate halves
//! (`Next(2) != 0`) verbatim; the creative spread-stop rides the
//! `allowed_to_spread` input.

use crate::UnifiedRandom;
use crate::world::World;

/// `TileID.Sets.SpreadsCorruption` (`TileID.cs:580`).
pub const SPREADS_CORRUPTION: [u16; 10] = [23, 661, 25, 112, 398, 400, 163, 32, 636, 24];
/// `TileID.Sets.SpreadsCrimson` (`TileID.cs:589`).
pub const SPREADS_CRIMSON: [u16; 10] = [199, 662, 203, 234, 399, 401, 200, 352, 205, 201];

/// `Convert`'s tile map (`:55139-55205`): the source kind → the converted
/// kind per the evil flavour.
pub(crate) fn convert_tile(kind: u16, corruption: bool) -> Option<u16> {
    Some(match (kind, corruption) {
        // The grass family (the Conversion.Grass set: 2, 477, 60, 69...):
        // the corrupt grass 23 / the crimson grass 199.
        (2, c) => {
            if c {
                23
            } else {
                199
            }
        }
        // The jungle grass (the Conversion.JungleGrass set): 23/662.
        (60, c) => {
            if c {
                23
            } else {
                662
            }
        }
        // The stone + the moss family (the Conversion.Stone set): the
        // ebonstone 25 / the crimstone 203.
        (1, c) => {
            if c {
                25
            } else {
                203
            }
        }
        // The ice (the Conversion.Ice set): the purple 163 / the red 200.
        (161, c) => {
            if c {
                163
            } else {
                200
            }
        }
        // The sand: the ebonsand 112 / the crimsand 234.
        (53, c) => {
            if c {
                112
            } else {
                234
            }
        }
        // The hardened sand: 398/399.
        (397, c) => {
            if c {
                398
            } else {
                399
            }
        }
        // The sandstone: 400/401.
        (396, c) => {
            if c {
                400
            } else {
                401
            }
        }
        _ => return None,
    })
}

/// The natural-wall slice of `WallID.Sets.Conversion` (`:55090-55135`): the
/// stone wall 1 → 192/3, the dirt wall 2 → 193/69.
/// M5-SPREAD-2c: the natural-wall slice is the C#'s full wall chains, so the
/// fn routes through `growth::convert_wall_by_case` (every conversion type's
/// `Convert_ActuallyConvertWall` ladder, `WorldGen.cs:55100+`). The bool
/// keeps the two evil types (1 corruption / 4 crimson) the evil callers use.
pub(crate) fn convert_wall(wall: u16, corruption: bool) -> Option<u16> {
    crate::growth::convert_wall_by_case(wall, if corruption { 1 } else { 4 }, 0, 0.0, None)
}

/// `Convert`'s single-tile overload (`:55085-55673`). `chlorophyte` = the
/// conversion types 8/9 (`:55665-55673`: the mud 59 and the jungle grass 60
/// become the chlorophyte 211).
pub fn convert_cell(world: &mut World, x: i32, y: i32, corruption: bool, chlorophyte: bool) {
    let (kind, wall) = match world.tile(x, y) {
        Some(t) => (t.kind, t.wall),
        None => return,
    };
    if !world.tile(x, y).is_some_and(|t| t.active()) && wall == 0 {
        return;
    }
    if chlorophyte {
        if matches!(kind, 59 | 60) {
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = 211;
            }
        }
        return;
    }
    if let Some(new_kind) = convert_tile(kind, corruption) {
        if let Some(t) = world.tile_mut(x, y) {
            t.kind = new_kind;
            t.frame_x = 0;
            t.frame_y = 0;
        }
    }
    if let Some(new_wall) = convert_wall(wall, corruption) {
        if let Some(t) = world.tile_mut(x, y) {
            t.wall = new_wall;
        }
    }
}

/// `CanChlorophyteGrow` (`:69464-69500`): the density gates — the
/// chlorophyte count in the radius-35 box ≤ 40 (the box ×1.5 and the count
/// /2 above the rock layer) and ≤ 130 in the radius-85 box.
/// The three conversion flavors the conversion tables key on — the C#'s
/// `WorldGen.Convert`'s conversion types (1 = corruption, 4 = crimson,
/// 2 = hallow).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConversionFlavor {
    Corruption,
    Crimson,
    Hallow,
}

/// `WorldGen.Convert(i, j, conversionType, tiles: true, walls: false)`'s
/// single-cell arm (`:55085-55140`): the flavor's tile/wall maps over the
/// cell. The SpreadGrass arms call it with `walls: false`.
pub fn convert_cell_flavor(world: &mut World, x: i32, y: i32, flavor: ConversionFlavor, walls: bool) {
    let (kind, wall) = match world.tile(x, y) {
        Some(t) => (t.kind, t.wall),
        None => return,
    };
    if !world.tile(x, y).is_some_and(|t| t.active()) && wall == 0 {
        return;
    }
    match flavor {
        ConversionFlavor::Corruption | ConversionFlavor::Crimson => {
            let corruption = flavor == ConversionFlavor::Corruption;
            if let Some(new_kind) = convert_tile(kind, corruption) {
                if let Some(t) = world.tile_mut(x, y) {
                    t.kind = new_kind;
                    t.frame_x = 0;
                    t.frame_y = 0;
                }
            }
            if walls {
                if let Some(new_wall) = convert_wall(wall, corruption) {
                    if let Some(t) = world.tile_mut(x, y) {
                        t.wall = new_wall;
                    }
                }
            }
        }
        ConversionFlavor::Hallow => {
            if let Some(new_kind) = convert_tile_hallow(kind) {
                if let Some(t) = world.tile_mut(x, y) {
                    t.kind = new_kind;
                    t.frame_x = 0;
                    t.frame_y = 0;
                }
            }
            if walls {
                if let Some(new_wall) = convert_wall_hallow(wall) {
                    if let Some(t) = world.tile_mut(x, y) {
                        t.wall = new_wall;
                    }
                }
            }
        }
    }
}

/// `CanChlorophyteGrow` (`:69464-69515`): the density gates — the
/// chlorophyte count in the ±35 box ≤ 40 and in the ±85 box ≤ 130. The
/// remix world widens both (`+5`, `×15`, `−4`, `−8`, `:69470-69476`);
/// otherwise a cell ABOVE the rock layer halves BOTH caps and widens BOTH
/// boxes ×1.5 (`:69477-69483`). The scans are half-open `[i−r, i+r)` with
/// the `InWorld` guard.
pub fn can_chlorophyte_grow(world: &World, x: i32, y: i32) -> bool {
    let (mut cap_small, mut cap_big, mut r_small, mut r_big) = (40i32, 130i32, 35i32, 85i32);
    if world.seeds.contains(crate::seeds::SeedFlags::REMIX) {
        cap_small += 5;
        cap_big *= 15;
        r_small -= 4;
        r_big -= 8;
    } else if (y as f64) < world.rock_layer {
        cap_small /= 2;
        cap_big /= 2;
        r_small = (r_small as f64 * 1.5) as i32;
        r_big = (r_big as f64 * 1.5) as i32;
    }
    let count = |r: i32| -> i32 {
        let mut n = 0;
        for k in (x - r)..(x + r) {
            for l in (y - r)..(y + r) {
                if in_world(world, k, l, 0)
                    && world
                        .tile(k, l)
                        .is_some_and(|t| t.active() && t.kind == 211)
                {
                    n += 1;
                }
            }
        }
        n
    };
    if count(r_small) > cap_small {
        return false;
    }
    count(r_big) <= cap_big
}

/// `nearbyChlorophyte` (`:69517-69558`): the defence radius — the ±5 box
/// counts the 211 chlorophyte AND the 346 chlorophyte jungle grass; ≥3
/// always defends, 1-2 roll `Next(-1, 4) < count` (the remix world
/// `Next(-1, 3)`). The 10-tile border margin returns false without drawing.
fn nearby_chlorophyte(world: &World, x: i32, y: i32, rng: &mut UnifiedRandom) -> bool {
    let num2 = 5;
    if x <= num2 + 5 || x >= world.width - num2 - 5 {
        return false;
    }
    if y <= num2 + 5 || y >= world.height - num2 - 5 {
        return false;
    }
    let mut num = 0.0f64;
    for k in (x - num2)..=(x + num2) {
        for l in (y - num2)..=(y + num2) {
            if world
                .tile(k, l)
                .is_some_and(|t| t.active() && matches!(t.kind, 211 | 346))
            {
                num += 1.0;
                if num >= 3.0 {
                    return true;
                }
            }
        }
    }
    if num > 0.0 {
        if world.seeds.contains(crate::seeds::SeedFlags::REMIX) {
            if (rng.next_range(-1, 3) as f64) < num {
                return true;
            }
        } else if (rng.next_range(-1, 4) as f64) < num {
            return true;
        }
    }
    false
}

/// `ChlorophyteDefense` (`:69456-69462`): the remix worlds run the type-10
/// purge conversion on the defended cell; the normal worlds do nothing.
fn chlorophyte_defense(world: &mut World, x: i32, y: i32) {
    if world.seeds.contains(crate::seeds::SeedFlags::REMIX) {
        convert_purge_cell(world, x, y, 10);
    }
}

/// `CountNearBlocksTypes(i, j, radius, cap, tiletypes)` (`:69560-69598`):
/// the clamped-box count of the active matching tiles; a positive cap
/// returns early once the count reaches it.
fn count_near_blocks_types(world: &World, i: i32, j: i32, radius: i32, cap: i32, kinds: &[u16]) -> i32 {
    if kinds.is_empty() {
        return 0;
    }
    let x0 = (i - radius).clamp(0, world.width - 1);
    let x1 = (i + radius).clamp(0, world.width - 1);
    let y0 = (j - radius).clamp(0, world.height - 1);
    let y1 = (j + radius).clamp(0, world.height - 1);
    let mut num = 0;
    for k in x0..=x1 {
        for l in y0..=y1 {
            let Some(t) = world.tile(k, l) else {
                continue;
            };
            if !t.active() {
                continue;
            }
            if kinds.contains(&t.kind) {
                num += 1;
                if cap <= 0 || num < cap {
                    continue;
                }
                return num;
            }
        }
    }
    num
}

/// `WorldGen.hardUpdateWorld` (`WorldGen.cs:69600-69955`), verbatim: the
/// `hardModeWorldUpdates` gate (`:69602` — the caller passes the flag the
/// `UpdateWorld` head derives, `:71501`), the actuated-cell skip, the
/// crystal-shard arm, the chlorophyte arms (the 60-grass seeding
/// `:69652-69660`, the 211/346 growth + the single type-9 purge pick
/// `:69661-69701`), then the three infection loops (corruption `:69706`,
/// crimson `:69795`, hallow `:69884`) with the `continue`-retry semantics
/// (an out-of-world pick or a sunflower-blocked pick re-draws; a defended
/// cell, an unconvertible target or a failed `Next(2)` re-roll ends the
/// loop).
///
/// Deviations: the C#'s `Main.rand` purge-pick draws (`:69693-69694`) ride
/// the same stream as `genRand` under the SwapRandom (the port's single
/// per-phase stream, the `update_growth` convention); the shared
/// `convert_cell_flavor` approximation parks the type-2 hallow Convert's
/// torch/golf/mud-adjacency arms (the `micro_surface.rs` precedent) and the
/// shared `convert_tile` map has no moss/477/69 rows — the loops' draws
/// stay verbatim, those kinds convert as no-ops until the shared map grows
/// the rows.
pub fn hard_update_world(
    world: &mut World,
    x: i32,
    y: i32,
    downed_plant_boss: bool,
    allowed_to_spread: bool,
    rng: &mut UnifiedRandom,
) {
    hard_update_world_gated(world, x, y, true, downed_plant_boss, allowed_to_spread, rng);
}

/// The gated form — `hard_mode_world_updates` mirrors
/// `WorldGen.hardModeWorldUpdates` (`:71501`: `hardMode || (remix &&
/// getGood && !anniversary)`); the `UpdateWorld` callers pass the derived
/// flag, the direct callers (the tests, the legacy seams) pass true.
pub fn hard_update_world_gated(
    world: &mut World,
    x: i32,
    y: i32,
    hard_mode_world_updates: bool,
    downed_plant_boss: bool,
    allowed_to_spread: bool,
    rng: &mut UnifiedRandom,
) {
    // `:69602`: the gate + the actuated-cell skip.
    if !hard_mode_world_updates {
        return;
    }
    let Some(t) = world.tile(x, y) else {
        return;
    };
    if t.inactive() {
        return;
    }
    let kind = t.kind;
    let remix = world.seeds.contains(crate::seeds::SeedFlags::REMIX);
    // `:69607-69649`: the crystal-shard arm runs FIRST. The full condition
    // (the shard-stone kind, the rock-layer/remix depth, the `Next(5)`
    // roll) short-circuits here exactly like the C#'s `if`; the placement
    // body (`:69609-69648`) lives in `grow_crystal_shards`.
    if kind > 0
        && crate::growth::CAN_GROW_CRYSTAL_SHARDS.contains(&kind)
        && ((y as f64) > world.rock_layer || remix)
        && rng.next_max(5) == 0
    {
        crate::growth::grow_crystal_shards(world, x, y, rng);
    }
    // `:69650-69701`: the chlorophyte arms below the surface/rock midpoint
    // (or anywhere in the remix world).
    if (y as f64) > (world.world_surface + world.rock_layer) / 2.0 || remix {
        // The 60-grass seeding arm (`:69652-69660`): the 1/300 roll, the
        // ±10 target, the mud cell with a free (or non-tree) top, gated by
        // `CanChlorophyteGrow` — `Convert(…, 8)` grows the chlorophyte 211.
        if kind == 60 && rng.next_max(300) == 0 {
            let nx = x + rng.next_range(-10, 11);
            let ny = y + rng.next_range(-10, 11);
            let top_ok = !world.tile(nx, ny - 1).is_some_and(|t| {
                t.active() && matches!(t.kind, 5 | 236 | 702 | 238)
            });
            if in_world(world, nx, ny, 2)
                && world.tile(nx, ny).is_some_and(|t| t.active() && t.kind == 59)
                && top_ok
                && can_chlorophyte_grow(world, nx, ny)
            {
                convert_cell(world, nx, ny, false, true);
            }
        }
        // The 211/346 source arms (`:69661-69701`).
        if matches!(kind, 211 | 346) {
            // `Next(3) != 0`: the 4-dir neighbour growth into the mud/the
            // jungle grass, gated by `CanChlorophyteGrow`.
            if rng.next_max(3) != 0 {
                let (dx, dy) = match rng.next_max(4) {
                    0 => (1, 0),
                    1 => (-1, 0),
                    2 => (0, 1),
                    _ => (0, -1),
                };
                let (nx, ny) = (x + dx, y + dy);
                if in_world(world, nx, ny, 2)
                    && world
                        .tile(nx, ny)
                        .is_some_and(|t| t.active() && matches!(t.kind, 59 | 60))
                    && can_chlorophyte_grow(world, nx, ny)
                {
                    convert_cell(world, nx, ny, false, true);
                }
            }
            // The ±6 purge pick (`:69689-69700`): the C#'s `while (flag)`
            // sets `flag = false` at the top and NEVER re-arms it (no
            // `continue`, no `flag = true` in the body) — it runs EXACTLY
            // ONE iteration: one `Main.rand.Next(-6, 7)` pair, then
            // `Convert(…, 9)` on the pick iff it is an in-world ACTIVE cell
            // (the grasses jungle-ify, the dirt mud-ifies, the evil
            // stones/sands purify, the thorns die).
            let nx = x + rng.next_range(-6, 7);
            let ny = y + rng.next_range(-6, 7);
            if in_world(world, nx, ny, 2)
                && world.tile(nx, ny).is_some_and(|t| t.active())
            {
                convert_purge_cell(world, nx, ny, 9);
            }
        }
    }
    // `:69702-69705`: the post-PlantBoss worlds halve the spread; the
    // creative spread-stop gate.
    if downed_plant_boss && rng.next_max(2) != 0 {
        return;
    }
    if !allowed_to_spread {
        return;
    }
    // The corruption loop (`:69706-69794`).
    if SPREADS_CORRUPTION.contains(&kind) {
        spread_loop(world, x, y, 1, rng);
    }
    // The crimson loop (`:69795-69883`).
    if SPREADS_CRIMSON.contains(&kind) {
        spread_loop(world, x, y, 4, rng);
    }
    // The hallow loop (`:69884-69954`) — no chlorophyte defence, the
    // sunflower check shares the InWorld gate. `TileID.Sets.SpreadsHallow`
    // (`TileID.cs:584`) is the generated bitset.
    if crate::tiledata::SET_SPREADS_HALLOW.has(kind) {
        spread_loop_hallow(world, x, y, rng);
    }
}

/// The evil infection loop (`WorldGen.cs:69706-69883`, the corruption and
/// crimson bodies are identical bar the conversion type): the ±3 pick, the
/// `InWorld(…, 10)` retry, the chlorophyte defence (the remix purge), the
/// sunflower retry, then the per-kind ladder — each convertible kind rolls
/// `Next(2) == 0` to re-draw and converts with `walls: false`. An
/// unconvertible target ends the loop (the C#'s `flag2` stays false).
fn spread_loop(world: &mut World, x: i32, y: i32, conversion_type: u8, rng: &mut UnifiedRandom) {
    loop {
        let tx = x + rng.next_range(-3, 4);
        let ty = y + rng.next_range(-3, 4);
        if !in_world(world, tx, ty, 10) {
            continue;
        }
        if nearby_chlorophyte(world, tx, ty, rng) {
            chlorophyte_defense(world, tx, ty);
            // The defended cell ends this iteration WITHOUT a re-roll
            // (the C#'s else-arm never runs, `flag2` stays false).
            break;
        }
        if count_near_blocks_types(world, tx, ty, 2, 1, &[27]) > 0 {
            continue;
        }
        let Some(t) = world.tile(tx, ty) else {
            break;
        };
        let kind = t.kind;
        // The ladder (`:69728-69791` / `:69817-69880`): the grass family,
        // the stone + moss, the sand, the sandstone, the hardened sand, the
        // jungle grass, the sandstone-69 arm, the ice.
        let convertible = matches!(kind, 2 | 477 | 1 | 53 | 396 | 397 | 60 | 69 | 161)
            || crate::tiledata::TILE_MOSS.has(kind);
        if !convertible {
            break;
        }
        let reroll = rng.next_max(2) == 0;
        convert_cell_flavor(
            world,
            tx,
            ty,
            if conversion_type == 1 {
                ConversionFlavor::Corruption
            } else {
                ConversionFlavor::Crimson
            },
            false,
        );
        if !reroll {
            break;
        }
    }
}

/// The hallow infection loop (`WorldGen.cs:69884-69954`): the same shape
/// minus the chlorophyte defence; the ladder is the grass 2, the golf
/// grass 477, the stone + moss, the sand 53, the sandstone 396, the
/// hardened sand 397, the ice 161 — each converts with type 2.
fn spread_loop_hallow(world: &mut World, x: i32, y: i32, rng: &mut UnifiedRandom) {
    loop {
        let tx = x + rng.next_range(-3, 4);
        let ty = y + rng.next_range(-3, 4);
        if !in_world(world, tx, ty, 10)
            || count_near_blocks_types(world, tx, ty, 2, 1, &[27]) > 0
        {
            continue;
        }
        let Some(t) = world.tile(tx, ty) else {
            break;
        };
        let kind = t.kind;
        let convertible =
            matches!(kind, 2 | 477 | 1 | 53 | 396 | 397 | 161) || crate::tiledata::TILE_MOSS.has(kind);
        if !convertible {
            break;
        }
        let reroll = rng.next_max(2) == 0;
        convert_cell_flavor(world, tx, ty, ConversionFlavor::Hallow, false);
        if !reroll {
            break;
        }
    }
}

/// `Convert`'s type-9/10 purge arms (`WorldGen.cs:55671-55763`): the
/// chlorophyte defence's jungle conversion. Type 9 also purifies the
/// grasses (2/477/492 → 60), type 10 keeps only the evil grasses
/// (23/199/661/662 → 60); both map the evil stones → 1, the evil sands →
/// 53, the hardened sands → 397, the sandstones → 396, and KILL the thorn
/// family (24/32/201/205/352/636). Type 9's dirt 0 → mud 59 arm is the
/// jungle-ification the purge pick rides.
fn convert_purge_cell(world: &mut World, x: i32, y: i32, conversion_type: u8) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    // The Convert head guard (`:55094-55097`): `!active && wall <= 0` —
    // an actuated cell WITH a wall still converts (the type arms read
    // `type`, not `active`).
    if !t.active() && t.wall == 0 {
        return;
    }
    let kind = t.kind;
    let new_kind = match kind {
        23 | 199 | 661 | 662 => Some(60),
        2 | 477 | 492 if conversion_type == 9 => Some(60),
        109 if conversion_type == 9 => Some(60),
        0 if conversion_type == 9 => Some(59),
        25 | 203 => Some(1),
        112 | 234 => Some(53),
        398 | 399 => Some(397),
        400 | 401 => Some(396),
        _ => None,
    };
    if let Some(k) = new_kind {
        if let Some(t) = world.tile_mut(x, y) {
            t.kind = k;
            t.frame_x = 0;
            t.frame_y = 0;
        }
        return;
    }
    // The thorn kill (`:55707-55718` / `:55750-55761`) — `KillTile(i, j)`
    // with the default `noItem: false` (the `convert_cleanse_cell`
    // precedent).
    if matches!(kind, 24 | 32 | 201 | 205 | 352 | 636) {
        let _ = crate::edit::kill_tile(world, x, y, false);
    }
}

/// `GERunner`'s hallow (good) map (`WorldGen.cs:76169-76290`): the hallowed
/// grass 109, the pearlstone 117, the hallowed sand 116, the hallowed ice
/// 164, the hallowed sandstone 403, the hallowed hardened sand 402 — and
/// the purify arm (the corrupt/crimson jungle grass 661/662 → the jungle
/// grass 60).
pub(crate) fn convert_tile_hallow(kind: u16) -> Option<u16> {
    Some(match kind {
        2 => 109,
        1 | 25 | 203 => 117,
        53 | 123 | 112 | 234 => 116,
        161 | 163 | 200 => 164,
        396 => 403,
        397 => 402,
        23 | 199 => 109,
        661 | 662 => 60,
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// `WorldGen.Convert` — the spray/cleanse entry (S1-D-4 unarmed styles, the
// aiStyle-31 Clentaminator arm's `WorldGen.Convert(point.X, point.Y, 0, size)`
// call, `Projectile.cs:26834`). Only conversion type 0 (the pure spray 145's
// `num245 = 0` default arm, `:26793-26795`) is ported here — the other rowed
// spray kinds (146-149, 1015-1017) have no `proj_data` rows yet.

/// `TileID.Sets.Conversion.Grass` (`TileID.cs:86`).
const CONV_GRASS: [u16; 6] = [2, 23, 199, 109, 477, 492];
/// `TileID.Sets.Conversion.JungleGrass` (`TileID.cs:84`).
const CONV_JUNGLE_GRASS: [u16; 3] = [60, 661, 662];
/// `TileID.Sets.Conversion.MushroomGrass` (`TileID.cs:85`).
const CONV_MUSHROOM_GRASS: [u16; 1] = [70];
/// `TileID.Sets.Conversion.Snow` (`TileID.cs:89`).
const CONV_SNOW: [u16; 1] = [147];
/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9695`).
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `WorldGen.Convert(i, j, conversionType, size)` (`WorldGen.cs:55067-55083`):
/// the square walk with the Manhattan `< 6` gate. The port carries the type-0
/// (cleanse) cell only — the spray's conversion type.
pub fn convert_cleanse(world: &mut World, i: i32, j: i32, size: i32) {
    for k in (i - size)..=(i + size) {
        for l in (j - size)..=(j + size) {
            if in_world(world, k, l, 1) && (k - i).abs() + (l - j).abs() < 6 {
                convert_cleanse_cell(world, k, l);
            }
        }
    }
}

/// `Convert`'s type-0 arm (`WorldGen.cs:55527-55664`): the wall chain, then
/// the torch/grass/stone/sand/ice tile ladder, the mushroom grass and the
/// thorn kill. `Convert_ActuallyConvertTile`/`Wall` (`:56312-56343`) write
/// only when the cell differs, and re-frame through `SquareTileFrame` /
/// `SquareWallFrame` — the port's [`crate::framing::reframe_area`] superset
/// (its documented deviation).
///
/// Deviations (ANALYSIS "The unarmed projectile styles (S1-D-4)"): the wall
/// 69/70/81 surface roll and the wall-80 depth rolls draw `WorldGen.genRand`
/// (`:55535`, `:55584`) — the port has no runtime genRand stream (the
/// `edit.rs` precedent), so the surface arm takes the no-rng 64 (the shared
/// `convert_wall_by_case(wall, 0, .., None)` behaviour) and the wall-80 arm
/// compares against the unrolled band edges. The tree cross-conversion
/// (`ConvertTreeAndGround`, `:54902+`) is unported — an invalid tree above a
/// converted cell is killed, never re-typed.
fn convert_cleanse_cell(world: &mut World, x: i32, y: i32) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    let (kind, wall, active) = (t.kind, t.wall, t.active());
    if !active && wall == 0 {
        return;
    }
    // The wall half (`:55528-55611`). The 80 arm (`:55582-55592`) sits in the
    // C#'s default chain between the NewWall ladder and the HardenedSand arm;
    // the shared `convert_wall_by_case` has no 80 row, so it is handled here
    // first (the two arms are wall-disjoint). The switch's no-op cases
    // 61/185/204-207 (`:55602-55608`) break WITHOUT converting — the shared
    // ladder's Stone/NewWall sets would otherwise catch them, so they are
    // gated out here.
    let new_wall = if matches!(wall, 61 | 185 | 204 | 205 | 206 | 207) {
        None
    } else if wall == 80 {
        // `j2 < worldSurface + 4 + Next(3) || j2 > (maxTilesY + rockLayer)/2
        // - 3 + Next(3)` — the unrolled band edges (the genRand skip).
        let mid = (world.height as f64 + world.rock_layer) / 2.0 - 3.0;
        Some(if (y as f64) < world.world_surface + 4.0 || (y as f64) > mid {
            15u16
        } else {
            64
        })
    } else {
        crate::growth::convert_wall_by_case(wall, 0, y, world.world_surface, None)
    };
    if let Some(new_wall) = new_wall {
        // `Convert_ActuallyConvertWall` (`:56329-56343`): non-zero, different.
        if wall != 0 && wall != new_wall {
            if let Some(t) = world.tile_mut(x, y) {
                t.wall = new_wall;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
    }
    // The tile half (`:55616-55663`) — the else-if ladder in source order.
    let torch_style = if active && kind == 4 {
        // `TileID.Sets.Conversion.Torch(tile, 0)` (`TileID.cs:51-56`): the
        // corrupt/crimson/hallow rows 18-20 and the mushroom row 22.
        let num = world.tile(x, y).map_or(0, |t| t.frame_y as i32 / 22);
        if (18..=20).contains(&num) || num == 22 {
            // `Convert_ActuallyConvertTorch` (`:56287-56310`): the type-0
            // mushroom quirk — style 22 lands on 21, not 0.
            Some(if num == 22 { 21i16 } else { 0 })
        } else {
            None
        }
    } else {
        None
    };
    if let Some(style) = torch_style {
        let cur = world.tile(x, y).map_or(0, |t| t.frame_y as i32 / 22);
        if cur != style as i32 {
            if let Some(t) = world.tile_mut(x, y) {
                t.frame_y = style * 22;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
        return;
    }
    let new_kind: Option<u16> = if kind == 492 {
        Some(477)
    } else if CONV_JUNGLE_GRASS.contains(&kind) && kind != 60 {
        Some(60)
    } else if CONV_GRASS.contains(&kind) && kind != 2 && kind != 477 {
        Some(2)
    } else if crate::tiledata::SET_CONVERSION_STONE.has(kind) && kind != 1 {
        Some(1)
    } else if crate::tiledata::SET_CONVERSION_SAND.has(kind) && kind != 53 {
        Some(53)
    } else if crate::tiledata::SET_CONVERSION_HARDENED_SAND.has(kind) && kind != 397 {
        Some(397)
    } else if crate::tiledata::SET_CONVERSION_SANDSTONE.has(kind) && kind != 396 {
        Some(396)
    } else if crate::tiledata::SET_ICE.has(kind) && kind != 161 {
        // `TileID.Sets.Conversion.Ice` = {161, 163, 164, 200} (`TileID.cs:91`)
        // — the generated Ice set carries the same four convertible ids.
        Some(161)
    } else if CONV_MUSHROOM_GRASS.contains(&kind) {
        Some(60)
    } else {
        None
    };
    if let Some(new_kind) = new_kind {
        // `Convert_ActuallyConvertTile` (`:56312-56327`): active, different,
        // the tree-above validity check, then the write + re-frame.
        if active && kind != new_kind {
            try_kill_invalid_tree_above(world, x, y, new_kind);
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = new_kind;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
    } else if kind == 32 || kind == 352 {
        // `:55656-55662` — the thorn bushes die (the netMode arm is MP-only).
        let _ = crate::edit::kill_tile(world, x, y, false);
    }
}

/// `TryConvertingOrKillingTreesAboveIfTheyWouldBecomeInvalid`
/// (`WorldGen.cs:54802-54899`): the tree trunk (or palm 323) directly above
/// the converted cell dies when the new floor no longer fits its ground
/// test. The gem/vanity/ash profiles (`GrowTreeSettings.Profiles.
/// TryGetFromTreeId`, `:3890-3928`) override the vanilla test; the
/// `ConvertTreeAndGround` cross-conversion half is the documented skip.
fn try_kill_invalid_tree_above(world: &mut World, x: i32, y: i32, new_floor: u16) {
    if !in_world(world, x, y, 2) {
        return;
    }
    let Some(above) = world.tile(x, y - 1) else {
        return;
    };
    if !above.active() {
        return;
    }
    let tree = above.kind;
    let is_trunk = crate::torch::TREE_TRUNKS.contains(&tree);
    if !is_trunk && tree != 323 {
        return;
    }
    let grass_side = CONV_GRASS.contains(&new_floor)
        || CONV_JUNGLE_GRASS.contains(&new_floor)
        || CONV_SNOW.contains(&new_floor)
        || CONV_MUSHROOM_GRASS.contains(&new_floor);
    let mut fits = true;
    match tree {
        5 => fits = grass_side,
        323 => fits = crate::tiledata::SET_CONVERSION_SAND.has(new_floor),
        _ => {}
    }
    // The profile override (`:54840-54843`): gem trees 583-589 take
    // `GemTreeGroundTest` (Stone | Moss, `:30304-30315`), the vanity trees
    // 596/616 `VanityTreeGroundTest` (Grass minus 23/199, `:30317-30328`),
    // the ash tree 634 `AshTreeGroundTest` (633, `:30330-30341`).
    if (583..=589).contains(&tree) {
        fits = crate::tiledata::SET_CONVERSION_STONE.has(new_floor)
            || crate::tiledata::SET_CONVERSION_MOSS.has(new_floor);
    } else if tree == 596 || tree == 616 {
        fits = CONV_GRASS.contains(&new_floor) && new_floor != 23 && new_floor != 199;
    } else if tree == 634 {
        fits = new_floor == 633;
    }
    if !fits {
        let _ = crate::edit::kill_tile(world, x, y - 1, false);
    }
}

/// `GERunner`'s wall arms (`:76237-76252`): the grass-wall family
/// {63, 65, 66, 68, 69, 81} → the hallowed grass wall 70, the hardened-sand
/// wall 216 → 219, the sandstone wall 187 → 222, the ebonstone/crimstone
/// walls 3/83 → the hallowed stone wall 28. (The pre-tail port carried only
/// the 187/3/83 slice; the family arms landed with the GERunner rewrite.)
pub(crate) fn convert_wall_hallow(wall: u16) -> Option<u16> {
    Some(match wall {
        63 | 65 | 66 | 68 | 69 | 81 => 70,
        216 => 219,
        187 => 222,
        3 | 83 => 28,
        _ => return None,
    })
}

/// `WorldGen.GERunner` (`WorldGen.cs:76169-76441`): the V-shaped strip
/// walker, verbatim. The shimmer pre-scan (`:76175-76190`) counts the
/// active shimmer 225 tiles in the 20-margin box — over 200000 the `flag`
/// arms convert the shimmer/honey family too. The disc radius
/// `num2 = (int)(Next(200, 250) · maxTilesX/4200)` (`:76191`), the position
/// starts at `(i, j)` and the velocity at the two `Next(-10, 11) · 0.1`
/// draws UNLESS a speed was passed (`:76197-76203`). Each step converts the
/// Manhattan disc (the per-cell `Next(-10, 11) · 0.015` rim wobble,
/// `:76231`), then drifts `vx += Next(-10, 11) · 0.05` clamped to
/// `speedX ± 1` (`:76427-76435`); the walk ends when the position leaves
/// the `±num2` band around the world (`:76436-76439`).
///
/// Deviations: `Skyblock.lowTiles` (`:76171-76174`) has no ported runtime
/// state on this branch (the f24 skyblock lane owns it) — a classic world
/// always enters. The per-conversion `SquareTileFrame` rides the port's
/// `framing::reframe_area` convention (the `grow_moss_arm` precedent).
pub fn gerunner(
    world: &mut World,
    i: i32,
    j: i32,
    speed_x: f64,
    speed_y: f64,
    good: bool,
    corruption: bool,
    rng: &mut UnifiedRandom,
) {
    // The shimmer pre-scan (`:76175-76190`).
    let mut shimmer_count = 0;
    for k in 20..world.width - 20 {
        for l in 20..world.height - 20 {
            if world.tile(k, l).is_some_and(|t| t.active() && t.kind == 225) {
                shimmer_count += 1;
            }
        }
    }
    let flag = shimmer_count > 200000;
    // `:76191-76203`.
    let num2 = (rng.next_range(200, 250) as f64 * (world.width as f64 / 4200.0)) as i32;
    let num3 = num2 as f64;
    let (mut px, mut py) = (i as f64, j as f64);
    let (mut vx, mut vy) = (
        rng.next_range(-10, 11) as f64 * 0.1,
        rng.next_range(-10, 11) as f64 * 0.1,
    );
    if speed_x != 0.0 || speed_y != 0.0 {
        vx = speed_x;
        vy = speed_y;
    }
    loop {
        // The clamped box (`:76207-76226`).
        let x0 = ((px - num3 * 0.5) as i32).max(0);
        let x1 = ((px + num3 * 0.5) as i32).min(world.width);
        let y0 = ((py - num3 * 0.5) as i32).max(0);
        let y1 = ((py + num3 * 0.5) as i32).min(world.height - 5);
        for m in x0..x1 {
            for n in y0..y1 {
                // The Manhattan gate with the per-cell wobble draw
                // (`:76231`) — the draw happens for EVERY box cell.
                let manhattan = (m as f64 - px).abs() + (n as f64 - py).abs();
                let wobble = 1.0 + rng.next_range(-10, 11) as f64 * 0.015;
                if manhattan >= num2 as f64 * 0.5 * wobble {
                    continue;
                }
                let Some(t) = world.tile(m, n) else {
                    continue;
                };
                let (kind, wall) = (t.kind, t.wall);
                if good {
                    // The hallow wall ladder (`:76237-76252`).
                    let new_wall = match wall {
                        63 | 65 | 66 | 68 | 69 | 81 => Some(70),
                        216 => Some(219),
                        187 => Some(222),
                        3 | 83 => Some(28),
                        _ => None,
                    };
                    if let Some(w) = new_wall {
                        if let Some(t) = world.tile_mut(m, n) {
                            t.wall = w;
                        }
                    }
                    // The hallow tile ladder (`:76253-76302`).
                    let new_kind = if flag && kind == 225 {
                        Some(117)
                    } else if flag && kind == 230 {
                        Some(402)
                    } else {
                        match kind {
                            2 => Some(109),
                            1 | 25 | 203 => Some(117),
                            53 | 123 | 112 | 234 => Some(116),
                            661 | 662 => Some(60),
                            23 | 199 => Some(109),
                            161 | 163 | 200 => Some(164),
                            396 => Some(403),
                            397 => Some(402),
                            _ => None,
                        }
                    };
                    if let Some(k) = new_kind {
                        if let Some(t) = world.tile_mut(m, n) {
                            t.kind = k;
                        }
                        crate::framing::reframe_area(world, m, n, 1, true);
                    }
                } else if !corruption {
                    // The crimson ladder (`:76304-76362`).
                    let new_wall = match wall {
                        63 | 65 | 66 | 68 => Some(81),
                        216 => Some(218),
                        187 => Some(221),
                        _ => None,
                    };
                    if let Some(w) = new_wall {
                        if let Some(t) = world.tile_mut(m, n) {
                            t.wall = w;
                        }
                    }
                    let new_kind = if flag && kind == 225 {
                        Some(203)
                    } else if flag && kind == 230 {
                        Some(399)
                    } else {
                        match kind {
                            60 | 661 => Some(662),
                            2 | 109 => Some(199),
                            1 | 117 => Some(203),
                            53 | 123 | 116 => Some(234),
                            161 | 164 => Some(200),
                            396 => Some(401),
                            397 => Some(399),
                            _ => None,
                        }
                    };
                    if let Some(k) = new_kind {
                        if let Some(t) = world.tile_mut(m, n) {
                            t.kind = k;
                        }
                        crate::framing::reframe_area(world, m, n, 1, true);
                    }
                } else {
                    // The corruption ladder (`:76364-76423`).
                    let new_wall = match wall {
                        63 | 65 | 66 | 68 => Some(69),
                        216 => Some(217),
                        187 => Some(220),
                        _ => None,
                    };
                    if let Some(w) = new_wall {
                        if let Some(t) = world.tile_mut(m, n) {
                            t.wall = w;
                        }
                    }
                    let new_kind = if flag && kind == 225 {
                        Some(25)
                    } else if flag && kind == 230 {
                        Some(398)
                    } else {
                        match kind {
                            60 | 662 => Some(661),
                            2 | 109 => Some(23),
                            1 | 117 => Some(25),
                            53 | 123 | 116 => Some(112),
                            161 | 164 => Some(163),
                            396 => Some(400),
                            397 => Some(398),
                            _ => None,
                        }
                    };
                    if let Some(k) = new_kind {
                        if let Some(t) = world.tile_mut(m, n) {
                            t.kind = k;
                        }
                        crate::framing::reframe_area(world, m, n, 1, true);
                    }
                }
            }
        }
        // The drift + clamp (`:76426-76435`).
        px += vx;
        py += vy;
        vx += rng.next_range(-10, 11) as f64 * 0.05;
        if vx > speed_x + 1.0 {
            vx = speed_x + 1.0;
        }
        if vx < speed_x - 1.0 {
            vx = speed_x - 1.0;
        }
        // The exit band (`:76436-76439`).
        if px < -(num2 as f64)
            || py < -(num2 as f64)
            || px > (world.width + num2) as f64
            || py > (world.height + num2) as f64
        {
            break;
        }
    }
}

/// `WorldGen.initializeHardMode` (`WorldGen.cs:31570-31699`): the hardmode
/// transform. The head draws (`:31581-31591`): the two band fractions
/// `Next(300, 400) · 0.001` / `Next(200, 300) · 0.001`, the `Next(2)` side
/// flip (`num5 = ±1`), then the dungeon-side bias (`:31592-31615`) pulls
/// the dungeon-opposite anchor toward the `num2` fraction. The dual
/// dungeons seed skips the V runners entirely (`:31576-31580`).
///
/// The remix arm (`:31618-31656`) paints a hallow band `width/7` wide plus
/// a `width/14` evil-recruit fringe on the dungeon-OPPOSITE edge, each
/// column starting at `worldSurface + Next(-1, 2)`; the classic arm runs
/// the two `GERunner` Vs (`:31659-31660`). The moss-wall tail
/// (`:31663-31698`) lands in the next slice.
///
/// Deviations: the `Main.rand == null` re-seed (`:31572-31575`) has no port
/// equivalent (the caller always supplies the stream); the remix arm's
/// `Convert(i, j, 2)` rides the port's `convert_cell_flavor` hallow seam
/// (the `micro_surface.rs` documented approximation).
pub fn initialize_hard_mode(world: &mut World, corruption: bool, rng: &mut UnifiedRandom) {
    let flag = !world
        .micro_seeds
        .contains(crate::seeds::MicroSeedFlags::DUAL_DUNGEONS);
    // `:31581-31591`.
    let num = rng.next_range(300, 400) as f64 * 0.001;
    let num2 = rng.next_range(200, 300) as f64 * 0.001;
    let width = world.width as f64;
    let mut num3 = (width * num) as i32;
    let mut num4 = (width * (1.0 - num)) as i32;
    let mut num5 = 1;
    if rng.next_max(2) == 0 {
        num4 = (width * num) as i32;
        num3 = (width * (1.0 - num)) as i32;
        num5 = -1;
    }
    // The dungeon-side bias (`:31592-31615`).
    let num6 = if world.dungeon_x < world.width / 2 { -1 } else { 1 };
    if num6 < 0 {
        if num4 < num3 {
            num4 = (width * num2) as i32;
        } else {
            num3 = (width * num2) as i32;
        }
    } else if num4 > num3 {
        num4 = (width * (1.0 - num2)) as i32;
    } else {
        num3 = (width * (1.0 - num2)) as i32;
    }
    if flag {
        if world.seeds.contains(crate::seeds::SeedFlags::REMIX) {
            // The remix hallow band (`:31618-31656`).
            let num7 = world.width / 7;
            let num8 = world.width / 14;
            let surface = world.world_surface as i32;
            if world.dungeon_x < world.width / 2 {
                for i in (world.width - num7 - num8)..world.width {
                    let start = surface + rng.next_range(-1, 2);
                    for j in start..world.height - 10 {
                        let recruit = i > world.width - num7
                            || world.tile(i, j).is_some_and(|t| {
                                crate::tiledata::sets::SET_CRIMSON.has(t.kind)
                                    || crate::tiledata::sets::SET_CORRUPT.has(t.kind)
                            });
                        if recruit {
                            convert_cell_flavor(world, i, j, ConversionFlavor::Hallow, true);
                        }
                    }
                }
            } else {
                for k in 0..(num7 + num8) {
                    let start = surface + rng.next_range(-1, 2);
                    for l in start..world.height - 10 {
                        let recruit = k < num7
                            || world.tile(k, l).is_some_and(|t| {
                                crate::tiledata::sets::SET_CRIMSON.has(t.kind)
                                    || crate::tiledata::sets::SET_CORRUPT.has(t.kind)
                            });
                        if recruit {
                            convert_cell_flavor(world, k, l, ConversionFlavor::Hallow, true);
                        }
                    }
                }
            }
        } else {
            // The classic V pair (`:31659-31660`).
            gerunner(world, num3, 0, 3.0 * num5 as f64, 5.0, true, corruption, rng);
            gerunner(world, num4, 0, -3.0 * num5 as f64, 5.0, false, corruption, rng);
        }
    }
    // The moss-wall tail (`:31663-31698`).
    hardmode_moss_walls(world, rng);
}

/// `initializeHardMode`'s moss-wall tail (`WorldGen.cs:31663-31698`):
/// `(int)(25 · maxTilesX/4200)` wall-ring placements. Each attempt draws a
/// `RandomWorldPoint(worldSurface − 100, 1, 190, 1)` (the `worldSurface <=
/// 100` arm uses top 20, `:31672`); a crimson/corrupt/hallowed floor tile
/// with air above rolls the unsafe-wall family (`192 + Next(4)` crimson,
/// `188 + Next(4)` corrupt, `200 + Next(4)` hallow, `:31676-31687`), then
/// floods the pocket above through `ShapeFloodFill(1000)` carrying
/// `IsNotSolid → OnlyWalls(the 31-wall natural set) → Blank.Output`
/// (`:31690`). A drained pocket over 50 cells takes the wall ring
/// (`OuterOutline(useDiagonals, useInterior)` + `PlaceWall`, `:31693`);
/// every 15000th failed attempt burns one placement (`:31668-31671`).
///
/// The shape/action framework is ported inline (the `wall_variety.rs`
/// precedent): the `OnlyWalls` list verbatim (the duplicated 71 kept), the
/// `PlaceWall` `SquareWallFrame` self + 4 neighbours deferred per the
/// port's plain-write convention.
fn hardmode_moss_walls(world: &mut World, rng: &mut UnifiedRandom) {
    let mut num9 = (25.0 * (world.width as f64 / 4200.0)) as i32;
    let mut shape = MossShape::default();
    let mut num10 = 0;
    while num9 > 0 {
        num10 += 1;
        if num10 % 15000 == 0 {
            num9 -= 1;
        }
        // `RandomWorldPoint` (`:27111-27114`): X first, then Y.
        let top = if world.world_surface > 100.0 {
            world.world_surface as i32 - 100
        } else {
            20
        };
        let px = rng.next_range(1, world.width - 1);
        let py = rng.next_range(top, world.height - 190);
        let Some(tile) = world.tile(px, py) else {
            continue;
        };
        let kind = tile.kind;
        let mut num11: u16 = 0;
        if crate::tiledata::sets::SET_CRIMSON.has(kind) {
            num11 = 192 + rng.next_max(4) as u16;
        } else if crate::tiledata::sets::SET_CORRUPT.has(kind) {
            num11 = 188 + rng.next_max(4) as u16;
        } else if crate::tiledata::sets::SET_HALLOW.has(kind) {
            num11 = 200 + rng.next_max(4) as u16;
        }
        let above_active = world.tile(px, py - 1).is_some_and(|t| t.active());
        if tile.active() && num11 != 0 && !above_active {
            let flag2 = moss_pocket_fill(world, (px, py - 1), &mut shape);
            if shape.count() > 50 && flag2 {
                moss_outline_place(world, (px, py), &shape, num11);
                num9 -= 1;
            }
            shape.clear();
        }
    }
}

/// The `OnlyWalls` list of the hardmode moss chain (`WorldGen.cs:31690`) —
/// the natural unsafe-wall families; the duplicated 71 is the C#'s own.
const MOSS_POCKET_WALLS: [u16; 31] = [
    0, 54, 55, 56, 57, 58, 59, 61, 185, 212, 213, 214, 215, 2, 196, 197, 198, 199, 15, 40, 71,
    64, 204, 205, 206, 207, 208, 209, 210, 211, 71,
];

/// `ShapeData` (`ShapeData.cs:9-114`) — the relative cell set, insertion
/// order beside the membership set (the `wall_variety.rs` precedent).
#[derive(Default)]
struct MossShape {
    points: Vec<(i32, i32)>,
    set: std::collections::HashSet<(i32, i32)>,
}

impl MossShape {
    fn add(&mut self, x: i32, y: i32) {
        if self.set.insert((x, y)) {
            self.points.push((x, y));
        }
    }
    fn contains(&self, x: i32, y: i32) -> bool {
        self.set.contains(&(x, y))
    }
    fn count(&self) -> usize {
        self.set.len()
    }
    fn clear(&mut self) {
        self.points.clear();
        self.set.clear();
    }
}

/// `ShapeFloodFill(1000)` (`ShapeFloodFill.cs:17-58`) carrying
/// `Chain(IsNotSolid, OnlyWalls(MOSS_POCKET_WALLS), Blank.Output(shape))`
/// (`WorldGen.cs:31690`): a cell passes only when it is not solid-or-sloped
/// AND its wall is in the natural set; a pass records the relative cell and
/// enqueues the 4 neighbours (the 1-tile border guards). Returns the C#
/// `queue.Count == 0` after the drain loop — true only when the pocket
/// drained within the 1000-action budget.
fn moss_pocket_fill(world: &World, origin: (i32, i32), shape: &mut MossShape) -> bool {
    use std::collections::{HashSet, VecDeque};
    let (ox, oy) = origin;
    let mut queue: VecDeque<(i32, i32)> = VecDeque::new();
    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    queue.push_back(origin);
    let mut num = 1000;
    while !queue.is_empty() && num > 0 {
        let (x, y) = queue.pop_front().unwrap();
        if visited.contains(&(x, y)) {
            continue;
        }
        // `Modifiers.IsNotSolid` (`Modifiers.cs:573-583`) — inactive or not
        // solid-or-sloped (`WorldGen.cs:69985-70009`: tileSolid, not
        // tileSolidTop, not wired-inactive; slopes/half-bricks pass).
        let not_solid = !world.tile(x, y).is_some_and(|t| {
            t.active()
                && crate::tiledata::TILE_SOLID.has(t.kind)
                && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
                && (t.flags & crate::tile::FLAG_INACTIVE) == 0
        });
        // `Modifiers.OnlyWalls` (`Modifiers.cs:220-230`).
        let wall_ok = world
            .tile(x, y)
            .is_some_and(|t| MOSS_POCKET_WALLS.contains(&t.wall));
        if !not_solid || !wall_ok {
            continue;
        }
        shape.add(x - ox, y - oy);
        visited.insert((x, y));
        num -= 1;
        if x + 1 < world.width - 1 {
            queue.push_back((x + 1, y));
        }
        if x - 1 >= 1 {
            queue.push_back((x - 1, y));
        }
        if y + 1 < world.height - 1 {
            queue.push_back((x, y + 1));
        }
        if y - 1 >= 1 {
            queue.push_back((x, y - 1));
        }
    }
    // The drain loop (`:48-57`).
    while let Some(item) = queue.pop_front() {
        if !visited.contains(&item) {
            queue.push_front(item);
            break;
        }
    }
    queue.is_empty()
}

/// `ModShapes.OuterOutline(shape, useDiagonals: true, useInterior: true)` +
/// `Actions.PlaceWall(num11)` (`WorldGen.cs:31693`, `ModShapes.cs:43-61`):
/// every recorded cell plus its 8-neighbour ring cells NOT in the set, in
/// discovery order. The origin is the FLOOR cell while the data was
/// recorded relative to the air cell above — the ring lands one row lower,
/// verbatim. The `PlaceWall` `SquareWallFrame` self + 4 neighbours
/// (`Actions.cs:536-547`) is deferred per the plain-write convention.
fn moss_outline_place(world: &mut World, origin: (i32, i32), shape: &MossShape, wall: u16) {
    const OFFSETS: [(i32, i32); 8] = [
        (1, 0),
        (-1, 0),
        (0, 1),
        (0, -1),
        (1, 1),
        (1, -1),
        (-1, 1),
        (-1, -1),
    ];
    let (ox, oy) = origin;
    for &(dx, dy) in &shape.points {
        if let Some(t) = world.tile_mut(ox + dx, oy + dy) {
            t.wall = wall;
        }
        for (px, py) in OFFSETS {
            if !shape.contains(dx + px, dy + py) {
                if let Some(t) = world.tile_mut(ox + dx + px, oy + dy + py) {
                    t.wall = wall;
                }
            }
        }
    }
}

#[cfg(test)]
mod runner_tests {
    use super::*;
    use crate::tile::{FLAG_ACTIVE, Tile};

    fn world() -> World {
        let mut w = World::new("t".into(), 800, 600);
        w.rock_layer = 300.0;
        // The ground band: the grass over the stone.
        for x in 0..800 {
            for y in 300..340 {
                put(&mut w, x, y, if y == 300 { 2 } else { 1 });
            }
            for y in 340..600 {
                put(&mut w, x, y, 1);
            }
        }
        w
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
    fn the_hallow_map_is_verbatim() {
        // `GERunner`'s good arms (`WorldGen.cs:76262-76290`): the hallowed
        // grass 109, the pearlstone 117, the hallowed sand 116, the
        // hallowed ice 164, 403, 402 — and the purify arm (661/662 → 60).
        assert_eq!(convert_tile_hallow(2), Some(109));
        assert_eq!(convert_tile_hallow(1), Some(117));
        assert_eq!(convert_tile_hallow(25), Some(117));
        assert_eq!(convert_tile_hallow(203), Some(117));
        assert_eq!(convert_tile_hallow(53), Some(116));
        assert_eq!(convert_tile_hallow(112), Some(116));
        assert_eq!(convert_tile_hallow(234), Some(116));
        assert_eq!(convert_tile_hallow(161), Some(164));
        assert_eq!(convert_tile_hallow(163), Some(164));
        assert_eq!(convert_tile_hallow(396), Some(403));
        assert_eq!(convert_tile_hallow(397), Some(402));
        assert_eq!(convert_tile_hallow(23), Some(109));
        assert_eq!(convert_tile_hallow(199), Some(109));
        assert_eq!(convert_tile_hallow(661), Some(60), "the purify arm");
        assert_eq!(convert_tile_hallow(662), Some(60));
        assert_eq!(convert_tile_hallow(211), None);
        // The walls (`:76237-76252`): the grass family → 70, 216 → 219,
        // 187 → 222, 3/83 → 28.
        assert_eq!(convert_wall_hallow(63), Some(70));
        assert_eq!(convert_wall_hallow(69), Some(70));
        assert_eq!(convert_wall_hallow(81), Some(70));
        assert_eq!(convert_wall_hallow(216), Some(219));
        assert_eq!(convert_wall_hallow(187), Some(222));
        assert_eq!(convert_wall_hallow(3), Some(28));
        assert_eq!(convert_wall_hallow(83), Some(28));
    }

    #[test]
    fn the_gerunner_carves_the_v_strip() {
        let mut w = world();
        let mut rng = UnifiedRandom::new(9);
        gerunner(&mut w, 400, 0, 2.0, 5.0, true, false, &mut rng);
        // The strip: the hallowed grass/pearlstone along the V from the
        // sky to the bottom.
        let hallowed = w
            .tiles
            .iter()
            .filter(|t| t.active() && matches!(t.kind, 109 | 117))
            .count();
        assert!(hallowed > 300, "the hallowed tiles ({hallowed})");
        // The V spans the ground band's full depth: the hallowed tiles at
        // the top (300-340) and at the bottom (560-599) of the band.
        let top = (300..340).any(|y| {
            (0..800).any(|x| {
                w.tile(x, y)
                    .is_some_and(|t| t.active() && matches!(t.kind, 109 | 117))
            })
        });
        let bottom = (560..600).any(|y| {
            (0..800).any(|x| {
                w.tile(x, y)
                    .is_some_and(|t| t.active() && matches!(t.kind, 109 | 117))
            })
        });
        assert!(top, "the V top");
        assert!(bottom, "the V bottom");
    }

    #[test]
    fn the_evil_runner_uses_the_flavour() {
        // The evil runner with the corruption: the grass → the corrupt
        // grass 23, the stone → the ebonstone 25.
        let mut w = world();
        let mut rng = UnifiedRandom::new(10);
        gerunner(&mut w, 400, 0, -2.0, 5.0, false, true, &mut rng);
        let corrupt = w
            .tiles
            .iter()
            .filter(|t| t.active() && matches!(t.kind, 23 | 25))
            .count();
        assert!(corrupt > 300, "the corrupted tiles ({corrupt})");
    }

    #[test]
    fn initialize_hard_mode_runs_both_runners() {
        // The corruption world (the crimson world swaps the evil kinds).
        let mut w = world();
        let mut rng = UnifiedRandom::new(11);
        initialize_hard_mode(&mut w, true, &mut rng);
        let hallowed = w
            .tiles
            .iter()
            .filter(|t| t.active() && matches!(t.kind, 109 | 117 | 116 | 164))
            .count();
        let corrupt = w
            .tiles
            .iter()
            .filter(|t| t.active() && matches!(t.kind, 23 | 25 | 112))
            .count();
        assert!(hallowed > 200, "the hallow V ({hallowed})");
        assert!(corrupt > 200, "the evil V ({corrupt})");
        // The crimson world: the evil kinds swap (199/203/234).
        let mut w2 = world();
        let mut rng = UnifiedRandom::new(11);
        initialize_hard_mode(&mut w2, false, &mut rng);
        let crimson = w2
            .tiles
            .iter()
            .filter(|t| t.active() && matches!(t.kind, 199 | 203 | 234))
            .count();
        assert!(crimson > 200, "the crimson V ({crimson})");
    }

    /// W4-d103 tail (P2): `initializeHardMode`'s head draws
    /// (`WorldGen.cs:31581-31591`) — `Next(300, 400)`, `Next(200, 300)`,
    /// `Next(2)` — precede the runners' draws. The mirror consumes exactly
    /// the three head draws, then a twin world runs ONLY the two GERunner
    /// calls from that state: equal streams prove the head drew exactly
    /// three words in the cited order (the runners are deterministic given
    /// the same anchors, which the twin re-derives from the same draws).
    #[test]
    fn w4_d103_p2_the_hardmode_head_draws_three_before_the_runners() {
        let mut w = World::new("t".into(), 60, 60);
        w.dungeon_x = 45; // the right side: num6 = +1
        let mut rng = UnifiedRandom::new(7);
        initialize_hard_mode(&mut w, true, &mut rng);
        // The three head draws, verbatim order, then the anchor maths.
        let mut mirror = UnifiedRandom::new(7);
        let num = mirror.next_range(300, 400) as f64 * 0.001;
        let num2 = mirror.next_range(200, 300) as f64 * 0.001;
        let width = 60.0;
        let (mut num3, mut num4, mut num5) =
            ((width * num) as i32, (width * (1.0 - num)) as i32, 1);
        if mirror.next_max(2) == 0 {
            num4 = (width * num) as i32;
            num3 = (width * (1.0 - num)) as i32;
            num5 = -1;
        }
        // num6 = +1 (dungeon right): the `else if num4 > num3` arm.
        if num4 > num3 {
            num4 = (width * (1.0 - num2)) as i32;
        } else {
            num3 = (width * (1.0 - num2)) as i32;
        }
        // The runners from the post-head state.
        let mut twin = World::new("t".into(), 60, 60);
        twin.dungeon_x = 45;
        gerunner(&mut twin, num3, 0, 3.0 * num5 as f64, 5.0, true, true, &mut mirror);
        gerunner(&mut twin, num4, 0, -3.0 * num5 as f64, 5.0, false, true, &mut mirror);
        assert_eq!(rng, mirror, "exactly the three head draws + the runners");
        assert_eq!(w.tiles, twin.tiles, "the same conversions landed");
    }

    /// W4-d103 tail (P2): the dungeon-side bias (`:31592-31615`) — a LEFT
    /// dungeon pulls the dungeon-opposite anchor to the `num2` fraction.
    /// The two worlds differ ONLY in `dungeon_x`; the head draws are the
    /// same stream prefix, so the divergence proves the bias reads it.
    #[test]
    fn w4_d103_p2_the_dungeon_side_biases_the_anchors() {
        let mut left = world();
        left.dungeon_x = 100; // < 400: num6 = -1
        let mut right = world();
        right.dungeon_x = 700; // >= 400: num6 = +1
        let mut rng_a = UnifiedRandom::new(21);
        let mut rng_b = UnifiedRandom::new(21);
        initialize_hard_mode(&mut left, true, &mut rng_a);
        initialize_hard_mode(&mut right, true, &mut rng_b);
        assert_ne!(
            left.tiles, right.tiles,
            "the dungeon side moves the V anchors"
        );
    }

    /// W4-d103 tail (P3): the remix arm (`:31618-31656`) paints the hallow
    /// band on the dungeon-opposite edge — a RIGHT dungeon puts the band
    /// on the LEFT `width/7 + width/14` columns, each column's start
    /// jittered by `Next(-1, 2)`.
    #[test]
    fn w4_d103_p3_the_remix_band_hallows_the_opposite_edge() {
        let mut w = world();
        w.seeds = crate::seeds::SeedFlags::none();
        w.seeds.insert(crate::seeds::SeedFlags::REMIX);
        w.dungeon_x = 700; // the right side: the band lands left
        let mut rng = UnifiedRandom::new(5);
        initialize_hard_mode(&mut w, true, &mut rng);
        // The left band (0..width/7 = 0..114) carries hallowed stone.
        let left_hallow = (0..114).any(|x| {
            (300..590).any(|y| {
                w.tile(x, y)
                    .is_some_and(|t| t.active() && matches!(t.kind, 109 | 117))
            })
        });
        assert!(left_hallow, "the remix hallow band on the left edge");
        // The far right (past width - width/7 - width/14) stays untouched
        // by the band arm (no evil tiles there to recruit).
        let right_hallow = (700..800).any(|x| {
            (300..590).any(|y| {
                w.tile(x, y)
                    .is_some_and(|t| t.active() && matches!(t.kind, 109 | 117))
            })
        });
        assert!(!right_hallow, "the band stays on the dungeon-opposite edge");
    }

    /// W4-d103 tail (P2): the dual-dungeons seed skips the V runners
    /// (`:31576-31580`) — the head still draws, the tiles stay untouched.
    #[test]
    fn w4_d103_p2_dual_dungeons_skips_the_runners() {
        let mut w = world();
        w.micro_seeds = crate::seeds::MicroSeedFlags::from(crate::seeds::MicroSeedFlags::DUAL_DUNGEONS);
        let before = w.tiles.clone();
        let mut rng = UnifiedRandom::new(3);
        let pristine = rng.clone();
        initialize_hard_mode(&mut w, true, &mut rng);
        assert_eq!(w.tiles, before, "no V conversions");
        assert_ne!(rng, pristine, "the head still drew");
    }

    /// W4-d103 tail (P5): the moss-wall tail (`:31663-31698`) — a corrupt
    /// floor tile with a drained wall-0 pocket above takes the 188-191
    /// unsafe-wall ring (the outline lands one row lower than the pocket,
    /// the floor-cell origin quirk). The fixture: a stone ceiling at row
    /// 180, stone dividers every 10 columns, the corrupt floor at row 198 —
    /// each 10x17 air pocket (>50 cells) drains within the 1000 budget, and
    /// EVERY non-divider floor column qualifies, so the drawn point lands
    /// one within the 15000-attempt burn budget.
    #[test]
    fn w4_d103_p5_the_moss_wall_ring_paints_the_pocket_rim() {
        fn fixture() -> World {
            let mut w = World::new("t".into(), 200, 400);
            w.world_surface = 150.0;
            // The stone ground from row 198 down + the ceiling at 180.
            for x in 0..200 {
                for y in 198..400 {
                    put(&mut w, x, y, 1);
                }
                put(&mut w, x, 180, 1);
            }
            // The dividers every 10 columns (rows 181..197).
            for x in (0..200).step_by(10) {
                for y in 181..198 {
                    put(&mut w, x, y, 1);
                }
            }
            // The corrupt floor row.
            for x in 0..200 {
                put(&mut w, x, 198, 25);
            }
            w
        }
        let mut w = fixture();
        let mut rng = UnifiedRandom::new(17);
        hardmode_moss_walls(&mut w, &mut rng);
        // Some ring cell took a 188-191 wall (the placement point is drawn,
        // so the exact cell varies — the family is the assertion).
        let painted = w
            .tiles
            .iter()
            .filter(|t| (188..=191).contains(&t.wall))
            .count();
        assert!(painted > 0, "the corrupt unsafe-wall ring ({painted})");
        // a == b: the same seed replays byte-identical.
        let mut w2 = fixture();
        let mut rng2 = UnifiedRandom::new(17);
        hardmode_moss_walls(&mut w2, &mut rng2);
        assert_eq!(w.tiles, w2.tiles, "a == b");
        assert_eq!(rng, rng2, "a == b stream");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `hardUpdateWorld`'s first arm is the crystal shards (`:69604-69644`):
    /// a shard-capable stone below the rock layer grows tile 129 through
    /// `hard_update_world`, at the vanilla gates (the `Next(5)` roll, the
    /// radius-6 density).
    #[test]
    fn hard_update_world_grows_the_crystal_shards_first() {
        // 117 = Pearlstone (`CAN_GROW_CRYSTAL_SHARDS`), below the rock layer.
        let mut w = World::new("t".into(), 60, 120);
        w.rock_layer = 40.0;
        for x in 0..60 {
            for y in 20..120 {
                let i = w.index(x, y);
                w.tiles[i] = crate::tile::Tile {
                    kind: 117,
                    ..Default::default()
                };
                w.tiles[i].flags |= crate::tile::FLAG_ACTIVE;
            }
        }
        // Air around the source so the 4-dir pick has somewhere to plant.
        for (dx, dy) in [(-1, 0), (1, 0), (0, 1)] {
            let i = w.index(30 + dx, 80 + dy);
            w.tiles[i] = crate::tile::Tile::default();
        }
        // Sweep seeds: over enough picks the 1/5 gate and the 4-dir roll land.
        let mut grew = 0;
        let mut runs = 0;
        for seed in 1u64..400 {
            let mut w = w.clone();
            let mut rng = UnifiedRandom::new(seed as i32);
            hard_update_world(&mut w, 30, 80, false, true, &mut rng);
            runs += 1;
            grew += w
                .tiles
                .iter()
                .filter(|t| t.active() && t.kind == 129)
                .count();
        }
        assert!(grew > 0, "the shard arm plants 129 ({grew} over {runs} picks)");
    }

    use super::*;
    use crate::tile::{FLAG_ACTIVE, Tile};

    fn world() -> World {
        let mut w = World::new("t".into(), 400, 400);
        w.rock_layer = 200.0;
        w
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
    fn the_convert_map_is_verbatim() {
        // `:55139-55205`: the corruption targets (the ebonstone 25, the
        // corrupt grass 23, the ebonsand 112, the purple ice 163, 398, 400)
        // and the crimson twins (203, 199, 234, 200, 399, 401).
        // The corruption (type 1): the ebonstone 25, the corrupt grass 23,
        // the ebonsand 112, the purple ice 163, 398, 400. The crimson
        // (type 4): the crimstone 203, the crimson grass 199, the jungle
        // grass 662, the crimsand 234, the red ice 200, 399, 401.
        assert_eq!(convert_tile(1, true), Some(25));
        assert_eq!(convert_tile(2, true), Some(23));
        assert_eq!(convert_tile(60, true), Some(23));
        assert_eq!(convert_tile(53, true), Some(112));
        assert_eq!(convert_tile(161, true), Some(163));
        assert_eq!(convert_tile(397, true), Some(398));
        assert_eq!(convert_tile(396, true), Some(400));
        assert_eq!(convert_tile(1, false), Some(203));
        assert_eq!(convert_tile(2, false), Some(199));
        assert_eq!(convert_tile(60, false), Some(662));
        assert_eq!(convert_tile(53, false), Some(234));
        assert_eq!(convert_tile(161, false), Some(200));
        assert_eq!(convert_tile(396, false), Some(401));
        assert_eq!(convert_tile(399, false), None, "the already-corrupt tile");
        // The walls (`:55261-55296` corruption / `:55100-55135` crimson): the
        // stone 1 → 3 / 83 — the C#'s chain has no dirt-wall row, so the old
        // natural-slice 2 → 69/193 pair was the deviation M5-SPREAD-2c
        // replaces (the dirt-wall arm lands on conversion types 0/7/11).
        assert_eq!(convert_wall(1, true), Some(3));
        assert_eq!(convert_wall(1, false), Some(83));
        assert_eq!(convert_wall(2, true), None);
        assert_eq!(convert_wall(2, false), None);
    }

    /// M5-SPREAD-2c: the extended conversion types' wall chains are the C#
    /// verbatim (`:55446+` purify, `:55409+` snow, `:55363+` desert,
    /// `:55337+` glowing mushroom, `:55883+` corruption-II).
    #[test]
    fn m5_spread_2c_the_wall_chains_are_verbatim_across_the_types() {
        // Purify (7): the natural walls → 349, the sand/dirt family → 2, the
        // NewWall family → 212-215, the 262/274 switch.
        assert_eq!(growth_wall(1, 7), Some(349));
        // Wall 2 is the Dirt set's own target (`wall != 2` guard, `:55465`) —
        // the second dirt wall 16 converts.
        assert_eq!(growth_wall(2, 7), None);
        assert_eq!(growth_wall(16, 7), Some(2));
        assert_eq!(growth_wall(188, 7), Some(212));
        assert_eq!(growth_wall(262, 7), Some(61));
        assert_eq!(growth_wall(274, 7), Some(185));
        // Snow (6): the stone side → 71, the sand/dirt side → 40.
        assert_eq!(growth_wall(1, 6), Some(71));
        assert_eq!(growth_wall(2, 6), Some(40));
        // Desert (5): 187 / 216.
        assert_eq!(growth_wall(1, 5), Some(187));
        assert_eq!(growth_wall(2, 5), Some(216));
        // Glowing mushroom (3): the four mushroom-safe walls → 80.
        assert_eq!(growth_wall(15, 3), Some(80));
        assert_eq!(growth_wall(2, 3), None);
        // Corruption-II (12): the NewWall family → the corruption analog, else
        // any live non-ebonstone wall → 3.
        // The guards make the set's own members fall through to the ebonstone
        // arm (188 ∈ NewWall1, so `wall != 188` fails and 3 wins).
        assert_eq!(growth_wall(192, 12), Some(188));
        assert_eq!(growth_wall(188, 12), Some(3));
        assert_eq!(growth_wall(187, 12), Some(3));
        // Cleanse (0): the grass family → 64, the 262/274/292-295 switch.
        assert_eq!(growth_wall(69, 0), Some(64));
        assert_eq!(growth_wall(292, 0), Some(204));
        assert_eq!(growth_wall(2, 0), None, "the dirt wall has no type-0 arm");
    }

    /// The growth module's wall-chain entry, straight from the test.
    fn growth_wall(wall: u16, conversion_type: u8) -> Option<u16> {
        crate::growth::convert_wall_by_case(wall, conversion_type, 0, 1.0e9, None)
    }

    #[test]
    fn the_spread_converts_the_neighbours() {
        // The ebonstone source: the grass neighbours convert to the corrupt
        // grass, the stone to the ebonstone.
        let mut w = world();
        put(&mut w, 200, 200, 25); // the ebonstone source
        put(&mut w, 201, 200, 2); // the grass
        put(&mut w, 199, 200, 1); // the stone
        put(&mut w, 200, 199, 53); // the sand
        put(&mut w, 200, 201, 161); // the ice
        // The stone's wall set before the spread — the C#'s infection
        // Convert calls are ALL `walls: false` (`:69734`, `:69742`, …), so
        // the wall stays natural even as the tile corrupts.
        let i = w.index(199, 200);
        w.tiles[i].wall = 1;
        let mut rng = UnifiedRandom::new(1);
        for _ in 0..200 {
            hard_update_world(&mut w, 200, 200, false, true, &mut rng);
        }
        assert_eq!(w.tile(201, 200).unwrap().kind, 23, "the grass corrupted");
        assert_eq!(w.tile(199, 200).unwrap().kind, 25, "the stone corrupted");
        assert_eq!(w.tile(200, 199).unwrap().kind, 112, "the sand corrupted");
        assert_eq!(w.tile(200, 201).unwrap().kind, 163, "the ice corrupted");
        assert_eq!(
            w.tile(199, 200).unwrap().wall,
            1,
            "the wall stays — the spread Convert is walls: false"
        );
    }

    #[test]
    fn the_sunflower_and_chlorophyte_defend() {
        // The sunflower (27) within 2 tiles stops the spread. The C#'s
        // `continue` re-draws a sunflower-blocked pick (`:69724-69727`), so
        // the assertion is statistical: over many seeds the guarded cell
        // converts far less often than the open one.
        let mut guarded_hits = 0;
        let mut open_hits = 0;
        for seed in 1u64..60 {
            let mut w = world();
            put(&mut w, 200, 200, 25);
            put(&mut w, 201, 200, 2);
            put(&mut w, 204, 200, 27); // the sunflower two tiles out
            put(&mut w, 203, 200, 2);
            let mut rng = UnifiedRandom::new(seed as i32);
            for _ in 0..100 {
                hard_update_world(&mut w, 200, 200, false, true, &mut rng);
            }
            if w.tile(203, 200).unwrap().kind == 23 {
                guarded_hits += 1;
            }
            if w.tile(201, 200).unwrap().kind == 23 {
                open_hits += 1;
            }
        }
        assert!(
            guarded_hits < open_hits,
            "the sunflower guards ({guarded_hits} vs {open_hits})"
        );
        // The chlorophyte defence (`ChlorophyteDefense` is the remix-only
        // conversion — the normal worlds just stop the spread): the
        // corrupt grass next to THREE chlorophytes (the >=3 short-circuit)
        // stays put across the seeds.
        let mut defended_hits = 0;
        for seed in 1u64..60 {
            let mut w = world();
            put(&mut w, 199, 199, 211);
            put(&mut w, 200, 199, 211);
            put(&mut w, 201, 199, 211);
            put(&mut w, 202, 200, 23); // the corrupt grass in the defence box
            put(&mut w, 203, 200, 2); // the grass beside it
            let mut rng = UnifiedRandom::new(seed as i32);
            for _ in 0..100 {
                hard_update_world(&mut w, 202, 200, false, true, &mut rng);
            }
            if w.tile(203, 200).unwrap().kind == 23 {
                defended_hits += 1;
            }
        }
        assert!(
            defended_hits < 30,
            "the >=3 defence holds most picks ({defended_hits}/60)"
        );
    }

    #[test]
    fn the_chlorophyte_grows_into_the_mud() {
        // The 211 source: the mud neighbours convert, gated by
        // `CanChlorophyteGrow` (the local density ≤ 40).
        let mut w = world();
        for x in 190..210 {
            for y in 190..210 {
                put(&mut w, x, y, 59); // the mud field
            }
        }
        put(&mut w, 200, 200, 211);
        let mut rng = UnifiedRandom::new(5);
        let mut grew = false;
        for _ in 0..200 {
            hard_update_world(&mut w, 200, 200, false, true, &mut rng);
            if w.tile(201, 200).unwrap().kind == 211 {
                grew = true;
                break;
            }
        }
        assert!(grew, "the chlorophyte spreads into the mud");
    }

    #[test]
    fn the_spread_gates_respect_the_inputs() {
        // `allowed_to_spread = false` freezes the spread.
        let mut w = world();
        put(&mut w, 200, 200, 25);
        put(&mut w, 201, 200, 2);
        let mut rng = UnifiedRandom::new(6);
        for _ in 0..100 {
            hard_update_world(&mut w, 200, 200, false, false, &mut rng);
        }
        assert_eq!(w.tile(201, 200).unwrap().kind, 2, "the spread frozen");
    }

    /// W4-d103 tail (P8/P13): the `hardModeWorldUpdates` gate
    /// (`WorldGen.cs:69602`) — a pre-hardmode classic world draws NOTHING
    /// and converts nothing; the flag derivation (`:71501`) opens it for
    /// hardmode and for the remix·getGood·!anniversary combo.
    #[test]
    fn w4_d103_p8_the_hardmode_gate_freezes_the_pre_hardmode_world() {
        let mut w = world();
        put(&mut w, 200, 200, 25);
        put(&mut w, 201, 200, 2);
        let mut rng = UnifiedRandom::new(7);
        let pristine = rng.clone();
        for _ in 0..50 {
            hard_update_world_gated(&mut w, 200, 200, false, false, true, &mut rng);
        }
        assert_eq!(rng, pristine, "the gated-off arm draws nothing");
        assert_eq!(w.tile(201, 200).unwrap().kind, 2, "no spread");
        // The derivation: classic pre-hardmode false, hardmode true, the
        // remix·getGood combo true, the anniversary veto false.
        let mut c = World::new("t".into(), 100, 100);
        assert!(!crate::growth::hard_mode_world_updates(&c));
        c.hard_mode = true;
        assert!(crate::growth::hard_mode_world_updates(&c));
        c.hard_mode = false;
        c.seeds.insert(crate::seeds::SeedFlags::REMIX);
        c.seeds.insert(crate::seeds::SeedFlags::FOR_THE_WORTHY);
        assert!(crate::growth::hard_mode_world_updates(&c));
        c.seeds.insert(crate::seeds::SeedFlags::TENTH_ANNIVERSARY);
        assert!(!crate::growth::hard_mode_world_updates(&c));
    }

    /// W4-d103 tail (P8): the actuated-cell skip (`:69602`'s `inActive()`)
    /// — an actuated source draws nothing.
    #[test]
    fn w4_d103_p8_the_actuated_source_skips() {
        let mut w = world();
        put(&mut w, 200, 200, 25);
        let i = w.index(200, 200);
        w.tiles[i].flags |= crate::tile::FLAG_INACTIVE;
        put(&mut w, 201, 200, 2);
        let mut rng = UnifiedRandom::new(8);
        let pristine = rng.clone();
        hard_update_world(&mut w, 200, 200, false, true, &mut rng);
        assert_eq!(rng, pristine, "the actuated cell draws nothing");
        assert_eq!(w.tile(201, 200).unwrap().kind, 2);
    }

    /// W4-d103 tail (P8): the 60-grass chlorophyte seeding arm
    /// (`:69652-69660`) — below the surface/rock midpoint, a jungle-grass
    /// source rolls `Next(300)`; on 0 the ±10 mud target with a free top
    /// grows the chlorophyte 211 through `Convert(…, 8)`.
    #[test]
    fn w4_d103_p8_the_jungle_grass_seeds_chlorophyte() {
        let mut w = world();
        w.world_surface = 100.0;
        w.rock_layer = 200.0;
        // The mud field below the midpoint (150).
        for x in 180..220 {
            for y in 180..220 {
                put(&mut w, x, y, 59);
            }
        }
        put(&mut w, 200, 200, 60); // the jungle-grass source
        let mut rng = UnifiedRandom::new(1);
        let mut grew = false;
        for _ in 0..4000 {
            hard_update_world(&mut w, 200, 200, false, true, &mut rng);
            if (180..220).any(|x| {
                (180..220).any(|y| {
                    w.tile(x, y).is_some_and(|t| t.active() && t.kind == 211)
                })
            }) {
                grew = true;
                break;
            }
        }
        assert!(grew, "the 1/300 arm seeded a chlorophyte into the mud");
    }

    /// W4-d103 tail (P8): the type-9 purge pick (`:69689-69700`) — the C#'s
    /// `while (flag)` clears `flag` at the top and never re-arms it, so it
    /// runs EXACTLY ONCE: one `Next(-6, 7)` pair, `Convert(…, 9)` on the pick
    /// iff it is an in-world ACTIVE cell. The witness drives a fully-solid
    /// box (every pick is active, so a looping port would keep drawing) and
    /// asserts the stream advanced by EXACTLY the head draws + one pair, and
    /// that a == b replays byte-identical.
    #[test]
    fn w4_d103_p8_the_purge_runs_exactly_one_pick() {
        let build = || {
            let mut w = world();
            w.world_surface = 100.0;
            w.rock_layer = 200.0;
            // A solid stone box around the 211 source: every ±6 pick lands
            // on an active cell, so a "loop until air" port would spin.
            for x in 190..211 {
                for y in 190..211 {
                    put(&mut w, x, y, 1);
                }
            }
            put(&mut w, 200, 200, 211);
            w
        };
        // The 211 source is NOT shard-capable, so the head's `Next(5)` is
        // skipped; below the midpoint the 211/346 arm draws `Next(3)` (the
        // 4-dir growth gate), then the single purge `Next(-6, 7)` pair.
        let mut w = build();
        let mut rng = UnifiedRandom::new(2);
        let mut mirror = rng.clone();
        // Replay the head draws the C# makes before the purge pick: the
        // `Next(3)` gate, and — only when it fires (!= 0) — the `Next(4)`
        // direction. Then the one purge pair.
        let gate = mirror.next_max(3);
        if gate != 0 {
            let _dir = mirror.next_max(4);
        }
        let _px = mirror.next_range(-6, 7);
        let _py = mirror.next_range(-6, 7);
        hard_update_world(&mut w, 200, 200, false, true, &mut rng);
        assert_eq!(rng, mirror, "the purge draws exactly ONE Next(-6,7) pair");

        // a == b: the twin replays byte-identical (the single pick included).
        let mut w2 = build();
        let mut rng2 = UnifiedRandom::new(2);
        hard_update_world(&mut w2, 200, 200, false, true, &mut rng2);
        assert_eq!(rng, rng2, "a == b stream");
        assert_eq!(w.tiles, w2.tiles, "a == b tiles");
    }

    /// W4-d103 tail (P8): the hallow infection loop (`:69884-69954`) — a
    /// pearlstone 117 source hallow-ifies the grass/stone/sand neighbours
    /// (the type-2 Convert), with no chlorophyte defence.
    #[test]
    fn w4_d103_p8_the_hallow_source_spreads() {
        let mut w = world();
        put(&mut w, 200, 200, 117); // the pearlstone source
        put(&mut w, 201, 200, 2); // the grass
        put(&mut w, 199, 200, 1); // the stone
        put(&mut w, 200, 199, 53); // the sand
        let mut rng = UnifiedRandom::new(3);
        for _ in 0..300 {
            hard_update_world(&mut w, 200, 200, false, true, &mut rng);
        }
        assert_eq!(w.tile(201, 200).unwrap().kind, 109, "the grass hallowed");
        assert_eq!(w.tile(199, 200).unwrap().kind, 117, "the stone hallowed");
        assert_eq!(w.tile(200, 199).unwrap().kind, 116, "the sand hallowed");
    }

    /// W4-d103 tail (P11): `nearbyChlorophyte` (`:69517-69558`) — the ±5
    /// box counts 211 AND 346; ≥3 always defends, 1-2 roll the
    /// `Next(-1, 4) < count` gate. The 10-tile border returns false
    /// without drawing.
    #[test]
    fn w4_d103_p11_the_chlorophyte_defence_counts_both_sources() {
        let mut w = world();
        // Three sources in the box: always true, no draw.
        put(&mut w, 199, 200, 211);
        put(&mut w, 200, 200, 346);
        put(&mut w, 201, 200, 211);
        let mut rng = UnifiedRandom::new(4);
        let pristine = rng.clone();
        assert!(nearby_chlorophyte(&w, 200, 200, &mut rng));
        assert_eq!(rng, pristine, "the >=3 short-circuit draws nothing");
        // The border margin: false, no draw.
        assert!(!nearby_chlorophyte(&w, 5, 200, &mut rng));
        assert_eq!(rng, pristine, "the border short-circuit draws nothing");
        // One source: the roll decides (the mirror consumes one draw).
        let mut w2 = world();
        put(&mut w2, 200, 200, 211);
        let mut rng2 = UnifiedRandom::new(5);
        let mut mirror = rng2.clone();
        let got = nearby_chlorophyte(&w2, 200, 200, &mut rng2);
        let roll = mirror.next_range(-1, 4);
        assert_eq!(got, (roll as f64) < 1.0);
        assert_eq!(rng2, mirror, "exactly one Next(-1, 4)");
    }
}
