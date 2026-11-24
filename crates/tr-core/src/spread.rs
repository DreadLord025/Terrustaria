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
        convert_chlorophyte_cell(world, x, y, 10);
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
                convert_chlorophyte_cell(world, nx, ny, 8);
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
                    convert_chlorophyte_cell(world, nx, ny, 8);
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
                convert_chlorophyte_cell(world, nx, ny, 9);
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

// W4-d103 POST FINAL PF-3 keep-one (research/12c, 12318e505): d103's
// `convert_purge_cell` (the P8 tiles-only approximation) DROPPED at the
// merge — f25 C5's `convert_chlorophyte_cell` is the survivor (the same
// `WorldGen.cs:55665-55763` window, plus the `Convert_ActuallyConvertTile`
// active+diff gate `:56314`, the `try_kill_invalid_tree_above` cross-kill
// and the `reframe_area` superset). The three consumers rewired: the
// `:69658`/`:69686` type-8 growth arms + the `:447` type-9 purge pick +
// the `:292` type-10 ChlorophyteDefense. Draw-neutral: none of the three
// arms draws caller RNG (the reframe rolls ride `world.frame_rng`), so
// the P8 stream witnesses stay valid.

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

/// `WorldGen.TileIsExposedToAir(x, y)` (`WorldGen.cs:54782-54800`, W4-f25
/// helper C1): the 3x3 box around the cell holds an inactive tile, a
/// non-solid tile or a platform — any of the three reads "exposed to air".
/// The `InWorld(2)` gate refuses the border band. WIRED through the C4
/// type-7 purify cell's sand-family dirt arm (`:55512`) and the closeout's
/// Convert 15/18/19 cells (`:56031`/`:56171`/`:56222`); the c32
/// biome-dungeon rooms (`BiomeDungeonRoom.cs:230/:246`) are the other C#
/// callers (their bodies are c32's lane — the room port, not the cells).
pub fn tile_is_exposed_to_air(world: &World, x: i32, y: i32) -> bool {
    // `:54784-54787`.
    if !in_world(world, x, y, 2) {
        return false;
    }
    // `:54788-54798`: `tile != null && (!tile.active() ||
    // !Main.tileSolid[tile.type] || TileID.Sets.Platforms[tile.type])`.
    for i in (x - 1)..=(x + 1) {
        for j in (y - 1)..=(y + 1) {
            if let Some(t) = world.tile(i, j) {
                if !t.active()
                    || !crate::tiledata::TILE_SOLID.has(t.kind)
                    || crate::tiledata::SET_PLATFORMS.has(t.kind)
                {
                    return true;
                }
            }
        }
    }
    false
}

/// `WorldGen.BlockBelowMakesSandConvertIntoHardenedSand(i, j)`
/// (`WorldGen.cs:86562-86583`, W4-f25 helper C2): the cell BELOW is inactive
/// or a non-solid kind — sand converting over such a floor hardens (397)
/// instead of staying loose (53). The bottom row refuses (no cell below).
/// WIRED through the C4 type-5 desertify cell's sand arm (`:55386`) and the
/// closeout's Convert 16 sand arm (`:56064`); `BiomeDungeonRoom.cs:264/:280`
/// (c32's lane — the room bodies) are the other C# callers.
pub fn block_below_makes_sand_convert_into_hardened_sand(
    world: &World,
    i: i32,
    j: i32,
) -> bool {
    // `:86565-86568`: `j >= Main.maxTilesY - 1` → false.
    if j >= world.height - 1 {
        return false;
    }
    // `:86569-86573`: the null tile → false.
    let Some(tile) = world.tile(i, j + 1) else {
        return false;
    };
    // `:86574-86581`: `!tile.nactive()` → true; else the in-range non-solid
    // kind → true (the generated TILE_SOLID set carries the range gate).
    !tile.active() || !crate::tiledata::TILE_SOLID.has(tile.kind)
}

