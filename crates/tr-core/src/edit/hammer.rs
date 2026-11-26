//! The hammer interaction (S1-P-i31): `WorldGen.CanPoundTile` / `ForbidsSloping`
//! / `SlopeTile` / `PoundTile` (`WorldGen.cs:80695-80838`) and the player-side
//! pound cycle `Player.ItemCheck_UseMiningTools_TryPoundingTile`
//! (`Player.cs:45969-46311`), the hammer-target predicates (`:45536-45556`),
//! `Player.CanPlayerSmashWall` (`:45917-45940`) and
//! `ItemCheck_UseMiningTools_TryFindingWallToHammer` (`:45832-45906`).
//!
//! The world-state halves live here; the sound cues, the player lift's
//! `gfxOffY` bookkeeping and the `poundRelease` latch ride the app seam
//! (`world_sim/editing.rs`). The generation-time slices
//! (`worldgen/temple.rs::slope_tile_p2`, `tileobject.rs::slope_tile_flatten`)
//! predate this port and keep their documented ungated shapes — the player
//! path uses the gated functions below.

use super::{can_kill_tile, is_locked_door, solid_tile, touch};
use crate::tiledata as td;
use crate::unified_random::UnifiedRandom;
use crate::world::World;

/// `WorldGen.ForbidsSloping` (`WorldGen.cs:80745-80766`): the furniture kinds
/// whose presence ABOVE a cell refuses the pound/slope (21 Chest, 26
/// DemonAltar, 77 Hellforge, 88 Dresser, 235 Teleporter, 237 Timer, 441
/// WeaponRack, 467 Chest2, 468 Chest3, 470 DisplayDoll, 475 HatRack, 488
/// FallenLog, 597 Pylon — `TileID.cs`).
pub fn forbids_sloping(kind: u16) -> bool {
    matches!(
        kind,
        21 | 26 | 77 | 88 | 235 | 237 | 441 | 467 | 468 | 470 | 475 | 488 | 597
    )
}

/// `WorldGen.CanPoundTile` (`WorldGen.cs:80695-80743`). `generating` mirrors
/// `WorldGen.isGeneratingOrLoadingWorld` (the 190 mushroom / 30 wood
/// exclusions only apply during generation, `:80726-80736`); the player path
/// passes `false`. The null-tile guards (`:80697-80708`) fold into the port's
/// `Option` reads — an out-of-world cell refuses.
pub fn can_pound_tile(
    world: &World,
    x: i32,
    y: i32,
    generating: bool,
    chest_holds_items: impl Fn(i32, i32) -> bool,
) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    // `:80709-80720` — the mechanism/container switch.
    if matches!(t.kind, 10 | 48 | 137 | 232 | 380 | 387 | 388 | 476 | 484) {
        return false;
    }
    // `:80722-80725` — `TileID.Sets.Boulders`.
    if td::SET_BOULDERS.has(t.kind) {
        return false;
    }
    // `:80726-80736` — the generation-only exclusions.
    if generating && (t.kind == 190 || t.kind == 30) {
        return false;
    }
    // `:80737-80740` — the active furniture above forbids sloping. The C#
    // reads `Main.tile[x, y - 1]` (null-initialized at `:80701-80704`); an
    // out-of-world above reads as inactive here.
    if world
        .tile(x, y - 1)
        .is_some_and(|a| a.active() && forbids_sloping(a.kind))
    {
        return false;
    }
    // `:80741` — the break refusals.
    can_kill_tile(world, x, y, chest_holds_items)
}

/// `WorldGen.SlopeTile(i, j, slope, noEffects: false, quiet: true)`
/// (`WorldGen.cs:80768-80803`): the `CanPoundTile` gate, then
/// `halfBrick(false); slope(slope)`. The non-generation effects are the
/// fail-dust (`KillTile(fail, effectOnly)` — skipped, the port has no
/// per-tick dust path), the Dig 0 sound (app-side) and `SquareTileFrame`
/// (`:80783` — the port's `square_tile_frame_default`, whose object drops
/// are discarded like the wiring callers'). The flatten player-push
/// (`:80785-80796`) is app-side: the caller reads `slope() == 0` after a
/// `true` return. `noEffects`/`quiet` never vary at the ported call sites.
///
/// `rng` is the ACTIVE `genRand` stream of the caller's context (W4-g83
/// contract §2 — the player pound runs under the `UpdatePlayers` swap); the
/// reframe's framing dispatch draws its seaweed tip re-rolls on it.
pub fn slope_tile(
    world: &mut World,
    x: i32,
    y: i32,
    slope: u8,
    generating: bool,
    chest_holds_items: impl Fn(i32, i32) -> bool,
    rng: &mut UnifiedRandom,
) -> bool {
    if !can_pound_tile(world, x, y, generating, chest_holds_items) {
        return false;
    }
    let i = world.index(x, y);
    world.tiles[i].flags &= !crate::tile::FLAG_HALF_BRICK;
    world.tiles[i].set_slope(slope);
    if !generating {
        // `:80783` — the 3×3 reframe (the cascade's drops ride the wiring
        // precedent: `let _ = square_tile_frame_default`, ANALYSIS).
        let _ = super::square_tile_frame_default(world, x, y, rng);
    } else {
        touch(world, x, y);
    }
    true
}

/// `WorldGen.PoundTile(i, j)` (`WorldGen.cs:80805-80838`): the `CanPoundTile`
/// gate, then the half-brick TOGGLE. The effects mirror [`slope_tile`]; the
/// un-half player-push (`:80824-80835`) is app-side (the caller reads
/// `!half_brick()` after a `true` return). `rng` threads the caller's ACTIVE
/// `genRand` stream (W4-g83 contract §2, the [`slope_tile`] note).
pub fn pound_tile(
    world: &mut World,
    x: i32,
    y: i32,
    generating: bool,
    chest_holds_items: impl Fn(i32, i32) -> bool,
    rng: &mut UnifiedRandom,
) -> bool {
    if !can_pound_tile(world, x, y, generating, chest_holds_items) {
        return false;
    }
    let i = world.index(x, y);
    // `:80811-80818` — the toggle.
    world.tiles[i].flags ^= crate::tile::FLAG_HALF_BRICK;
    if !generating {
        // `:80823`.
        let _ = super::square_tile_frame_default(world, x, y, rng);
    } else {
        touch(world, x, y);
    }
    true
}

/// `Tile.SmoothSlope` (`Tile.cs:792-865`, W4-d8 retire (1)): the
/// neighbour-aware slope picker the Drill's `UseDrill` calls after every
/// block pick (`Mount.cs:3119`, `applyToNeighbors: true, sync: true`).
/// `apply_to_neighbors` smooths the four cardinals FIRST (each single-cell),
/// then the cell itself — the C# recursion order. The `sync` arm
/// (`:847-864`, NetMessage 17 sub-commands 23/7/14) is MP transport; the
/// port's edit seams queue their own tile-sync, so the flag is carried for
/// the citation but the sends ride the caller's net seam. The worldgen
/// callers (`MarbleBiome.cs:150`, `GraniteBiome.cs:326`, `WorldGen.cs:16586`,
/// `TrackGenerator.cs:114/117`, the dungeon pillars) keep their own lane
/// copies until the g40 merge — THIS is the play-path (M2-MOUNT-4) seam.
/// `generating` mirrors `WorldGen.isGeneratingOrLoadingWorld` (the C# reads
/// the global inside `CanPoundTile`; the port threads it like `slope_tile`
/// — the drill passes false, the g40 worldgen copies pass true; the merge
/// keep-one dedupes onto THIS signature).
pub fn smooth_slope(
    world: &mut World,
    x: i32,
    y: i32,
    apply_to_neighbors: bool,
    generating: bool,
    chest_holds_items: impl Fn(i32, i32) -> bool + Copy,
) {
    if apply_to_neighbors {
        // `:796-799` — right, left, below, above, each `applyToNeighbors: false`.
        smooth_slope_one(world, x + 1, y, generating, chest_holds_items);
        smooth_slope_one(world, x - 1, y, generating, chest_holds_items);
        smooth_slope_one(world, x, y + 1, generating, chest_holds_items);
        smooth_slope_one(world, x, y - 1, generating, chest_holds_items);
    }
    smooth_slope_one(world, x, y, generating, chest_holds_items);
}

