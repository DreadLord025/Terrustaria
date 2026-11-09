//! `WorldGen`'s "Living Trees" pass (`WorldGen.cs:15476-15715`) and
//! `GrowLivingTree` (`WorldGen.cs:27973-28627`): the big hollow trees built
//! from living wood (tile 191) and leaf blocks (tile 192), with wall 244
//! (`LivingWoodUnsafe`) lining the interiors, the descending passage and the
//! wood-tree walls.
//!
//! Simplifications against the source, all recorded here:
//! * The secret-seed variants other than the drunk and M5-SEED-2
//!   `extraLivingTrees` arms (the 1.4.5 micro seed `error`, the bees, remix,
//!   anniversary and tenth-anniversary ladders), the skyblock/remix layouts and
//!   the dungeon-wall checks are not modelled. The M5-SEED-1 drunk arms are the
//!   tree-count ladder (`WorldGen.cs:15504`, `num4 += (int)(2.0 * num3)`) and
//!   the per-side patch count (`:15628`, `num8 += genRand.Next(2, 5)`); the
//!   M5-SEED-2 `extraLivingTrees` arms (`:15494-15499`, `:15588-15592`,
//!   `:15620-15623`) are the `num4 += 30*num3` / `3*num3` count ladder with the
//!   `Variations.extraLivingTreesReducedAmount` switch (`:217-231`), the
//!   `num = 150` spawn-exclusion half-width, the cloud tiles
//!   (189/196/460/717/718/719) no longer invalidating a site, the
//!   `GenVars.numMCaves` ±50 veto being skipped, and the per-side patch count
//!   `num8 += genRand.Next(5, 11)`. They are flag-gated on
//!   [`crate::seeds::MicroSeedFlags::EXTRA_LIVING_TREES`], so a classic world
//!   (and a drunk world) keeps exactly its M5-SEED-1 draws.
//! * `WorldGen.cs:15554` / `:15624` — drunk's other two "arms" are pure
//!   exclusions: drunk is barred from the `notTheBees && tenthAnniversaryWorldGen`
//!   branches of the anchor ground-type test (`:15554`'s
//!   `(notTheBees && type == 60) || (notTheBees && anniversary && !drunk)`)
//!   and of `GrowLivingTree`'s trunk widening (`:28037`, `:28063`). The port
//!   carries neither not-the-bees branch, so its generic tests already are the
//!   drunk behaviour and nothing is written for them (deferred whole: the
//!   bees/anniversary seed arms belong to their own seeds).
//! * The hollow room's furniture (door style 7, chair style 5, Living Loom,
//!   the buried Living Wood Chest) needs `TileObjectData` (M1.4-TO), which the
//!   port does not have yet — the room is carved with wall 244 but left empty.
//! * `GrowLivingTree_HorizontalTunnel`'s door placement and the spike/wall
//!   aborts are reduced to the plain tunnel (no tile 48 spikes exist in this
//!   generator).
//! * The passage floor platforms (tile 19 style 23) need platform placement
//!   (frame data) and are left out.
use super::noise::Rng;
use crate::{tiledata as td, World};

pub(crate) const TILE_LIVING_WOOD: u16 = 191;
pub(crate) const TILE_LEAVES: u16 = 192;
pub(crate) const WALL_LIVING_WOOD: u16 = 244;

/// `SecretSeed.Variations.extraLivingTreesReducedAmount`
/// (`WorldGen.cs:217-231`): the halved count when the seed list is crowded —
/// `activeSecretSeedCount < 6` reads `noSurface.Enabled`, otherwise `true`:
///
/// ```text
/// if (SecretSeed.extraLivingTrees.Enabled) {
///     if (SecretSeed.activeSecretSeedCount < 6) return SecretSeed.noSurface.Enabled;
///     return true;
/// }
/// return false;
/// ```
///
/// `activeSecretSeedCount` counts both registries, so the port folds the
/// classic `SeedFlags` population in through
/// [`crate::seeds::active_secret_seed_count`]. The port carries no `noSurface`
/// arm yet, so its `activeSecretSeedCount < 6` branch reads `false` — the
/// documented deviation; the `>= 6` branch is exact.
fn extra_living_trees_reduced_amount(world: &World) -> bool {
    if crate::seeds::active_secret_seed_count(world.micro_seeds, world.seeds) < 6 {
        // `SecretSeed.noSurface.Enabled` — the port's `NO_SURFACE` bit. The
        // seed's *behaviour* (M5-SEED-2's `noSurface` body) is another
        // agent's file; the flag itself is registered here.
        world
            .micro_seeds
            .contains(crate::seeds::MicroSeedFlags::NO_SURFACE)
    } else {
        true
    }
}

/// `WorldGen.cs:15482-15515`: the full-tree count — `num3` (the world-width
/// unit), the base roll, the zero rescue and the seed-count ladder's arms.
/// The draws stay in vanilla's order (the base roll, then the rescue, then
/// the ladder's own addition) so a classic world's stream is untouched.
///
/// The ladder is an if/else-if chain in the C# (`extraLivingTrees` `:15494` /
/// `notTheBees && anniversary && !drunk` `:15500` / `drunk` `:15504` /
/// `tenthAnniversaryWorld` `:15508` / `remixWorldGen` `:15512`); the port
/// carries the `extraLivingTrees` and drunk arms.
fn living_tree_count(world: &World, rng: &mut Rng) -> i32 {
    let num3 = world.width as f64 / 4200.0;
    let mut count = rng.range(0, (2.0 * num3) as i32 + 1);
    if count == 0 && rng.chance(2) {
        count += 1;
    }
    // `WorldGen.cs:15494-15499`: the extraLivingTrees arm —
    // `num4 += (int)(30.0 * num3)`, or `+ (int)(3.0 * num3)` under
    // `Variations.extraLivingTreesReducedAmount`. Flag-gated (and ahead of the
    // drunk arm, exactly as the C# chain orders it), so a classic and a drunk
    // world take the same two draws above and nothing more.
    if world
        .micro_seeds
        .contains(crate::seeds::MicroSeedFlags::EXTRA_LIVING_TREES)
    {
        let per_tree = if extra_living_trees_reduced_amount(world) {
            3.0
        } else {
            30.0
        };
        count += (per_tree * num3) as i32;
    } else if world.seeds.contains(crate::seeds::SeedFlags::DRUNK) {
        // `WorldGen.cs:15504-15507`: the drunk arm — `num4 += (int)(2.0 * num3)`,
        // two extra trees per 4200-column world-width unit.
        count += (2.0 * num3) as i32;
    }
    count
}