/// `WorldGen.Convert(i, j, conversionType, size)` (`WorldGen.cs:55067-55083`):
/// the square walk with the Manhattan `< 6` gate. The port carries the type-0
/// (cleanse) cell only — the spray's conversion type.
/// `rng` is the ACTIVE `genRand` stream of the caller's context (W4-g83
/// contract §2): the Clentaminator spray runs under the `UpdateProjectiles`
/// swap (`Main.cs:17775`), so the thorn/tree kills' framing dispatch draws
/// its seaweed tip re-rolls on that stream.
pub fn convert_cleanse(
    world: &mut World,
    i: i32,
    j: i32,
    size: i32,
    rng: &mut UnifiedRandom,
) {
    for k in (i - size)..=(i + size) {
        for l in (j - size)..=(j + size) {
            if in_world(world, k, l, 1) && (k - i).abs() + (l - j).abs() < 6 {
                convert_cleanse_cell(world, k, l, rng);
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
fn convert_cleanse_cell(world: &mut World, x: i32, y: i32, rng: &mut UnifiedRandom) {
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
            try_kill_invalid_tree_above(world, x, y, new_kind, rng);
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = new_kind;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
    } else if kind == 32 || kind == 352 {
        // `:55656-55662` — the thorn bushes die (the netMode arm is MP-only).
        // W4-g83: the cascade rides the caller's ACTIVE genRand stream.
        let _ = crate::edit::kill_tile(world, x, y, false, rng);
    }
}

// ---------------------------------------------------------------------------
// `WorldGen.Convert` — the powder entry (W4-d1, the aiStyle-6 arm's
// `WorldGen.Convert(x, y, 11/1/4, 1)` calls, `Projectile.cs:24193-24201`).
// Conversion types 1 (corruption) / 4 (crimson) / 11 (purify) join the
// type-0 cleanse port above; the cell arms are the C#'s own else-if ladders.

/// `TileID.Sets.Conversion.Thorn` (`TileID.cs:95`).
const CONV_THORN: [u16; 4] = [32, 352, 69, 655];

/// `TileID.Sets.Conversion.Torch(tile, conversionType)` (`TileID.cs:42-79`)
/// for the powder types: type 11 accepts the corrupt/crimson/hallow styles
/// 18-20 only (`:57-62`); types 1/4 take the default arm's style set
/// (`:63-77`). The type-0 arm lives in [`convert_cleanse_cell`].
fn torch_convertible(style: i32, conversion_type: u8) -> bool {
    match conversion_type {
        11 => (18..=20).contains(&style),
        _ => matches!(style, 0 | 9 | 16 | 18 | 19 | 20 | 21 | 22),
    }
}

/// `WorldGen.Convert(i, j, conversionType, size)` (`WorldGen.cs:55067-55083`)
/// for the powder types 1/4/11: the square walk with the Manhattan `< 6`
/// gate, each cell through [`convert_powder_cell`]. The aiStyle-6 arm calls
/// it with size 1 (`Projectile.cs:24193-24201`); size 0 isolates one cell.
/// Only types 1/4/11 are ported here — type 0 rides [`convert_cleanse`],
/// the other spray types have no ported callers yet (a no-op, not a
/// mis-conversion).
pub fn convert_sized(world: &mut World, i: i32, j: i32, conversion_type: u8, size: i32) {
    if !matches!(conversion_type, 1 | 4 | 11) {
        return;
    }
    for k in (i - size)..=(i + size) {
        for l in (j - size)..=(j + size) {
            if in_world(world, k, l, 1) && (k - i).abs() + (l - j).abs() < 6 {
                convert_powder_cell(world, k, l, conversion_type);
            }
        }
    }
}

/// `Convert`'s type-1/4/11 cell arms (`WorldGen.cs:55085-55175` crimson,
/// `:55261-55336` corruption, `:55764-55882` purify): the wall chain, then
/// the torch / moss-stone / jungle-grass / grass / ice / sand / hardened-
/// sand / sandstone / thorn tile ladder in source order.
/// `Convert_ActuallyConvertTile`/`Wall`/`Torch` (`:56287-56343`) write only
/// when the cell differs and re-frame through `SquareTileFrame` /
/// `SquareWallFrame` — the port's [`crate::framing::reframe_area`] superset
/// (the `convert_cleanse_cell` deviation).
///
/// Deviations (ANALYSIS "The powder/fertilizer cloud (W4-d1)"): the type-11
/// wall 69/70/81 surface roll draws `WorldGen.genRand.Next(10)` (`:55783`)
/// — the port has no runtime genRand stream, so the shared
/// `convert_wall_by_case(wall, 11, .., None)` takes the no-rng 64 (the
/// `convert_cleanse_cell` precedent). The `RainbowSandAndBlackSandWalls`
/// paint arms (`:56302-56306`/`:56319-56323`/`:56335-56339`) gate on
/// `Main.onlyShimmerOceanWorlds`, a mode the port does not generate
/// (`worldgen/finish_seeds.rs`) — skipped.
fn convert_powder_cell(world: &mut World, x: i32, y: i32, conversion_type: u8) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    let (kind, wall, active) = (t.kind, t.wall, t.active());
    // `:55087-55097` — the InWorld gate rides `convert_sized`; the empty
    // cell (no tile, no wall) converts nothing.
    if !active && wall == 0 {
        return;
    }
    // The wall half. Types 1/4 ride the shared flat ladder
    // (`convert_wall_by_case`'s Grass/Stone/HardenedSand/Sandstone/NewWall
    // chain, `:55101-55134`/`:55262-55295`). Type 11's switch no-op cases
    // 61/185/204-207 (`:55836-55842`) break WITHOUT converting — the shared
    // arm's Stone tail would otherwise catch 61/185, so they are gated out
    // here (the `convert_cleanse_cell` precedent).
    let new_wall = match conversion_type {
        11 => {
            if matches!(wall, 61 | 185 | 204 | 205 | 206 | 207) {
                None
            } else {
                crate::growth::convert_wall_by_case(wall, 11, y, world.world_surface, None)
            }
        }
        1 | 4 => {
            crate::growth::convert_wall_by_case(wall, conversion_type, y, world.world_surface, None)
        }
        _ => None,
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
    // The tile half — the else-if ladder in source order. The torch arm
    // short-circuits the ladder even when the style already matches
    // (`Convert_ActuallyConvertTorch`'s `num != newStyle` gate, `:56298`).
    let torch_style = if active && kind == 4 {
        let num = world.tile(x, y).map_or(0, |t| t.frame_y as i32 / 22);
        if torch_convertible(num, conversion_type) {
            Some(match conversion_type {
                11 => 0i16, // `:55848`
                1 => 18,    // `:55301`
                _ => 19,    // `:55140`
            })
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
    let new_kind: Option<u16> = match conversion_type {
        // The purify ladder (`:55850-55881`): jungle grass → 60, golf grass
        // → 477, grass → 2, stone → 1, ice → 161, sand → 53, hardened sand
        // → 397, sandstone → 396. NO thorn/mushroom arm.
        11 => {
            if CONV_JUNGLE_GRASS.contains(&kind) && kind != 60 {
                Some(60)
            } else if crate::tiledata::SET_GOLF_GRASS.has(kind) && kind != 477 {
                Some(477)
            } else if CONV_GRASS.contains(&kind) && kind != 2 && kind != 477 {
                Some(2)
            } else if crate::tiledata::SET_CONVERSION_STONE.has(kind) && kind != 1 {
                Some(1)
            } else if crate::tiledata::SET_ICES.has(kind) && kind != 161 {
                // `TileID.Sets.Ices` = {161, 200, 163, 164} (`TileID.cs:559`).
                Some(161)
            } else if crate::tiledata::SET_CONVERSION_SAND.has(kind) && kind != 53 {
                Some(53)
            } else if crate::tiledata::SET_CONVERSION_HARDENED_SAND.has(kind) && kind != 397 {
                Some(397)
            } else if crate::tiledata::SET_CONVERSION_SANDSTONE.has(kind) && kind != 396 {
                Some(396)
            } else {
                None
            }
        }
        // The evil ladders (`:55138-55173` crimson / `:55299-55334`
        // corruption): the moss OR stone arm first, then jungle grass,
        // grass, ice, sand, hardened sand, sandstone, thorn.
        1 | 4 => {
            let corruption = conversion_type == 1;
            if (crate::tiledata::SET_CONVERSION_MOSS.has(kind)
                || crate::tiledata::SET_CONVERSION_STONE.has(kind))
                && kind != if corruption { 25 } else { 203 }
            {
                Some(if corruption { 25 } else { 203 })
            } else if CONV_JUNGLE_GRASS.contains(&kind) && kind != if corruption { 661 } else { 662 }
            {
                Some(if corruption { 661 } else { 662 })
            } else if CONV_GRASS.contains(&kind) && kind != if corruption { 23 } else { 199 } {
                Some(if corruption { 23 } else { 199 })
            } else if crate::tiledata::SET_ICE.has(kind) && kind != if corruption { 163 } else { 200 }
            {
                Some(if corruption { 163 } else { 200 })
            } else if crate::tiledata::SET_CONVERSION_SAND.has(kind)
                && kind != if corruption { 112 } else { 234 }
            {
                Some(if corruption { 112 } else { 234 })
            } else if crate::tiledata::SET_CONVERSION_HARDENED_SAND.has(kind)
                && kind != if corruption { 398 } else { 399 }
            {
                Some(if corruption { 398 } else { 399 })
            } else if crate::tiledata::SET_CONVERSION_SANDSTONE.has(kind)
                && kind != if corruption { 400 } else { 401 }
            {
                Some(if corruption { 400 } else { 401 })
            } else if CONV_THORN.contains(&kind) && kind != if corruption { 32 } else { 352 } {
                Some(if corruption { 32 } else { 352 })
            } else {
                None
            }
        }
        _ => None,
    };
    if let Some(new_kind) = new_kind {
        // `Convert_ActuallyConvertTile` (`:56312-56327`): active, different,
        // the tree-above validity check, then the write + re-frame.
        if active && kind != new_kind {
            try_kill_invalid_tree_above_legacy(world, x, y, new_kind);
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = new_kind;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
    }
}

/// `Convert`'s type-3 cell (`WorldGen.cs:55336-55359`), the glowing-mushroom
/// flavour (W4-g42 G7): the wall arm (`WallID.Sets.
/// CanBeConvertedToGlowingMushroom` = {64, 67, 15, 247}, `WallID.cs:88`) →
/// 80 through `Convert_ActuallyConvertWall`'s non-zero+different gate
/// (`:56329-56343`), then the tile else-if ladder — the torch style-22 arm
/// (`TileID.Sets.Conversion.Torch(tile, 3)`'s default switch {0, 9, 16, 18,
/// 19, 20, 21, 22}, `TileID.cs:63-77`), the jungle grass 60 → 70
/// (`Convert_ActuallyConvertTile`'s active+different + tree-above kill,
/// `:56312-56327`) and the thorn kill (`Conversion.Thorn` = {32, 352, 69,
/// 655}, `TileID.cs:95`). The type-3 arm draws NO RNG (the wall-80 depth
/// rolls live in the type-0 cleanse arm only). The `SquareTileFrame` /
/// `SquareWallFrame` calls ride the port's `reframe_area` superset (the
/// `convert_cleanse_cell` precedent); the netMode sends are MP-only.
pub fn convert_cell_mushroom(world: &mut World, x: i32, y: i32) {
    // `!InWorld(i2, j2, 1)` (`:55087`) — the size-0 walk's only cell gate.
    if !in_world(world, x, y, 1) {
        return;
    }
    let Some(t) = world.tile(x, y) else {
        return;
    };
    let (kind, wall, active) = (t.kind, t.wall, t.active());
    // `!theTile.active() && wall <= 0` (`:55092-55095`).
    if !active && wall == 0 {
        return;
    }
    // The wall half (`:55337-55340`): the shared type-3 ladder row.
    if let Some(new_wall) = crate::growth::convert_wall_by_case(wall, 3, y, world.world_surface, None)
    {
        // `Convert_ActuallyConvertWall`: non-zero, different.
        if wall != 0 && wall != new_wall {
            if let Some(t) = world.tile_mut(x, y) {
                t.wall = new_wall;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
    }
    // The tile half (`:55345-55358`) — the else-if ladder in source order.
    if active && kind == 4 {
        // `Conversion.Torch(tile, 3)` — the default-switch style set.
        let num = world.tile(x, y).map_or(0, |t| t.frame_y as i32 / 22);
        if matches!(num, 0 | 9 | 16 | 18 | 19 | 20 | 21 | 22) {
            // `Convert_ActuallyConvertTorch` (`:56287-56310`): style 22, the
            // type-0 mushroom quirk does not apply (conversionType 3).
            if num != 22 {
                if let Some(t) = world.tile_mut(x, y) {
                    t.frame_y = 22 * 22;
                }
                crate::framing::reframe_area(world, x, y, 1, false);
            }
            return;
        }
        // A torch outside the set falls through the C# else-if chain to the
        // 60/thorn tests (which a type-4 tile never matches) — a no-op.
        return;
    }
    if kind == 60 {
        // `Convert_ActuallyConvertTile`: active, different, the tree-above
        // validity check, then the write + re-frame.
        if active && kind != 70 {
            try_kill_invalid_tree_above_legacy(world, x, y, 70);
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = 70;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
        return;
    }
    if matches!(kind, 32 | 352 | 69 | 655) {
        // `Conversion.Thorn[type]` → `KillTile` (`:55352-55357`; the netMode
        // send is MP-only). The cleanse arm's kill precedent.
        let _ = crate::edit::kill_tile_legacy(world, x, y, false);
    }
}

/// `WorldGen.Convert(i, j, 3, size)` (`WorldGen.cs:55067-55083`) — the sized
/// mushroom walk for the Clentaminator's DarkBlueSolution spray (W4-g42
/// GROWTH SEAL: the FOURTH Convert(3) call site, `Projectile.cs:26834` —
/// the aiStyle-31 ladder `:26805-26808` maps projectile 148 → type 3, the
/// shoot math `Player.cs:53487-53490`: Clentaminator 779 shoot 145 +
/// DarkBlueSolution 783 shoot 3 = 148). The SAME square walk + Manhattan
/// `< 6` gate as [`convert_cleanse`] — only the per-cell arm differs
/// ([`convert_cell_mushroom`], G7, RNG-free: the type-3 wall/torch/60→70/
/// thorn ladder draws nothing).
///
/// MERGER CONTRACT (the projectile lane — d18 tip 8248a54b rowed the seven
/// solution kinds 146-149/1015-1017; f25 tip d587b128d wired the `ai_spray`
/// dispatch to `match conversion { 0 => convert_cleanse, 2 =>
/// convert_sized_hallow, 5|6|7 => convert_sized_spray, _ => {} }`): the
/// merger adds ONE arm — `3 => tr_core::spread::convert_sized_mushroom(
/// world, tx, ty, size)` — into f25's match (types 1/4 stay d1's
/// `convert_sized`). The 133 cap / `ai[0]++` / spin already run for every
/// kind — this arm is the only missing piece. f25's 12g ledger already
/// records "g42 owns 3".
pub fn convert_sized_mushroom(world: &mut World, i: i32, j: i32, size: i32) {
    for k in (i - size)..=(i + size) {
        for l in (j - size)..=(j + size) {
            if in_world(world, k, l, 1) && (k - i).abs() + (l - j).abs() < 6 {
                convert_cell_mushroom(world, k, l);
            }
        }
    }
}

/// `TryConvertingOrKillingTreesAboveIfTheyWouldBecomeInvalid`
/// (`WorldGen.cs:54802-54899`): the tree trunk (or palm 323) directly above
/// the converted cell dies when the new floor no longer fits its ground
/// test. The gem/vanity/ash profiles (`GrowTreeSettings.Profiles.
/// TryGetFromTreeId`, `:3890-3928`) override the vanilla test; the
/// `ConvertTreeAndGround` cross-conversion half is the documented skip.
/// [`try_kill_invalid_tree_above`] with the seed stand-in — the conversion
/// helpers below predate the W4-g83 stream threading (legacy-path stand-in).
fn try_kill_invalid_tree_above_legacy(world: &mut World, x: i32, y: i32, new_floor: u16) {
    let mut gen_rng = crate::UnifiedRandom::new(world.seed as i32);
    try_kill_invalid_tree_above(world, x, y, new_floor, &mut gen_rng);
}

fn try_kill_invalid_tree_above(
    world: &mut World,
    x: i32,
    y: i32,
    new_floor: u16,
    rng: &mut UnifiedRandom,
) {
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
    let sand_side = crate::tiledata::SET_CONVERSION_SAND.has(new_floor);
    let mut fits = true;
    match tree {
        5 => fits = grass_side,
        323 => fits = sand_side,
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
        // W4-g83: the cascade rides the caller's ACTIVE genRand stream.
        let _ = crate::edit::kill_tile(world, x, y - 1, false, rng);
    }
}

// ---------------------------------------------------------------------------
// `WorldGen.Convert` — the hallow cell (W4-f25 helper C3, conversion type 2).
// WIRING: the aiStyle-31 Clentaminator's 146 Blue Solution spray
// (`projectile_ai_misc::ai_spray`), the holy-water death splash
// (`projectile_ai_thrown::water_death`, `Projectile.cs:77635`) and
// SpreadGrass's 109 dispatch (`growth/grass.rs`, `:75218`, tiles-only via
// `convert_hallow_cell_halves`) call these cells. The gen-time consumers —
// DoHallowOnSurface's map approximation (`micro_surface.rs`), GERunner /
// initializeHardMode (d103's verbatim rewrite, pushed) and ConvertSkyIslands
// (unported) — keep their own copies; the ChlorophyteBolt death is unported.

/// `TileID.Sets.Conversion.Torch(tile, conversionType)`'s default arm
/// (`TileID.cs:63-77`): the styles every non-0/non-11 conversion type accepts.
fn torch_convertible_default(style: i32) -> bool {
    matches!(style, 0 | 9 | 16 | 18 | 19 | 20 | 21 | 22)
}

/// `WorldGen.Convert(i, j, 2, size)` (`WorldGen.cs:55067-55083` walk +
/// `:55176-55260` cell): the square walk with the Manhattan `< 6` gate, each
/// cell through [`convert_hallow_cell`] (both halves). The callers that pass
/// `tiles: false`/`walls: false` ride [`convert_hallow_cell_halves`] directly.
pub fn convert_sized_hallow(world: &mut World, i: i32, j: i32, size: i32) {
    for k in (i - size)..=(i + size) {
        for l in (j - size)..=(j + size) {
            if in_world(world, k, l, 1) && (k - i).abs() + (l - j).abs() < 6 {
                convert_hallow_cell(world, k, l);
            }
        }
    }
}

/// `Convert`'s type-2 cell arm, both halves (`WorldGen.cs:55176-55260`) —
/// the single-cell overload's default `tiles: true, walls: true`.
pub fn convert_hallow_cell(world: &mut World, x: i32, y: i32) {
    convert_hallow_cell_halves(world, x, y, true, true);
}

/// `Convert`'s type-2 single-cell overload (`WorldGen.cs:55085-55097` +
/// `:55176-55260`): the `InWorld(1)` gate, then the wall chain (the flat
/// Grass/Stone/HardenedSand/Sandstone/NewWall ladder → 70/28/219/222/200-203,
/// `:55179-55210`, gated on `walls`), then the torch → style 20 / moss-or-
/// stone → 117 / golf grass → 492 / grass → 109 / ice → 164 / sand → 116 /
/// hardened sand → 402 / sandstone → 403 / thorn-kill ladder in source order
/// and the INDEPENDENT mud arm (`:55256-55259`: a mud 59 cell with a hallow-
/// grass 109 4-neighbour → dirt 0), both gated on `tiles` (the C#'s
/// `if (!tiles) break;` at `:55212` sits before them).
/// `Convert_ActuallyConvertTile`/`Wall`/`Torch` (`:56287-56343`) write only
/// when the cell differs and re-frame — the port's `reframe_area` superset
/// (the `convert_cleanse_cell` deviation). The genRand-free wall ladder takes
/// `None` (type 2 draws nothing); the `RainbowSandAndBlackSandWalls` paint
/// arms are the d1-documented skip.
pub fn convert_hallow_cell_halves(world: &mut World, x: i32, y: i32, tiles: bool, walls: bool) {
    // `:55087` — the InWorld(1) gate + the both-halves-off refusal.
    if !in_world(world, x, y, 1) || (!tiles && !walls) {
        return;
    }
    let Some(t) = world.tile(x, y).copied() else {
        return;
    };
    let (kind, wall, active) = (t.kind, t.wall, t.active());
    // `:55094-55097` — the empty cell converts nothing.
    if !active && wall == 0 {
        return;
    }
    // The wall half (`:55177-55211`): the shared type-2 flat ladder.
    if walls {
        if let Some(new_wall) =
            crate::growth::convert_wall_by_case(wall, 2, y, world.world_surface, None)
        {
            // `Convert_ActuallyConvertWall` (`:56329-56343`): non-zero, different.
            if wall != 0 && wall != new_wall {
                if let Some(t) = world.tile_mut(x, y) {
                    t.wall = new_wall;
                }
                crate::framing::reframe_area(world, x, y, 1, false);
            }
        }
    }
    // `:55212-55215` — the tiles-off break skips the whole tile half.
    if !tiles {
        return;
    }
    // The tile half (`:55216-55255`) — the else-if ladder in source order.
    if active && kind == 4 && torch_convertible_default(t.frame_y as i32 / 22) {
        // `Convert_ActuallyConvertTorch(.., 20)` (`:55218`).
        let cur = t.frame_y as i32 / 22;
        if cur != 20 {
            if let Some(t) = world.tile_mut(x, y) {
                t.frame_y = 20 * 22;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
    } else {
        // `Main.tileMoss[type] || Conversion.Stone[type]` (`:55220`) — the
        // generated TILE_MOSS set is Main.cs:7007-7047's eleven ids.
        let new_kind: Option<u16> = if (crate::tiledata::TILE_MOSS.has(kind)
            || crate::tiledata::SET_CONVERSION_STONE.has(kind))
            && kind != 117
        {
            Some(117)
        } else if crate::tiledata::SET_GOLF_GRASS.has(kind) && kind != 492 {
            Some(492)
        } else if CONV_GRASS.contains(&kind) && kind != 109 && kind != 492 {
            Some(109)
        } else if crate::tiledata::SET_ICE.has(kind) && kind != 164 {
            Some(164)
        } else if crate::tiledata::SET_CONVERSION_SAND.has(kind) && kind != 116 {
            Some(116)
        } else if crate::tiledata::SET_CONVERSION_HARDENED_SAND.has(kind) && kind != 402 {
            Some(402)
        } else if crate::tiledata::SET_CONVERSION_SANDSTONE.has(kind) && kind != 403 {
            Some(403)
        } else {
            None
        };
        if let Some(new_kind) = new_kind {
            // `Convert_ActuallyConvertTile` (`:56312-56327`).
            if active && kind != new_kind {
                try_kill_invalid_tree_above_legacy(world, x, y, new_kind);
                if let Some(t) = world.tile_mut(x, y) {
                    t.kind = new_kind;
                }
                crate::framing::reframe_area(world, x, y, 1, false);
            }
        } else if CONV_THORN.contains(&kind) {
            // `:55248-55255` — the thorn bushes die (the netMode arm is MP).
            let _ = crate::edit::kill_tile_legacy(world, x, y, false);
        }
    }
    // The independent mud arm (`:55256-55259`): the SNAPSHOT type 59 with a
    // live 109 4-neighbour → dirt 0 through the same actually-convert shape.
    if kind == 59
        && [
            world.tile(x - 1, y),
            world.tile(x + 1, y),
            world.tile(x, y - 1),
            world.tile(x, y + 1),
        ]
        .iter()
        .any(|t| t.is_some_and(|t| t.kind == 109))
    {
        if active && kind != 0 {
            try_kill_invalid_tree_above_legacy(world, x, y, 0);
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = 0;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
    }
}

// ---------------------------------------------------------------------------
// `WorldGen.Convert` — the spray cells (W4-f25 helper C4, conversion types
// 5/6/7: desertify / snow / purify). WIRING: the aiStyle-31 Clentaminator's
// 1015 Sand / 1016 Snow / 1017 Purple Solution sprays
// (`projectile_ai_misc::ai_spray`, `Projectile.cs:26813-26823`) call
// `convert_sized_spray`. The C1 air-exposure + C2 hardening-floor tests ride
// the type-7/5 dirt/sand arms inside the cell (their only ported consumers).

/// `WorldGen.Convert(i, j, conversionType, size)` for the spray types 5/6/7
/// (`WorldGen.cs:55067-55083` walk): the square walk with the Manhattan
/// `< 6` gate, each cell through [`convert_spray_cell`].
pub fn convert_sized_spray(world: &mut World, i: i32, j: i32, conversion_type: u8, size: i32) {
    if !matches!(conversion_type, 5 | 6 | 7) {
        return;
    }
    for k in (i - size)..=(i + size) {
        for l in (j - size)..=(j + size) {
            if in_world(world, k, l, 1) && (k - i).abs() + (l - j).abs() < 6 {
                convert_spray_cell(world, k, l, conversion_type);
            }
        }
    }
}

/// `Convert`'s type-5/6/7 cell arms (`WorldGen.cs:55363-55408` desertify,
/// `:55409-55445` snow, `:55446-55526` purify): the wall chain, then the
/// torch / tile ladder in source order.
///
/// Deviations (the `convert_cleanse_cell` / d1 precedent): the wall halves
/// ride the shared `convert_wall_by_case` (types 5/6 verbatim; type 7's
/// `:55465` dirt arm there carries only the Dirt set — the HardenedSand/Snow
/// union half is pre-gated here, order-equivalent: the union is disjoint from
/// the natural Stone/Ice/Sandstone arm, the 262/274 switch and the NewWall
/// sets). The `RainbowSandAndBlackSandWalls` paint arms are the documented
/// skip; the re-frame is the `reframe_area` superset.
pub fn convert_spray_cell(world: &mut World, x: i32, y: i32, conversion_type: u8) {
    let Some(t) = world.tile(x, y).copied() else {
        return;
    };
    let (kind, wall, active) = (t.kind, t.wall, t.active());
    // `:55087-55097` — the empty cell converts nothing.
    if !active && wall == 0 {
        return;
    }
    // The wall half. Type 7's `:55465` union pre-gate (see the fn doc).
    let new_wall = if conversion_type == 7
        && (crate::growth::conversion_hardened_sand_walls_contains(wall)
            || crate::growth::conversion_snow_walls_contains(wall))
        && wall != 2
    {
        Some(2u16)
    } else {
        crate::growth::convert_wall_by_case(wall, conversion_type, y, world.world_surface, None)
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
    // The torch arm (`:55379`/`:55425`/`:55493`): the default style set,
    // styles 16 / 9 / 0 per type.
    if active && kind == 4 && torch_convertible_default(t.frame_y as i32 / 22) {
        let style = match conversion_type {
            5 => 16i16,
            6 => 9,
            _ => 0,
        };
        let cur = t.frame_y as i32 / 22;
        if cur != style as i32 {
            if let Some(t) = world.tile_mut(x, y) {
                t.frame_y = style * 22;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
        return;
    }
    // The tile ladders in source order. `Conversion.Dirt` = {0}
    // (`TileID.cs:88`), `Conversion.Snow` = {147} (`:89`).
    let dirt = kind == 0;
    let snow = CONV_SNOW.contains(&kind);
    let moss_or = |k: u16| crate::tiledata::TILE_MOSS.has(k);
    let new_kind: Option<u16> = match conversion_type {
        // Desertify (`:55383-55399`): grass/sand/snow/dirt → 53 (397 over a
        // hardening floor), hardened sand → 397, moss/stone/ice/sandstone →
        // 396.
        5 => {
            if (CONV_GRASS.contains(&kind)
                || crate::tiledata::SET_CONVERSION_SAND.has(kind)
                || snow
                || dirt)
                && kind != 53
            {
                Some(if block_below_makes_sand_convert_into_hardened_sand(world, x, y) {
                    397
                } else {
                    53
                })
            } else if crate::tiledata::SET_CONVERSION_HARDENED_SAND.has(kind) && kind != 397 {
                Some(397)
            } else if (moss_or(kind)
                || crate::tiledata::SET_CONVERSION_STONE.has(kind)
                || crate::tiledata::SET_ICE.has(kind)
                || crate::tiledata::SET_CONVERSION_SANDSTONE.has(kind))
                && kind != 396
            {
                Some(396)
            } else {
                None
            }
        }
        // Snow (`:55429-55436`): grass/sand/hardened/snow/dirt → 147,
        // moss/stone/ice/sandstone → 161.
        6 => {
            if (CONV_GRASS.contains(&kind)
                || crate::tiledata::SET_CONVERSION_SAND.has(kind)
                || crate::tiledata::SET_CONVERSION_HARDENED_SAND.has(kind)
                || snow
                || dirt)
                && kind != 147
            {
                Some(147)
            } else if (moss_or(kind)
                || crate::tiledata::SET_CONVERSION_STONE.has(kind)
                || crate::tiledata::SET_ICE.has(kind)
                || crate::tiledata::SET_CONVERSION_SANDSTONE.has(kind))
                && kind != 161
            {
                Some(161)
            } else {
                None
            }
        }
        // Purify (`:55497-55517`): stone/ice/sandstone → 1 (NO moss arm),
        // golf grass → 477, grass → 2, sand/hardened/snow/dirt → 0 (2 when
        // exposed to air).
        _ => {
            if (crate::tiledata::SET_CONVERSION_STONE.has(kind)
                || crate::tiledata::SET_ICE.has(kind)
                || crate::tiledata::SET_CONVERSION_SANDSTONE.has(kind))
                && kind != 1
            {
                Some(1)
            } else if crate::tiledata::SET_GOLF_GRASS.has(kind) && kind != 477 {
                Some(477)
            } else if CONV_GRASS.contains(&kind) && kind != 2 && kind != 477 {
                Some(2)
            } else if (crate::tiledata::SET_CONVERSION_SAND.has(kind)
                || crate::tiledata::SET_CONVERSION_HARDENED_SAND.has(kind)
                || snow
                || dirt)
                && kind != 0
            {
                Some(if tile_is_exposed_to_air(world, x, y) { 2 } else { 0 })
            } else {
                None
            }
        }
    };
    if let Some(new_kind) = new_kind {
        // `Convert_ActuallyConvertTile` (`:56312-56327`).
        if active && kind != new_kind {
            try_kill_invalid_tree_above_legacy(world, x, y, new_kind);
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = new_kind;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
    } else if CONV_THORN.contains(&kind) && kind != 69 {
        // `:55400-55407`/`:55437-55444`/`:55518-55525` — the thorn kill
        // (the 69 corrupt-thorn exemption is the C#'s own; the netMode arm
        // is MP-only).
        let _ = crate::edit::kill_tile_legacy(world, x, y, false);
    }
}

// ---------------------------------------------------------------------------
// `WorldGen.Convert` — the chlorophyte cells (W4-f25 helper C5, conversion
// types 8/9/10). WIRED at the merge (W4-d103 POST FINAL PF-3 keep-one,
// research/12c): hardUpdateWorld's chlorophyte arms (`:69658`/`:69686`
// type 8, `:69697` type 9) and ChlorophyteDefense's remix-only type 10
// (`:69460`) ride `convert_chlorophyte_cell` — d103's tiles-only
// `convert_purge_cell` approximation dropped (the diff gate / the
// tree cross-kill / the reframe superset are the survivor's).

/// `WorldGen.Convert(i, j, conversionType, size)` for the chlorophyte types
/// 8/9/10 (`WorldGen.cs:55067-55083` walk): the square walk with the
/// Manhattan `< 6` gate, each cell through [`convert_chlorophyte_cell`].
/// The C# call sites all use the single-cell overload (size 0).
pub fn convert_sized_chlorophyte(
    world: &mut World,
    i: i32,
    j: i32,
    conversion_type: u8,
    size: i32,
) {
    if !matches!(conversion_type, 8 | 9 | 10) {
        return;
    }
    for k in (i - size)..=(i + size) {
        for l in (j - size)..=(j + size) {
            if in_world(world, k, l, 1) && (k - i).abs() + (l - j).abs() < 6 {
                convert_chlorophyte_cell(world, k, l, conversion_type);
            }
        }
    }
}

/// `Convert`'s type-8/9/10 cell arms (`WorldGen.cs:55665-55670` /
/// `:55671-55720` / `:55721-55763`): tiles-only (no wall half in any of the
/// three), no torch arm.
/// - 8: the mud 59 / jungle grass 60 → the chlorophyte 211.
/// - 9: the FULL grass set (2/23/109/199/477/492/661/662) → 60, dirt 0 → 59,
///   the evil stone/sand/hardened/sandstone pairs → their pure kinds, the
///   six thorn/evil kinds (24/32/201/205/352/636) die.
/// - 10: the same WITHOUT the pure-grass/dirt rows (only the four infection
///   grasses 23/199/661/662 → 60) — the ChlorophyteDefense remix conversion.
/// `Convert_ActuallyConvertTile` (`:56312-56327`) writes only when the cell
/// differs; the re-frame is the `reframe_area` superset (the cleanse/d1
/// precedent).
pub fn convert_chlorophyte_cell(world: &mut World, x: i32, y: i32, conversion_type: u8) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    let (kind, wall, active) = (t.kind, t.wall, t.active());
    // `:55087-55097` — the empty cell converts nothing.
    if !active && wall == 0 {
        return;
    }
    // Type 8 (`:55665-55670`): `tiles && (type == 59 || type == 60)`.
    if conversion_type == 8 {
        if matches!(kind, 59 | 60) && active && kind != 211 {
            try_kill_invalid_tree_above_legacy(world, x, y, 211);
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = 211;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
        return;
    }
    // Types 9/10 (`:55676-55719` / `:55726-55762`): the switch on type.
    let new_kind: Option<u16> = match kind {
        // The type-9-only pure rows (`:55678-55690`): the plain/golf/hallow
        // grasses and the dirt.
        2 | 109 | 477 | 492 if conversion_type == 9 => Some(60),
        0 if conversion_type == 9 => Some(59),
        // The shared infection-grass row (`:55679`/`:55684-55685` vs
        // `:55728-55731`).
        23 | 199 | 661 | 662 => Some(60),
        25 | 203 => Some(1),
        112 | 234 => Some(53),
        398 | 399 => Some(397),
        400 | 401 => Some(396),
        _ => None,
    };
    if let Some(new_kind) = new_kind {
        if active && kind != new_kind {
            try_kill_invalid_tree_above_legacy(world, x, y, new_kind);
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = new_kind;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
    } else if matches!(kind, 24 | 32 | 201 | 205 | 352 | 636) {
        // `:55707-55718` / `:55750-55761` — the thorn/evil kinds die (the
        // netMode arm is MP-only).
        let _ = crate::edit::kill_tile_legacy(world, x, y, false);
    }
}

// ---------------------------------------------------------------------------
// `WorldGen.Convert` — the dungeon-biome cells (W4-f25 closeout, conversion
// types 15/16/18/19: glowing-mushroom / desert / pure / jungle). The C#
// switch arms (`:56008-56040` / `:56041-56086` / `:56124-56189` /
// `:56190-56231`) exist but have ZERO call sites in the whole decompilation
// (the biome-dungeon rooms use `DungeonUtils.ChangeTileType` directly,
// `BiomeDungeonRoom.cs:232/:236/:248/:252/:266/:270/:282/:286` — c32's lane
// owns those room bodies; the Convert-family cells land here). UNWIRED: no
// executed step calls them, both world-hash pins unchanged. The C1/C2
// helpers ride the 15/18/19 exposure arms and the 16 hardening floor.

/// `WorldGen.Convert(i, j, conversionType, size)` for the dungeon-biome
/// types 15/16/18/19 (`WorldGen.cs:55067-55083` walk): the square walk with
/// the Manhattan `< 6` gate, each cell through [`convert_biome_cell`].
pub fn convert_sized_biome(world: &mut World, i: i32, j: i32, conversion_type: u8, size: i32) {
    if !matches!(conversion_type, 15 | 16 | 18 | 19) {
        return;
    }
    for k in (i - size)..=(i + size) {
        for l in (j - size)..=(j + size) {
            if in_world(world, k, l, 1) && (k - i).abs() + (l - j).abs() < 6 {
                convert_biome_cell(world, k, l, conversion_type);
            }
        }
    }
}

/// `Convert`'s type-15/16/18/19 cell arms (`WorldGen.cs:56008-56231`): the
/// wall chain, the torch arm, the tile ladder, then — unlike the spray /
/// hallow / chlorophyte cells — the INDEPENDENT `!tileFrameImportant[type]`
/// catch-all (`:56029`/`:56082`/`:56185`/`:56220`): any non-frame-important
/// tile converts to the biome's base kind (mushroom 70/59 by exposure,
/// sandstone 396, stone 1, jungle 60/59 by exposure). The thorn arms differ
/// per type: 15/18 KILL the thorn, 16 kills it with the 69 exemption, 19
/// CONVERTS it to the jungle thorn 69.
///
/// Deviations (the `convert_cleanse_cell` / C3-C5 precedent): the wall
/// halves ride the cell-local chains below (the shared
/// `convert_wall_by_case` carries no 15/16/18/19 arm); the re-frame is the
/// `reframe_area` superset; the `RainbowSandAndBlackSandWalls` paint arms
/// (`:56302-56306` etc.) are the d1-documented skip; the `NetMessage` tails
/// are MP-only. The type-19 wall switch's `case 15` no-op and the default
/// 15 arm are verbatim (`:56193-56206`).
pub fn convert_biome_cell(world: &mut World, x: i32, y: i32, conversion_type: u8) {
    let Some(t) = world.tile(x, y).copied() else {
        return;
    };
    let (kind, wall, active) = (t.kind, t.wall, t.active());
    // `:55087-55097` — the empty cell converts nothing.
    if !active && wall == 0 {
        return;
    }
    // The wall half — each type's own chain.
    let new_wall: Option<u16> = match conversion_type {
        // `:56009-56012` — any wall → the mushroom wall 80.
        15 => Some(80),
        // `:56042-56052` — the HardenedSand/Dirt/Snow union → 216, else 187.
        16 => {
            if (crate::growth::conversion_hardened_sand_walls_contains(wall)
                || crate::growth::conversion_dirt_walls_contains(wall)
                || crate::growth::conversion_snow_walls_contains(wall))
                && wall != 216
            {
                Some(216)
            } else if wall != 187 {
                Some(187)
            } else {
                None
            }
        }
        // `:56125-56151` — the HardenedSand/Snow/Dirt union → 2, the NewWall
        // family → 212-215, else 349. The else-if fall-through is load-bearing:
        // a wall already at its union/NewWall target (2 / 212-215) falls to
        // the final `wall != 349` arm → 349 (the C# chain, verbatim).
        18 => {
            if (crate::growth::conversion_hardened_sand_walls_contains(wall)
                || crate::growth::conversion_snow_walls_contains(wall)
                || crate::growth::conversion_dirt_walls_contains(wall))
                && wall != 2
            {
                Some(2)
            } else if let Some(set) = crate::growth::conversion_new_wall_set(wall) {
                let target = 212 + set as u16;
                if wall != target {
                    Some(target)
                } else if wall != 349 {
                    Some(349)
                } else {
                    None
                }
            } else if wall != 349 {
                Some(349)
            } else {
                None
            }
        }
        // `:56191-56207` — the wall>0 switch: the grass family 63/69/70/81
        // → 64, the wall 15 no-op, the default → 15.
        19 => {
            if wall > 0 {
                match wall {
                    63 | 69 | 70 | 81 => Some(64),
                    15 => None,
                    _ => Some(15),
                }
            } else {
                None
            }
        }
        _ => None,
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
    // `:56013-56016` etc. — the tiles-off break skips the whole tile half.
    // (The single-cell overload's `tiles` flag defaults true; the port's
    // callers all pass tiles, so the half always runs here.)
    // The torch arm (`:56017`/`:56057`/`:56156`/`:56212`): the default style
    // set, styles 22 / 16 / 0 / 21 per type. The torch is frame-important,
    // so the catch-all below never re-types it.
    if active && kind == 4 && torch_convertible_default(t.frame_y as i32 / 22) {
        let style = match conversion_type {
            15 => 22i16,
            16 => 16,
            18 => 0,
            _ => 21,
        };
        let cur = t.frame_y as i32 / 22;
        if cur != style as i32 {
            if let Some(t) = world.tile_mut(x, y) {
                t.frame_y = style * 22;
            }
            crate::framing::reframe_area(world, x, y, 1, false);
        }
        return;
    }
    // The tile ladder in source order. `Conversion.Dirt` = {0}
    // (`TileID.cs:88`), `Conversion.Snow` = {147} (`:89`).
    let dirt = kind == 0;
    let snow = CONV_SNOW.contains(&kind);
    let thorn = CONV_THORN.contains(&kind);
    // The frame-important read (`Main.tileFrameImportant[type]`) — the
    // catch-all's gate. The port's `frame_important` vec carries the boot
    // statics; an out-of-range kind reads false (the C# array is
    // `TileID.Count`-long, so every u16 kind is in range there).
    let important = world
        .frame_important
        .get(kind as usize)
        .copied()
        .unwrap_or(false);
    // The 15/19 catch-alls re-read the LIVE tile after the thorn arm: the
    // C#'s `theTile` is a `ref Tile`, so a thorn `KillTile` deactivates it
    // and `Convert_ActuallyConvertTile`'s `theTile.active()` gate then
    // refuses the catch-all write. `live_convert` models that re-read.
    let live_convert = |world: &mut World, new_kind: u16| {
        if let Some(t) = world.tile(x, y) {
            let (k, a) = (t.kind, t.active());
            actually_convert_tile(world, x, y, k, a, new_kind);
        }
    };
    match conversion_type {
        // Type 15 (`:56021-56039`): the thorn kill, then the exposure
        // catch-all → mushroom grass 70 (air) / mud 59 (sealed).
        15 => {
            if thorn {
                let _ = crate::edit::kill_tile_legacy(world, x, y, false);
            }
            if !important {
                let new_kind = if tile_is_exposed_to_air(world, x, y) { 70 } else { 59 };
                live_convert(world, new_kind);
            }
        }
        // Type 16 (`:56061-56085`): grass/sand/snow/dirt → 53 (397 over a
        // hardening floor), hardened sand → 397, the thorn kill (69 exempt),
        // then the non-396 catch-all → sandstone 396.
        16 => {
            if (CONV_GRASS.contains(&kind)
                || crate::tiledata::SET_CONVERSION_SAND.has(kind)
                || snow
                || dirt)
                && kind != 53
            {
                let new_kind = if block_below_makes_sand_convert_into_hardened_sand(world, x, y) {
                    397
                } else {
                    53
                };
                actually_convert_tile(world, x, y, kind, active, new_kind);
            } else if crate::tiledata::SET_CONVERSION_HARDENED_SAND.has(kind) && kind != 397 {
                actually_convert_tile(world, x, y, kind, active, 397);
            } else if thorn && kind != 69 {
                let _ = crate::edit::kill_tile_legacy(world, x, y, false);
            } else if kind != 396 && !important {
                actually_convert_tile(world, x, y, kind, active, 396);
            }
        }
        // Type 18 (`:56160-56188`): golf grass → 477, grass → 2, the
        // sand/hardened/snow/dirt union → 0 (2 when exposed), the thorn kill,
        // then the non-1 catch-all → stone 1.
        18 => {
            if crate::tiledata::SET_GOLF_GRASS.has(kind) && kind != 477 {
                actually_convert_tile(world, x, y, kind, active, 477);
            } else if CONV_GRASS.contains(&kind) && kind != 2 && kind != 477 {
                actually_convert_tile(world, x, y, kind, active, 2);
            } else if (crate::tiledata::SET_CONVERSION_SAND.has(kind)
                || crate::tiledata::SET_CONVERSION_HARDENED_SAND.has(kind)
                || snow
                || dirt)
                && kind != 0
            {
                let new_kind = if tile_is_exposed_to_air(world, x, y) { 2 } else { 0 };
                actually_convert_tile(world, x, y, kind, active, new_kind);
            } else if thorn {
                let _ = crate::edit::kill_tile_legacy(world, x, y, false);
            } else if kind != 1 && !important {
                actually_convert_tile(world, x, y, kind, active, 1);
            }
        }
        // Type 19 (`:56216-56230`): the thorn CONVERTS to the jungle thorn
        // 69 (not a kill), then the exposure catch-all → jungle grass 60
        // (air) / mud 59 (sealed). The catch-all re-reads the LIVE tile: a
        // thorn that just became 69 is still non-frame-important, so it
        // converts AGAIN to 60/59 (the C# `theTile.type` re-read, verbatim).
        19 => {
            if thorn {
                actually_convert_tile(world, x, y, kind, active, 69);
            }
            if !important {
                let new_kind = if tile_is_exposed_to_air(world, x, y) { 60 } else { 59 };
                live_convert(world, new_kind);
            }
        }
        _ => {}
    }
}

/// `Convert_ActuallyConvertTile` (`WorldGen.cs:56312-56327`): the active +
/// different gate, the tree-invalidity kill, the kind write, the re-frame.
/// The `RainbowSandAndBlackSandWalls` paint arm (`:56319-56323`) is the
/// d1-documented skip; the `NetMessage` tail is MP-only.
fn actually_convert_tile(world: &mut World, x: i32, y: i32, kind: u16, active: bool, new_kind: u16) {
    if active && kind != new_kind {
        try_kill_invalid_tree_above_legacy(world, x, y, new_kind);
        if let Some(t) = world.tile_mut(x, y) {
            t.kind = new_kind;
        }
        crate::framing::reframe_area(world, x, y, 1, false);
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

/// `WorldGen.initializeHardMode`'s runner half (`WorldGen.cs:31659-31660`):
/// the hallow V on one side and the evil V on the other (the C# `GERunner(
/// num3, 0, 3·num5, 5f)` good + `GERunner(num4, 0, 3·−num5, 5f, good:
/// false)`); the start xs sit ±`maxTilesX/7` off the middle.
///
/// W4-g69 retire sweep: the evil flavour is `WorldGen.crimson` read LIVE
/// inside `GERunner` (`WorldGen.cs:76304`) — the port reads `world.crimson`
/// (the persisted/UI-picked field) instead of the old `corruption` bool the
/// callers re-derived from the seed's high bit (the `roll_crimson` stand-in,
/// which ignored the UI pick and the drunk dawn flip).
pub fn initialize_hard_mode(world: &mut World, rng: &mut UnifiedRandom) {
    let offset = world.width / 7;
    // `num5 = ±1` (`WorldGen.cs:31585-31590`) — the velocity `3·num5` gives
    // the 3:5 V slope verbatim.
    let speed = 3.0;
    // The evil side's flavour: the C# `GERunner` reads the `WorldGen.crimson`
    // static at `:76304`; `convert_tile`/`convert_wall` take the OPPOSITE
    // bool (`corruption = !crimson`).
    let corruption = !world.crimson;
    // The hallow runner: the east side heading further east.
    gerunner(world, world.width / 2 + offset, 0, speed, 5.0, true, corruption, rng);
    // The evil runner: the west side heading further west.
    gerunner(world, world.width / 2 - offset, 0, -speed, 5.0, false, corruption, rng);
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
        w.crimson = false;
        let mut rng = UnifiedRandom::new(11);
        initialize_hard_mode(&mut w, &mut rng);
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
        w2.crimson = true;
        let mut rng = UnifiedRandom::new(11);
        initialize_hard_mode(&mut w2, &mut rng);
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

    /// W4-g42 G7: the type-3 cell (`WorldGen.cs:55336-55359`) — the four
    /// mushroom-safe walls → 80, the jungle grass 60 → 70, the torch
    /// default-switch styles → 22, the thorn set dies, and the
    /// inactive+wall-0 cell early-outs. RNG-free by construction.
    #[test]
    fn w4_g42_the_mushroom_conversion_cell_is_verbatim() {
        let mut w = world();
        // The wall set {64, 67, 15, 247} (`WallID.cs:88`) — each converts
        // to 80 on an INACTIVE cell (the wall arm runs before the tile
        // ladder, gated only by `!active && wall <= 0`).
        for (x, wall) in [(10i32, 64u16), (12, 67), (14, 15), (16, 247)] {
            let i = w.index(x, 100);
            w.tiles[i] = Tile { wall, ..Default::default() };
            convert_cell_mushroom(&mut w, x, 100);
            assert_eq!(w.tile(x, 100).unwrap().wall, 80, "wall {wall} → 80");
        }
        // A non-set wall stands (the type-3 ladder has no other row).
        let i = w.index(18, 100);
        w.tiles[i] = Tile { wall: 2, ..Default::default() };
        convert_cell_mushroom(&mut w, 18, 100);
        assert_eq!(w.tile(18, 100).unwrap().wall, 2, "the dirt wall stands");
        // The inactive + wall-0 cell early-outs (`:55092-55095`).
        let i = w.index(20, 100);
        w.tiles[i] = Tile { kind: 60, ..Default::default() }; // stored 60, inactive
        convert_cell_mushroom(&mut w, 20, 100);
        assert_eq!(w.tile(20, 100).unwrap().kind, 60, "the dead cell stands");
        // The jungle grass 60 → 70 (`:55347-55350`).
        put(&mut w, 22, 100, 60);
        convert_cell_mushroom(&mut w, 22, 100);
        assert_eq!(w.tile(22, 100).unwrap().kind, 70, "60 → 70");
        // The torch default-switch set (`TileID.cs:63-77`): styles
        // {0, 9, 16, 18, 19, 20, 21, 22} convert to 22; style 5 does not.
        for (x, style) in [(24i32, 0i16), (26, 9), (28, 16), (30, 21)] {
            put(&mut w, x, 100, 4);
            let i = w.index(x, 100);
            w.tiles[i].frame_y = style * 22;
            convert_cell_mushroom(&mut w, x, 100);
            assert_eq!(w.tile(x, 100).unwrap().frame_y, 22 * 22, "torch {style} → 22");
        }
        put(&mut w, 32, 100, 4);
        let i = w.index(32, 100);
        w.tiles[i].frame_y = 5 * 22;
        convert_cell_mushroom(&mut w, 32, 100);
        assert_eq!(w.tile(32, 100).unwrap().frame_y, 5 * 22, "style 5 stands");
        // The thorn set {32, 352, 69, 655} (`TileID.cs:95`) dies.
        for (x, kind) in [(34i32, 32u16), (36, 352), (38, 69), (40, 655)] {
            put(&mut w, x, 100, kind);
            convert_cell_mushroom(&mut w, x, 100);
            assert!(!w.tile(x, 100).unwrap().active(), "thorn {kind} dies");
        }
    }

    /// W4-g42 G7: the 60 → 70 write kills an invalid tree above
    /// (`Convert_ActuallyConvertTile` → `TryConvertingOrKillingTreesAbove…`,
    /// `:56316`): a palm 323 above a jungle-grass cell dies (the mushroom
    /// grass 70 is not in `Conversion.Sand`), a tile-5 tree survives (70 is
    /// in the grass-side union).
    #[test]
    fn w4_g42_the_mushroom_conversion_kills_the_invalid_tree_above() {
        let mut w = world();
        put(&mut w, 50, 100, 60);
        put(&mut w, 50, 99, 323); // the palm trunk
        convert_cell_mushroom(&mut w, 50, 100);
        assert_eq!(w.tile(50, 100).unwrap().kind, 70);
        assert!(!w.tile(50, 99).unwrap().active(), "the palm dies on 70");
        let mut w2 = world();
        put(&mut w2, 50, 100, 60);
        put(&mut w2, 50, 99, 5); // the vanilla tree trunk
        convert_cell_mushroom(&mut w2, 50, 100);
        assert!(w2.tile(50, 99).unwrap().active(), "the tree survives on 70");
    }

    /// W4-g42 GROWTH SEAL: the sized type-3 walk (`WorldGen.cs:55067-55083`,
    /// the Clentaminator DarkBlueSolution 148 site `Projectile.cs:26834`) —
    /// the SAME square + Manhattan `< 6` gate as the cleanse walk: at
    /// size 2 every (±2, ±2) cell sits inside (max Manhattan 4); the gate
    /// only bites at size 3 (the solution spray), where the (±3, ±3)
    /// corners (Manhattan 6) refuse. Every 60 in the diamond becomes 70;
    /// the out-of-diamond 60 stands. RNG-free — the a == b halves compare
    /// the full tile state.
    #[test]
    fn w4_g42_seal_the_sized_mushroom_walk_matches_the_convert_gate() {
        let run = |size: i32| {
            let mut w = world();
            for dx in -3..=3i32 {
                for dy in -3..=3i32 {
                    put(&mut w, 100 + dx, 100 + dy, 60);
                }
            }
            convert_sized_mushroom(&mut w, 100, 100, size);
            w
        };
        let w2 = run(2);
        for dx in -2..=2i32 {
            for dy in -2..=2i32 {
                assert_eq!(w2.tile(100 + dx, 100 + dy).unwrap().kind, 70,
                    "size 2: ({dx},{dy}) inside the walk");
            }
        }
        assert_eq!(w2.tile(103, 100).unwrap().kind, 60, "size 2: the ring outside stands");
        let w3 = run(3);
        assert_eq!(w3.tile(103, 103).unwrap().kind, 60,
            "size 3: the Manhattan-6 corner refuses");
        assert_eq!(w3.tile(103, 102).unwrap().kind, 70,
            "size 3: Manhattan 5 converts");
        assert_eq!(run(3).tiles, run(3).tiles, "a == b — the full tile state");
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

    // W4-d1 — the powder Convert types (the aiStyle-6 arm's calls,
    // `Projectile.cs:24193-24201`). WRITTEN NOT RUN (code-only round).

    #[test]
    fn w4_d1_the_purify_ladder_is_verbatim() {
        // `WorldGen.cs:55850-55881` — the type-11 tile ladder.
        let mut w = world();
        let rows = [
            (23, 2), (109, 2), (199, 2), (662, 60), (661, 60), (492, 477),
            (25, 1), (117, 1), (203, 1), (200, 161), (163, 161), (164, 161),
            (112, 53), (116, 53), (234, 53), (398, 397), (399, 397),
            (402, 397), (400, 396), (401, 396), (403, 396),
        ];
        for (i, (src, _)) in rows.iter().enumerate() {
            put(&mut w, 400 + i as i32, 200, *src);
        }
        // The already-pure cells stay (the `!= target` gates).
        put(&mut w, 450, 200, 2);
        put(&mut w, 451, 200, 477);
        put(&mut w, 452, 200, 60);
        for (i, _) in rows.iter().enumerate() {
            convert_sized(&mut w, 400 + i as i32, 200, 11, 0);
        }
        convert_sized(&mut w, 450, 200, 11, 0);
        convert_sized(&mut w, 451, 200, 11, 0);
        convert_sized(&mut w, 452, 200, 11, 0);
        for (i, (src, dst)) in rows.iter().enumerate() {
            assert_eq!(w.tile(400 + i as i32, 200).unwrap().kind, *dst, "{src} purifies");
        }
        assert_eq!(w.tile(450, 200).unwrap().kind, 2);
        assert_eq!(w.tile(451, 200).unwrap().kind, 477);
        assert_eq!(w.tile(452, 200).unwrap().kind, 60);
    }

    #[test]
    fn w4_d1_the_evil_ladders_are_verbatim() {
        // `WorldGen.cs:55138-55173` (crimson) / `:55299-55334` (corruption):
        // the moss-or-stone arm first, then the grass/ice/sand ladder.
        let mut w = world();
        let corrupt = [
            (2, 23), (60, 661), (1, 25), (179, 25), (161, 163), (53, 112),
            (397, 398), (396, 400), (352, 32),
        ];
        let crimson = [
            (2, 199), (60, 662), (1, 203), (179, 203), (161, 200), (53, 234),
            (397, 399), (396, 401), (32, 352),
        ];
        for (i, (src, _)) in corrupt.iter().enumerate() {
            put(&mut w, 400 + i as i32, 200, *src);
            put(&mut w, 400 + i as i32, 210, *src);
        }
        for (i, _) in corrupt.iter().enumerate() {
            convert_sized(&mut w, 400 + i as i32, 200, 1, 0);
            convert_sized(&mut w, 400 + i as i32, 210, 4, 0);
        }
        for (i, (_, dst)) in corrupt.iter().enumerate() {
            assert_eq!(w.tile(400 + i as i32, 200).unwrap().kind, *dst, "corrupt");
        }
        for (i, (_, dst)) in crimson.iter().enumerate() {
            assert_eq!(w.tile(400 + i as i32, 210).unwrap().kind, *dst, "crimson");
        }
    }

    #[test]
    fn w4_d1_the_powder_wall_chains() {
        // Type 1/4 flat ladder (`:55262-55295`/`:55101-55134`); type 11's
        // stone → 349 (`:55768`), the 262/274/292 switch (`:55788-55796`),
        // the 69/70/81 no-rng 64 (`:55780-55784`, the genRand skip) and the
        // 61/185/204-207 no-op cases (`:55836-55842`).
        let mut w = world();
        let cases: [(u16, u8, u16); 10] = [
            (1, 1, 3), (63, 1, 69), (63, 4, 81), (3, 11, 349), (69, 11, 64),
            (262, 11, 61), (274, 11, 185), (292, 11, 204), (61, 11, 61),
            (204, 11, 204),
        ];
        for (i, (wall, _, _)) in cases.iter().enumerate() {
            put(&mut w, 400 + i as i32, 200, 1);
            w.tile_mut(400 + i as i32, 200).unwrap().wall = *wall;
        }
        for (i, (_, ty, _)) in cases.iter().enumerate() {
            convert_sized(&mut w, 400 + i as i32, 200, *ty, 0);
        }
        for (i, (src, ty, dst)) in cases.iter().enumerate() {
            assert_eq!(
                w.tile(400 + i as i32, 200).unwrap().wall, *dst,
                "wall {src} type {ty}"
            );
        }
    }

    #[test]
    fn w4_d1_the_powder_torch_styles() {
        // `TileID.cs:42-79` — type 11 takes styles 18-20 only → 0
        // (`:55846-55848`); types 1/4 take the default set → 18/19
        // (`:55299-55301`/`:55138-55140`).
        let mut w = world();
        for (i, (style, ty, want)) in [(18, 11u8, 0i16), (0, 1, 18), (9, 4, 19)]
            .iter()
            .enumerate()
        {
            put(&mut w, 400 + i as i32, 200, 4);
            w.tile_mut(400 + i as i32, 200).unwrap().frame_y = style * 22;
            convert_sized(&mut w, 400 + i as i32, 200, *ty, 0);
            assert_eq!(
                w.tile(400 + i as i32, 200).unwrap().frame_y,
                want * 22,
                "torch style {style} type {ty}"
            );
        }
        // A non-convertible style stays (type 11's 18-20 gate).
        put(&mut w, 410, 200, 4);
        w.tile_mut(410, 200).unwrap().frame_y = 9 * 22;
        convert_sized(&mut w, 410, 200, 11, 0);
        assert_eq!(w.tile(410, 200).unwrap().frame_y, 9 * 22);
    }

    #[test]
    fn w4_d1_the_sized_walk_carries_the_manhattan_gate() {
        // `WorldGen.cs:55073-55082` — the (2·size+1)² square walk keeps
        // only the cells with `|k-i| + |l-j| < 6`.
        let mut w = world();
        for x in 397..=403 {
            for y in 197..=203 {
                put(&mut w, x, y, 2);
            }
        }
        convert_sized(&mut w, 400, 200, 1, 3);
        assert_eq!(w.tile(400, 200).unwrap().kind, 23, "the centre");
        assert_eq!(w.tile(400, 197).unwrap().kind, 23, "Manhattan 3");
        assert_eq!(w.tile(397, 197).unwrap().kind, 2, "Manhattan 6 skips");
        assert_eq!(w.tile(403, 203).unwrap().kind, 2, "the far corner skips");
    }

    #[test]
    fn w4_d1_the_empty_cell_converts_nothing() {
        // `:55094-55097` — `!active && wall <= 0` returns; the size-0 walk
        // over open air writes no tile.
        let mut w = world();
        convert_sized(&mut w, 400, 200, 11, 0);
        assert!(!w.tile(400, 200).unwrap().active());
        assert_eq!(w.tile(400, 200).unwrap().wall, 0);
    }
}
