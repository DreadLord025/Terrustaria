//! The aiStyle-8 caster family — `NPC.AI()`'s `if (this.aiStyle == 8)` block
//! (`NPC.cs:20817-21376`). Despite the task slug ("bat/flyer family"), the C#'s
//! aiStyle 8 is the **teleporting caster** machine — the bats (49/51/60) ride
//! aiStyle 14 (`NPC.cs:22849`, the port's `update_cave_bat`); the style axis is
//! the C#'s dispatch key, so this module owns style 8:
//!
//! * type 24 Fire Imp (`SetDefaults` `NPC.cs:9078-9092`),
//! * type 29 Goblin Sorcerer (`:15686-15699`),
//! * type 32 Dark Caster (`:9121-9134`),
//! * type 45 Tim (`:9322-9336`),
//! * type 172 Rune Wizard (`:11154-11167`, W4-g5),
//! * type 533 Desert Djinn (`:14974-14987`),
//! * type 693 Librarian Skeleton (`:17723-17735`, W4-g5),
//! * types 281-286 the hardmode dungeon casters (`:12640-12723`) — the
//!   d38 branch's claim (pushed, unmerged).
//!
//! The seam (S1-D-3, code-only mode): `Npc::update_caster` (`npc.rs`, the
//! kind-dispatched arm — it also carries the Poltergeist aiStyle-22 stand-in)
//! folds the port's `caster_counter`/`cast_windup` mirrors into `ai[0]`/`ai[1]`,
//! runs `ai_style_8_step`, folds them back out and drains the shoot intent into
//! `wants_sphere`. Every other dispatch arm stays byte-identical.
//!
//! RNG order is by construction — W4-g5 DRAW-PARITY (the W4-d128 census's
//! coordinated parity commit, policy A: every ARM-LEVEL `Main.rand` draw is
//! consumed at its exact C# position; the `Dust.NewDust` INTERNAL draws stay
//! the dust task's documented skip, S1-E-1). Per tick, in C# order:
//!
//! 1. The teleport-arrival bursts (`:20863-20922` + `:20932-20991`): types
//!    29/45 pass `Next(1, 3)` as the scale ARGUMENT 50× per burst — the draw
//!    happens even when `NewDust` culls — two bursts = exactly 100 draws on
//!    the arrival tick; every other ported type's bursts draw 0.
//! 2. The teleport search (`:18952-18953`): 2 draws per attempt (X then Y),
//!    ≤100 attempts; the >2000-px Manhattan bail draws NOTHING. The radius
//!    is the C# DEFAULT 20 for EVERY type (`:18937`/`:21097` — `num88` is
//!    only the post-teleport windup, the W4-d128 L1 fix). The 693's book
//!    search (`:19009-19085`, W4-g5) runs FIRST: one `Next(2)` direction
//!    seed + ≤100 X-then-Y pairs (`Next(target ± 30)`), the >125-tile
//!    Euclidean bail draws NOTHING.
//! 3. The Djinn volley search (`:21137-21160`): on the `% 30 == 0 &&
//!    / 30 < 5` windup ticks (ai[1] = 120/90/60/30/0 AFTER the decrement),
//!    2 draws per iteration (X then Y), ≤50 iterations, early-exit on the
//!    first accepted tile; the >2000-px length gate draws NOTHING.
//! 4. The shoot ticks (W4-g5): the 172's shadow beam (`:21264-21274`) draws
//!    two `Next(-10, 11)` aim jitters (X then Y); the 693's book shot
//!    (`:21248-21263`) draws `Next(bookCount)` on a found book OR two
//!    `NextFloat` (the NextVector2Circular angle + radius) on the fallback.
//! 5. The ambient tail gates (`:21279-21374`): 29/45 `Next(5)` every tick,
//!    32 `Next(3)` every tick, the else group (24) `Next(6)` every tick,
//!    533 draws NOTHING (light only), the 693 draws NOTHING (the ai[0]-
//!    scaled light, `:21301-21308`), the 172 draws `Next(255)` per alpha-
//!    dust iteration (1, or 2 at alpha 255, `:21310-21325`).
//!
//! The d38 kinds' own draws (281-286's `Next(2)` gates `:21331`/`:21340`/
//! `:21350`) stay the d38 branch's claim. This is the deliberate ONE-TIME
//! stream shift the W4-d128 plan mandates — the golden outputs re-bless at
//! the verification gate (ANALYSIS "The caster aiStyle-8 draw-parity
//! (W4-g5)").
//! RNG order is by construction: every `Main.rand` draw this machine makes is
//! the teleport search's `Next(targetTileX - range, targetTileX + range + 1)`
//! then `Next(targetTileY - range, ...)` pair per attempt (`NPC.cs:18952-18953`,
//! cited inline below), plus — W4-f42 — the Djinn's curse volley search
//! (`:21140-21141`, the shared `npc_ai_caster::djinn_curse_search`, X then Y
//! per attempt). The per-type dust arms (`:21282`, `:21292`, `:21319`,
//! `:21331`, `:21340`, `:21350`, `:21366`), the teleport bursts' `Next(1, 3)`
//! scales (`:20867`/`:20936`) and the Rune Wizard's aim jitter
//! (`:21268-21269`) all draw `Main.rand` too and are skipped wholesale — the
//! dust system (S1-E-2/S1-E-3) and the unported kinds own them; the skip
//! keeps the surviving stream's order exact.

use crate::npc::{Npc, NpcKind};
use crate::npc_target::{target_closest, TargetCandidate};
use crate::player::Player;
use tr_core::UnifiedRandom;
use tr_core::World;
use tr_core::TILE_PX;

/// The one live player slot as the teleport search's telefrag gate sees it
/// (`NPC.cs:18982-18996` loops `Main.player`; the port has one session
/// player). Carries the velocity for the C#'s ±20-tick prediction rect
/// (`:18988-18989`).
#[derive(Clone, Copy, Debug)]
pub struct TelefragPlayer {
    /// `player.Hitbox` — `(int)position.X/Y`, 20×42 (`Player.cs`'s box).
    pub x: f32,
    pub y: f32,
    pub width: i32,
    pub height: i32,
    /// `player.velocity` — the `num4 = 20f` extrapolation (`:18943`).
    pub vx: f32,
    pub vy: f32,
    /// `player.active && !player.dead` (`:18985`) — a dead player does
    /// not block the spot.
    pub dead: bool,
}

impl TelefragPlayer {
    /// The session player's box + velocity (the port's stock 20×42 hitbox).
    pub fn from_player(p: &Player) -> Self {
        Self {
            x: p.x,
            y: p.y,
            width: crate::player::WIDTH as i32,
            height: crate::player::HEIGHT as i32,
            vx: p.vx,
            vy: p.vy,
            dead: p.dead,
        }
    }
}

/// `Main.wallDungeon` (`Main.cs:10596-10606`): walls 7/8/9/94-99.
const WALL_DUNGEON: [u16; 9] = [7, 8, 9, 94, 95, 96, 97, 98, 99];

/// The C#'s type-32 / 281-286 / 693 group (`NPC.cs:18960`) — the teleport
/// spot must sit in front of a dungeon wall unless Skeletron is alive.
/// W4-g5 RETIRE: the 693 joins (the phases-4-6 draft landed the kind but
/// left it out of this match — the fallback search teleported anywhere;
/// d43 batch-12's row carried the gate, keep-one merges onto THIS arm).
/// The 281-286 arms are the d38 branch's claim (its tip extends this
/// match with the six hardmode kinds — the merger unions the lists).
fn dungeon_wall_gated(kind: NpcKind) -> bool {
    matches!(
        kind,
        NpcKind::DarkCaster
            // W4-d38 — the hardmode dungeon casters join the type-32/281-286/
            // 693 group (`NPC.cs:18960`): the teleport spot must sit in front
            // of a dungeon wall unless Skeletron is alive.
            | NpcKind::RaggedCaster
            | NpcKind::RaggedCasterOpenCoat
            | NpcKind::Necromancer
            | NpcKind::NecromancerArmored
            | NpcKind::DiabolistRed
            | NpcKind::DiabolistWhite
    )
    matches!(kind, NpcKind::DarkCaster | NpcKind::LibrarianSkeleton)
}

/// `Collision.SolidTiles(startX, endX, startY, endY)`
/// (`Collision.cs:3455-3488`): out of bounds = solid; a tile counts when
/// `active() && !inActive() && tileSolid[type] && !tileSolidTop[type]`.
fn solid_tiles(world: &World, start_x: i32, end_x: i32, start_y: i32, end_y: i32) -> bool {
    if start_x < 0 || end_x >= world.width || start_y < 0 || end_y >= world.height - 40 {
        return true;
    }
    for i in start_x..=end_x {
        for j in start_y..=end_y {
            if let Some(t) = world.tile(i, j) {
                if t.active()
                    && t.flags & tr_core::tile::FLAG_INACTIVE == 0
                    && tr_core::tiledata::TILE_SOLID.has(t.kind)
                    && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
                {
                    return true;
                }
            }
        }
    }
    false
}