/// `WorldGen.cs:15619-15631`: the per-side patch-tree count — `num8 =
/// genRand.Next(4)`, then the ladder's arms. Same chain as the count above
/// (`extraLivingTrees` +`Next(5, 11)` `:15620`, `notTheBees && anniversary &&
/// !drunk` +`Next(3, 7)` `:15624`, `drunk || anniversary` +`Next(2, 5)`
/// `:15628`, `remixWorldGen` +`Next(1, 6)` `:15632`, `errorWorld`
/// +`Next(1, 4)` `:15636`); the port carries the extraLivingTrees and drunk
/// arms.
fn patch_tree_count(world: &World, rng: &mut Rng) -> i32 {
    let mut attempts = rng.range(0, 4);
    // `WorldGen.cs:15620-15623`: `if (SecretSeed.extraLivingTrees.Enabled)
    // num8 += genRand.Next(5, 11)` — six extra attempts per side.
    if world
        .micro_seeds
        .contains(crate::seeds::MicroSeedFlags::EXTRA_LIVING_TREES)
    {
        attempts += rng.range(5, 11);
    } else if world.seeds.contains(crate::seeds::SeedFlags::DRUNK) {
        // `WorldGen.cs:15628-15631`: `else if (drunkWorldGen ||
        // Main.tenthAnniversaryWorld) num8 += genRand.Next(2, 5)`.
        attempts += rng.range(2, 5);
    }
    attempts
}

/// `WorldGen.cs:15482` / `:15496`: the spawn-area exclusion half-width,
/// `num = 200` — narrowed to `num = 150` under `extraLivingTrees`. Factored
/// out of the pass so the arm is directly testable; the anchor and the patch
/// gate both read it.
fn living_tree_exclusion(world: &World) -> i32 {
    if world
        .micro_seeds
        .contains(crate::seeds::MicroSeedFlags::EXTRA_LIVING_TREES)
    {
        150
    } else {
        200
    }
}