/// The single-cell body (`Tile.cs:801-846`): the `CanPoundTile` +
/// `SolidOrSlopedTile` gate, then the 4-neighbour mask → slope/half-brick
/// switch. No RNG. `flag2` (a non-solid but non-empty tile above) vetoes
/// the two top-slope arms WITHOUT falling through to the default — a
/// pre-existing slope under a torch keeps its state.
pub fn smooth_slope_one(
    world: &mut World,
    x: i32,
    y: i32,
    generating: bool,
    chest_holds_items: impl Fn(i32, i32) -> bool,
) {
    // `:802-805` — the pound gate.
    if !can_pound_tile(world, x, y, generating, chest_holds_items)
        || !solid_or_sloped_at(world, x, y)
    {
        return;
    }
    // `:806-811` — `flag` reads TileEmpty (absent OR actuated-off is empty),
    // the other three read SolidOrSlopedTile.
    let flag = !tile_empty_at(world, x, y - 1);
    let flag2 = !solid_or_sloped_at(world, x, y - 1) && flag;
    let flag3 = solid_or_sloped_at(world, x, y + 1);
    let flag4 = solid_or_sloped_at(world, x - 1, y);
    let flag5 = solid_or_sloped_at(world, x + 1, y);
    let mask =
        ((flag as i32) << 3) | ((flag3 as i32) << 2) | ((flag4 as i32) << 1) | (flag5 as i32);
    let Some(t) = world.tile_mut(x, y) else {
        return;
    };
    match mask {
        // `:816-829` — the flag2 veto breaks without a write.
        10 => {
            if !flag2 {
                t.flags &= !crate::tile::FLAG_HALF_BRICK;
                t.set_slope(3);
            }
        }
        9 => {
            if !flag2 {
                t.flags &= !crate::tile::FLAG_HALF_BRICK;
                t.set_slope(4);
            }
        }
        // `:830-837` — the two bottom ramps.
        6 => {
            t.flags &= !crate::tile::FLAG_HALF_BRICK;
            t.set_slope(1);
        }
        5 => {
            t.flags &= !crate::tile::FLAG_HALF_BRICK;
            t.set_slope(2);
        }
        // `:838-841` — below-only: the half brick.
        4 => {
            t.set_slope(0);
            t.flags |= crate::tile::FLAG_HALF_BRICK;
        }
        // `:842-845` — re-solidify.
        _ => {
            t.flags &= !crate::tile::FLAG_HALF_BRICK;
            t.set_slope(0);
        }
    }
}

/// `WorldGen.SolidOrSlopedTile(x, y)` (`WorldGen.cs:69985-70023`,
/// `includePlatforms: false`): active, solid, not solid-top, not actuated.
fn solid_or_sloped_at(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| {
        t.active() && td::TILE_SOLID.has(t.kind) && !td::TILE_SOLID_TOP.has(t.kind) && !t.inactive()
    })
}

/// `WorldGen.TileEmpty` (`WorldGen.cs:69976-69983`): absent or actuated-off
/// reads empty.
fn tile_empty_at(world: &World, x: i32, y: i32) -> bool {
    !world.tile(x, y).is_some_and(|t| t.active() && !t.inactive())
}

/// `Player.IsTilePoundable` (`Player.cs:45545-45556`): not a `tileHammer`
/// tile, and either solid or one of the five special non-solids (314
/// MinecartTrack, 424 WirePipe, 442 ProjectilePressurePad, 351 ChimneySmoke,
/// 470 DisplayDoll — `TileID.cs:1257/:1477/:1513/:1331/:1569`) or a living
/// fire (`TileID.Sets.IsLivingFire`, `TileID.cs:533`: 336/340-344).
pub fn is_tile_poundable(kind: u16) -> bool {
    if crate::tileobject::tile_hammer(kind) {
        return false;
    }
    if !td::TILE_SOLID.has(kind)
        && kind != 314
        && kind != 424
        && kind != 442
        && kind != 351
        && kind != 470
    {
        return td::SET_IS_LIVING_FIRE.has(kind);
    }
    true
}

/// `Player.IsAHammerTarget` (`Player.cs:45536-45543`): a `tileHammer` tile or
/// a poundable one — the `poundRelease` clear gate (`:45611`).
pub fn is_a_hammer_target(kind: u16) -> bool {
    crate::tileobject::tile_hammer(kind) || is_tile_poundable(kind)
}

// Gate merge (i31 ⨉ i33): `can_player_smash_wall` lives ONCE, in
// `edit::wall` (the i33 copy) — it reads the FULL `Main.wallHouse` table
// (`wall_house`, 279 members, `Main.cs:9741-9789`/`:10353-10577`), which is
// the table `Player.cs:45924` actually reads; the i31 copy used the narrower
// housing subset (recorded deviation, corrected at the gate). The finder
// below rides the same full table. Since the g44e dedupe (W4-d119)
// `housing::is_house_wall` delegates to this table too — one copy, all gates.

/// `Player.ItemCheck_UseMiningTools_TryFindingWallToHammer`
/// (`Player.cs:45832-45906`): the wall cell the hammer's wall pass targets.
/// `mouse` is the mouse WORLD position (`Main.mouseX + Main.screenPosition`).
///
/// The cursor cell keeps its coordinates when it holds a house wall
/// (`flag = false`, `:45855`), an unsafe wall with a differing 3×3 neighbour
/// (the edge, `:45845-45848`) or an active tile (`:45858`). Otherwise — an
/// interior unsafe wall or a wall-less cell — the mouse-side 2×2 scan runs
/// (`:45862-45905`): the quadrant offsets come from the BANKER'S-rounded
/// mouse tile (`Math.Round`, `:45863-45871` — a `.5` fraction rounds to
/// even, so the scan side flips with the tile's parity). The scan overwrites
/// `(wX, wY)` with EVERY visited cell (`:45880-45881`, before the wall
/// check) and stops assigning once `flag` falls — the C#'s last-cell quirk,
/// kept 1:1.
pub fn try_finding_wall_to_hammer(world: &World, tx: i32, ty: i32, mouse: (f32, f32)) -> (i32, i32) {
    let (mut wx, mut wy) = (tx, ty);
    let mut flag = true;
    let wall = world.tile(wx, wy).map(|t| t.wall).unwrap_or(0);
    if wall > 0 {
        // `:45839` — `Main.wallHouse` is the FULL 279-wall table
        // (`edit::wall_house`, S1-P-i33) — the same table
        // `housing::is_house_wall` now delegates to (the W4-g44 dedupe).
        if !super::wall_house(wall) {
            // `:45841-45851` — the same-wall 3×3 census.
            for i in wx - 1..wx + 2 {
                for j in wy - 1..wy + 2 {
                    if world.tile(i, j).map(|n| n.wall).unwrap_or(0) != wall {
                        flag = false;
                        break;
                    }
                }
            }
        } else {
            flag = false;
        }
    }
    // `:45858-45861`.
    if !flag || world.tile(wx, wy).is_some_and(|t| t.active()) {
        return (wx, wy);
    }
    // `:45862-45871` — the quadrant offsets: `(float)/16f < Math.Round(...)`
    // picks the lower cell when the fraction rounds UP. The C# divides in
    // f32 then rounds the promoted double — the port mirrors both steps.
    let fx = mouse.0 / 16.0;
    let fy = mouse.1 / 16.0;
    let num = if (fx as f64) < crate::damage::round_bankers(fx as f64) as f64 {
        0
    } else {
        -1
    };
    let num2 = if (fy as f64) < crate::damage::round_bankers(fy as f64) as f64 {
        0
    } else {
        -1
    };
    // `:45872-45905` — the 2×2 scan.
    for k in tx + num..=tx + num + 1 {
        for l in ty + num2..=ty + num2 + 1 {
            if !flag {
                continue;
            }
            wx = k;
            wy = l;
            let w = world.tile(wx, wy).map(|t| t.wall).unwrap_or(0);
            if w == 0 {
                continue;
            }
            if !super::wall_house(w) {
                for m in wx - 1..wx + 2 {
                    for n in wy - 1..wy + 2 {
                        if world.tile(m, n).map(|t| t.wall).unwrap_or(0) != w {
                            flag = false;
                            break;
                        }
                    }
                }
            } else {
                flag = false;
            }
        }
    }
    (wx, wy)
}

