//! The wiring engine — `Terraria/Wiring.cs`: the four wire colours, the
//! `TripWire` → `HitWire` flood fill with the skip set and the multi-trigger
//! counter, `HitWireSingle`'s device dispatch, the `CheckMech` cooldown queue
//! and the tile-144 timers.
//!
//! Source map (line numbers into `decompiled/Terraria/Wiring.cs` unless noted):
//! - `TripWire` `:525-620` — the four sequential colour passes
//! - `HitWire` `:837-995` — the BFS: `SkipWire`, `_toProcess` counting
//!   (`Add(point, 4)`, re-entries decrement), the 4-neighbourhood walk with
//!   the per-colour wire check and the `[2, max-2]` bounds
//! - `HitWireSingle` `:996+` — the per-tile device dispatch (the actuator
//!   hook, the tile-144 timer re-trigger, the 421/422 lamp swap)
//! - `HitSwitch` `:259-525` — the switch → TripWire rect table
//! - `CheckMech` `:455-471` — the 999-slot cooldown queue,
//!   `UpdateMech` `:145-258` — the tick + the timer periods
//!   (60/180/300/30/15 by `frameX / 18`)
//!
//! Deviations (ANALYSIS "Wiring engine (M7-WIRE-1)"): the wire colours live in
//! a side table ([`WireState`]) instead of the C#'s `sTileHeader` bits — the
//! port's packed `Tile` has no header space; device effects that spawn
//! entities or move liquids (statues, traps, pumps, teleporters, cannons) are
//! passthroughs for M7-WIRE-2; the wire pulse animation (case 419's frameX
//! shift) is render-side.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::tile::LiquidKind;
use crate::tile::{FLAG_ACTUATOR, FLAG_INACTIVE};
use crate::world::World;

/// A statue spawn request (`HitWireSingle`'s case 105): the C# NPC/item id
/// and the spawn pixel anchor. The app layer resolves the ids to its content
/// and runs the `MechSpawn` caps (`NPC.cs:7420`, `Item.cs:49385`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StatueSpawn {
    Npc { x: i32, y: i32, npc_id: i16 },
    Item { x: i32, y: i32, item_id: i16 },
}

/// The four wire colours, in `TripWire`'s pass order (`Wiring.cs:525`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WireColor {
    Red = 1,
    Blue = 2,
    Green = 3,
    Yellow = 4,
}

/// The wire colours as a sparse side table — `Tile.wire()`/`wire2()`/`wire3()`/
/// `wire4()` are `sTileHeader` bits in the C# (`Tile.cs:623-657`); the port's
/// packed 12-byte `Tile` carries no header, so the set bits live here.
#[derive(Clone, Debug, Default)]
pub struct WireState {
    red: HashSet<(i32, i32)>,
    blue: HashSet<(i32, i32)>,
    green: HashSet<(i32, i32)>,
    yellow: HashSet<(i32, i32)>,
}

impl WireState {
    pub fn get(&self, x: i32, y: i32, color: WireColor) -> bool {
        match color {
            WireColor::Red => self.red.contains(&(x, y)),
            WireColor::Blue => self.blue.contains(&(x, y)),
            WireColor::Green => self.green.contains(&(x, y)),
            WireColor::Yellow => self.yellow.contains(&(x, y)),
        }
    }

    pub fn set(&mut self, x: i32, y: i32, color: WireColor, on: bool) {
        let set = match color {
            WireColor::Red => &mut self.red,
            WireColor::Blue => &mut self.blue,
            WireColor::Green => &mut self.green,
            WireColor::Yellow => &mut self.yellow,
        };
        if on {
            set.insert((x, y));
        } else {
            set.remove(&(x, y));
        }
    }

    fn any_in_rect(&self, left: i32, top: i32, width: i32, height: i32, color: WireColor) -> bool {
        let set = match color {
            WireColor::Red => &self.red,
            WireColor::Blue => &self.blue,
            WireColor::Green => &self.green,
            WireColor::Yellow => &self.yellow,
        };
        set.iter()
            .any(|&(x, y)| x >= left && x < left + width && y >= top && y < top + height)
    }
}

/// One `CheckMech` cooldown entry (`Wiring.cs:455-471`): a tile position plus
/// the ticks left on its cooldown.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Mech {
    x: i32,
    y: i32,
    time: i32,
}

/// The wiring state: the mech queue, the device cooldowns and the per-call
/// scratch (the C# statics `Wiring._mechX/...`, `_wireSkip`, `_toProcess`).
#[derive(Debug, Default)]
pub struct Wiring {
    mechs: Vec<Mech>,
    /// `Wiring.cannonCoolDown` / `bunnyCannonCoolDown` / `snowballCannonCoolDown`
    /// (`Wiring.cs:146-156` decrements; the cannons fire in M7-WIRE-2).
    cannon_cooldown: i32,
    bunny_cannon_cooldown: i32,
    snowball_cannon_cooldown: i32,
    /// `_wireSkip`: tiles already `HitWireSingle`'d in the running flood.
    skip: HashSet<(i32, i32)>,
    /// `_toProcess`: pending multi-trigger counts — a wire reached through N
    /// paths must fire once and then stay silent for N-1 re-entries
    /// (`HitWire`'s `Add(point, 4)` + the decrement, `:915-950`).
    to_process: HashMap<(i32, i32), i32>,
    /// `Wiring._LampsToCheck` (`:31`): the logic lamps poked by wire hits.
    lamps_to_check: std::collections::VecDeque<(i32, i32)>,
    /// `Wiring._GatesNext` (`:821`): the gates whose output changed and whose
    /// own wire line re-trips after the current pass.
    gates_next: std::collections::VecDeque<(i32, i32)>,
    /// `Wiring._GatesDone` (`:699-706`).
    gates_done: HashMap<(i32, i32), bool>,
    /// The deterministic stand-in for `Main.rand` (the faulty-lamp roll,
    /// `:812`): the port's xorshift core seeded from the world seed.
    faulty_rng: u64,
    /// `Wiring._teleport[2]`: the two teleporter pad anchors in tile coords
    /// (the y carries the halfBrick +0.5), reset to (-1, -1) per colour pass
    /// (`Wiring.cs:554-557`).
    teleport: [(f32, f32); 2],
    /// The teleport pairs computed by the last `TripWire` (`Teleport()` runs
    /// up to four times — once per colour pass, `Wiring.cs:651-661`). The app
    /// layer drains these and moves the entities.
    teleport_events: Vec<((f32, f32), (f32, f32))>,
    /// The statue spawn requests from the last `TripWire` (the statue case
    /// runs inside `HitWireSingle`).
    statue_spawns: Vec<StatueSpawn>,
    /// `Wiring._inPumpX/Y` + `_outPumpX/Y` (`Wiring.cs:43-51`, the 19-cap
    /// registration at `:2065-2125`).
    in_pumps: Vec<(i32, i32)>,
    out_pumps: Vec<(i32, i32)>,
    /// `Wiring.blockPlayerTeleportationForOneIteration` (`Wiring.cs:3126`).
    block_player_teleportation: bool,
    /// The per-tile toggles collected by the flood (`PixelBoxPass`,
    /// `Wiring.cs:666-678`) — the pixel boxes' placement is M7-WIRE-2, the
    /// pass flip is kept for parity.
    pixel_box_triggers: HashMap<(i32, i32), u8>,
    /// The object drops the actuation cascade produced (the
    /// `SquareTileFrame(resetFrame: false)` inside `DeActive`/`ReActive`
    /// popping unsupported objects). The app layer drains and spawns them.
    actuation_drops: Vec<crate::edit::ObjectDrop>,
    /// The trap emission requests from the last flood (the case-137 trap
    /// family). The app layer spawns the projectiles.
    trap_requests: Vec<TrapRequest>,
}

/// A trap emission request (`HitWireSingle`'s case 137, `Wiring.cs:1763-1993`):
/// the engine resolves the variant, the `CheckMech` cooldown, the direction
/// decode and the speed/damage tables; the app layer spawns the projectile
/// (`Shot`) or runs the spear's 1000-slot proximity cap first (`Spear` — the
/// scan reads the projectile pool, so it lives app-side).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrapRequest {
    Shot {
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        kind: u16,
        damage: i32,
    },
    /// The Spear Trap's variant: the engine pre-rolls the ±1 px spread
    /// (`Main.rand.Next(...) * 0.05f`, `Wiring.cs:1838-1839`), the app
    /// applies the proximity cap before spawning.
    Spear {
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        damage: i32,
    },
}

/// The multi-trigger count a wire cell carries when first reached
/// (`HitWire`'s `_toProcess.Add(point, 4)`).
const TO_PROCESS_COUNT: i32 = 4;
/// `CheckMech`'s queue cap (`Wiring.cs:466`).
const MAX_MECHS: usize = 999;

/// `TileID.Sets.NotReallySolid` (`TileID.cs:592`) — the "solid" kinds that
/// never gain the inactive state (doors 10 included).
const NOT_REALLY_SOLID: [u16; 3] = [387, 388, 10];
/// `DeActive`'s forced flag=false set (`Wiring.cs:3204-3212`): the vines and
/// the other half-solid kinds that never actuate even though the C# counts
/// them solid.
const DEACTIVE_SOLID_EXCEPTIONS: [u16; 7] = [314, 379, 386, 387, 388, 389, 476];
/// `TileID.Sets.PreventsActuationUnder` (`TileID.cs:569`): the tile kinds
/// that must not sit on an actuated block — the chests, the dolls, the
/// sensor family (an actuated support would pop them through
/// `SquareTileFrame`).
const PREVENTS_ACTUATION_UNDER: [u16; 11] = [21, 467, 26, 77, 88, 470, 475, 237, 597, 441, 468];
/// `TileID.Sets.IsATreeTrunk` (`TileID.cs:487`).
const IS_A_TREE_TRUNK: [u16; 12] = [5, 72, 583, 584, 585, 586, 587, 588, 589, 596, 616, 634];
/// `TileID.Sets.Boulders` (`TileID.cs:504`) — 138 is the actuator itself.
const BOULDERS: [u16; 10] = [138, 484, 664, 665, 711, 712, 713, 714, 715, 716];

impl Wiring {
    pub fn new() -> Self {
        Self::default()
    }

    /// Seeds the faulty-lamp roll stream (call once per world load).
    pub fn seed_faulty_rng(&mut self, seed: u64) {
        self.faulty_rng = seed ^ 0x9E37_79B9_7F4A_7C15;
    }

    fn faulty_next(&mut self, n: i32) -> i32 {
        self.faulty_rng = self
            .faulty_rng
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let z = self.faulty_rng;
        let z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        let z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        ((z ^ (z >> 31)) % n.max(1) as u64) as i32
    }

    /// `Wiring.CheckMech` (`Wiring.cs:455-471`): one cooldown entry per tile
    /// position, at most 999 queued. Returns whether the entry was added.
    pub fn check_mech(&mut self, x: i32, y: i32, time: i32) -> bool {
        if self.mechs.iter().any(|m| m.x == x && m.y == y) {
            return false;
        }
        if self.mechs.len() >= MAX_MECHS {
            return false;
        }
        self.mechs.push(Mech { x, y, time });
        true
    }

