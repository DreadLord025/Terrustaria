//! The aiStyle-122 family — the Pirate Ghost
//! (`NPC.AI_122_PirateGhost`, `NPC.cs:44843-44882`, dispatched at
//! `:41108-41111`). W4-d33, research/25 row 122, CODE-ONLY MODE (no
//! compiler invoked; the witness tests below are written, not run).
//!
//! ## Census
//!
//! The family has exactly ONE vanilla member: the PirateGhost 662
//! (`NPCID.PirateGhost`, `ID/NPCID.cs:12001`; `SetDefaults` row
//! `NPC.cs:17367-17383`). The census of `this.aiStyle = 122;` returns
//! exactly that one hit (`:17371`) — no projectile-space user. The
//! port's roster has no pirate-ghost kind — the module lands UNWIRED
//! (the `npc_ai_style73.rs` precedent).
//!
//! ## The machine
//!
//! The flying-board ghost: it fades in, chases the player with a
//! capped hover, separates from its siblings, and fades out (dies)
//! when no target remains.
//!
//! The spawn cue (`:44845-44849`): `localAI[0] == 0` latches 1 and
//! plays `SoundID.NPCHit6` `(3, 6)` at the position (the audio seam).
//!
//! The target (`:44850-44865`): `GetTargetData` — invalid →
//! TargetClosest + re-read; STILL invalid → the fade-out: `velocity *=
//! 0.9`, `alpha = Clamp(alpha + 5, 0, 255)`, at 255
//! `StrikeNPCNoInteraction(9999, 0, 0)` (the port's instant-kill hp=0
//! convention, `npc.rs:11697`), RETURN.
//!
//! The chase (`:44866-44881`): `alpha = Clamp(alpha - 5, 0, 255)`;
//! `velocity = MoveTowards(velocity, Zero.MoveTowards(target -
//! Center, 4), 2/15)` — the desired velocity is the target vector
//! capped at 4, the actual velocity walks toward it at 2/15 per tick.
//! The separation: every OTHER active same-type NPC within 50 px
//! pushes `velocity -= normalize(delta)*0.1` PLUS an extra
//! `velocity.X -= delta.X*0.1` (the X double-push).
//!
//! ## RNG order (by construction)
//!
//! ZERO draws — the spawn cue is a sound (the audio seam), the chase
//! and the separation are pure maths, TargetClosest is RNG-free.
//!
//! ## FindFrame case 662
//!
//! `NPC.cs:74836-74884` (the shared 662/664 case): the 662 path skips
//! the type-60/634 dust arms and the 210/211 double-counter; the
//! vx-sign spriteDirection, `rotation = vx * 0.1`, the 6-tick row
//! advance over `Main.npcFrameCount[662] = 4`.
//! [`ghost_find_frame`] ports the pair (the style73 convention).
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use crate::projectile::move_towards;
use glam::Vec2;
use tr_core::World;

/// The type-662 `SetDefaults` row (`NPC.cs:17367-17383`), verbatim.
pub struct PirateGhostDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit1`.
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath6` (`SoundID.cs:1320`).
    pub death_sound: (i32, i32),
    pub knock_back_resist: f32,
    pub value: f32,
    pub no_tile_collide: bool,
    pub lava_immune: bool,
    pub trap_immune: bool,
    pub no_gravity: bool,
}

/// `NPC.cs:17367-17383` — `else if (this.type == 662)`.
pub const PIRATE_GHOST: PirateGhostDefaults = PirateGhostDefaults {
    width: 18,
    height: 40,
    ai_style: 122,
    damage: 75,
    defense: 22,
    life_max: 500,
    hit_sound: (3, 1),
    death_sound: (4, 6),
    knock_back_resist: 0.2,
    value: 0.0,
    no_tile_collide: true,
    lava_immune: true,
    trap_immune: true,
    no_gravity: true,
};

