//! `WorldGen`'s "Living Trees" pass (`WorldGen.cs:15476-15715`) and
//! `GrowLivingTree` (`WorldGen.cs:27973-28627`): the big hollow trees built
//! from living wood (tile 191) and leaf blocks (tile 192), with wall 244
//! (`LivingWoodUnsafe`) lining the interiors, the descending passage and the
//! wood-tree walls.
//!
//! Simplifications against the source, all recorded here:
//! * The secret-seed variants (extra/drunk/bees/remix/anniversary), the
//!   skyblock/remix layouts and the dungeon-wall checks are not modelled.
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

/// The "Living Trees" pass (`WorldGen.cs:15476-15715`).
pub(crate) fn plant_living_trees(world: &mut World, rng: &mut Rng, surface: &[i32], beach: i32) {
    // `WorldGen.cs:15482-15492`: the count scales with the world width.
    let num3 = world.width as f64 / 4200.0;
    let mut count = rng.range(0, (2.0 * num3) as i32 + 1);
    if count == 0 && rng.chance(2) {
        count += 1;
    }
    // `WorldGen.cs:15482`: the spawn-area exclusion half-width.
    let exclusion = 200;
    let half = world.width / 2;
    for _ in 0..count {
        // `WorldGen.cs:15543-15546`: the anchor attempts, up to maxTilesX/2.
        let mut anchor = None;
        for _ in 0..world.width / 2 {
            let x = rng.range(beach, world.width - beach);
            // `WorldGen.cs:15537`: grow OUTSIDE the spawn plateau (the anchor
            // must satisfy `x <= half - num || x >= half + num`).
            if x > half - exclusion && x < half + exclusion {
                continue;
            }
            // `WorldGen.cs:15546`: scan down to the first active tile.
            let column_top = surface[x as usize];
            let mut y: i32 = 0;
            while y <= column_top
                && world
                    .tile(x, y)
                    .is_none_or(|t| !t.active())
            {
                y += 1;
            }
            if y > column_top {
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
            // `WorldGen.cs:15549-15552`: the anchor sits in the air tile above
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
            anchor = Some((x, anchor_y));
            break;
        }
        let Some((x, y)) = anchor else {
            continue;
        };
        if !grow_living_tree(world, rng, x, y, false) {
            continue;
        }
        // `WorldGen.cs:15611-15701`: the patch trees step outward from the
        // anchor, `Next(4)` attempts per side, `Next(13, 31)` tiles apart.
        for side in [-1i32, 1] {
            let attempts = rng.range(0, 4);
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
    let (x0, x1) = if patch { (i - 20, i + 20) } else { (i - 50, i + 50) };
    for k in x0..=x1 {
        for l in 20..j - 5 {
            if let Some(t) = world.tile(k, l) {
                if t.active() && (!patch || !matches!(t.kind, 2 | 0 | 1 | 191 | 192)) {
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
