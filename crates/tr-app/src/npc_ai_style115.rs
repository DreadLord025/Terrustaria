//! The aiStyle-115 family — the Lady Bugs and the Stinkbug
//! (`NPC.AI_115_LadyBugs`, `NPC.cs:47580-47714`, dispatched at
//! `:41080-41083`). W4-d33, research/25 row 115, CODE-ONLY MODE (no
//! compiler invoked; the witness tests below are written, not run).
//!
//! ## Census
//!
//! The family has THREE vanilla members: the LadyBug 604 / GoldLadyBug
//! 605 (`ID/NPCID.cs:11885-11887`; the shared `SetDefaults` row
//! `NPC.cs:16763-16780`) and the Stinkbug 669 (`:12015`; row
//! `:17467-17480`). The census of `this.aiStyle = 115;` returns exactly
//! those two rows (`:16767`, `:17471`) — no projectile-space user. The
//! port's roster has no ladybug kind — the module lands UNWIRED (the
//! `npc_ai_style73.rs` precedent).
//!
//! ## The machine
//!
//! The wind-drifting critter with a walk mode: `ai[0]` is the flight
//! ANGLE, `ai[1]` the scale seed (0.7-0.9), `ai[2]` the mode (0 fly,
//! 1 walk).
//!
//! The non-client head (`:47582-47620`): the `ai[1] == 0` scale seed
//! `NextFloat()*0.2 + 0.7`; the `--localAI[0] <= 0` re-roll (reset
//! `Next(60,181)`): the `Next(5) == 0` mode flip — fly→walk zeroes
//! `ai[0]`; walk→fly runs TargetClosest + the random-angle pick
//! (`NextFloat()*2π`, the >700-px `AngleTo + NextFloatDirection()*0.3`
//! override) — then the UNCONDITIONAL second TargetClosest + angle
//! pick (the C# runs BOTH — the walk→fly pick is overwritten the same
//! tick, but its DRAWS stay; verbatim).
//!
//! `scale = ai[1]` (`:47521`).
//!
//! Fly mode (`:47622-47671`): the desired velocity
//! `ai[0].ToRotationVector2() + (windSpeedTarget*0.8, 0)`, the 0.0125
//! Lerp. A falling vy probes the 4-row centre column (solid OR liquid)
//! — each hit NEGATES the angle and damps vy ×0.9 (the per-row loop,
//! no break). A rising vy scans the 30-row column (NO break — the flag
//! ORs every row); no hit negates the angle + damps. The collideX
//! reflect: `ai[0] = -ai[0] + π`, `vx *= -0.2`.
//!
//! Walk mode (`:47672-47712`): a falling vy probes the 4-row
//! direction-shifted column for LIQUID only — a hit runs TargetClosest,
//! `vy = -1`, mode 0, the `-π/2 + NextFloat()*π/4` upward angle (the
//! >700 override), netUpdate + RETURN. A non-zero vy damps vx ×0.98 and
//! Lerps vy toward 2 at 0.005; a zero vy Lerps toward
//! `(direction, 0)` at 0.05, adds 0.2 gravity, and the collideX flips
//! the facing + `vx *= -0.2` + netUpdate.
//!
//! The tail (`:47713`): `direction = vx > 0 ? 1 : -1` — the 0-on-tie
//! folds to -1 (NOT Math.Sign).
//!
//! ## RNG order (by construction)
//!
//! The scale seed: ONE NextFloat. The re-roll: Next(60,181), Next(5);
//! the mode-flip pick (walk→fly only): NextFloat (+ NextFloatDirection
//! when >700); then the unconditional pick: NextFloat (+ the
//! direction). The walk-mode liquid escape: NextFloat (+ the
//! direction). TargetClosest is RNG-free; the probes draw nothing.
//!
//! ## FindFrame cases 604/605/669
//!
//! `NPC.cs:66961-66999` (the two cases are identical): the counter
//! advances ONLY while moving (`velocity != Zero`), wrapping at 8;
//! `vy == 0` picks the walk rows 0-3 (`counter/2`), else the fly rows
//! 4-7. `Main.npcFrameCount[604/605/669] = 8`. [`ladybug_find_frame`]
//! ports the pair.
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use glam::Vec2;
use tr_core::tiledata::TILE_SOLID;
use tr_core::{UnifiedRandom, World};

/// The type-604/605 `SetDefaults` row (`NPC.cs:16763-16780`), verbatim.
pub struct LadyBugDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit1`.
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath1`.
    pub death_sound: (i32, i32),
    pub npc_slots: f32,
    pub no_gravity: bool,
    /// `catchItem = type - 604 + 4361` (4361/4362).
    pub catch_item_base: i32,
    /// The Gold 605's rarity 3.
    pub gold_rarity: i32,
}

/// `NPC.cs:16763-16780` — `else if (this.type == 604 || 605)`.
pub const LADY_BUG: LadyBugDefaults = LadyBugDefaults {
    width: 10,
    height: 10,
    ai_style: 115,
    damage: 0,
    defense: 0,
    life_max: 5,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    npc_slots: 0.25,
    no_gravity: true,
    catch_item_base: 4361,
    gold_rarity: 3,
};

/// The type-669 Stinkbug row (`NPC.cs:17467-17480`), verbatim — the
/// same stats, `catchItem = 5132`, no rarity write.
pub const STINKBUG: LadyBugDefaults = LadyBugDefaults {
    width: 10,
    height: 10,
    ai_style: 115,
    damage: 0,
    defense: 0,
    life_max: 5,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    npc_slots: 0.25,
    no_gravity: true,
    catch_item_base: 5132,
    gold_rarity: 0, // no rarity write — the head default stands
};

/// `Main.npcFrameCount[604/605/669]` = 8 (`Main.cs:67144`/`:67150`).
pub const LADY_BUG_FRAME_COUNT: u32 = 8;

/// The per-tick environment.
pub struct Style115Env<'a> {
    pub world: &'a World,
    pub rng: &'a mut UnifiedRandom,
    /// The `Main.player[]` slots as `TargetClosest` sees them.
    pub players: &'a [TargetCandidate],
    /// `Main.windSpeedTarget` — the fly drift.
    pub wind_speed_target: f32,
    /// `Main.netMode`.
    pub net_mode: i32,
    /// `Main.EverLastingTicker` for the selection's `targetSetFrame`.
    pub ticker: u64,
}

/// XNA `Vector2.Distance` — the f32 sum widened to f64 for the root.
fn distance(a: Vec2, b: Vec2) -> f32 {
    let d = a - b;
    ((d.x * d.x + d.y * d.y) as f64).sqrt() as f32
}

