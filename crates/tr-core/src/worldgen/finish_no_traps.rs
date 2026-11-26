//! W4-g41 finish gap 1: the classic no-traps seed's own tail —
//! `WorldGen.FinishNoTraps` (`WorldGen.cs:25986-26224`), dispatched from
//! `GenerateWorld`'s tail FIRST (`:22423-22426`, before the anniversary
//! tail). Seam: `pipeline.rs::step_seed_finish`'s no-traps slot; the pass
//! draws from its own `pass_rng("FinishNoTraps")` stream, so a classic
//! world's pipeline is untouched (the `finish_seeds.rs` contract).
//!
//! What the pass ports, arm by arm:
//!
//! - The boulder solidity window (`:25988`/`:26186`, body `:11009-11020`):
//!   `tileSolid[138/484/664/711-716] = false` for the sweep, restored after.
//!   The port's boot table is immutable, so the window is the local
//!   [`solid_tile_window`] veto (the `not_the_bees.rs` 379-window precedent);
//!   nothing is restored after. 665 is NOT in the window list.
//! - The 50-border heart sweep (`:25994-26005`): the per-cell
//!   `Next(maxValue) == 0` draw runs FIRST (the C# `&&` order);
//!   `maxValue = 5`, 2 under FTW-&&!anniversary (`:25989-25993`). A framed
//!   heart 12 (frameX/frameY 0) becomes the 2×2 life-crystal boulder 665.
//! - The even-cell statue/plate-chest gate (`:26006`), the 3×3 red-wire
//!   veto (`:26010-26020`), the 625-draw ±25 wire search + the L-shaped red
//!   wire run (`:26023-26066`), the statue-only 225-attempt ±15
//!   pressure-plate placement (`:26067-26112` — `PlaceTile(135, forced,
//!   style Next(2,4))`, the style draw runs EVERY attempt), the 467-only
//!   (`!tenth || remix`) 64-attempt ±8 explosives placement on
//!   `{0, 1, Ore, 59, 151}` cells with a fully-solid 3×3 (`:26114-26178`)
//!   and the **`else { j++; }`** row-skip (`:26180-26183` — it fires for
//!   statue cells AND for 467 cells under `tenth && !remix`).
//! - The 20-border unwired plate-chest sweep (`:26187-26203`): a 467
//!   frameX 144 frameY 0 whose 2×2 cells are all wire-free becomes the 2×2
//!   chest 21 at frameX 36/54.
//!
//! Deviations (the census `docs/research/95-finish-no-traps-remix-census.md`):
//!
//! - **The chest 5346 add-on** (`:26204-26223`) ports to nothing — the
//!   port's chests carry no item stacks, so the 8000-chest loop iterates
//!   nothing: no RNG draws, no writes (the `finish_seeds.rs` module note).
//! - **The plate/explosives placements** ride the port's `Place1x1` slice
//!   (`:45188-45235` — the `mini_biomes.rs::place_explosives_tile`
//!   precedent): `InWorld(x, y, 2)`, the mechanism-on-boulder floor veto
//!   (plates only), a `SolidTile2` floor and an empty cell. `forced: true`
//!   bypasses the `PlaceTile` occupancy gate (`:59373`) only, not this one.
//! - **`TileFrame(resetFrame: true)`** (`:26145`) rides the post-pipeline
//!   framer (the M5-STRUCT-5 idiom); the kind-141 recheck (`:26146`) reads
//!   the written kind.

use super::noise::Rng;
use super::pipeline::Streams;
use super::GenOptions;
use crate::edit::solid_tile2;
use crate::seeds::SeedFlags;
use crate::tile::FLAG_ACTIVE;
use crate::tiledata as td;
use crate::world::World;
use crate::wiring::WireColor;

// --- the tile ids the sweeps read ------------------------------------------

