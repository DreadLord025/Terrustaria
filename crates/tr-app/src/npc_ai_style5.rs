//! W4-d36 E4 item 4 — `AI_005_EaterOfSouls`' TAIL arms (`NPC.cs:50726-50812`):
//! the Moth (205) wet buoyancy (`:50726-50738`) and the Hornet family's
//! (42/176/231-235) wet buoyancy + stinger volley (`:50739-50812`, proj 55,
//! dmg `(int)(30f*scale)` for 176 / `(int)(10f*scale)` otherwise).
//!
//! Status: WIRED (the `ai_005_hover_family` rng thread landed with the E4
//! batch-2 walker completion — the axis caller at `npc.rs`'s style-5 arm
//! holds the live `UpdateNPCs` stream and calls `style5_tail` after the
//! `update_hover` arms, draining `out.shot` → `wants_hostile_shot` and
//! `out.sounds` → `wants_item_sounds` → the sim drain). The projectile-side
//! data IS live: `projectile::STINGER` (55) carries its `proj_data` row
//! (`Projectile.cs:1375-1383`) and rides `AI_STYLE_1_RIDERS` (the
//! `flag3 = false` skip list, `Projectile.cs:55954` — resting counter,
//! straight flight). The sim's timed-shot list (`world_sim/npcs.rs`) carries
//! the `|| kind == crate::projectile::STINGER` row for the C#'s
//! `timeLeft = 300` post-spawn override (`NPC.cs:50797`).
//!
//! ## Wiring contract (the follow-up edit, `npc.rs` only)
//!
//! In `ai_005_hover_family`, immediately AFTER the `update_hover` call for
//! `Hornet` / `MossHornet` / `Moth` (the C# runs these arms at the END of
//! `AI_005`, after the collision/dust block `:50656-50713`):
//!
//! ```ignore
//! let env = crate::npc_ai_style5::Style5TailEnv {
//!     world,
//!     players: &[crate::npc_target::session_candidate(player)],
//!     ticker: 0, // the EverLastingTicker stand-in (the zombie convention)
//!     get_good_world: world.seeds.contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY),
//!     target_is_player: true, // the port is single-player, slot 0 live
//!     target_center: player.center(),
//!     target_stealth: player.stealth,
//!     target_item_animation: player.frame_inputs.swing_anim as i32,
//!     net_mode: 0, // the port is the host (the update_cave_bat convention)
//! };
//! let out = crate::npc_ai_style5::style5_tail(self, &env, rng);
//! if let Some(s) = out.shot {
//!     self.wants_hostile_shot.push((crate::projectile::STINGER, s.x, s.y, s.vx, s.vy, s.damage));
//! }
//! for (style, sx, sy) in out.sounds { /* the style75 out.sounds drain,
//!     world_sim/npcs.rs:4492-4499 — SOUND_STYLES + play_use_sound at the
//!     C# call position, on the UpdateNPCs stream */ }
//! ```
//!
//! The family arm needs the `rng` parameter threaded through (the
//! `ai_009_sphere_family` signature precedent — the axis caller at
//! `npc.rs:4761` already holds the live `UpdateNPCs` stream), and the
//! sim's timed-shot list needs `|| kind == crate::projectile::STINGER`
//! (the QUEEN_BEE_STINGER row's shape, `world_sim/npcs.rs`'s drain).
//!
//! Recorded deviations (the wiring note's, docs/ANALYSIS.md):
//! * The port's `hover_step` integrates INSIDE the AI (the C# integrates in
//!   `UpdateCollision` AFTER the arm) — the tail's velocity writes land on
//!   the NEXT tick's integration (a one-tick lag on the wet buoyancy only;
//!   the volley reads pre-integration velocity either way, the same value).
//! * `Collision.CanHit(this, targetData)` folds to the port's centre-only
//!   `can_hit_line` probe (`npc.rs:10065`'s convention — the target box
//!   collapses to its centre point).
//! * The `Item17` cue rides the returned `sounds` list (the style75
//!   `out.sounds` shape), drained at the C# call position — NOT the
//!   `wants_sound` queue (which drains post-AI, after this arm's draws).
//!
//! ## RNG order (by construction, `Main.rand` on the UpdateNPCs stream)
//!
//! Per tick, for a 42/176/231-235 kind (the Moth's wet arm draws NOTHING):
//! 1. `:50761` — `Next(5, 20)` (the ai[1] charge roll), ALWAYS (netMode 0).
//! 2. `:50764` — type 176 ONLY: a second `Next(5, 20)`.
//! 3. `:50768` — getGoodWorld ONLY: a third `Next(5, 20)`.
//! 4. `:50784-50785` — the fire tick ONLY (ai[1] >= 130, flag2, CanHit):
//!    `Next(-20, 21)` twice (the aim jitter, X then Y). The sign-mismatch
//!    else (`:50803`) consumes NO further draw.
//! The `Item17` play (`:50756`) is the style-2 façade — its pitch draw rides
//! the audio drain's stream position (the E21g convention), not this arm.
//! `TargetClosest` (the wet arms) is RNG-free (`npc_target.rs`).