/// The target slot's candidate — the style16 `target_of` fallback.
//! The aiStyle-115 family — the ladybugs (`NPC.AI()`'s
//! `else if (this.aiStyle == 115) { this.AI_115_LadyBugs(); }` arm,
//! `NPC.cs:41081-41083`; the machine `AI_115_LadyBugs`,
//! `NPC.cs:47580-47714`). W4-e26, CODE-ONLY MODE (no compiler invoked;
//! the witness tests below are written, not run).
//!
//! NOT the projectile aiStyle 115 — that is a different system
//! (`Projectile.cs`); this module owns the NPC style.
//!
//! ## Census (every `this.aiStyle = 115;` row in `NPC.cs`)
//!
//! TWO kinds, the shared `SetDefaults` row (`NPC.cs:16763-16780`):
//! * **LadyBug 604** (`NPCID.LadyBug`, `ID/NPCID.cs:11885`) — 10×10,
//!   dmg 0, def 0, 5 HP, NPCHit1/NPCDeath1, npcSlots 0.25, noGravity,
//!   catchItem 4361 (`type - 604 + 4361`).
//! * **GoldLadyBug 605** (`ID/NPCID.cs:11887`) — the same row + rarity 3
//!   (`:16775-16778`), catchItem 4362; the gold flag doubles the luck
//!   windows (`LadyBugKilled`/`LadyBugLuck`'s `type == 605`,
//!   `NPC.cs:81588`/`:81616`). `NPCID.Sets.IsGoldCritter` member
//!   (`ID/NPCID.cs:10126`).
//!
//! The arm itself reads NO `this.type` — both kinds run the identical
//! flight; the gold distinction lives entirely in the luck setters
//! (`luck.rs`) and the catch/convert tables (`bug_net.rs`), not here.
//!
//! ## The machine (`NPC.cs:47580-47714`, in C# order)
//!
//! A drifting flyer with a perch state. `ai[0]` = the flight angle,
//! `ai[1]` = the scale latch, `ai[2]` = the perch flag (0 fly / 1 perch),
//! `localAI[0]` = the rearm timer.
//! 1. `:47582-47620` — `Main.netMode != 1` (folds true, the port is
//!    single-player): the rearm block.
//!    * `:47584-47588` — `ai[1] == 0`: ONE `NextFloat() * 0.2 + 0.7`
//!      scale latch + netUpdate.
//!    * `:47589` — `--localAI[0] <= 0`: rearm. `localAI[0] =
//!      Next(60, 181)` (`:47591`); `Next(5) == 0` (`:47592`) toggles the
//!      perch: fly→perch sets `ai[2] = 1, ai[0] = 0` (`:47594-47599`),
//!      perch→fly runs `TargetClosest()` then `ai[2] = 0`, `ai[0] =
//!      NextFloat() * 2π` with the >700px `AngleTo + NextFloatDirection
//!      * 0.3` redirect (`:47600-47610`). THEN, unconditionally,
//!      `TargetClosest()` + the same `ai[0]` roll/redirect (`:47612-
//!      47618`) — the toggle's roll is OVERWRITTEN by this tail roll
//!      (the C# draws both; the second wins). netUpdate.
//! 2. `:47621` — `scale = ai[1]` every tick.
//! 3. `:47622-47671` — `ai[2] == 0` (fly): the aim vector
//!    `ai[0].ToRotationVector2() * 1`, `X += windSpeedTarget * 0.8`,
//!    `velocity = Lerp(velocity, aim, 0.0125)`. Falling (`vy > 0`): the
//!    4-tile down-scan, solid OR liquid → `ai[0] = -ai[0]` + `vy *= 0.9`
//!    per blocking tile (`:47627-47643`). Rising (`vy < 0`): the 30-tile
//!    down-scan for ANY solid OR liquid; none → `ai[0] = -ai[0]` +
//!    `vy *= 0.9` ONCE (`:47644-47665`). `collideX` → `ai[0] = -ai[0] +
//!    π`, `vx *= -0.2` (`:47666-47670`).
//! 4. `:47672-47712` — else (perch): falling (`vy > 0`) the 4-tile
//!    down-scan at `centerX/16 + direction` for LIQUID only →
//!    `TargetClosest()`, `vy = -1`, `ai[2] = 0`, `ai[0] = NextFloat() *
//!    π/4 - π/2` + the >700px redirect, netUpdate, RETURN (`:47674-
//!    47695`). `vy != 0`: `vx *= 0.98`, `vy = Lerp(vy, 2, 0.005)`
//!    (`:47696-47700`). Else grounded: `velocity = Lerp(velocity,
//!    UnitX * direction, 0.05)`, `vy += 0.2`, `collideX` → `direction
//!    *= -1`, `vx *= -0.2`, netUpdate (`:47701-47711`).
//! 5. `:47713` — `direction = (vx > 0) ? 1 : -1` (the tail, every tick).
//!
//! ## RNG order (the rearm block, `Main.rand`)
//!
//! Per rearm tick (`localAI[0]` expires): `Next(60,181)`, `Next(5)`; on
//! the 1/5 toggle's perch→fly arm `NextFloat()` (+ conditional
//! `NextFloatDirection()`); then the tail `NextFloat()` (+ conditional
//! `NextFloatDirection()`). The one-time scale latch draws ONE
//! `NextFloat()`. The perch water-launch draws `NextFloat()` (+
//! conditional `NextFloatDirection()`). Every other path draws NOTHING.
//! `TargetClosest` is RNG-free (`npc_target.rs`). The witnesses mirror
//! the cited draws on a cloned `UnifiedRandom` and compare stream state.
//!
//! ## Seam
//!
//! The module writes only the fields the C# arm writes — velocity,
//! `ai[0..2]`, `localAI[0]`, `scale`, `direction` (`Npc::facing`) and
//! `netUpdate`. It does NOT integrate position: the arm ends before
//! `UpdateCollision`. The ladybugs are noGravity (`NPC.cs:16774`) but
//! tile-collide (the row writes no noTileCollide), so the wiring's
//! integration tail is the no-gravity `UpdateCollision`
//! (`Npc::integrate_flyer_collision`, the Goldfish/bat convention).
//! `wind_speed_target` is `Main.windSpeedTarget` — the sim stamps it
//! (`Npc::ladybug_wind`); the solo path passes 0. `TargetClosest()`
//! rides `npc_target::target_closest` UNMODIFIED (style 115 — the
//! `:77761` fighter arm is style-3-only, so no directionY rewrite).

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::tiledata::TILE_SOLID;
use tr_core::{UnifiedRandom, World};

