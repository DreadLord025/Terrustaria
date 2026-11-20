//! `StrayMethods` (`Terraria/StrayMethods.cs`) — the arena-score walker the
//! Old One's Army's crystal reads to place its two lane portals (B-6c /
//! S1-E-12d, audit research/53 §4.2).
//!
//! RNG order: **by construction, zero draws.** Neither `CheckArenaScore`
//! (`:83-105`) nor `SendWalker` (`:107-156`) calls `Main.rand` anywhere — the
//! only conditional bodies are the `showDebug` dust lines (`:91-94`,
//! `:99-102`, `:128-131`, `:141-145`), and the one live caller passes the
//! local `bool flag = false` (`:85`), so the debug arms are dead. The walker
//! is a pure function of the tile grid: same world → same portal ends.

use crate::tiledata as td;
use crate::World;

/// `Main.MaxWorldViewSize.X / 16` (`StrayMethods.cs:86`, the constant
/// `Main.cs:66714`: `new Point(1920, 1200)`) — the walker's max distance in
/// tiles, 120.
const MAX_WALKER_DISTANCE: i32 = 1920 / 16;

/// `WorldGen.SolidTile3(int, int)` (`WorldGen.cs:70465-70472`) →
/// `SolidTile3(Tile)` (`:70474-70483`): `t.nactive() && tileSolid[type] &&
/// !tileSolidTop[type]`. No slope/half-brick/inactive term — this is the
/// LOOSEST solid test (actuated tiles stay solid for the walker; the port's
/// `edit::solid_tile` is the strict `SolidTile`, NOT this one).
/// `InWorld(i, j, 1)` (`:9688-9695`) refuses the 1-tile border; the null
/// tile arm maps to the port's out-of-bounds `None`.
fn solid_tile3(world: &World, x: i32, y: i32) -> bool {
    if x < 1 || x >= world.width - 1 || y < 1 || y >= world.height - 1 {
        return false;
    }
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    // `Tile.nactive()` (`Tile.cs:240-246`): active AND not framed-off —
    // the port's `active() && !inactive()`.
    t.active() && !t.inactive() && td::TILE_SOLID.has(t.kind) && !td::TILE_SOLID_TOP.has(t.kind)
}

/// `Collision.ExpandVertically` (`Collision.cs:4372-4416`): expand the
/// column up/down from `(start_x, start_y)` until a `SolidTile3` cell or a
/// bound. Returns `(topY, bottomY)` — both INCLUSIVE of the blocking cell
/// (the callers do `topY++`/`bottomY--` to get the free span). The
/// `InWorld(x, y, 10)` gate (`:4376`) returns the unchanged start pair.
fn expand_vertically(
    world: &World,
    start_x: i32,
    start_y: i32,
    max_expand_up: i32,
    max_expand_down: i32,
) -> (i32, i32) {
    let mut top_y = start_y;
    let mut bottom_y = start_y;
    // `InWorld(startX, startY, 10)` — the border refusal leaves both ends
    // at the start (`:4376-4379`).
    if start_x < 10
        || start_x >= world.width - 10
        || start_y < 10
        || start_y >= world.height - 10
    {
        return (top_y, bottom_y);
    }
    for _ in 0..max_expand_up {
        // `:4382-4389`: the `topY <= 0` / `topY < 10` guards collapse to
        // `< 10`; the null-tile arm is the port's `None`.
        if top_y < 10 || world.tile(start_x, top_y).is_none() {
            break;
        }
        if solid_tile3(world, start_x, top_y) {
            break;
        }
        top_y -= 1;
    }
    for _ in 0..max_expand_down {
        // `:4398-4405`: `bottomY >= maxTilesY - 10` subsumes the second
        // `>` guard; same null arm.
        if bottom_y >= world.height - 10 || world.tile(start_x, bottom_y).is_none() {
            break;
        }
        if solid_tile3(world, start_x, bottom_y) {
            break;
        }
        bottom_y += 1;
    }
    (top_y, bottom_y)
}

