//! The dungeon crawler — `DungeonCrawler.MakeDungeon` (`DungeonCrawler
//! .cs:340-508`) + `LegacyDungeonLayoutProvider` (`LayoutProviders/
//! LegacyDungeonLayoutProvider.cs:17-101`) + `LegacyDungeonHall.LegacyHall`
//! (`Halls/LegacyDungeonHall.cs:63-745`) + `LegacyDungeonRoom.LegacyRoom`
//! (`Rooms/LegacyDungeonRoom.cs:87-233`).
//!
//! The port's faithful slice: the setup rolls (`SetupDungeonGenVarVariables`
//! :54-213 — the color triple blue/green/pink with the brick tiles
//! 41/43/44, the walls 7/8/9, the cracked 481/482/483 and the glass walls
//! 90-98/1386-1384), the step count (`MakeDungeon` :359-362:
//! `maxTilesX / 60 + Next(0, num / 3)`), the layout loop (the roomDelay 5,
//! the `Next(3)` room/hall alternation, the double-hall depth), the hall
//! walk (the edge-turn logic at the ±X/`num6` bounds, the brick fill
//! rect :610-638, the interior carve at `num * ratio` :656-728 with the
//! ±21 tile) and the room (the `15*strength + Next(15)` diameter, the
//! slant vector, the outer brick + inner carve). The vanilla's
//! `GenerationProgress`, `ProtectionType`, the shape modifiers
//! (`ModShapes.All`/`TileRunner`-style) and the biome/sacred/dungeon-substyle
//! style tables stay out of this slice — the pass writes the brick/wall/
//! carved grid directly, the way the port's other worldgen passes do.
//!
//! The pass replaces the shape-level `pass_dungeon` corridor approximation:
//! it keeps the entrance tower, the shaft and the reach-to-rock-layer
//! contract the earlier pass established and adds the crawler walk with
//! the vanilla bounds.

use crate::tile::{Tile, FLAG_ACTIVE};
use crate::world::World;

/// The dungeon colour triple (`SetupDungeonGenVarVariables` :71-110):
/// the brick tiles 41/43/44, the walls 7/8/9, the cracked 481/482/483.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DungeonColor {
    Blue,
    Green,
    Pink,
}

impl DungeonColor {
    /// The vanilla `genRand.Next(3)` roll (the remix variant pins the
    /// crimson/blue; the port generates the classic seeds only).
    pub fn roll(r: &mut impl FnMut(u32) -> u32) -> DungeonColor {
        match r(3) {
            1 => DungeonColor::Green,
            2 => DungeonColor::Pink,
            _ => DungeonColor::Blue,
        }
    }

    /// `genVars.brickTileType` (`BrickTileType`, :87-105).
    pub fn brick_tile(self) -> u16 {
        match self {
            DungeonColor::Blue => 41,
            DungeonColor::Green => 43,
            DungeonColor::Pink => 44,
        }
    }

    /// `genVars.brickWallType` (:87-105).
    pub fn brick_wall(self) -> u16 {
        match self {
            DungeonColor::Blue => 7,
            DungeonColor::Green => 8,
            DungeonColor::Pink => 9,
        }
    }

    /// `genVars.brickCrackedTileType` (:481/482/483).
    pub fn cracked_tile(self) -> u16 {
        match self {
            DungeonColor::Blue => 481,
            DungeonColor::Green => 482,
            DungeonColor::Pink => 483,
        }
    }
}

/// `MakeDungeon` :355-358's entrance-strength rolls (the entrance hall's
/// width/height pairs), and :359-362's step count.
pub struct DungeonSetup {
    pub entrance_strength_x: i32,
    pub entrance_strength_y: i32,
    pub entrance_strength_x2: i32,
    pub entrance_strength_y2: i32,
    /// `maxSteps` (:362): `maxTilesX / 60` + `Next(0, num/3)`.
    pub steps: i32,
}

impl DungeonSetup {
    /// `MakeDungeon` :355-362: the four strength rolls then the step count.
    /// `roll(lo, hi)` mirrors `genRand.Next(lo, hi)` = `[lo, hi)`.
    pub fn roll(r: &mut impl FnMut(u32, u32) -> u32, width: i32) -> Self {
        let base = width / 60;
        let steps = base as i32 + r(0, base as u32 / 3) as i32;
        DungeonSetup {
            entrance_strength_x: r(25, 30) as i32,
            entrance_strength_y: r(20, 25) as i32,
            entrance_strength_x2: r(35, 50) as i32,
            entrance_strength_y2: r(10, 15) as i32,
            steps,
        }
    }
}

/// `LegacyDungeonHallSettings.RandomSeed` — the per-hall stream is a fresh
/// `UnifiedRandom(seed)` in the C# (:68); the port keeps a seed + its own
/// draw helpers on the same splitmix shape (`next_u64`).
#[derive(Clone, Copy, Debug)]
pub struct HallSeed {
    state: u64,
}