/// `Utils.ToRotationVector2` (`Utils.cs:1665-1668`): `((float)cos(f),
/// (float)sin(f))` — the f64 trig over the f32 angle.
fn to_rotation_vector2(f: f32) -> (f32, f32) {
    ((f as f64).cos() as f32, (f as f64).sin() as f32)
}

/// `MathHelper.Lerp(a, b, t)` — `a + (b - a) * t` (the XNA scalar form;
/// `Vector2.Lerp` applies it per component).
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// `Vector2.Distance` — `sqrt(DistanceSquared)`, f32 sum, f64 sqrt cast
/// back (the `npc_ai_style64.rs` convention).
fn vector_distance(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = ax - bx;
    let dy = ay - by;
    ((dx * dx + dy * dy) as f64).sqrt() as f32
}

/// `Entity.AngleTo` (`Entity.cs:185-188`): `(float)Math.Atan2(dest.Y -
/// Center.Y, dest.X - Center.X)` — the f64 atan2 over the f32 deltas.
fn angle_to(cx: f32, cy: f32, dx: f32, dy: f32) -> f32 {
    ((dy - cy) as f64).atan2((dx - cx) as f64) as f32
}

/// The arm's `Main.tileSolid[type]` read — the RUNTIME table: the
/// generated `TILE_SOLID` minus the living-tree leaves (192) that
/// `WorldGen.cs:15714` clears after generation (the
/// `npc_ai_style64.rs::tile_solid_runtime` precedent). `tileSolidTop`
/// (platforms) is NOT in this set.
fn tile_solid_runtime(kind: u16) -> bool {
    TILE_SOLID.has(kind) && kind != 192
}

/// `Tile.nactive()` (`Tile.cs:240-247`): active AND not actuated-off.
fn nactive(t: &tr_core::tile::Tile) -> bool {
    t.active() && !t.inactive()
}

/// The target slot's candidate — the `npc_ai_style74.rs::target_of`
/// fallback (the C# indexes `Main.player[this.target]`, 255 slots always
/// exist; an unset/stale slot falls back to slot 0).
fn target_of<'a>(npc: &Npc, players: &'a [TargetCandidate]) -> &'a TargetCandidate {
    let i = if npc.target >= 0 && (npc.target as usize) < players.len() {
        npc.target as usize
    } else {
        0
    };
    &players[i]
}

/// The random-angle pick (`:47604-47608` / `:47613-47617` /
/// `:47686-47690`): `NextFloat() * span + base`, then the >700-px
/// `AngleTo(target) + NextFloatDirection()*0.3` override. `base`/`span`
/// are 0/2π for the re-roll, -π/2/π/4 for the liquid escape.
fn pick_angle(npc: &Npc, env: &mut Style115Env<'_>, base: f32, span: f32, center: Vec2) -> f32 {
    let mut angle = env.rng.next_float() * span + base;
    let p = target_of(npc, env.players);
    let pc = Vec2::new(p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32);
    if distance(center, pc) > 700.0 {
        // `Entity.AngleTo` (Entity.cs:185-188): atan2 in double of the
        // f32 differences, cast to f32.
        let aim = ((pc.y - center.y) as f64).atan2((pc.x - center.x) as f64) as f32;
        angle = aim + env.rng.next_float_direction() * 0.3;
    }
    angle
}

/// One fixed step of the aiStyle-115 arm (`NPC.cs:47580-47714`), in C#
/// order. The caller owns the movement integration (the rows'
/// `noGravity` is true — the walk mode's 0.2 is the arm's own).
pub fn style115_step(npc: &mut Npc, env: &mut Style115Env<'_>) {
    let (w, h) = npc.size();
    let center = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
    // `:47582-47620` — the non-client head.
    if env.net_mode != 1 {
        if npc.ai[1] == 0.0 {
            npc.ai[1] = env.rng.next_float() * 0.2 + 0.7;
            npc.net_update = true;
        }
        npc.local_ai[0] -= 1.0; // `--this.localAI[0]`
        if npc.local_ai[0] <= 0.0 {
            npc.local_ai[0] = env.rng.next_range(60, 181) as f32;
            if env.rng.next_max(5) == 0 {
                if npc.ai[2] == 0.0 {
                    npc.ai[2] = 1.0;
                    npc.ai[0] = 0.0;
                    npc.net_update = true;
                } else if npc.ai[2] == 1.0 {
                    if !env.players.is_empty() {
                        target_closest(npc, env.world, env.players, true, 115, env.ticker);
                    }
                    npc.ai[2] = 0.0;
                    npc.ai[0] = pick_angle(npc, env, 0.0, std::f32::consts::PI * 2.0, center);
                    npc.net_update = true;
                }
            }
            // `:47612-47618` — the UNCONDITIONAL second pick (the
            // walk→fly pick above is overwritten, its draws stay).
            if !env.players.is_empty() {
                target_closest(npc, env.world, env.players, true, 115, env.ticker);
            }
            npc.ai[0] = pick_angle(npc, env, 0.0, std::f32::consts::PI * 2.0, center);
            npc.net_update = true;
        }
    }
    // `:47621` — the scale.
    npc.scale = npc.ai[1];
    if npc.ai[2] == 0.0 {
        // `:47622-47671` — the fly mode.
        let a = npc.ai[0];
        let mut value = Vec2::new((a as f64).cos() as f32, (a as f64).sin() as f32);
        value.x += env.wind_speed_target * 0.8;
        let v = Vec2::new(npc.vx, npc.vy);
        let blended = v + (value - v) * 0.0125;
        npc.vx = blended.x;
        npc.vy = blended.y;
        if npc.vy > 0.0 {
            // `:47627-47643` — the 4-row fall probe (per-row, no break).
            let col = center.x as i32 / 16;
            let row = center.y as i32 / 16;
            for i in row..row + 4 {
                let blocked = env.world.tile(col, i).is_some_and(|t| {
                    (t.active() && !t.inactive() && TILE_SOLID.has(t.kind)) || t.liquid > 0
                });
                if blocked {
                    npc.ai[0] = 0.0 - npc.ai[0];
                    if npc.vy > 0.0 {
                        npc.vy *= 0.9;
/// `Entity.Center` (`Entity.cs:50-55`) for a player candidate.
fn candidate_center(p: &TargetCandidate) -> (f32, f32) {
    (p.x + p.width as f32 / 2.0, p.y + p.height as f32 / 2.0)
                    }
                }
            }
        }
    }
}
}