/// `Main.npcFrameCount[662]` = 4 (`Main.cs:67150`, the 660-669 row).
pub const PIRATE_GHOST_FRAME_COUNT: u32 = 4;

/// One `Main.npc[i]` slot as the separation scan sees it (`:44870`).
#[derive(Clone, Copy, Debug)]
pub struct GhostNeighbor {
    /// `i != whoAmI` — the self skip, precomputed by the wiring.
    pub is_self: bool,
    pub active: bool,
    /// `Main.npc[i].type`.
    pub ty: i32,
    pub x: f32,
    pub y: f32,
    pub width: i32,
    pub height: i32,
}

/// The per-tick environment.
pub struct Style122Env<'a> {
    pub world: &'a World,
    /// The `Main.player[]` slots as `TargetClosest` sees them.
    pub players: &'a [TargetCandidate],
    /// The NPC slots in `Main.npc` order, self included (`is_self`).
    pub neighbors: &'a [GhostNeighbor],
    /// `Main.EverLastingTicker` for the selection's `targetSetFrame`.
    pub ticker: u64,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style122Outcome {
    /// The fade-out expiry (`:44862`): `StrikeNPCNoInteraction(9999)`
    /// — the port's instant-kill hp=0 already applied; the caller owes
    /// the death processing (the style100 `died` seam).
    pub died: bool,
    /// The spawn cue `SoundEngine.PlaySound(SoundID.NPCHit6,
    /// position)` (`:44848`) — `(type, style, x, y)` = `(3, 6)`.
    pub sounds: Vec<(i32, i32, f32, f32)>,
}

/// XNA `Vector2.Length()` — the f32 sum widened to f64 for the root.
fn length(v: Vec2) -> f32 {
    ((v.x * v.x + v.y * v.y) as f64).sqrt() as f32
}

/// XNA `Vector2.Normalize` — UNGUARDED (the style100 convention).
fn normalize_xna(v: Vec2) -> Vec2 {
    let n = 1.0 / ((v.x * v.x + v.y * v.y) as f64).sqrt() as f32;
    Vec2::new(v.x * n, v.y * n)
}

/// `NPC.GetTargetData` for 662 (NOT in `UsesNewTargeting` —
/// player-only): the target player's centre, or `None` when invalid
/// (`HasValidTarget`, `NPC.cs:6657-6671`).
fn ghost_target(npc: &Npc, env: &Style122Env<'_>) -> Option<Vec2> {
    if npc.target >= 0 && (npc.target as usize) < env.players.len() {
        let p = &env.players[npc.target as usize];
        if p.active && !p.dead && !p.ghost {
            return Some(Vec2::new(
                p.x + (p.width / 2) as f32,
                p.y + (p.height / 2) as f32,
            ));
        }
    }
    None
}

