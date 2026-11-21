//! The aiStyle-118 family — the Seahorses
//! (`NPC.AI_118_Seahorses`, `NPC.cs:47071-47125`, dispatched at
//! `:41092-41095`). W4-d33, research/25 row 118, CODE-ONLY MODE (no
//! compiler invoked; the witness tests below are written, not run).
//!
//! ## Census
//!
//! The family has TWO vanilla members: the Seahorse 626 /
//! GoldSeahorse 627 (`ID/NPCID.cs:11929-11931`; the shared
//! `SetDefaults` row `NPC.cs:17051-17071`). The census of
//! `this.aiStyle = 118;` returns exactly that one row (`:17055`) — no
//! projectile-space user. The port's roster has no seahorse kind —
//! the module lands UNWIRED (the `npc_ai_style73.rs` precedent).
//!
//! ## The machine
//!
//! The swimming critter: `ai[0]` is the swim ANGLE, `ai[1]` the rest
//! timer. `noGravity = wet` (`:47073` — the outcome seam; the dry
//! seahorse falls under the caller's gravity).
//!
//! The surface flag (`:47074-47075`): `GetWaterLineIterate(Center
//! tile)` — the line > 0 AND `Top.Y - line < 20` (the near-surface
//! band). The C# writes `waterLineHeight = 0` on failure; the port's
//! `None` reads the same (the flag false).
//!
//! Dry (`:47076-47083`): a zero vy damps `vx *= 0.95`; the rotation
//! ACCUMULATES `(vx + vy)/2 * 0.05` (the flop spin).
//!
//! Wet (`:47084-47108`): `ai[1]--`; at ≤ 0 the swim kick —
//! `velocity += ai[0].ToRotationVector2() * 0.06`; the length check
//! `num > 3 || num < -3` (the SECOND clause is DEAD — Length() is
//! never negative; verbatim) clamps ONLY vx to ±3 (the C# writes X
//! alone — the vy keeps its kick), re-rolls `ai[1] = Next(450, 600)`
//! and `ai[0] = NextFloat() * 2π`, the near-surface `ai[0] > π`
//! subtracts π (the downward-angle fold), netUpdate. The rest arm
//! damps ×0.95. `rotation = vx * 0.1` (the OVERWRITE, not the dry
//! accumulate).
//!
//! The bounce tail (`:47109-47124`): `flag2 = collideY && wet &&
//! (!surface || vy < 0)`; `collideX | flag2` (the NON-short-circuit
//! `|`) reflects the angle's X (collideX) / Y (flag2) components,
//! re-derives `ai[0] = v.ToRotation()`, and re-aims the velocity at
//! the SAME length; netUpdate.
//!
//! ## RNG order (by construction)
//!
//! The wet kick's over-speed arm: Next(450, 600) then ONE NextFloat.
//! Everything else draws NOTHING.
//!
//! ## FindFrame cases 626/627
//!
//! `NPC.cs:76780-76819`: the vx-sign spriteDirection (the `!(vx < 0)`
//! form — 0 faces +1); the bestiary-dummy arm (the documented skip);
//! the LIVE arm is a pure velocity-length → row map: dry → 0,
//! `len > 0.15` → 3, `> 0.05` → 2, `> 0.025` → 1, else 0. NO counter.
//! `Main.npcFrameCount[626/627] = 6`. [`seahorse_frame_row`] ports the
//! map.
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::collision::get_water_line_iterate;
use crate::npc::Npc;
use glam::Vec2;
use tr_core::{UnifiedRandom, World};

/// The shared type-626/627 `SetDefaults` row (`NPC.cs:17051-17071`),
/// verbatim. NOTE: no `noGravity` write — the ARM sets it per tick
/// (`= wet`).
pub struct SeahorseDefaults {
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
    pub knock_back_resist: f32,
    /// The catch items: 626 → 4480, 627 → 4482.
    pub catch_items: [i32; 2],
    /// The Gold 627's rarity 3.
    pub gold_rarity: i32,
}