/// The `ai[0]` roll + the >700px redirect, shared by the rearm tail
/// (`:47613-47617`), the perch→fly toggle (`:47604-47608`) and the perch
/// water-launch (`:47686-47690`, which passes the π/4 - π/2 base). ONE
/// `NextFloat()`; the redirect adds ONE `NextFloatDirection()` only when
/// the target is > 700px away. `base_angle` is the roll's scale factor:
/// `2π` for the rearm/toggle, `π/4` (offset `-π/2`) for the launch.
fn roll_angle(
    npc: &Npc,
    players: &[TargetCandidate],
    rng: &mut UnifiedRandom,
    scale: f32,
    offset: f32,
) -> f32 {
    let mut a = rng.next_float() * scale + offset;
    let (cx, cy) = npc.center();
    let (px, py) = candidate_center(target_of(npc, players));
    if vector_distance(cx, cy, px, py) > 700.0 {
        a = angle_to(cx, cy, px, py) + rng.next_float_direction() * 0.3;
    }
    a
}

/// One fixed step of the aiStyle-115 arm (`NPC.cs:47580-47714`), in C#
/// order. `wind_speed_target` is `Main.windSpeedTarget` (the sim stamps
/// it; the solo path passes 0); `ticker` feeds `TargetClosest`'s
/// `targetSetFrame` (0 until plumbed — the S1-D-3p precedent); `rng` is
/// the caller's `Main.rand` (`UpdateNPCs`) stream.
pub fn ai_115_step(
    npc: &mut Npc,
    world: &World,
    players: &[TargetCandidate],
    wind_speed_target: f32,
    ticker: u64,
    rng: &mut UnifiedRandom,
) {
    // `:47582-47620` — the rearm block (`Main.netMode != 1` folds true).
    // `:47584-47588` — the one-time scale latch.
    if npc.ai[1] == 0.0 {
        npc.ai[1] = rng.next_float() * 0.2 + 0.7;
        npc.net_update = true;
    }
    // `:47589` — the decrement runs EVERY tick, BEFORE the expiry test.
    npc.local_ai[0] -= 1.0;
    if npc.local_ai[0] <= 0.0 {
        // `:47591` — the rearm timer.
        npc.local_ai[0] = rng.next_range(60, 181) as f32;
        // `:47592` — the 1/5 perch toggle.
        if rng.next_max(5) == 0 {
            if npc.ai[2] == 0.0 {
                // `:47594-47599` — fly → perch.
                npc.ai[2] = 1.0;
                npc.ai[0] = 0.0;
                npc.net_update = true;
            } else if npc.ai[2] == 1.0 {
                // `:47600-47610` — perch → fly: TargetClosest, then the
                // roll (OVERWRITTEN by the tail roll below — the C#
                // draws both).
                target_closest(npc, world, players, true, 115, ticker);
                npc.ai[2] = 0.0;
                npc.ai[0] = roll_angle(npc, players, rng, std::f32::consts::TAU, 0.0);
                npc.net_update = true;
            }
        }
        // `:47612-47618` — the unconditional tail: TargetClosest + the
        // roll/redirect (the second roll wins).
        target_closest(npc, world, players, true, 115, ticker);
        npc.ai[0] = roll_angle(npc, players, rng, std::f32::consts::TAU, 0.0);
        npc.net_update = true;
    }

    // `:47621` — scale rides the latched ai[1] every tick.
    npc.scale = npc.ai[1];

    if npc.ai[2] == 0.0 {
        // `:47622-47671` — the flight.
        // `:47624-47626` — the aim vector + the wind drift + the 1/80
        // blend (0.0125 = 1/80).
        let (mut ax, ay) = to_rotation_vector2(npc.ai[0]);
        ax += wind_speed_target * 0.8;
        npc.vx = lerp(npc.vx, ax, 0.0125);
        npc.vy = lerp(npc.vy, ay, 0.0125);
        // `:47627-47643` — falling: the 4-tile down-scan, solid OR
        // liquid → flip the aim + damp ×0.9 per blocking tile.
        if npc.vy > 0.0 {
            let (cx, cy) = npc.center();
            let tx = (cx as i32) / 16;
            let ty = (cy as i32) / 16;
            for row in ty..ty + 4 {
                if let Some(t) = world.tile(tx, row) {
                    if (nactive(t) && tile_solid_runtime(t.kind)) || t.liquid > 0 {
                        npc.ai[0] = 0.0 - npc.ai[0];
                        if npc.vy > 0.0 {
                            npc.vy *= 0.9;
                        }
                    }
                }
            }
        }
        if npc.vy < 0.0 {
            // `:47644-47665` — the 30-row ceiling scan (NO break).
            let col = center.x as i32 / 16;
            let row = center.y as i32 / 16;
            let mut ceiling = false;
            for j in row..row + 30 {
                let blocked = env.world.tile(col, j).is_some_and(|t| {
                    (t.active() && !t.inactive() && TILE_SOLID.has(t.kind)) || t.liquid > 0
                });
                if blocked {
                    ceiling = true;
                }
            }
            if !ceiling {
        // `:47644-47665` — rising: the 30-tile down-scan for ANY solid
        // OR liquid; none → flip the aim + damp ONCE.
        if npc.vy < 0.0 {
            let mut found = false;
            let (cx, cy) = npc.center();
            let tx = (cx as i32) / 16;
            let ty = (cy as i32) / 16;
            for row in ty..ty + 30 {
                if let Some(t) = world.tile(tx, row) {
                    if (nactive(t) && tile_solid_runtime(t.kind)) || t.liquid > 0 {
                        found = true;
                    }
                }
            }
            if !found {
                npc.ai[0] = 0.0 - npc.ai[0];
                if npc.vy < 0.0 {
                    npc.vy *= 0.9;
                }
            }
        }
        }
        // `:47666-47670` — the collideX reflect.
        // `:47666-47670` — the wall bounce.
        if npc.collide_x {
            npc.ai[0] = 0.0 - npc.ai[0] + std::f32::consts::PI;
            npc.vx *= -0.2;
        }
    } else {
        // `:47672-47712` — the walk mode.
        if npc.vy > 0.0 {
            // `:47674-47695` — the liquid escape.
            let col = center.x as i32 / 16 + npc.facing;
            let row = center.y as i32 / 16;
            for k in row..row + 4 {
                if env.world.tile(col, k).is_some_and(|t| t.liquid > 0) {
                    if !env.players.is_empty() {
                        target_closest(npc, env.world, env.players, true, 115, env.ticker);
                    }
                    npc.vy = -1.0;
                    npc.ai[2] = 0.0;
                    npc.ai[0] = pick_angle(
                        npc,
                        env,
                        -std::f32::consts::FRAC_PI_2,
                        std::f32::consts::PI / 4.0,
                        center,
                    );
                    npc.net_update = true;
                    return; // `:47692` — the tail facing is SKIPPED
                }
            }
        }
        if npc.vy != 0.0 {
            // `:47696-47700` — the fall.
            npc.vx *= 0.98;
            npc.vy += (2.0 - npc.vy) * 0.005;
        } else {
            // `:47701-47711` — the ground walk.
            let want = Vec2::new(npc.facing as f32, 0.0);
            let v = Vec2::new(npc.vx, npc.vy);
            let blended = v + (want - v) * 0.05;
            npc.vx = blended.x;
            npc.vy = blended.y + 0.2;
        // `:47672-47712` — the perch.
        // `:47674-47695` — falling: the 4-tile down-scan at
        // centerX/16 + direction for LIQUID only → the water launch.
        if npc.vy > 0.0 {
            let (cx, cy) = npc.center();
            let tx = (cx as i32) / 16 + npc.facing;
            let ty = (cy as i32) / 16;
            for row in ty..ty + 4 {
                if let Some(t) = world.tile(tx, row) {
                    if t.liquid > 0 {
                        target_closest(npc, world, players, true, 115, ticker);
                        npc.vy = -1.0;
                        npc.ai[2] = 0.0;
                        npc.ai[0] = roll_angle(
                            npc,
                            players,
                            rng,
                            std::f32::consts::FRAC_PI_4,
                            -std::f32::consts::FRAC_PI_2,
                        );
                        npc.net_update = true;
                        // `:47692` — the RETURN: the direction tail
                        // (`:47713`) does NOT run on the launch tick.
                        return;
                    }
                }
            }
        }
        // `:47696-47700` — airborne perch: the X damp + the Y sink blend.
        if npc.vy != 0.0 {
            npc.vx *= 0.98;
            npc.vy = lerp(npc.vy, 2.0, 0.005);
        } else {
            // `:47701-47711` — grounded perch: the walk blend + the
            // 0.2 gravity + the wall turn.
            npc.vx = lerp(npc.vx, npc.facing as f32, 0.05);
            npc.vy = lerp(npc.vy, 0.0, 0.05);
            npc.vy += 0.2;
            if npc.collide_x {
                npc.facing *= -1;
                npc.vx *= -0.2;
                npc.net_update = true;
            }
        }
    }
    }
    // `:47713` — the tail facing (the 0-tie folds to -1).
    npc.facing = if npc.vx > 0.0 { 1 } else { -1 };
}
}

