//! The Lihzahrd temple — the M5-BIOME phase-6 stub: the faithful
//! `LihzahrdTemple` placement search (`WorldGen.cs:15834-15926`) writing the
//! `GenVars` the later temple passes read, over the shape-level room body.
//!
//! The C# temple (`makeTemple`, `WorldGen.cs:33435-34237`): the brick box
//! (the Lihzahrd brick 226, the walls 87 unsafe / 112 safe) buried in the
//! jungle cavern below the rock layer, the interior rooms carved, the altar
//! (237) in the main room, the dart traps (137) in the walls, the
//! teleporter pads, the chests. The M5-TEMPLE seam (what is verbatim vs
//! shape-level here): the placement search is the verbatim pass; the room
//! generation is the shape-level slice — vanilla's non-intersecting room
//! walk (`:33444-33548`), `makeTemple_GenerateBricks` (`:34241+`), the
//! jitter carve sweeps (`:33554-33684`), the `templePather` corridors
//! (`:33686-33787`), the `outerTempled` sweeps (`:33819-33850`), the
//! style-11 door (`:33950`), the `templeCleaner` sweeps (`:33954-33961`,
//! `:33335`) and the spike/platform loop (`:34049-34217`) are not ported;
//! the stub keeps the box + hall + altar-room body. What publishes state:
//! the bounds `t_left/t_right/t_top/t_bottom` = the filled extent ± 10 (the
//! C# room-union ± 10, `:33788-33817`, published `:34231-34235`),
//! `t_rooms = 2` (the C# publishes the final room count, `:34235`) and the
//! altar anchor `l_altar_x/l_altar_y` = the 3×2 altar's top-left
//! (`:34000-34001`, hand-built fallback `:34038-34039`) — the fields
//! `LihzahrdTemplePart2` (`:17701`) and `LihzahrdAltar` (`:21943`) consume
//! (M5-TEMPLE's row owns the full interior).
//!
//! Deviations (ANALYSIS "Lihzahrd temple (M1.4-WGEN-6)"): the C#'s
//! multi-room crawl with the doors and the teleporters is the shape-level
//! approximation (the box + the 3 corridor rooms); the Plantera-gate (the
//! altar needs the Plantera downed to interact) waits for M5-HARDFORCE;
//! the trap wiring arms ride the wiring engine (the traps 137 are the
//! case-137 variants the wiring trip fires).

use super::pipeline::TempleBounds;
use super::structures::place_chest;
use crate::tile::{FLAG_ACTIVE, Tile};
use crate::world::World;

pub(crate) const TILE_TEMPLE_BRICK: u16 = 226;
pub(crate) const TILE_ALTAR: u16 = 237;
pub(crate) const WALL_TEMPLE_UNSAFE: u16 = 87;
pub(crate) const WALL_TEMPLE: u16 = 112;

/// The vanilla `DungeonSide.Right` (`WorldBuilding/DungeonSide.cs:13-15` —
/// Left −1 / None 0 / Right 1). The port's dungeon sits on the right
/// (`plan_spans`' `dungeon_x`; decision M5-BIOME #10), so the search's
/// `-dungeonSide` multiplier is the constant −1 and the jungle (the temple's
/// host) is on the dungeon-opposite side.
const DUNGEON_SIDE_RIGHT: i32 = 1;

/// The `UnifiedRandom.NextDouble()` stand-in: one xorshift draw scaled to
/// [0, 1). The xorshift-for-`UnifiedRandom` substitution is the recorded
/// stream-model deviation (`Streams::pass_rng`'s docs); the fraction mapping
/// is the port-level convention.
fn next_double(rng: &mut super::Rng) -> f64 {
    f64::from(rng.next_u32()) / 4294967296.0
}