/// The pound's `damageAmount` (`Player.cs:45988-45992`): 100, zeroed when a
/// locked golden door sits directly above or below (`WorldGen.IsLockedDoor`,
/// `:69186-69198`). The caller feeds it to `hitTile.AddDamage(tileHitId, …)`;
/// a sub-100 result is the fail tick (the Dig 0 cue) and KEEPS `poundRelease`.
pub fn pound_damage(world: &World, x: i32, y: i32) -> i32 {
    let locked = |wx: i32, wy: i32| world.tile(wx, wy).is_some_and(|t| is_locked_door(t));
    if locked(x, y - 1) || locked(x, y + 1) {
        0
    } else {
        100
    }
}

/// The outcome of [`try_pound_tile`] — what the app seam does next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoundOutcome {
    /// `Player.cs:45976-45985` — no hammer / inactive / not poundable: the
    /// caller clears `poundRelease`, nothing happens.
    Refused,
    /// `:45989-45998` — a locked golden door above/below zeroes the damage:
    /// the fail tick (the Dig 0 cue) plays and `poundRelease` is KEPT (the
    /// C# returns before the `:46310` clear).
    LockedDoor,
    /// `:46004-46007` (470 DisplayDoll `TEDisplayDoll.TryChangePose`): the
    /// latch is consumed; the port has no behaviour half (the doll's TE
    /// payload is discarded, `tile_entities.rs`) — recorded seam, ANALYSIS.
    Consumed,
    /// `:46056-46062` (314 `Minecart.FrameTrack(x, y, pound: true)` — mute
    /// FALSE): the track hammer cycle. The tables live app-side
    /// (`tr-app/src/minecart.rs::frame_track_hammer`, the M2-PHYS-7 module);
    /// the app runs the pound branch, and on the `(pound & flag4) && !mute`
    /// condition (`Minecart.cs:1163-1166`) the `KillTile(fail: true)` 314
    /// slice — the Item52 cue (`WorldGen.cs:66531-66533`) + the
    /// `SquareTileFrame` wake (`:63735`) whose case-314 `TileFrameImportant`
    /// arm refits with `FrameTrack(pound: false)` (`:85290-85294`). The latch
    /// clears either way (`:46310`); the C# bool only feeds the SP-discarded
    /// `netMode == 1` SendData (`:46058-46061`). Research-71 §3b.4.
    TrackHammer,
    /// `:46063-46238` — the frame-only arms (137 trap orientation, 424 wire
    /// pipe, 442 pressure pad): the frame write lands, NO sound plays (the
    /// C# arms carry only `SendTileSquare`).
    Framed,
    /// The slope/half-brick cycle ran (`:46008-46054`, `:46239-46309`) and
    /// at least one gated write LANDED: the app plays the Dig 0 cue (the
    /// C# sound sits INSIDE `SlopeTile`/`PoundTile` after the
    /// `CanPoundTile` gate, `WorldGen.cs:80770-80782`/`:80807-80822`).
    /// `lift` marks the flatten player-push (`WorldGen.cs:80785-80796` /
    /// `:80824-80835`): a `SlopeTile` that left slope 0 or a `PoundTile`
    /// that cleared the half brick.
    Pounded { lift: bool },
    /// The cycle arm ran but EVERY `SlopeTile`/`PoundTile` call refused at
    /// the `CanPoundTile` gate (a chest above, a boulder, …): no write, NO
    /// sound (the gate precedes the cue, `WorldGen.cs:80770`/`:80807`) —
    /// the latch still clears (`Player.cs:46310`). The platform's last
    /// step may have written `slope(0)` directly (`:46047`, ungated); the
    /// app wakes the caches either way.
    GateRefused,
}

