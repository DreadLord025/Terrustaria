//! The aiStyle-89 machine — the Mothron Egg (`NPC.AI()`'s
//! `else if (this.aiStyle == 89)` arm, `NPC.cs:38352-38393`). W4-d32,
//! research/25 row 89 (MISSING → ported), CODE-ONLY MODE (no compiler
//! invoked; the witness tests below are written, not run).
//!
//! ## Census
//!
//! The census of `this.aiStyle = 89;` across the decompiled tree
//! returns exactly ONE assignment: `NPC.cs:15620`, the
//! `else if (this.type == 478)` row (`:15616-15629`) — the MothronEgg
//! (`ID/NPCID.cs:11629`). No dynamic `aiStyle` reassignment to 89
//! exists (the egg's `Transform(479)` swaps the WHOLE defaults row —
//! the new type carries aiStyle 90). The kind has no `NpcKind` yet —
//! the module lands UNWIRED (the style73/style86 precedent).
//!
//! ## The machine
//!
//! The roll damping (`:38354-38363`): grounded (vy == 0) — vx ×0.9,
//! rotation += vx ×0.02; airborne — vx ×0.99, rotation += vx ×0.04.
//! NO noGravity write: the egg falls under the caller's gravity
//! (the SetDefaults row leaves it false).
//!
//! The hatch clock (`:38364-38381`): 900 ticks (expert 600); a
//! justHit tick SUBTRACTS `Next(10,21)` — and a SECOND `Next(10,21)`
//! when NOT expert (`:38372-38375`, the classic double-draw); then
//! `ai[0]++`; at ≥ the threshold the `Transform(479)` request
//! (`NPC.cs:81084-81114` — the port's `wants_transform` seam, the
//! MothronSpawn 479 has no kind yet so the request rides the vanilla
//! type id).
//!
//! The pre-hatch wiggle (`:38382-38392`): grounded + |vx| < 0.2 (the
//! f64 comparisons) + `ai[0] >= threshold × 0.75` — the progress
//! fraction `(ai[0] − t×0.75) / (t×0.25)`, the gate
//! `(float)Next(-10,120) < fraction × 100`, then the THREE draws:
//! vy −= `Next(20,40) × 0.025`, vx += `Next(-20,20) × 0.025`,
//! velocity ×= `1 + fraction × 2`, netUpdate.
//!
//! ## RNG census (by construction)
//!
//! 1. The justHit hatch acceleration: `Next(10,21)` ALWAYS, a second
//!    `Next(10,21)` only when NOT expert (`:38371-38375`).
//! 2. The wiggle gate `Next(-10,120)` (`:38385`) — only when the
//!    grounded/slow/75% conditions hold; on success the TWO kick
//!    draws `Next(20,40)` + `Next(-20,20)` (`:38387-38388`).
//! Everything else is deterministic arithmetic.
//!
//! ## Seams (the wiring obligations)
//!
//! * The egg integrates under GRAVITY with tile collision (the
//!   caller's movement step — no flags written by the arm).
//! * `Transform(479)` is a REQUEST ([`Style89Outcome::transform`])
//!   — the sim's transform drain owns the defaults swap + the
//!   bottom-anchored reposition (`npc.rs::transform_to_kind`).
//! * `Main.expertMode` rides [`Style89Env::expert`].
//!
//! See docs/ANALYSIS.md "The Mothron Egg (W4-d32 aiStyle-89, code-only)".
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::npc::Npc;
use tr_core::UnifiedRandom;

/// The type-478 `SetDefaults` row (`NPC.cs:15616-15629`), verbatim.
pub struct MothronEggDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit1` (`ID/SoundID.cs:1257`).
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath1` (`ID/SoundID.cs:1315`).
    pub death_sound: (i32, i32),
    pub value: f32,
    pub knock_back_resist: f32,
    pub npc_slots: f32,
}

/// `NPC.cs:15616-15629` — `else if (this.type == 478)`.
pub const MOTHRON_EGG: MothronEggDefaults = MothronEggDefaults {
    width: 34,
    height: 34,
    ai_style: 89,
    damage: 0,
    defense: 30,
    life_max: 200,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    value: 0.0,
    knock_back_resist: 0.7,
    npc_slots: 0.0,
};

/// The per-tick environment.
pub struct Style89Env<'a> {
    /// `Main.expertMode` — the 600-vs-900 clock + the single-vs-double
    /// justHit draw.
    pub expert: bool,
    pub rng: &'a mut UnifiedRandom,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Default, Debug, PartialEq)]