/// The faithful `LihzahrdTemple` placement search (`WorldGen.cs:15834-15926`):
/// per attempt sample `y` from `[rockLayerClamped, maxTilesY - 600)` and `x`
/// from the dungeon-opposite reach `((NextDouble() * num3 + 0.1) *
/// -dungeonSide + 0.5) * maxTilesX`, accept the first sample landing on
/// active jungle grass (tile 60), widen `num3` by 0.05 after every 1,000,000
/// failed attempts (capped 0.35 — `Math.Min` lands it exactly on 0.35, so the
/// `== 0.35` check fires on the next overflow) and give up to the fallback
/// position after 11 overflows at the cap. Returns the bury point
/// `makeTemple(x, y)` receives.
///
/// Deviations (the sub-vanilla valve, decision M5-BIOME #12's shape): when
/// `maxTilesY - 600 <= 0` the vanilla y band cannot exist in-world (vanilla's
/// smallest world is 4200×1200, where the band floor is 600) — every sample
/// would miss and the C# loop would spin its full ~13M-attempt termination
/// before falling through to the fallback below, so the port skips straight
/// to it. The remix branch (`:15852-15876`) is seed-gated (`remixWorldGen`/
/// `notTheBees`; M5-SEED scope) and never runs on the classic chain.
pub(crate) fn find_temple_site(
    world: &World,
    dungeon_x: i32,
    rock_layer: f64,
    rng: &mut super::Rng,
) -> (i32, i32) {
    let mut found = false;
    let mut site = (0, 0);
    // The sub-vanilla valve (see the doc comment): `num5 <= 0` means the
    // vanilla band `[(int)rockLayer, maxTilesY - 600)` has no in-world rows.
    if world.height - 600 > 0 {
        // `:15837-15840`: the widen-stage counter, the failed-attempt counter
        // and the x-reach factor.
        let mut num: i32 = 0;
        let mut num2: u64 = 0;
        let mut num3: f64 = 0.25;
        loop {
            // `:15842-15847`: the y band (recomputed per iteration in the C#;
            // `(int)Main.rockLayer` truncates like `as i32`, and `num4` is
            // clamped so `Next(num4, num5)` stays a valid 1-wide band).
            let num5 = world.height - 600;
            let mut num4 = rock_layer as i32;
            if num4 > num5 - 1 {
                num4 = num5 - 1;
            }
            // `:15848`: the y draw first, then the x fraction.
            let num6 = rng.range(num4, num5);
            // `:15848`: the x formula — `NextDouble()` then the
            // dungeon-opposite reach; the C# `(int)` cast truncates toward
            // zero like `as i32` (the value is positive: the reach is
            // [0.05·maxTilesX, 0.4·maxTilesX] for `dungeonSide == Right`).
            let num7 = (((next_double(rng) * num3 + 0.1) * -(DUNGEON_SIDE_RIGHT as f64) + 0.5)
                * world.width as f64) as i32;
            // `:15877-15882` (the remix branch is above): accept the first
            // active jungle-grass tile.
            if world
                .tile(num7, num6)
                .is_some_and(|t| t.active() && t.kind == 60)
            {
                found = true;
                site = (num7, num6);
                break;
            }
            // `:15884-15893`: the widening valve — the post-increment compare,
            // then `num3 = Math.Min(0.35, num3 + 0.05)` with the overflow
            // counter giving up after 11 stages at the cap.
            let old = num2;
            num2 += 1;
            if old > 1_000_000 {
                if num3 == 0.35 {
                    num += 1;
                    if num > 10 {
                        break;
                    }
                }
                num3 = (num3 + 0.05).min(0.35);
                num2 = 0;
            }
        }
    }
    if !found {
        // `:15926-15930`: the fallback — the dungeon-mirrored x
        // (`generatingDungeonPositionX` is the dungeon entrance x,
        // `DungeonCrawler.cs:352/:399`; the port's stand-in is
        // `BiomeSpans.dungeon_x`) and 100 rows below the rock layer.
        site = (world.width - dungeon_x, rock_layer as i32 + 100);
    }
    site
}