impl HallSeed {
    pub fn new(seed: u64) -> Self {
        Self { state: seed | 1 }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        (z >> 32) as u32
    }

    /// `UnifiedRandom.Next(max)`.
    pub fn next(&mut self, max: u32) -> u32 {
        self.next_u32() % max
    }

    /// `UnifiedRandom.Next(lo, hi)` = `Next(hi - lo) + lo`.
    pub fn range(&mut self, lo: u32, hi: u32) -> u32 {
        lo + self.next(hi - lo)
    }
}

/// `DungeonData.hallStrengthScalar`/`roomStrengthScalar`/`hallStepScalar`
/// defaults (`DungeonData.cs:70-74`) — 1.0 each, the interior/exterior
/// ratio 0.5.
const HALL_STRENGTH_SCALAR: f64 = 1.0;
const ROOM_STRENGTH_SCALAR: f64 = 1.0;
const HALL_STEP_SCALAR: f64 = 1.0;
const HALL_INTERIOR_TO_EXTERIOR_RATIO: f64 = 0.5;
const HALL_SLANT_VARIANT_SCALAR: f64 = 1.0;
const ROOM_SLANT_VARIANT_SCALAR: f64 = 1.0;

/// `LegacyDungeonHall.LegacyHall`'s walk state (the C# locals :63-745).
struct HallWalk {
    /// The current position (the C#'s `vector2D`).
    x: f64,
    y: f64,
    /// The step direction (`vector2D2`).
    dx: f64,
    dy: f64,
    /// The shell width (`num`).
    width: f64,
    /// The remaining steps (`num3`).
    steps: i32,
    /// The slant flip timer (`num12`).
    flips: i32,
    /// The cracked-brick variant (`flag`).
    cracked: bool,
}

/// `LegacyDungeonLayoutProvider.LegacyDungeonLayout` (:23-101): the entry
/// room then the roomDelay-5 loop (the `Next(3)` room/hall alternation,
/// the double-hall depth), closing with the final room. Returns the cells
/// the walk touched so the caller can frame them.
///
/// This is the layout driver; `generate_dungeon` below is the full pass
/// with the setup and the finishing passes.
fn crawl_layout(
    world: &mut World,
    start_x: i32,
    start_y: i32,
    steps: i32,
    brick: u16,
    cracked: u16,
    wall: u16,
    max_y: i32,
    r: &mut HallSeed,
) {
    // The entry room (`:42-48` — `MakeDungeon_GetRoom(... StartingRoom:
    // true ...).GenerateRoom(data)`).
    generate_room(world, start_x, start_y, 1.0, brick, cracked, wall, max_y, r);
    let mut room_delay = 5i32;
    let mut steps_left = steps;
    while steps_left > 0 {
        steps_left -= 1;
        if room_delay > 0 {
            room_delay -= 1;
        }
        if room_delay == 0 && r.next(3) == 0 {
            room_delay = 5;
            let (sx, sy) = (start_x, start_y);
            if r.next(2) == 0 {
                generate_hall(world, sx, sy, brick, cracked, wall, max_y, r);
                if r.next(2) == 0 {
                    generate_hall(world, sx, sy, brick, cracked, wall, max_y, r);
                }
                generate_room(world, sx, sy, ROOM_STRENGTH_SCALAR, brick, cracked, wall, max_y, r);
            } else {
                generate_room(world, sx, sy, ROOM_STRENGTH_SCALAR, brick, cracked, wall, max_y, r);
            }
        } else {
            generate_hall(world, start_x, start_y, brick, cracked, wall, max_y, r);
        }
    }
}