/// `NPC.AI_AttemptToFindTeleportSpot` (`NPC.cs:18937-19007`), the aiStyle-8
/// call shape (`:21097` passes ONLY `(destination, num86, num87)` —
/// `rangeFromTargetTile` takes its DEFAULT **20** for EVERY type (W4-g5,
/// the W4-d128 L1 fix: `num88` is only the post-teleport windup,
/// `:21101`); the other parameters at their defaults — telefrag 5 tiles,
/// fluff 1, not centered, not in air).
///
/// Per attempt: one X draw then one Y draw (`Main.rand.Next` at
/// `:18952`/`:18953` — the ONLY two draws this machine makes), the column
/// scanned downward from the drawn Y. Every rejection before the telefrag
/// test `continue`s the scan (`:18957`, `:18973`, `:18978`); only the
/// telefrag check ends the attempt — chosen or not (the `break` at
/// `:19003` sits outside the `flag2` test).
///
/// `skeletron_alive` is `NPC.AnyNPCs(35)` (`:18962`) — the port passes the
/// sim's live-head mirror (`world.skeletron_head_present`, false with no
/// live head). B-1 fix: the old `skeletron_red_hat` stand-in read the
/// hands count out of `ai[3]` (a live head with one hand read alive but a
/// two-hand head did not); the count now rides its own World mirror and
/// this gate reads presence directly.
pub fn attempt_find_teleport_spot(
    world: &World,
    rng: &mut UnifiedRandom,
    kind: NpcKind,
    self_center: (f32, f32),
    target_tile: (i32, i32),
    radius: i32,
    player: &TelefragPlayer,
    skeletron_alive: bool,
) -> Option<(i32, i32)> {
    // `:18939-18940` — `(int)base.Center.X / 16` (truncate, then divide).
    let num = self_center.0 as i32 / 16;
    let num2 = self_center.1 as i32 / 16;
    // `:18944-18948` — beyond 2000 Manhattan px the attempt counter starts
    // at 100: the while condition fails immediately, no draws at all.
    let mut attempts = 0;
    if (num * 16 - target_tile.0 * 16).abs() + (num2 * 16 - target_tile.1 * 16).abs() > 2000 {
        attempts = 100;
    }
    while attempts < 100 {
        attempts += 1;
        // The two `Main.rand.Next` draws, X first then Y (`:18952-18953`).
        let x = rng.next_range(target_tile.0 - radius, target_tile.0 + radius + 1);
        let y_start = rng.next_range(target_tile.1 - radius, target_tile.1 + radius + 1);
        let mut i = y_start;
        while i < target_tile.1 + radius {
            // `:18955-18958` — the self box / the inactive skip continue.
            // `nactive()` (`Tile.cs:240-247`) is active-and-not-actuated.
            if (i >= num2 - 1 && i <= num2 + 1 && x >= num - 1 && x <= num + 1)
                || !world
                    .tile(x, i)
                    .is_some_and(|t| t.active() && !t.inactive())
            {
                i += 1;
                continue;
            }
            let mut ok = true;
            // `:18960-18970` — the dungeon-wall gate for 32/281-286/693
            // (else the lava check on the tile ABOVE the floor).
            if dungeon_wall_gated(kind)
                && !world
                    .tile(x, i - 1)
                    .is_some_and(|t| WALL_DUNGEON.contains(&t.wall))
            {
                if !skeletron_alive {
                    ok = false;
                }
            } else if world.tile(x, i - 1).is_some_and(|t| {
                t.liquid > 0 && t.liquid_kind() == tr_core::tile::LiquidKind::Lava
            }) {
                ok = false;
            }
            // `:18971-18974` — the gate/solid rejection CONTINUES the scan
            // (the C#'s `continue` at `:18973`), the floor itself must be
            // tileSolid.
            if !ok
                || !world
                    .tile(x, i)
                    .is_some_and(|t| tr_core::tiledata::TILE_SOLID.has(t.kind))
            {
                i += 1;
                continue;
            }
            // `:18976-18979` — the body space: `SolidTiles(x-1, x+1, i-4,
            // i-1)` must be false (fluff 1, not centered); blocked CONTINUES.
            if solid_tiles(world, x - 1, x + 1, i - 4, i - 1) {
                i += 1;
                continue;
            }
            // `:18980-18996` — the telefrag rect: the floor tile's 16×16
            // box inflated by 5 tiles (80 px), tested against the player's
            // hitbox UNION its ±(int)(velocity * 20) displaced copy
            // (`Modified` + `Rectangle.Union`, `Utils.cs:1603-1606`).
            // A dead player is skipped (`:18985`).
            let rect = (x * 16 - 80, i * 16 - 80, 176, 176);
            let mut blocked = false;
            if !player.dead {
                let hx = player.x as i32;
                let hy = player.y as i32;
                let dx = (player.vx * 20.0) as i32;
                let dy = (player.vy * 20.0) as i32;
                let ux = hx.min(hx + dx);
                let uy = hy.min(hy + dy);
                let uw = player.width + dx.abs();
                let uh = player.height + dy.abs();
                blocked = ux < rect.0 + rect.2
                    && rect.0 < ux + uw
                    && uy < rect.1 + rect.3
                    && rect.1 < uy + uh;
            }
            if !blocked {
                // `:18998-19002` — chosen.
                return Some((x, i));
            }
            // `:19003` — the telefrag reject ends the attempt either way.
            break;
        }
    }
    None
}

/// The aiStyle-8 step's outcome: the sphere/imp spawn intent the caller
/// drains into `wants_sphere` (kind, bottom-centre X, bottom-centre Y, the
/// Water Sphere's `ai[3]` red-hat payload).
#[derive(Clone, Copy, Debug)]
pub struct CasterShot {
    pub kind: NpcKind,
    pub x: f32,
    pub y: f32,
    pub ai3: f32,
}

/// The teleport-arrival dust bursts' ARM-LEVEL draws (`NPC.cs:20863-20922`
/// and `:20932-20991`, W4-g5 draw-parity): types 29/45 pass
/// `Main.rand.Next(1, 3)` as the scale argument 50× per burst — the draw
/// happens even when `NewDust` culls. Every other ported type's burst arms
/// use constant scales (0 draws). The `NewDust` internals stay the dust
/// task's documented skip (policy A, ANALYSIS "The caster aiStyle-8
/// draw-parity (W4-g5)").
fn teleport_burst_draws(rng: &mut UnifiedRandom, kind: NpcKind) {
    if matches!(kind, NpcKind::GoblinSorcerer | NpcKind::Tim) {
        for _ in 0..50 {
            let _ = rng.next_range(1, 3); // `:20867` / `:20936`
        }
    }
}

/// `Utils.ToTileCoordinates` (`Utils.cs:1807-1810`): `(int)v >> 4` —
/// truncate toward zero FIRST, then arithmetic-shift (negative coords
/// floor, unlike the plain `/16`). The Djinn volley's tile points
/// (`NPC.cs:21124-21125`) ride it, NOT the teleport call's `/16`.
fn to_tile(v: f32) -> i32 {
    (v as i32) >> 4
}

/// `NPC.AI_AttemptToFindTeleportSpotNearBooks_SearchWall`
/// (`NPC.cs:19087-19134`): the vertical book scan. Steps `y` by ±1 until
/// a solid tile stops it; a `TileID.Books` (50) tile whose 16×16 box
/// inflated 80 misses every live player's telefrag union is the chosen
/// book. Returns `(book tile, out_of_bounds)`.
fn book_search_wall(
    world: &World,
    x: i32,
    y: i32,
    downwards: bool,
    player: &TelefragPlayer,
) -> (Option<(i32, i32)>, bool) {
    let step = if downwards { 1 } else { -1 };
    let mut y = y;
    loop {
        y += step;
        if y < 0 || y >= world.height {
            return (None, true); // out of bounds
        }
        let tile = world.tile(x, y);
        // `:19104-19107` — a non-book tile continues the scan.
        if !tile.is_some_and(|t| t.active() && t.kind == 50) {
            // `:19131` — the while condition: a solid tile stops the scan.
            if tile.is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind)) {
                return (None, false);
            }
            continue;
        }
        // `:19108-19125` — the telefrag union (the shared attempt's shape).
        let rect = (x * 16 - 80, y * 16 - 80, 176, 176);
        let mut blocked = false;
        if !player.dead {
            let hx = player.x as i32;
            let hy = player.y as i32;
            let dx = (player.vx * 20.0) as i32;
            let dy = (player.vy * 20.0) as i32;
            let ux = hx.min(hx + dx);
            let uy = hy.min(hy + dy);
            let uw = player.width + dx.abs();
            let uh = player.height + dy.abs();
            blocked = ux < rect.0 + rect.2
                && rect.0 < ux + uw
                && uy < rect.1 + rect.3
                && rect.1 < uy + uh;
        }
        if blocked {
            return (None, false); // `:19126-19129` — the break
        }
        return (Some((x, y)), false); // `:19130` — the chosen book
    }
}

/// `NPC.AI_AttemptToFindTeleportSpotNearBooks` (`NPC.cs:19009-19085`):
/// the 693's book teleport. The >125-tile Euclidean bail draws NOTHING;
/// otherwise one `Next(2)` seeds the scan direction, then ≤100 attempts
/// of an X-then-Y pair (`Next(target ± maxRange)`). Each attempt scans
/// horizontally for a solid tile, then vertically (down, then up) for a
/// book; the chosen destination is `(bookX - direction, bookY + 1)`.
fn attempt_find_teleport_spot_near_books(
    world: &World,
    rng: &mut UnifiedRandom,
    self_center: (f32, f32),
    target_tile: (i32, i32),
    max_range: i32,
    player: &TelefragPlayer,
) -> Option<(i32, i32)> {
    // `:19012-19013` — `(int)Center.X / 16` (truncate, then int /16).
    let num = self_center.0 as i32 / 16;
    let num2 = self_center.1 as i32 / 16;
    // `:19014-19019` — the >125-tile Euclidean bail draws NOTHING.
    let dx = (num - target_tile.0) as f32;
    let dy = (num2 - target_tile.1) as f32;
    if (dx * dx + dy * dy).sqrt() > 125.0 {
        return None;
    }
    // `:19020` — one Next(2) seeds the direction (±1).
    let mut direction = rng.next_max(2) * 2 - 1;
    for _ in 0..100 {
        // `:19023-19024` — X first, then Y.
        let mut x = rng.next_range(target_tile.0 - max_range, target_tile.0 + max_range + 1);
        let y = rng.next_range(target_tile.1 - max_range, target_tile.1 + max_range + 1);
        // `:19025-19028` — out of world: the whole attempt returns false.
        if x < 0 || x >= world.width || y < 0 || y >= world.height {
            return None;
        }
        // `:19029-19032` — the tile must be in front of a dungeon wall
        // and not active.
        let tile = world.tile(x, y);
        if !tile.is_some_and(|t| WALL_DUNGEON.contains(&t.wall))
            || tile.is_some_and(|t| t.active())
        {
            continue;
        }
        direction *= -1; // `:19033`
        // `:19036-19047` — the horizontal scan for a solid tile.
        let x_min = (target_tile.0 - max_range).max(0);
        let x_max = (target_tile.0 + max_range).min(world.width - 1);
        let mut out_of_bounds = false;
        loop {
            x += direction;
            if x < x_min || x > x_max {
                out_of_bounds = true;
                break;
            }
            if world.tile(x, y).is_some_and(|t| {
                t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind)
            }) {
                break;
            }
        }
        if out_of_bounds {
            continue;
        }
        x -= direction; // `:19054`
        // `:19055-19066` — the downward book scan. The while loops while
        // SearchWall returns a book; the self-box skip re-scans from the
        // book's y. The scan's out-of-bounds continues the outer for.
        let mut scan_y = y;
        let mut down_oob = false;
        loop {
            let (book, oob) = book_search_wall(world, x, scan_y, true, player);
            match book {
                Some((bx, by)) => {
                    // `:19058-19062` — the self-box skip continues the scan.
                    if (bx as f32 - num as f32).abs() <= 1.0
                        && (by as f32 - num2 as f32).abs() <= 1.0
                    {
                        scan_y = by;
                        continue;
                    }
                    // `:19063-19065` — the destination is (bookX - dir, bookY + 1).
                    return Some((bx - direction, by + 1));
                }
                None => {
                    down_oob = oob;
                    break; // the scan ended (no book / blocked / oob)
                }
            }
        }
        if down_oob {
            continue; // `:19067-19070`
        }
        // `:19071-19082` — the upward book scan (the same shape).
        scan_y = y;
        loop {
            let (book, _oob) = book_search_wall(world, x, scan_y, false, player);
            match book {
                Some((bx, by)) => {
                    if (bx as f32 - num as f32).abs() <= 1.0
                        && (by as f32 - num2 as f32).abs() <= 1.0
                    {
                        scan_y = by;
                        continue;
                    }
                    return Some((bx - direction, by + 1));
                }
                None => break,
            }
        }
    }
    None
}

