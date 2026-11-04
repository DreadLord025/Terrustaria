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

use crate::tile::{FLAG_ACTUATOR, FLAG_INACTIVE};
use crate::world::World;

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
}

/// The multi-trigger count a wire cell carries when first reached
/// (`HitWire`'s `_toProcess.Add(point, 4)`).
const TO_PROCESS_COUNT: i32 = 4;
/// `CheckMech`'s queue cap (`Wiring.cs:466`).
const MAX_MECHS: usize = 999;

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

    /// `Wiring.TripWire` (`Wiring.cs:525-620`): for each colour in order, the
    /// wire tiles inside the rect seed one `HitWire` flood.
    pub fn trip_wire(&mut self, world: &mut World, left: i32, top: i32, width: i32, height: i32) {
        self.trip_wire_color(world, left, top, width, height, WireColor::Red);
        self.trip_wire_color(world, left, top, width, height, WireColor::Blue);
        self.trip_wire_color(world, left, top, width, height, WireColor::Green);
        self.trip_wire_color(world, left, top, width, height, WireColor::Yellow);
        // `Wiring.LogicGatePass()` at the end of `TripWire` (`Wiring.cs:665`).
        self.logic_gate_pass(world);
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
        if !world.wires.any_in_rect(left, top, width, height, color) {
            return;
        }
        let seeds: Vec<(i32, i32)> = (left..left + width)
            .flat_map(|i| (top..top + height).map(move |j| (i, j)))
            .filter(|&(i, j)| world.wires.get(i, j, color))
            .collect();
        self.hit_wire(world, seeds, color);
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
        let kind = tile.kind;
        // `HitWireSingle`'s `doSkipWires = true` → `SkipWire` (`Wiring.cs:998`,
        // `:1000`-area): the fired cell never re-fires within this flood.
        self.skip.insert((i, j));
        if tile.flags & FLAG_ACTUATOR != 0 {
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
            _ => {}
        }
    }

    /// `Wiring.ActuateForced` (`Wiring.cs:389-407`), the single-tile core: the
    /// actuated bit flips on tiles that carry an actuator. The multi-tile
    /// region walk (chests, doors, `TileObjectData` frames) is M7-WIRE-3.
    fn actuate_forced(&mut self, world: &mut World, i: i32, j: i32) {
        let Some(t) = world.tile_mut(i, j) else {
            return;
        };
        if t.flags & FLAG_ACTUATOR == 0 {
            return;
        }
        t.flags ^= FLAG_INACTIVE;
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