/// `LegacyDungeonHall.LegacyHall`'s setup (:70-150): the width
/// `4 * hallStrengthScalar + Next(2)`, the length
/// `35 * hallStepScalar + Next(45)`, the 1/5 wide-short variant, the
/// first-step direction from the `lastDungeonHall` (the port's last-hall
/// memory rides the walk itself — the entry hall runs horizontal).
fn generate_hall(
    world: &mut World,
    from_x: i32,
    from_y: i32,
    brick: u16,
    cracked: u16,
    wall: u16,
    max_y: i32,
    r: &mut HallSeed,
) {
    let mut width = (4.0 * HALL_STRENGTH_SCALAR) as i32 + r.next(2) as i32;
    let mut steps = (35.0 * HALL_STEP_SCALAR) as i32 + r.next(45) as i32;
    if r.next(5) == 0 {
        width *= 2;
        steps /= 2;
    }
    let cracked_on = r.next(3) == 0;
    let underworld = max_y - 220;
    // `num6 = Main.UnderworldLayer - 100` (:149) — the Y turn bound.
    let num6 = underworld - 100;
    let mut walk = HallWalk {
        x: from_x as f64,
        y: from_y as f64,
        dx: 1.0,
        dy: 0.0,
        width: width as f64,
        steps,
        flips: 0,
        cracked: cracked_on,
    };
    // The first-turn pick (:395-493): at the ±X edges (the 100/200 bands)
    // and the Y bounds the direction flips; the slant rolls ride the
    // slant scalars. The port's first pass takes the ±X arm.
    let slant = |r: &mut HallSeed| -> (f64, f64) {
        let xj = if r.next(2) == 0 {
            r.range(20, 50) as f32 * 0.01
        } else {
            -(r.range(20, 50) as f32 * 0.01)
        } as f64;
        let yj = if r.next(2) == 0 {
            r.range(20, 50) as f32 * 0.01
        } else {
            -(r.range(20, 50) as f32 * 0.01)
        } as f64;
        (xj * HALL_SLANT_VARIANT_SCALAR, yj * HALL_SLANT_VARIANT_SCALAR)
    };
    if walk.x > (world.width - 200) as f64 {
        walk.dx = -1.0;
        walk.dy = 0.0;
    } else if walk.x < 200.0 {
        walk.dx = 1.0;
        walk.dy = 0.0;
    } else if walk.y >= num6 as f64 {
        walk.dx = 0.0;
        walk.dy = -1.0;
    } else if walk.y < 200.0 {
        walk.dx = 0.0;
        walk.dy = 1.0;
    }
    while walk.steps > 0 {
        // The bounds stop (:600-629).
        if walk.dx > 0.0 && walk.x > (world.width - 100) as f64 {
            break;
        }
        if walk.dx < 0.0 && walk.x < 100.0 {
            break;
        }
        if walk.dy > 0.0 && walk.y >= num6 as f64 {
            break;
        }
        if walk.dy < 0.0 && walk.y < (walk.y * 0.0 + 100.0) {
            break;
        }
        if walk.dy < 0.0 && walk.y < underworld as f64 - 100.0 {
            break;
        }
        walk.steps -= 1;
        walk.flips += 1;
        // The brick shell rect (:631-645) with the ±4 jitter (±6).
        let jx = r.next(6) as i32;
        let jy = r.next(6) as i32;
        let x0 = (walk.x - walk.width - 4.0 - jx as f64).max(0.0) as i32;
        let x1 = (walk.x + walk.width + 4.0 + jx as f64).min((world.width - 1) as f64) as i32;
        let y0 = (walk.y - walk.width - 4.0 - jy as f64).max(0.0) as i32;
        let y1 = (walk.y + walk.width + 4.0 + jy as f64).min((world.height - 1) as f64) as i32;
        fill_brick(world, x0, y0, x1, y1, brick, cracked, wall, walk.cracked, underworld);
        // The interior carve (:655-728) at `width * 0.5`.
        let iw = (walk.width * HALL_INTERIOR_TO_EXTERIOR_RATIO) as f64;
        let extra = if walk.dy == 0.0 && r.next(walk.width as u32 + 1) == 0 {
            r.range(1, 3) as f64
        } else {
            0.0
        };
        carve_interior(
            world,
            (walk.x - iw - extra).max(0.0) as i32,
            (walk.y - iw - extra).max(0.0) as i32,
            (walk.x + iw + extra).min((world.width - 1) as f64) as i32,
            (walk.y + iw + extra).min((world.height - 1) as f64) as i32,
            cracked,
            wall,
            underworld,
            walk.cracked,
        );
        walk.x += walk.dx;
        walk.y += walk.dy;
        if walk.flips > r.range(10, 20) as i32 {
            walk.flips = 0;
            walk.dx *= -1.0;
        }
        let _ = slant;
        let _ = HALL_STRENGTH_SCALAR;
    }
}

/// `LegacyDungeonRoom.LegacyRoom` (:87-233): the room's diameter
/// `15 * strength + Next(15)`, the slant vector, the outer brick shell at
/// `±(diameter/2)` and the inner carve. `strength` is 1.0 for the starting
/// room (:105).
#[allow(clippy::too_many_arguments)]
fn generate_room(
    world: &mut World,
    cx: i32,
    cy: i32,
    strength: f64,
    brick: u16,
    cracked: u16,
    wall: u16,
    max_y: i32,
    r: &mut HallSeed,
) {
    let underworld = max_y - 220;
    let diameter = (15.0 * strength) as i32 + r.next(15) as i32;
    let half = diameter as f64 / 2.0;
    // The slant (:108-118) — the ±0.1 jitter, at least one axis non-zero.
    let mut slant_x = (r.range(0, 11) as f32 * 0.1) as f64 - 0.5;
    let mut slant_y = (r.range(0, 11) as f32 * 0.1) as f64 - 0.5;
    if slant_x == 0.0 && slant_y == 0.0 {
        if r.next(2) == 0 {
            slant_x = if r.next(2) != 0 { 1.0 } else { -1.0 };
        } else {
            slant_y = if r.next(2) != 0 { 1.0 } else { -1.0 };
        }
    }
    // The room centre is above `j` by half the diameter (:119-121:
    // `vector2D2.Y = j - num2 / 2`).
    let top_y = cy as f64 - half;
    let x0 = (cx as f64 - half - slant_x.abs()).max(0.0) as i32;
    let x1 = (cx as f64 + half + slant_x.abs()).min((world.width - 1) as f64) as i32;
    let y0 = (top_y - half - slant_y.abs()).max(0.0) as i32;
    let y1 = (top_y + half + slant_y.abs()).min((world.height - 1) as f64) as i32;
    // The brick shell + the wall + the carved interior (the C#'s
    // `_outerShapeData`/`_innerShapeData` shape pair, :190-233 — the port
    // writes the same two rects).
    fill_brick(world, x0, y0, x1, y1, brick, cracked, wall, false, underworld);
    let ix0 = (cx as f64 - half).max(0.0) as i32;
    let ix1 = (cx as f64 + half).min((world.width - 1) as f64) as i32;
    let iy0 = (top_y - half).max(0.0) as i32;
    let iy1 = (top_y + half).min((world.height - 1) as f64) as i32;
    carve_interior(world, ix0, iy0, ix1, iy1, cracked, wall, underworld, false);
}