/// `TileID.Heart` (`TileID.cs:653`) — the life-crystal boulder's source.
const TILE_HEART: u16 = 12;
/// `TileID.Containers` (`TileID.cs:671`) — the chest the plate-chest becomes.
const TILE_CHEST: u16 = 21;
/// `TileID.Statues` (`TileID.cs:839`).
const TILE_STATUE: u16 = 105;
/// `TileID.PressurePlates` (`TileID.cs:899`) — the statue wiring's trigger.
const TILE_PRESSURE_PLATE: u16 = 135;
/// `TileID.Explosives` (`TileID.cs:911`) — the plate-chest wiring's trigger.
const TILE_EXPLOSIVES: u16 = 141;
/// `TileID.SandstoneBrick` (`TileID.cs:1019`-area) — an explosives seat.
const TILE_SANDSTONE_BRICK: u16 = 151;
/// `TileID.Containers2` (`TileID.cs:1563`) — the trapped chest family.
const TILE_PLATE_CHEST: u16 = 467;
/// `TileID.LifeCrystalBoulder` — the heart sweep's target.
const TILE_LIFE_CRYSTAL_BOULDER: u16 = 665;

/// `WorldGen.SetBoulderSolidity`'s nine kinds (`WorldGen.cs:11009-11020`) —
/// the sweep-window `tileSolid` veto. 665 is deliberately NOT in the list.
const BOULDER_WINDOW: [u16; 9] = [138, 484, 664, 711, 712, 713, 714, 715, 716];

// --- small helpers ----------------------------------------------------------

fn kind_at(world: &World, x: i32, y: i32) -> u16 {
    world.tile(x, y).map_or(0, |t| t.kind)
}

fn active_at(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| t.active())
}

fn wire_at(world: &World, x: i32, y: i32) -> bool {
    world.wires.get(x, y, WireColor::Red)
}

fn set_wire(world: &mut World, x: i32, y: i32) {
    world.wires.set(x, y, WireColor::Red, true);
}

/// `WorldGen.SolidTile` (`:70111-70132`) under the open boulder window:
/// the nine `SetBoulderSolidity` kinds read non-solid (the boot table is
/// immutable — the window is this veto, the `not_the_bees.rs` precedent).
fn solid_tile_window(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && td::TILE_SOLID.has(t.kind)
        && !BOULDER_WINDOW.contains(&t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// The L-shaped red-wire run (`:26041-26065`, the two copies `:26086-26110`
/// / `:26152-26176` are the same shape): the start cell, the horizontal run
/// at the START row towards the target column, then the vertical run at the
/// TARGET column towards the target row (the `mini_biomes.rs::wire_line`
/// endpoint-capture precedent).
fn wire_run(world: &mut World, sx: i32, sy: i32, tx: i32, ty: i32) {
    let (mut x, mut y) = (sx, sy);
    set_wire(world, x, y);
    while x != tx {
        if x < tx {
            x += 1;
        }
        if x > tx {
            x -= 1;
        }
        set_wire(world, x, y);
    }
    while y != ty {
        if y < ty {
            y += 1;
        }
        if y > ty {
            y -= 1;
        }
        set_wire(world, x, y);
    }
}

/// `WorldGen.PlaceTile(x, y, 135, mute, forced, -1, style)`'s `Place1x1`
/// slice (`:60130-60140` → `:45188-45235`): `InWorld(x, y, 2)`, the
/// mechanism-on-boulder floor veto (`IsAMechanism[135]` is true,
/// `TileID.cs` Wiring sets; `Boulders` includes 665 — `TileID.cs:504`), a
/// `SolidTile2` floor and an empty cell, then active + kind 135 +
/// `frameY = 18 * style` (the default branch; `frameX` keeps its stored
/// value). The `forced` flag bypasses the `PlaceTile` occupancy gate only.
fn place_plate(world: &mut World, x: i32, y: i32, style: i32) {
    if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
        return;
    }
    if td::SET_BOULDERS.has(kind_at(world, x, y + 1)) {
        return;
    }
    if !solid_tile2(world, x, y + 1) || active_at(world, x, y) {
        return;
    }
    let i = world.index(x, y);
    let t = &mut world.tiles[i];
    t.flags |= FLAG_ACTIVE;
    t.kind = TILE_PRESSURE_PLATE;
    t.frame_y = (18 * style) as i16;
}

/// `WorldGen.Place1x1(x, y, 141, style 0)` (`:45188-45235`) — the
/// explosives write (the `mini_biomes.rs::place_explosives_tile` verbatim
/// slice): no boulder veto (141 is not a mechanism), `frameY = 0`.
fn place_explosives(world: &mut World, x: i32, y: i32) {
    if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
        return;
    }
    if !solid_tile2(world, x, y + 1) || active_at(world, x, y) {
        return;
    }
    let i = world.index(x, y);
    let t = &mut world.tiles[i];
    t.flags |= FLAG_ACTIVE;
    t.kind = TILE_EXPLOSIVES;
    t.frame_y = 0;
}