/// The "Living Trees" pass (`WorldGen.cs:15476-15715`). The beach anchors
/// are the vanilla GenVars pair (`GenVars.cs:90-92`): placements stay inland
/// of both beaches.
pub(crate) fn plant_living_trees(
    world: &mut World,
    rng: &mut Rng,
    surface: &[i32],
    left_beach_end: i32,
    right_beach_start: i32,
    desert: (i32, i32),
) {
    let count = living_tree_count(world, rng);
    let exclusion = living_tree_exclusion(world);
    #[cfg(test)]
    let mut placed_trees = 0u32;
    let half = world.width / 2;
    // The anchor span stays valid on sub-vanilla worlds: the vanilla minimum
    // is 4200 wide, so `leftBeachEnd > rightBeachStart` (the 512 pin worlds)
    // is a degenerate case vanilla never sees — order the span, don't panic.
    let (anchor_lo, anchor_hi) = if left_beach_end <= right_beach_start {
        (left_beach_end, right_beach_start)
    } else {
        (right_beach_start, left_beach_end)
    };
    let anchor_hi = anchor_hi.max(anchor_lo + 1);
    // M5-SEED-2: the C#'s anchor loop closes with the 50×50 site scan
    // (`WorldGen.cs:15566-15595`, the ebonstone/cloud switch) and the
    // `GenVars.numMCaves` ±50 proximity veto (`:15597-15608`). The site scan
    // is folded into `grow_living_tree`'s clearance scan (the port has no
    // separate anchor scan); the mCave veto is NOT ported — the port has no
    // `Mountinater` pass and therefore no `GenVars.mCaveX/mCaveY` anchors, so
    // neither the veto nor its `extraLivingTrees` skip (`:15598-15599`) can
    // fire. Recorded as a compact-model deviation; adding a veto would reject
    // sites on classic worlds, which the byte-identity contract forbids.
    for _ in 0..count {
        // `WorldGen.cs:15543-15546`: the anchor attempts, up to maxTilesX/2.
        let mut anchor = None;
        for _ in 0..world.width / 2 {
            let x = rng.range(anchor_lo, anchor_hi);
            // `WorldGen.cs:15537`: grow OUTSIDE the spawn plateau (the anchor
            // must satisfy `x <= half - num || x >= half + num`).
            if x > half - exclusion && x < half + exclusion {
                continue;
            }
            // `WorldGen.cs:15546`: scan down to the first active tile. The
            // port starts at the height-map row — the sky carries the
            // floating islands now, and the C#'s from-zero scan (empty C#
            // skies) would anchor on the island discs. The `extraLivingTrees`
            // arm that would reset the scan's start row to
            // `FindLowestCloud() + 1` (`:15497`) is not ported: the port has
            // no cloud line, and its own anchor already sits below the
            // island discs.
            let column_top = surface[x as usize];
            let mut y: i32 = column_top;
            while y <= column_top + 20
                && world
                    .tile(x, y)
                    .is_none_or(|t| !t.active())
            {
                y += 1;
            }
            if y > column_top + 20 {
                continue;
            }
            // The vanilla gates the anchor behind `j >= worldSurface → continue`
            // (`WorldGen.cs:15549`), rejecting the ocean columns whose floor
            // sits below the surface line. The port's terrain sits lower than
            // its `world_surface` line, so the equivalent here is: the anchor
            // cell (or its ground) must not be submerged.
            if world.tile(x, y).is_some_and(|t| t.liquid > 0)
                || world.tile(x, y - 1).is_some_and(|t| t.liquid > 0)
            {
                continue;
            }
            // The desert band grows no living trees (the vanilla dunes stay
            // bare — the C# dunes pass runs after the living trees and
            // overwrites their sand columns).
            if x >= desert.0 && x < desert.1 {
                continue;
            }
            // `WorldGen.cs:15546`: scan down to the first active tile. The
            // port starts at the height-map row — the sky carries the
            // floating islands now, and the C#'s from-zero scan (empty C#
            // skies) would anchor on the island discs.
            let column_top = surface[x as usize];
            let mut y: i32 = column_top;
            while y <= column_top + 20
                && world
                    .tile(x, y)
                    .is_none_or(|t| !t.active())
            {
                y += 1;
            }
            if y > column_top + 20 {
                continue;
            }            // `WorldGen.cs:15549-15552`: the anchor sits in the air tile above
            // the ground, one more step up if the ground is dirt.
            let mut anchor_y = y as i32 - 1;
            if world
                .tile(x, anchor_y)
                .is_some_and(|t| t.active() && t.kind == 0)
            {
                anchor_y -= 1;
            }
            // `WorldGen.cs:15560`: no living wood/leaves within 10 tiles.
            if nearby_living(world, x, anchor_y, 10) {
                continue;
            }
            // `WorldGen.cs:15553-15555`: `flag = GrowLivingTree(...)` inside
            // the anchor-attempt loop — the clearance failures (the islands
            // in the port's sky) retry the next candidate.
            if !grow_living_tree(world, rng, x, anchor_y, false) {
                continue;
            }
            anchor = Some((x, anchor_y));
            break;
        }
        let Some((x, y)) = anchor else {
            continue;
        };
        // `WorldGen.cs:15611-15701`: the patch trees step outward from the
        // anchor, `Next(4)` attempts per side plus the seed ladder's arm
        // ([`patch_tree_count`]), `Next(13, 31)` tiles apart.
        for side in [-1i32, 1] {
            let attempts = patch_tree_count(world, rng);
            let mut px = x;
            for _ in 0..attempts {
                px += rng.range(13, 31) * side;
                if px > half - exclusion && px < half + exclusion {
                    continue;
                }
                // `WorldGen.cs:15690-15700`: re-scan the surface at the patch.
                let column_top = surface[px as usize];
                let mut py: i32 = 0;
                while py <= column_top
                    && world
                        .tile(px, py)
                        .is_none_or(|t| !t.active())
                {
                    py += 1;
                }
                if py > column_top {
                    continue;
                }
                let mut patch_y = py - 1;
                if world
                    .tile(px, patch_y)
                    .is_some_and(|t| t.active() && t.kind == 0)
                {
                    patch_y -= 1;
                }
                grow_living_tree(world, rng, px, patch_y, true);
            }
        }
    }
}

/// `WorldGen.IsTileNearby` (`WorldGen.cs:24373`): any active `kind` within the
/// square radius around the point.
fn nearby_living(world: &World, x: i32, y: i32, radius: i32) -> bool {
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            if world
                .tile(x + dx, y + dy)
                .is_some_and(|t| t.active() && (t.kind == 191 || t.kind == 192))
            {
                return true;
            }
        }
    }
    false
}

