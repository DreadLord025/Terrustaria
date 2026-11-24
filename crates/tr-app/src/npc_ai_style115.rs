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

/// `Entity.Center` (`Entity.cs:50-55`) for a player candidate.
fn candidate_center(p: &TargetCandidate) -> (f32, f32) {
    (p.x + p.width as f32 / 2.0, p.y + p.height as f32 / 2.0)
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
        // `:47666-47670` — the wall bounce.
        if npc.collide_x {
            npc.ai[0] = 0.0 - npc.ai[0] + std::f32::consts::PI;
            npc.vx *= -0.2;
        }
    } else {
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
    // `:47713` — the direction tail (every tick but the launch return).
    npc.facing = if npc.vx > 0.0 { 1 } else { -1 };
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