/// `Player.ItemCheck_UseMiningTools_TryPoundingTile` (`Player.cs:45969-46311`)
/// — one hammer pound on the active cell. The caller owns the `poundRelease`
/// latch, the `hammer > 0` gate and the `ApplyItemTime` (`:45987`); the
/// `hitTile.AddDamage(100)` accumulator (`:45993`) always reaches 100 for a
/// fresh pound, so only the locked-door zero-damage arm refuses.
///
/// The `ClearMiningCacheAt` + re-check (`:45999-46003`) is dead code in the
/// C# (nothing clears `poundRelease` between) and folds away. The net
/// messages (17's subtypes 7/14/15/23 + `SendTileSquare`) have no port
/// replication kind yet — ANALYSIS.
/// `rng` threads the caller's ACTIVE `genRand` stream (W4-g83 contract §2 —
/// the pound cycle runs under the `UpdatePlayers` swap, `Main.cs:17754`).
pub fn try_pound_tile(
    world: &mut World,
    x: i32,
    y: i32,
    chest_holds_items: impl Fn(i32, i32) -> bool + Copy,
    rng: &mut UnifiedRandom,
) -> PoundOutcome {
    // `:45975-45985` — the active + poundable gates (the hammer gate is the
    // caller's, `:45976`).
    let Some(tile) = world.tile(x, y).copied() else {
        return PoundOutcome::Refused;
    };
    if !tile.active() || !is_tile_poundable(tile.kind) {
        return PoundOutcome::Refused;
    }
    // `:45988-45998` — the locked-door zero damage.
    let locked = |wx: i32, wy: i32| {
        world
            .tile(wx, wy)
            .is_some_and(|t| is_locked_door(t))
    };
    if locked(x, y - 1) || locked(x, y + 1) {
        return PoundOutcome::LockedDoor;
    }
    // The write census: `wrote` marks a LANDED `SlopeTile`/`PoundTile` (the
    // C# Dig-0 cue sits inside them AFTER the `CanPoundTile` gate,
    // `WorldGen.cs:80770-80782`/`:80807-80822` — a refused write plays
    // nothing and draws no sound RNG). `lift` marks the flatten player-push:
    // a successful `SlopeTile` that leaves slope 0 or a `PoundTile` that
    // clears the half brick lifts the players (`WorldGen.cs:80785-80796` /
    // `:80824-80835`).
    #[derive(Default)]
    struct Writes {
        wrote: bool,
        lift: bool,
    }
    let mut st = Writes::default();
    let slope = |world: &mut World, s: u8, st: &mut Writes, rng: &mut UnifiedRandom| {
        if slope_tile(world, x, y, s, false, chest_holds_items, rng) {
            st.wrote = true;
            if world.tile(x, y).is_some_and(|t| t.slope() == 0) {
                st.lift = true;
            }
        }
    };
    let pound = |world: &mut World, st: &mut Writes, rng: &mut UnifiedRandom| {
        if pound_tile(world, x, y, false, chest_holds_items, rng) {
            st.wrote = true;
            if world.tile(x, y).is_some_and(|t| !t.half_brick()) {
                st.lift = true;
            }
        }
    };
    let outcome = |st: &Writes| {
        if st.wrote {
            PoundOutcome::Pounded { lift: st.lift }
        } else {
            PoundOutcome::GateRefused
        }
    };
    if tile.kind == 470 {
        // `:46004-46007` — `TEDisplayDoll.TryChangePose` (the pose cycle,
        // `TEDisplayDoll.cs:894-921`): no ported doll entity — seam.
        return PoundOutcome::Consumed;
    }
    if td::SET_PLATFORMS.has(tile.kind) {
        // `:46008-46054` — the platform arm (the PLAYER's inline copy, not
        // `WorldGen.PoundPlatform` `:80840-80889`: the last step gates on
        // `CanPoundTile` and writes `slope(0)` directly).
        if tile.half_brick() {
            // `:46010-46017`.
            pound(world, &mut st, rng);
        } else {
            // `:46020-46026` — the diagonal flip: a platform at the
            // upper-right / lower-left diagonal, or a solid right neighbour
            // without a solid left one, swaps the cycle's direction. The
            // C# reads `.type` regardless of `active()` — the port's
            // out-of-world read falls to kind 0 (not a platform).
            let kind_at = |wx: i32, wy: i32| world.tile(wx, wy).map(|t| t.kind).unwrap_or(0);
            let (mut num, mut second) = (1u8, 2u8);
            if td::SET_PLATFORMS.has(kind_at(x + 1, y - 1))
                || td::SET_PLATFORMS.has(kind_at(x - 1, y + 1))
                || (solid_tile(world, x + 1, y) && !solid_tile(world, x - 1, y))
            {
                num = 2;
                second = 1;
            }
            let cur = world.tile(x, y).map(|t| t.slope()).unwrap_or(0);
            if cur == 0 {
                // `:46027-46035`.
                slope(world, num, &mut st, rng);
            } else if cur == num {
                // `:46036-46044`.
                slope(world, second, &mut st, rng);
            } else if can_pound_tile(world, x, y, false, chest_holds_items) {
                // `:46045-46053` — `slope(0)` + `PoundTile` (the half brick
                // the pound sets means no lift here). The direct `slope(0)`
                // write is ungated (`:46047`) — it lands even if the
                // following `PoundTile` were refused; ONE cue in the C#
                // (only `PoundTile` plays — the slope write is direct).
                let i = world.index(x, y);
                world.tiles[i].set_slope(0);
                pound(world, &mut st, rng);
            }
        }
        return outcome(&st);
    }
    if tile.kind == 314 {
        // `:46056-46062` — `Minecart.FrameTrack(x, y, pound: true)` (mute
        // false): the app seam owns the tables (`frame_track_hammer`) and the
        // fail-kill slice; research-71 §3b.4.
        return PoundOutcome::TrackHammer;
    }
    if tile.kind == 137 {
        // `:46063-46119` — the trap/dart orientation cycle.
        let mut num4 = 0i32;
        match i32::from(tile.frame_y) / 18 {
            0 | 1 | 2 | 5 => match i32::from(tile.frame_x) / 18 {
                0 => num4 = 2,
                1 => num4 = 3,
                2 => num4 = 4,
                3 => num4 = 5,
                4 => num4 = 1,
                5 => num4 = 0,
                _ => {}
            },
            3 | 4 => match i32::from(tile.frame_x) / 18 {
                0 | 1 => num4 = 3,
                3 => num4 = 2,
                2 => num4 = 4,
                4 => num4 = 0,
                _ => {}
            },
            _ => {}
        }
        let i = world.index(x, y);
        world.tiles[i].frame_x = (num4 * 18) as i16;
        touch(world, x, y);
        return PoundOutcome::Framed;
    }
    if tile.kind == 424 {
        // `:46120-46138` — the wire pipe's 0 → 18 → 36 → 0 cycle.
        let next = match tile.frame_x {
            0 => 18,
            18 => 36,
            _ => 0,
        };
        let i = world.index(x, y);
        world.tiles[i].frame_x = next;
        touch(world, x, y);
        return PoundOutcome::Framed;
    }
    if tile.kind == 442 {
        // `:46139-46238` — the projectile pressure pad's orientation census.
        framed_pressure_pad(world, x, y);
        return PoundOutcome::Framed;
    }
    if (tile.half_brick() || tile.slope() != 0) && !td::TILE_SOLID_TOP.has(tile.kind) {
        // `:46239-46301` — the four-slope cycle. `num12`/`num13` are the
        // first/second slopes, `num11` the vertical direction.
        let (mut num12, mut num13) = (1u8, 2u8);
        // `:46244-46248` — the right-side lean: a solid right neighbour OR
        // a right neighbour sloping 1/3, without a solid left one.
        let right = world.tile(x + 1, y).copied();
        if (solid_tile(world, x + 1, y)
            || right.is_some_and(|r| r.slope() == 1 || r.slope() == 3))
            && !solid_tile(world, x - 1, y)
        {
            num12 = 2;
            num13 = 1;
        }
        // `:46249-46252` — a solid ceiling without a solid floor flips the
        // cycle to the bottom slopes first.
        let num11 = if solid_tile(world, x, y - 1) && !solid_tile(world, x, y + 1) {
            -1
        } else {
            1
        };
        let cur = world.tile(x, y).map(|t| t.slope()).unwrap_or(0);
        if num11 == 1 {
            // `:46253-46275` — the down-first cycle 0 → a → b → a+2 → b+2 → 0.
            if cur == 0 {
                slope(world, num12, &mut st, rng);
            } else if cur == num12 {
                slope(world, num13, &mut st, rng);
            } else if cur == num13 {
                slope(world, num12 + 2, &mut st, rng);
            } else if cur == num12 + 2 {
                slope(world, num13 + 2, &mut st, rng);
            } else {
                slope(world, 0, &mut st, rng);
            }
        } else {
            // `:46276-46295` — the up-first cycle 0 → a+2 → b+2 → a → b → 0.
            if cur == 0 {
                slope(world, num12 + 2, &mut st, rng);
            } else if cur == num12 + 2 {
                slope(world, num13 + 2, &mut st, rng);
            } else if cur == num13 + 2 {
                slope(world, num12, &mut st, rng);
            } else if cur == num12 {
                slope(world, num13, &mut st, rng);
            } else {
                slope(world, 0, &mut st, rng);
            }
        }
        return outcome(&st);
    }
    // `:46302-46309` — the flat solid: the half-brick toggle.
    pound(world, &mut st, rng);
    outcome(&st)
}