/// `WorldGen.GrowLivingTree` (`WorldGen.cs:27973-28627`).
pub(crate) fn grow_living_tree(
    world: &mut World,
    rng: &mut Rng,
    i: i32,
    j: i32,
    patch: bool,
) -> bool {
    // `WorldGen.cs:27993-28010`: the ground check.
    // `WorldGen.cs:28004-28010`: the ground must be dirt/grass/stone/clay or
    // ore — sand (53) is NOT in the list, so living trees reject beaches.
    // `WorldGen.cs:28004-28010`: the ground must be dirt/grass/stone/clay or
    // ore — sand (53) is NOT in the list, so living trees reject beaches. The
    // anchor cell must also be dry (trees never start submerged).
    let ground_ok = world.tile(i, j + 1).is_some_and(|t| {
        t.liquid == 0
            && t.active()
            && (matches!(t.kind, 0 | 2 | 1) || crate::tiledata::TILE_SOLID.has(t.kind))
    }) && world.tile(i, j).is_none_or(|t| t.liquid == 0);
    if !ground_ok {
        return false;
    }
    if world.tile(i, j).is_some_and(|t| t.active()) {
        return false;
    }
    if j < 150 {
        return false;
    }

    // `WorldGen.cs:28025-28054`: the trunk width. `Next(2, 3)` yields 2, so the
    // base trunk spans four tiles; one side widens by 1 with p=1/5.
    let (mut left, mut right) = (i - 2, i + 2);
    if patch {
        // `WorldGen.cs:28046-28049`: patch trees are narrower.
        left = i - rng.range(1, 3);
        right = i + rng.range(1, 3);
    }
    if rng.chance(5) {
        if rng.chance(2) {
            left -= 1;
        } else {
            right += 1;
        }
    }
    if patch {
        left = i - rng.range(1, 3);
        right = i + rng.range(1, 3);
    }
    let width = right - left;
    // `flag = num5 >= 4`: only wide trunks get the hollow passage.
    let hollow = width >= 4;

    // `WorldGen.cs:28080-28101`: the clearance scan (50 tiles wide, 20 tall).
    //
    // M5-SEED-2 `extraLivingTrees` (`WorldGen.cs:15577-15592`): the C#'s anchor
    // site scan rejects the cloud family {189, 196, 460, 717, 718, 719} unless
    // the seed is enabled ("clouds no longer invalidate a site"). The port has
    // no separate anchor scan — the site gate is this one, which is where the
    // floating-island discs land — so the arm is folded in here: under the flag
    // the cloud family is tolerated, anything else active still rejects. A
    // classic (and a drunk) world takes the unchanged rule.
    let cloud_ok = world
        .micro_seeds
        .contains(crate::seeds::MicroSeedFlags::EXTRA_LIVING_TREES);
    let (x0, x1) = if patch { (i - 20, i + 20) } else { (i - 50, i + 50) };
    for k in x0..=x1 {
        for l in 20..j - 5 {
            if let Some(t) = world.tile(k, l) {
                if t.active() && (!patch || !matches!(t.kind, 2 | 0 | 1 | 191 | 192)) {
                    // `case 189: case 196: case 460: case 717: case 718: case 719:`.
                    if cloud_ok && matches!(t.kind, 189 | 196 | 460 | 717 | 718 | 719) {
                        continue;
                    }
                    return false;
                }
            }
        }
    }

    // `WorldGen.cs:28103`: spikes go non-solid for the body — the port has no
    // spikes, so the flip is not modelled.

    // `WorldGen.cs:28110-28113`: the branch cadence and the taper odds.
    let mut branch_timer = rng.range(-8, -4);
    let mut branch_dir = rng.range(0, 2);
    let mut branch_every = rng.range(5, 15);
    let taper_denom = 2;

    // The branch base bookkeeping (arrays 2-6 in the source).
    let mut branch_bases: Vec<(i32, i32)> = Vec::new();
    // `array7`: the branch endpoints that grow round canopies.
    let mut round_canopies: Vec<(i32, i32)> = Vec::new();

    // `WorldGen.cs:28120-28195`: the upward tapering trunk.
    let (mut minl, mut minr) = (left, right);
    let mut row = j;
    loop {
        // The row of living wood.
        for m in minl..=minr {
            place_wood(world, m, row);
        }
        branch_timer += 1;
        if branch_timer > branch_every {
            branch_every = rng.range(5, 15);
            branch_timer = 0;
            // `WorldGen.cs:28138`: the branch base sits a few rows up.
            branch_bases.push((if branch_dir == 0 { minl } else { minr }, row + rng.range(0, 5)));
            if rng.chance(5) {
                branch_dir = 1 - branch_dir;
            }
            // The taper: one edge steps inward with p = 1/taper_denom.
            if rng.chance(taper_denom as u32) {
                if branch_dir == 0 {
                    minl += 1;
                } else {
                    minr -= 1;
                }
                branch_dir = 1;
            } else {
                branch_dir = 0;
            }
            // `WorldGen.cs:28193`: the edges met — the trunk tapered to a point.
            if minl == minr {
                break;
            }
        }
        if row <= 30 {
            break;
        }
        row -= 1;
    }
    let top_row = row;

    // `WorldGen.cs:28236-28306`: the side branches off the recorded bases.
    for &(base_x, base_y) in &branch_bases {
        // `num18 = width * (1 + Next(20, 30) * 0.1)` — the branch length.
        let mut length =
            ((width as f64) * (1.0 + rng.range(20, 30) as f64 * 0.1)) as i32;
        let dir = if base_x <= (left + right) / 2 { -1 } else { 1 };
        let (mut bx, mut by) = (base_x, base_y + 1);
        place_wood(world, bx, by);
        let mut thicken = rng.range(3, 5);
        while length > 0 {
            length -= 1;
            if dir > 0 {
                bx += 1;
            } else {
                bx -= 1;
            }
            place_wood(world, bx, by);
            if rng.chance(10) {
                by += if rng.chance(2) { 1 } else { -1 };
                place_wood(world, bx, by);
            }
            if thicken > 0 {
                thicken -= 1;
            } else if rng.chance(2) {
                thicken = rng.range(2, 5);
                // The 2-tall thickening records a leaf centre.
                place_wood(world, bx, by);
                place_wood(world, bx, by - 1);
                round_canopies.push((bx, by));
            }
            if length == 0 {
                round_canopies.push((bx, by));
            }
        }
    }

    // `WorldGen.cs:28308-28409`: the upper trunk with the big-branch canopy.
    let mut up_left_timer = 0;
    let mut up_right_timer = 0;
    let mut cx = (minl + minr) / 2;
    let mut cy = top_row;
    let mut upper = rng.range(width * 3, width * 5);
    while upper > 0 && cy >= 30 {
        upper -= 1;
        place_wood(world, cx, cy);
        if up_left_timer > 0 {
            up_left_timer -= 1;
        }
        if up_right_timer > 0 {
            up_right_timer -= 1;
        }
        // The branch every other step: -1 left, +1 right.
        for side in [-1i32, 1] {
            if side == 0
                || (side < 0 && up_left_timer != 0)
                || (side > 0 && up_right_timer != 0)
                || rng.chance(2)
            {
                continue;
            }
            let (mut bx, mut by) = (cx, cy);
            let mut length = rng.range(width, width * 3);
            if side < 0 {
                up_left_timer = rng.range(3, 5);
            } else {
                up_right_timer = rng.range(3, 5);
            }
            let mut wobble = 0;
            while length > 0 {
                length -= 1;
                bx += side;
                place_wood(world, bx, by);
                if length == 0 {
                    round_canopies.push((bx, by));
                }
                if rng.chance(5) {
                    by += if rng.chance(2) { 1 } else { -1 };
                    place_wood(world, bx, by);
                }
                if wobble > 0 {
                    wobble -= 1;
                } else if rng.chance(3) {
                    wobble = rng.range(2, 4);
                    let wy = by + if rng.chance(2) { 1 } else { -1 };
                    place_wood(world, bx, wy);
                }
            }
        }
        cy -= 1;
    }

    // `WorldGen.cs:28413-28530`: the roots.
    for trunk_x in minl..=minr {
        // `num33 = Next(1, 6)` rows of wood straight down.
        let mut depth = rng.range(1, 6);
        let mut ry = j + 1;
        while depth > 0 {
            if solid_at(world, trunk_x, ry) {
                depth -= 1;
            }
            place_wood(world, trunk_x, ry);
            ry += 1;
        }
        // `num36 = Next(2, width+1)` root arms per column.
        let arms = rng.range(2, width + 1);
        for _ in 0..arms {
            let mut length = rng.range((width as f64 * 3.5) as i32, width * 6);
            let centre = (minl + minr) / 2;
            let mut dx = if trunk_x >= centre { 1 } else { -1 };
            if trunk_x == centre || (width > 6 && (trunk_x == centre - 1 || trunk_x == centre + 1))
            {
                dx = 0;
            }
            let base_dx = dx;
            let mut dy = 1;
            let mut rx = trunk_x;
            let mut ryy = ry;
            while length > 0 {
                length -= 1;
                rx += dx;
                if world.tile(rx, ryy).is_none_or(|t| t.wall != WALL_LIVING_WOOD) {
                    place_wood(world, rx, ryy);
                }
                ryy += dy;
                if world.tile(rx, ryy).is_none_or(|t| t.wall != WALL_LIVING_WOOD) {
                    place_wood(world, rx, ryy);
                }
                if world
                    .tile(rx, ryy + 1)
                    .is_none_or(|t| !t.active())
                {
                    dx = 0;
                    dy = 1;
                }
                if rng.chance(3) {
                    dx = if base_dx < 0 {
                        if dx == 0 { -1 } else { 0 }
                    } else if base_dx <= 0 {
                        rng.range(-1, 2)
                    } else if dx == 0 {
                        1
                    } else {
                        0
                    };
                }
                if rng.chance(3) {
                    dy = if dy == 0 { 1 } else { 0 };
                }
            }
        }
    }

    // `WorldGen.cs:28487-28573`: the canopy blobs and the leaf piles.
    for &(centre_x, centre_y) in &round_canopies {
        let mut radius = rng.range(5, 8);
        radius = ((radius as f64) * (1.0 + width as f64 * 0.05)) as i32;
        let round = round_canopies.contains(&(centre_x, centre_y));
        if round {
            radius = rng.range(6, 12) + width;
        }
        let squash = 2.0 - rng.range(0, 5) as f64 * 0.1;
        for x in (centre_x - radius * 2)..=(centre_x + radius * 2) {
            for y in (centre_y - radius * 2)..=(centre_y + radius * 2) {
                if !can_place_leaves(world, x, y) {
                    continue;
                }
                if round {
                    let d = Vec2::new(centre_x - x, centre_y - y);
                    if d.length() < radius as f64 * 0.9 {
                        place_leaves(world, x, y);
                    }
                } else if (centre_x - x).abs() as f64 + (centre_y - y).abs() as f64 * squash
                    < radius as f64
                {
                    place_leaves(world, x, y);
                }
            }
        }
    }

    // `WorldGen.cs:28534-28573`: the leaf-pile decorations (tiles 187/185 with
    // `PlaceTile` frame tables) are not ported — they need the `PlaceTile`
    // machinery.

    // `WorldGen.cs:28575-28600`: the hollow passage for wide trunks.
    if hollow {
        // The open-sky check over rows j..j+20: a gap means the trunk hangs
        // over air and the passage is skipped.
        let mut open_sky = false;
        'outer: for yy in j..j + 20 {
            if yy as f64 >= world.world_surface - 2.0 {
                break;
            }
            for xx in minl..=minr {
                if world
                    .tile(xx, yy)
                    .is_none_or(|t| t.wall == 0 && !is_solid(world, xx, yy))
                {
                    open_sky = true;
                    break 'outer;
                }
            }
        }
        if !open_sky {
            make_passage(world, rng, j, width, minl, minr, patch);
        }
    }
    true
}

struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    fn new(x: i32, y: i32) -> Self {
        Self {
            x: x as f64,
            y: y as f64,
        }
    }
    fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

fn place_wood(world: &mut World, x: i32, y: i32) {
    let Some(t) = world.tile_mut(x, y) else {
        return;
    };
    t.kind = TILE_LIVING_WOOD;
    t.flags |= crate::tile::FLAG_ACTIVE;
    t.flags &= !crate::tile::FLAG_HALF_BRICK;
}

fn place_leaves(world: &mut World, x: i32, y: i32) {
    let Some(t) = world.tile_mut(x, y) else {
        return;
    };
    t.kind = TILE_LEAVES;
    t.flags |= crate::tile::FLAG_ACTIVE;
    t.flags &= !crate::tile::FLAG_HALF_BRICK;
}

fn solid_at(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind))
}

fn is_solid(world: &World, x: i32, y: i32) -> bool {
    solid_at(world, x, y)
}

/// `GrowLivingTree_CanPlaceLeaves` (`WorldGen.cs:28607-28627`).
fn can_place_leaves(world: &World, x: i32, y: i32) -> bool {
    if x < 5 || y < 5 || x >= world.width - 5 || y >= world.height - 5 {
        return false;
    }
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if t.wall == WALL_LIVING_WOOD || t.wall == 78 {
        return false;
    }
    if !t.active() {
        return true;
    }
    !(t.kind == TILE_LIVING_WOOD || matches!(t.kind, 189 | 196 | 460 | 717 | 718 | 719))
}