fn put_brick(world: &mut World, x: i32, y: i32, wall: u16) {
    if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
        return;
    }
    let i = world.index(x, y);
    let t = &mut world.tiles[i];
    t.kind = TILE_TEMPLE_BRICK;
    t.flags |= FLAG_ACTIVE;
    t.liquid = 0;
    t.set_slope(0);
    t.wall = wall;
}

fn carve(world: &mut World, x: i32, y: i32, wall: u16) {
    if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
        return;
    }
    let i = world.index(x, y);
    world.tiles[i] = Tile::default();
    world.tiles[i].wall = wall;
}

/// `AddLihzahrdAltar`'s altar write — the 3×2 altar (237) with the 3-wide
/// brick base row; the frame layout matches the C# `PlaceTile(237)` result
/// (`frameX = i*18`, `frameY = j*18`; the hand-built fallback writes the
/// same 3×2, `WorldGen.cs:34049-34062`) and the anchor it leaves is the
/// top-left tile the C# publishes as `lAltarX/lAltarY` (`:34000-34001`).
pub(crate) fn place_lihzahrd_altar(world: &mut World, x: i32, y: i32) {
    for i in 0..3 {
        for j in 0..2 {
            let px = x + i;
            let py = y + j;
            if px < 2 || py < 2 || px >= world.width - 2 || py >= world.height - 2 {
                return;
            }
            let wall = world.tile(px, py).map(|t| t.wall).unwrap_or(0);
            let idx = world.index(px, py);
            // The 3×2 frame layout the C# `PlaceTile(237)` result and the
            // hand-built fallback share (`frameX = i*18`, `frameY = j*18`,
            // `WorldGen.cs:34049-34062`). (The pre-phase-6 code shadowed the
            // loop `i` with the world index here — the garbage frames were
            // invisible while 237 was re-framed at load; the phase-6 boot
            // static make 237 frame-important, so the stored layout must be
            // the faithful one.)
            world.tiles[idx] = Tile {
                kind: TILE_ALTAR,
                frame_x: (i * 18) as i16,
                frame_y: (j * 18) as i16,
                ..Default::default()
            };
            world.tiles[idx].flags |= FLAG_ACTIVE;
            world.tiles[idx].wall = wall;
        }
        // The base row (`:32090-32100`).
        let py = y + 2;
        if x + i < 2 || py >= world.height - 2 {
            continue;
        }
        let idx2 = world.index(x + i, py);
        let t = &mut world.tiles[idx2];
        t.kind = TILE_TEMPLE_BRICK;
        t.flags |= FLAG_ACTIVE;
        t.set_slope(0);
    }
}