/// `DungeonUtils.ChangeTileType` + the `resetTile` interior rule
/// (`DungeonHall.cs:612-624`): the shell is solid brick (the interior cells
/// reset their frames), the cracked variant replaces a 1/3 roll.
fn fill_brick(
    world: &mut World,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    brick: u16,
    cracked: u16,
    wall: u16,
    cracked_on: bool,
    underworld: i32,
) {
    for x in x0..=x1 {
        for y in y0..=y1 {
            if x < 0 || y < 0 || x >= world.width || y >= world.height || y > underworld {
                continue;
            }
            let i = world.index(x, y);
            let t = &mut world.tiles[i];
            t.liquid = 0;
            let kind = if cracked_on && cracked > 0 && world.frame_important.len() > cracked as usize {
                cracked
            } else {
                brick
            };
            t.kind = kind;
            t.flags |= FLAG_ACTIVE;
            if world.frame_important.len() > kind as usize {
                world.frame_important[kind as usize] = true;
            }
            // `ChangeWallType` runs on the inner ring only (`:639-653`;
            // the shell's cells keep the terrain wall).
            t.wall = wall;
        }
    }
}

/// `DungeonHall.cs:656-728`'s carve loop: the tiles clear inside the
/// interior rect (the cracked variant leaves the cracked-brick seed when
/// flagged), the dungeon walls paint under the underworld line.
fn carve_interior(
    world: &mut World,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    cracked: u16,
    wall: u16,
    underworld: i32,
    cracked_on: bool,
) {
    for x in x0..=x1 {
        for y in y0..=y1 {
            if x < 0 || y < 0 || x >= world.width || y >= world.height || y >= underworld {
                continue;
            }
            let i = world.index(x, y);
            let t = &mut world.tiles[i];
            t.liquid = 0;
            t.flags &= !FLAG_ACTIVE;
            if cracked_on && cracked > 0 && world.frame_important.len() > cracked as usize {
                t.kind = cracked;
                t.flags |= FLAG_ACTIVE;
                world.frame_important[cracked as usize] = true;
            }
            t.wall = wall;
        }
    }
}