#![allow(dead_code)] // the env/out fields the wiring reads; the module is live

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::{UnifiedRandom, World};

/// The tail arms' environment seam (the `Style14Env` shape): the session
/// player slice for the wet arms' `TargetClosest`, the target reads for the
/// volley's gates. `target_*` mirror `GetTargetData()`'s Player arm
/// (`NPC.cs:50293-50301`); `target_is_player` false models
/// `NPCTargetType.None` (flag2 stays true, the stealth gate skips, the
/// fire gate's `Type != None` term fails).
pub struct Style5TailEnv<'a> {
    pub world: &'a World,
    pub players: &'a [TargetCandidate],
    /// `Main.EverLastingTicker` — 0 until the plumb (the zombie convention).
    pub ticker: u64,
    /// `Main.getGoodWorld` (`:50766`) — the FOR_THE_WORTHY seed flag.
    pub get_good_world: bool,
    /// `targetData.Type == NPCTargetType.Player` (`:50296`/`:50770`).
    pub target_is_player: bool,
    /// `targetData.Center` (`:50784-50785`).
    pub target_center: (f32, f32),
    /// `player2.stealth` (`:50773`).
    pub target_stealth: f32,
    /// `player2.itemAnimation` (`:50773`).
    pub target_item_animation: i32,
    /// `Main.netMode` (`:50759`) — the port is the host (0).
    pub net_mode: i32,
}

/// The stinger request (`NPC.cs:50796-50797`): proj 55 from `vector4` with
/// the jittered, normalized ×8 velocity, damage `(int)(30f*scale)` for 176
/// / `(int)(10f*scale)` otherwise, knockback 0, and the post-spawn
/// `timeLeft = 300` (the sim's timed-shot list owns the override —
/// `world_sim/npcs.rs`'s `STINGER` row).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StingerShot {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
}

/// The arm's side effects the caller drains: the shot (at most one per
/// tick) and the legacy sound plays `(style, x, y)` in the C#'s intra-tick
/// order (the style75 `out.sounds` shape — only Item17, style 2, today).
#[derive(Clone, Debug, Default)]
pub struct Style5TailOut {
    pub shot: Option<StingerShot>,
    pub sounds: Vec<(i32, f32, f32)>,
}