/// The `LihzahrdTemple` pass (`WorldGen.cs:15834+` → `makeTemple(x, y)`):
/// the temple box in the jungle cavern below the rock layer — the outer
/// brick shell (the walls 87), the carved room corridors (the walls 112),
/// the altar room with the 3×2 altar, the dart traps (137) embedded in the
/// walls, the chests in the rooms. The bury point comes from the faithful
/// search ([`find_temple_site`]); the published [`TempleBounds`] is the
/// M5-TEMPLE seam (see the module docs for the verbatim/shape-level split).
pub(crate) fn pass_lihzahrd_temple(
    world: &mut World,
    dungeon_x: i32,
    rock_layer: f64,
    rng: &mut super::Rng,
) -> TempleBounds {
    // `WorldGen.cs:15834-15930`: the placement search (or its fallback).
    let (cx, cy) = find_temple_site(world, dungeon_x, rock_layer, rng);
    // The shape-level room body (the M5-TEMPLE seam — the module docs carry
    // the split): the outer box with the unsafe walls.
    let hw = rng.range(34, 46);
    let hh = rng.range(22, 30);
    for px in (cx - hw)..=(cx + hw) {
        for py in (cy - hh)..=(cy + hh) {
            put_brick(world, px, py, WALL_TEMPLE_UNSAFE);
        }
    }
    // The corridor rooms: the central hall + the two side rooms, carved
    // with the safe walls (`LihzahrdTemplePart2`'s room carving).
    let hall_hw = hw - 6;
    let hall_hh = hh - 8;
    for px in (cx - hall_hw)..=(cx + hall_hw) {
        for py in (cy - hall_hh)..=(cy + hall_hh) {
            carve(world, px, py, WALL_TEMPLE);
        }
    }
    // The pillars: the brick columns inside the hall (the temple look).
    for px in (cx - hall_hw + 6..=cx + hall_hw - 6).step_by(9) {
        for py in (cy - hall_hh)..=(cy + hall_hh) {
            put_brick(world, px, py, WALL_TEMPLE);
            put_brick(world, px, py, WALL_TEMPLE);
        }
    }
    // The altar room: the west end of the hall.
    let altar_x = cx - hall_hw + 4;
    let altar_y = cy + hall_hh - 5;
    for px in (altar_x - 3)..=(altar_x + 6) {
        for py in (altar_y - 4)..=altar_y {
            carve(world, px, py, WALL_TEMPLE);
        }
    }
    place_lihzahrd_altar(world, altar_x, altar_y - 2);
    // The dart traps (137) embedded in the hall's brick walls facing the
    // rooms (the case-137 wiring variants fire them).
    for px in (cx - hall_hw + 3..=cx + hall_hw - 3).step_by(12) {
        let py = cy - hall_hh + 2;
        let i = world.index(px, py);
        world.tiles[i] = Tile {
            kind: 137, // `TileID.Trap` — the dart variant (frameY 0)
            ..Default::default()
        };
        world.tiles[i].flags |= FLAG_ACTIVE;
        world.tiles[i].wall = WALL_TEMPLE;
    }
    // The chests in the hall's corners.
    let mut chests = 0;
    for (dx, dy) in [(-hall_hw + 2, -hall_hh + 1), (hall_hw - 3, hall_hh - 2)] {
        if place_chest(world, cx + dx, cy + dy) {
            chests += 1;
        }
    }
    let _ = chests;
    // The GenVars publication — the M5-TEMPLE seam: the bounds are the
    // filled extent ± 10 (the C# room-union ± 10, `:33788-33817`, published
    // `:34231-34235`), `t_rooms` the carved-room count (the C# publishes the
    // final walk count, `:34235`) and the altar anchor the 3×2's top-left
    // (`:34000-34001`).
    TempleBounds {
        t_left: cx - hw - 10,
        t_right: cx + hw + 10,
        t_top: cy - hh - 10,
        t_bottom: cy + hh + 10,
        t_rooms: 2,
        l_altar_x: altar_x,
        l_altar_y: altar_y - 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::noise::Rng;

    /// An active solid-kind tile patch — the synthetic jungle-grass stand-in.
    fn paint(world: &mut World, x0: i32, x1: i32, y0: i32, y1: i32, kind: u16) {
        for x in x0..x1 {
            for y in y0..y1 {
                let i = world.index(x, y);
                world.tiles[i] = Tile {
                    kind,
                    flags: FLAG_ACTIVE,
                    ..Default::default()
                };
            }
        }
    }

    /// DW-6.1: the acceptance semantics — with the whole vanilla y band
    /// painted as jungle grass (60), the first sample is accepted: the bury
    /// point sits on active tile 60, inside the C# x reach for `num3 = 0.25`
    /// (`(0.15·maxTilesX, 0.4·maxTilesX]` — `dungeonSide` fixed Right) and
    /// the y band `[(int)rockLayer, maxTilesY - 600)`. The draw ledger: the
    /// search consumed exactly 2 draws (the y `Next` then the x
    /// `NextDouble`), so the stream state equals a fresh rng after 2 draws.
    #[test]
    fn test_DW_6_1_search_accepts_jungle_grass() {
        let mut world = World::new("t".into(), 800, 800);
        // The band rows (50..200) painted slightly past the x reach edges
        // (118..322 covers [120, 320]) so the first sample always hits.
        paint(&mut world, 118, 322, 48, 202, 60);
        let mut rng = Rng(0xDEAD_BEEF);
        let site = find_temple_site(&world, 300, 50.0, &mut rng);
        assert!(site.0 >= 120 && site.0 <= 320, "x reach ({})", site.0);
        assert!(site.1 >= 50 && site.1 < 200, "y band ({})", site.1);
        let t = world.tile(site.0, site.1).unwrap();
        assert!(t.active() && t.kind == 60, "the bury point is jungle grass");
        // The draw ledger: `Next(num4, num5)` then `NextDouble()`.
        let mut fresh = Rng(0xDEAD_BEEF);
        fresh.next_u32();
        fresh.next_u32();
        assert_eq!(rng.0, fresh.0, "the search draws exactly y then x-fraction");
    }

    /// DW-6.1: the widening valve + the tile-60-only acceptance. Jungle grass
    /// exists only in the `num3 = 0.35`-exclusive reach `[0.05·W, 0.10·W)`
    /// (at 0.25 the reach starts past 0.15·W, at 0.30 past 0.10·W); every
    /// earlier sample lands on mud (59) and must not accept. The site can
    /// therefore only come from the two 1,000,000-attempt widening stages
    /// (0.25 → 0.30 → 0.35) — the C# counter semantics
    /// (`WorldGen.cs:15884-15893`, `Math.Min` landing `num3` exactly on 0.35).
    #[test]
    fn test_DW_6_1_search_widens_and_accepts_only_tile_60() {
        let mut world = World::new("t".into(), 800, 800);
        // Mud across the whole reach, then grass only in the widened reach.
        paint(&mut world, 40, 322, 48, 202, 59);
        paint(&mut world, 40, 80, 48, 202, 60);
        let mut rng = Rng(42);
        let site = find_temple_site(&world, 300, 50.0, &mut rng);
        assert!(site.0 > 40 && site.0 < 80, "the widened-reach x ({})", site.0);
        assert!(site.1 >= 50 && site.1 < 200, "y band ({})", site.1);
        let t = world.tile(site.0, site.1).unwrap();
        assert!(t.active() && t.kind == 60, "the bury point is jungle grass");
    }

    /// DW-6.1: the sub-vanilla valve — on `maxTilesY - 600 <= 0` the vanilla
    /// y band cannot exist in-world, so the search goes straight to the C#
    /// fallback (`WorldGen.cs:15926-15930`): `x = maxTilesX -
    /// generatingDungeonPositionX`, `y = (int)rockLayer + 100`, consuming no
    /// draws at all.
    #[test]
    fn test_DW_6_1_degenerate_band_goes_to_the_fallback() {
        let world = World::new("t".into(), 500, 500);
        let mut rng = Rng(999);
        let site = find_temple_site(&world, 300, 50.0, &mut rng);
        assert_eq!(site, (500 - 300, 50 + 100), "the fallback position");
        assert_eq!(rng.0, 999, "the valve skips the loop without drawing");
    }

    /// DW-7.1 (the batch-5-6 review note): the 11-stage give-up branch
    /// (`WorldGen.cs:15884-15893`) — a world with no jungle grass anywhere in
    /// the sampled band never accepts: the search widens 0.25 → 0.30 → 0.35,
    /// then runs 11 more 1,000,000-attempt stages at the cap (`num > 10`)
    /// and gives up to the C# fallback (`:15926-15930`). The draw ledger pins
    /// the full counter ladder — 13 widen-stage runs × 1,000,002 attempts ×
    /// 2 draws; a break one stage early would be 2,000,004 draws short.
    #[test]
    fn test_DW_7_1_temple_search_gives_up_after_eleven_cap_stages() {
        let world = World::new("t".into(), 800, 800); // no tile 60 anywhere
        let mut rng = Rng(0x5EED_5EED);
        let site = find_temple_site(&world, 300, 50.0, &mut rng);
        assert_eq!(site, (800 - 300, 50 + 100), "the C# fallback position");
        // The exact attempt ladder: two stages to reach the 0.35 cap + the
        // eleven cap stages, every stage 1,000,002 attempts (the
        // post-increment compare), 2 draws per attempt (the y `Next` + the
        // x `NextDouble`).
        let mut fresh = Rng(0x5EED_5EED);
        for _ in 0..13 * 1_000_002 {
            fresh.next_u32();
            fresh.next_u32();
        }
        assert_eq!(rng.0, fresh.0, "the full 11-stage give-up ladder consumed");
    }

    /// DW-6.1: the shell publishes the GenVars the M5-TEMPLE passes read —
    /// `t_rooms` is the carved-room count, the bounds are the filled extent
    /// ± 10 (`WorldGen.cs:33788-33817`, published `:34231-34235`) and
    /// contain both the bury point and the altar anchor; the 3×2 altar (237)
    /// sits at the anchor with the faithful frame layout
    /// (`frameX = i*18`, `frameY = j*18`, `:34049-34062`) over the brick
    /// base row. The valve path makes the anchor deterministic for the
    /// assertions that pin ranges (the `hw`/`hh` box rolls stay rng-drawn).
    #[test]
    fn test_DW_6_1_shell_publishes_bounds_and_altar_anchor() {
        let mut world = World::new("t".into(), 500, 500);
        let mut rng = Rng(7);
        let bounds = pass_lihzahrd_temple(&mut world, 300, 50.0, &mut rng);
        let (bx, by) = (200, 150); // the deterministic fallback bury point
        assert_eq!(bounds.t_rooms, 2);
        // The box ± 10 ranges: hw ∈ [34, 46], hh ∈ [22, 30].
        assert!(
            (144..=156).contains(&bounds.t_left)
                && (244..=256).contains(&bounds.t_right)
                && (110..=118).contains(&bounds.t_top)
                && (182..=190).contains(&bounds.t_bottom),
            "the published bounds {:?}",
            bounds
        );
        assert!(
            bounds.t_left <= bx
                && bx <= bounds.t_right
                && bounds.t_top <= by
                && by <= bounds.t_bottom,
            "the bury point sits inside the bounds"
        );
        // The altar anchor: hall_hw = hw - 6 ∈ [28, 40], hall_hh = hh - 8 ∈
        // [14, 22] → the anchor ranges are deterministic.
        assert!(
            (164..=176).contains(&bounds.l_altar_x)
                && (157..=165).contains(&bounds.l_altar_y),
            "the altar anchor ({},{})",
            bounds.l_altar_x,
            bounds.l_altar_y
        );
        assert!(
            bounds.t_left <= bounds.l_altar_x
                && bounds.l_altar_x + 2 < bounds.t_right
                && bounds.t_top <= bounds.l_altar_y
                && bounds.l_altar_y + 1 < bounds.t_bottom,
            "the anchor sits inside the bounds"
        );
        // The 3×2 altar with the faithful frames over the brick base row.
        for i in 0..3i32 {
            for j in 0..2i32 {
                let t = world
                    .tile(bounds.l_altar_x + i, bounds.l_altar_y + j)
                    .unwrap();
                assert!(
                    t.active()
                        && t.kind == TILE_ALTAR
                        && t.frame_x == (i * 18) as i16
                        && t.frame_y == (j * 18) as i16,
                    "the altar tile ({i},{j}) kind {} frames ({},{})",
                    t.kind,
                    t.frame_x,
                    t.frame_y
                );
            }
            let base = world.tile(bounds.l_altar_x + i, bounds.l_altar_y + 2).unwrap();
            assert!(
                base.active() && base.kind == TILE_TEMPLE_BRICK,
                "the brick base row at +2"
            );
        }
    }
}