pub struct Style89Outcome {
    /// The `Transform(479)` request (`:38380`) — the vanilla type id
    /// (the MothronSpawn has no kind yet; the sim's drain resolves).
    pub transform: Option<i32>,
}

/// One fixed step of the aiStyle-89 arm (`NPC.cs:38352-38393`), in
/// C# order. `npc` is the egg (type 478). The caller owns the
/// gravity integration and drains the transform request.
pub fn style89_step(npc: &mut Npc, env: &mut Style89Env<'_>) -> Style89Outcome {
    let mut out = Style89Outcome::default();
    // `:38354-38363` — the roll damping.
    if npc.vy == 0.0 {
        npc.vx *= 0.9;
        npc.rotation += npc.vx * 0.02;
    } else {
        npc.vx *= 0.99;
        npc.rotation += npc.vx * 0.04;
    }
    // `:38364-38368` — the hatch clock.
    let num1255 = if env.expert { 600 } else { 900 };
    // `:38369-38376` — the justHit acceleration (the classic double
    // draw, verbatim order).
    if npc.just_hit {
        npc.ai[0] -= env.rng.next_range(10, 21) as f32;
        if !env.expert {
            npc.ai[0] -= env.rng.next_range(10, 21) as f32;
        }
    }
    // `:38377-38381` — the tick + the hatch.
    npc.ai[0] += 1.0;
    if npc.ai[0] >= num1255 as f32 {
        out.transform = Some(479);
    }
    // `:38382-38392` — the pre-hatch wiggle (the f64 comparisons).
    if npc.vy == 0.0
        && (npc.vx.abs() as f64) < 0.2
        && (npc.ai[0] as f64) >= num1255 as f64 * 0.75
    {
        let num1256 =
            (npc.ai[0] - num1255 as f32 * 0.75) / (num1255 as f32 * 0.25);
        if (env.rng.next_range(-10, 120) as f32) < num1256 * 100.0 {
            npc.vy -= env.rng.next_range(20, 40) as f32 * 0.025;
            npc.vx += env.rng.next_range(-20, 20) as f32 * 0.025;
            npc.vx *= 1.0 + num1256 * 2.0;
            npc.vy *= 1.0 + num1256 * 2.0;
            npc.net_update = true;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A 478-shaped NPC: no kind yet — the placeholder + the C# box
    /// (the style86 convention).
    fn egg(x: f32, y: f32) -> Npc {
        let d = &MOTHRON_EGG;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n
    }

    /// `NPC.cs:15616-15629` — the SetDefaults row, field by field.
    #[test]
    fn w4_d32_s89_the_defaults_row() {
        let d = &MOTHRON_EGG;
        assert_eq!((d.width, d.height), (34, 34));
        assert_eq!(d.ai_style, 89);
        assert_eq!((d.damage, d.defense, d.life_max), (0, 30, 200));
        assert_eq!(d.hit_sound, (3, 1), "SoundID.NPCHit1");
        assert_eq!(d.death_sound, (4, 1), "SoundID.NPCDeath1");
        assert_eq!(d.value, 0.0);
        assert_eq!(d.knock_back_resist, 0.7);
        assert_eq!(d.npc_slots, 0.0);
    }

    /// `:38354-38363` — the grounded ×0.9 / airborne ×0.99 damping
    /// with the 0.02/0.04 rotation accrual (AFTER the damp).
    #[test]
    fn w4_d32_s89_the_roll_damping_and_rotation() {
        let mut rng = UnifiedRandom::new(7);
        let mut n = egg(100.0, 100.0);
        n.vx = 10.0;
        n.vy = 0.0;
        let mut e = Style89Env { expert: false, rng: &mut rng };
        let _ = style89_step(&mut n, &mut e);
        assert_eq!(n.vx, 9.0);
        assert_eq!(n.rotation, 9.0 * 0.02);
        n.vy = 1.0;
        n.rotation = 0.0;
        let _ = style89_step(&mut n, &mut e);
        assert_eq!(n.vx, 9.0 * 0.99);
        assert_eq!(n.rotation, 9.0 * 0.99 * 0.04);
    }

    /// `:38369-38376` — the justHit draws: classic TWO Next(10,21),
    /// expert ONE; the mirrored oracle pins the order.
    #[test]
    fn w4_d32_s89_the_justhit_double_draw_is_classic_only() {
        let mut rng = UnifiedRandom::new(89);
        let mut oracle = UnifiedRandom::new(89);
        let mut n = egg(100.0, 100.0);
        n.just_hit = true;
        n.ai[0] = 100.0;
        n.vy = 1.0; // airborne: no wiggle draws
        let mut e = Style89Env { expert: false, rng: &mut rng };
        let _ = style89_step(&mut n, &mut e);
        let a = oracle.next_range(10, 21);
        let b = oracle.next_range(10, 21);
        assert_eq!(n.ai[0], 100.0 - a as f32 - b as f32 + 1.0);
        assert_eq!(rng.peek(), oracle.peek(), "the classic double draw");
        // Expert: one draw.
        let mut rng2 = UnifiedRandom::new(89);
        let mut oracle2 = UnifiedRandom::new(89);
        let mut n2 = egg(100.0, 100.0);
        n2.just_hit = true;
        n2.ai[0] = 100.0;
        n2.vy = 1.0;
        let mut e2 = Style89Env { expert: true, rng: &mut rng2 };
        let _ = style89_step(&mut n2, &mut e2);
        let a2 = oracle2.next_range(10, 21);
        assert_eq!(n2.ai[0], 100.0 - a2 as f32 + 1.0);
        assert_eq!(rng2.peek(), oracle2.peek(), "the expert single draw");
    }

    /// `:38377-38381` — the hatch at 900 (classic) / 600 (expert):
    /// the Transform(479) request fires on the threshold tick.
    #[test]
    fn w4_d32_s89_the_hatch_thresholds() {
        let mut rng = UnifiedRandom::new(7);
        let mut n = egg(100.0, 100.0);
        n.ai[0] = 898.0;
        n.vy = 1.0; // airborne: no wiggle
        let mut e = Style89Env { expert: false, rng: &mut rng };
        let out = style89_step(&mut n, &mut e);
        assert!(out.transform.is_none(), "899 < 900");
        let out = style89_step(&mut n, &mut e);
        assert_eq!(out.transform, Some(479), "the 900th tick hatches");
        let mut n2 = egg(100.0, 100.0);
        n2.ai[0] = 599.0;
        n2.vy = 1.0;
        let mut e2 = Style89Env { expert: true, rng: &mut rng };
        let out = style89_step(&mut n2, &mut e2);
        assert_eq!(out.transform, Some(479), "the expert 600th tick");
    }

    /// `:38382-38392` — the wiggle: at the 75% mark the fraction is
    /// 0 → the gate `Next(-10,120) < 0` fires only on the negative
    /// rolls; the THREE draws (gate, Y kick, X kick) ride the oracle.
    #[test]
    fn w4_d32_s89_the_wiggle_gate_and_kicks() {
        let mut rng = UnifiedRandom::new(478);
        let mut oracle = UnifiedRandom::new(478);
        let mut n = egg(100.0, 100.0);
        n.ai[0] = 674.0; // the bump makes 675 = 900 × 0.75 exactly
        n.vx = 0.1;
        n.vy = 0.0;
        let mut e = Style89Env { expert: false, rng: &mut rng };
        let _ = style89_step(&mut n, &mut e);
        // vx damped to 0.09 first; fraction = 0 → gate < 0.
        let gate = oracle.next_range(-10, 120);
        let mut vx = 0.1f32 * 0.9;
        let mut vy = 0.0f32;
        if (gate as f32) < 0.0 {
            vy -= oracle.next_range(20, 40) as f32 * 0.025;
            vx += oracle.next_range(-20, 20) as f32 * 0.025;
            vx *= 1.0;
            vy *= 1.0;
        }
        assert_eq!(n.vx, vx);
        assert_eq!(n.vy, vy);
        assert_eq!(rng.peek(), oracle.peek(), "the wiggle draw order");
    }

    /// The below-75% zero-draw census: a quiet grounded egg under
    /// the wiggle threshold consumes NO `Main.rand`.
    #[test]
    fn w4_d32_s89_below_the_wiggle_threshold_draws_nothing() {
        let mut rng = UnifiedRandom::new(89);
        let mut n = egg(100.0, 100.0);
        n.vx = 0.05;
        let mut e = Style89Env { expert: false, rng: &mut rng };
        for _ in 0..600 {
            let _ = style89_step(&mut n, &mut e);
        }
        assert_eq!(rng.peek(), UnifiedRandom::new(89).peek(), "no draws");
        assert_eq!(n.ai[0], 600.0);
    }
}