/// `NPC.FindFrame` cases 604/605/669 (`NPC.cs:66961-66999`) — the
/// counter pair (the style73 convention): the counter advances ONLY
/// while moving, wrapping at 8; `vy == 0` picks the walk rows
/// (`counter/2`), else the fly rows (`4 + counter/2`). The
/// `spriteDirection = direction` write is the caller's.
pub fn ladybug_find_frame(_frame_row: u32, frame_counter: f32, vx: f32, vy: f32) -> (u32, f32) {
    let mut counter = frame_counter;
    if vx != 0.0 || vy != 0.0 {
        counter += 1.0;
        if counter >= 8.0 {
            counter = 0.0;
        }
    }
    let row = if vy == 0.0 {
        (counter as i32 / 2) as u32
    } else {
        (4 + counter as i32 / 2) as u32
    };
    (row, counter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// A 604-shaped NPC (the style73 placeholder convention).
    fn bug(x: f32, y: f32) -> Npc {
        let d = &LADY_BUG;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n.facing = 1;
        n.ai[1] = 0.8; // the scale seed already rolled
        n.local_ai[0] = 50.0; // no re-roll this tick
        n
    // `:47713` — the direction tail (every tick but the launch return).
    npc.facing = if npc.vx > 0.0 { 1 } else { -1 };
}
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN, NOT RUN (code-only mode). Each pins one
    //! C# branch against its citation; the RNG witnesses mirror the
    //! cited draws on a cloned `UnifiedRandom` and compare the stream
    //! state (`PartialEq`) so a drifted draw order fails by construction.
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn air() -> World {
        World::new("t".into(), 400, 300)
    }

    fn cand(x: f32, y: f32) -> TargetCandidate {
        TargetCandidate {
            active: true,
            dead: false,
            ghost: false,
            x,
            y,
            width: 20,
            height: 42,
            aggro: 0,
            item_animation: 0,
            npc_type_no_aggro: false,
            tank_pet: None,
        }
    }

    fn air() -> World {
        World::new("t".into(), 400, 300)
    }

    /// `NPC.cs:16763-16780` + `:17467-17480` — the two rows.
    #[test]
    fn w4_d33_s115_the_defaults_rows() {
        let d = &LADY_BUG;
        assert_eq!((d.width, d.height), (10, 10));
        assert_eq!(d.ai_style, 115);
        assert_eq!((d.damage, d.defense, d.life_max), (0, 0, 5));
        assert_eq!(d.hit_sound, (3, 1));
        assert_eq!(d.death_sound, (4, 1));
        assert_eq!(d.npc_slots, 0.25);
        assert!(d.no_gravity);
        assert_eq!(d.catch_item_base, 4361);
        assert_eq!(d.gold_rarity, 3);
        assert_eq!(STINKBUG.catch_item_base, 5132);
        assert_eq!(STINKBUG.gold_rarity, 0);
        assert_eq!(LADY_BUG_FRAME_COUNT, 8);
    }

    /// `:47584-47588` — the scale seed: ONE NextFloat, 0.7-0.9.
    #[test]
    fn w4_d33_s115_the_scale_seed() {
        let w = air();
        let players = [cand(100000.0, 100000.0)];
        let mut rng = UnifiedRandom::new(115);
        let mut oracle = UnifiedRandom::new(115);
        let mut n = bug(100.0, 100.0);
        n.ai[1] = 0.0;
        let mut env = Style115Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            wind_speed_target: 0.0,
            net_mode: 0,
            ticker: 7,
        };
        style115_step(&mut n, &mut env);
        let f = oracle.next_float();
        assert_eq!(n.ai[1], f * 0.2 + 0.7);
        assert_eq!(n.scale, n.ai[1]);
        assert_eq!(rng.peek(), oracle.peek(), "one draw only");
    }

    /// `:47589-47619` — the re-roll: Next(60,181), Next(5); the
    /// walk→fly flip draws TWO angle picks (the first overwritten,
    /// both consumed).
    #[test]
    fn w4_d33_s115_the_reroll_double_pick() {
        let w = air();
        let players = [cand(200.0, 100.0)]; // within 700 — no direction draws
        let mut rng = UnifiedRandom::new(115);
        let mut oracle = UnifiedRandom::new(115);
        let mut n = bug(100.0, 100.0);
        n.local_ai[0] = 1.0; // -- → 0 → the re-roll
        n.ai[2] = 1.0; // walk → the flip arm
        let mut env = Style115Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            wind_speed_target: 0.0,
            net_mode: 0,
            ticker: 7,
        };
        style115_step(&mut n, &mut env);
        oracle.next_range(60, 181);
        let flip = oracle.next_max(5);
        if flip == 0 {
            oracle.next_float(); // the walk→fly pick (overwritten)
        }
        oracle.next_float(); // the unconditional pick
        assert_eq!(rng.peek(), oracle.peek(), "the double-pick order");
        if flip == 0 {
            assert_eq!(n.ai[2], 0.0, "the flip landed");
        }
        assert!(n.net_update);
    }

    /// `:47622-47626` — the fly Lerp: the angle vector + wind*0.8 at
    /// 0.0125.
    #[test]
    fn w4_d33_s115_the_fly_lerp() {
        let w = air();
        let players = [cand(100000.0, 100000.0)];
        let mut rng = UnifiedRandom::new(115);
        let mut n = bug(100.0, 100.0);
        n.ai[0] = 0.0; // due right
        n.vx = 0.0;
        n.vy = 0.0;
        let mut env = Style115Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            wind_speed_target: 0.5,
            net_mode: 0,
            ticker: 7,
        };
        style115_step(&mut n, &mut env);
        // value = (1 + 0.4, 0); v' = v + (value - v)*0.0125.
        assert!((n.vx - 1.4 * 0.0125).abs() < 1e-6, "vx {}", n.vx);
        assert_eq!(n.vy, 0.0);
    }

    /// `:47666-47670` — the collideX reflect: angle = -angle + π,
    /// vx ×-0.2.
    #[test]
    fn w4_d33_s115_the_collide_reflect() {
        let w = air();
        let players = [cand(100000.0, 100000.0)];
        let mut rng = UnifiedRandom::new(115);
        let mut n = bug(100.0, 100.0);
        n.ai[0] = 0.5;
        n.collide_x = true;
        n.vx = 1.0;
        let mut env = Style115Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            wind_speed_target: 0.0,
            net_mode: 0,
            ticker: 7,
        };
        style115_step(&mut n, &mut env);
        assert!((n.ai[0] - (-0.5 + std::f32::consts::PI)).abs() < 1e-5);
        // The Lerp ran first; the reflect then scales the blended vx.
        assert!(n.vx < 0.0, "the reflected vx");
    }

    /// `:47674-47695` — the walk-mode liquid escape: vy=-1, mode 0,
    /// the -π/2..-π/4 angle, the RETURN skips the tail facing.
    #[test]
    fn w4_d33_s115_the_liquid_escape() {
        let mut w = air();
        // Water in the direction-shifted column below the centre.
        let n0 = bug(100.0, 100.0);
        let col = (n0.x + 5.0) as i32 / 16 + 1;
        let row = (n0.y + 5.0) as i32 / 16;
        let i = w.index(col, row);
        w.tiles[i].liquid = 100;
        let players = [cand(200.0, 100.0)];
        let mut rng = UnifiedRandom::new(115);
        let mut oracle = UnifiedRandom::new(115);
        let mut n = bug(100.0, 100.0);
        n.ai[2] = 1.0; // walk
        n.vy = 1.0; // falling
        n.vx = 5.0; // the tail facing would flip to +1 — the return skips it
        n.facing = 1;
        let mut env = Style115Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            wind_speed_target: 0.0,
            net_mode: 0,
            ticker: 7,
        };
        style115_step(&mut n, &mut env);
        let f = oracle.next_float();
        assert_eq!(rng.peek(), oracle.peek(), "the escape pick");
        assert_eq!(n.vy, -1.0);
        assert_eq!(n.ai[2], 0.0);
        let want = f * (std::f32::consts::PI / 4.0) - std::f32::consts::FRAC_PI_2;
        assert!((n.ai[0] - want).abs() < 1e-6);
        assert!(n.net_update);
    }

    /// `:47701-47711` — the ground walk: the 0.05 Lerp toward
    /// (direction,0), the +0.2 gravity, the collideX flip.
    #[test]
    fn w4_d33_s115_the_ground_walk() {
        let w = air();
        let players = [cand(100000.0, 100000.0)];
        let mut rng = UnifiedRandom::new(115);
        let mut n = bug(100.0, 100.0);
        n.ai[2] = 1.0;
        n.vy = 0.0;
        n.vx = 0.0;
        let mut env = Style115Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            wind_speed_target: 0.0,
            net_mode: 0,
            ticker: 7,
        };
        style115_step(&mut n, &mut env);
        assert!((n.vx - 0.05).abs() < 1e-6, "the 0.05 Lerp");
        assert_eq!(n.vy, 0.2);
        assert_eq!(n.facing, 1);
        let mut n = bug(100.0, 100.0);
        n.ai[2] = 1.0;
        n.vy = 0.0;
        n.collide_x = true;
        let mut env = Style115Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            wind_speed_target: 0.0,
            net_mode: 0,
            ticker: 7,
        };
        style115_step(&mut n, &mut env);
        assert!(n.net_update);
        // The flip made facing -1, then the tail re-faces from vx
        // (0.05 * -0.2 = -0.01 < 0 → -1).
        assert_eq!(n.facing, -1);
    }

    /// `NPC.cs:66961-66999` — the moving-only counter, the walk/fly
    /// row split.
    #[test]
    fn w4_d33_s115_the_find_frame_split() {
        // Still: the counter holds, the walk row.
        assert_eq!(ladybug_find_frame(0, 3.0, 0.0, 0.0), (1, 3.0));
        // Moving: the counter advances; vy != 0 → the fly rows.
        assert_eq!(ladybug_find_frame(0, 3.0, 1.0, 0.0), (2, 4.0));
        assert_eq!(ladybug_find_frame(4, 3.0, 0.0, 1.0), (6, 4.0));
        // The 8 wrap.
        assert_eq!(ladybug_find_frame(3, 7.0, 1.0, 0.0), (0, 0.0));
    }

    /// The stream parity: 60 ticks match a cloned twin bit-for-bit.
    #[test]
    fn w4_d33_s115_the_run_is_twin_stable() {
        let w = air();
        let players = [cand(300.0, 300.0)];
        let mut a = bug(280.0, 280.0);
        a.ai[1] = 0.0;
        a.local_ai[0] = 1.0;
        let mut b = a.clone();
        let mut rng_a = UnifiedRandom::new(604);
        let mut rng_b = UnifiedRandom::new(604);
        for tick in 0..60 {
            let mut env_a = Style115Env {
                world: &w,
                rng: &mut rng_a,
                players: &players,
                wind_speed_target: 0.3,
                net_mode: 0,
                ticker: 7,
            };
            style115_step(&mut a, &mut env_a);
            let mut env_b = Style115Env {
                world: &w,
                rng: &mut rng_b,
                players: &players,
                wind_speed_target: 0.3,
                net_mode: 0,
                ticker: 7,
            };
            style115_step(&mut b, &mut env_b);
            assert_eq!(rng_a.peek(), rng_b.peek(), "stream @{tick}");
            assert_eq!(a.vx.to_bits(), b.vx.to_bits(), "vx @{tick}");
            assert_eq!(a.ai[0].to_bits(), b.ai[0].to_bits(), "angle @{tick}");
        }
    /// A ladybug body: the C#'s 10×10 box (`NPC.cs:16765-16766`), 5 HP,
    /// the scale latch pre-set so the one-time draw stays out of the way
    /// unless a test wants it; `local_ai[0]` high so no rearm fires.
    fn bug(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(crate::npc::NpcKind::LadyBug, x, y, 1, 0.0);
        n.ai[1] = 1.0;
        n.local_ai[0] = 1000.0;
        n
    }

    fn solid(w: &mut World, x: i32, y: i32) {
        let i = w.index(x, y);
        w.tiles[i] = Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    /// `NPC.cs:47584-47588` — the one-time scale latch: ONE NextFloat,
    /// `* 0.2 + 0.7`, netUpdate; ai[1] != 0 never re-draws.
    #[test]
    fn w4_e26_s115_the_scale_latch_draws_once() {
        let w = air();
        let players = [cand(2000.0, 1500.0)];
        let mut n = bug(1600.0, 1500.0);
        n.ai[1] = 0.0;
        let mut rng = UnifiedRandom::new(7);
        let mut mirror = rng.clone();
        let want = mirror.next_float() * 0.2 + 0.7;
        ai_115_step(&mut n, &w, &players, 0.0, 0, &mut rng);
        assert_eq!(n.ai[1], want, ":47586");
        assert_eq!(n.scale, want, ":47621");
        assert!(n.net_update, ":47587");
        assert_eq!(rng, mirror, "exactly one NextFloat");
        // A second tick with ai[1] set draws nothing for the latch.
        let mut rng2 = UnifiedRandom::new(7);
        let before = rng2.clone();
        n.local_ai[0] = 1000.0;
        ai_115_step(&mut n, &w, &players, 0.0, 0, &mut rng2);
        assert_eq!(rng2, before, "no rearm, no latch — zero draws");
    }

    /// `NPC.cs:47624-47626` — the flight blend: aim = ToRotationVector2
    /// (ai[0]) + wind*0.8 on X, velocity = Lerp(v, aim, 0.0125).
    #[test]
    fn w4_e26_s115_the_flight_blends_toward_the_wind_drifted_aim() {
        let w = air();
        let players = [cand(2000.0, 1500.0)];
        let mut n = bug(1600.0, 1500.0);
        n.ai[0] = 0.0; // aim = (1, 0)
        n.vx = 0.0;
        n.vy = 0.0;
        let mut rng = UnifiedRandom::new(7);
        ai_115_step(&mut n, &w, &players, 0.5, 0, &mut rng);
        // aim.X = cos(0) + 0.5*0.8 = 1.4; aim.Y = sin(0) = 0.
        assert_eq!(n.vx, lerp(0.0, 1.4, 0.0125), ":47625-47626");
        assert_eq!(n.vy, lerp(0.0, 0.0, 0.0125), ":47626");
    }

    /// `NPC.cs:47627-47643` — the falling down-scan: a solid 2 rows down
    /// flips the aim and damps ×0.9 (one blocking tile → one damp).
    #[test]
    fn w4_e26_s115_the_falling_scan_flips_and_damps_per_tile() {
        let mut w = air();
        let players = [cand(2000.0, 1500.0)];
        // centre at tile (100, 100); a solid at row 102 (inside the 4).
        solid(&mut w, 100, 102);
        let mut n = bug(1600.0 - 5.0, 1600.0 - 5.0);
        n.ai[0] = 1.0;
        n.vy = 2.0;
        let mut rng = UnifiedRandom::new(7);
        ai_115_step(&mut n, &w, &players, 0.0, 0, &mut rng);
        // The blend runs first (vy stays > 0), then the scan flips+damps.
        assert_eq!(n.ai[0], -1.0, ":47636");
        let blended = lerp(2.0, (1.0f32 as f64).sin() as f32, 0.0125);
        assert_eq!(n.vy, blended * 0.9, ":47639 — one damp");
    }

    /// `NPC.cs:47644-47665` — the rising up-scan: 30 clear rows → ONE
    /// flip + ONE damp; a solid within the 30 keeps the rise.
    #[test]
    fn w4_e26_s115_the_rising_scan_flips_once_when_clear() {
        let w = air();
        let players = [cand(2000.0, 1500.0)];
        let mut n = bug(1600.0 - 5.0, 1600.0 - 5.0);
        n.ai[0] = 1.0;
        n.vy = -2.0;
        let mut rng = UnifiedRandom::new(7);
        ai_115_step(&mut n, &w, &players, 0.0, 0, &mut rng);
        assert_eq!(n.ai[0], -1.0, ":47659");
        let blended = lerp(-2.0, (1.0f32 as f64).sin() as f32, 0.0125);
        assert_eq!(n.vy, blended * 0.9, ":47662 — one damp, not per-tile");
        // A solid 20 rows down (inside the 30): the rise stands.
        let mut w2 = air();
        solid(&mut w2, 100, 114);
        let mut n2 = bug(1600.0 - 5.0, 1600.0 - 5.0);
        n2.ai[0] = 1.0;
        n2.vy = -2.0;
        let mut rng2 = UnifiedRandom::new(7);
        ai_115_step(&mut n2, &w2, &players, 0.0, 0, &mut rng2);
        assert_eq!(n2.ai[0], 1.0, "no flip — the floor was found");
        assert_eq!(n2.vy, blended, "no damp");
    }

    /// `NPC.cs:47666-47670` — the wall bounce: ai[0] = -ai[0] + π,
    /// vx *= -0.2.
    #[test]
    fn w4_e26_s115_the_wall_bounce_turns_the_aim() {
        let w = air();
        let players = [cand(2000.0, 1500.0)];
        let mut n = bug(1600.0, 1500.0);
        n.ai[0] = 0.5;
        n.vx = 1.0;
        n.collide_x = true;
        let mut rng = UnifiedRandom::new(7);
        ai_115_step(&mut n, &w, &players, 0.0, 0, &mut rng);
        assert_eq!(n.ai[0], -0.5 + std::f32::consts::PI, ":47668");
        let blended = lerp(1.0, (0.5f32 as f64).cos() as f32, 0.0125);
        assert_eq!(n.vx, blended * -0.2, ":47669");
    }

    /// `NPC.cs:47696-47700` — the airborne perch: vx *= 0.98, vy =
    /// Lerp(vy, 2, 0.005); no aim/flight writes.
    #[test]
    fn w4_e26_s115_the_airborne_perch_damps_and_sinks() {
        let w = air();
        let players = [cand(2000.0, 1500.0)];
        let mut n = bug(1600.0, 1500.0);
        n.ai[2] = 1.0; // perch
        n.vx = 2.0;
        n.vy = 1.0;
        let mut rng = UnifiedRandom::new(7);
        ai_115_step(&mut n, &w, &players, 0.0, 0, &mut rng);
        assert_eq!(n.vx, 2.0 * 0.98, ":47698");
        assert_eq!(n.vy, lerp(1.0, 2.0, 0.005), ":47699");
        assert_eq!(n.ai[2], 1.0, "still perched");
    }

    /// `NPC.cs:47701-47711` — the grounded perch: the walk blend toward
    /// UnitX*direction, vy += 0.2, the collideX turn.
    #[test]
    fn w4_e26_s115_the_grounded_perch_walks_and_turns() {
        let w = air();
        let players = [cand(2000.0, 1500.0)];
        let mut n = bug(1600.0, 1500.0);
        n.ai[2] = 1.0;
        n.facing = 1;
        n.vx = 0.0;
        n.vy = 0.0;
        let mut rng = UnifiedRandom::new(7);
        ai_115_step(&mut n, &w, &players, 0.0, 0, &mut rng);
        assert_eq!(n.vx, lerp(0.0, 1.0, 0.05), ":47703");
        assert_eq!(n.vy, 0.2, ":47704 (Lerp(0,0,.05)=0, +0.2)");
        // The wall turn: direction flips, vx kicks back.
        let mut n2 = bug(1600.0, 1500.0);
        n2.ai[2] = 1.0;
        n2.facing = 1;
        n2.vx = 1.0;
        n2.vy = 0.0;
        n2.collide_x = true;
        let mut rng2 = UnifiedRandom::new(7);
        ai_115_step(&mut n2, &w, &players, 0.0, 0, &mut rng2);
        assert_eq!(n2.facing, -1, ":47707 — but the tail re-derives it");
        // The tail (`:47713`) re-derives facing from vx's sign: the
        // blended vx (lerp(1,1,.05)=1) *-0.2 = -0.2 < 0 → facing -1.
        assert_eq!(n2.vx, lerp(1.0, 1.0, 0.05) * -0.2, ":47708");
        assert!(n2.net_update, ":47709");
    }

    /// `NPC.cs:47674-47695` — the perch water-launch: liquid in the
    /// direction-offset down-scan → vy = -1, ai[2] = 0, the π/4 - π/2
    /// roll, netUpdate, and the RETURN (the direction tail skips).
    #[test]
    fn w4_e26_s115_the_perch_water_launch_returns_early() {
        let mut w = air();
        let players = [cand(1600.0, 1500.0)]; // near: no redirect draw
        // centre tile (100,100); facing +1 → scan column 101; liquid at
        // row 102 (inside the 4).
        let i = w.index(101, 102);
        w.tiles[i].liquid = 255;
        let mut n = bug(1600.0 - 5.0, 1600.0 - 5.0);
        n.ai[2] = 1.0;
        n.facing = 1;
        n.vy = 1.0;
        let mut rng = UnifiedRandom::new(7);
        let mut mirror = rng.clone();
        let want = mirror.next_float() * std::f32::consts::FRAC_PI_4
            - std::f32::consts::FRAC_PI_2;
        ai_115_step(&mut n, &w, &players, 0.0, 0, &mut rng);
        assert_eq!(n.vy, -1.0, ":47684");
        assert_eq!(n.ai[2], 0.0, ":47685");
        assert_eq!(n.ai[0], want, ":47686");
        assert!(n.net_update, ":47691");
        assert_eq!(rng, mirror, "one NextFloat, no redirect (near)");
        // The RETURN: facing keeps its pre-step value (the tail skipped).
        assert_eq!(n.facing, 1, ":47692 return — no direction tail");
    }

    /// `NPC.cs:47589-47618` — the rearm RNG order: Next(60,181), Next(5),
    /// then the tail NextFloat (+ conditional NextFloatDirection). With
    /// the toggle NOT firing (Next(5) != 0) only the tail roll draws.
    #[test]
    fn w4_e26_s115_the_rearm_draws_in_csharp_order() {
        let w = air();
        let players = [cand(1600.0, 1500.0)]; // near: no redirect
        let mut n = bug(1600.0, 1500.0);
        n.local_ai[0] = 1.0; // expires this tick (decrement → 0)
        // Find a seed whose Next(5) != 0 so the toggle stays out.
        let mut seed = 0;
        loop {
            let mut probe = UnifiedRandom::new(seed);
            probe.next_float(); // the scale latch is pre-set, skip
            let _ = probe.next_range(60, 181);
            if probe.next_max(5) != 0 {
                break;
            }
            seed += 1;
        }
        let mut rng = UnifiedRandom::new(seed);
        let mut mirror = rng.clone();
        let timer = mirror.next_range(60, 181) as f32;
        let _toggle = mirror.next_max(5);
        let angle = mirror.next_float() * std::f32::consts::TAU;
        ai_115_step(&mut n, &w, &players, 0.0, 0, &mut rng);
        assert_eq!(n.local_ai[0], timer, ":47591");
        assert_eq!(n.ai[0], angle, ":47613 — the tail roll");
        assert_eq!(rng, mirror, "Next(60,181) + Next(5) + NextFloat");
    }

    /// `NPC.cs:47713` — the direction tail: vx > 0 → 1, else -1 (the
    /// vx == 0 case folds to -1, the C#'s ternary).
    #[test]
    fn w4_e26_s115_the_direction_tail_reads_the_velocity_sign() {
        let w = air();
        let players = [cand(2000.0, 1500.0)];
        let mut n = bug(1600.0, 1500.0);
        n.ai[0] = 0.0; // aim (1,0) → vx blends positive
        n.vx = 0.0;
        let mut rng = UnifiedRandom::new(7);
        ai_115_step(&mut n, &w, &players, 0.0, 0, &mut rng);
        assert!(n.vx > 0.0);
        assert_eq!(n.facing, 1, ":47713");
    }
}
}