/// `NPC.cs:17051-17071` — `else if (this.type == 626 || 627)`.
pub const SEAHORSE: SeahorseDefaults = SeahorseDefaults {
    width: 16,
    height: 20,
    ai_style: 118,
    damage: 0,
    defense: 0,
    life_max: 5,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    knock_back_resist: 1.0,
    catch_items: [4480, 4482],
    gold_rarity: 3,
};

/// `Main.npcFrameCount[626/627]` = 6 (`Main.cs:67146`, the 620-629
/// row: `21, 1, 1, 1, 11, 12, 6, 6, 17, 6`).
pub const SEAHORSE_FRAME_COUNT: u32 = 6;

/// The per-tick environment.
pub struct Style118Env<'a> {
    pub world: &'a World,
    pub rng: &'a mut UnifiedRandom,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Style118Outcome {
    /// The `noGravity = wet` write (`:47073`) — the `Npc` slot has no
    /// field (the style24 seam).
    pub no_gravity: bool,
}

/// XNA `Vector2.Length()` — the f32 sum widened to f64 for the root.
fn length(v: Vec2) -> f32 {
    ((v.x * v.x + v.y * v.y) as f64).sqrt() as f32
}

/// One fixed step of the aiStyle-118 arm (`NPC.cs:47071-47125`), in C#
/// order. The caller owns the movement integration (the gravity tail
/// applies when `no_gravity` is false) and drains
/// [`Style118Outcome`].
pub fn style118_step(npc: &mut Npc, env: &mut Style118Env<'_>) -> Style118Outcome {
    let mut out = Style118Outcome::default();
    // `:47073` — the per-tick gravity flip.
    out.no_gravity = npc.wet;
    let (w, h) = npc.size();
    let center = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
    // `:47074-47075` — the surface flag.
    let tx = center.x as i32 >> 4;
    let ty = center.y as i32 >> 4;
    let water_line = get_water_line_iterate(env.world, tx, ty).unwrap_or(0.0);
    let near_surface = water_line > 0.0 && npc.y - water_line < 20.0; // flag
    if !npc.wet {
        // `:47076-47083` — the dry flop.
        if npc.vy == 0.0 {
            npc.vx *= 0.95;
        }
        npc.rotation += (npc.vx + npc.vy) / 2.0 * 0.05;
    } else {
        // `:47084-47108` — the swim.
        npc.ai[1] -= 1.0;
        if npc.ai[1] <= 0.0 {
            let a = npc.ai[0];
            let kick = Vec2::new((a as f64).cos() as f32, (a as f64).sin() as f32) * 0.06;
            npc.vx += kick.x;
            npc.vy += kick.y;
            let len = length(Vec2::new(npc.vx, npc.vy)); // num
                                                         // `:47091` — the `num < -3` clause is DEAD (Length() >= 0);
                                                         // verbatim.
            if len > 3.0 || len < -3.0 {
                // `:47093` — the clamp writes X ALONE.
                npc.vx = 3.0f32.min((-3.0f32).max(npc.vx));
                npc.ai[1] = env.rng.next_range(450, 600) as f32;
                npc.ai[0] = env.rng.next_float() * (std::f32::consts::PI * 2.0);
                if near_surface && npc.ai[0] > std::f32::consts::PI {
                    npc.ai[0] -= std::f32::consts::PI;
                }
                npc.net_update = true;
            }
        } else {
            npc.vx *= 0.95;
            npc.vy *= 0.95;
        }
        npc.rotation = npc.vx * 0.1;
    }
    // `:47109-47124` — the bounce tail.
    let bounce_y = npc.collide_y && npc.wet && (!near_surface || npc.vy < 0.0); // flag2
    if npc.collide_x | bounce_y {
        let a = npc.ai[0];
        let mut v = Vec2::new((a as f64).cos() as f32, (a as f64).sin() as f32);
        if npc.collide_x {
            v.x *= -1.0;
        }
        if bounce_y {
            v.y *= -1.0;
        }
        npc.ai[0] = (v.y as f64).atan2(v.x as f64) as f32;
        let len = length(Vec2::new(npc.vx, npc.vy));
        let a = npc.ai[0];
        let reaim = Vec2::new((a as f64).cos() as f32, (a as f64).sin() as f32) * len;
        npc.vx = reaim.x;
        npc.vy = reaim.y;
        npc.net_update = true;
    }
    out
}

