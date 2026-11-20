//! The aiStyle-91 machine — the Granite Flyer (`NPC.AI()`'s
//! `else if (this.aiStyle == 91)` arm, `NPC.cs:38629-38809`). W4-d32,
//! research/25 row 91 (MISSING → ported), CODE-ONLY MODE (no compiler
//! invoked; the witness tests below are written, not run).
//!
//! ## Census
//!
//! The census of `this.aiStyle = 91;` across the decompiled tree
//! returns exactly ONE assignment: `NPC.cs:14567`, the
//! `else if (this.type == 483)` row (`:14562-14575`) — the
//! GraniteFlyer (`ID/NPCID.cs:11639`). No dynamic `aiStyle`
//! reassignment to 91 exists. The kind has no `NpcKind` yet — the
//! module lands UNWIRED (the style73/style86 precedent).
//!
//! ## The machine
//!
//! The head (`:38631-38633`): `noGravity = true`,
//! `noTileCollide = false`, `dontTakeDamage = false`.
//!
//! The expert flinch (`:38634-38639`): justHit + expert +
//! `Next(6) == 0` → netUpdate, state −1, ai[1] = 0.
//!
//! State −1 burrow (`:38640-38650`): dontTakeDamage, noGravity
//! FALSE (it falls), vx ×0.98, `ai[1]++`; at ≥120 the chained
//! `ai[0] = ai[1] = ai[2] = ai[3] = 0` reset (verbatim).
//!
//! State 0 seek (`:38651-38698`): retarget; LOS → state 1 + RETURN.
//! Else the (player −height/4 Y) delta (the INT division), the
//! X-aligned waypoint probe (`center33`), the Y-aligned override
//! (`center34` — needs BOTH the flyer→waypoint and the
//! waypoint→player.position LOS), the else-branch Y-aligned probe;
//! any hit → state 3 with the waypoint in ai[1]/ai[2]. Still 0 →
//! localAI[0] = 0, the normalized delta ×0.5 kick, state 4.
//!
//! State 1 attack (`:38699-38712`): the `2 + len/200` ×50-weight
//! steer; LOS-lost → state 0.
//!
//! State 2 (`:38713-38727`): noTileCollide, the ×2 / 4-weight
//! steer, <600 + free-space → 0. NO arm writes ai[0] = 2 — the
//! state is externally reachable only (kept verbatim).
//!
//! State 3 waypoint (`:38728-38746`): the ×1 / 3-weight steer at
//! (ai[1], ai[2]); collideX/Y → state 4; arrived (<1) / stale
//! (>800) / LOS-gained → state 0.
//!
//! State 4 wander (`:38747-38808`): the collide ×−0.8 bounces; a
//! zero velocity re-kicks along the (player −height/4) delta ×0.1;
//! the normalized-velocity ×1.5 / 20-weight steer; >180 ticks → 0;
//! LOS → 0; every 5 localAI ticks (unless inside the ±10 inflated
//! solid box) the waypoint re-probe — the X-aligned triple-CanHit
//! (the DUPLICATE second test verbatim, `:38793`) then the
//! Y-aligned double-CanHit → state 3.
//!
//! ## RNG census (by construction)
//!
//! ONE draw site: the expert justHit `Next(6)` (`:38634`) — gated
//! by justHit AND expert (the C# `&&` short-circuit: a classic hit
//! draws NOTHING). Everything else (TargetClosest, the CanHit
//! probes, the steers) is deterministic.
//!
//! ## Seams (the wiring obligations)
//!
//! * The flyer integrates with tile collision under gravity-off
//!   flight (the head's noGravity) — the caller's movement step;
//!   the burrow's noGravity=false falls (the outcome flags).
//! * `Collision.CanHit` rides `crate::projectile::can_hit` (the
//!   ported LOS walk); the waypoint probes pass 1×1 boxes.
//! * `Main.expertMode` rides [`Style91Env::expert`].
//!
//! See docs/ANALYSIS.md "The Granite Flyer (W4-d32 aiStyle-91, code-only)".
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::{UnifiedRandom, World};