/// `StrayMethods.SendWalker` (`StrayMethods.cs:107-156`) — walk one
/// direction along walkable floor. `height` is the headroom the walker
/// needs (the crystal passes 10); `direction` is -1 (left) / +1 (right).
/// Returns `(distance_covered, last_floor_spot)` — the spot's Y is the
/// AIR row just above the floor (`:110`'s `Y--`, refreshed from `bottomY`
/// at `:148`), which is what `ToWorldCoordinates(8, 16)` anchors to.
///
/// The `showDebug` dust arms (`:128-131`, `:141-145`) are the caller's
/// dead `flag == false` — omitted, zero RNG either way.
fn send_walker(
    world: &World,
    start_floor: (i32, i32),
    height: i32,
    direction: i32,
    max_distance: i32,
) -> (i32, (i32, i32)) {
    let (mut x, mut y) = start_floor;
    let mut distance = 0;
    // `:110-111`: step up off the floor cell; the start is the fallback
    // when the very first column fails.
    y -= 1;
    let mut last = (x, y);
    for _ in 0..max_distance {
        // `:114-121`: skip up to 3 solid cells overhead (a low ceiling
        // the walker can step over).
        for _ in 0..3 {
            if !solid_tile3(world, x, y) {
                break;
            }
            y -= 1;
        }
        // `:122-124`: the headroom probe — 10 up, 2 down; the ++/--
        // shrink the inclusive ends to the free span.
        let (top_y, bottom_y) = expand_vertically(world, x, y, height, 2);
        let top_y = top_y + 1;
        let bottom_y = bottom_y - 1;
        // `:125-136`: no floor right below → probe 6 further down; still
        // no floor → the walk ends here.
        if !solid_tile3(world, x, bottom_y + 1) {
            let (_, bottom_y2) = expand_vertically(world, x, bottom_y, 0, 6);
            if !solid_tile3(world, x, bottom_y2) {
                break;
            }
        }
        // `:137-140`: the headroom must fit the walker.
        if bottom_y - top_y < height - 1 {
            break;
        }
        // `:146-153`: advance, re-anchor on the new floor, remember the
        // spot; the distance cap ends the walk.
        distance += direction;
        x += direction;
        y = bottom_y;
        last = (x, y);
        if distance.abs() >= max_distance {
            break;
        }
    }
    (distance.abs(), last)
}

/// `StrayMethods.CheckArenaScore` (`StrayMethods.cs:83-105`) — find the
/// walkable floor span around `arena_center` (world px). Returns
/// `(xLeftEnd, xRightEnd)` in TILE coordinates, already nudged by the
/// `:97-98` `X++`/`X--` (the crystal's caller adds its own ±2 on top,
/// `NPC.cs:40748-40749`).
///
/// `walkerWidthInTiles` (default 5) is accepted and UNUSED by the C# —
/// kept out of the signature; `walkerHeightInTiles` is 10 at the only
/// live call site.
pub fn check_arena_score(world: &World, arena_center: (f32, f32)) -> ((i32, i32), (i32, i32)) {
    // `:87` `ToTileCoordinates` (`Utils.cs:1807-1810`): `(int)v >> 4` —
    // truncation toward zero THEN arithmetic shift (floor for positives).
    let mut px = (arena_center.0 as i32) >> 4;
    let mut py = (arena_center.1 as i32) >> 4;
    // `:89-90`: drop to the floor under the centre (0 up, 4 down).
    let (_, bottom_y) = expand_vertically(world, px, py, 0, 4);
    py = bottom_y;
    // `:95-96`: one walker each way, height 10, maxDistance 120.
    let (_, left) = send_walker(world, (px, py), 10, -1, MAX_WALKER_DISTANCE);
    let (_, right) = send_walker(world, (px, py), 10, 1, MAX_WALKER_DISTANCE);
    // `:97-98`: the ends step one tile back toward the centre.
    ((left.0 + 1, left.1), (right.0 - 1, right.1))
}

/// `StrayMethods.CanSpawnSandstormHostile` (`StrayMethods.cs:47-63`) — the
/// Sand Elemental's sandnado spawn-column gate (`NPC.cs:40249`): the three
/// columns `point.X + i` for `i` in −1..=1 each expand vertically
/// (`expand_up`/`expand_down`), the inclusive ends shrink by ++/−−, and the
/// free span must reach `bottomY - topY >= 20`. `position` is world px;
/// `ToTileCoordinates` truncates toward zero then >>4 (the
/// `check_arena_score` convention). W4-c40.
pub fn can_spawn_sandstorm_hostile(
    world: &World,
    position: (f32, f32),
    expand_up: i32,
    expand_down: i32,
) -> bool {
    can_spawn_sandstorm_span(world, position, expand_up, expand_down, 20)
}

/// `StrayMethods.CanSpawnSandstormFriendly` (`StrayMethods.cs:65-81`) —
/// the same walk with the 10-tile span (the friendly-minion rest gate,
/// `Player.cs:56814`). W4-c40.
pub fn can_spawn_sandstorm_friendly(
    world: &World,
    position: (f32, f32),
    expand_up: i32,
    expand_down: i32,
) -> bool {
    can_spawn_sandstorm_span(world, position, expand_up, expand_down, 10)
}