/// One fixed step of the aiStyle-122 arm (`NPC.cs:44843-44882`), in C#
/// order. The caller owns the movement integration (the row's
/// noGravity/noTileCollide are true) and drains [`Style122Outcome`].
pub fn style122_step(npc: &mut Npc, env: &mut Style122Env<'_>) -> Style122Outcome {
    let mut out = Style122Outcome::default();
    let (w, h) = npc.size();
    let center = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
    // `:44845-44849` — the spawn cue.
    if npc.local_ai[0] == 0.0 {
        npc.local_ai[0] = 1.0;
        out.sounds.push((3, 6, npc.x, npc.y));
    }
    // `:44850-44855` — the target re-pick.
    let mut target = ghost_target(npc, env);
    if target.is_none() {
        if !env.players.is_empty() {
            target_closest(npc, env.world, env.players, true, 122, env.ticker);
        }
        target = ghost_target(npc, env);
    }
    // `:44856-44865` — the fade-out.
    let Some(target) = target else {
        npc.vx *= 0.9;
        npc.vy *= 0.9;
        npc.alpha = npc.alpha.saturating_add(5); // Clamp(+5, 0, 255)
        if npc.alpha >= 255 {
            // StrikeNPCNoInteraction(9999, 0, 0) — the instant-kill
            // hp=0 convention (npc.rs:11697).
            npc.hp = 0;
            out.died = true;
        }
        return out;
    };
    // `:44866-44867` — the fade-in + the capped hover.
    npc.alpha = npc.alpha.saturating_sub(5); // Clamp(-5, 0, 255)
    let desired = move_towards(Vec2::ZERO, target - center, 4.0);
    let v = move_towards(Vec2::new(npc.vx, npc.vy), desired, 2.0 / 15.0);
    npc.vx = v.x;
    npc.vy = v.y;
    // `:44868-44881` — the separation.
    for n in env.neighbors {
        if n.active && n.ty == 662 && !n.is_self {
            let their_center = Vec2::new(n.x + n.width as f32 / 2.0, n.y + n.height as f32 / 2.0);
            let delta = their_center - center;
            if length(delta) < 50.0 {
                let push = normalize_xna(delta) * 0.1;
                npc.vx -= push.x;
                npc.vy -= push.y;
                // The X double-push (`:44878`).
                npc.vx -= push.x * 1.0;
            }
        }
    }
    out
}