/// The type-483 `SetDefaults` row (`NPC.cs:14562-14575`), verbatim.
pub struct GraniteFlyerDefaults {
    pub npc_slots: f32,
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit7` (`ID/SoundID.cs:1263`).
    pub hit_sound: (i32, i32),
    pub knock_back_resist: f32,
    /// `(SoundType, Style)` — `SoundID.NPCDeath6` (`ID/SoundID.cs:1320`).
    pub death_sound: (i32, i32),
    pub value: f32,
}

/// `NPC.cs:14562-14575` — `else if (this.type == 483)`.
pub const GRANITE_FLYER: GraniteFlyerDefaults = GraniteFlyerDefaults {
    npc_slots: 0.5,
    width: 20,
    height: 30,
    ai_style: 91,
    damage: 24,
    defense: 8,
    life_max: 40,
    hit_sound: (3, 7),
    knock_back_resist: 0.6,
    death_sound: (4, 6),
    value: 1000.0,
};

/// The per-tick environment.
pub struct Style91Env<'a> {
    pub world: &'a World,
    pub players: &'a [TargetCandidate],
    pub ticker: u64,
    /// `Main.expertMode` — the flinch gate (`:38634`).
    pub expert: bool,
    pub rng: &'a mut UnifiedRandom,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Default, Debug, PartialEq)]
pub struct Style91Outcome {
    /// The head's `noGravity = true` (`:38631`); the burrow's false
    /// (`:38643`).
    pub no_gravity: bool,
    /// The head's `noTileCollide = false` (`:38632`); state 2's true
    /// (`:38715`).
    pub no_tile_collide: bool,
    /// The head's `dontTakeDamage = false` (`:38633`); the burrow's
    /// true (`:38642`) — mirrored for the witnesses.
    pub dont_take_damage: bool,
}

/// `(float)Math.Sqrt(x*x + y*y)` — XNA `Vector2.Length()` (the
/// style74 `sqrt_f32` shape).
fn sqrt_f32(x: f32, y: f32) -> f32 {
    ((x * x + y * y) as f64).sqrt() as f32
}

/// XNA `Vector2.Normalize()` — UNGUARDED (the style74 precedent).
fn normalize_xna(x: f32, y: f32) -> (f32, f32) {
    let num = 1.0f32 / ((x * x + y * y) as f64).sqrt() as f32;
    (x * num, y * num)
}

/// The target slot's candidate — the style74 `target_of` fallback.
fn target_of<'a>(npc: &Npc, players: &'a [TargetCandidate]) -> &'a TargetCandidate {
    let i = if npc.target >= 0 && (npc.target as usize) < players.len() {
        npc.target as usize
    } else {
        0
    };
    &players[i]
}

fn candidate_center(p: &TargetCandidate) -> (f32, f32) {
    (p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32)
}

/// `Collision.CanHit(a,1,1,b,1,1)` between two points — the ported
/// LOS walk.
fn can_hit_points(world: &World, ax: f32, ay: f32, bx: f32, by: f32) -> bool {
    crate::projectile::can_hit(world, ax, ay, 1.0, 1.0, bx, by, 1.0, 1.0)
}

/// One fixed step of the aiStyle-91 arm (`NPC.cs:38629-38809`), in
/// C# order. `npc` is the Granite Flyer (type 483). The caller owns
/// the movement integration (the outcome flags) after `AI()`.
pub fn style91_step(npc: &mut Npc, env: &mut Style91Env<'_>) -> Style91Outcome {
    let mut out = Style91Outcome {
        no_gravity: true,
        no_tile_collide: false,
        dont_take_damage: false,
    };
    // `:38631-38633` — the head.
    npc.dont_take_damage = false;
    // `:38634-38639` — the expert flinch (netMode != 1 folds true;
    // the && short-circuit keeps the draw expert-only).
    if npc.just_hit && env.expert && env.rng.next_max(6) == 0 {
        npc.net_update = true;
        npc.ai[0] = -1.0;
        npc.ai[1] = 0.0;
    }
    if npc.ai[0] == -1.0 {
        // `:38640-38650` — the burrow.
        npc.dont_take_damage = true;
        out.dont_take_damage = true;
        out.no_gravity = false;
        npc.vx *= 0.98;
        npc.ai[1] += 1.0;
        if npc.ai[1] >= 120.0 {
            // The chained assignment, verbatim (`:38648`).
            npc.ai[3] = 0.0;
            npc.ai[2] = npc.ai[3];
            npc.ai[1] = npc.ai[2];
            npc.ai[0] = npc.ai[1];
        }
    } else if npc.ai[0] == 0.0 {
        // `:38651-38698` — the seek.
        target_closest(npc, env.world, env.players, true, 91, env.ticker);
        let p = target_of(npc, env.players);
        let (pcx, pcy) = candidate_center(p);
        let (cx, cy) = npc.center();
        if can_hit_points(env.world, cx, cy, pcx, pcy) {
            npc.ai[0] = 1.0;
            return out; // the C# `return` (`:38657`)
        }
        // `:38659-38661` — the (player −height/4) delta (INT /4).
        let mut dx = pcx - cx;
        let mut dy = pcy - cy;
        dy -= (p.height / 4) as f32;
        // `:38662-38678` — the X-aligned probe + the Y-aligned override.
        let (w33x, w33y) = (pcx, cy);
        let v189_len = sqrt_f32(w33x - cx, w33y - cy);
        if v189_len > 8.0 && can_hit_points(env.world, cx, cy, w33x, w33y) {
            npc.ai[0] = 3.0;
            npc.ai[1] = w33x;
            npc.ai[2] = w33y;
            let (w34x, w34y) = (cx, pcy);
            if v189_len > 8.0
                && can_hit_points(env.world, cx, cy, w34x, w34y)
                && can_hit_points(env.world, w34x, w34y, p.x, p.y)
            {
                npc.ai[0] = 3.0;
                npc.ai[1] = w34x;
                npc.ai[2] = w34y;
            }
        } else {
            // `:38679-38689` — the else-branch Y-aligned probe.
            let (w33x, w33y) = (cx, pcy);
            if sqrt_f32(w33x - cx, w33y - cy) > 8.0
                && can_hit_points(env.world, cx, cy, w33x, w33y)
            {
                npc.ai[0] = 3.0;
                npc.ai[1] = w33x;
                npc.ai[2] = w33y;
            }
        }
        if npc.ai[0] == 0.0 {
            // `:38690-38697` — the wander kick.
            npc.local_ai[0] = 0.0;
            let (nx, ny) = normalize_xna(dx, dy);
            npc.vx += nx * 0.5;
            npc.vy += ny * 0.5;
            npc.ai[0] = 4.0;
            npc.ai[1] = 0.0;
        }
    } else if npc.ai[0] == 1.0 {
        // `:38699-38712` — the attack.
        let p = target_of(npc, env.players);
        let (pcx, pcy) = candidate_center(p);
        let (cx, cy) = npc.center();
        let dx = pcx - cx;
        let dy = pcy - cy;
        let num1266 = 2.0 + sqrt_f32(dx, dy) / 200.0;
        let num1267 = 50.0;
        let (nx, ny) = normalize_xna(dx, dy);
        npc.vx = (npc.vx * (num1267 - 1.0) + nx * num1266) / num1267;
        npc.vy = (npc.vy * (num1267 - 1.0) + ny * num1266) / num1267;
        if !can_hit_points(env.world, cx, cy, pcx, pcy) {
            npc.ai[0] = 0.0;
            npc.ai[1] = 0.0;
        }
    } else if npc.ai[0] == 2.0 {
        // `:38713-38727` — the externally-reachable state (no arm
        // writes 2; kept verbatim).
        out.no_tile_collide = true;
        let p = target_of(npc, env.players);
        let (pcx, pcy) = candidate_center(p);
        let (cx, cy) = npc.center();
        let dx = pcx - cx;
        let dy = pcy - cy;
        let num1268 = sqrt_f32(dx, dy);
        let (nx, ny) = normalize_xna(dx, dy);
        npc.vx = (npc.vx * (4.0 - 1.0) + nx * 2.0) / 4.0;
        npc.vy = (npc.vy * (4.0 - 1.0) + ny * 2.0) / 4.0;
        let (w, h) = npc.size();
        if num1268 < 600.0
            && !crate::player::Player::solid_collision(env.world, npc.x, npc.y, w, h)
        {
            npc.ai[0] = 0.0;
        }
    } else if npc.ai[0] == 3.0 {
        // `:38728-38746` — the waypoint.
        let (cx, cy) = npc.center();
        let dx = npc.ai[1] - cx;
        let dy = npc.ai[2] - cy;
        let num1271 = sqrt_f32(dx, dy);
        let (nx, ny) = normalize_xna(dx, dy);
        npc.vx = (npc.vx * (3.0 - 1.0) + nx * 1.0) / 3.0;
        npc.vy = (npc.vy * (3.0 - 1.0) + ny * 1.0) / 3.0;
        if npc.collide_x || npc.collide_y {
            npc.ai[0] = 4.0;
            npc.ai[1] = 0.0;
        }
        let p = target_of(npc, env.players);
        let (pcx, pcy) = candidate_center(p);
        if num1271 < 1.0 || num1271 > 800.0 || can_hit_points(env.world, cx, cy, pcx, pcy) {
            npc.ai[0] = 0.0;
        }
    } else if npc.ai[0] == 4.0 {
        // `:38747-38808` — the wander.
        if npc.collide_x {
            npc.vx *= -0.8;
        }
        if npc.collide_y {
            npc.vy *= -0.8;
        }
        let p = target_of(npc, env.players);
        let (pcx, pcy) = candidate_center(p);
        let (cx, cy) = npc.center();
        if npc.vx == 0.0 && npc.vy == 0.0 {
            // `:38762-38768` — the zero-velocity re-kick.
            let dx = pcx - cx;
            let dy = pcy - cy - (p.height / 4) as f32;
            let (nx, ny) = normalize_xna(dx, dy);
            npc.vx = nx * 0.1;
            npc.vy = ny * 0.1;
        }
        // `:38769-38774` — the normalized-velocity steer.
        let (nx, ny) = normalize_xna(npc.vx, npc.vy);
        npc.vx = (npc.vx * (20.0 - 1.0) + nx * 1.5) / 20.0;
        npc.vy = (npc.vy * (20.0 - 1.0) + ny * 1.5) / 20.0;
        npc.ai[1] += 1.0;
        if npc.ai[1] > 180.0 {
            npc.ai[0] = 0.0;
            npc.ai[1] = 0.0;
        }
        if can_hit_points(env.world, cx, cy, pcx, pcy) {
            npc.ai[0] = 0.0;
        }
        npc.local_ai[0] += 1.0;
        // `:38786-38789` — the 5-tick probe gate + the ±10 solid box.
        let (w, h) = npc.size();
        if npc.local_ai[0] < 5.0
            || crate::player::Player::solid_collision(
                env.world,
                npc.x - 10.0,
                npc.y - 10.0,
                w + 20.0,
                h + 20.0,
            )
        {
            return out;
        }
        npc.local_ai[0] = 0.0;
        // `:38791-38799` — the X-aligned triple probe (the DUPLICATE
        // second CanHit verbatim, `:38793`).
        let (w35x, w35y) = (pcx, cy);
        if can_hit_points(env.world, cx, cy, w35x, w35y)
            && can_hit_points(env.world, cx, cy, w35x, w35y)
            && can_hit_points(env.world, pcx, pcy, w35x, w35y)
        {
            npc.ai[0] = 3.0;
            npc.ai[1] = w35x;
            npc.ai[2] = w35y;
            return out;
        }
        // `:38800-38807` — the Y-aligned double probe.
        let (w35x, w35y) = (cx, pcy);
        if can_hit_points(env.world, cx, cy, w35x, w35y)
            && can_hit_points(env.world, pcx, pcy, w35x, w35y)
        {
            npc.ai[0] = 3.0;
            npc.ai[1] = w35x;
            npc.ai[2] = w35y;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A 483-shaped NPC: no kind yet — the placeholder + the C# box
    /// (the style86 convention).
    fn flyer(x: f32, y: f32) -> Npc {
        let d = &GRANITE_FLYER;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n
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

    fn env<'a>(
        w: &'a World,
        players: &'a [TargetCandidate],
        rng: &'a mut UnifiedRandom,
    ) -> Style91Env<'a> {
        Style91Env {
            world: w,
            players,
            ticker: 0,
            expert: false,
            rng,
        }
    }

    /// `NPC.cs:14562-14575` — the SetDefaults row, field by field.
    #[test]
    fn w4_d32_s91_the_defaults_row() {
        let d = &GRANITE_FLYER;
        assert_eq!(d.npc_slots, 0.5);
        assert_eq!((d.width, d.height), (20, 30));
        assert_eq!(d.ai_style, 91);
        assert_eq!((d.damage, d.defense, d.life_max), (24, 8, 40));
        assert_eq!(d.hit_sound, (3, 7), "SoundID.NPCHit7");
        assert_eq!(d.knock_back_resist, 0.6);
        assert_eq!(d.death_sound, (4, 6), "SoundID.NPCDeath6");
        assert_eq!(d.value, 1000.0);
    }

    /// `:38634-38639` — the flinch draw is EXPERT-ONLY (the &&
    /// short-circuit): a classic justHit draws nothing; an expert
    /// justHit draws exactly one Next(6).
    #[test]
    fn w4_d32_s91_the_flinch_draw_is_expert_only() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(91);
        let mut n = flyer(100.0, 100.0);
        n.just_hit = true;
        n.ai[0] = 1.0;
        let mut e = env(&w, &players, &mut rng);
        let _ = style91_step(&mut n, &mut e);
        assert_eq!(rng.peek(), UnifiedRandom::new(91).peek(), "classic: no draw");
        // Expert: one Next(6); find a seed that rolls 0.
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(6) != 0 {
            seed += 1;
        }
        let mut rng2 = UnifiedRandom::new(seed);
        let mut oracle = UnifiedRandom::new(seed);
        let mut n2 = flyer(100.0, 100.0);
        n2.just_hit = true;
        n2.ai[0] = 1.0;
        let mut e2 = env(&w, &players, &mut rng2);
        e2.expert = true;
        let _ = style91_step(&mut n2, &mut e2);
        oracle.next_max(6);
        assert_eq!(rng2.peek(), oracle.peek(), "one draw");
        assert_eq!(n2.ai[0], -1.0, "the flinch burrows");
        assert!(n2.net_update);
    }

    /// `:38640-38650` — the burrow: dontTakeDamage + noGravity
    /// false + ×0.98, the 120-tick chained reset.
    #[test]
    fn w4_d32_s91_the_burrow_falls_and_resets_at_120() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(7);
        let mut n = flyer(100.0, 100.0);
        n.ai[0] = -1.0;
        n.ai[1] = 119.0;
        n.ai[2] = 5.0;
        n.ai[3] = 6.0;
        n.vx = 10.0;
        let mut e = env(&w, &players, &mut rng);
        let out = style91_step(&mut n, &mut e);
        assert!(n.dont_take_damage);
        assert!(!out.no_gravity, "the burrow falls");
        assert_eq!(n.vx, 9.8);
        assert_eq!(n.ai, [0.0, 0.0, 0.0, 0.0], "the chained reset");
    }

    /// `:38651-38658` — the seek's LOS short-circuit: in the air
    /// world the flyer sees the player → state 1 + the RETURN (no
    /// waypoint probes, no kick).
    #[test]
    fn w4_d32_s91_the_seek_los_returns_early() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(7);
        let mut n = flyer(100.0, 100.0);
        let mut e = env(&w, &players, &mut rng);
        let _ = style91_step(&mut n, &mut e);
        assert_eq!(n.ai[0], 1.0);
        assert_eq!(n.vx, 0.0, "the return skips the kick");
    }

    /// `:38699-38712` — the attack steer: the 2+len/200 speed over
    /// the 50-weight blend; the air world keeps LOS → stays 1.
    #[test]
    fn w4_d32_s91_the_attack_steer_blends_at_50() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(7);
        let mut n = flyer(100.0, 100.0);
        n.ai[0] = 1.0;
        let mut e = env(&w, &players, &mut rng);
        let _ = style91_step(&mut n, &mut e);
        let (cx, cy) = n.center();
        let dx = 310.0 - cx;
        let dy = 121.0 - cy;
        let speed = 2.0 + sqrt_f32(dx, dy) / 200.0;
        let (nx, ny) = normalize_xna(dx, dy);
        assert_eq!(n.vx, nx * speed / 50.0);
        assert_eq!(n.vy, ny * speed / 50.0);
        assert_eq!(n.ai[0], 1.0);
    }

    /// `:38747-38774` — the wander: the collide ×−0.8 bounce, the
    /// zero-velocity re-kick (the INT height/4 = 10 offset), the
    /// ×1.5 / 20-weight steer.
    #[test]
    fn w4_d32_s91_the_wander_bounce_and_rekick() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(7);
        let mut n = flyer(100.0, 100.0);
        n.ai[0] = 4.0;
        n.collide_x = true;
        n.vx = 5.0;
        let mut e = env(&w, &players, &mut rng);
        let _ = style91_step(&mut n, &mut e);
        // −4 then the steer: (−4×19 + nx×1.5)/20 with nx = −1.
        assert_eq!(n.vx, (-4.0 * 19.0 - 1.5) / 20.0);
        // The air world's LOS flips the wander back to the seek.
        assert_eq!(n.ai[0], 0.0);
        // The zero-velocity re-kick: the delta Y carries −10 (42/4).
        let mut n2 = flyer(100.0, 100.0);
        n2.ai[0] = 4.0;
        let _ = style91_step(&mut n2, &mut e);
        let (cx, cy) = n2.center();
        let dx = 310.0 - cx;
        let dy = 121.0 - cy - 10.0;
        let (nx, ny) = normalize_xna(dx, dy);
        let kx = nx * 0.1;
        let ky = ny * 0.1;
        let (sx, sy) = normalize_xna(kx, ky);
        assert_eq!(n2.vx, (kx * 19.0 + sx * 1.5) / 20.0);
        assert_eq!(n2.vy, (ky * 19.0 + sy * 1.5) / 20.0);
    }

    /// The zero-draw census away from the flinch: 600 ticks of the
    /// seek/attack/wander cycle consume NO `Main.rand`.
    #[test]
    fn w4_d32_s91_the_cycle_draws_nothing_without_a_hit() {
        let w = air();
        let players = [cand(900.0, 300.0)];
        let mut rng = UnifiedRandom::new(483);
        let mut n = flyer(100.0, 100.0);
        let mut e = env(&w, &players, &mut rng);
        for _ in 0..600 {
            let _ = style91_step(&mut n, &mut e);
        }
        assert_eq!(rng.peek(), UnifiedRandom::new(483).peek(), "no draws");
    }
}