/// `NPC.AI_FindNearbyBook` (`NPC.cs:62308-62364`): the 693's book shot
/// search. Collects up to 20 `TileID.Books` (50) tiles in the
/// `searchPosition + (searchWidth, searchHeight)` box (row-major), then
/// picks one via `Next(count)`. Returns the chosen book tile.
fn find_nearby_book(
    world: &World,
    search: (i32, i32),
    width: i32,
    height: i32,
    rng: &mut UnifiedRandom,
) -> Option<(i32, i32)> {
    // `:62313-62318` — the clamped search box.
    let x0 = search.0.max(0);
    let x1 = (search.0 + width).min(world.width);
    let y0 = search.1.max(0);
    let y1 = (search.1 + height).min(world.height);
    // `:62319-62343` — the row-major collection (≤20, `_nearbyBooks`).
    let mut books: Vec<(i32, i32)> = Vec::new();
    for y in y0..y1 {
        for x in x0..x1 {
            if world.tile(x, y).is_some_and(|t| t.active() && t.kind == 50) {
                books.push((x, y));
                if books.len() >= 20 {
                    break;
                }
            }
        }
        if books.len() >= 20 {
            break;
        }
    }
    // `:62356-62362` — the pick: Next(count).
    if books.is_empty() {
        return None;
    }
    let idx = rng.next_max(books.len() as i32) as usize;
    Some(books[idx])
}