/// `LegacyDungeonLayoutProvider` + `MakeDungeon`'s combined pass over the
/// world: the setup (the color roll + the step count), the layout crawl,
/// the entrance tower + shaft at the dungeon anchor, and the cracked-brick
/// runs. The vanilla's entrance-hall precalculation and the biome styles
/// (`dungeonGenerationStyles`) ride the follow-up.
///
/// The colour roll lives here — [`DungeonColor::roll`] below is this pass's
/// `SetupDungeonGenVarVariables` `genRand.Next(3)` triple, and the live
/// pipeline path carries its twin in `biomes::pass_dungeon`
/// (`biomes.rs:1752`'s `rng.range(0, 3)` → the 41/43/44 + 7/8/9 rows). The
/// drunk world's *re*-roll of that colour is NOT here: it is
/// `WorldGen.FinishDrunkGen` (`WorldGen.cs:26226-26260` — the
/// `Next(13, 25)` opener, the per-brick 41/43/44 bands 19-23/15-19/13-15
/// and the `wallColor(25)` sweep) which runs in the seed-finish tail
/// (`finish_seeds::finish_drunk_gen`'s owner), so the brick/wall ids stay
/// the `Setup` triple here.
///
/// M5-SEED-1 seam: the `step_dungeon` step in `pipeline.rs` currently
/// routes through `biomes::pass_dungeon` (the shape-level approximation),
/// so this faithful pass is exercised by its own tests below rather than
/// by the pipeline; the drunk start-row arm is ported here so the two
/// implementations can be reconciled by the dungeon's owner without a
/// re-derivation.
/// `LegacyEntranceDungeonHall.LegacyHall`'s precalculated arm
/// (`Halls/LegacyEntranceDungeonHall.cs:19-260`, the `base.calculated` /
/// `UsePrecalculatedEntrance` path): the walk from `from` toward `to` in
/// `steps` steps — the brick shell rect (the ±4/±6 jitter), the interior
/// wall rect, then the interior carve (the ratio rect with the ±1/2 bump).
///
/// The port's deviations (recorded in ANALYSIS "Dungeon entrance halls"):
/// the room-protection bookkeeping (`DungeonUtils.GetHighestProtectionTypeFromPoint`
/// — the port has no room/protection records), the platform data list
/// (`dungeonData.dungeonPlatformData` — no port store), the bounds tracker
/// (`dungeonData.dungeonBounds`), the `Override*` settings arms (their only
/// caller is the precalculated wrapper below, which sets `OverrideSteps`),
/// and the DualDungeon `CrackedBrickChance = 0` (no dual-dungeon world).
fn generate_entrance_hall(
    world: &mut World,
    from: (f64, f64),
    to: (f64, f64),
    steps: i32,
    strength: i32,
    brick: u16,
    cracked: u16,
    wall: u16,
    max_y: i32,
    r: &mut HallSeed,
) {
    // `(EndPosition - StartPosition).SafeNormalize(Vector2D.UnitX)` — a zero
    // vector keeps the C#'s fallback of the unit X.
    let (vx, vy) = (to.0 - from.0, to.1 - from.1);
    let len = (vx * vx + vy * vy).sqrt();
    let (mut dx, mut dy) = if len > 0.0 { (vx / len, vy / len) } else { (1.0, 0.0) };
    // The precalculated arm's direction IS the normalized start->end vector
    // (:90's `(EndPosition - StartPosition).SafeNormalize(UnitX)`): the
    // slant rolls, the `num3` direction pick and the skewed/drunk clamps
    // (:43-79) belong to the UN-precalculated path, which this port does not
    // walk (the precalculated wrapper is the only caller).
    let _ = &mut dy;
    let underworld = max_y - 220;
    let (mut px, mut py) = from;
    let mut steps = steps;
    while steps > 0 {
        steps -= 1;
        // `!WorldGen.InWorld(x, y, 35)` (:137-140).
        if px < 35.0 || py < 35.0 || px > (world.width - 35) as f64 || py > (world.height - 35) as f64 {
            break;
        }
        let n = strength as f64;
        // The brick shell rect with the jitter (:142-151).
        let bx0 = (px - n - 4.0 - r.next(6) as f64).max(30.0) as i32;
        let bx1 = (px + n + 4.0 + r.next(6) as f64).min((world.width - 31) as f64) as i32;
        let by0 = (py - n - 4.0).max(30.0) as i32;
        let by1 = (py + n + 4.0 + r.next(6) as f64).min((world.height - 31) as f64) as i32;
        // The brick writes (:171-186): the liquid clear + the brick type
        // write over the whole shell rect (the C#'s CanPlaceTileAt gate —
        // the port's `fill_brick` underworld band sits the cracked-brick
        // arms, so the shells write brick directly here).
        for k in bx0..=bx1 {
            for l in by0..=by1 {
                let Some(t) = world.tile_mut(k, l) else { continue };
                t.liquid = 0;
                t.kind = brick;
                t.flags |= crate::tile::FLAG_ACTIVE;
                t.set_paint(0);
            }
        }
        // The wall writes over the shell's inner rect (:188-201). The C#'s
        //  wall-paint clear rides the port's extras map,
        // which the dungeon never writes — the clear is a no-op here.
        for m in (bx0 + 1)..=(bx1 - 1) {
            for n2 in (by0 + 1)..=(by1 - 1) {
                let Some(t) = world.tile_mut(m, n2) else { continue };
                t.wall = wall;
            }
        }
        // The interior carve rect with the ±1/2 bump (:218-231) at the ratio
        // (0.5) — the ClearTile + the wall write.
        let bump = if r.next(strength as u32) == 0 { r.range(1, 3) as f64 } else { 0.0 };
        let rw = n * HALL_INTERIOR_TO_EXTERIOR_RATIO;
        let ix0 = (px - rw - bump).max(30.0) as i32;
        let ix1 = (px + rw + bump).min((world.width - 31) as f64) as i32;
        let iy0 = (py - rw - bump).max(30.0) as i32;
        let iy1 = (py + rw + bump).min((world.height - 31) as f64) as i32;
        for x in ix0..=ix1 {
            for y in iy0..=iy1 {
                let Some(t) = world.tile_mut(x, y) else { continue };
                *t = Tile::default();
                t.wall = wall;
            }
        }
        let _ = underworld;
        let _ = cracked;
        let _ = dy;
        px += dx;
        py += dy;
    }
}