// --- the pass body ----------------------------------------------------------

/// `WorldGen.FinishNoTraps` (`WorldGen.cs:25986-26224`): the no-traps
/// world's trap re-wiring — hearts become life-crystal boulders, unwired
/// statues grow pressure plates on red wires, unwired trapped chests grow
/// explosives, and the leftover plate-chests become plain chests.
pub(crate) fn finish_no_traps(world: &mut World, streams: &mut Streams, opts: &GenOptions) {
    let flags = opts.flags;
    if !flags.contains(SeedFlags::NO_TRAPS) {
        return;
    }
    let mut rng = streams.pass_rng("FinishNoTraps");
    let (width, height) = (world.width, world.height);
    let tenth = flags.contains(SeedFlags::TENTH_ANNIVERSARY);
    let remix = flags.contains(SeedFlags::REMIX);
    // `:25989-25993` — the heart roll's band: 5, or 2 under FTW-&&!anniversary.
    let max_value = if flags.contains(SeedFlags::FOR_THE_WORTHY) && !tenth { 2 } else { 5 };

    // `:25994-26184` — the 50-border sweep (the boulder window is open).
    for i in 50..width - 50 {
        let mut j = 50;
        while j < height - 50 {
            // `:25999-26005` — the draw runs FIRST (the C# `&&` order), for
            // EVERY cell of the interior; a framed heart becomes the 2×2
            // life-crystal boulder.
            if rng.range(0, max_value) == 0 {
                let heart = world
                    .tile(i, j)
                    .is_some_and(|t| t.active() && t.kind == TILE_HEART && t.frame_x == 0 && t.frame_y == 0);
                if heart {
                    for &(dx, dy) in &[(0i32, 0i32), (0, 1), (1, 0), (1, 1)] {
                        if let Some(t) = world.tile_mut(i + dx, j + dy) {
                            t.kind = TILE_LIFE_CRYSTAL_BOULDER;
                        }
                    }
                }
            }
            // `:26006` — the even-cell statue/plate-chest gate.
            let gate = i % 2 == 0
                && j % 2 == 0
                && world.tile(i, j).is_some_and(|t| {
                    t.active()
                        && (t.kind == TILE_STATUE
                            || (t.kind == TILE_PLATE_CHEST && (t.frame_x == 144 || t.frame_x == 162)))
                });
            if gate {
                statue_chest_cell(world, &mut rng, i, j, tenth, remix);
            }
            // `:26114/:26180-26183` — the row-skip: the `else` binds to the
            // 467 test, so a statue cell skips a row too, and a 467 cell
            // under `tenth && !remix` takes the else as well.
            if !(kind_at(world, i, j) == TILE_PLATE_CHEST && (!tenth || remix)) {
                j += 1;
            }
            j += 1;
        }
    }

    // `:26186` — SetBoulderSolidity(true): the boot table already carries
    // the solid kinds; the window veto above simply stops applying.

    // `:26187-26203` — the 20-border unwired plate-chest sweep.
    for x in 20..width - 20 {
        for y in 20..height - 20 {
            let plate_chest = world.tile(x, y).is_some_and(|t| {
                t.active() && t.kind == TILE_PLATE_CHEST && t.frame_x == 144 && t.frame_y == 0
            });
            if !plate_chest {
                continue;
            }
            if wire_at(world, x, y)
                || wire_at(world, x + 1, y)
                || wire_at(world, x, y + 1)
                || wire_at(world, x + 1, y + 1)
            {
                continue;
            }
            // The 2×2 chest at frameX 36/54 (the C# writes all four cells).
            for &(dx, dy, fx) in &[(0i32, 0i32, 36i16), (1, 0, 54), (0, 1, 36), (1, 1, 54)] {
                if let Some(t) = world.tile_mut(x + dx, y + dy) {
                    t.kind = TILE_CHEST;
                    t.frame_x = fx;
                }
            }
        }
    }
    // `:26204-26223` — the 8000-chest 1-in-15 item-5346 add-on ports to
    // nothing: the port's chests carry no item stacks (the module note).
}