/// `GrowLivingTree_MakePassage` (`WorldGen.cs:28865-29133`): the 3-wide hollow
/// passage with wall 244 lining, drifting left/right, ending at the underworld.
/// The furniture room, the doors, the platforms and the buried chest need
/// `TileObjectData` (M1.4-TO) and stay out of this pass.
fn make_passage(world: &mut World, rng: &mut Rng, j: i32, width: i32, minl: i32, minr: i32, patch: bool) {
    let mut centre = (minl + minr) / 2;
    let mut extra_shell = 1;
    let mut y = j;
    let mut since_branch = 0;
    let end = (world.height - 200).min(world.height - 1);
    // `WorldGen.cs:28874`: the passage runs `Next(400, 700)` rows at most.
    let mut remaining = rng.range(400, 700);
    while y < end && remaining > 0 {
        remaining -= 1;
        // The drift every 6 rows.
        if y % 6 == 0 {
            let roll = rng.range(0, 3);
            if roll == 0 {
                centre -= 1;
            }
        }
        // The 3-wide interior: deactivate and wall.
        for x in (centre - 1)..=(centre + 1) {
            if let Some(t) = world.tile_mut(x, y) {
                t.flags &= !crate::tile::FLAG_ACTIVE;
                t.wall = WALL_LIVING_WOOD;
            }
        }
        // The one-tile wall skirts on both sides.
        for x in [(centre - 2), (centre + 2)] {
            if let Some(t) = world.tile_mut(x, y) {
                t.wall = WALL_LIVING_WOOD;
            }
        }
        // The shell becomes living wood.
        for x in (centre - 2 - extra_shell)..=(centre + 2 + extra_shell) {
            if world.tile(x, y).is_none_or(|t| !(t.active() && t.kind == TILE_LIVING_WOOD)) {
                place_wood(world, x, y);
            }
        }
        // `WorldGen.cs:29058`: patch trees branch horizontal tunnels.
        since_branch += 1;
        if patch && since_branch >= rng.range(5, 16) {
            since_branch = 0;
            horizontal_tunnel(world, rng, centre, y);
        }
        // The floor under the passage is open for 4 rows → stop.
        let mut open_rows = 0;
        for dx in -1..=1 {
            if world
                .tile(centre + dx, y + 1)
                .is_none_or(|t| !t.active())
            {
                open_rows += 1;
            }
        }
        if open_rows == 3 {
            break;
        }
        y += 1;
        if width > 4 && y as f64 > world.world_surface {
            extra_shell = 2;
        }
    }
}