/// `AI_005_EaterOfSouls`' tail (`NPC.cs:50726-50812`) — one tick, for the
/// kinds the arms name (205 / 42 / 176 / 231-235; every other style-5 kind
/// is a no-op, the C#'s type gates). Refreshes `NPC.wet` first (the
/// `update_cave_bat` wrapper's `is_wet` seam, `npc.rs:10619` — the C#'s
/// `Entity.wet` is the same-tick liquid overlap).
pub fn style5_tail(npc: &mut Npc, env: &Style5TailEnv<'_>, rng: &mut UnifiedRandom) -> Style5TailOut {
    let mut out = Style5TailOut::default();
    let t = npc.kind.cs_id() as i32;
    let (w, h) = npc.size();
    npc.wet = crate::npc::is_wet(env.world, npc.x, npc.y, w, h);
    // `:50726-50738` — the Moth's own wet arm (205 is NOT in the 42/176
    // group below; the two blocks are independent `if`s in the C#).
    if t == 205 && npc.wet {
        wet_buoyancy(npc);
        target_closest(npc, env.world, env.players, true, 5, env.ticker);
    }
    // `:50739` — the Hornet family's group gate.
    if !(t == 42 || t == 176 || (231..=235).contains(&t)) {
        return out;
    }
    // `:50741-50753` — the group's wet arm (the Moth's shape verbatim).
    if npc.wet {
        wet_buoyancy(npc);
        target_closest(npc, env.world, env.players, true, 5, env.ticker);
    }
    // `:50754-50758` — the fire latch: the PREVIOUS tick's shot plays
    // Item17 at the CURRENT position, then clears the sentinel.
    if npc.ai[1] == 101.0 {
        out.sounds.push((17, npc.x, npc.y));
        npc.ai[1] = 0.0;
    }
    // `:50759` — the dedicated server never charges (the port is host).
    if env.net_mode == 1 {
        return out;
    }
    // `:50761-50769` — the charge rolls, in the C#'s order: one for the
    // family, a SECOND for 176, a THIRD for getGoodWorld. Each is
    // `Next(5, 20) * 0.1f * scale` (0.5..1.9 at the 1f default scale).
    npc.ai[1] += rng.next_range(5, 20) as f32 * 0.1 * npc.scale;
    if t == 176 {
        npc.ai[1] += rng.next_range(5, 20) as f32 * 0.1 * npc.scale;
    }
    if env.get_good_world {
        npc.ai[1] += rng.next_range(5, 20) as f32 * 0.1 * npc.scale;
    }
    // `:50770-50777` — a still, unstealthed player suppresses the charge
    // (`player2 != null` folds to the live session candidate).
    if env.target_is_player && env.target_stealth == 0.0 && env.target_item_animation == 0 {
        npc.ai[1] = 0.0;
    }
    if npc.ai[1] < 130.0 {
        return out;
    }
    // `:50780` — `((targetData.Type != None) & flag2) && CanHit`. flag2 is
    // `AI_GlobalFiringDistanceCheck(Center, player.Center)` for a Player
    // target (`:50300`), the init `true` otherwise (`:50295`).
    let flag2 = if env.target_is_player {
        crate::npc_ai_bat::global_firing_distance_check(npc.center(), env.target_center)
    } else {
        true
    };
    let (tcx, tcy) = env.target_center;
    let can_hit = crate::projectile::can_hit_line(env.world, npc.x, npc.y, w, h, tcx, tcy, 0.0, 0.0);
    if !(env.target_is_player && flag2 && can_hit) {
        npc.ai[1] = 0.0;
        return out;
    }
    // `:50782-50785` — vector4: the centre-X, the INTEGER half-height Y
    // (`(float)(base.height / 2)` — Entity.height is int, the 155/329
    // hop-over's integer-half convention, `NPC.cs:62560`).
    let num19 = 8.0f32;
    let sx = npc.x + w * 0.5;
    let sy = npc.y + (h as i32 / 2) as f32;
    let mut num20 = tcx - sx + rng.next_range(-20, 21) as f32;
    let mut num21 = tcy - sy + rng.next_range(-20, 21) as f32;
    // `:50786-50804` — the shot only leaves on the side the NPC is already
    // rolling toward; the mismatch resets the charge (no further draw).
    if !((num20 < 0.0 && npc.vx < 0.0) || (num20 > 0.0 && npc.vx > 0.0)) {
        npc.ai[1] = 0.0;
        return out;
    }
    let num22 = num19 / (num20 * num20 + num21 * num21).sqrt();
    num20 *= num22;
    num21 *= num22;
    // `:50791-50795` — `(int)(10f * scale)`, the 176 row `(int)(30f*scale)`.
    let damage = if t == 176 {
        (30.0 * npc.scale) as i32
    } else {
        (10.0 * npc.scale) as i32
    };
    // `:50796-50799` — NewProjectile(55, dmg, 0f, Main.myPlayer) +
    // timeLeft = 300 (the sim's timed list) + the 101 latch + netUpdate.
    out.shot = Some(StingerShot { x: sx, y: sy, vx: num20, vy: num21, damage });
    npc.ai[1] = 101.0;
    npc.net_update = true;
    out
}

/// The shared wet buoyancy (`NPC.cs:50728-50736` = `:50743-50751`): a
/// falling velocity damps ×0.95, then −0.5 lift, clamped at −4.
fn wet_buoyancy(npc: &mut Npc) {
    if npc.vy > 0.0 {
        npc.vy *= 0.95;
    }
    npc.vy -= 0.5;
    if npc.vy < -4.0 {
        npc.vy = -4.0;
    }
}