/// The 442 pressure-pad arm of `TryPoundingTile` (`Player.cs:46139-46238`):
/// the eight-neighbour attachment census, then the orientation cycle over
/// `frameX / 22`. The neighbour kinds read `-1` for an inactive/actuated
/// (`nactive`) or missing cell — the C#'s null checks (`:46157-46188`).
fn framed_pressure_pad(world: &mut World, x: i32, y: i32) {
    use crate::tileobject::nactive;
    // `:46141-46148` — the eight neighbours.
    let at = |wx: i32, wy: i32| world.tile(wx, wy).copied();
    let (up, down, left, right) = (at(x, y - 1), at(x, y + 1), at(x - 1, y), at(x + 1, y));
    let (dl, dr, ul, ur) = (
        at(x - 1, y + 1),
        at(x + 1, y + 1),
        at(x - 1, y - 1),
        at(x + 1, y - 1),
    );
    // `:46149-46188` — the kind census (`-1` = no usable neighbour).
    let mut num5 = -1i32; // below
    let mut num6 = -1i32; // above
    let mut num7 = -1i32; // left
    let mut num8 = -1i32; // right
    let mut tree = -1i32; // down-left
    let mut tree2 = -1i32; // down-right
    let mut tree3 = -1i32; // up-left
    let mut tree4 = -1i32; // up-right
    if up.is_some_and(|t| nactive(&t) && !t.bottom_slope()) {
        num6 = i32::from(up.unwrap().kind);
    }
    if down.is_some_and(|t| nactive(&t) && !t.half_brick() && !t.top_slope()) {
        num5 = i32::from(down.unwrap().kind);
    }
    // `:46165` — the left neighbour: slope 0 or NOT odd (0/2/4).
    if left.is_some_and(|t| nactive(&t) && (t.slope() == 0 || t.slope() % 2 != 1)) {
        num7 = i32::from(left.unwrap().kind);
    }
    // `:46169` — the right neighbour: slope 0 or NOT even (0/1/3).
    if right.is_some_and(|t| nactive(&t) && (t.slope() == 0 || t.slope() % 2 != 0)) {
        num8 = i32::from(right.unwrap().kind);
    }
    if dl.is_some_and(|t| nactive(&t)) {
        tree = i32::from(dl.unwrap().kind);
    }
    if dr.is_some_and(|t| nactive(&t)) {
        tree2 = i32::from(dr.unwrap().kind);
    }
    if ul.is_some_and(|t| nactive(&t)) {
        tree3 = i32::from(ul.unwrap().kind);
    }
    if ur.is_some_and(|t| nactive(&t)) {
        tree4 = i32::from(ur.unwrap().kind);
    }
    let solid = |k: i32| k >= 0 && td::TILE_SOLID.has(k as u16);
    let no_attach = |k: i32| k >= 0 && crate::torch::NO_ATTACH.contains(&(k as u16));
    let platform = |k: i32| k >= 0 && td::SET_PLATFORMS.has(k as u16);
    let beam = |k: i32| k >= 0 && td::SET_IS_BEAM.has(k as u16);
    let tree_type = |k: i32| k >= 0 && crate::tileobject::is_a_tree_trunk(k as u16);
    // `:46189-46208` — the four attachment flags.
    let mut flag = false; // up
    let mut flag2 = false; // left
    let mut flag3 = false; // right
    let mut flag4 = false; // down
    if solid(num5)
        && (!no_attach(num5) || platform(num5))
        && down.is_some_and(|t| (t.bottom_slope() || t.slope() == 0) && !t.half_brick())
    {
        flag4 = true;
    }
    if solid(num6)
        && (!no_attach(num6) || (platform(num6) && up.is_some_and(|t| t.half_brick())))
        && up.is_some_and(|t| t.top_slope() || t.slope() == 0 || t.half_brick())
    {
        flag = true;
    }
    if (solid(num7)
        && !no_attach(num7)
        && left.is_some_and(|t| (t.slope() == 2 || t.slope() == 4 || t.slope() == 0) && !t.half_brick()))
        || beam(num7)
        || (tree_type(num7) && tree_type(tree3) && tree_type(tree))
    {
        flag2 = true;
    }
    if (solid(num8)
        && !no_attach(num8)
        && right.is_some_and(|t| (t.slope() == 1 || t.slope() == 3 || t.slope() == 0) && !t.half_brick()))
        || beam(num8)
        || (tree_type(num8) && tree_type(tree4) && tree_type(tree2))
    {
        flag3 = true;
    }
    // `:46209-46237` — the orientation cycle. `-1` keeps the frame, `-2`
    // falls back to 0.
    let here = world.tile(x, y).copied().unwrap_or_default();
    let num9 = i32::from(here.frame_x) / 22;
    let mut num10: i32 = -2;
    match num9 {
        0 => num10 = if !flag2 { if flag { 1 } else if !flag3 { -1 } else { 3 } } else { 2 },
        2 => num10 = if flag { 1 } else if !flag3 { if !flag4 { -1 } else { 0 } } else { 3 },
        1 => num10 = if !flag3 { if !flag4 { if !flag2 { -1 } else { 2 } } else { 0 } } else { 3 },
        3 => num10 = if !flag4 { if !flag2 { if flag { 1 } else { -1 } } else { 2 } } else { 0 },
        _ => {}
    }
    if num10 != -1 {
        if num10 == -2 {
            num10 = 0;
        }
        let i = world.index(x, y);
        world.tiles[i].frame_x = (22 * num10) as i16;
        touch(world, x, y);
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;
    // Gate merge: the one `can_player_smash_wall` lives in `edit::wall`.
    use crate::edit::can_player_smash_wall;
    use crate::edit::tests::{ground_world, set_active};
    use crate::tile::FLAG_ACTIVE;

    fn no_chest(_: i32, _: i32) -> bool {
        false
    }

    // W4-g83: the pound family threads the caller's ACTIVE genRand stream
    // (the reframe's framing dispatch). The witnesses pin slopes/frames, not
    // streams — a fresh seed-0 stream per call.
    fn rng() -> UnifiedRandom {
        UnifiedRandom::new(0)
    }

    /// A lone active stone at (10, 15) in the 32×32 ground fixture's air.
    fn lone(kind: u16) -> World {
        let mut w = ground_world();
        set_active(&mut w, 10, 15, kind);
        w
    }

    // ---- CanPoundTile / ForbidsSloping (WorldGen.cs:80695-80766) ----------

    #[test]
    fn s1_p_i31_can_pound_refuses_the_mechanism_switch() {
        // `:80711-80720` — every listed kind refuses.
        for kind in [10u16, 48, 137, 232, 380, 387, 388, 476, 484] {
            let w = lone(kind);
            assert!(!can_pound_tile(&w, 10, 15, false, no_chest), "kind {kind}");
        }
        // A plain stone passes (`:80741` CanKillTile).
        let w = lone(1);
        assert!(can_pound_tile(&w, 10, 15, false, no_chest));
    }

    #[test]
    fn s1_p_i31_can_pound_refuses_boulders_and_generation_kinds() {
        // `:80722-80725` — SET_BOULDERS (`TileID.cs:504`): 138, 484, 664,
        // 665, 711-716.
        let w = lone(138);
        assert!(!can_pound_tile(&w, 10, 15, false, no_chest));
        // `:80726-80736` — 190/30 refuse ONLY while generating.
        for kind in [190u16, 30] {
            let w = lone(kind);
            assert!(!can_pound_tile(&w, 10, 15, true, no_chest), "gen {kind}");
            assert!(can_pound_tile(&w, 10, 15, false, no_chest), "play {kind}");
        }
    }

    #[test]
    fn s1_p_i31_forbids_sloping_above_blocks_the_pound() {
        // `:80737-80740` + `:80745-80766` — an active chest above refuses;
        // an INACTIVE one does not (the `active()` gate).
        let mut w = lone(1);
        set_active(&mut w, 10, 14, 21);
        assert!(!can_pound_tile(&w, 10, 15, false, no_chest));
        let i = w.index(10, 14);
        w.tiles[i].flags &= !FLAG_ACTIVE;
        assert!(can_pound_tile(&w, 10, 15, false, no_chest));
        // The full ForbidsSloping table.
        for kind in [21u16, 26, 77, 88, 235, 237, 441, 467, 468, 470, 475, 488, 597] {
            assert!(forbids_sloping(kind), "kind {kind}");
        }
        assert!(!forbids_sloping(1));
    }

    // ---- SlopeTile / PoundTile (WorldGen.cs:80768-80838) ------------------

    #[test]
    fn s1_p_i31_slope_tile_clears_the_half_brick_and_sets_the_slope() {
        let mut w = lone(1);
        let i = w.index(10, 15);
        w.tiles[i].flags |= crate::tile::FLAG_HALF_BRICK;
        assert!(slope_tile(&mut w, 10, 15, 3, false, no_chest, &mut rng()));
        let t = w.tile(10, 15).unwrap();
        assert!(!t.half_brick());
        assert_eq!(t.slope(), 3);
        // The gate refuses a trap (`:80770`).
        let mut w2 = lone(137);
        assert!(!slope_tile(&mut w2, 10, 15, 1, false, no_chest, &mut rng()));
        assert_eq!(w2.tile(10, 15).unwrap().slope(), 0);
    }

    #[test]
    fn s1_p_i31_pound_tile_toggles_the_half_brick() {
        let mut w = lone(1);
        assert!(pound_tile(&mut w, 10, 15, false, no_chest, &mut rng()));
        assert!(w.tile(10, 15).unwrap().half_brick());
        assert!(pound_tile(&mut w, 10, 15, false, no_chest, &mut rng()));
        assert!(!w.tile(10, 15).unwrap().half_brick());
    }

    // ---- the target predicates (Player.cs:45536-45556) --------------------

    #[test]
    fn s1_p_i31_poundable_and_hammer_target_tables() {
        // Solids pound; the five special non-solids pound; living fire
        // pounds; a torch (non-solid, not special) does not; tileHammer
        // kinds never pound but ARE hammer targets.
        for kind in [1u16, 19, 314, 424, 442, 351, 470, 336, 340, 341, 342, 343, 344] {
            assert!(is_tile_poundable(kind), "kind {kind} poundable");
            assert!(is_a_hammer_target(kind), "kind {kind} target");
        }
        assert!(!is_tile_poundable(4)); // torch
        assert!(!is_a_hammer_target(4));
        for kind in [26u16, 695, 31, 696] {
            assert!(!is_tile_poundable(kind), "tileHammer {kind}");
            assert!(is_a_hammer_target(kind), "tileHammer {kind} target");
        }
    }

    // ---- CanPlayerSmashWall (Player.cs:45917-45940) -----------------------

    #[test]
    fn s1_p_i31_smash_wall_house_edge_interior_and_350() {
        let mut w = ground_world();
        // A 5×5 patch of unsafe dirt wall (2) in the air rows.
        for x in 8..13 {
            for y in 12..17 {
                let i = w.index(x, y);
                w.tiles[i].wall = 2;
            }
        }
        // The interior cell refuses (`:45927-45937` — no open/house neighbour).
        assert!(!can_player_smash_wall(&w, 10, 14));
        // The edge cell breaks (its outside neighbour is wall 0).
        assert!(can_player_smash_wall(&w, 8, 14));
        // A house wall always breaks (`:45924`).
        let i = w.index(10, 14);
        w.tiles[i].wall = 24;
        assert!(can_player_smash_wall(&w, 10, 14));
        // Wall 350 never breaks (`:45919-45922`).
        w.tiles[i].wall = 350;
        assert!(!can_player_smash_wall(&w, 10, 14));
    }

    // ---- TryFindingWallToHammer (Player.cs:45832-45906) -------------------

    #[test]
    fn s1_p_i31_wall_finder_keeps_house_and_edge_cells() {
        let mut w = ground_world();
        let i = w.index(10, 14);
        w.tiles[i].wall = 24; // house wall → flag false → keep the cursor
        assert_eq!(
            try_finding_wall_to_hammer(&w, 10, 14, (10.0 * 16.0 + 8.0, 14.0 * 16.0 + 8.0)),
            (10, 14)
        );
        // An edge unsafe wall (a wall-0 neighbour) also keeps the cursor.
        w.tiles[i].wall = 2;
        assert_eq!(
            try_finding_wall_to_hammer(&w, 10, 14, (10.0 * 16.0 + 8.0, 14.0 * 16.0 + 8.0)),
            (10, 14)
        );
        // An ACTIVE tile keeps the cursor regardless (`:45858`).
        set_active(&mut w, 10, 14, 1);
        assert_eq!(
            try_finding_wall_to_hammer(&w, 10, 14, (10.0 * 16.0 + 8.0, 14.0 * 16.0 + 8.0)),
            (10, 14)
        );
    }

    #[test]
    fn s1_p_i31_wall_finder_scans_the_bankers_quadrant() {
        // An interior unsafe patch; the cursor sits on its interior at
        // (10, 14). The mouse fraction .5 rounds BANKER'S: tile 10 is even
        // → round(10.5) = 10 → 10.5 < 10 is false → num = -1 → the scan
        // covers columns 9..10; the edge column 8 is NOT in the scan, so
        // the last visited cell (10, 14) wins (the C# overwrite quirk).
        let mut w = ground_world();
        for x in 8..13 {
            for y in 12..17 {
                let i = w.index(x, y);
                w.tiles[i].wall = 2;
            }
        }
        let mouse = (10.0 * 16.0 + 8.0, 14.0 * 16.0 + 8.0); // (10.5, 14.5)
        assert_eq!(try_finding_wall_to_hammer(&w, 10, 14, mouse), (10, 14));
        // Tile 11 is odd → round(11.5) = 12 → 11.5 < 12 → num = 0 → the
        // scan covers columns 11..12; at (12, 13) the 3×3 reaches column 13
        // (wall 0) → flag falls, and (12, 13) is the LAST assigned cell
        // (the C# overwrites wX/wY before the wall check, `:45880-45881`).
        let mouse = (11.0 * 16.0 + 8.0, 14.0 * 16.0 + 8.0);
        assert_eq!(try_finding_wall_to_hammer(&w, 11, 14, mouse), (12, 13));
    }

    // ---- the pound cycle (Player.cs:45969-46311) --------------------------

    #[test]
    fn s1_p_i31_the_flat_block_cycles_half_then_four_slopes() {
        // The isolated stone: flat → half → slope 1 → 2 → 3 → 4 → flat.
        let mut w = lone(1);
        let steps: Vec<(bool, u8)> = vec![
            (true, 0),  // half brick
            (false, 1), // the cycle arm clears the half, slope 1
            (false, 2),
            (false, 3),
            (false, 4),
            (false, 0), // SlopeTile(0) — the flatten (lift)
            (true, 0),  // half again
        ];
        for (half, slope) in steps {
            let out = try_pound_tile(&mut w, 10, 15, no_chest, &mut rng());
            assert!(matches!(out, PoundOutcome::Pounded { .. }), "{out:?}");
            let t = w.tile(10, 15).unwrap();
            assert_eq!(t.half_brick(), half, "half at slope {slope}");
            assert_eq!(t.slope(), slope);
        }
    }

    #[test]
    fn s1_p_i31_the_right_lean_flips_the_cycle_direction() {
        // `:46244-46248` — a solid right neighbour without a solid left one
        // swaps num12/num13: the cycle runs 2 → 1 → 4 → 3.
        let mut w = lone(1);
        set_active(&mut w, 11, 15, 1);
        let i = w.index(10, 15);
        w.tiles[i].flags |= crate::tile::FLAG_HALF_BRICK;
        let mut seen = Vec::new();
        for _ in 0..4 {
            try_pound_tile(&mut w, 10, 15, no_chest, &mut rng());
            seen.push(w.tile(10, 15).unwrap().slope());
        }
        assert_eq!(seen, vec![2, 1, 4, 3]);
    }

    #[test]
    fn s1_p_i31_the_ceiling_flips_the_cycle_to_bottom_slopes() {
        // `:46249-46252` — solid above, no solid below → num11 = -1: the
        // cycle runs 3 → 4 → 1 → 2.
        let mut w = lone(1);
        set_active(&mut w, 10, 14, 1);
        let i = w.index(10, 15);
        w.tiles[i].flags |= crate::tile::FLAG_HALF_BRICK;
        let mut seen = Vec::new();
        for _ in 0..4 {
            try_pound_tile(&mut w, 10, 15, no_chest, &mut rng());
            seen.push(w.tile(10, 15).unwrap().slope());
        }
        assert_eq!(seen, vec![3, 4, 1, 2]);
    }

    #[test]
    fn s1_p_i31_the_platform_cycles_slopes_then_half() {
        // `:46008-46054` — the isolated platform: slope 1 → 2 → half → flat.
        let mut w = lone(19);
        try_pound_tile(&mut w, 10, 15, no_chest, &mut rng());
        assert_eq!(w.tile(10, 15).unwrap().slope(), 1);
        try_pound_tile(&mut w, 10, 15, no_chest, &mut rng());
        assert_eq!(w.tile(10, 15).unwrap().slope(), 2);
        try_pound_tile(&mut w, 10, 15, no_chest, &mut rng());
        let t = w.tile(10, 15).unwrap();
        assert!(t.half_brick() && t.slope() == 0, "the CanPoundTile arm");
        try_pound_tile(&mut w, 10, 15, no_chest, &mut rng());
        assert!(!w.tile(10, 15).unwrap().half_brick());
    }

    #[test]
    fn s1_p_i31_the_platform_diagonal_flips_its_cycle() {
        // `:46022-46026` — a platform at the lower-left diagonal swaps the
        // direction: slope 2 first. The diagonal reads `.type` even when
        // INACTIVE (the C# reads the field, not nactive).
        let mut w = lone(19);
        let i = w.index(9, 16);
        w.tiles[i].kind = 19; // inactive platform cell
        try_pound_tile(&mut w, 10, 15, no_chest, &mut rng());
        assert_eq!(w.tile(10, 15).unwrap().slope(), 2);
        try_pound_tile(&mut w, 10, 15, no_chest, &mut rng());
        assert_eq!(w.tile(10, 15).unwrap().slope(), 1);
    }

    #[test]
    fn s1_p_i31_the_locked_door_refuses_and_keeps_the_latch() {
        // `:45989-45998` — the golden door's locked frames above.
        let mut w = lone(1);
        set_active(&mut w, 10, 14, 10);
        let i = w.index(10, 14);
        w.tiles[i].frame_y = 594;
        w.tiles[i].frame_x = 0;
        assert_eq!(try_pound_tile(&mut w, 10, 15, no_chest, &mut rng()), PoundOutcome::LockedDoor);
        let t = w.tile(10, 15).unwrap();
        assert!(!t.half_brick() && t.slope() == 0, "nothing changed");
    }

    #[test]
    fn s1_p_i31_the_wire_pipe_cycles_0_18_36() {
        // `:46120-46138`.
        let mut w = lone(424);
        for want in [18i16, 36, 0, 18] {
            assert_eq!(try_pound_tile(&mut w, 10, 15, no_chest, &mut rng()), PoundOutcome::Framed);
            assert_eq!(w.tile(10, 15).unwrap().frame_x, want);
        }
    }

    #[test]
    fn s1_p_i31_the_trap_orientation_tables() {
        // `:46063-46119` — frameY row 0: frameX/18 0→2→4→1→3→5→0;
        // frameY row 3 (frameY 54): 0→3→2→4→0.
        let mut w = lone(137);
        // 137 is in CanPoundTile's refusal switch — but TryPoundingTile's
        // 137 arm runs BEFORE any SlopeTile/PoundTile call, so the frame
        // cycle still lands (the C# never calls CanPoundTile on this arm).
        let mut seen = Vec::new();
        for _ in 0..6 {
            assert_eq!(try_pound_tile(&mut w, 10, 15, no_chest, &mut rng()), PoundOutcome::Framed);
            seen.push(i32::from(w.tile(10, 15).unwrap().frame_x) / 18);
        }
        assert_eq!(seen, vec![2, 4, 1, 3, 5, 0]);
        let i = w.index(10, 15);
        w.tiles[i].frame_y = 54;
        w.tiles[i].frame_x = 0;
        let mut seen = Vec::new();
        for _ in 0..4 {
            try_pound_tile(&mut w, 10, 15, no_chest, &mut rng());
            seen.push(i32::from(w.tile(10, 15).unwrap().frame_x) / 18);
        }
        assert_eq!(seen, vec![3, 2, 4, 0]);
    }

    #[test]
    fn s1_p_i31_the_pressure_pad_finds_its_support() {
        // `:46139-46238` — a pad at frameX 0 with a solid floor below:
        // flag4 true, the others false → case 0 → num10 = -1 → the frame
        // KEEPS (the pad already faces its only support).
        let mut w = lone(442);
        set_active(&mut w, 10, 16, 1);
        try_pound_tile(&mut w, 10, 15, no_chest, &mut rng());
        assert_eq!(w.tile(10, 15).unwrap().frame_x, 0);
        // Add a solid ceiling: case 0 with flag true → num10 = 1 → frameX 22.
        set_active(&mut w, 10, 14, 1);
        try_pound_tile(&mut w, 10, 15, no_chest, &mut rng());
        assert_eq!(w.tile(10, 15).unwrap().frame_x, 22);
        // Remove every support: case 1 (frameX 22) with all flags false →
        // num10 = -1 → the frame keeps.
        let i = w.index(10, 16);
        w.tiles[i].flags &= !FLAG_ACTIVE;
        let i = w.index(10, 14);
        w.tiles[i].flags &= !FLAG_ACTIVE;
        try_pound_tile(&mut w, 10, 15, no_chest, &mut rng());
        assert_eq!(w.tile(10, 15).unwrap().frame_x, 22);
    }

    #[test]
    fn s1_p_i31_the_special_seams_consume_without_change() {
        // 470 — the Consumed seam (`:46004-46007`, the doll's TE payload is
        // unported). 314 rides its own TrackHammer seam now (research-71
        // §3b.4 — the app owns `frame_track_hammer`); the core hands it off
        // WITHOUT touching the frames.
        let mut w = lone(470);
        assert_eq!(try_pound_tile(&mut w, 10, 15, no_chest, &mut rng()), PoundOutcome::Consumed);
        let mut w = lone(314);
        assert_eq!(try_pound_tile(&mut w, 10, 15, no_chest, &mut rng()), PoundOutcome::TrackHammer);
        // A tileHammer kind refuses the pound (`:45981-45985`).
        let mut w = lone(26);
        assert_eq!(try_pound_tile(&mut w, 10, 15, no_chest, &mut rng()), PoundOutcome::Refused);
    }

    #[test]
    fn s1_p_i31_the_gate_refused_pound_writes_and_sounds_nothing() {
        // A chest ABOVE the stone refuses every SlopeTile/PoundTile at the
        // CanPoundTile gate (`WorldGen.cs:80737-80740`): the cycle arm runs
        // but nothing lands — GateRefused (no Dig-0 cue, no sound RNG; the
        // latch still clears app-side, `Player.cs:46310`).
        let mut w = lone(1);
        set_active(&mut w, 10, 14, 21);
        assert_eq!(try_pound_tile(&mut w, 10, 15, no_chest, &mut rng()), PoundOutcome::GateRefused);
        let t = w.tile(10, 15).unwrap();
        assert!(!t.half_brick() && t.slope() == 0, "nothing changed");
        // The platform cycle refuses the same way (its first SlopeTile).
        let mut w = lone(19);
        set_active(&mut w, 10, 14, 21);
        assert_eq!(try_pound_tile(&mut w, 10, 15, no_chest, &mut rng()), PoundOutcome::GateRefused);
        assert_eq!(w.tile(10, 15).unwrap().slope(), 0);
    }

    #[test]
    fn s1_p_i31_the_flatten_lift_flag() {
        // The half → flat pound lifts; the flat → half pound does not
        // (WorldGen.cs:80824-80835); the cycle's final SlopeTile(0) lifts.
        let mut w = lone(1);
        assert_eq!(
            try_pound_tile(&mut w, 10, 15, no_chest, &mut rng()),
            PoundOutcome::Pounded { lift: false }
        );
        assert_eq!(
            try_pound_tile(&mut w, 10, 15, no_chest, &mut rng()),
            PoundOutcome::Pounded { lift: false } // half → slope 1, no flatten
        );
        for _ in 0..3 {
            try_pound_tile(&mut w, 10, 15, no_chest, &mut rng()); // slopes 2, 3, 4
        }
        assert_eq!(
            try_pound_tile(&mut w, 10, 15, no_chest, &mut rng()),
            PoundOutcome::Pounded { lift: true } // slope 4 → 0
        );
    }

    // ---- Tile.SmoothSlope (Tile.cs:792-865, W4-d8 retire (1)) -------------

    /// The five mask arms + the default re-solidify (`Tile.cs:814-845`).
    /// The lone stone at (10,15) sits in air: mask 0 → slope 0, no half.
    /// Each neighbour added flips one mask bit; the arms are read at the
    /// single-cell seam so the neighbour cascade cannot mask them.
    #[test]
    fn w4_d8_smooth_slope_maps_the_five_mask_arms() {
        // mask 4 (below only): the half brick (`:838-841`).
        let mut w = lone(1);
        set_active(&mut w, 10, 16, 1);
        smooth_slope_one(&mut w, 10, 15, false, no_chest);
        let t = w.tile(10, 15).unwrap();
        assert!(t.half_brick() && t.slope() == 0, "mask 4 → half");
        // mask 5 (below + right): slope 2 (`:834-837`).
        set_active(&mut w, 11, 15, 1);
        smooth_slope_one(&mut w, 10, 15, false, no_chest);
        let t = w.tile(10, 15).unwrap();
        assert!(!t.half_brick() && t.slope() == 2, "mask 5 → slope 2");
        // mask 6 (below + left): slope 1 (`:830-833`).
        let mut w = lone(1);
        set_active(&mut w, 10, 16, 1);
        set_active(&mut w, 9, 15, 1);
        smooth_slope_one(&mut w, 10, 15, false, no_chest);
        assert_eq!(w.tile(10, 15).unwrap().slope(), 1, "mask 6 → slope 1");
        // mask 9 (above + right): slope 4 (`:823-829`).
        let mut w = lone(1);
        set_active(&mut w, 10, 14, 1);
        set_active(&mut w, 11, 15, 1);
        smooth_slope_one(&mut w, 10, 15, false, no_chest);
        assert_eq!(w.tile(10, 15).unwrap().slope(), 4, "mask 9 → slope 4");
        // mask 10 (above + left): slope 3 (`:816-822`).
        let mut w = lone(1);
        set_active(&mut w, 10, 14, 1);
        set_active(&mut w, 9, 15, 1);
        smooth_slope_one(&mut w, 10, 15, false, no_chest);
        assert_eq!(w.tile(10, 15).unwrap().slope(), 3, "mask 10 → slope 3");
        // mask 0 (lone): the default re-solidify (`:842-845`) — a
        // pre-sloped/half cell flattens.
        let mut w = lone(1);
        let i = w.index(10, 15);
        w.tiles[i].set_slope(2);
        w.tiles[i].flags |= crate::tile::FLAG_HALF_BRICK;
        smooth_slope_one(&mut w, 10, 15, false, no_chest);
        let t = w.tile(10, 15).unwrap();
        assert!(!t.half_brick() && t.slope() == 0, "the default flattens");
    }

    /// The flag2 veto (`:807`/`:817`/`:824`): a NON-solid but non-empty
    /// tile above (a torch, kind 4) sets `flag` (bit 3) without
    /// `SolidOrSlopedTile` — masks 9/10 break WITHOUT a write, so a
    /// pre-existing slope under the torch survives (the default arm does
    /// NOT run). A torch above with no side neighbour is mask 8 → the
    /// default DOES flatten.
    #[test]
    fn w4_d8_smooth_slope_flag2_vetoes_the_top_arms_without_writing() {
        // mask 10 with a torch above: slope 3 pre-set survives.
        let mut w = lone(1);
        set_active(&mut w, 10, 14, 4); // torch: active, not solid
        set_active(&mut w, 9, 15, 1);
        let i = w.index(10, 15);
        w.tiles[i].set_slope(3);
        smooth_slope_one(&mut w, 10, 15, false, no_chest);
        assert_eq!(w.tile(10, 15).unwrap().slope(), 3, "the veto keeps it");
        // mask 9 with the torch: slope 4 survives the same way.
        let mut w = lone(1);
        set_active(&mut w, 10, 14, 4);
        set_active(&mut w, 11, 15, 1);
        let i = w.index(10, 15);
        w.tiles[i].set_slope(4);
        smooth_slope_one(&mut w, 10, 15, false, no_chest);
        assert_eq!(w.tile(10, 15).unwrap().slope(), 4, "the veto keeps it");
        // mask 8 (torch above, no sides): the default flattens.
        let mut w = lone(1);
        set_active(&mut w, 10, 14, 4);
        let i = w.index(10, 15);
        w.tiles[i].set_slope(2);
        smooth_slope_one(&mut w, 10, 15, false, no_chest);
        assert_eq!(w.tile(10, 15).unwrap().slope(), 0, "mask 8 → default");
        // An ACTUATED-OFF solid above reads TileEmpty (`:69976-69983`):
        // flag false → bit 3 clear → mask 2 (left only) → default flatten.
        let mut w = lone(1);
        set_active(&mut w, 10, 14, 1);
        let i = w.index(10, 14);
        w.tiles[i].flags |= crate::tile::FLAG_INACTIVE;
        set_active(&mut w, 9, 15, 1);
        let i = w.index(10, 15);
        w.tiles[i].set_slope(3);
        smooth_slope_one(&mut w, 10, 15, false, no_chest);
        assert_eq!(w.tile(10, 15).unwrap().slope(), 0, "actuated above is empty");
    }

    /// The gate (`:802-805`) + the neighbour cascade order (`:794-800`):
    /// a chest above refuses the whole cell; `apply_to_neighbors: true`
    /// smooths right/left/below/above FIRST (each single-cell), then the
    /// cell — the drill's post-pick call shape (`Mount.cs:3119`).
    #[test]
    fn w4_d8_smooth_slope_gates_and_cascades() {
        // The pound gate: a chest above → no write.
        let mut w = lone(1);
        set_active(&mut w, 10, 14, 21);
        let i = w.index(10, 15);
        w.tiles[i].set_slope(2);
        smooth_slope(&mut w, 10, 15, true, false, no_chest);
        assert_eq!(w.tile(10, 15).unwrap().slope(), 2, "the gate refuses");
        // A solid-top platform (19) is not SolidOrSlopedTile → refused.
        let mut w = lone(19);
        smooth_slope_one(&mut w, 10, 15, false, no_chest);
        assert_eq!(w.tile(10, 15).unwrap().slope(), 0);
        // The cascade: a 3-wide floor row at y=16 with the picked cell's
        // hole at (10,15) filled — after the smooth the row's inner cells
        // carry slope states derived from their own neighbours.
        let mut w = ground_world(); // solid from row 20
        set_active(&mut w, 10, 19, 1); // one stone above the floor
        smooth_slope(&mut w, 10, 19, true, false, no_chest);
        // (10,19): below (10,20) solid, left/right air, above air → mask 4
        // → the half brick.
        let t = w.tile(10, 19).unwrap();
        assert!(t.half_brick() && t.slope() == 0, "the picked cell halves");
        // The below neighbour (10,20) was smoothed FIRST: above now holds
        // the (pre-smooth, still full) stone, left/right/below solid →
        // mask 15 → default → stays full.
        let t = w.tile(10, 20).unwrap();
        assert!(!t.half_brick() && t.slope() == 0, "the buried cell stays full");
    }
}