/// `GrowLivingTree_HorizontalTunnel` (`WorldGen.cs:28629-28771`): a 3-wide
/// tunnel running sideways until it exits the tree.
fn horizontal_tunnel(world: &mut World, rng: &mut Rng, x: i32, y: i32) {
    let dir = if rng.chance(2) { 1 } else { -1 };
    let mut tx = x;
    let mut length = rng.range(30, 80);
    while length > 0 {
        length -= 1;
        tx += dir;
        if tx < 5 || tx >= world.width - 5 {
            return;
        }
        for dy in -1..=1 {
            if let Some(t) = world.tile_mut(tx, y + dy) {
                t.flags &= !crate::tile::FLAG_ACTIVE;
                t.wall = WALL_LIVING_WOOD;
            }
        }
        for dx in [tx - 1, tx + 1] {
            for dy in -2..=2 {
                if let Some(t) = world.tile_mut(dx, y + dy) {
                    if !(t.active() && t.kind == TILE_LIVING_WOOD) {
                        place_wood(world, dx, y + dy);
                    }
                }
            }
        }
        // Stop 2 tiles short when the tunnel reaches living wood again.
        if world.tile(tx + dir * 2, y).is_some_and(|t| t.active() && t.kind == TILE_LIVING_WOOD) {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tree-friendly grassland: the empty sky `GrowLivingTree`'s clearance
    /// scan wants (50 wide, 20 tall), grass from row 200 down (the anchor
    /// cell above it must be air, and `j >= 150` must hold), and vanilla
    /// beach anchors so the patch offsets stay inside the world.
    fn grassland(w: i32, h: i32) -> World {
        let mut world = World::new("lt".into(), w, h);
        world.world_surface = 200.0;
        for x in 0..w {
            for y in 200..h {
                let i = world.index(x, y);
                world.tiles[i].kind = 2;
                world.tiles[i].flags |= crate::tile::FLAG_ACTIVE;
            }
        }
        world
    }

    fn flat_surface(w: i32) -> Vec<i32> {
        vec![200; w as usize]
    }

    fn living_wood(world: &World) -> usize {
        world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == TILE_LIVING_WOOD)
            .count()
    }

    #[test]
    fn the_drunk_count_ladder_adds_two_per_world_unit() {
        // `WorldGen.cs:15504-15507`: `num4 += (int)(2.0 * num3)` — the
        // 4200-wide vanilla world (num3 = 1.0) grows exactly two more trees.
        let classic = World::new("lt".into(), 4200, 1200);
        let mut drunk = World::new("lt".into(), 4200, 1200);
        drunk.seeds |= crate::seeds::SeedFlags::DRUNK;
        for seed in [1u64, 7, 42, 5162020] {
            let mut a = Rng(seed);
            let mut b = Rng(seed);
            let base = living_tree_count(&classic, &mut a);
            let added = living_tree_count(&drunk, &mut b);
            assert_eq!(added, base + 2, "seed {seed}");
        }
        // And the narrower world scales with num3: the 2100 pin world adds
        // one (the `(int)` truncation of 2.0 * 0.5).
        let narrow = World::new("lt".into(), 2100, 1200);
        let mut narrow_drunk = World::new("lt".into(), 2100, 1200);
        narrow_drunk.seeds |= crate::seeds::SeedFlags::DRUNK;
        for seed in [3u64, 11] {
            let base = living_tree_count(&narrow, &mut Rng(seed));
            let added = living_tree_count(&narrow_drunk, &mut Rng(seed));
            assert_eq!(added, base + 1, "seed {seed}");
        }
    }

    #[test]
    fn the_drunk_patch_ladder_adds_two_to_four_attempts() {
        // `WorldGen.cs:15628-15631`: `num8 += genRand.Next(2, 5)` on top of
        // the `Next(4)` per-side count.
        let classic = World::new("lt".into(), 4200, 1200);
        let mut drunk = World::new("lt".into(), 4200, 1200);
        drunk.seeds |= crate::seeds::SeedFlags::DRUNK;
        for seed in [1u64, 5, 9] {
            let base = patch_tree_count(&classic, &mut Rng(seed));
            let added = patch_tree_count(&drunk, &mut Rng(seed));
            assert!(
                (2..=4).contains(&(added - base)),
                "seed {seed}: {base} + {added}"
            );
        }
    }

    #[test]
    fn the_classic_ladders_draw_nothing_extra() {
        // The golden rule: the classic path spends exactly vanilla's draws —
        // the base roll (+ the zero rescue) for the count, the single
        // `Next(4)` per side for the patches. Each helper's stream is
        // replayed against an independent ledger below.
        let world = World::new("lt".into(), 4200, 1200);
        for seed in [1u64, 7, 42] {
            let mut a = Rng(seed);
            let count = living_tree_count(&world, &mut a);
            let after_count = a.next_u32();
            let mut b = Rng(seed);
            let mut replay = b.range(0, 3);
            if replay == 0 && b.chance(2) {
                replay += 1;
            }
            assert_eq!(count, replay, "count at seed {seed}");
            assert_eq!(after_count, b.next_u32(), "count draws at seed {seed}");

            let mut c = Rng(seed);
            let attempts = patch_tree_count(&world, &mut c);
            let after_attempts = c.next_u32();
            let mut d = Rng(seed);
            let replay2 = d.range(0, 4);
            assert_eq!(attempts, replay2, "attempts at seed {seed}");
            assert_eq!(
                after_attempts,
                d.next_u32(),
                "attempts draws at seed {seed}"
            );
        }
    }

    #[test]
    fn the_drunk_world_grows_more_living_wood() {
        // Both ladders wired into the pass: over a seed sweep the drunk
        // worlds grow strictly more living wood than the classic ones on the
        // same seeds (the classic runs are the control).
        let mut classic_total = 0usize;
        let mut drunk_total = 0usize;
        let surface = flat_surface(4200);
        for seed in 1..=8u64 {
            let mut classic = grassland(4200, 1200);
            plant_living_trees(&mut classic, &mut Rng(seed), &surface, 320, 3880, (0, 0));
            classic_total += living_wood(&classic);

            let mut drunk = grassland(4200, 1200);
            drunk.seeds |= crate::seeds::SeedFlags::DRUNK;
            plant_living_trees(&mut drunk, &mut Rng(seed), &surface, 320, 3880, (0, 0));
            drunk_total += living_wood(&drunk);
        }
        assert!(
            drunk_total > classic_total,
            "drunk {drunk_total} wood tiles vs classic {classic_total}"
        );
    }

    /// A 4200-wide world (num3 = 1.0) carrying the `extraLivingTrees` micro
    /// seed and nothing else — the port stands in for `activeSecretSeedCount`.
    fn extra_trees_world() -> World {
        let mut world = World::new("lt".into(), 4200, 1200);
        world.micro_seeds |= crate::seeds::MicroSeedFlags::EXTRA_LIVING_TREES;
        world
    }

    #[test]
    fn the_extra_living_trees_count_ladder_adds_thirty_per_world_unit() {
        // `WorldGen.cs:15494-15499`: `num4 += (int)(30.0 * num3)` when
        // `!Variations.extraLivingTreesReducedAmount`. A lone
        // `extraLivingTrees` world has count 1 (< 6), and the port's noSurface
        // bit is clear, so the variation is FALSE — the 30 arm.
        let classic = World::new("lt".into(), 4200, 1200);
        let extra = extra_trees_world();
        for seed in [1u64, 7, 42, 5162020] {
            let base = living_tree_count(&classic, &mut Rng(seed));
            let added = living_tree_count(&extra, &mut Rng(seed));
            assert_eq!(added, base + 30, "seed {seed}");
        }
        // The reduced arm (3 per world unit) when the seed list is crowded:
        // the port reaches `activeSecretSeedCount >= 6` with five more micro
        // bits beside `extraLivingTrees`.
        let mut crowded = extra_trees_world();
        crowded.micro_seeds |= crate::seeds::MicroSeedFlags::PAINT_EVERYTHING_GRAY
            | crate::seeds::MicroSeedFlags::ERROR_WORLD
            | crate::seeds::MicroSeedFlags::RAINS_FOR_A_YEAR
            | crate::seeds::MicroSeedFlags::RANDOM_SPAWN
            | crate::seeds::MicroSeedFlags::ADD_TELEPORTERS;
        assert_eq!(crowded.micro_seeds.count(), 6);
        for seed in [1u64, 7, 42, 5162020] {
            let base = living_tree_count(&classic, &mut Rng(seed));
            let reduced = living_tree_count(&crowded, &mut Rng(seed));
            assert_eq!(reduced, base + 3, "seed {seed}");
        }
    }

    #[test]
    fn the_extra_living_trees_patch_ladder_adds_five_to_ten() {
        // `WorldGen.cs:15620-15623`: `num8 += genRand.Next(5, 11)` on top of
        // the `Next(4)` per-side count.
        let classic = World::new("lt".into(), 4200, 1200);
        let extra = extra_trees_world();
        for seed in [1u64, 5, 9] {
            let base = patch_tree_count(&classic, &mut Rng(seed));
            let added = patch_tree_count(&extra, &mut Rng(seed));
            assert!(
                (5..=10).contains(&(added - base)),
                "seed {seed}: {base} + {added}"
            );
        }
    }

    #[test]
    fn the_extra_living_trees_exclusion_band_narrows_to_150() {
        // `WorldGen.cs:15482` / `:15496`: `num = 200` → `num = 150`. The band
        // gate itself, read directly (the classic world is the control), plus
        // the pass-level effect: a canopy can hang 50 tiles over the band, so
        // no living wood may appear inside `half - (exclusion - 50)`.
        let classic = World::new("lt".into(), 4200, 1200);
        let extra = extra_trees_world();
        assert_eq!(living_tree_exclusion(&classic), 200);
        assert_eq!(living_tree_exclusion(&extra), 150);
        let surface = flat_surface(4200);
        let wood_in_band = |world: &World, lo: i32, hi: i32| -> usize {
            (lo..hi)
                .flat_map(|x| {
                    (0..1200).filter_map(move |y| {
                        world
                            .tile(x, y)
                            .filter(|t| t.active() && matches!(t.kind, 191 | 192))
                    })
                })
                .count()
        };
        for seed in 1..=8u64 {
            let mut flagged = grassland(4200, 1200);
            flagged.micro_seeds |= crate::seeds::MicroSeedFlags::EXTRA_LIVING_TREES;
            plant_living_trees(&mut flagged, &mut Rng(seed), &surface, 320, 3880, (0, 0));
            assert_eq!(
                wood_in_band(&flagged, 2050, 2151),
                0,
                "seed {seed}: living wood inside the flagged 150 band"
            );
            let mut classic_world = grassland(4200, 1200);
            plant_living_trees(&mut classic_world, &mut Rng(seed), &surface, 320, 3880, (0, 0));
            assert_eq!(
                wood_in_band(&classic_world, 2000, 2201),
                0,
                "seed {seed}: living wood inside the classic 200 band"
            );
        }
    }

    #[test]
    fn the_extra_living_trees_world_grows_more_living_wood() {
        // Both ladders wired into the pass, with the classic worlds as the
        // control on the same seeds.
        let mut classic_total = 0usize;
        let mut extra_total = 0usize;
        let surface = flat_surface(4200);
        for seed in 1..=8u64 {
            let mut classic = grassland(4200, 1200);
            plant_living_trees(&mut classic, &mut Rng(seed), &surface, 320, 3880, (0, 0));
            classic_total += living_wood(&classic);

            let mut extra = grassland(4200, 1200);
            extra.micro_seeds |= crate::seeds::MicroSeedFlags::EXTRA_LIVING_TREES;
            plant_living_trees(&mut extra, &mut Rng(seed), &surface, 320, 3880, (0, 0));
            extra_total += living_wood(&extra);
        }
        assert!(
            extra_total > classic_total,
            "extraLivingTrees {extra_total} wood tiles vs classic {classic_total}"
        );
    }

    #[test]
    fn the_cloud_family_no_longer_blocks_an_extra_trees_site() {
        // `WorldGen.cs:15577-15592`: the anchor site scan's
        // `case 189/196/460/717/718/719` arm stops rejecting when
        // `extraLivingTrees` is on. The port folds the site gate into
        // `grow_living_tree`'s clearance scan, so a cloud strip at the left
        // edge of the scan window (rows above the anchor) rejects the classic
        // site while the flagged one grows. The strip sits 48 columns left of
        // the anchor, outside the trunk's own reach, so it cannot abort the
        // body for any other reason.
        let mut classic = grassland(4200, 1200);
        let mut extra = grassland(4200, 1200);
        extra.micro_seeds |= crate::seeds::MicroSeedFlags::EXTRA_LIVING_TREES;
        for l in 150..190 {
            let i = classic.index(1150, l);
            classic.tiles[i].kind = 189;
            classic.tiles[i].flags |= crate::tile::FLAG_ACTIVE;
            let j = extra.index(1150, l);
            extra.tiles[j].kind = 189;
            extra.tiles[j].flags |= crate::tile::FLAG_ACTIVE;
        }
        let (x, anchor_y) = (1200, 199);
        assert!(
            !grow_living_tree(&mut classic, &mut Rng(1), x, anchor_y, false),
            "the classic world must reject the cloud-blocked site"
        );
        assert!(
            grow_living_tree(&mut extra, &mut Rng(1), x, anchor_y, false),
            "the extraLivingTrees world must accept it"
        );
    }
}