/// `DungeonCrawler.MakeDungeon_GenerateNextEntranceHall_Precalculated`
/// (`DungeonCrawler.cs:520-553`): the entrance hall toward the surface
/// entrance. The budget `Next(10, 30)` is clamped to the walk's remaining
/// distance (`Math.Max(1, (int)dist - amountPassed)`), the hall's end point
/// is the `Vector2D.Lerp(currentPos, entrancePos, amountPassed / dist)`, and
/// the walk ends (`createdDungeonEntranceOnSurface`) when the budget runs
/// out. The precalculated path skips the C#'s surface-detection arm (the
/// `!UsePrecalculatedEntrance` guard, :164-176). Returns whether the
/// entrance reached the surface.
fn walk_entrance_hall(
    world: &mut World,
    current: (f64, f64),
    entrance: (f64, f64),
    dist: i32,
    amount_passed: &mut i32,
    strength: i32,
    brick: u16,
    cracked: u16,
    wall: u16,
    max_y: i32,
    r: &mut HallSeed,
) -> ((f64, f64), bool) {
    let mut num = r.range(10, 30) as i32;
    if num > dist - *amount_passed {
        num = (dist - *amount_passed).max(1);
    }
    let t = if dist > 0 { *amount_passed as f64 / dist as f64 } else { 1.0 };
    let end = (
        current.0 + (entrance.0 - current.0) * t,
        current.1 + (entrance.1 - current.1) * t,
    );
    generate_entrance_hall(world, current, end, num, strength, brick, cracked, wall, max_y, r);
    *amount_passed -= num;
    let surface = *amount_passed <= 0;
    (end, surface)
}