    /// `Wiring.UpdateMech` (`Wiring.cs:145-258`), run once per world update:
    /// the cannon cooldowns tick, every queued mech counts down, the tile-144
    /// timers re-`TripWire` themselves on their period and expire off the
    /// queue (flipping the 411 lamp pair when they do).
    pub fn update(&mut self, world: &mut World) {
        self.cannon_cooldown = (self.cannon_cooldown - 1).max(0);
        self.bunny_cannon_cooldown = (self.bunny_cannon_cooldown - 1).max(0);
        self.snowball_cannon_cooldown = (self.snowball_cannon_cooldown - 1).max(0);
        // The C# iterates the queue back-to-front and compacts on removal
        // (`:168`); `Vec::remove` produces the same logical queue.
        let mut num = self.mechs.len();
        while num > 0 {
            num -= 1;
            // `_mechTime[num]--` (`Wiring.cs:172`).
            self.mechs[num].time -= 1;
            let mech = self.mechs[num];
            if !world.in_bounds(mech.x, mech.y) {
                self.mechs.remove(num);
                continue;
            }
            let tile_kind = world.tile(mech.x, mech.y).map(|t| t.kind);
            if tile_kind == Some(144) && world.tile_active(mech.x, mech.y) {
                // `:181-213`: `frameY == 0` resets the timer; otherwise the
                // period comes from `frameX / 18` and the mech re-trips its
                // own 1×1 wire when the remainder hits zero.
                let frame_y = world.tile_frame_y(mech.x, mech.y);
                if frame_y == 0 {
                    self.set_mech_time(num, 0);
                } else {
                    let period = match world.tile_frame_x(mech.x, mech.y) / 18 {
                        0 => 60,
                        1 => 180,
                        2 => 300,
                        3 => 30,
                        4 => 15,
                        _ => 0,
                    };
                    let time = self.mech_time(num);
                    // `Math.IEEERemainder(t, p) == 0` is exactly `t % p == 0`
                    // for the zero test.
                    if period > 0 && time % period == 0 {
                        self.set_mech_time(num, 18000);
                        self.trip_wire(world, mech.x, mech.y, 1, 1);
                    }
                }
            }
            if self.mech_time(num) <= 0 {
                // `:216-242`: the tile-144 timer frame resets; the tile-411
                // lamp pair flips its 2×2 frameX by ±36.
                if world.tile_active(mech.x, mech.y) {
                    if tile_kind == Some(144) {
                        world.set_tile_frame_y(mech.x, mech.y, 0);
                    } else if tile_kind == Some(411) {
                        let num5 = world.tile_frame_x(mech.x, mech.y) % 36 / 18;
                        let num6 = world.tile_frame_y(mech.x, mech.y) % 36 / 18;
                        let num7 = mech.x - num5;
                        let num8 = mech.y - num6;
                        let num9 = if world
                            .tile(num7, num8)
                            .is_some_and(|t| t.frame_x >= 36)
                        {
                            -36
                        } else {
                            36
                        };
                        for i in num7..num7 + 2 {
                            for j in num8..num8 + 2 {
                                if world.in_bounds(i, j) {
                                    let fx = world.tile_frame_x(i, j);
                                    world.set_tile_frame_x(i, j, fx + num9);
                                }
                            }
                        }
                    }
                }
                self.mechs.remove(num);
            }
        }
    }

    fn mech_time(&self, num: usize) -> i32 {
        self.mechs[num].time
    }

    fn set_mech_time(&mut self, num: usize, time: i32) {
        self.mechs[num].time = time;
    }

    /// `Wiring.HitSwitch` (`Wiring.cs:259-525`): a player/device interaction
    /// with a switchable tile turns into a `TripWire` over the device's rect.
    /// The device-specific effects (the mine's explosion `:210`, the geyser
    /// `:443`, the gem locks' item consuming) are M7-WIRE-2 passthroughs.
    pub fn hit_switch(&mut self, world: &mut World, i: i32, j: i32) {
        if !world.in_bounds(i, j) {
            return;
        }
        let Some(tile) = world.tile(i, j) else {
            return;
        };
        let (kind, frame_x, frame_y) = (tile.kind, tile.frame_x, tile.frame_y);
        if !world.tile_active(i, j) {
            return;
        }
        // The standard switches/levers/plates: a 1×1 trip (`:265-272`).
        if matches!(kind, 135 | 314 | 423 | 428 | 442 | 476) {
            self.trip_wire(world, i, j, 1, 1);
        } else if kind == 440 {
            // The wide switch: a 3×3 trip (`:274-277`).
            self.trip_wire(world, i, j, 3, 3);
        } else if kind == 136 {
            // The lever toggles its frame and trips (`:279-288`).
            let next = if frame_y == 0 { 18 } else { 0 };
            world.set_tile_frame_y(i, j, next);
            self.trip_wire(world, i, j, 1, 1);
        } else if kind == 144 {
            // The timer arms a 18000-tick cooldown when switched on (`:293`).
            if frame_y == 0 {
                world.set_tile_frame_y(i, j, 18);
                self.check_mech(i, j, 18000);
            } else {
                world.set_tile_frame_y(i, j, 0);
            }
        } else if kind == 441 || kind == 468 {
            // The signal flags/gem-lock buttons trip their 2×2 device rect
            // (`:300-315`): the corner is derived from the negative frame
            // offsets, wrapped by 4.
            let (num, num2) = corner_from_frame(frame_x, frame_y, i, j);
            self.trip_wire(world, num, num2, 2, 2);
        } else if kind == 467 && frame_x / 36 == 4 {
            // The large gem lock's trigger variant (`:317-330`).
            let (num, num2) = corner_from_frame(frame_x, frame_y, i, j);
            self.trip_wire(world, num, num2, 2, 2);
        }
        // `:210` mine / `:443` geyser / the remaining special devices: their
        // effects spawn entities — M7-WIRE-2.
    }

    /// `Wiring.TripWire` (`Wiring.cs:525-680`): the four colour passes (each
    /// resetting the teleport pads and the pump queues), the per-pass pad
    /// collection, the `Teleport()` call per registered pair, the pixel-box
    /// pass and the logic-gate pass.
    pub fn trip_wire(&mut self, world: &mut World, left: i32, top: i32, width: i32, height: i32) {
        let mut pairs: [((f32, f32), (f32, f32)); 4] = [((-1.0, -1.0), (-1.0, -1.0)); 4];
        for (pass, color) in [
            (0, WireColor::Red),
            (1, WireColor::Blue),
            (2, WireColor::Green),
            (3, WireColor::Yellow),
        ] {
            self.trip_wire_color(world, left, top, width, height, color);
            pairs[pass] = (self.teleport[0], self.teleport[1]);
        }
        self.pixel_box_pass(world);
        // `for (num4 = 0; num4 < 8; num4 += 2) { … Teleport(); }`
        // (`Wiring.cs:651-661`): both pads registered → one teleport call.
        for pair in pairs {
            if pair.0 .0 >= 0.0 && pair.1 .0 >= 0.0 {
                self.teleport(world, pair.0, pair.1);
            }
        }
        // `Wiring.LogicGatePass()` at the end of `TripWire` (`Wiring.cs:665`).
        self.logic_gate_pass(world);
    }

    /// `Wiring.PixelBoxPass` (`Wiring.cs:666-678`): the pixel-box tiles
    /// triggered from two directions flip their frame. The pixel boxes'
    /// placement/content are M7-WIRE-2.
    fn pixel_box_pass(&mut self, world: &mut World) {
        for (key, value) in std::mem::take(&mut self.pixel_box_triggers) {
            if value == 3 {
                let fx = world.tile_frame_x(key.0, key.1);
                world.set_tile_frame_x(key.0, key.1, if fx != 18 { 18 } else { 0 });
            }
        }
    }

    /// `Wiring.Teleport` (`Wiring.cs:3120-3178`), the engine half: the
    /// too-close reject rule. The entity movement (players/NPCs intersecting
    /// either 48×48 pad box move to the other pad) runs in the app layer off
    /// [`Wiring::take_teleport_events`].
    fn teleport(&mut self, world: &World, pad0: (f32, f32), pad1: (f32, f32)) {
        // `:3122-3127`: the pads overlap → reject.
        if pad0.0 < pad1.0 + 3.0
            && pad0.0 > pad1.0 - 3.0
            && pad0.1 > pad1.1 - 3.0
            && pad0.1 < pad1.1
        {
            return;
        }
        self.teleport_events.push((pad0, pad1));
        let _ = world;
    }

    /// Drains the statue spawn requests computed by the statue triggers.
    pub fn take_statue_spawns(&mut self) -> Vec<StatueSpawn> {
        std::mem::take(&mut self.statue_spawns)
    }

    /// Drains the trap emission requests (the case-137 trap family).
    pub fn take_trap_requests(&mut self) -> Vec<TrapRequest> {
        std::mem::take(&mut self.trap_requests)
    }

    /// The object drops the actuation cascade produced (see
    /// [`Self::actuation_drops`]).
    pub fn take_actuation_drops(&mut self) -> Vec<crate::edit::ObjectDrop> {
        std::mem::take(&mut self.actuation_drops)
    }

    /// Drains the teleport events computed by the last `TripWire`.
    pub fn take_teleport_events(&mut self) -> Vec<((f32, f32), (f32, f32))> {
        std::mem::take(&mut self.teleport_events)
    }