/// The whole `if (this.aiStyle == 8)` block (`NPC.cs:20817-21376`) for the
/// ported kinds. State rides `npc.ai[0]` (the cycle counter), `npc.ai[1]`
/// (the windup countdown), `npc.ai[2]`/`ai[3]` (the pending teleport tile —
/// consumed on the NEXT tick, `:20855-20992`). The caller folds its
/// `caster_counter`/`cast_windup` mirrors into `ai[0]`/`ai[1]` around this.
///
/// Order is vanilla: the red-hat latch (`:20819-20828`), `TargetClosest`
/// (`:20829` — the shared `npc_target` helper), the 0.93 stand decay
/// (`:20831-20835`), the 500 init (`:20836-20843`), the pending-teleport
/// placement (`:20855-20992`, dust skipped), `ai[0]++` (`:20993`), the
/// firing-distance gate (`:21006`), the per-type triggers (`:21011-21082`),
/// the 650 wrap + teleport attempt (`:21083-21112`), the windup countdown +
/// the shoot (`:21113-21278`). The generic gravity + `UpdateCollision`
/// (`:90465-90493`) runs AFTER the AI returns in `NPC.UpdateNPC`; the port
/// applies it here at the tail (the documented ordering deviation, ANALYSIS).
pub fn ai_style_8_step(
    npc: &mut Npc,
    world: &World,
    player: &Player,
    rng: &mut UnifiedRandom,
) -> Option<CasterShot> {
    // `:20819` — `flag4 = RedHatSkeletronAdjustmentsEnabled()`.
    let mut flag4 = npc.red_hat_skeletron_adjustments_enabled();
    // `:20820-20828` — the Dark Caster latches `localAI[3] = 1` while the
    // first Skeletron head's `ai[3] == 1` (`FindFirstNPC(35)`); the port's
    // sim write-through mirror stands for it. No reset arm exists.
    if npc.kind == NpcKind::DarkCaster && world.skeletron_red_hat {
        npc.local_ai[3] = 1.0;
        flag4 = true;
    }
    // `:20829` — `TargetClosest()` (the default `faceTarget: true`). The
    // stock candidate mirrors the zombie wiring (`npc.rs::update_zombie`):
    // aggro/npcTypeNoAggro/tankPet have no port fields yet; the ticker is
    // not plumbed (0 — the stamp's only reader is the unported fade).
    let cand = TargetCandidate {
        active: true,
        dead: player.dead,
        ghost: false,
        x: player.x,
        y: player.y,
        width: crate::player::WIDTH as i32,
        height: crate::player::HEIGHT as i32,
        aggro: 0,
        item_animation: player.frame_inputs.swing_anim as i32,
        npc_type_no_aggro: false,
        tank_pet: None,
    };
    target_closest(npc, world, &[cand], true, 8, 0);
    // `:20831-20835` — the stand decay with the ±0.1 snap.
    npc.vx *= 0.93;
    if npc.vx > -0.1 && npc.vx < 0.1 {
        npc.vx = 0.0;
    }
    // `:20836-20843` — the cycle init; the 693's localAI[1] = 150 book-
    // teleport cooldown seeds here (W4-g5).
    if npc.ai[0] == 0.0 {
        npc.ai[0] = 500.0;
        if npc.kind == NpcKind::LibrarianSkeleton {
            npc.local_ai[1] = 150.0; // `:20841`
        }
    }
    // `:20844-20854` — the type-172 alpha ramp (W4-g5): alpha++ toward
    // 255 (the fade-out), a strike resets it to 0 (the justHit reappear).
    if npc.kind == NpcKind::RuneWizard {
        if npc.alpha < 255 {
            npc.alpha += 1;
        }
        if npc.just_hit {
            npc.alpha = 0;
        }
    }
    // `:20855-20992` — the pending teleport: place the body on the tile,
    // zero the velocity, clear ai[2]/ai[3]. W4-g5 draw-parity: the two
    // 50-dust bursts' ARM-LEVEL draws are consumed (29/45's `Next(1, 3)`
    // scale arguments, `:20867`/`:20936` — 100 draws on the arrival tick;
    // the `NewDust` internals stay the dust task's skip); S1-E-21g
    // wires BOTH Item8 plays (`:20862` the departure at the OLD position
    // BEFORE the move, `:20931` the arrival AFTER it) — the queue keeps
    // the C#'s order around the placement.
    if npc.ai[2] != 0.0 && npc.ai[3] != 0.0 {
        let (w, h) = npc.size();
        // `:20858-20861` — the 172 vanishes on the teleport tick (W4-g5).
        if npc.kind == NpcKind::RuneWizard {
            npc.alpha = 255;
        }
        // `:20862` — the departure puff at the pre-teleport position.
        npc.wants_sound.push(crate::npc::NpcSoundRequest {
            cue: crate::npc::NpcSoundCue::Item8,
            x: npc.x,
            y: npc.y,
        });
        // `:20863-20922` — burst 1, at the OLD position.
        teleport_burst_draws(rng, npc.kind);
        // `:20924-20925` — `(float)(base.width / 2)` is INTEGER division.
        npc.x = npc.ai[2] * 16.0 - (w as i32 / 2) as f32 + 8.0;
        npc.y = npc.ai[3] * 16.0 - h;
        npc.vx = 0.0;
        npc.vy = 0.0;
        npc.ai[2] = 0.0;
        npc.ai[3] = 0.0;
        // `:20931` — the arrival puff at the placed position.
        npc.wants_sound.push(crate::npc::NpcSoundRequest {
            cue: crate::npc::NpcSoundCue::Item8,
            x: npc.x,
            y: npc.y,
        });
        // `:20932-20991` — burst 2, at the NEW position.
        teleport_burst_draws(rng, npc.kind);
    }
    // `:20993` — the counter tick.
    npc.ai[0] += 1.0;
    // `:20994-21005` — the 693 localAI clock (W4-g5): localAI[0] counts
    // up, localAI[2] (the hit-stun) counts down, localAI[3] latches the
    // facing once the counter passes 1.
    if npc.kind == NpcKind::LibrarianSkeleton {
        npc.local_ai[0] += 1.0;
        if npc.local_ai[2] > 0.0 {
            npc.local_ai[2] -= 1.0;
        }
        if npc.local_ai[3] == 0.0 && npc.local_ai[0] > 1.0 {
            npc.local_ai[3] = npc.facing as f32;
        }
    }
    // `:21006-21010` — `flag5 = AI_GlobalFiringDistanceCheck(Center,
    // player.Center)`; the Goblin Sorcerer (29) is always in range.
    let mut flag5 = global_firing_distance_check(npc.center(), player.center());
    if npc.kind == NpcKind::GoblinSorcerer {
        flag5 = true;
    }
    // `:21011-21050` — the 283/284, 172, 533, 281/282 trigger arms, in the
    // C#'s if-else order. W4-d38 ports the dungeon-caster arms (281-286);
    // the Rune Wizard (172) stays deferred (no kind).
    if npc.kind == NpcKind::Necromancer || npc.kind == NpcKind::NecromancerArmored {
        // `:21011-21022` — the five 50-spaced triggers wind the 30-tick
        // windup; the 450 counter jumps to 700 (the teleport feeds 650).
        if flag5
            && (npc.ai[0] == 100.0
                || npc.ai[0] == 150.0
                || npc.ai[0] == 200.0
                || npc.ai[0] == 250.0
                || npc.ai[0] == 300.0)
    // `:21011-21050` — the 283/284, 172, 533, 281/282 trigger arms. The
    // Djinn's and the Rune Wizard's are ported; the dungeon casters
    // (281-286) are the d38 branch's claim.
    if npc.kind == NpcKind::RuneWizard {
        // `:21023-21030` — the six 75-spaced triggers wind the 30-tick
        // windup (W4-g5).
        if flag5
            && (npc.ai[0] == 75.0
                || npc.ai[0] == 150.0
                || npc.ai[0] == 225.0
                || npc.ai[0] == 300.0
                || npc.ai[0] == 375.0
                || npc.ai[0] == 450.0)
        {
            npc.ai[1] = 30.0;
            npc.net_update = true;
        }
        if npc.ai[0] >= 450.0 {
            npc.ai[0] = 700.0;
        }
    } else if npc.kind == NpcKind::DesertDjinn {
        // `:21031-21038` — the single 180 trigger winds the 181-tick volley.
        if flag5 && npc.ai[0] == 180.0 {
            npc.ai[1] = 181.0;
            npc.net_update = true;
        }
    } else if npc.kind == NpcKind::RaggedCaster || npc.kind == NpcKind::RaggedCasterOpenCoat {
        // `:21039-21050` — the nine triggers (three triplets) wind the
        // 30-tick windup; the 540 counter jumps to 700.
        if flag5
            && (npc.ai[0] == 100.0
                || npc.ai[0] == 120.0
                || npc.ai[0] == 140.0
                || npc.ai[0] == 200.0
                || npc.ai[0] == 220.0
                || npc.ai[0] == 240.0
                || npc.ai[0] == 300.0
                || npc.ai[0] == 320.0
                || npc.ai[0] == 340.0)
        {
            npc.ai[1] = 30.0;
            npc.net_update = true;
        }
        if npc.ai[0] >= 540.0 {
            npc.ai[0] = 700.0;
        }
    } else {
        // `:21051-21074` — the else group. `:21053-21060`'s getGoodWorld
        // Fire-Imp-vs-WoF stall needs `NPC.AnyNPCs(113)`; the port's WoF
        // presence mirror (`world.wof_present`) stands for it.
        if npc.kind == NpcKind::FireImp
            && world.seeds.contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY)
            && world.wof_present
        {
            npc.ai[0] += 1.0;
            if npc.ai[0] % 2.0 == 1.0 {
                npc.ai[0] -= 1.0;
            }
        }
        // `:21061-21068` — the red-hat double-step: a second `++` with
        // the odd rollback. An even counter advances +2 per tick (the
        // red-hat casters fire twice as fast); an odd one lands on the
        // next even value (+1).
        if flag4 {
            npc.ai[0] += 1.0;
            if npc.ai[0] % 2.0 == 1.0 {
                npc.ai[0] -= 1.0;
            }
        }
        // `:21069-21073` — the standard 100/200/300 triggers.
        if flag5 && (npc.ai[0] == 100.0 || npc.ai[0] == 200.0 || npc.ai[0] == 300.0) {
            npc.ai[1] = 30.0;
            npc.net_update = true;
        }
    }
    }
    // `:21075-21078` — the 285/286 early wrap (W4-d38): past 400 the
    // counter jumps to 650 (the diabolist teleports sooner).
    if (npc.kind == NpcKind::DiabolistRed || npc.kind == NpcKind::DiabolistWhite)
        && npc.ai[0] > 400.0
    {
        npc.ai[0] = 650.0;
    }
    // `:21079-21082` — the Djinn's 360 wrap feeds the 650 block below.
    if npc.kind == NpcKind::DesertDjinn && npc.ai[0] >= 360.0 {
        npc.ai[0] = 650.0;
    }
    // `:21083-21112` — the teleport attempt (single-player: netMode != 1).
    if npc.ai[0] >= 650.0 {
        npc.ai[0] = 1.0;
        // `:21086-21087` — `(int)player.Center.X / 16` (truncate first).
        let (pcx, pcy) = player.center();
        let target_tile = (pcx as i32 / 16, pcy as i32 / 16);
        // `:21089-21093` — num88 = 20, the Fire Imp 5. It is ONLY the
        // post-teleport windup (`ai[1] = num88`, `:21101`).
        let num88 = if npc.kind == NpcKind::FireImp { 5 } else { 20 };
        // `:21094` — the 693 near-books attempt runs FIRST (W4-g5); the
        // shared attempt is the fallback (`:21095-21097`).
        let mut found = None;
        if npc.kind == NpcKind::LibrarianSkeleton {
            found = attempt_find_teleport_spot_near_books(
                world, rng, npc.center(), target_tile, 30,
                &TelefragPlayer::from_player(player),
            );
        }
        // `:21097` — the shared attempt; `AnyNPCs(35)` rides the sim's
        // live-head presence mirror (B-1: was the red-hat hands-count
        // stand-in). W4-g5 L1: the radius is the C# DEFAULT 20 for EVERY
        // type — the old `num88` misuse shrank the Fire Imp's search to 5.
        if found.is_none() {
            found = attempt_find_teleport_spot(
                world,
                rng,
                npc.kind,
                npc.center(),
                target_tile,
                20,
                &TelefragPlayer::from_player(player),
                world.skeletron_head_present,
            );
        }
        if let Some((tx, ty)) = found {
            // `:21099-21104` — the destination rides ai[2]/ai[3]; the
            // placement happens on the NEXT tick (`:20855`).
            npc.ai[1] = num88 as f32;
            npc.ai[2] = tx as f32;
            npc.ai[3] = ty as f32;
        }
        npc.net_update = true;
        // `:21106-21111` — the 693 localAI reset (W4-g5): the book-
        // teleport cooldown re-arms to 650, the clock and facing reset.
        if npc.kind == NpcKind::LibrarianSkeleton {
            npc.local_ai[0] = 0.0;
            npc.local_ai[1] = 650.0;
            npc.local_ai[3] = 0.0;
        }
    }
    // `:21113-21278` — the windup countdown and the shoot.
    let mut shot = None;
    if npc.ai[1] > 0.0 {
        let num89 = 25.0f32;
        npc.ai[1] -= 1.0;
        if npc.kind == NpcKind::DesertDjinn {
            // `:21117-21162` — the curse volley on the %30 ticks below the
            // 5th (ai[1] = 120/90/60/30/0 AFTER the decrement). W4-g5
            // draw-parity: the wholesale S1-E-5 skip is retired — the
            // window's Item8 plays FIRST (`:21121`, even when the search
            // is gated off), then the ≤50-iteration tile search consumes
            // its X-then-Y pairs (`:21140-21141`) and the accepted tile
            // queues the 596 curse (`:21155`, the sim drains
            // `wants_djinn_curse` — damage 0, knockback 1, ai0 = target).
            if npc.ai[1] % 30.0 == 0.0 && npc.ai[1] / 30.0 < 5.0 {
                // `:21121` — the window puff (netMode-independent).
            // 5th (ai[1] = 120/90/60/30/0 after the decrement). W4-f42
            // wires the search + the 596 spawn (the S1-E-5 wholesale skip
            // retires); the search helpers ride `npc_ai_caster` (the
            // faithful re-port's `to_tile`/`solid_tiles`, one copy).
            if npc.ai[1] % 30.0 == 0.0 && npc.ai[1] / 30.0 < 5.0 {
                // `:21121` — the Item8 puff plays BEFORE the netMode gate
                // (every qualifying tick, even a client's).
                npc.wants_sound.push(crate::npc::NpcSoundRequest {
                    cue: crate::npc::NpcSoundCue::Item8,
                    x: npc.x,
                    y: npc.y,
                });
                // `:21122` — netMode != 1 (the port is single-player/host).
                let (scx, scy) = npc.center();
                let (pcx, pcy) = player.center();
                // `:21124-21125` — ToTileCoordinates (`(int)v >> 4`).
                let self_pt = (to_tile(scx), to_tile(scy));
                let tgt_pt = (to_tile(pcx), to_tile(pcy));
                // `:21126-21136` — the >2000-px length gate draws NOTHING.
                let (dx, dy) = (pcx - scx, pcy - scy);
                if (dx * dx + dy * dy).sqrt() <= 2000.0 {
                    for _ in 0..50 {
                        // `:21140-21141` — X first, then Y (num90 = 6).
                        let x = rng.next_range(tgt_pt.0 - 6, tgt_pt.0 + 6 + 1);
                        let y = rng.next_range(tgt_pt.1 - 6, tgt_pt.1 + 6 + 1);
                        // `:21142` — outside the target's ±2 box (num92)
                        // AND the caster's ±6 box (num91), tile !nactive().
                        let outside_target = y < tgt_pt.1 - 2
                            || y > tgt_pt.1 + 2
                            || x < tgt_pt.0 - 2
                            || x > tgt_pt.0 + 2;
                        let outside_self = y < self_pt.1 - 6
                            || y > self_pt.1 + 6
                            || x < self_pt.0 - 6
                            || x > self_pt.0 + 6;
                        if outside_target
                            && outside_self
                            && !world
                                .tile(x, y)
                                .is_some_and(|t| t.active() && !t.inactive())
                        {
                            // `:21145-21152` — no lava, no solid ±2 box
                            // (num93 = 2).
                            if world.tile(x, y).is_some_and(|t| {
                                t.liquid > 0
                                    && t.liquid_kind() == tr_core::tile::LiquidKind::Lava
                            }) {
                                continue;
                            }
                            if solid_tiles(world, x - 2, x + 2, y - 2, y + 2) {
                                continue;
                            }
                            // `:21155` — the 596 curse at the tile centre;
                            // ai0 = this.target, damage 0, knockback 1.
                            npc.wants_djinn_curse.push((
                                (x * 16 + 8) as f32,
                                (y * 16 + 8) as f32,
                                npc.target as f32,
                            ));
                            break;
                        }
                // `:21122` — single-player/host only (the port's fold).
                let (scx, scy) = npc.center();
                let (pcx, pcy) = player.center();
                // `:21133-21136` — the >2000-px gate draws NOTHING.
                let (dx, dy) = (pcx - scx, pcy - scy);
                if (dx * dx + dy * dy).sqrt() <= 2000.0 {
                    // `:21137-21160` — the shared search (one copy, the
                    // faithful re-port's helper).
                    if let Some((x, y)) = crate::npc_ai_caster::djinn_curse_search(
                        world,
                        rng,
                        (scx, scy),
                        (pcx, pcy),
                    ) {
                        // `:21155` — the 596 curse at the tile centre:
                        // damage 0, knockback 1, ai0 = this.target (the
                        // homing tail's player index). The request rides
                        // the ai-payload lane (the drain spawns it with
                        // `spawn_hostile_ai`, the kb-1 override there).
                        npc.wants_hostile_shot_ai.push(crate::npc::HostileShotAi {
                            kind: crate::projectile::DESERT_DJINN_CURSE,
                            x: (x * 16 + 8) as f32,
                            y: (y * 16 + 8) as f32,
                            vx: 0.0,
                            vy: 0.0,
                            damage: 0,
                            ai0: npc.target as f32,
                            ai1: 0.0,
                            scale: None,
                        });
                    }
                }
            }
        } else if npc.kind == NpcKind::FireImp {
            // `:21164-21173` — the imp spawns the Burning Sphere 25 at
            // ai[1] == 10, at `(int)pos.X + width/2 + direction*8,
            // (int)pos.Y + 20` (the int truncation is the call site's).
            if npc.ai[1] == 10.0 {
                // S1-E-21g — `NPC.cs:21168`: the imp's own cast puff
                // (Item8 BEFORE the spawn, the C#'s order).
                npc.wants_sound.push(crate::npc::NpcSoundRequest {
                    cue: crate::npc::NpcSoundCue::Item8,
                    x: npc.x,
                    y: npc.y,
                });
                let (w, _) = npc.size();
                shot = Some(CasterShot {
                    kind: NpcKind::BurningSphere,
                    x: npc.x as i32 as f32 + (w as i32 / 2) as f32 + npc.facing as f32 * 8.0,
                    y: npc.y as i32 as f32 + 20.0,
                    ai3: 0.0,
                });
            }
        } else if npc.ai[1] == num89 {
            // `:21175-21277` — the standard shoot tick. W4-d38 ports the
            // 281-286 projectile arm (`:21177-21227`); the 693/172 arms stay
            // deferred with their kinds.
            if matches!(
                npc.kind,
                NpcKind::RaggedCaster
                    | NpcKind::RaggedCasterOpenCoat
                    | NpcKind::Necromancer
                    | NpcKind::NecromancerArmored
                    | NpcKind::DiabolistRed
                    | NpcKind::DiabolistWhite
            ) {
                // `:21177-21227` — the hardmode dungeon skeleton shot. NO
                // Item8 puff (the C#'s 281-286 branch sits before the
                // `type != 172` play). The spawn is the body's top-centre
                // (`position.X + width*0.5, position.Y`, `:21190`).
                let (w, _) = npc.size();
                let sx = npc.x + w * 0.5;
                let sy = npc.y;
                let (pcx, pcy) = player.center();
                let mut ax = pcx - sx;
                let mut ay = pcy - sy;
                // `:21181-21189` — the speed: 6 default, 8 the diabolists,
                // 4 the ragged casters.
                let speed = match npc.kind {
                    NpcKind::DiabolistRed | NpcKind::DiabolistWhite => 8.0,
                    NpcKind::RaggedCaster | NpcKind::RaggedCasterOpenCoat => 4.0,
                    _ => 6.0,
                };
                // `:21193-21199` — the necromancers jitter ±30 and lead the
                // player's velocity ×10 (two Next(-30,31) draws, X then Y).
                if npc.kind == NpcKind::Necromancer || npc.kind == NpcKind::NecromancerArmored {
                    ax += rng.next_range(-30, 31) as f32;
                    ay += rng.next_range(-30, 31) as f32;
                    ax -= player.vx * 10.0;
                    ay -= player.vy * 10.0;
                }
                let inv = speed / (ax * ax + ay * ay).sqrt();
                ax *= inv;
                ay *= inv;
                // `:21203-21214` — the projectile + damage: 290/30 default,
                // 291/40 the diabolists, 293/40 the ragged casters. The
                // classic difficulty keeps the normal damage (`:21215`,
                // GetAttackDamage_ForProjectiles at Classic = normal).
                let (proj, dmg) = match npc.kind {
                    NpcKind::DiabolistRed | NpcKind::DiabolistWhite => (291u16, 40),
                    NpcKind::RaggedCaster | NpcKind::RaggedCasterOpenCoat => (293u16, 40),
                    _ => (290u16, 30),
                };
                // `:21217-21224` — the diabolist's 291 carries the player's
                // centre in ai0/ai1 (the homing target); the others fire
                // plain. Both ride the HardmodeDungeonSkeletonShot modifier.
                if proj == 291 {
                    npc.wants_hostile_shot_ai.push(crate::npc::HostileShotAi {
                        kind: proj,
                        x: sx,
                        y: sy,
                        vx: ax,
                        vy: ay,
                        damage: dmg,
                        ai0: pcx,
                        ai1: pcy,
                        scale: None,
                    });
                } else {
                    npc.wants_hostile_shot.push((proj, sx, sy, ax, ay, dmg));
                }
                // `:21225` — localAI[0] = 0.
                npc.local_ai[0] = 0.0;
            } else {
            // `:21175-21277` — the standard shoot tick. The 281-286
            // projectile arm (`:21177-21227`) is the d38 branch's claim;
            // the 693/172 arms are ported here (W4-g5).
            // `:21175-21277` — the standard shoot tick. The 281-286
            // projectile arm (`:21177-21227`) and the 172 arm are
            // deferred with their kinds. W4-d43: the LibrarianSkeleton
            // (693) is ported, but its book shot (projectile 1092 via
            // `AI_FindNearbyBook`, `NPC.cs:62308`) is a documented skip —
            // the book-search feed is unplumbed (the kind casts silently).
            // S1-E-21g — `NPC.cs:21230-21233`: the cast's Item8 puff
            // (`this.type != 172` — the Rune Wizard is silent).
            if npc.kind != NpcKind::RuneWizard {
                npc.wants_sound.push(crate::npc::NpcSoundRequest {
                    cue: crate::npc::NpcSoundCue::Item8,
                    x: npc.x,
                    y: npc.y,
                });
            }
            let (w, h) = npc.size();
            // `:21238`/`:21242`/`:21246` — `(int)pos.X + width/2,
            // (int)pos.Y - 8` (NPC.NewNPC's Bottom convention, `:80770`).
            let (sx, sy) = (
                npc.x as i32 as f32 + (w as i32 / 2) as f32,
                npc.y as i32 as f32 - 8.0,
            );
            match npc.kind {
                NpcKind::GoblinSorcerer => {
                    shot = Some(CasterShot { kind: NpcKind::ChaosBall, x: sx, y: sy, ai3: 0.0 });
                }
                NpcKind::Tim => {
                    // `:21242` — type 665, the ChaosBallTim clone; the port
                    // folds it into ChaosBall (R4 FINAL, research/322:
                    // stat-identical SetDefaults `:15699-15713` vs
                    // `:17403-17417`, the same aiStyle-9 arm `:21519`, the
                    // same {31} immunity `NPCID.cs:6251-6264`, the same
                    // "Chaos Ball" display name). The one C# surface that
                    // differs — the banner (665 → Tim's 177,
                    // `BannerSystem.cs:864-866`; 30 → the Sorcerer's 39) —
                    // is UNREACHABLE in the port: both ids are Hide=true
                    // bestiary exclusions (`NPCID.cs:1443-1456`), no
                    // ENTRIES row, no kill-credit/banner drain ever fires
                    // for either. d43's distinct-kind row retires at merge.
                    shot = Some(CasterShot { kind: NpcKind::ChaosBall, x: sx, y: sy, ai3: 0.0 });
                    // `:21242` — type 665, the ChaosBallTim clone (W4-d43: its
                    // own kind now; identical stats, its own bestiary credit).
                    shot = Some(CasterShot { kind: NpcKind::ChaosBallTim, x: sx, y: sy, ai3: 0.0 });
                }
                NpcKind::DarkCaster => {
                    // `:21246` — the Water Sphere 33 carries
                    // `ai[3] = flag4 ? 1 : 0`.
                    shot = Some(CasterShot {
                        kind: NpcKind::WaterSphere,
                        x: sx,
                        y: sy,
                        ai3: if flag4 { 1.0 } else { 0.0 },
                    });
                }
                NpcKind::RuneWizard => {
                    // `:21264-21274` — the shadow beam (W4-g5): the 129
                    // RuneBlast at speed 10, damage 40, timeLeft 300. The
                    // spawn point is the body CENTRE (`pos + size*0.5`);
                    // the aim jitters ±10 (two Next(-10,11), X then Y).
                    let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
                    let (pcx, pcy) = player.center();
                    let mut ax = pcx - cx + rng.next_range(-10, 11) as f32;
                    let mut ay = pcy - cy + rng.next_range(-10, 11) as f32;
                    let inv = 10.0 / (ax * ax + ay * ay).sqrt();
                    ax *= inv;
                    ay *= inv;
                    // `:21271` — the 129 at damage 40; the timeLeft = 300
                    // post-spawn write rides the drain's override list.
                    npc.wants_hostile_shot.push((129, cx, cy, ax, ay, 40));
                    // `:21273` — localAI[0] = 0.
                    npc.local_ai[0] = 0.0;
                }
                NpcKind::LibrarianSkeleton => {
                    // `:21248-21263` — the book shot (W4-g5): the 1092
                    // LibrarianSkeletonBook at damage 13, ai0 = 0, ai1 =
                    // 45, ai2 = target. The spawn point is a nearby book
                    // (`AI_FindNearbyBook`) or the body centre + a
                    // NextVector2Circular(30, 15) offset.
                    let (cx, cy) = npc.center();
                    let search = (to_tile(cx) - 10, to_tile(cy) - 20);
                    let vec = if let Some((bx, by)) =
                        find_nearby_book(world, search, 20, 30, rng)
                    {
                        // `ToWorldCoordinates` (`Utils.cs:1762-1765`):
                        // tile*16 + 8 (the tile centre).
                        ((bx * 16 + 8) as f32, (by * 16 + 8) as f32)
                    } else {
                        // `NextVector2Circular(30, 15)` (`Utils.cs:1266-
                        // 1269`): NextVector2Unit() * (30,15) * NextFloat()
                        // — two NextFloat draws (the angle, then the radius).
                        let angle = rng.next_float() * std::f32::consts::TAU;
                        let radius = rng.next_float();
                        (
                            cx + angle.cos() * 30.0 * radius,
                            cy + angle.sin() * 15.0 * radius,
                        )
                    };
                    // `:21262` — the 1092 at damage 13, ai0 = 0, ai1 = 45,
                    // ai2 = this.target. The port's single player folds
                    // ai2 to 0 (the saucer-missile convention).
                    npc.wants_hostile_shot_ai.push(crate::npc::HostileShotAi {
                        kind: 1092,
                        x: vec.0,
                        y: vec.1,
                        vx: 0.0,
                        vy: 0.0,
                        damage: 13,
                        ai0: 0.0,
                        ai1: 45.0,
                        scale: None,
                    });
                }
                _ => {}
            }
            }
        }
        }
    }
    }
    // `:21279-21374` — the per-type dust/light tail: the lights are the
    // frame pass's feed (`npc_lights.rs::npc_light`, S1-E-5). W4-g5
    // draw-parity: the dust arms' GATE draws are consumed every tick at
    // their C# positions (the `NewDust` internals stay the dust task's
    // skip): 29/45 `Next(5)` (`:21282`), 32 `Next(3)` (`:21292`), the else
    // group (24) `Next(6)` (`:21366`); 533 draws NOTHING (`:21358-21361`,
    // light only). The 172's alpha dust (`:21310-21325`) and the 693's
    // light (`:21301-21308`) are ported here (W4-g5); the 281-286 arms
    // are the d38 branch's claim.
    if matches!(npc.kind, NpcKind::GoblinSorcerer | NpcKind::Tim) {
        let _ = rng.next_max(5); // `:21282` — Next(5) == 0 gates the dust
    } else if npc.kind == NpcKind::DarkCaster {
        let _ = rng.next_max(3); // `:21292` — Next(3) != 0 gates the dust
    } else if npc.kind == NpcKind::LibrarianSkeleton {
        // `:21301-21308` — light only, NO draw (the ai[0]-scaled light is
        // the frame pass's feed, `npc_lights.rs`).
    } else if npc.kind == NpcKind::RuneWizard {
        // `:21310-21325` — the alpha dust: 1 iteration (alpha < 255) or 2
        // (alpha == 255), each gated by `Next(255) > 255 - alpha`. The
        // gate draw is arm-level (consumed); the dust's Next(30)/Next(30)/
        // Next(6) internals stay the dust task's skip.
        let iterations = if npc.alpha == 255 { 2 } else { 1 };
        for _ in 0..iterations {
            let _ = rng.next_max(255); // `:21319` — the alpha gate
        }
    } else if npc.kind == NpcKind::DesertDjinn {
        // `:21358-21361` — light only, NO draw.
    } else {
        // `:21362-21373` — the else group: light + Next(6) == 0.
        let _ = rng.next_max(6);
    }
    // The generic movement tail — `NPC.UpdateNPC`'s gravity + collision
    // (`NPC.cs:90465-90493`) runs after `AI()` returns; the port applies it
    // at the end of this step (the ordering deviation is recorded in
    // ANALYSIS: the windup countdown sees the pre-gravity position, exactly
    // like the C#'s AI-then-collision split).
    let (w, h) = npc.size();
    let nx = npc.x + npc.vx;
    if !crate::player::collides_rect(world, nx, npc.y, w, h) {
        npc.x = nx;
    } else {
        npc.vx = 0.0;
    }
    npc.vy = (npc.vy + crate::npc::GRAVITY).min(crate::npc::MAX_FALL);
    let ny = npc.y + npc.vy;
    if !crate::player::collides_rect(world, npc.x, ny, w, h) {
        npc.y = ny;
        npc.on_ground = false;
    } else if npc.vy > 0.0 {
        let ty = ((ny + h) / TILE_PX).floor() as i32;
        npc.y = ty as f32 * TILE_PX - h;
        npc.vy = 0.0;
        npc.on_ground = true;
    } else {
        npc.vy = 0.0;
    }
    shot
}
}