/// `NPC.FindFrame` cases 626/627 (`NPC.cs:76780-76819`) — the LIVE
/// velocity-length → row map (the bestiary-dummy arm is the
/// documented skip; NO counter). The `spriteDirection = !(vx < 0) ?
/// 1 : -1` write is the caller's.
pub fn seahorse_frame_row(wet: bool, vx: f32, vy: f32) -> u32 {
    if !wet {
        return 0;
    }
    let len = length(Vec2::new(vx, vy));
    if len > 0.15 {
        3
    } else if len > 0.05 {
        2
    } else if len > 0.025 {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A 626-shaped NPC (the style73 placeholder convention).
    fn horse(x: f32, y: f32) -> Npc {
        let d = &SEAHORSE;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n.facing = 1;
        n
    }

    fn air() -> World {
        World::new("t".into(), 400, 300)
    }

    /// `NPC.cs:17051-17071` — the shared row, field by field.
    #[test]
    fn w4_d33_s118_the_defaults_row() {
        let d = &SEAHORSE;
        assert_eq!((d.width, d.height), (16, 20));
        assert_eq!(d.ai_style, 118);
        assert_eq!((d.damage, d.defense, d.life_max), (0, 0, 5));
        assert_eq!(d.hit_sound, (3, 1));
        assert_eq!(d.death_sound, (4, 1));
        assert_eq!(d.knock_back_resist, 1.0);
        assert_eq!(d.catch_items, [4480, 4482]);
        assert_eq!(d.gold_rarity, 3);
        assert_eq!(SEAHORSE_FRAME_COUNT, 6);
    }

    /// `:47073` + `:47076-47083` — the dry flop: noGravity false, the
    /// zero-vy ×0.95 damp, the ACCUMULATING rotation.
    #[test]
    fn w4_d33_s118_the_dry_flop() {
        let w = air();
        let mut rng = UnifiedRandom::new(118);
        let mut n = horse(100.0, 100.0);
        n.vx = 1.0;
        n.vy = 0.0;
        n.rotation = 0.5;
        let mut env = Style118Env {
            world: &w,
            rng: &mut rng,
        };
        let out = style118_step(&mut n, &mut env);
        assert!(!out.no_gravity);
        assert_eq!(n.vx, 0.95);
        // rotation += (0.95 + 0)/2 * 0.05.
        assert!((n.rotation - (0.5 + 0.95 / 2.0 * 0.05)).abs() < 1e-6);
        assert_eq!(
            rng.peek(),
            UnifiedRandom::new(118).peek(),
            "the dry arm draws nothing"
        );
    }

    /// `:47086-47107` — the wet rest damp + the rotation OVERWRITE;
    /// the kick at ai[1] ≤ 0 adds the 0.06 angle vector.
    #[test]
    fn w4_d33_s118_the_wet_rest_and_kick() {
        let w = air();
        let mut rng = UnifiedRandom::new(118);
        let mut n = horse(100.0, 100.0);
        n.wet = true;
        n.ai[1] = 5.0; // the rest arm
        n.vx = 1.0;
        n.vy = 1.0;
        n.rotation = 0.5;
        let mut env = Style118Env {
            world: &w,
            rng: &mut rng,
        };
        let out = style118_step(&mut n, &mut env);
        assert!(out.no_gravity);
        assert_eq!(n.vx, 0.95);
        assert_eq!(n.vy, 0.95);
        assert_eq!(n.rotation, 0.095, "the overwrite, not the accumulate");
        assert_eq!(n.ai[1], 4.0);
        // The kick tick: ai[1] 1 → 0, the angle-0 kick +0.06 X.
        let mut n = horse(100.0, 100.0);
        n.wet = true;
        n.ai[1] = 1.0;
        n.ai[0] = 0.0;
        let mut env = Style118Env {
            world: &w,
            rng: &mut rng,
        };
        let _ = style118_step(&mut n, &mut env);
        assert!((n.vx - 0.06).abs() < 1e-6);
        assert_eq!(n.vy, 0.0);
    }

    /// `:47089-47101` — the over-speed re-roll: the X-ONLY clamp, the
    /// Next(450,600) + NextFloat draws, the near-surface π fold.
    #[test]
    fn w4_d33_s118_the_overspeed_reroll() {
        let w = air();
        let mut rng = UnifiedRandom::new(118);
        let mut oracle = UnifiedRandom::new(118);
        let mut n = horse(100.0, 100.0);
        n.wet = true;
        n.ai[1] = 1.0;
        n.ai[0] = 0.0;
        n.vx = 3.0; // the kick pushes past 3
        let mut env = Style118Env {
            world: &w,
            rng: &mut rng,
        };
        let _ = style118_step(&mut n, &mut env);
        let rest = oracle.next_range(450, 600);
        let angle = oracle.next_float() * (std::f32::consts::PI * 2.0);
        assert_eq!(rng.peek(), oracle.peek(), "the re-roll draw order");
        assert_eq!(n.vx, 3.0, "the X clamp");
        assert_eq!(n.ai[1], rest as f32);
        assert_eq!(n.ai[0], angle);
        assert!(n.net_update);
    }

    /// `:47109-47124` — the bounce: collideX reflects the angle's X,
    /// the velocity re-aims at the SAME length.
    #[test]
    fn w4_d33_s118_the_bounce_reflect() {
        let w = air();
        let mut rng = UnifiedRandom::new(118);
        let mut n = horse(100.0, 100.0);
        n.wet = true;
        n.ai[1] = 5.0; // the rest arm — no kick draws
        n.ai[0] = 0.0; // due right
        n.vx = 2.0;
        n.vy = 0.0;
        n.collide_x = true;
        let mut env = Style118Env {
            world: &w,
            rng: &mut rng,
        };
        let _ = style118_step(&mut n, &mut env);
        // The rest damp made vx 1.9; the reflect re-aims at length 1.9
        // along the flipped angle (π) → vx -1.9.
        assert!((n.vx - (-1.9)).abs() < 1e-4, "vx {}", n.vx);
        assert!(n.vy.abs() < 1e-4);
        assert!(n.net_update);
        // The angle is now π (or -π).
        assert!(n.ai[0].abs() > 3.0);
    }

    /// `NPC.cs:76780-76819` — the velocity-length row map.
    #[test]
    fn w4_d33_s118_the_frame_row_map() {
        assert_eq!(seahorse_frame_row(false, 5.0, 5.0), 0, "dry → 0");
        assert_eq!(seahorse_frame_row(true, 0.2, 0.0), 3);
        assert_eq!(seahorse_frame_row(true, 0.06, 0.0), 2);
        assert_eq!(seahorse_frame_row(true, 0.03, 0.0), 1);
        assert_eq!(seahorse_frame_row(true, 0.01, 0.0), 0);
    }

    /// The stream parity: 300 ticks match a cloned twin bit-for-bit.
    #[test]
    fn w4_d33_s118_the_run_is_twin_stable() {
        let w = air();
        let mut a = horse(280.0, 280.0);
        a.wet = true;
        let mut b = a.clone();
        let mut rng_a = UnifiedRandom::new(626);
        let mut rng_b = UnifiedRandom::new(626);
        for tick in 0..300 {
            let mut env_a = Style118Env {
                world: &w,
                rng: &mut rng_a,
            };
            let _ = style118_step(&mut a, &mut env_a);
            let mut env_b = Style118Env {
                world: &w,
                rng: &mut rng_b,
            };
            let _ = style118_step(&mut b, &mut env_b);
            assert_eq!(rng_a.peek(), rng_b.peek(), "stream @{tick}");
            assert_eq!(a.vx.to_bits(), b.vx.to_bits(), "vx @{tick}");
            assert_eq!(a.ai[0].to_bits(), b.ai[0].to_bits(), "angle @{tick}");
        }
    }
}