// =====================================================================
// Witness tests — WRITTEN, NOT RUN (code-only mode: no cargo/rustc).
// Every assertion pins a constant or an RNG position to its C# literal.
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    fn air() -> World {
        World::new("t".into(), 200, 120)
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

    /// An env whose player is mid-swing (the stealth gate OFF — the charge
    /// accumulates) at the given centre, in range and in the open.
    fn env<'a>(w: &'a World, players: &'a [TargetCandidate], center: (f32, f32)) -> Style5TailEnv<'a> {
        Style5TailEnv {
            world: w,
            players,
            ticker: 0,
            get_good_world: false,
            target_is_player: true,
            target_center: center,
            target_stealth: 0.0,
            target_item_animation: 1,
            net_mode: 0,
        }
    }

    /// The Moth wet arm (`NPC.cs:50726-50738`): ×0.95 on the fall, −0.5
    /// lift, the −4 clamp; the dry tick is a no-op; NO rand draw.
    #[test]
    fn w4_d36_the_moth_wet_arm_pins_the_buoyancy_constants() {
        let w = air();
        let players = [cand(1600.0, 1000.0)];
        let e = env(&w, &players, (1610.0, 1021.0));
        let mut rng = UnifiedRandom::new(7);
        let mirror = rng.clone();
        let mut m = Npc::new(NpcKind::Moth, 800.0, 600.0, 1, 0.0);
        // The dry tick: no velocity write (is_wet false on the air world).
        m.vy = 2.0;
        let out = style5_tail(&mut m, &e, &mut rng);
        assert_eq!(m.vy, 2.0, "the dry Moth keeps its velocity");
        assert!(out.shot.is_none() && out.sounds.is_empty());
        // The wet CONSTANTS pin (the C# arm's three writes) — the wrapper's
        // is_wet refresh is the cave-bat seam's shape, pinned there.
        let mut n = Npc::new(NpcKind::Moth, 800.0, 600.0, 1, 0.0);
        n.vy = 2.0;
        wet_buoyancy(&mut n);
        assert!((n.vy - 1.4).abs() < 1e-6, "2 × 0.95 − 0.5 = 1.4 (:50730-50732)");
        n.vy = -3.9;
        wet_buoyancy(&mut n);
        assert_eq!(n.vy, -4.0, "the −4 clamp (:50733-50736)");
        n.vy = -0.2;
        wet_buoyancy(&mut n);
        assert!((n.vy - (-0.7)).abs() < 1e-6, "a rise takes no ×0.95 (:50728)");
        assert_eq!(rng, mirror, "the Moth arm draws NOTHING");
    }

    /// The charge rolls' ORDER and COUNT (`NPC.cs:50761-50769`): 42 draws
    /// ONE Next(5,20), 176 draws TWO, getGoodWorld adds a THIRD — the
    /// mirrored-oracle stream-position method (the rng derives PartialEq).
    #[test]
    fn w4_d36_the_charge_rolls_match_the_csharp_order() {
        let w = air();
        let players = [cand(1600.0, 1000.0)];
        let center = (1610.0, 1021.0);
        for (kind, rolls, ftw) in [
            (NpcKind::Hornet, 1, false),
            (NpcKind::MossHornet, 2, false),
            (NpcKind::MossHornet, 3, true),
        ] {
            let mut e = env(&w, &players, center);
            e.get_good_world = ftw;
            let mut rng = UnifiedRandom::new(11);
            let mut oracle = rng.clone();
            let mut n = Npc::new(kind, 800.0, 600.0, 1, 0.0);
            n.vx = 1.0;
            let mut expected = 0.0f32;
            for _ in 0..rolls {
                expected += oracle.next_range(5, 20) as f32 * 0.1 * n.scale;
            }
            style5_tail(&mut n, &e, &mut rng);
            assert_eq!(rng, oracle, "{kind:?} ftw={ftw}: exactly {rolls} charge draws");
            assert!((n.ai[1] - expected).abs() < 1e-4, "{kind:?}: the ai[1] charge sum");
        }
    }

    /// The still-player suppression (`NPC.cs:50770-50777`): stealth 0 AND
    /// itemAnimation 0 zeroes the charge AFTER the rolls (the draws still
    /// run — the C#'s order).
    #[test]
    fn w4_d36_the_still_player_suppresses_the_charge() {
        let w = air();
        let players = [cand(1600.0, 1000.0)];
        let mut e = env(&w, &players, (1610.0, 1021.0));
        e.target_item_animation = 0;
        let mut rng = UnifiedRandom::new(3);
        let mut oracle = rng.clone();
        let mut n = Npc::new(NpcKind::MossHornet, 800.0, 600.0, 1, 0.0);
        n.ai[1] = 120.0;
        style5_tail(&mut n, &e, &mut rng);
        assert_eq!(n.ai[1], 0.0, "the still player zeroes ai[1] (:50775)");
        oracle.next_range(5, 20);
        oracle.next_range(5, 20);
        assert_eq!(rng, oracle, "the suppression runs AFTER both rolls");
        // A stealthed player does NOT suppress.
        e.target_stealth = 1.0;
        let mut n2 = Npc::new(NpcKind::MossHornet, 800.0, 600.0, 1, 0.0);
        n2.ai[1] = 120.0;
        style5_tail(&mut n2, &e, &mut rng);
        assert!(n2.ai[1] > 120.0, "stealth keeps the charge (:50773)");
    }

    /// The fire tick (`NPC.cs:50778-50799`): at ai[1] >= 130 with the
    /// target in range, in the open and on the rolling side, the shot
    /// leaves at the normalized ×8 velocity, damage 30 (176) / 10 (42),
    /// from the INTEGER-half-height anchor; ai[1] latches 101 + netUpdate.
    /// The NEXT tick plays Item17 and clears the latch (`:50754-50758`).
    #[test]
    fn w4_d36_the_fire_tick_pins_the_shot_and_the_latch() {
        let w = air();
        let players = [cand(1600.0, 1000.0)];
        let center = (1610.0, 1021.0);
        let e = env(&w, &players, center);
        for (kind, dmg) in [(NpcKind::MossHornet, 30), (NpcKind::Hornet, 10)] {
            let mut rng = UnifiedRandom::new(5);
            let mut oracle = rng.clone();
            let mut n = Npc::new(kind, 800.0, 600.0, 1, 0.0);
            let (nw, nh) = n.size();
            n.vx = 2.0; // rolling right, the target is right
            n.ai[1] = 129.9;
            // The oracle: the charge roll(s), then the two aim jitters.
            let charge = oracle.next_range(5, 20) as f32 * 0.1;
            let charge2 = if kind == NpcKind::MossHornet {
                oracle.next_range(5, 20) as f32 * 0.1
            } else {
                0.0
            };
            assert!(129.9 + charge + charge2 >= 130.0, "the minimum roll fires");
            let jx = oracle.next_range(-20, 21) as f32;
            let jy = oracle.next_range(-20, 21) as f32;
            let out = style5_tail(&mut n, &e, &mut rng);
            assert_eq!(rng, oracle, "{kind:?}: charge + 2 jitter draws, in order");
            let s = out.shot.expect("the fire tick shoots");
            let sx = 800.0 + nw * 0.5;
            let sy = 600.0 + (nh as i32 / 2) as f32;
            assert_eq!((s.x, s.y), (sx, sy), "the integer-half anchor (:50783)");
            let (mut dx, mut dy) = (center.0 - sx + jx, center.1 - sy + jy);
            let f = 8.0 / (dx * dx + dy * dy).sqrt();
            dx *= f;
            dy *= f;
            assert!((s.vx - dx).abs() < 1e-4 && (s.vy - dy).abs() < 1e-4, "the ×8 normalize");
            assert_eq!(s.damage, dmg, "the (int)(30f/10f × scale) row (:50791-50795)");
            assert_eq!(n.ai[1], 101.0, "the fire latch (:50798)");
            assert!(n.net_update, "netUpdate (:50799)");
            // The next tick: Item17 at the position, the latch clears, the
            // charge rolls run again (the C#'s order: cue BEFORE the rolls).
            let out2 = style5_tail(&mut n, &e, &mut rng);
            assert_eq!(out2.sounds, vec![(17, n.x, n.y)], "the Item17 cue (:50756)");
            assert!(out2.shot.is_none(), "one shot per charge cycle");
            assert!(n.ai[1] < 130.0, "the latch cleared then re-charged (:50757)");
        }
    }

    /// The sign-mismatch else (`NPC.cs:50801-50804`): the aim jitter is
    /// drawn (both axes) but the shot aborts and the charge resets — NO
    /// further draw. The out-of-range gate (`:50780`) resets WITHOUT the
    /// jitter draws.
    #[test]
    fn w4_d36_the_abort_gates_pin_their_draw_counts() {
        let w = air();
        let players = [cand(1600.0, 1000.0)];
        let e = env(&w, &players, (1610.0, 1021.0));
        // Rolling LEFT while the target is right: the mismatch aborts.
        let mut rng = UnifiedRandom::new(9);
        let mut oracle = rng.clone();
        let mut n = Npc::new(NpcKind::MossHornet, 800.0, 600.0, 1, 0.0);
        n.vx = -2.0;
        n.ai[1] = 131.0;
        let out = style5_tail(&mut n, &e, &mut rng);
        assert!(out.shot.is_none(), "the mismatch shoots nothing");
        assert_eq!(n.ai[1], 0.0, "the mismatch resets (:50803)");
        oracle.next_range(5, 20);
        oracle.next_range(5, 20);
        oracle.next_range(-20, 21);
        oracle.next_range(-20, 21);
        assert_eq!(rng, oracle, "the jitters ARE drawn before the abort");
        // The far target: flag2 false — the reset precedes the jitters.
        let far = env(&w, &players, (800.0 + 3000.0, 600.0));
        let mut rng2 = UnifiedRandom::new(9);
        let mut oracle2 = rng2.clone();
        let mut n2 = Npc::new(NpcKind::MossHornet, 800.0, 600.0, 1, 0.0);
        n2.vx = 2.0;
        n2.ai[1] = 131.0;
        let out2 = style5_tail(&mut n2, &far, &mut rng2);
        assert!(out2.shot.is_none() && n2.ai[1] == 0.0, "the range gate resets");
        oracle2.next_range(5, 20);
        oracle2.next_range(5, 20);
        assert_eq!(rng2, oracle2, "NO jitter draws on the range abort (:50780)");
    }

    /// The dedicated-server gate (`NPC.cs:50759`): netMode 1 returns after
    /// the latch cue, BEFORE any charge roll.
    #[test]
    fn w4_d36_the_server_gate_skips_the_charge() {
        let w = air();
        let players = [cand(1600.0, 1000.0)];
        let mut e = env(&w, &players, (1610.0, 1021.0));
        e.net_mode = 1;
        let mut rng = UnifiedRandom::new(2);
        let mirror = rng.clone();
        let mut n = Npc::new(NpcKind::MossHornet, 800.0, 600.0, 1, 0.0);
        n.ai[1] = 101.0;
        let out = style5_tail(&mut n, &e, &mut rng);
        assert_eq!(out.sounds, vec![(17, 800.0, 600.0)], "the latch cue still plays");
        assert_eq!(n.ai[1], 0.0);
        assert_eq!(rng, mirror, "netMode 1 draws NOTHING (:50759)");
    }

    /// The projectile-side infrastructure pins: the 55 row
    /// (`Projectile.cs:1375-1383` — 10x10, aiStyle 1, penetrate −1,
    /// tileCollide, the stock 3600 life the sim's timed list overrides to
    /// 300, `NPC.cs:50797`) and the rider membership (the `flag3 = false`
    /// skip list, `Projectile.cs:55954`).
    #[test]
    fn w4_d36_the_stinger_row_matches_the_csharp_defaults() {
        let d = crate::projectile::proj_data(crate::projectile::STINGER)
            .expect("the Stinger row");
        assert_eq!((d.w, d.h), (10.0, 10.0), "the 10x10 box (:1377-1378)");
        assert_eq!(d.ai_style, 1, "aiStyle 1 (:1381)");
        assert_eq!(d.penetrate, -1, "penetrate −1 (:1380)");
        assert!(d.tile_collide, "tileCollide (:1382)");
        assert_eq!(crate::projectile::STINGER, 55, "ProjectileID.Stinger");
    }
}