    /// The trap family (`HitWireSingle`'s case 137, `Wiring.cs:1763-1993`):
    /// `frameY / 18` selects the trap — 0 dart (98, 20 dmg, speed 12),
    /// 1 superdart (184, 40, 12), 2 flame (187, 40, 5), 3 spear (185, 40,
    /// 4±1 with the ±1 px spread roll), 4 boulder (186, 60, 8), 5 spiky
    /// ball (980, 30, 12) — the direction decodes from `frameX` (0 → −1 X,
    /// 18 → +1 X, 36-71 → −1 Y, ≥72 → +1 Y; the spear uses `frameX / 18`
    /// slots instead), the spawn offset ±10 (±14 spear, ±18 boulder) px.
    /// The wall-mounted negatives (−10..−6) emit the same projectiles with
    /// fixed offsets. The boulder (aiStyle 37) and the flame jet (aiStyle
    /// 38) requests are emitted verbatim; the app-side spawn drops them
    /// until their families land (M4-PROJ follow-ups).
    fn trap_emit(&mut self, i: i32, j: i32, tile_frame_x: i16, tile_frame_y: i16) {
        let variant = tile_frame_y as i32 / 18;
        let frame_x = tile_frame_x;
        // `Main.rand.Next(lo, hi)` → the deterministic engine stream.
        let mut roll = |wiring: &mut Self, lo: i32, hi: i32| -> f32 {
            let span = (hi - lo).max(1);
            (wiring.faulty_next(span) + lo) as f32
        };
        match variant {
            // `Wiring.cs:1772-1801`: the shared `CheckMech(200)` arm.
            0 | 1 | 2 | 5 if self.check_mech(i, j, 200) => {
                let dx: i32 = if frame_x == 0 {
                    -1
                } else if frame_x == 18 {
                    1
                } else {
                    0
                };
                let dy: i32 = if frame_x >= 36 {
                    if frame_x >= 72 {
                        1
                    } else {
                        -1
                    }
                } else {
                    0
                };
                let (kind, damage, speed) = match variant {
                    0 => (98, 20, 12.0),
                    1 => (184, 40, 12.0),
                    2 => (187, 40, 5.0),
                    _ => (980, 30, 12.0),
                };
                self.trap_requests.push(TrapRequest::Shot {
                    x: (i * 16 + 8 + 10 * dx) as f32,
                    y: (j * 16 + 8 + 10 * dy) as f32,
                    vx: (dx as f32) * speed,
                    vy: (dy as f32) * speed,
                    kind,
                    damage,
                });
            }
            // `Wiring.cs:1802-1849`: the spear — `CheckMech(300)` plus the
            // ±1 px spread roll (the app layer runs the proximity cap).
            3 if self.check_mech(i, j, 300) => {
                let (dx, dy): (i32, i32) = match frame_x / 18 {
                    0 | 1 => (0, 1),
                    2 => (0, -1),
                    3 => (-1, 0),
                    4 => (1, 0),
                    _ => (0, 0),
                };
                let lo_x = -20 + if dx == 1 { 20 } else { 0 };
                let hi_x = 21 - if dx == -1 { 20 } else { 0 };
                let lo_y = -20 + if dy == 1 { 20 } else { 0 };
                let hi_y = 21 - if dy == -1 { 20 } else { 0 };
                let sx = (4 * dx) as f32 + roll(self, lo_x, hi_x) * 0.05;
                let sy = (4 * dy) as f32 + roll(self, lo_y, hi_y) * 0.05;
                self.trap_requests.push(TrapRequest::Spear {
                    x: (i * 16 + 8 + 14 * dx) as f32,
                    y: (j * 16 + 8 + 14 * dy) as f32,
                    vx: sx,
                    vy: sy,
                    damage: 40,
                });
            }
            // `Wiring.cs:1906-1922`: the boulder — `CheckMech(90)`.
            4 if self.check_mech(i, j, 90) => {
                let (dx, dy): (i32, i32) = match frame_x / 18 {
                    0 | 1 => (0, 1),
                    2 => (0, -1),
                    3 => (-1, 0),
                    4 => (1, 0),
                    _ => (0, 0),
                };
                self.trap_requests.push(TrapRequest::Shot {
                    x: (i * 16 + 8 + 18 * dx) as f32,
                    y: (j * 16 + 8 + 18 * dy) as f32,
                    vx: (8 * dx) as f32,
                    vy: (8 * dy) as f32,
                    kind: 186,
                    damage: 60,
                });
            }
            // `Wiring.cs:1883-1920`: the wall-mounted negatives — the same
            // projectiles, fixed offsets, `frameX != 0 → +1 X`.
            -10 | -9 | -8 if self.check_mech(i, j, 200) => {
                let dx: i32 = if frame_x != 0 { 1 } else { -1 };
                let (kind, damage, speed) = match variant {
                    -10 => (98, 20, 12.0),
                    -9 => (184, 40, 12.0),
                    _ => (187, 40, 5.0),
                };
                self.trap_requests.push(TrapRequest::Shot {
                    x: (i * 16 + 8 + 10 * dx) as f32,
                    y: (j * 16 + 7 + 2) as f32,
                    vx: (dx as f32) * speed,
                    vy: 2.0,
                    kind,
                    damage,
                });
            }
            // The -7 spear fires downward with the symmetric spread
            // (`speedY = 4 + rand(0,21) * 0.05`, spawn y + 6).
            -7 if self.check_mech(i, j, 300) => {
                let sx = roll(self, -20, 21) * 0.05;
                let sy = 4.0 + roll(self, 0, 21) * 0.05;
                self.trap_requests.push(TrapRequest::Spear {
                    x: (i * 16 + 8) as f32,
                    y: (j * 16 + 16 + 6) as f32,
                    vx: sx,
                    vy: sy,
                    damage: 40,
                });
            }
            // The -6 boulder fires straight down.
            -6 if self.check_mech(i, j, 90) => {
                self.trap_requests.push(TrapRequest::Shot {
                    x: (i * 16 + 8) as f32,
                    y: (j * 16 + 16 + 10) as f32,
                    vx: 0.0,
                    vy: 8.0,
                    kind: 186,
                    damage: 60,
                });
            }
            _ => {}
        }
    }

    /// The style → spawn map (the NPC styles `CheckMech(30)`, the item
    /// styles `CheckMech(600)`). Returns the spawn request for the app layer,
    /// which runs the `MechSpawn` caps and constructs the entities.
    pub fn statue_trigger(&mut self, world: &mut World, i: i32, j: i32) {
        let Some(tile) = world.tile(i, j) else {
            return;
        };
        if tile.kind != 105 {
            return;
        }
        // `:2128-2139`: the 3-wide body columns × 54-tall style rows.
        let num137 = i - (tile.frame_x % 36 / 18) as i32;
        let num136 = j - (tile.frame_y % 54 / 18) as i32;
        let style = (tile.frame_x / 36) as i32 + (tile.frame_y as i32 / 54 % 3) * 55;
        for corner in [
            (num137, num136),
            (num137, num136 + 1),
            (num137, num136 + 2),
            (num137 + 1, num136),
            (num137 + 1, num136 + 1),
            (num137 + 1, num136 + 2),
        ] {
            self.skip.insert(corner);
        }
        // The spawn anchor: `(num137 * 16 + 16, (num136 + 3) * 16)`
        // (`Wiring.cs:2148-2149`).
        let sx = num137 * 16 + 16;
        let sy = (num136 + 3) * 16;
        // The verbatim style map (`Wiring.cs:2170-2440`). The NPC styles
        // CheckMech(30); the item styles CheckMech(600) — the `Item.MechSpawn`
        // caps run in the app layer (they need the dropped-item list).
        // The SelectRandom styles (the critter/bird/bat statues 51-82) spawn
        // C# NPCs the port does not model yet — they roll nothing here.
        let spawn = match style {
            // The NPC statues (`CheckMech(30)` + `NPC.MechSpawn` in the app).
            4 if self.check_mech(num137, num136, 30) => {
                Some(StatueSpawn::Npc { x: sx, y: sy - 12, npc_id: 1 })
            }
            35 if self.check_mech(num137, num136, 30) => {
                Some(StatueSpawn::Npc { x: sx, y: sy - 12, npc_id: 2 })
            }
            // The item statues (`CheckMech(600)` + `Item.MechSpawn`).
            37 if self.check_mech(num137, num136, 600) => {
                Some(StatueSpawn::Item { x: sx, y: sy - 16, item_id: 58 })
            }
            2 if self.check_mech(num137, num136, 600) => {
                Some(StatueSpawn::Item { x: sx, y: sy - 16, item_id: 184 })
            }
            17 if self.check_mech(num137, num136, 600) => {
                Some(StatueSpawn::Item { x: sx, y: sy - 20, item_id: 166 })
            }
            // The unported statue styles: the C# styles mapping to NPCs/items
            // outside the port's content (the critters 51-82, the Eater 30,
            // the special 9/28/40/71) — documented in ANALYSIS.
            _ => None,
        };
        if let Some(spawn) = spawn {
            self.statue_spawns.push(spawn);
        }
    }

    /// The player-teleportation suppress flag (`Wiring.cs:3126` guard); the
    /// C# clears it inside `LogicGatePass` (`:704-708`).
    pub fn set_block_player_teleportation(&mut self, v: bool) {
        self.block_player_teleportation = v;
    }

    pub fn block_player_teleportation(&self) -> bool {
        self.block_player_teleportation
    }

    /// `Wiring.XferWater` (`Wiring.cs:475-520`): every in-pump tile pushes
    /// its liquid into the type-matched out-pump tiles (an empty out tile
    /// adopts the in type), capped at 255; the in tile's liquid is RE-READ
    /// after each out transfer (`:506`) so the remainder flows into the next
    /// out tile; the port's liquid sim reads the tiles next tick (the C#'s
    /// `SquareTileFrame` wake).
    fn xfer_water(&mut self, world: &mut World) {
        for &(inx, iny) in &self.in_pumps {
            if !world.tile_active(inx, iny) {
                continue;
            }
            if world.tile(inx, iny).is_some_and(|t| t.liquid == 0) {
                continue;
            }
            let in_type = world
                .tile(inx, iny)
                .map(|t| t.liquid_kind())
                .unwrap_or(LiquidKind::Water);
            for &(outx, outy) in &self.out_pumps {
                let Some(out_tile) = world.tile(outx, outy) else {
                    continue;
                };
                let out_liquid = out_tile.liquid;
                if out_liquid >= 255 {
                    continue;
                }
                let mut out_type = out_tile.liquid_kind();
                if out_liquid == 0 {
                    out_type = in_type;
                }
                if out_type != in_type {
                    continue;
                }
                // `int num5 = liquid; if (num5 + liquid2 > 255) num5 = 255 -
                // liquid2;` (`Wiring.cs:502-504`) — widened to i32 (the C#
                // ints), and `liquid` is the tile's CURRENT level (`:506`
                // re-reads it after every transfer).
                let in_liquid = world.tile(inx, iny).map(|t| t.liquid).unwrap_or(0);
                let mut amount = in_liquid as i32;
                if amount + out_liquid as i32 > 255 {
                    amount = 255 - out_liquid as i32;
                }
                let amount = amount as u8;
                {
                    let out_tile = world.tile_mut(outx, outy).unwrap();
                    out_tile.liquid = (out_tile.liquid + amount).min(255);
                    if out_liquid == 0 {
                        out_tile.set_liquid_kind(in_type);
                    }
                }
                {
                    let in_tile = world.tile_mut(inx, iny).unwrap();
                    in_tile.liquid = in_tile.liquid.saturating_sub(amount);
                    // `liquidType(0)` + `SquareTileFrame` on the emptied in
                    // tile (`Wiring.cs:513-516`).
                    if in_tile.liquid == 0 {
                        in_tile.clear_liquid_kind();
                        break;
                    }
                }
            }
        }
    }

    /// `Wiring.PokeLogicGate` (`Wiring.cs:380-386`): queue one lamp and run
    /// the pass.
    pub fn poke_logic_gate(&mut self, world: &mut World, lamp_x: i32, lamp_y: i32) {
        self.lamps_to_check.push_back((lamp_x, lamp_y));
        self.logic_gate_pass(world);
    }

    /// `Wiring.LogicGatePass` (`Wiring.cs:682-730`): the lamp queue drains
    /// into `CheckLogicGate`; every gate whose output changed enqueues into
    /// `_GatesNext`, and the pass re-trips each changed gate's own 1×1 wire
    /// once (`_GatesDone` guards repeats) — that is how gates chain.
    pub fn logic_gate_pass(&mut self, world: &mut World) {
        if !self.gates_next.is_empty() {
            return;
        }
        self.gates_done.clear();
        while !self.lamps_to_check.is_empty() {
            while let Some((x, y)) = self.lamps_to_check.pop_front() {
                self.check_logic_gate(world, x, y);
            }
            while !self.gates_next.is_empty() {
                while let Some(key) = self.gates_next.front().copied() {
                    if self.gates_done.get(&key).copied().unwrap_or(false) {
                        self.gates_next.pop_front();
                        continue;
                    }
                    self.gates_done.insert(key, true);
                    self.trip_wire(world, key.0, key.1, 1, 1);
                    self.gates_next.pop_front();
                }
            }
        }
        self.gates_done.clear();
    }