/// The gated statue/plate-chest cell (`:26010-26178`): the 3×3 wire veto,
/// the ±25 wire search, then the statue's plate placement or the
/// plate-chest's explosives placement, each with its own L-shaped wire run.
fn statue_chest_cell(world: &mut World, rng: &mut Rng, i: i32, j: i32, tenth: bool, remix: bool) {
    // `:26010-26021` — any red wire in the 3×3 around the cell vetoes the
    // SEARCH arms only; the 467 explosives arm below sits OUTSIDE the
    // `if (!flag)` block in the C# and runs regardless.
    let mut near_wire = false;
    for k in i - 1..=i + 1 {
        for l in j - 1..=j + 1 {
            if wire_at(world, k, l) {
                near_wire = true;
            }
        }
    }
    let mut num2 = -1i32;
    let mut num3 = -1i32;
    if !near_wire {
        // `:26023-26036` — the 625-draw ±25 wire search (x draw first).
        let num = 25;
        let mut found_wire = false;
        for _ in 0..num * num {
            num2 = rng.range(i - num, i + num + 1);
            num3 = rng.range(j - num, j + num + 1);
            if wire_at(world, num2, num3) {
                found_wire = true;
                break;
            }
        }
        if found_wire {
            // `:26039-26065` — wire the cell to the found wire.
            wire_run(world, i, j, num2, num3);
        } else if kind_at(world, i, j) == TILE_STATUE {
            // `:26067-26111` — the statue's 225-attempt ±15 plate placement.
            // The style draw runs EVERY attempt (it is the PlaceTile argument).
            let num = 15;
            let mut placed = false;
            for _ in 0..num * num {
                num2 = i + rng.range(-num, num + 1);
                num3 = j + rng.range(-num, num + 1);
                let style = rng.range(2, 4);
                place_plate(world, num2, num3, style);
                if kind_at(world, num2, num3) == TILE_PRESSURE_PLATE {
                    placed = true;
                    break;
                }
            }
            if placed {
                wire_run(world, i, j, num2, num3);
            }
        }
    }
    // `:26114-26178` — the plate-chest's explosives placement (the row-skip
    // decision lives in the caller, reading the same gate).
    if kind_at(world, i, j) == TILE_PLATE_CHEST && (!tenth || remix) {
        let num8 = 8;
        for _ in 0..num8 * num8 {
            let num10 = i + rng.range(-num8, num8 + 1);
            let num11 = j + rng.range(-num8, num8 + 1);
            // `:26121` — the seat kinds: dirt/stone/ore/mud/sandstone brick.
            let seat = matches!(kind_at(world, num10, num11), 0 | 1 | 59 | TILE_SANDSTONE_BRICK)
                || td::SET_ORE.has(kind_at(world, num10, num11));
            if !seat {
                continue;
            }
            // `:26125-26139` — the fully-solid 3×3 (the boulder window).
            let mut all_solid = true;
            for k in num10 - 1..=num10 + 1 {
                for l in num11 - 1..=num11 + 1 {
                    if !solid_tile_window(world, k, l) {
                        all_solid = false;
                    }
                }
            }
            if !all_solid {
                continue;
            }
            // `:26140-26145` — the C# writes 141 over the seat DIRECTLY,
            // then `TileFrame(resetFrame: true)` validates it: an
            // explosives needs a solid floor (`CheckExplosives`), so the
            // port models the write + the check as the `Place1x1` slice
            // (the `mini_biomes.rs` precedent). A failed placement keeps
            // the seat — the C# would pop the unsupported 141 back to air;
            // the seat-keeping is the documented framer-seam deviation.
            place_explosives(world, num10, num11);
            // `:26146-26149` — the kind recheck after the frame check.
            if kind_at(world, num10, num11) != TILE_EXPLOSIVES {
                continue;
            }
            wire_run(world, i, j, num10, num11);
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small all-air test world (the `finish_seeds.rs` test idiom).
    fn air(width: i32, height: i32) -> World {
        World::new("nt".into(), width, height)
    }

    fn put(world: &mut World, x: i32, y: i32, kind: u16) {
        let i = world.index(x, y);
        world.tiles[i].kind = kind;
        world.tiles[i].flags |= FLAG_ACTIVE;
    }

    fn framed(world: &mut World, x: i32, y: i32, kind: u16, fx: i16, fy: i16) {
        put(world, x, y, kind);
        let i = world.index(x, y);
        world.tiles[i].frame_x = fx;
        world.tiles[i].frame_y = fy;
    }

    fn opts(flags: u16) -> GenOptions {
        GenOptions {
            flags: SeedFlags::from(flags),
            ..Default::default()
        }
    }

    /// The seed gate: a classic world never enters the pass — the tiles
    /// and wires are byte-identical.
    #[test]
    fn classic_world_is_untouched() {
        let mut w = air(300, 300);
        framed(&mut w, 100, 100, TILE_HEART, 0, 0);
        put(&mut w, 120, 120, TILE_STATUE);
        let before = w.tiles.clone();
        finish_no_traps(&mut w, &mut Streams::new(7), &opts(0));
        assert_eq!(w.tiles, before, "a classic world reads no NO_TRAPS flag");
        assert!(!wire_at(&w, 100, 100) && !wire_at(&w, 120, 120), "no wire draws");
    }

    /// The L-run shape (`:26041-26065`): horizontal at the START row,
    /// vertical at the TARGET column — both directions.
    #[test]
    fn the_wire_run_is_l_shaped() {
        let mut w = air(300, 300);
        wire_run(&mut w, 100, 100, 104, 97);
        for x in 100..=104 {
            assert!(wire_at(&w, x, 100), "the horizontal run rides row 100");
        }
        for y in 97..=100 {
            assert!(wire_at(&w, 104, y), "the vertical run rides column 104");
        }
        assert!(!wire_at(&w, 100, 97), "the corner (start.x, end.y) stays unwired");
    }

    /// The plate write (`:45188-45235`): a SolidTile2 floor + empty cell
    /// places 135 at frameY 18*style; an occupied cell or a boulder floor
    /// refuses (665 IS in the Boulders veto set).
    #[test]
    fn the_plate_needs_a_solid_floor_and_refuses_boulder_seats() {
        let mut w = air(300, 300);
        put(&mut w, 100, 101, 1); // stone floor
        place_plate(&mut w, 100, 100, 3);
        let t = w.tiles[w.index(100, 100)];
        assert!(t.active() && t.kind == TILE_PRESSURE_PLATE);
        assert_eq!(t.frame_y, 54);
        // occupied cell refuses
        place_plate(&mut w, 100, 100, 2);
        assert_eq!(w.tiles[w.index(100, 100)].frame_y, 54);
        // boulder floor refuses (the mechanism-on-boulder veto)
        put(&mut w, 120, 121, TILE_LIFE_CRYSTAL_BOULDER);
        place_plate(&mut w, 120, 120, 2);
        assert!(!active_at(&w, 120, 120));
        // air floor refuses
        place_plate(&mut w, 140, 140, 2);
        assert!(!active_at(&w, 140, 140));
    }

    /// The boulder window (`:11009-11020`): the nine kinds read non-solid
    /// inside the sweep; 665 is NOT in the window (it stays solid).
    #[test]
    fn the_boulder_window_vetoes_the_nine_kinds_only() {
        let mut w = air(300, 300);
        put(&mut w, 100, 100, 138);
        put(&mut w, 110, 100, 665);
        put(&mut w, 120, 100, 1);
        assert!(!solid_tile_window(&w, 100, 100), "138 is windowed out");
        // 665 is NOT in the nine-kind window, but it is also not `tileSolid`
        // at all (frame-important life-crystal boulder, `Main.cs:7434`): it
        // never reads solid in the first place.
        assert!(!solid_tile_window(&w, 110, 100), "665 is not a solid tile");
        assert!(solid_tile_window(&w, 120, 100));
    }

    /// The unwired plate-chest sweep (`:26187-26203`): a frameX-144 467
    /// with wire-free 2×2 becomes the chest 21 at frameX 36/54; a wired
    /// one stays.
    #[test]
    fn unwired_plate_chests_become_plain_chests() {
        let mut w = air(300, 300);
        framed(&mut w, 100, 100, TILE_PLATE_CHEST, 144, 0);
        framed(&mut w, 101, 100, TILE_PLATE_CHEST, 162, 0);
        framed(&mut w, 100, 101, TILE_PLATE_CHEST, 144, 18);
        framed(&mut w, 101, 101, TILE_PLATE_CHEST, 162, 18);
        framed(&mut w, 150, 150, TILE_PLATE_CHEST, 144, 0);
        set_wire(&mut w, 151, 151); // the second chest's 2×2 carries a wire
        finish_no_traps(&mut w, &mut Streams::new(7), &opts(SeedFlags::NO_TRAPS));
        assert_eq!(kind_at(&w, 100, 100), TILE_CHEST);
        assert_eq!(w.tiles[w.index(100, 100)].frame_x, 36);
        assert_eq!(w.tiles[w.index(101, 100)].frame_x, 54);
        assert_eq!(w.tiles[w.index(100, 101)].frame_x, 36);
        assert_eq!(w.tiles[w.index(101, 101)].frame_x, 54);
        assert_eq!(kind_at(&w, 150, 150), TILE_PLATE_CHEST, "the wired chest stays");
    }

    /// The statue wiring (`:26023-26066`): a statue whose ±25 search box
    /// is dense with red wire (outside its 3×3 veto) grows the L-run to
    /// the found wire (no plate placement). The dense box makes the
    /// 625-draw search hit on the first non-veto draw.
    #[test]
    fn a_statue_near_a_wire_grows_the_l_run() {
        let mut w = air(300, 300);
        put(&mut w, 100, 100, TILE_STATUE);
        for x in 75i32..=125 {
            for y in 75i32..=125 {
                if (x - 100).abs() > 1 || (y - 100).abs() > 1 {
                    set_wire(&mut w, x, y);
                }
            }
        }
        finish_no_traps(&mut w, &mut Streams::new(7), &opts(SeedFlags::NO_TRAPS));
        assert!(wire_at(&w, 100, 100), "the statue cell wires");
        assert_eq!(kind_at(&w, 100, 100), TILE_STATUE, "no plate replaces the statue");
    }

    /// The full-pipeline a==b witness: a NO_TRAPS world generates
    /// byte-identically across two runs and differs from a classic world.
    /// Written NOT run (code-only).
    #[test]
    fn the_no_traps_world_is_reproducible_and_distinct_from_classic() {
        let nt = GenOptions {
            width: 512,
            height: 512,
            seed: 42,
            flags: SeedFlags::from(SeedFlags::NO_TRAPS),
            ..Default::default()
        };
        let a = crate::worldgen::generate(&nt);
        let b = crate::worldgen::generate(&nt);
        assert_eq!(a.tiles, b.tiles, "the NO_TRAPS world is deterministic");
        let classic = GenOptions { width: 512, height: 512, seed: 42, ..Default::default() };
        let c = crate::worldgen::generate(&classic);
        assert_ne!(a.tiles, c.tiles, "the no-traps tail changes the world");
    }
}