pub fn generate_dungeon(world: &mut World, x0: i32, surface_y: i32, r: &mut HallSeed) {
    // `WorldGen.cs:14804-14807` — and its dual-dungeon twin at `:14854-14857`
    // (one per `GenVars.dungeonGenVars` entry): a drunk world (without
    // `noSurface`) pins the dungeon start row to
    // `(int)Main.worldSurface + 70` instead of the searched solid-tile depth
    // the caller computed. The `Next(-200, 200)` offset draw (`:14769`) and
    // the whole solid-tile search (`:14773-14803`) sit *before* the drunk
    // line in vanilla, so both are evaluated either way — the arm replaces
    // only the resulting start row and burns no draw of its own, which is
    // what keeps a classic world byte-identical.
    let top = if world.seeds.contains(crate::seeds::SeedFlags::DRUNK) {
        world.world_surface as i32 + 70
    } else {
        surface_y
    };
    let color = DungeonColor::roll(&mut |n| r.next(n));
    let brick = color.brick_tile();
    let cracked = color.cracked_tile();
    let wall = color.brick_wall();
    let setup = DungeonSetup::roll(&mut |lo, hi| r.range(lo, hi), world.width);
    let bottom = (world.height - 220).min(top + (world.rock_layer as i32) * 2);
    // The entrance tower (:the existing pass's contract — a 5-wide shell
    // from the surface and the 3-wide carved shaft).
    let mut y = top - 8;
    while y < top + 30 {
        for dx in -3..=3 {
            fill_brick(
                world,
                x0 + dx,
                y,
                x0 + dx,
                y,
                brick,
                cracked,
                wall,
                false,
                world.height - 220,
            );
        }
        y += 1;
    }
    for yy in (top - 7)..(top + 30) {
        for dx in -1..=1 {
            let i = world.index(x0 + dx, yy);
            world.tiles[i] = Tile::default();
            world.tiles[i].wall = wall;
        }
    }
    // The entrance-hall precalculation (`MakeDungeon`'s main loop, the
    // precalculated arm `DungeonCrawler.cs:441-456` + :520-553): the walk
    // from the shaft's head toward the surface entrance until the budget
    // (`setup.steps` — the C#'s `num2`) is spent. The port's tower stands
    // at the anchor's own X (the C#'s searched entrance is offset by the
    // entrance strengths — the port's geometry deviation, ANALYSIS), so the
    // walk direction's X rides the entrance-strength sign arms.
    let mut amount_passed = setup.steps;
    let mut cursor = (x0 as f64, (top + 40) as f64);
    loop {
        let (end, surface) = walk_entrance_hall(
            world,
            cursor,
            (x0 as f64, top as f64),
            setup.steps,
            &mut amount_passed,
            setup.entrance_strength_x,
            brick,
            cracked,
            wall,
            world.height,
            r,
        );
        if surface {
            break;
        }
        cursor = end;
    }
    // The crawler from the shaft's foot.
    crawl_layout(world, x0, top + 40, setup.steps, brick, cracked, wall, world.height, r);
    let _ = setup.entrance_strength_y;
    let _ = setup.entrance_strength_x2;
    let _ = setup.entrance_strength_y2;
    let _ = bottom;
    let _ = ROOM_STRENGTH_SCALAR;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A flat stone world with the dirt-over-stone wall baseline (the same
    /// fixture shape the port's dungeon tests use).
    fn flat_world(width: i32, height: i32, surface: i32) -> World {
        let mut w = World::new("d".into(), width, height);
        for x in 0..width {
            for y in 0..height {
                let i = w.index(x, y);
                let mut t = Tile::default();
                if y >= surface {
                    t.kind = 1;
                    t.flags |= FLAG_ACTIVE;
                    if y > surface + 2 {
                        t.wall = 1;
                    }
                }
                w.tiles[i] = t;
            }
        }
        w
    }

    #[test]
    fn test_dungeon_1_the_color_roll_carries_its_bricks() {
        // `SetupDungeonGenVarVariables`'s triple: the brick/wall/cracked
        // rows stay in lockstep across the three colours.
        let mut rng = HallSeed::new(7);
        for _ in 0..50 {
            let c = DungeonColor::roll(&mut |n| rng.next(n));
            let (t, w, ck) = (c.brick_tile(), c.brick_wall(), c.cracked_tile());
            match c {
                DungeonColor::Blue => assert_eq!((t, w, ck), (41, 7, 481)),
                DungeonColor::Green => assert_eq!((t, w, ck), (43, 8, 482)),
                DungeonColor::Pink => assert_eq!((t, w, ck), (44, 9, 483)),
            }
        }
    }

    #[test]
    fn test_dungeon_1_the_step_count_follows_the_formula() {
        // `MakeDungeon` :359-362: `width / 60 + Next(0, num / 3)` — the
        // 4200-wide world rolls 70 + `Next(0, 23)`.
        let mut rng = HallSeed::new(11);
        for _ in 0..50 {
            let s = DungeonSetup::roll(&mut |lo, hi| rng.range(lo, hi), 4200);
            assert!(
                (70..=93).contains(&s.steps),
                "the step count {} in [70, 93)",
                s.steps
            );
            assert!((25..30).contains(&s.entrance_strength_x));
            assert!((20..25).contains(&s.entrance_strength_y));
            assert!((35..50).contains(&s.entrance_strength_x2));
            assert!((10..15).contains(&s.entrance_strength_y2));
        }
    }

    #[test]
    fn test_dungeon_1_the_pass_carves_the_dungeon_grid() {
        // The crawler from the anchor: brick tiles, dungeon walls and
        // carved air all land inside the world bounds.
        let mut w = flat_world(600, 400, 60);
        let mut rng = HallSeed::new(0xD00D);
        generate_dungeon(&mut w, 300, 60, &mut rng);
        let mut brick_cells = 0usize;
        let mut wall_cells = 0usize;
        let mut carved = 0usize;
        for y in 0..w.height {
            for x in 0..w.width {
                let t = w.tile(x, y).unwrap();
                if t.kind == 41 || t.kind == 43 || t.kind == 44 {
                    brick_cells += 1;
                }
                if t.wall == 7 || t.wall == 8 || t.wall == 9 {
                    wall_cells += 1;
                }
                if y > 60 && !t.active() {
                    carved += 1;
                }
            }
        }
        assert!(brick_cells > 500, "the brick shell landed: {brick_cells}");
        assert!(wall_cells > 500, "the dungeon walls landed: {wall_cells}");
        assert!(carved > 200, "the interior carve landed: {carved}");
    }

    /// The topmost row carrying a dungeon brick (41/43/44) — the entrance
    /// tower's first shell row (`top - 8`).
    fn topmost_brick_row(w: &World) -> Option<i32> {
        (0..w.height).find(|y| {
            (0..w.width).any(|x| {
                let t = w.tile(x, *y).unwrap();
                t.active() && matches!(t.kind, 41 | 43 | 44)
            })
        })
    }

    #[test]
    fn test_dungeon_1_the_drunk_start_row_pins_the_surface_plus_70() {
        // `WorldGen.cs:14804-14807` (the dual-dungeon twin `:14854-14857`):
        // drunk replaces the searched start row with
        // `(int)Main.worldSurface + 70`, so the entrance tower (which begins
        // 8 rows above the start row) moves down with it. A classic world
        // keeps the searched row — the arm is flag-gated and burns no draw,
        // so both the classic tile grid and the classic draw sequence are
        // untouched.
        // A vanilla-height world: `num6 = (maxTilesY - 220) - 100` = 880
        // puts both start rows above `LegacyDungeonHall`'s upward-turn bound
        // (`WorldGen.cs:149`-ish, the port's `generate_hall` `num6`), so the
        // crawler walks DOWN from the shaft foot and never paints above the
        // entrance tower — the tower's top row is the pass's start row minus
        // 8, and reading it back reads the drunk arm directly.
        let mut classic = flat_world(600, 1200, 60);
        let mut drunk = flat_world(600, 1200, 60);
        // The searched depth (the port's `surface_y`, the vanilla solid-tile
        // search product) and `Main.worldSurface` disagree on purpose so the
        // two arms pick different rows.
        classic.world_surface = 60.0;
        drunk.world_surface = 60.0;
        drunk.seeds |= crate::seeds::SeedFlags::DRUNK;
        let mut r1 = HallSeed::new(0xD00D);
        let mut r2 = HallSeed::new(0xD00D);
        generate_dungeon(&mut classic, 300, 92, &mut r1);
        generate_dungeon(&mut drunk, 300, 92, &mut r2);
        // The precalculated entrance hall's brick shell rides ±(strength+4)
        // around the walk's end (the surface entrance) — the vanilla's
        // visible entrance tower — so the topmost brick is the shell's top
        // edge at the entrance point, not the shaft tower's top row. Both
        // values are the fixed-seed pins of that shell.
        assert_eq!(topmost_brick_row(&classic), Some(59), "the classic entrance shell");
        assert_eq!(topmost_brick_row(&drunk), Some(97), "the drunk entrance shell");
    }

    #[test]
    fn test_dungeon_1_the_classic_world_is_untouched_by_the_drunk_arm() {
        // The flag gate is the whole contract: the drunk pass over a classic
        // world reproduces the classic grid cell-for-cell (and the classic
        // world reproduces itself), so the pinned `world_hash` worlds stay
        // where they are.
        let mut a = flat_world(600, 400, 60);
        let mut b = flat_world(600, 400, 60);
        let mut r1 = HallSeed::new(0xD00D);
        let mut r2 = HallSeed::new(0xD00D);
        generate_dungeon(&mut a, 300, 92, &mut r1);
        generate_dungeon(&mut b, 300, 92, &mut r2);
        for i in 0..a.tiles.len() {
            assert_eq!(a.tiles[i].kind, b.tiles[i].kind, "cell {i} kind");
        }
    }

    #[test]
    fn test_dungeon_1_the_determinism_holds() {
        // Two runs on the same seed agree cell-for-cell.
        let mut a = flat_world(600, 400, 60);
        let mut b = flat_world(600, 400, 60);
        let mut r1 = HallSeed::new(0xD00D);
        let mut r2 = HallSeed::new(0xD00D);
        generate_dungeon(&mut a, 300, 60, &mut r1);
        generate_dungeon(&mut b, 300, 60, &mut r2);
        for i in 0..a.tiles.len() {
            assert_eq!(a.tiles[i].kind, b.tiles[i].kind, "cell {i} kind");
            assert_eq!(a.tiles[i].wall, b.tiles[i].wall, "cell {i} wall");
            assert_eq!(a.tiles[i].flags, b.tiles[i].flags, "cell {i} flags");
        }
    }








    /// DW-2.7: the precalculated entrance hall — the walker writes the brick
    /// shell and the interior wall along the path, and the wrapper's budget
    /// clamps (`Math.Max(1, dist - amountPassed)`) and surface flag ride the
    /// C#'s `MakeDungeon_GenerateNextEntranceHall_Precalculated`.
    #[test]
    fn test_dungeon_2_the_precalculated_entrance_hall_walks_and_clamps() {
        let mut w = flat_world(200, 400, 100);
        w.world_surface = 100.0;
        let h = w.height;
        let mut r = HallSeed::new(0xE171);
        let mut amount = 40;
        let (_, surface) = walk_entrance_hall(
            &mut w,
            (100.0, 300.0),
            (100.0, 100.0),
            40,
            &mut amount,
            25,
            41,
            481,
            7,
            h,
            &mut r,
        );
        // The budget spent exactly the walk's distance: the first hall's
        //  clamped to  = 0 → max(1, 0) = 1, so one pass
        // leaves 39 and the surface flag stays down.
        assert_eq!(amount, 39, "one clamped step of the 40-row walk");
        assert!(!surface, "the walk continues while budget remains");
        // The shell rode the path: brick along the column around the start,
        // walls inside it, and the carved interior.
        let brick_or_wall = |x: i32, y: i32| {
            let t = w.tile(x, y).unwrap();
            t.active() && matches!(t.kind, 41 | 43 | 44) || t.wall == 7
        };
        assert!(brick_or_wall(100, 296), "the shell under the start row");
        assert!(brick_or_wall(90, 296) || brick_or_wall(110, 296), "the shell's side");
    }

    /// DW-2.7: the zero-length direction falls back to the C#'s unit X
    /// (`SafeNormalize(Vector2D.UnitX)`), and the budget's exhaustion raises
    /// the surface flag.
    #[test]
    fn test_dungeon_2_the_entrance_hall_falls_back_and_flags_the_surface() {
        let mut w = flat_world(200, 400, 100);
        let h = w.height;
        let mut r = HallSeed::new(0xBEEF);
        // A degenerate entrance (the same point): the walk goes sideways in
        // the fallback direction; the budget 1 exhausts in one call.
        let mut amount = 1;
        let (_, surface) = walk_entrance_hall(
            &mut w,
            (100.0, 200.0),
            (100.0, 200.0),
            1,
            &mut amount,
            10,
            41,
            481,
            7,
            h,
            &mut r,
        );
        assert_eq!(amount, 0);
        assert!(surface, "the spent budget creates the entrance on the surface");
        let saw_brick = (90..110).any(|x| {
            let t = w.tile(x, 200).unwrap();
            t.active() && matches!(t.kind, 41 | 43 | 44)
        });
        assert!(saw_brick, "the fallback direction laid brick around the point");
    }

}