/// `NPC.FindFrame` case 662 (`NPC.cs:74836-74884`) — the 6-tick row
/// advance over the 4-row sheet (the style73 convention). The vx-sign
/// spriteDirection + `rotation = vx * 0.1` writes are the caller's.
pub fn ghost_find_frame(frame_row: u32, frame_counter: f32) -> (u32, f32) {
    let mut counter = frame_counter + 1.0;
    let mut row = frame_row;
    if counter >= 6.0 {
        row += 1;
        counter = 0.0;
    }
    if row >= PIRATE_GHOST_FRAME_COUNT {
        row = 0;
    }
    (row, counter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::UnifiedRandom;

    /// A 662-shaped NPC (the style73 placeholder convention).
    fn ghost(x: f32, y: f32) -> Npc {
        let d = &PIRATE_GHOST;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n.facing = 1;
        n.local_ai[0] = 1.0; // skip the spawn cue by default
        n.alpha = 100;
        n.target = 0;
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

    /// `NPC.cs:17367-17383` — the SetDefaults row, field by field.
    #[test]
    fn w4_d33_s122_the_defaults_row() {
        let d = &PIRATE_GHOST;
        assert_eq!((d.width, d.height), (18, 40));
        assert_eq!(d.ai_style, 122);
        assert_eq!((d.damage, d.defense, d.life_max), (75, 22, 500));
        assert_eq!(d.hit_sound, (3, 1));
        assert_eq!(d.death_sound, (4, 6));
        assert_eq!(d.knock_back_resist, 0.2);
        assert_eq!(d.value, 0.0);
        assert!(d.no_tile_collide && d.lava_immune && d.trap_immune && d.no_gravity);
        assert_eq!(PIRATE_GHOST_FRAME_COUNT, 4);
    }

    /// `:44845-44849` — the spawn cue fires once (the localAI[0] latch).
    #[test]
    fn w4_d33_s122_the_spawn_cue() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut n = ghost(100.0, 100.0);
        n.local_ai[0] = 0.0;
        let mut env = Style122Env {
            world: &w,
            players: &players,
            neighbors: &[],
            ticker: 7,
        };
        let out = style122_step(&mut n, &mut env);
        assert!(out.sounds.iter().any(|&(t, s, _, _)| (t, s) == (3, 6)));
        assert_eq!(n.local_ai[0], 1.0);
        let out = style122_step(&mut n, &mut env);
        assert!(out.sounds.is_empty(), "the cue fires once");
    }

    /// `:44856-44865` — the no-target fade-out: the ×0.9 damp, the +5
    /// alpha, the 255 expiry (hp=0 + died).
    #[test]
    fn w4_d33_s122_the_fade_out() {
        let w = air();
        let players: [TargetCandidate; 0] = [];
        let mut n = ghost(100.0, 100.0);
        n.vx = 1.0;
        n.alpha = 250;
        let mut env = Style122Env {
            world: &w,
            players: &players,
            neighbors: &[],
            ticker: 7,
        };
        let out = style122_step(&mut n, &mut env);
        assert_eq!(n.vx, 0.9);
        assert_eq!(n.alpha, 255);
        assert!(out.died);
        assert_eq!(n.hp, 0);
        // Below 255: no death yet.
        let mut n = ghost(100.0, 100.0);
        n.alpha = 100;
        let out = style122_step(&mut n, &mut env);
        assert_eq!(n.alpha, 105);
        assert!(!out.died);
    }

    /// `:44866-44867` — the chase: the -5 alpha, the desired velocity
    /// capped at 4, the actual walking at 2/15.
    #[test]
    fn w4_d33_s122_the_capped_hover() {
        let w = air();
        // The player far right — the desired velocity caps at 4.
        let players = [cand(1000.0, 100.0)];
        let mut n = ghost(100.0, 100.0);
        n.vx = 0.0;
        n.vy = 0.0;
        let mut env = Style122Env {
            world: &w,
            players: &players,
            neighbors: &[],
            ticker: 7,
        };
        let _ = style122_step(&mut n, &mut env);
        assert_eq!(n.alpha, 95);
        // desired = (4, 0); v = 0 + (4-0)*(2/15) = 8/15.
        assert!((n.vx - 8.0 / 15.0).abs() < 1e-6, "vx {}", n.vx);
        assert!(n.vy.abs() < 1e-6);
    }

    /// `:44868-44881` — the separation: the near sibling pushes away
    /// with the X double-push.
    #[test]
    fn w4_d33_s122_the_separation() {
        let w = air();
        let players = [cand(1000.0, 100.0)];
        // A sibling 30 px right (within 50).
        let sib = GhostNeighbor {
            is_self: false,
            active: true,
            ty: 662,
            x: 130.0,
            y: 100.0,
            width: 18,
            height: 40,
        };
        let neighbors = [
            GhostNeighbor {
                is_self: true,
                active: true,
                ty: 662,
                x: 100.0,
                y: 100.0,
                width: 18,
                height: 40,
            },
            sib,
        ];
        let mut n = ghost(100.0, 100.0);
        let mut env = Style122Env {
            world: &w,
            players: &players,
            neighbors: &neighbors,
            ticker: 7,
        };
        let _ = style122_step(&mut n, &mut env);
        // The chase gave vx 8/15; the sibling right pushes vx -= 0.1
        // then -= 0.1 again (the X double). vy unchanged (level).
        assert!((n.vx - (8.0 / 15.0 - 0.2)).abs() < 1e-5, "vx {}", n.vx);
        assert!(n.vy.abs() < 1e-6);
    }

    /// `NPC.cs:74836-74884` — the 6-tick row advance wraps at 4.
    #[test]
    fn w4_d33_s122_the_find_frame_wraps_at_four() {
        assert_eq!(ghost_find_frame(0, 4.0), (0, 5.0));
        assert_eq!(ghost_find_frame(0, 5.0), (1, 0.0));
        assert_eq!(ghost_find_frame(3, 5.0), (0, 0.0), "the wrap");
    }

    /// The whole arm is draw-free: 60 ticks leave the stream untouched.
    #[test]
    fn w4_d33_s122_the_arm_draws_nothing() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(122);
        let mut n = ghost(100.0, 100.0);
        let mut env = Style122Env {
            world: &w,
            players: &players,
            neighbors: &[],
            ticker: 7,
        };
        for _ in 0..60 {
            let _ = style122_step(&mut n, &mut env);
        }
        assert_eq!(
            rng.peek(),
            UnifiedRandom::new(122).peek(),
            "the 122 arm is draw-free"
        );
    }
}