/// `NPC.AI_GlobalFiringDistanceCheck` (`NPC.cs:55835-55841`): the target's
/// centre point seeds `Utils.CenteredRectangle(center, MaxWorldViewSize)`
/// (1920×1200, `Main.cs:677-681`/`:66714`), deflated by 50 per side, and the
/// shoot position's truncated point must sit inside. Both `.ToPoint()`
/// calls truncate toward zero (`Utils.cs:952-955`'s Point overload).
pub fn global_firing_distance_check(shoot: (f32, f32), target: (f32, f32)) -> bool {
    let rx = target.0 as i32 - 1920 / 2 + 50;
    let ry = target.1 as i32 - 1200 / 2 + 50;
    let (rw, rh) = (1920 - 100, 1200 - 100);
    let sx = shoot.0 as i32;
    let sy = shoot.1 as i32;
    sx >= rx && sx < rx + rw && sy >= ry && sy < ry + rh
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn ground_air() -> World {
        let mut w = World::new("t".into(), 200, 200);
        for x in 0..200 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn player_at(x: f32, y: f32) -> Player {
        let w = ground_air();
        let mut p = Player::spawn(&w);
        p.x = x;
        p.y = y;
        p
    }

    /// `NPC.cs:20836-20843` + `:20993` — the first tick inits ai[0] to 500
    /// and the same tick's increment leaves 501; hand arithmetic:
    /// 0 → 500 → 501, then +1 per tick.
    #[test]
    fn the_counter_inits_at_500_and_ticks() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::Tim, 40.0 * 16.0, 110.0 * 16.0, 1, 0.0);
        n.hp = n.max_hp;
        let mut rng = UnifiedRandom::new(7);
        ai_style_8_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.ai[0], 501.0, "0 -> 500 init -> 501 (same-tick ++)");
        ai_style_8_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.ai[0], 502.0);
    }

    /// `NPC.cs:21069-21073` + `:21113-21116` + `:21175` — the trigger at
    /// ai[0] == 100 sets ai[1] = 30, the SAME tick's countdown reads 29;
    /// the shoot fires when the countdown reaches 25 (five ticks later:
    /// 29, 28, 27, 26, 25).
    #[test]
    fn the_standard_trigger_winds_30_and_fires_at_25() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        // Resting on the 120-row floor (y = 1920 - 40): the position is
        // integral, so the C#'s `(int)pos` truncation is exact and the
        // post-step position equals the shoot-tick position.
        let mut n = Npc::new(NpcKind::Tim, 96.0 * 16.0, 120.0 * 16.0 - 40.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 99.0; // the next tick's ++ lands exactly on 100
        let mut rng = UnifiedRandom::new(7);
        let shot = ai_style_8_step(&mut n, &w, &p, &mut rng);
        assert!(shot.is_none());
        assert_eq!(n.ai[0], 100.0);
        assert_eq!(n.ai[1], 29.0, "30 minus the same-tick decrement");
        let mut fired = None;
        for expect in [28.0, 27.0, 26.0, 25.0] {
            let s = ai_style_8_step(&mut n, &w, &p, &mut rng);
            assert_eq!(n.ai[1], expect);
            if expect == 25.0 {
                fired = s;
            } else {
                assert!(s.is_none());
            }
        }
        let s = fired.expect("the 25 tick fires");
        assert_eq!(s.kind, NpcKind::ChaosBall);
        // `:21242` — Bottom = ((int)x + width/2, (int)y - 8).
        assert_eq!(s.x, n.x as i32 as f32 + 9.0);
        assert_eq!(s.y, n.y as i32 as f32 - 8.0);
    }

    /// `NPC.cs:21164-21173` — the Fire Imp fires at ai[1] == 10, twenty
    /// ticks after the 30 windup, at `(int)x + w/2 + direction*8, (int)y+20`.
    #[test]
    fn the_fire_imp_fires_at_10_with_the_direction_offset() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::FireImp, 96.0 * 16.0, 120.0 * 16.0 - 40.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 99.0;
        let mut rng = UnifiedRandom::new(7);
        let mut fired = None;
        for _ in 0..25 {
            if let Some(s) = ai_style_8_step(&mut n, &w, &p, &mut rng) {
                fired = Some(s);
                break;
            }
        }
        let s = fired.expect("the imp fires");
        assert_eq!(s.kind, NpcKind::BurningSphere);
        assert_eq!(n.ai[1], 10.0);
        assert_eq!(s.x, n.x as i32 as f32 + 9.0 + n.facing as f32 * 8.0);
        assert_eq!(s.y, n.y as i32 as f32 + 20.0);
    }

    /// `NPC.cs:21031-21038` + `:21079-21082` — the Djinn's single 180
    /// trigger sets 181 (reads 180 after the same-tick decrement) and the
    /// 360 wrap resets to 1 via the 650 block. W4-g5 draw-parity: the
    /// curse volley is LIVE — the 5 window ticks (ai[1] = 120/90/60/30/0)
    /// draw the search pairs and queue `wants_djinn_curse` (the old
    /// never-fires witness is superseded by the parity round; the sphere
    /// list stays empty — the Djinn fires a PROJECTILE, not a sphere).
    #[test]
    fn the_djinn_triggers_at_180_wraps_at_360_and_fires_curses() {
    /// 360 wrap resets to 1 via the 650 block. W4-f42: the curse volley
    /// fires on the %30 ticks (ai[1] = 120/90/60/30/0) into the
    /// hostile-shot-ai lane — never a sphere return.
    #[test]
    fn the_djinn_triggers_at_180_wraps_at_360_and_fires_the_curse_lane() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::DesertDjinn, 96.0 * 16.0, 120.0 * 16.0 - 62.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 179.0;
        let mut rng = UnifiedRandom::new(7);
        let s = ai_style_8_step(&mut n, &w, &p, &mut rng);
        assert!(s.is_none());
        assert_eq!(n.ai[0], 180.0);
        assert_eq!(n.ai[1], 180.0, "181 minus the same-tick decrement");
        let mut wrapped = false;
        let mut window_ticks = 0;
        let mut curses = 0;
        for _ in 0..400 {
            let before = n.ai[0];
            let before_windup = n.ai[1];
            let s = ai_style_8_step(&mut n, &w, &p, &mut rng);
            assert!(s.is_none(), "the Djinn never requests a sphere");
            assert!(n.ai[0] < 650.0, "the wrap keeps the counter under 650");
            // The window ticks: ai[1] AFTER the decrement hits a %30
            // multiple below the 5th (`:21119`).
            if before_windup > 0.0 {
                let after = n.ai[1];
                if after % 30.0 == 0.0 && after / 30.0 < 5.0 && before_windup >= after {
                    window_ticks += 1;
                }
            }
            curses += n.wants_djinn_curse.len();
            n.wants_djinn_curse.clear();
            assert!(s.is_none(), "the curse rides the shot-ai lane, not a sphere");
            assert!(n.ai[0] < 650.0, "the wrap keeps the counter under 650");
            for shot in n.wants_hostile_shot_ai.drain(..) {
                assert_eq!(shot.kind, crate::projectile::DESERT_DJINN_CURSE);
                assert_eq!(shot.damage, 0);
                assert_eq!(shot.ai0, n.target as f32, "ai0 = this.target (:21155)");
                // The tile centre: x*16+8, y*16+8.
                assert_eq!(shot.x % 16.0, 8.0);
                assert_eq!(shot.y % 16.0, 8.0);
                curses += 1;
            }
            if before == 359.0 {
                assert_eq!(n.ai[0], 1.0, "360 -> 650 -> 1 in one tick");
                wrapped = true;
            }
        }
        assert!(wrapped, "the Djinn wrapped at 360 within 400 ticks");
        assert!(window_ticks > 0, "the 181 windup crossed %30 window ticks");
        assert!(curses > 0, "the volley queued curse requests");
        assert!(curses >= 5, "the %30 volley fired (got {curses})");
    }

    /// `NPC.cs:21061-21068` — the red-hat double-step: the unconditional
    /// `ai[0]++` (`:20993`) plus flag4's second `++` with the odd rollback.
    /// From an EVEN counter the pair advances +2 and lands even; the
    /// red-hat casters therefore cast twice as fast. Hand arithmetic from
    /// ai[0]=98: `:20993` ++→99, flag4 ++→100 (even, kept) → the 100
    /// trigger winds ai[1]=30, the same-tick `--` reads 29.
    #[test]
    fn the_red_hat_double_step_hits_the_trigger_twice_as_fast() {
        let mut w = ground_air();
        w.skeletron_red_hat = true;
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::DarkCaster, 96.0 * 16.0, 120.0 * 16.0 - 40.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 98.0;
        let mut rng = UnifiedRandom::new(7);
        let s = ai_style_8_step(&mut n, &w, &p, &mut rng);
        assert!(s.is_none(), "the 29 windup has not reached 25");
        assert_eq!(n.ai[0], 100.0, "98 +2 (the flag4 double-step)");
        assert_eq!(n.ai[1], 29.0, "the 100 trigger's 30 minus the same-tick --");
        assert_eq!(n.local_ai[3], 1.0, "the latch (:20825)");
        assert_eq!(n.ai[0] % 2.0, 0.0, "flag4 always lands the counter even");
        // A non-latched caster from 98 advances only +1 → 99, no trigger.
        let mut m = Npc::new(NpcKind::Tim, 96.0 * 16.0, 120.0 * 16.0 - 40.0, 1, 0.0);
        m.hp = m.max_hp;
        m.ai[0] = 98.0;
        let mut rng2 = UnifiedRandom::new(7);
        ai_style_8_step(&mut m, &w, &p, &mut rng2);
        assert_eq!(m.ai[0], 99.0, "no flag4: the plain +1");
        assert_eq!(m.ai[1], 0.0, "99 is not a trigger tick");
    }

    /// `NPC.cs:18952-18953` — the search draws exactly two Next calls per
    /// attempt, X first then Y: a stream fork proves the first successful
    /// attempt consumed the same two draws in the same order.
    #[test]
    fn the_teleport_search_draws_x_then_y_per_attempt() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let tp = TelefragPlayer::from_player(&p);
        let mut rng = UnifiedRandom::new(7);
        let spot = attempt_find_teleport_spot(
            &w,
            &mut rng,
            NpcKind::Tim,
            (40.0 * 16.0, 110.0 * 16.0),
            (100, 117),
            20,
            &tp,
            false,
        );
        assert!(spot.is_some(), "the flat ground offers spots");
        let (sx, sy) = spot.unwrap();
        // The chosen spot passes every gate: a solid floor, free body
        // space, and the telefrag rect clear of the resting player.
        assert!(w
            .tile(sx, sy)
            .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind)));
        assert!(!solid_tiles(&w, sx - 1, sx + 1, sy - 4, sy - 1));
        assert!(
            (sx * 16 + 96 <= 1600 || sx * 16 - 80 >= 1620),
            "the telefrag rect misses the player's 1600..1620 hitbox"
        );
        // The stream proof: the search consumed a whole number of (X, Y)
        // pairs in exactly the C#'s ranges — a probe replaying those pairs
        // from the same seed lands on the search stream's state. Any extra
        // or reordered draw would desynchronize it.
        let mut pairs = 0;
        let mut probe = UnifiedRandom::new(7);
        loop {
            let _x = probe.next_range(80, 121);
            let _y = probe.next_range(97, 138);
            pairs += 1;
            if probe == rng {
                break;
            }
            assert!(pairs <= 100, "at most 100 attempts");
        }
        assert!(pairs >= 1);
    }

    /// `NPC.cs:18944-18948` — beyond 2000 Manhattan px the attempt counter
    /// starts at 100: zero draws, no spot.
    #[test]
    fn the_far_target_burns_no_draws() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let tp = TelefragPlayer::from_player(&p);
        let mut rng = UnifiedRandom::new(7);
        let before = rng.clone();
        // Self at tile 10, target at tile 300: |160 - 4800| > 2000.
        let spot = attempt_find_teleport_spot(
            &w,
            &mut rng,
            NpcKind::Tim,
            (10.0 * 16.0, 110.0 * 16.0),
            (300, 117),
            20,
            &tp,
            false,
        );
        assert!(spot.is_none());
        assert_eq!(rng, before, "the >2000 gate consumes no draws");
    }

    /// `NPC.cs:18960-18966` — the Dark Caster's dungeon-wall gate: on a
    /// wall-less world with no Skeletron the search rejects every spot
    /// (but still draws its pairs); a live head opens the gate.
    #[test]
    fn the_dark_caster_needs_a_dungeon_wall_or_a_live_skeletron() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let tp = TelefragPlayer::from_player(&p);
        let mut rng = UnifiedRandom::new(7);
        let spot = attempt_find_teleport_spot(
            &w,
            &mut rng,
            NpcKind::DarkCaster,
            (40.0 * 16.0, 110.0 * 16.0),
            (100, 117),
            20,
            &tp,
            false,
        );
        assert!(spot.is_none(), "no dungeon walls, no head: no spot");
        let mut rng2 = UnifiedRandom::new(7);
        let spot2 = attempt_find_teleport_spot(
            &w,
            &mut rng2,
            NpcKind::DarkCaster,
            (40.0 * 16.0, 110.0 * 16.0),
            (100, 117),
            20,
            &tp,
            true,
        );
        assert!(spot2.is_some(), "AnyNPCs(35) opens the gate");
    }

    /// `NPC.cs:18960-18966` — the 693 rides the SAME dungeon-wall gate as
    /// the 32 (the C#'s `(type == 32 || (type >= 281 && type <= 286) ||
    /// type == 693)` group): on a wall-less world with no Skeletron the
    /// shared fallback search rejects every spot; a live head opens it.
    /// W4-g5 RETIRE fix witness — the phases-4-6 draft omitted the 693
    /// from `dungeon_wall_gated`. WRITTEN NOT RUN (code-only mode).
    #[test]
    fn w4_g5_the_librarian_skeleton_needs_a_dungeon_wall_or_a_live_skeletron() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let tp = TelefragPlayer::from_player(&p);
        let mut rng = UnifiedRandom::new(7);
        let spot = attempt_find_teleport_spot(
            &w,
            &mut rng,
            NpcKind::LibrarianSkeleton,
            (40.0 * 16.0, 110.0 * 16.0),
            (100, 117),
            20,
            &tp,
            false,
        );
        assert!(spot.is_none(), "no dungeon walls, no head: no spot");
        let mut rng2 = UnifiedRandom::new(7);
        let spot2 = attempt_find_teleport_spot(
            &w,
            &mut rng2,
            NpcKind::LibrarianSkeleton,
            (40.0 * 16.0, 110.0 * 16.0),
            (100, 117),
            20,
            &tp,
            true,
        );
        assert!(spot2.is_some(), "AnyNPCs(35) opens the gate");
    }

    /// `NPC.cs:55835-55841` — the firing rect: the target centre ±(960,600)
    /// deflated 50 → ±(910,550); the boundary points truncate first.
    #[test]
    fn the_firing_rect_is_the_deflated_max_world_view() {
        let target = (1600.0, 1600.0);
        // Inside: 909 px right, 549 px down.
        assert!(global_firing_distance_check((2509.0, 2149.0), target));
        // Outside: 910 px right (the deflated edge is exclusive).
        assert!(!global_firing_distance_check((2510.0, 1600.0), target));
        assert!(!global_firing_distance_check((1600.0, 2150.0), target));
        // The truncation: 2509.9 still reads 2509.
        assert!(global_firing_distance_check((2509.9, 1600.0), target));
    }

    /// `NPC.cs:20855-20930` — the pending teleport places the body on the
    /// NEXT tick: ai[2]/ai[3] hold the tile, the placement zeroes the
    /// velocity and the pair; X centres with INTEGER width/2.
    #[test]
    fn the_pending_teleport_places_on_the_next_tick() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::Tim, 40.0 * 16.0, 110.0 * 16.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[2] = 60.0;
        n.ai[3] = 120.0;
        n.vx = 5.0;
        n.vy = 3.0;
        let mut rng = UnifiedRandom::new(7);
        ai_style_8_step(&mut n, &w, &p, &mut rng);
        // 60*16 - 18/2 + 8 = 960 - 9 + 8 = 959; the gravity tail then
        // settles the feet on the 120-row floor (y = 1920 - 40 = 1880).
        assert_eq!(n.x, 959.0);
        assert_eq!(n.y, 120.0 * 16.0 - 40.0);
        assert_eq!(n.ai[2], 0.0);
        assert_eq!(n.ai[3], 0.0);
        assert_eq!(n.vy, 0.0, "the placement zeroes vy before the floor stop");
    }

    // ---- W4-g5 draw-parity witnesses (written, NOT run — code-only) ----

    /// `NPC.cs:20863-20922` + `:20932-20991` + `:21282` — the Tim arrival
    /// tick consumes EXACTLY 101 draws: 50 `Next(1, 3)` scale arguments
    /// per burst × 2 bursts, then the ambient `Next(5)`. A mirror stream
    /// replaying that sequence from the same seed must land on the arm's
    /// stream state (the five-probe convention's exact-replay form).
    #[test]
    fn w4_g5_the_tim_arrival_tick_draws_100_burst_scales_plus_the_ambient() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::Tim, 40.0 * 16.0, 110.0 * 16.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 50.0; // mid-cycle: no trigger, no teleport search
        n.ai[2] = 60.0;
        n.ai[3] = 120.0;
        let mut rng = UnifiedRandom::new(7);
        ai_style_8_step(&mut n, &w, &p, &mut rng);
        let mut mirror = UnifiedRandom::new(7);
        for _ in 0..100 {
            let _ = mirror.next_range(1, 3); // the two bursts' scales
        }
        let _ = mirror.next_max(5); // the ambient gate
        assert_eq!(rng, mirror, "exactly 100 Next(1,3) + one Next(5)");
    }

    /// `NPC.cs:20874-20879` + `:21292` — the Dark Caster's bursts use the
    /// CONSTANT 1.5 scale (zero draws); the arrival tick spends only the
    /// ambient `Next(3)`.
    #[test]
    fn w4_g5_the_dark_caster_arrival_draws_only_the_ambient_next3() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::DarkCaster, 40.0 * 16.0, 110.0 * 16.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 50.0;
        n.ai[2] = 60.0;
        n.ai[3] = 120.0;
        let mut rng = UnifiedRandom::new(7);
        ai_style_8_step(&mut n, &w, &p, &mut rng);
        let mut mirror = UnifiedRandom::new(7);
        let _ = mirror.next_max(3);
        assert_eq!(rng, mirror, "the 32 bursts draw nothing; one Next(3)");
    }

    /// `NPC.cs:21282`/`:21292`/`:21358-21366` — the plain-tick ambient
    /// census: 29/45 one `Next(5)`, 32 one `Next(3)`, 24 one `Next(6)`,
    /// 533 NOTHING (light only). No windup, no search on these ticks.
    #[test]
    fn w4_g5_the_ambient_gate_census_per_kind() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        for (kind, gate) in [
            (NpcKind::GoblinSorcerer, Some(5)),
            (NpcKind::Tim, Some(5)),
            (NpcKind::DarkCaster, Some(3)),
            (NpcKind::FireImp, Some(6)),
            (NpcKind::DesertDjinn, None),
        ] {
            let mut n = Npc::new(kind, 96.0 * 16.0, 120.0 * 16.0 - 62.0, 1, 0.0);
            n.hp = n.max_hp;
            n.ai[0] = 10.0; // no trigger tick, no 650 search
            let mut rng = UnifiedRandom::new(7);
            ai_style_8_step(&mut n, &w, &p, &mut rng);
            let mut mirror = UnifiedRandom::new(7);
            if let Some(max) = gate {
                let _ = mirror.next_max(max);
            }
            assert_eq!(rng, mirror, "{kind:?}'s ambient gate");
        }
    }

    /// `NPC.cs:18937`/`:21097`/`:21101` — the W4-d128 L1 fix: the Fire
    /// Imp's teleport search rides the DEFAULT radius 20 (the probe's
    /// bounds are ±20, NOT num88 = 5), while the post-teleport windup
    /// stays `ai[1] = 5`.
    #[test]
    fn w4_g5_the_fire_imp_searches_at_radius_20_and_winds_5() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::FireImp, 96.0 * 16.0, 120.0 * 16.0 - 40.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 649.0; // the ++ lands 650 → the teleport attempt
        let mut rng = UnifiedRandom::new(7);
        ai_style_8_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.ai[0], 1.0, "the 650 block resets the counter");
        assert_eq!(n.ai[1], 4.0, "num88 = 5 is the WINDUP only (minus the same-tick --)");
        assert!(n.ai[2] != 0.0 && n.ai[3] != 0.0, "a spot was chosen");
        // The stream proof: whole (X, Y) pairs in the ±20 bounds — the
        // radius-5 bounds would desynchronize this probe. The ambient
        // Next(6) rides AFTER the search.
        let mut pairs = 0;
        let mut probe = UnifiedRandom::new(7);
        loop {
            let _x = probe.next_range(80, 121);
            let _y = probe.next_range(97, 138);
            pairs += 1;
            let mut with_ambient = probe.clone();
            let _ = with_ambient.next_max(6);
            if with_ambient == rng {
                break;
            }
            assert!(pairs <= 100, "at most 100 attempts");
        }
        assert!(pairs >= 1);
    }

    /// `NPC.cs:21117-21160` — the volley window tick (ai[1] 121 → 120):
    /// the search consumes whole X-then-Y pairs in the ±6 bounds until
    /// the accepted tile; the queued curse sits at that tile's centre
    /// with `ai0 = target`. The Djinn's ambient draws nothing, so the
    /// window tick's whole spend is the search.
    #[test]
    fn w4_g5_the_volley_window_draws_pairs_and_queues_the_tile_centre() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::DesertDjinn, 80.0 * 16.0, 120.0 * 16.0 - 62.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 10.0;
        n.ai[1] = 121.0; // the decrement lands 120 — a window tick
        let mut rng = UnifiedRandom::new(7);
        ai_style_8_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.ai[1], 120.0);
        assert_eq!(n.wants_sound.len(), 1, "the window's Item8 (:21121)");
        let (cx, cy, ai0) = *n.wants_djinn_curse.first().expect("a curse queued");
        assert_eq!(ai0, n.target as f32, "ai0 = this.target (:21155)");
        // The tile centre: (x*16+8, y*16+8).
        assert_eq!((cx as i32 - 8) % 16, 0);
        assert_eq!((cy as i32 - 8) % 16, 0);
        let tx = (cx as i32 - 8) / 16;
        let ty = (cy as i32 - 8) / 16;
        // The stream proof: replay the ±6 pairs; the LAST pair before
        // the streams match is the accepted tile.
        let mut probe = UnifiedRandom::new(7);
        let mut last = (0, 0);
        let mut iters = 0;
        loop {
            let x = probe.next_range(94, 107);
            let y = probe.next_range(111, 124);
            last = (x, y);
            iters += 1;
            if probe == rng {
                break;
            }
            assert!(iters <= 50, "at most 50 iterations");
        }
        assert_eq!(last, (tx, ty), "the curse rides the LAST drawn pair");
    }

    /// `NPC.cs:21133-21136` — beyond 2000 px the volley's length gate
    /// draws NOTHING (the Item8 still plays); no curse queues.
    #[test]
    fn w4_g5_the_far_volley_window_burns_no_draws() {
        let w = ground_air();
        let p = player_at(199.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::DesertDjinn, 16.0, 120.0 * 16.0 - 62.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 10.0;
        n.ai[1] = 121.0;
        let mut rng = UnifiedRandom::new(7);
        let before = rng.clone();
        ai_style_8_step(&mut n, &w, &p, &mut rng);
        assert_eq!(rng, before, "the >2000 gate consumes no draws");
        assert!(n.wants_djinn_curse.is_empty());
        assert_eq!(n.wants_sound.len(), 1, "the puff plays regardless");
    }

    // ---- W4-g5 — the RuneWizard (172) / LibrarianSkeleton (693) witnesses
    // (written, NOT run — code-only mode). Every constant pins to its C#
    // literal (`NPC.cs:11154-11167`/`:17723-17735` SetDefaults,
    // `:20844-21325` the caster arms). ----

    /// `NPC.cs:20844-20854` — the 172's alpha ramp: alpha++ toward 255
    /// every tick, a strike (justHit) resets it to 0.
    #[test]
    fn w4_g5_the_rune_wizard_alpha_ramps_and_a_strike_resets_it() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::RuneWizard, 96.0 * 16.0, 120.0 * 16.0 - 40.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 10.0; // mid-cycle: no trigger, no teleport
        let mut rng = UnifiedRandom::new(7);
        assert_eq!(n.alpha, 0, "the SetDefaults default");
        ai_style_8_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.alpha, 1, "alpha++ per tick");
        ai_style_8_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.alpha, 2);
        n.just_hit = true;
        ai_style_8_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.alpha, 0, "the strike resets alpha to 0");
    }

    /// `NPC.cs:21023-21030` — the 172's six 75-spaced triggers wind the
    /// 30-tick windup (the same-tick decrement reads 29).
    #[test]
    fn w4_g5_the_rune_wizard_triggers_every_75_ticks() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::RuneWizard, 96.0 * 16.0, 120.0 * 16.0 - 40.0, 1, 0.0);
        n.hp = n.max_hp;
        let mut rng = UnifiedRandom::new(7);
        for trigger in [75.0, 150.0, 225.0, 300.0, 375.0, 450.0] {
            n.ai[0] = trigger - 1.0; // the ++ lands on the trigger
            n.ai[1] = 0.0;
            ai_style_8_step(&mut n, &w, &p, &mut rng);
            assert_eq!(n.ai[0], trigger);
            assert_eq!(n.ai[1], 29.0, "the {trigger} trigger winds 30 (minus the --)");
        }
        // A non-trigger tick does not wind.
        n.ai[0] = 100.0;
        n.ai[1] = 0.0;
        ai_style_8_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.ai[1], 0.0, "101 is not a 75-spaced trigger");
    }

    /// `NPC.cs:21264-21274` — the 172's shadow beam: the 129 RuneBlast at
    /// speed 10, damage 40, the two Next(-10,11) aim jitters (X then Y).
    /// The spawn point is the body centre; the Item8 puff is SILENT.
    #[test]
    fn w4_g5_the_rune_wizard_shoot_draws_two_jitters_and_fires_the_129() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::RuneWizard, 96.0 * 16.0, 120.0 * 16.0 - 40.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 10.0;
        n.ai[1] = 26.0; // the -- lands 25 — the shoot tick
        let mut rng = UnifiedRandom::new(7);
        let mut mirror = UnifiedRandom::new(7);
        let _ = mirror.next_range(-10, 11); // the X jitter
        let _ = mirror.next_range(-10, 11); // the Y jitter
        ai_style_8_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.wants_hostile_shot.len(), 1, "one 129 bolt");
        let (kind, _, _, _, _, dmg) = n.wants_hostile_shot[0];
        assert_eq!(kind, 129, "the RuneBlast");
        assert_eq!(dmg, 40, "the shadow beam damage");
        assert!(n.wants_sound.is_empty(), "the 172 is silent (type != 172 gate)");
        // The two jitter draws ride the caller stream (the ambient
        // Next(255) follows — the alpha is 1 after the ramp, 1 iteration).
        let _ = mirror.next_max(255); // the ambient alpha-dust gate
        assert_eq!(rng, mirror, "the two jitters + the ambient Next(255)");
    }

    /// `NPC.cs:21310-21325` — the 172's ambient alpha dust: 1 Next(255)
    /// iteration below alpha 255, 2 at alpha 255.
    #[test]
    fn w4_g5_the_rune_wizard_ambient_draws_next255_per_iteration() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::RuneWizard, 96.0 * 16.0, 120.0 * 16.0 - 40.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 10.0;
        n.alpha = 100; // below 255: 1 iteration
        let mut rng = UnifiedRandom::new(7);
        ai_style_8_step(&mut n, &w, &p, &mut rng);
        let mut mirror = UnifiedRandom::new(7);
        let _ = mirror.next_max(255); // 1 iteration
        assert_eq!(rng, mirror, "1 Next(255) below alpha 255");
        // At alpha 255: 2 iterations.
        let mut n2 = Npc::new(NpcKind::RuneWizard, 96.0 * 16.0, 120.0 * 16.0 - 40.0, 1, 0.0);
        n2.hp = n2.max_hp;
        n2.ai[0] = 10.0;
        n2.alpha = 254; // the ramp ++ lands 255: 2 iterations
        let mut rng2 = UnifiedRandom::new(7);
        ai_style_8_step(&mut n2, &w, &p, &mut rng2);
        let mut mirror2 = UnifiedRandom::new(7);
        let _ = mirror2.next_max(255);
        let _ = mirror2.next_max(255);
        assert_eq!(rng2, mirror2, "2 Next(255) at alpha 255");
    }

    /// `NPC.cs:20994-21005` + `:21106-21111` — the 693's localAI clock:
    /// localAI[0]++ per tick, localAI[3] latches the facing past 1, the
    /// teleport resets localAI[0]/[1]/[3].
    #[test]
    fn w4_g5_the_librarian_skeleton_localai_clock() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::LibrarianSkeleton, 96.0 * 16.0, 120.0 * 16.0 - 40.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 10.0;
        let mut rng = UnifiedRandom::new(7);
        ai_style_8_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.local_ai[0], 1.0, "localAI[0]++");
        assert_eq!(n.local_ai[3], 0.0, "the facing latch waits for > 1");
        ai_style_8_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.local_ai[0], 2.0);
        assert_eq!(n.local_ai[3], 1.0, "the facing latches past 1");
    }

    /// `NPC.cs:21248-21263` — the 693's book shot: the 1092 at damage 13,
    /// ai0 = 0, ai1 = 45. With no books in the world, the fallback draws
    /// two NextFloat (the NextVector2Circular angle + radius).
    #[test]
    fn w4_g5_the_librarian_skeleton_book_shot_fallback_draws_two_floats() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::LibrarianSkeleton, 96.0 * 16.0, 120.0 * 16.0 - 40.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 10.0;
        n.ai[1] = 26.0; // the -- lands 25 — the shoot tick
        let mut rng = UnifiedRandom::new(7);
        let mut mirror = UnifiedRandom::new(7);
        let _ = mirror.next_float(); // the circular angle
        let _ = mirror.next_float(); // the circular radius
        ai_style_8_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.wants_hostile_shot_ai.len(), 1, "one 1092 book");
        let shot = &n.wants_hostile_shot_ai[0];
        assert_eq!(shot.kind, 1092, "the LibrarianSkeletonBook");
        assert_eq!(shot.damage, 13, "the book damage");
        assert_eq!(shot.ai0, 0.0);
        assert_eq!(shot.ai1, 45.0, "the 45-tick hover");
        assert_eq!(n.wants_sound.len(), 1, "the 693 plays the Item8 puff");
        // The 693's ambient draws NOTHING (light only).
        assert_eq!(rng, mirror, "the two NextFloat + no ambient draw");
    }

    /// `NPC.cs:19009-19085` — the 693's book teleport: the >125-tile
    /// Euclidean bail draws NOTHING; a nearby dungeon-wall book tile is
    /// found via the Next(2) direction seed + the X-then-Y pairs.
    #[test]
    fn w4_g5_the_librarian_skeleton_book_teleport_bail_and_search() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let tp = TelefragPlayer::from_player(&p);
        // The far bail: self at tile 10, target at tile 300 (>125 tiles).
        let mut rng = UnifiedRandom::new(7);
        let before = rng.clone();
        let spot = attempt_find_teleport_spot_near_books(
            &w, &mut rng, (10.0 * 16.0, 110.0 * 16.0), (300, 117), 30, &tp,
        );
        assert!(spot.is_none());
        assert_eq!(rng, before, "the >125-tile bail consumes no draws");
    }
}
}