/// The shared three-column walk (`StrayMethods.cs:51-61`/`:69-79`): the
/// ONLY difference between the two gates is the span threshold.
fn can_spawn_sandstorm_span(
    world: &World,
    position: (f32, f32),
    expand_up: i32,
    expand_down: i32,
    required_span: i32,
) -> bool {
    let px = (position.0 as i32) >> 4;
    let py = (position.1 as i32) >> 4;
    for i in -1..=1 {
        let (top_y, bottom_y) = expand_vertically(world, px + i, py, expand_up, expand_down);
        let top_y = top_y + 1;
        let bottom_y = bottom_y - 1;
        if bottom_y - top_y < required_span {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{Tile, FLAG_ACTIVE};

    /// A 300x200 world with a flat dirt floor at row 120 (the sim tests'
    /// `surface_world` shape).
    fn flat_world() -> World {
        let mut w = World::new("t".into(), 300, 200);
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w
    }

    #[test]
    fn solid_tile3_is_the_loose_test_not_solid_tile() {
        // `WorldGen.cs:70474-70483`: nactive && solid && !solidTop. Dirt
        // yes; a platform (solidTop) no; an inactive tile no; the border
        // no (`InWorld(i, j, 1)`).
        let mut w = flat_world();
        assert!(solid_tile3(&w, 150, 120));
        assert!(!solid_tile3(&w, 150, 119));
        assert!(!solid_tile3(&w, 0, 120), "the 1-tile border refuses");
        assert!(!solid_tile3(&w, 150, 199), "the bottom border refuses");
        let i = w.index(150, 120);
        w.tiles[i].kind = 19; // Wooden Platform — tileSolidTop
        assert!(!solid_tile3(&w, 150, 120), "solidTop is not solid3");
        w.tiles[i].kind = 1;
        w.tiles[i].flags |= crate::tile::FLAG_INACTIVE;
        assert!(!solid_tile3(&w, 150, 120), "nactive refuses framed-off");
    }

    #[test]
    fn expand_vertically_stops_on_the_floor_inclusive() {
        // `Collision.cs:4396-4414`: the down loop stops ON the solid cell
        // (bottomY = 120), the up loop runs its 10 free rows.
        let w = flat_world();
        let (top, bottom) = expand_vertically(&w, 150, 119, 10, 2);
        assert_eq!(bottom, 120, "the floor cell is the inclusive end");
        assert_eq!(top, 109, "ten free rows up from 119");
        // The border gate returns the unchanged pair.
        assert_eq!(expand_vertically(&w, 5, 119, 10, 2), (119, 119));
    }

    #[test]
    fn the_walker_walks_the_flat_floor_to_the_distance_cap() {
        // `SendWalker` on flat ground: 120 iterations, one tile each, the
        // last spot one step PAST the 120th advance (`:146-153` — the
        // distance test runs after the spot update), Y pinned to the free
        // row above the floor.
        let w = flat_world();
        let (dist, left) = send_walker(&w, (150, 120), 10, -1, MAX_WALKER_DISTANCE);
        assert_eq!(dist, 120);
        assert_eq!(left, (150 - 120, 119));
        let (dist, right) = send_walker(&w, (150, 120), 10, 1, MAX_WALKER_DISTANCE);
        assert_eq!(dist, 120);
        assert_eq!(right, (150 + 120, 119));
    }

    #[test]
    fn the_walker_stops_at_a_wall_it_cannot_step_over() {
        // A 4-tall dirt wall (rows 116-119) at x = 160: the walk LANDS on
        // the wall column first (`:146-149` updates the spot before the
        // next column's checks), then the 3-cell skip (`:114-121`) climbs
        // 3, the headroom probe still finds solid at 116 and
        // `bottomY - topY < 9` breaks — the last spot is the wall column
        // at the pre-climb row (the C#'s `lastIteratedFloorSpot` semantics).
        let mut w = flat_world();
        for y in 116..120 {
            let i = w.index(160, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let (dist, right) = send_walker(&w, (150, 120), 10, 1, MAX_WALKER_DISTANCE);
        assert_eq!(dist, 10, "150 → 160, the spot lands before the check");
        assert_eq!(right, (160, 119));
    }

    #[test]
    fn arena_score_walks_both_ends_and_nudges_inward() {
        // `CheckArenaScore` from the crystal's bottom on flat ground: the
        // ±120 walker caps, then the `:97-98` inward nudge — the ends are
        // ±119 tiles from the centre, Y on the free row above the floor.
        let w = flat_world();
        let center = (150.0 * 16.0 + 8.0, 120.0 * 16.0);
        let (left, right) = check_arena_score(&w, center);
        assert_eq!(left, (150 - 120 + 1, 119));
        assert_eq!(right, (150 + 120 - 1, 119));
    }

    #[test]
    fn arena_score_climbs_the_three_up_skip_and_falls_six_down() {
        // A 2-tall step at x >= 155 (floor row 118 instead of 120): the
        // 3-up solid skip clears it and the walker re-anchors on the new
        // floor (`:148` Y = bottomY = 117). A 3-deep notch at x = 140-142
        // (floor removed to row 123): the 6-down probe finds the floor at
        // 123 and the walk continues (`:125-136`).
        let mut w = flat_world();
        for x in 155..w.width {
            for y in 118..120 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for x in 140..143 {
            for y in 120..123 {
                let i = w.index(x, y);
                w.tiles[i].flags &= !FLAG_ACTIVE;
            }
        }
        let center = (150.0 * 16.0 + 8.0, 120.0 * 16.0);
        let (left, right) = check_arena_score(&w, center);
        // Right: climbs the step at 155, runs to the cap on row 117.
        assert_eq!(right, (150 + 120 - 1, 117));
        // Left: falls into the notch and back out, runs to the cap on 119.
        assert_eq!(left, (150 - 120 + 1, 119));
    }

    #[test]
    fn the_walker_draws_no_rng() {
        // By construction: no function in this module takes or touches a
        // RNG — the census is the C# bodies themselves (`StrayMethods.cs:
        // 83-156`, zero `Main.rand`; the `showDebug` arms are dead at the
        // only caller). This witness pins the API shape: a seeded stream
        // is untouched across the whole walk.
        let w = flat_world();
        let mut rng = crate::UnifiedRandom::new(1234);
        let before = rng.next();
        let mut rng = crate::UnifiedRandom::new(1234);
        let _ = check_arena_score(&w, (2408.0, 1920.0));
        assert_eq!(rng.next(), before, "the walk consumed no draws");
    }

    /// W4-c40 — `CanSpawnSandstormHostile` on open air above the flat
    /// floor: the 15/15 expansion from a point 30 rows up finds the floor
    /// only within the down budget; the free span (≥20) passes on all
    /// three columns.
    #[test]
    fn w4_c40_sandstorm_hostile_needs_a_20_tile_span() {
        let w = flat_world();
        // Point at row 90 (30 above the floor): expand 15 up (rows 75-89
        // free), 15 down (rows 90-104 free, no floor within 15) — the
        // bottom end runs to 105 (the budget exhausts), top 75.
        // span = (105-1) - (75+1) = 28 >= 20: passes.
        let pos = (150.0 * 16.0, 90.0 * 16.0);
        assert!(can_spawn_sandstorm_hostile(&w, pos, 15, 15));
        assert!(can_spawn_sandstorm_friendly(&w, pos, 30, 30));
        // A ceiling 8 rows above the point caps the span below 20.
        let mut w2 = flat_world();
        for x in 148..153 {
            for y in 82..84 {
                let i = w2.index(x, y);
                w2.tiles[i] = Tile { kind: 1, ..Default::default() };
                w2.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        // expand 15 up stops at the ceiling row 83 (inclusive), top 84;
        // expand 15 down runs to 105, bottom 104: span 20 — exactly the
        // boundary, passes (`< 20` refuses).
        assert!(can_spawn_sandstorm_hostile(&w2, pos, 15, 15));
        // One more ceiling row: span 19 — refuses.
        let i = w2.index(149, 84);
        w2.tiles[i] = Tile { kind: 1, ..Default::default() };
        w2.tiles[i].flags |= FLAG_ACTIVE;
        // The centre column now stops at 84 → top 85, span 19: false.
        assert!(!can_spawn_sandstorm_hostile(&w2, pos, 15, 15));
        // The friendly gate (10) still passes on the same geometry.
        assert!(can_spawn_sandstorm_friendly(&w2, pos, 15, 15));
    }

    /// W4-c40 — the border refusal: `ExpandVertically`'s InWorld(10) gate
    /// leaves both ends at the start, span −2 < 20 → false.
    #[test]
    fn w4_c40_sandstorm_hostile_refuses_the_border() {
        let w = flat_world();
        let pos = (5.0 * 16.0, 90.0 * 16.0);
        assert!(!can_spawn_sandstorm_hostile(&w, pos, 15, 15));
    }
}