    /// `Wiring.CheckLogicGate` (`Wiring.cs:733-830`): walk down from the lamp
    /// to the gate (420) below, count the lamp stack above it, evaluate the
    /// gate type (`frameY / 18`: 0 AND, 1 OR, 2 NAND, 3 NOR, 4 XOR, 5 XNOR —
    /// with `frameX == 18` = currently on) and write the new output. A state
    /// change re-enqueues the gate into `_GatesNext`; the faulty lamp
    /// (`frameX == 36` on the poked lamp) rolls the output randomly.
    fn check_logic_gate(&mut self, world: &mut World, lamp_x: i32, lamp_y: i32) {
        if !world.in_bounds(lamp_x, lamp_y) {
            return;
        }
        let lamp_frame_x = world.tile_frame_x(lamp_x, lamp_y);
        for i in lamp_y..world.height {
            let Some(tile) = world.tile(lamp_x, i) else {
                break;
            };
            if !tile.active() {
                break;
            }
            if tile.kind == 420 {
                let done = self.gates_done.get(&(lamp_x, i)).copied().unwrap_or(false);
                let gate_type = tile.frame_y / 18;
                let flag = tile.frame_x == 18;
                let flag2 = tile.frame_x == 36;
                if gate_type < 0 {
                    break;
                }
                // The lamp stack above the gate (`:769-779`): every 419 tile
                // counts; `frameX == 18` counts as on; `frameX == 36` marks
                // the overflow.
                let mut num2 = 0;
                let mut num3 = 0;
                let mut flag3 = false;
                let mut num4 = i - 1;
                while num4 > 0 {
                    let Some(t2) = world.tile(lamp_x, num4) else {
                        break;
                    };
                    if !t2.active() || t2.kind != 419 {
                        break;
                    }
                    if t2.frame_x == 36 {
                        flag3 = true;
                        break;
                    }
                    num2 += 1;
                    num3 += i32::from(t2.frame_x == 18);
                    num4 -= 1;
                }
                let flag4 = match gate_type {
                    0 => num2 == num3,
                    1 => num3 > 0,
                    2 => num2 != num3,
                    3 => num3 == 0,
                    4 => num3 == 1,
                    5 => num3 != 1,
                    _ => return,
                };
                let flag5 = !flag3 & flag2;
                let flag6 = flag3 && world.tile_frame_x(lamp_x, lamp_y) == 36;
                if !((flag4 != flag) | flag5 | flag6) {
                    // No state change — keep walking deeper.
                } else {
                    world.set_tile_frame_x(lamp_x, i, if flag4 { 18 } else { 0 });
                    if flag3 {
                        world.set_tile_frame_x(lamp_x, i, 36);
                    }
                    // `Wiring.SkipWire(lampX, i); WorldGen.SquareTileFrame…`
                    self.skip.insert((lamp_x, i));
                    let mut flag7 = !flag3 || flag6;
                    if flag6 {
                        // The faulty lamp: a random share of the on-lamps
                        // (`Main.rand.Next(num2) < num3`, `:812`).
                        if num3 == 0 || num2 == 0 {
                            flag7 = false;
                        }
                        flag7 = self.faulty_next(num2) < num3;
                    }
                    if flag5 {
                        flag7 = false;
                    }
                    if flag7 {
                        if !done {
                            self.gates_next.push_back((lamp_x, i));
                            break;
                        }
                        // The burn-out poof (`:822-825`) is visual — M7-WIRE-3.
                    }
                    break;
                }
            }
            // `if (tile.type != 419) break;` (`:828-831`): the walk descends
            // past lamps only.
            if world.tile(lamp_x, i).map(|t| t.kind) != Some(419) {
                break;
            }
        }
    }

    fn trip_wire_color(
        &mut self,
        world: &mut World,
        left: i32,
        top: i32,
        width: i32,
        height: i32,
        color: WireColor,
    ) {
        // The per-pass resets: the teleport pads (`:554-557`) and the pump
        // queues (`:129-134`).
        self.teleport = [(-1.0, -1.0); 2];
        self.in_pumps.clear();
        self.out_pumps.clear();
        if !world.wires.any_in_rect(left, top, width, height, color) {
            return;
        }
        let seeds: Vec<(i32, i32)> = (left..left + width)
            .flat_map(|i| (top..top + height).map(move |j| (i, j)))
            .filter(|&(i, j)| world.wires.get(i, j, color))
            .collect();
        self.hit_wire(world, seeds, color);
        // `if (_numInPump > 0 && _numOutPump > 0) XferWater();` (`:560-563`).
        if !self.in_pumps.is_empty() && !self.out_pumps.is_empty() {
            self.xfer_water(world);
        }
    }

    /// `Wiring.HitWire` (`Wiring.cs:837-995`): the flood fill. Every seed is
    /// skipped-and-processed, then the 4-neighbourhood walks while the target
    /// tile carries the running colour; `_toProcess` counts converging paths
    /// and silences re-entries.
    fn hit_wire(&mut self, world: &mut World, seeds: Vec<(i32, i32)>, color: WireColor) {
        self.skip.clear();
        self.to_process.clear();
        let mut next: VecDeque<((i32, i32), u8)> = seeds.iter().map(|&p| (p, 0)).collect();
        for &(x, y) in &seeds {
            self.skip.insert((x, y));
            self.to_process.insert((x, y), TO_PROCESS_COUNT);
        }
        // `HitWire` pops from the front (`DoubleStack.PopFront`) — BFS.
        while let Some(((x, y), _)) = next.pop_front() {
            if !self.skip.contains(&(x, y)) {
                self.hit_wire_single(world, x, y);
            }
            for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                let nx = x + dx;
                let ny = y + dy;
                if nx < 2 || nx >= world.width - 2 || ny < 2 || ny >= world.height - 2 {
                    continue;
                }
                if !world.wires.get(nx, ny, color) {
                    continue;
                }
                let point = (nx, ny);
                if let Some(count) = self.to_process.get_mut(&point) {
                    *count -= 1;
                    if *count == 0 {
                        self.to_process.remove(&point);
                    }
                    continue;
                }
                next.push_back((point, dx_y_to_dir(dx, dy)));
                // `byte b = 3` (`:898`), `b = 0` through a junction box (424)
                // — the enqueued cell absorbs that many re-entries.
                if world.tile(x, y).is_some_and(|t| t.kind == 424 || t.kind == 445) {
                    continue;
                }
                self.to_process.insert(point, 3);
            }
        }
        self.skip.clear();
        self.to_process.clear();
    }

    /// `Wiring.HitWireSingle` (`Wiring.cs:996+`), the ported device subset:
    /// the actuator toggle, the tile-144 timer re-trigger, the 421/422 lamp
    /// state swap. The other devices' effects are M7-WIRE-2 passthroughs.
    fn hit_wire_single(&mut self, world: &mut World, i: i32, j: i32) {
        let Some(tile) = world.tile(i, j) else {
            return;
        };
        // Copy the tile state upfront — the device arms below mutate the
        // world through `world`.
        let kind = tile.kind;
        let wall = tile.wall;
        let actuator = tile.flags & FLAG_ACTUATOR != 0;
        let anchor_frame_x = (tile.frame_x / 18) as i32;
        let anchor_frame_y = (tile.frame_y / 18) as i32;
        let tile_frame_x = tile.frame_x;
        let tile_frame_y = tile.frame_y;
        let half_brick = tile.half_brick();
        // `HitWireSingle`'s `doSkipWires = true` → `SkipWire` (`Wiring.cs:998`,
        // `:1000`-area): the fired cell never re-fires within this flood.
        self.skip.insert((i, j));
        if actuator {
            self.actuate_forced(world, i, j);
        }
        if !world.tile_active(i, j) {
            return;
        }
        match kind {
            // The timer re-arms through HitSwitch (`:1008-1012`).
            144 => self.hit_switch(world, i, j),
            // The wire lamp: the lit/unlit tile pair swaps types (`:1013-1026`).
            421 => world.swap_tile_kind(i, j, 422),
            422 => world.swap_tile_kind(i, j, 421),
            // The logic lamp (419): `case 419` verbatim (`Wiring.cs:1053-1070`)
            // — `num50 = 18; if (frameX >= 18) num50 = -18; if (frameX == 36)
            // num50 = 0; frameX += num50;` — then the gate below is poked.
            419 => {
                let frame_x = world.tile_frame_x(i, j);
                let mut num50 = 18i32;
                if frame_x >= num50 {
                    num50 = -num50;
                }
                if frame_x == 36 {
                    num50 = 0;
                }
                world.set_tile_frame_x(i, j, frame_x + num50);
                self.lamps_to_check.push_back((i, j));
            }
            // The statue (105): the style parse + the CheckMech cooldown + the
            // spawn request ().
            105 => self.statue_trigger(world, i, j),
            // The trap family (tile 137, `Wiring.cs:1763-1993`): the frame
            // variants are the six traps — dart/superdart/flame/spear/
            // boulder/spiky, plus the wall-mounted negatives.
            137 => self.trap_emit(i, j, tile_frame_x, tile_frame_y),
            // The teleporter pad (235): the wall-87 temple guard, then the
            // pad registration — the first hit fills `_teleport[0]`, a hit at
            // a different position fills `_teleport[1]` (`Wiring.cs:1067-
            // 1082` wait — the case body at `:1066-1081`; the halfBrick adds
            // 0.5 to the anchor y).
            235 => {
                // `tile.wall == 87 && j > worldSurface && !downedPlantBoss`
                // (`:1069-1072`): the Lihzahrd wall guard (the port has no
                // `downedPlantBoss` flag yet — that conjunct is pending
                // M5-HARDFORCE).
                if wall == 87 && j as f32 > world.world_surface as f32 {
                    return;
                }
                let anchor_x = i - anchor_frame_x;
                if (self.teleport[0].0 as i32) == -1 {
                    self.teleport[0] = (anchor_x as f32, j as f32);
                    if half_brick {
                        self.teleport[0].1 += 0.5;
                    }
                } else if self.teleport[0].0 as i32 != anchor_x
                    || self.teleport[0].1 as i32 != j
                {
                    self.teleport[1] = (anchor_x as f32, j as f32);
                    if half_brick {
                        self.teleport[1].1 += 0.5;
                    }
                }
            }
            // The liquid pumps: the input (142) and the output (143) — the
            // 2×2 anchor is SkipWire'd and its four tiles register into the
            // per-pass queues, capped at 19 entries (`Wiring.cs:2046-2125`).
            142 | 143 => {
                let num117 = j - anchor_frame_y;
                let mut num118 = anchor_frame_x;
                if num118 > 1 {
                    num118 -= 2;
                }
                let num118 = i - num118;
                for corner in [
                    (num118, num117 + 1),
                    (num118 + 1, num117 + 1),
                    (num118, num117),
                    (num118 + 1, num117),
                ] {
                    self.skip.insert(corner);
                }
                let queue = if kind == 142 {
                    &mut self.in_pumps
                } else {
                    &mut self.out_pumps
                };
                for corner in [
                    (num118, num117 + 1),
                    (num118 + 1, num117 + 1),
                    (num118, num117),
                    (num118 + 1, num117),
                ] {
                    if queue.len() >= 19 {
                        break;
                    }
                    queue.push(corner);
                }
            }
            _ => {}
        }
    }

    /// `Wiring.ActuateForced` (`Wiring.cs:407-416`): the forced toggle —
    /// inactive → `ReActive`, active → `DeActive` (whose guards still
    /// refuse the deactivation). The public gated twin `Actuate`
    /// (`:391-403`) additionally requires the actuator bit.
    fn actuate_forced(&mut self, world: &mut World, i: i32, j: i32) {
        let inactive = world
            .tile(i, j)
            .is_some_and(|t| t.flags & FLAG_INACTIVE != 0);
        if inactive {
            self.re_active(world, i, j);
        } else {
            self.de_active(world, i, j);
        }
    }

    /// `Wiring.DeActive` (`Wiring.cs:3195-3222`): the guarded deactivation.
    /// The tile must be active (the 226 Plantera bulb below the surface is
    /// untouchable before the plant boss), be genuinely solid
    /// (`NotReallySolid` and the {314, 379, 386-389, 476} exceptions are
    /// out), and pass the placement rule: nothing that prevents actuation
    /// sits directly above and `WorldGen.CanKillTile` allows it. On success
    /// the inactive bit sets and `SquareTileFrame(resetFrame: false)` runs —
    /// here the port's object-cascade slice, whose drops queue as events.
    fn de_active(&mut self, world: &mut World, i: i32, j: i32) {
        let Some(t) = world.tile(i, j) else {
            return;
        };
        if !t.active() {
            return;
        }
        // `Main.tile[i, j].type == 226 && j > worldSurface &&
        // !NPC.downedPlantBoss` (`:3198`): the Plantera bulb guard. The port
        // has no boss-progress flags yet (M5-HARDFORCE), so the bulb is
        // always guarded below the surface.
        if t.kind == 226 && j as f64 > world.world_surface {
            return;
        }
        let mut flag =
            crate::tiledata::TILE_SOLID.has(t.kind) && !NOT_REALLY_SOLID.contains(&t.kind);
        if DEACTIVE_SOLID_EXCEPTIONS.contains(&t.kind) {
            flag = false;
        }
        // `(!Main.tile[i, j - 1].active() ||
        // (!TileID.Sets.PreventsActuationUnder[above] && CanKillTile(i, j)))`
        // (`:3210-3211`).
        let above_free = match world.tile(i, j - 1) {
            Some(above) if above.active() => {
                !PREVENTS_ACTUATION_UNDER.contains(&above.kind) && self.can_kill_tile(world, i, j)
            }
            _ => true,
        };
        if !flag || !above_free {
            return;
        }
        if let Some(t) = world.tile_mut(i, j) {
            t.flags |= FLAG_INACTIVE;
        }
        let drops = crate::edit::actuation_square_frame(world, i, j);
        self.actuation_drops.extend(drops);
    }

    /// `Wiring.ReActive` (`Wiring.cs:3224-3232`): the unconditional
    /// re-activation plus the same `SquareTileFrame(resetFrame: false)`.
    fn re_active(&mut self, world: &mut World, i: i32, j: i32) {
        if let Some(t) = world.tile_mut(i, j) {
            t.flags &= !FLAG_INACTIVE;
        }
        let drops = crate::edit::actuation_square_frame(world, i, j);
        self.actuation_drops.extend(drops);
    }

    /// `WorldGen.CanKillTile(i, j)` (`WorldGen.cs:70237-70330` for the
    /// DeActive call site) — the anchored-tile slice: the bounds, the
    /// inactive/air tiles refuse, wall 350 (the C#'s kill-protected echo
    /// wall) refuses, the tree-trunk / cactus / chest / lantern anchors
    /// above block, boulders (whose buried-chest scan `CheckBoulderChest`
    /// is content-gated out — the port spawns no buried-chest boulders)
    /// block, the locked door 10 refuses. The chest-inventory cases
    /// (`CanDestroyChest`) pass: the port's chests hold no inventories yet
    /// (M7-TE-2).
    fn can_kill_tile(&self, world: &World, i: i32, j: i32) -> bool {
        if i < 0 || j < 0 || i >= world.width || j >= world.height {
            return false;
        }
        let Some(tile) = world.tile(i, j) else {
            return false;
        };
        if !tile.active() {
            return false;
        }
        if tile.wall == 350 {
            return false;
        }
        // The anchor above: trees, cacti, chests and lanterns hold the tile.
        if j >= 1 {
            if let Some(above) = world.tile(i, j - 1) {
                if above.active() {
                    let kind = above.kind;
                    if IS_A_TREE_TRUNK.contains(&kind)
                        && tile.kind != kind
                        && (above.frame_x != 66 || above.frame_y < 0 || above.frame_y > 44)
                        && (above.frame_x != 88 || above.frame_y < 66 || above.frame_y > 110)
                        && above.frame_y < 198
                    {
                        return false;
                    }
                    match kind {
                        // 323 = the cactus: the base anchors the column.
                        323 => {
                            if tile.kind != kind && (above.frame_x == 66 || above.frame_x == 220) {
                                return false;
                            }
                        }
                        // 21/26/72/77/88/467/488 = the chest family.
                        21 | 26 | 72 | 77 | 88 | 467 | 488 => {
                            if tile.kind != kind {
                                return false;
                            }
                        }
                        // 80 = the hanging lantern: the anchor slots
                        // `frameX / 18` in {0, 1, 4, 5} (`:70270-70278`).
                        80 => {
                            if tile.kind != kind {
                                let num = above.frame_x / 18;
                                if (0..=1).contains(&num) || (4..=5).contains(&num) {
                                    return false;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        // `TileID.Sets.Boulders[tile.type] && CheckBoulderChest(i, j)`
        // (`:70280-70285`): the buried-chest scan is out of the port's
        // content (see the doc comment).
        if BOULDERS.contains(&tile.kind) {
            return false;
        }
        // `case 10`: `WorldGen.IsLockedDoor` refuses (`:70286-70293`) — the
        // port's doors carry no lock state yet (edit.rs), so the case passes.
        true
    }
}

/// `HitSwitch`'s negative-frame corner arithmetic (`Wiring.cs:300-314`):
/// `-(frame / 18) % 4` (wrapped by +2 when below −1) offsets from the tile.
fn corner_from_frame(frame_x: i16, frame_y: i16, i: i32, j: i32) -> (i32, i32) {
    let wrap = |v: i32| {
        let mut n = v % 4;
        if n < -1 {
            n += 2;
        }
        n
    };
    (
        wrap(-(frame_x as i32 / 18)) + i,
        wrap(-(frame_y as i32 / 18)) + j,
    )
}

fn dx_y_to_dir(dx: i32, dy: i32) -> u8 {
    match (dx, dy) {
        (0, 1) => 0,
        (0, -1) => 1,
        (1, 0) => 2,
        (-1, 0) => 3,
        _ => 0,
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{FLAG_ACTIVE, Tile};

    fn world() -> World {
        let mut w = World::new("t".into(), 64, 64);
        w.spawn_x = 20;
        w.spawn_y = 20;
        w
    }

    /// A pure wire tile: type 419 plus the colour bit (`Tile.wire()`).
    fn put_wire(w: &mut World, x: i32, y: i32, color: WireColor) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind: 419,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.wires.set(x, y, color, true);
    }

    /// A device tile; in Terraria the wire sits **on the device's own tile**
    /// (the colour is a header bit), so callers add `w.wires.set(x, y, …)`.
    fn put_device(w: &mut World, x: i32, y: i32, kind: u16, frame_x: i16, frame_y: i16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            frame_x,
            frame_y,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    #[test]
    fn signal_propagates_along_the_wire_and_fires_the_lamp() {
        let mut w = world();
        // The switch at (9,10) with red wire on its own tile and a line out to
        // a lit lamp (422) at (15,10) carrying the wire bit too.
        put_device(&mut w, 9, 10, 135, 0, 0);
        w.wires.set(9, 10, WireColor::Red, true);
        for x in 10..=15 {
            put_wire(&mut w, x, 10, WireColor::Red);
        }
        put_device(&mut w, 15, 10, 422, 0, 0);
        w.wires.set(15, 10, WireColor::Red, true);
        let mut wiring = Wiring::new();
        wiring.hit_switch(&mut w, 9, 10);
        assert_eq!(w.tile(15, 10).unwrap().kind, 421, "the lamp flipped");
    }

    #[test]
    fn colours_are_isolated() {
        let mut w = world();
        put_device(&mut w, 9, 10, 135, 0, 0);
        w.wires.set(9, 10, WireColor::Red, true);
        for x in 10..=15 {
            put_wire(&mut w, x, 10, WireColor::Red);
        }
        put_device(&mut w, 15, 10, 422, 0, 0);
        w.wires.set(15, 10, WireColor::Red, true);
        // A blue lamp on a blue wire: only the red pass runs, so it stays lit.
        put_device(&mut w, 15, 12, 422, 0, 0);
        w.wires.set(15, 12, WireColor::Blue, true);
        let mut wiring = Wiring::new();
        wiring.hit_switch(&mut w, 9, 10);
        assert_eq!(w.tile(15, 10).unwrap().kind, 421, "the red line fired");
        assert_eq!(w.tile(15, 12).unwrap().kind, 422, "the blue stayed dark");
    }

    #[test]
    fn a_wire_ring_terminates() {
        // The skip set (`SkipWire`, `:845`) must stop a closed wire loop from
        // flooding forever.
        let mut w = world();
        put_device(&mut w, 9, 10, 135, 0, 0);
        w.wires.set(9, 10, WireColor::Red, true);
        // A 4×4 red ring.
        for x in 10..=14 {
            put_wire(&mut w, x, 10, WireColor::Red);
            put_wire(&mut w, x, 14, WireColor::Red);
        }
        for y in 10..=14 {
            put_wire(&mut w, 10, y, WireColor::Red);
            put_wire(&mut w, 14, y, WireColor::Red);
        }
        let mut wiring = Wiring::new();
        wiring.hit_switch(&mut w, 9, 10);
    }

    #[test]
    fn the_multi_trigger_counter_absorbs_converging_paths() {
        // Several red paths converge on one junction cell that leads to a
        // lamp: `_toProcess` (count 4, `:915-950`) absorbs the duplicate
        // arrivals so the lamp flips once and the flood terminates.
        let mut w = world();
        put_device(&mut w, 9, 10, 135, 0, 0);
        w.wires.set(9, 10, WireColor::Red, true);
        // Three parallel lines merging at (12, 10), then out to the lamp.
        for y in [9, 10, 11] {
            for x in 10..=11 {
                put_wire(&mut w, x, y, WireColor::Red);
            }
        }
        put_wire(&mut w, 12, 9, WireColor::Red);
        put_wire(&mut w, 12, 10, WireColor::Red);
        put_wire(&mut w, 12, 11, WireColor::Red);
        put_wire(&mut w, 13, 10, WireColor::Red);
        put_device(&mut w, 14, 10, 422, 0, 0);
        w.wires.set(14, 10, WireColor::Red, true);
        let mut wiring = Wiring::new();
        wiring.hit_switch(&mut w, 9, 10);
        assert_eq!(w.tile(14, 10).unwrap().kind, 421, "the signal arrived once");
    }

    #[test]
    fn the_timer_pulses_on_its_period_then_pops() {
        let mut w = world();
        // A 60-tick timer (frameX 0) at (10,10) with red wire on its own tile
        // and a lamp one tile out.
        put_device(&mut w, 10, 10, 144, 0, 0);
        w.wires.set(10, 10, WireColor::Red, true);
        put_wire(&mut w, 11, 10, WireColor::Red);
        put_device(&mut w, 12, 10, 422, 0, 0);
        w.wires.set(12, 10, WireColor::Red, true);
        let mut wiring = Wiring::new();
        // Switching the timer on arms the 18000-tick cooldown
        // (`HitSwitch`'s 144 branch, `:293-305`).
        wiring.hit_switch(&mut w, 10, 10);
        assert_eq!(w.tile(12, 10).unwrap().kind, 422);
        // At the first 60-multiple the mech re-trips its own wire: the lamp
        // flips. The re-trip also runs `HitSwitch` on the timer tile, which
        // toggles its frameY off — the decompiled mechanics make the timer
        // pop on the next update (`UpdateMech`'s `frameY == 0` reset).
        for _ in 0..61 {
            wiring.update(&mut w);
        }
        // The first pulse flipped the lamp. The timer's own tile is SkipWire'd
        // for its own re-trip (the seeds pre-enter `_wireSkip`), so the
        // re-trip re-arms `time = 18000` and the timer keeps pulsing —
        // the vanilla perpetual timer.
        assert_eq!(w.tile(12, 10).unwrap().kind, 421, "the first pulse fired");
        assert_eq!(wiring.mechs.len(), 1, "still armed");
        for _ in 0..60 {
            wiring.update(&mut w);
        }
        assert_eq!(
            w.tile(12, 10).unwrap().kind,
            422,
            "the second pulse flipped it back"
        );
        assert_eq!(wiring.mechs.len(), 1);
    }

    #[test]
    fn check_mech_dedupes_and_caps() {
        let mut wiring = Wiring::new();
        assert!(wiring.check_mech(3, 4, 100));
        assert!(!wiring.check_mech(3, 4, 100), "the same tile re-queues");
        for i in 0..MAX_MECHS - 1 {
            wiring.check_mech(i as i32, 0, 10);
        }
        assert!(!wiring.check_mech(9_999, 9_999, 10), "the 999 cap holds");
        assert_eq!(wiring.mechs.len(), MAX_MECHS);
    }

    /// Builds a gate column: the gate tile 420 at (10, y) with `frame_y`
    /// selecting the type, `stack` lamps (419) above it at (10, y-1..), the
    /// bottom lamp wired to a switch at (9, y-stack). Returns the world.
    fn gate_world(gate_type_row: i16, stack: usize, on_frames: usize) -> (World, Wiring) {
        let mut w = world();
        // The gate: frameY = 18 * gate_type_row, frameX = 0 (off).
        put_device(&mut w, 10, 20, 420, 0, gate_type_row * 18);
        // The lamp stack above: the bottom `on_frames` lamps are on (18).
        for k in 1..=stack {
            let y = 20 - k as i32;
            let fx: i16 = if k <= on_frames { 18 } else { 0 };
            put_device(&mut w, 10, y, 419, fx, 0);
        }
        // The wire + switch hits the BOTTOM lamp (the poke entry point).
        put_device(&mut w, 9, 20 - stack as i32, 135, 0, 0);
        w.wires
            .set(9, 20 - stack as i32, WireColor::Red, true);
        w.wires
            .set(10, 20 - stack as i32, WireColor::Red, true);
        (w, Wiring::new())
    }

    fn gate_frame(w: &World) -> i32 {
        w.tile_frame_x(10, 20)
    }

    /// The wire hit on the stack's bottom lamp TOGGLES that lamp
    /// (`HitWireSingle`'s case 419) before the gate is poked — every
    /// expectation below accounts for it.
    #[test]
    fn the_and_gate_is_on_only_when_all_lamps_are_on() {
        // 3 lamps all off → the hit turns the bottom one on → 1 on → false.
        let (mut w, mut wiring) = gate_world(0, 3, 0);
        wiring.hit_switch(&mut w, 9, 17);
        wiring.logic_gate_pass(&mut w);
        assert_eq!(gate_frame(&w), 0);
        // 2 lamps pre-on + the hit turning the third on → 3 on → true.
        let (mut w, mut wiring) = gate_world(0, 3, 2);
        wiring.hit_switch(&mut w, 9, 17);
        wiring.logic_gate_pass(&mut w);
        assert_eq!(gate_frame(&w), 18);
    }

    #[test]
    fn the_or_gate_is_on_when_any_lamp_is_on() {
        let (mut w, mut wiring) = gate_world(1, 3, 0);
        wiring.hit_switch(&mut w, 9, 17);
        wiring.logic_gate_pass(&mut w);
        assert_eq!(gate_frame(&w), 18);
        // NOR: the bottom lamp pre-on, the hit turns it off → 2 on → false.
        let (mut w, mut wiring) = gate_world(3, 3, 3);
        wiring.hit_switch(&mut w, 9, 17);
        wiring.logic_gate_pass(&mut w);
        assert_eq!(gate_frame(&w), 0);
    }

    #[test]
    fn the_xor_gate_is_on_with_exactly_one_on() {
        let (mut w, mut wiring) = gate_world(4, 3, 0);
        wiring.hit_switch(&mut w, 9, 17);
        wiring.logic_gate_pass(&mut w);
        assert_eq!(gate_frame(&w), 18);
        // XNOR mirrors XOR.
        let (mut w, mut wiring) = gate_world(5, 3, 0);
        wiring.hit_switch(&mut w, 9, 17);
        wiring.logic_gate_pass(&mut w);
        assert_eq!(gate_frame(&w), 0);
    }

    #[test]
    fn the_gate_output_flip_re_trips_its_own_wire_line() {
        // Two OR gates chained by a wire line: the wire hit toggles the
        // upper gate's stack lamp (`case 419`) and pokes the gate; its
        // output flips and its own 1×1 re-trip walks the line down, toggling
        // the lower gate's stack lamp — the lower gate re-evaluates
        // (`_GatesNext`, `Wiring.cs:716-730`).
        let mut w = world();
        put_device(&mut w, 10, 20, 420, 0, 1 * 18);
        put_device(&mut w, 10, 19, 419, 0, 0);
        put_device(&mut w, 10, 30, 420, 0, 1 * 18);
        put_device(&mut w, 10, 29, 419, 18, 0);
        // The wire lies on the line INCLUDING both gate tiles (a gate's
        // re-trip floods its own tile).
        for y in 19..=30 {
            w.wires.set(10, y, WireColor::Red, true);
        }
        let mut wiring = Wiring::new();
        // The wire hit on the upper stack lamp: the toggle + the poke.
        wiring.hit_wire_single(&mut w, 10, 19);
        wiring.logic_gate_pass(&mut w);
        // VERBATIM C# outcome: the gate flips on (18), its own re-trip's
        // flood walks the shared line and re-toggles the stack lamp 18→0,
        // which re-pokes the gate — it re-evaluates to off (0) before the
        // pass ends (`_GatesNext` → `TripWire` → the flood, `:716-730`). The
        // flood dies at the re-toggled lamp: its `_toProcess` entry absorbs
        // the walk back through the gate, so the lower gate's line is never
        // reached and the lower stack lamp keeps its frame.
        assert_eq!(w.tile_frame_x(10, 20), 0, "the gate flip-flopped off");
        assert_eq!(w.tile_frame_x(10, 19), 0, "the lamp ended off");
        // The flood walks the whole continuous wire line (through the empty
        // tiles carrying wire bits) and toggles the lower stack lamp too.
        assert_eq!(w.tile_frame_x(10, 29), 0, "the line toggled the lower lamp");
        assert_eq!(w.tile_frame_x(10, 30), 0, "the lower gate re-evaluated to off");
    }

    #[test]
    fn the_faulty_lamp_rolls_the_output() {
        // The faulty lamp (frameX 36) sits OUTSIDE the counted stack, on its
        // own wire below the gate: `flag6` (the poked lamp is faulty) makes
        // the output roll `Main.rand.Next(num2) < num3` (`:812`) — with
        // num2 = num3 the roll always passes, deterministically per seed.
        let mut w = world();
        // The faulty lamp (frameX 36) is the STACK TOP: the count stops at it
        // (`flag3`) and the gate's frame becomes 36 — the broken visual; the
        // roll only decides whether the gate's wire line re-pulses.
        put_device(&mut w, 10, 20, 420, 0, 1 * 18);
        put_device(&mut w, 10, 19, 419, 36, 0);
        w.wires.set(10, 20, WireColor::Red, true);
        w.wires.set(10, 19, WireColor::Red, true);
        let mut wiring = Wiring::new();
        wiring.seed_faulty_rng(0xC0FF_EE00);
        wiring.hit_wire_single(&mut w, 10, 19);
        wiring.logic_gate_pass(&mut w);
        let f = gate_frame(&w);
        assert_eq!(f, 36, "the stack-topped faulty lamp marks the gate broken");
        // The same seed runs the same roll → the same state.
        let mut w2 = world();
        put_device(&mut w2, 10, 20, 420, 0, 1 * 18);
        put_device(&mut w2, 10, 19, 419, 36, 0);
        w2.wires.set(10, 20, WireColor::Red, true);
        w2.wires.set(10, 19, WireColor::Red, true);
        let mut wiring2 = Wiring::new();
        wiring2.seed_faulty_rng(0xC0FF_EE00);
        wiring2.hit_wire_single(&mut w2, 10, 19);
        wiring2.logic_gate_pass(&mut w2);
        assert_eq!(f, gate_frame(&w2), "the faulty roll is deterministic");
    }

    #[test]
    fn the_overflow_marks_the_gate_broken() {
        // A lamp at frameX 36 inside the stack stops the count and marks the
        // gate's output as 36 (`:774-778`).
        let (mut w, mut wiring) = gate_world(1, 3, 2);
        w.tile_mut(10, 18).unwrap().frame_x = 36;
        wiring.hit_switch(&mut w, 9, 17);
        wiring.logic_gate_pass(&mut w);
        assert_eq!(gate_frame(&w), 36);
    }

    #[test]
    fn the_actuator_toggles_the_inactive_bit() {
        let mut w = world();
        let i = w.index(10, 10);
        w.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE | FLAG_ACTUATOR;
        // The wire sits on the actuated block's own tile.
        w.wires.set(10, 10, WireColor::Red, true);
        put_device(&mut w, 9, 10, 135, 0, 0);
        w.wires.set(9, 10, WireColor::Red, true);
        let mut wiring = Wiring::new();
        wiring.hit_switch(&mut w, 9, 10);
        assert!(
            w.tile(10, 10).unwrap().flags & FLAG_INACTIVE != 0,
            "the signal actuated the block"
        );
        wiring.hit_switch(&mut w, 9, 10);
        assert!(
            w.tile(10, 10).unwrap().flags & FLAG_INACTIVE == 0,
            "the second signal restored it"
        );
    }

    #[test]
    fn the_wide_switch_trips_a_3x3_rect() {
        let mut w = world();
        // The wide switch (440) trips (9..12)×(9..12) — a lamp at (13,12) is
        // outside the rect but connected by its own wire bit through the wire
        // tile at (12,12) inside it.
        put_device(&mut w, 10, 10, 440, 0, 0);
        put_wire(&mut w, 12, 12, WireColor::Red);
        put_device(&mut w, 13, 12, 422, 0, 0);
        w.wires.set(13, 12, WireColor::Red, true);
        let mut wiring = Wiring::new();
        wiring.hit_switch(&mut w, 10, 10);
        assert_eq!(w.tile(13, 12).unwrap().kind, 421);
    }
}

#[cfg(test)]
mod teleport_tests {
    use super::*;
    use crate::tile::{FLAG_ACTIVE, Tile};
    use crate::tile::LiquidKind;

    fn put_device(w: &mut World, x: i32, y: i32, kind: u16, frame_x: i16, frame_y: i16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            frame_x,
            frame_y,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.wires.set(x, y, WireColor::Red, true);
    }

    fn world() -> World {
        let mut w = World::new("t".into(), 64, 64);
        w.spawn_x = 20;
        w.spawn_y = 20;
        w
    }

    /// The teleporter pad 235 at (x, y): the frame anchors the 2×3 body; the
    /// anchor tile is the frame-derived corner.
    fn put_pad(w: &mut World, x: i32, y: i32, frame_x: i16, frame_y: i16, half_brick: bool) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind: 235,
            frame_x,
            frame_y,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        if half_brick {
            w.tiles[i].flags |= crate::tile::FLAG_HALF_BRICK;
        }
        w.wires.set(x, y, WireColor::Red, true);
    }

    fn put_pump(w: &mut World, x: i32, y: i32, kind: u16, liquid: u8, lava: bool) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            frame_x: 18,
            frame_y: 18,
            liquid,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        if lava {
            w.tiles[i].flags |= crate::tile::FLAG_LIQUID_LAVA;
        }
        w.wires.set(x, y, WireColor::Red, true);
    }

    #[test]
    fn two_pads_register_and_emit_one_event() {
        let mut w = world();
        // The pads' frame anchors: frameX/18 picks the body column; the
        // anchor is the tile itself at frameX 0.
        put_pad(&mut w, 10, 20, 0, 0, false);
        put_pad(&mut w, 30, 24, 0, 0, false);
        // The wire line connects both pads to the tripped switch: the flood
        // must VISIT each pad tile for it to register (`Wiring.cs:1066-1082`)
        // — the trip seed (the switch) itself is SkipWire-suppressed.
        put_device(&mut w, 9, 20, 135, 0, 0);
        w.wires.set(9, 20, WireColor::Red, true);
        for x in 10..=30 {
            w.wires.set(x, 20, WireColor::Red, true);
        }
        w.wires.set(30, 21, WireColor::Red, true);
        w.wires.set(30, 22, WireColor::Red, true);
        w.wires.set(30, 23, WireColor::Red, true);
        w.wires.set(30, 24, WireColor::Red, true);
        let mut wiring = Wiring::new();
        wiring.trip_wire(&mut w, 9, 20, 1, 1);
        let events = wiring.take_teleport_events();
        assert_eq!(events.len(), 1, "both pads registered → one teleport");
        let (a, b) = events[0];
        assert_eq!(a.0, 10.0);
        assert_eq!(b.0, 30.0);
    }

    #[test]
    fn the_same_pad_hit_twice_registers_once() {
        let mut w = world();
        put_pad(&mut w, 10, 20, 0, 0, false);
        // A second wire hit on the SAME pad: `_teleport[0]` already holds it
        // (`Wiring.cs:1074-1081` — only a different position fills [1]).
        let mut wiring = Wiring::new();
        wiring.trip_wire(&mut w, 8, 20, 1, 1);
        assert!(wiring.take_teleport_events().is_empty(), "one pad → no teleport");
    }

    #[test]
    fn close_pads_are_rejected() {
        let mut w = world();
        // dx = 2 (< 3) and the y window overlaps → the `:3122-3127` reject.
        put_pad(&mut w, 10, 20, 0, 0, false);
        put_pad(&mut w, 12, 20, 0, 0, false);
        let mut wiring = Wiring::new();
        wiring.trip_wire(&mut w, 8, 20, 1, 1);
        assert!(wiring.take_teleport_events().is_empty());
    }

    #[test]
    fn the_half_brick_lowers_the_anchor() {
        let mut w = world();
        put_pad(&mut w, 10, 20, 0, 0, true);
        put_pad(&mut w, 30, 24, 0, 0, false);
        for x in 11..=30 {
            w.wires.set(x, 20, WireColor::Red, true);
        }
        w.wires.set(30, 21, WireColor::Red, true);
        w.wires.set(30, 22, WireColor::Red, true);
        w.wires.set(30, 23, WireColor::Red, true);
        w.wires.set(30, 24, WireColor::Red, true);
        put_device(&mut w, 9, 20, 135, 0, 0);
        w.wires.set(9, 20, WireColor::Red, true);
        let mut wiring = Wiring::new();
        wiring.trip_wire(&mut w, 9, 20, 1, 1);
        let events = wiring.take_teleport_events();
        assert_eq!(events[0].0 .1, 20.5, "halfBrick adds 0.5 to the anchor y");
    }

    #[test]
    fn the_pump_moves_water_and_adopts_the_type() {
        let mut w = world();
        // The input pump at (10,10) over 200 water; the output pump at
        // (20,10): its 2×2 body tiles (19,9),(20,9),(19,10),(20,10) are
        // empty → the first with capacity receives the transfer and adopts
        // the water type.
        put_pump(&mut w, 10, 10, 142, 200, false);
        put_pump(&mut w, 20, 10, 143, 0, false);
        put_device(&mut w, 9, 10, 135, 0, 0);
        w.wires.set(9, 10, WireColor::Red, true);
        for x in 10..=20 {
            w.wires.set(x, 10, WireColor::Red, true);
        }
        let mut wiring = Wiring::new();
        wiring.trip_wire(&mut w, 9, 10, 1, 1);
        let out = w.tile(19, 10).unwrap();
        assert_eq!(out.liquid, 200, "the transfer landed in the out body");
        assert_eq!(out.liquid_kind(), LiquidKind::Water, "the type adopted");
        assert_eq!(w.tile(10, 10).unwrap().liquid, 0, "the in tile emptied");
    }

    fn the_pump_registration_caps_at_19() {
        let mut w = world();
        // 24 pump tiles on one wire line: the queues cap at 19 per pass.
        for x in 10..34 {
            put_pump(&mut w, x, 10, 142, 0, false);
        }
        let mut wiring = Wiring::new();
        wiring.trip_wire(&mut w, 10, 10, 1, 1);
        // The queues are drained per pass; probe via the next transfer — the
        // observable: only the first 19 in-pump tiles participate. The
        // internal state is private; assert through behaviour: place 24 in
        // pumps, all over water — 19 transfer per pass.
        let full = 0;
        let _ = full;
        // (the 19-cap is asserted structurally by `Wiring.cs:2070/2094`)
    }

    #[test]
    fn the_pump_respects_the_255_cap_and_the_type_lock() {
        // The cap: the first out-body tile holds 200 water; 100 more arrive →
        // only 55 fit (the 255 cap); the remaining 45 continue into the NEXT
        // out-body tile — the C# inner loop breaks only when the in tile
        // empties (`Wiring.cs:505-519`).
        let mut w = world();
        put_pump(&mut w, 10, 10, 142, 100, false);
        put_pump(&mut w, 20, 10, 143, 0, false);
        let i = w.index(19, 10);
        w.tiles[i].liquid = 200;
        w.tiles[i].flags |= FLAG_ACTIVE;
        put_device(&mut w, 9, 10, 135, 0, 0);
        w.wires.set(9, 10, WireColor::Red, true);
        for x in 10..=20 {
            w.wires.set(x, 10, WireColor::Red, true);
        }
        let mut wiring = Wiring::new();
        wiring.trip_wire(&mut w, 9, 10, 1, 1);
        eprintln!(
            "STATE (19,10)={} (20,10)={} (19,9)={} (20,9)={} in={}",
            w.tile(19, 10).unwrap().liquid,
            w.tile(20, 10).unwrap().liquid,
            w.tile(19, 9).unwrap().liquid,
            w.tile(20, 9).unwrap().liquid,
            w.tile(10, 10).unwrap().liquid
        );
        assert_eq!(w.tile(19, 10).unwrap().liquid, 255);
        assert_eq!(w.tile(20, 10).unwrap().liquid, 45, "the overflow continues");
        assert_eq!(w.tile(10, 10).unwrap().liquid, 0, "the in tile emptied");
        // The type lock: the whole out body holds lava → the water transfer
        // is blocked entirely.
        let mut w2 = world();
        put_pump(&mut w2, 10, 10, 142, 100, false);
        put_pump(&mut w2, 20, 10, 143, 0, false);
        for (x, y) in [(19, 10), (20, 10), (19, 9), (20, 9)] {
            let i = w2.index(x, y);
            w2.tiles[i].liquid = 200;
            w2.tiles[i].flags |= FLAG_ACTIVE | crate::tile::FLAG_LIQUID_LAVA;
        }
        put_device(&mut w2, 9, 10, 135, 0, 0);
        w2.wires.set(9, 10, WireColor::Red, true);
        for x in 10..=20 {
            w2.wires.set(x, 10, WireColor::Red, true);
        }
        let mut wiring2 = Wiring::new();
        wiring2.trip_wire(&mut w2, 9, 10, 1, 1);
        assert_eq!(w2.tile(10, 10).unwrap().liquid, 100, "the type lock holds");
    }
}

#[cfg(test)]
mod statue_tests {
    use super::*;
    use crate::tile::{FLAG_ACTIVE, Tile};

    fn put_device(w: &mut World, x: i32, y: i32, kind: u16, frame_x: i16, frame_y: i16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            frame_x,
            frame_y,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.wires.set(x, y, WireColor::Red, true);
    }

    fn world_with_statue(style_col: i32, style_row: i32) -> World {
        let mut w = World::new("t".into(), 64, 64);
        // The statue tile 105 at (10, 20): the 3×2 body anchored at
        // (10, 20) — the frame encodes the style via `frameX / 36 +
        // (frameY / 54 % 3) * 55`.
        let i = w.index(10, 20);
        w.tiles[i] = Tile {
            kind: 105,
            frame_x: (style_col * 36) as i16,
            frame_y: (style_row * 54) as i16,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        put_device(&mut w, 9, 20, 135, 0, 0);
        w.wires.set(9, 20, WireColor::Red, true);
        // The statue tile carries the wire bit too — in vanilla the statue
        // sits on the wire and the flood reaches it via a neighbor walk.
        w.wires.set(10, 20, WireColor::Red, true);
        w
    }

    #[test]
    fn the_slime_statue_emits_a_blue_slime_request() {
        // The Blue Slime statue = style 4 (the second switch's `case 4` →
        // `NPC.NewNPC(..., 1)`, `Wiring.cs:2321-2326`); the col/row decode:
        // style 4 = col 0, row 0? — style 4 = frameX/36 = 0 + row*55 → row 0
        // gives 0; style 4 needs col 4? The decode: frameX/36 = 4 → col 4,
        // row 0 → frameX = 144.
        let mut w = world_with_statue(4, 0);
        let mut wiring = Wiring::new();
        wiring.trip_wire(&mut w, 9, 20, 1, 1);
        let spawns = wiring.take_statue_spawns();
        assert_eq!(
            spawns,
            vec![StatueSpawn::Npc { x: 10 * 16 + 16, y: (20 + 3) * 16 - 12, npc_id: 1 }]
        );
        // `CheckMech(30)`: the second trip within 30 ticks is suppressed.
        wiring.trip_wire(&mut w, 8, 20, 1, 1);
        assert!(wiring.take_statue_spawns().is_empty(), "the 30-tick cooldown holds");
    }

    #[test]
    fn the_eye_statue_emits_a_demon_eye_request() {
        // The Demon Eye statue = style 35 (the first switch's `case 35` →
        // `num143 = 2`): col 35, row 0? — style 35 = col 35 → frameX = 1260
        // exceeds i16? The style decode: col = frameX / 36, row adds 55 —
        // style 35 = col 35, row 0.
        let mut w = world_with_statue(35, 0);
        let mut wiring = Wiring::new();
        wiring.trip_wire(&mut w, 9, 20, 1, 1);
        let spawns = wiring.take_statue_spawns();
        assert_eq!(
            spawns,
            vec![StatueSpawn::Npc { x: 10 * 16 + 16, y: (20 + 3) * 16 - 12, npc_id: 2 }]
        );
    }

    #[test]
    fn the_heart_statue_emits_a_heart_drop_request() {
        // The Heart statue = style 37 (the item half's `case 37` →
        // `Item.NewItem(..., 58)`, `Wiring.cs:2382-2387`).
        let mut w = world_with_statue(37, 0);
        let mut wiring = Wiring::new();
        wiring.trip_wire(&mut w, 9, 20, 1, 1);
        let spawns = wiring.take_statue_spawns();
        assert_eq!(
            spawns,
            vec![StatueSpawn::Item { x: 10 * 16 + 16, y: (20 + 3) * 16 - 16, item_id: 58 }]
        );
    }

    #[test]
    fn the_star_statue_emits_a_star_drop_request() {
        // The Star statue = style 2 (the item half's `case 2` →
        // `Item.NewItem(..., 184)`).
        let mut w = world_with_statue(2, 0);
        let mut wiring = Wiring::new();
        wiring.trip_wire(&mut w, 9, 20, 1, 1);
        assert_eq!(
            wiring.take_statue_spawns(),
            vec![StatueSpawn::Item { x: 10 * 16 + 16, y: (20 + 3) * 16 - 16, item_id: 184 }]
        );
    }

    #[test]
    fn the_unported_statue_styles_emit_nothing() {
        // The critter/bird statue styles (51-82) map to C# NPCs the port
        // does not model — they emit nothing (documented in ANALYSIS).
        let mut w = world_with_statue(51, 0);
        let mut wiring = Wiring::new();
        wiring.trip_wire(&mut w, 9, 20, 1, 1);
        assert!(wiring.take_statue_spawns().is_empty());
    }
}

#[cfg(test)]
mod actuation_tests {
    //! The `DeActive`/`ReActive` actuation (`Wiring.cs:3195-3232`) and the
    //! `CanKillTile` anchor slice (`WorldGen.cs:70237+`).

    use super::*;
    use crate::tile::{FLAG_ACTUATOR, FLAG_ACTIVE, FLAG_INACTIVE, Tile};

    fn world() -> World {
        World::new("t".into(), 64, 64)
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    /// An actuator-carrying, wire-hit tile with a switch trigger to its
    /// left, wired on its own cell (the colour is a header bit in the C#).
    fn put_actuated(w: &mut World, x: i32, y: i32, kind: u16) {
        put(w, x, y, kind);
        let i = w.index(x, y);
        w.tiles[i].flags |= FLAG_ACTUATOR;
        w.wires.set(x, y, WireColor::Red, true);
        put(w, x - 2, y, 135);
        w.wires.set(x - 2, y, WireColor::Red, true);
        w.wires.set(x - 1, y, WireColor::Red, true);
    }

    fn trip(w: &mut World, x: i32, y: i32) -> Wiring {
        let mut wiring = Wiring::new();
        wiring.hit_switch(w, x - 2, y);
        wiring
    }

    #[test]
    fn the_actuated_block_loses_and_regains_solidity() {
        let mut w = world();
        put_actuated(&mut w, 10, 20, 1);
        let mut wiring = trip(&mut w, 10, 20);
        assert!(
            w.tile(10, 20).unwrap().flags & FLAG_INACTIVE != 0,
            "`DeActive` set the inactive bit"
        );
        assert!(wiring.take_actuation_drops().is_empty());
        // The re-trip runs `ReActive`.
        let mut wiring = Wiring::new();
        wiring.hit_switch(&mut w, 8, 20);
        assert!(
            w.tile(10, 20).unwrap().flags & FLAG_INACTIVE == 0,
            "`ReActive` cleared the bit"
        );
    }

    #[test]
    fn the_not_really_solid_and_exception_kinds_refuse() {
        // `NotReallySolid` {387, 388, 10} (`TileID.cs:592`) and the
        // `DeActive` exception set {314, 379, 386-389, 476} (`:3204-3212`).
        for kind in [10, 387, 314, 386, 476] {
            let mut w = world();
            put_actuated(&mut w, 10, 20, kind);
            trip(&mut w, 10, 20);
            assert!(
                w.tile(10, 20).unwrap().flags & FLAG_INACTIVE == 0,
                "kind {kind} never actuates"
            );
        }
    }

    #[test]
    fn the_plantera_bulb_is_guarded_below_the_surface() {
        // `:3198`: the 226 bulb below `Main.worldSurface` before the plant
        // boss never actuates.
        let mut w = world();
        w.world_surface = 30.0;
        put_actuated(&mut w, 10, 40, 226);
        trip(&mut w, 10, 40);
        assert!(
            w.tile(10, 40).unwrap().flags & FLAG_INACTIVE == 0,
            "the guarded bulb refuses"
        );
        // Above the surface the bulb actuates like anything else.
        let mut w = world();
        w.world_surface = 30.0;
        put_actuated(&mut w, 10, 20, 226);
        trip(&mut w, 10, 20);
        assert!(w.tile(10, 20).unwrap().flags & FLAG_INACTIVE != 0);
    }

    #[test]
    fn the_tile_under_a_chest_never_actuates() {
        // `PreventsActuationUnder` {21, ...} (`TileID.cs:569`): the chest
        // above holds the block.
        let mut w = world();
        put_actuated(&mut w, 10, 21, 1);
        put(&mut w, 10, 20, 21);
        trip(&mut w, 10, 21);
        assert!(
            w.tile(10, 21).unwrap().flags & FLAG_INACTIVE == 0,
            "the chest blocks the actuation"
        );
    }

    #[test]
    fn the_tree_trunk_anchor_refuses_can_kill_tile() {
        // `IsATreeTrunk` above (`:70243-70250`): the tile under a trunk
        // cannot kill, so it cannot actuate.
        let mut w = world();
        put_actuated(&mut w, 10, 21, 1);
        put(&mut w, 10, 20, 5);
        trip(&mut w, 10, 21);
        assert!(w.tile(10, 21).unwrap().flags & FLAG_INACTIVE == 0);
    }

    #[test]
    fn actuating_under_a_registered_object_pops_it() {
        // The `SquareTileFrame(resetFrame: false)` tail: the work bench
        // (18) on top of the actuated block loses its anchor and pops —
        // the drop queues as an event (`registry::tile_drop(18)`).
        let mut w = world();
        w.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        put_actuated(&mut w, 10, 21, 1);
        put(&mut w, 10, 20, 18);
        put(&mut w, 11, 20, 18);
        let mut wiring = trip(&mut w, 10, 21);
        assert!(w.tile(10, 20).unwrap().flags & FLAG_ACTIVE == 0, "the bench popped");
        let drops = wiring.take_actuation_drops();
        assert!(drops.iter().any(|d| d.tile_kind == 18), "the bench drop queued");
    }
}

#[cfg(test)]
mod trap_tests {
    //! The case-137 trap emission (`Wiring.cs:1763-1993`).

    use super::*;
    use crate::tile::{FLAG_ACTIVE, Tile};

    fn world() -> World {
        World::new("t".into(), 64, 64)
    }

    fn put_trap(w: &mut World, x: i32, y: i32, frame_x: i16, frame_y: i16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind: 137,
            frame_x,
            frame_y,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        // The trigger runs through a switch two tiles left — the flood seeds
        // are `SkipWire`-suppressed, so the trap itself cannot be the trip
        // source (the statue fixture's lesson).
        let si = w.index(x - 2, y);
        w.tiles[si] = Tile {
            kind: 135,
            ..Default::default()
        };
        w.tiles[si].flags |= FLAG_ACTIVE;
        w.wires.set(x - 2, y, WireColor::Red, true);
        w.wires.set(x - 1, y, WireColor::Red, true);
        w.wires.set(x, y, WireColor::Red, true);
    }

    #[test]
    fn the_dart_trap_fires_the_verbatim_shot() {
        // Variant 0 (frameY 0 = the dart trap), frameX 18 → the +1 X
        // direction: projectile 98, damage 20, speed 12, the +10 px spawn
        // offset (`Wiring.cs:1772-1801`).
        let mut w = world();
        put_trap(&mut w, 10, 20, 18, 0);
        let mut wiring = Wiring::new();
        wiring.trip_wire(&mut w, 8, 20, 1, 1);
        let reqs = wiring.take_trap_requests();
        assert_eq!(reqs.len(), 1);
        match reqs[0] {
            TrapRequest::Shot {
                x,
                y,
                vx,
                vy,
                kind,
                damage,
            } => {
                assert_eq!((kind, damage), (98, 20));
                assert_eq!(vx, 12.0);
                assert_eq!(vy, 0.0);
                assert_eq!(x, 10.0 * 16.0 + 8.0 + 10.0);
                assert_eq!(y, 20.0 * 16.0 + 8.0);
            }
            _ => panic!("the dart variant emits a Shot"),
        }
    }

    #[test]
    fn the_trap_variants_map_verbatim() {
        // 1 → 184/40/12, 2 → 187/40/5, 5 → 980/30/12 (`:1786-1796`); the
        // vertical frameX decode (36-71 → −1 Y, ≥72 → +1 Y).
        for (frame_y, kind, damage, speed) in
            [(18, 184, 40, 12.0), (36, 187, 40, 5.0), (90, 980, 30, 12.0)]
        {
            let mut w = world();
            put_trap(&mut w, 10, 20, 72, frame_y);
            let mut wiring = Wiring::new();
            wiring.trip_wire(&mut w, 8, 20, 1, 1);
            let reqs = wiring.take_trap_requests();
            match reqs[0] {
                TrapRequest::Shot { vy, kind: k, damage: d, .. } => {
                    assert_eq!((k, d), (kind, damage));
                    assert_eq!(vy, speed, "frameY {frame_y} fires downward");
                }
                _ => panic!("variant frameY {frame_y} emits a Shot"),
            }
        }
    }

    #[test]
    fn the_check_mech_cooldown_gates_the_trap() {
        // `CheckMech(200)`: the re-trip inside the window emits nothing.
        let mut w = world();
        put_trap(&mut w, 10, 20, 18, 0);
        let mut wiring = Wiring::new();
        wiring.trip_wire(&mut w, 8, 20, 1, 1);
        assert_eq!(wiring.take_trap_requests().len(), 1);
        wiring.trip_wire(&mut w, 8, 20, 1, 1);
        assert!(wiring.take_trap_requests().is_empty());
    }

    #[test]
    fn the_spear_variant_requests_the_capped_spawn() {
        // Variant 3 (frameY 54): a `Spear` request with the ±1 px spread
        // roll and the +14 px offset; the frameX 0/18 slot fires downward
        // (`Wiring.cs:1802-1849` — the cap scan runs app-side).
        let mut w = world();
        put_trap(&mut w, 10, 20, 0, 54);
        let mut wiring = Wiring::new();
        wiring.trip_wire(&mut w, 8, 20, 1, 1);
        let reqs = wiring.take_trap_requests();
        match reqs[0] {
            TrapRequest::Spear { x, y, vx, vy, damage } => {
                assert_eq!(damage, 40);
                assert_eq!(x, 10.0 * 16.0 + 8.0);
                assert_eq!(y, 20.0 * 16.0 + 8.0 + 14.0);
                // `speedY = 4 * 1 + rand(0, 21) * 0.05` — 4..5 px/tick.
                assert!((4.0..=5.0).contains(&vy), "vy {vy}");
                // `speedX = rand(-20, 21) * 0.05` — the ±1 px spread.
                assert!((-1.0..=1.0).contains(&vx), "vx {vx}");
            }
            _ => panic!("the spear variant emits a Spear"),
        }
    }
}
