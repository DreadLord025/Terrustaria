//! The aiStyle-116 family — the Water Striders
//! (`NPC.AI_116_WaterStriders`, `NPC.cs:47518-47578`, dispatched at
//! `:41084-41087`). W4-d33, research/25 row 116, CODE-ONLY MODE (no
//! compiler invoked; the witness tests below are written, not run).
//!
//! ## Census
//!
//! The family has TWO vanilla members: the WaterStrider 612 /
//! GoldWaterStrider 613 (`ID/NPCID.cs:11901-11903`; the shared
//! `SetDefaults` row `NPC.cs:16860-16879`). The census of
//! `this.aiStyle = 116;` returns exactly that one row (`:16864`) — no
//! projectile-space user. The `UpdateNPC` liquid pass also pins
//! `wetCount = 10` for the style (`:93077-93080` — the never-dry
//! strider; the wiring's liquid pass owns it, documented). The port's
//! roster has no strider kind — the module lands UNWIRED (the
//! `npc_ai_style73.rs` precedent).
//!
//! ## The machine
//!
//! The surface-skating hopper: it rides the water line, damps on the
//! surface, and hops on a random timer.
//!
//! The water line (`:47520-47545`): `GetWaterLine(Center tile)` —
//! found: below the line (`Center.Y > waterLine`) the -0.8 lift capped
//! -4 with the `Bottom.Y - 1 + vy` overshoot clamp to the line; at or
//! above it `vy = Min(vy, waterLine - (Bottom.Y - 1))` and the
//! on-surface flag sets. No line but wet: the -0.3 lift.
//!
//! The hop gate (`:47546-47577`): `(int)ai[0] != 0` RETURNS (the
//! caller's state lock). `ai[1]++`; `vx *= 0.9`; a zero vy damps
//! `vx *= 0.6` again. `flag2 = wet | onSurface` (the NON-short-circuit
//! `|`); `flag3 = flag2 || vy == 0`. The timer roll: `Next(120, 241)`
//! ALWAYS drawn, then `Next(60, 241)` OVERWRITES it when !flag2 (both
//! draws stay — verbatim). Below the threshold or !flag3: return.
//! The hop: `ai[1] = 0`, `vx = NextFloatDirection() * 5`, netUpdate;
//! the airborne hop (!flag2) also sets `vy = -2` when grounded and
//! `ai[1] = 60` (the post-hop cooldown).
//!
//! ## RNG order (by construction)
//!
//! Per non-locked tick: Next(120,241), then Next(60,241) when !flag2,
//! then (on the hop) ONE NextFloatDirection. The water-line probe and
//! the damps draw NOTHING.
//!
//! ## FindFrame cases 612/613
//!
//! `NPC.cs:69773-69834`: the bestiary-dummy arm (the 50-tick
//! WrappedLerp cycle — the port has no bestiary dummies, documented
//! skip); the live arm: the `frame.Y == 0 && Next(180) != 0` idle
//! freeze (the DRAW stays even when frozen), the vx-sign
//! spriteDirection, the `ai[1] == 1` / `|vy| > 0.1 || |vx| > 1`
//! jump-row snaps, the 10/2/60 tick rates, the 5-row wrap.
//! `Main.npcFrameCount[612/613] = 5`. [`strider_find_frame`] ports the
//! live pair (the style73 convention).
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::collision::get_water_line;
use crate::npc::Npc;
use glam::Vec2;
use tr_core::{UnifiedRandom, World};

/// The shared type-612/613 `SetDefaults` row (`NPC.cs:16860-16879`),
/// verbatim. NOTE: no `noGravity` write — the strider falls under the
/// caller's normal gravity; the arm's lifts fight it.
pub struct WaterStriderDefaults {
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
    /// `catchItem = type - 612 + 4418` (4418/4419).
    pub catch_item_base: i32,
    /// The Gold 613's rarity 3.
    pub gold_rarity: i32,
    /// The three liquid-movement speeds — all 1.
    pub water_movement_speed: f32,
    pub lava_movement_speed: f32,
    pub honey_movement_speed: f32,
}

/// `NPC.cs:16860-16879` — `else if (this.type == 612 || 613)`.
pub const WATER_STRIDER: WaterStriderDefaults = WaterStriderDefaults {
    width: 10,
    height: 10,
    ai_style: 116,
    damage: 0,
    defense: 0,
    life_max: 5,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    npc_slots: 0.25,
    catch_item_base: 4418,
    gold_rarity: 3,
    water_movement_speed: 1.0,
    lava_movement_speed: 1.0,
    honey_movement_speed: 1.0,
};

/// `Main.npcFrameCount[612/613]` = 5 (`Main.cs:67145`, the 610-619
/// row: `6, 13, 5, 5, 7, 5, 14, 14, 4, 6` — indices 612/613 are the
/// 3rd/4th entries).
pub const WATER_STRIDER_FRAME_COUNT: u32 = 5;

/// The per-tick environment.
pub struct Style116Env<'a> {
    pub world: &'a World,
    pub rng: &'a mut UnifiedRandom,
}

/// One fixed step of the aiStyle-116 arm (`NPC.cs:47518-47578`), in C#
/// order. The caller owns the movement integration (the row has NO
/// noGravity — the normal gravity tail applies after `AI()`).
pub fn style116_step(npc: &mut Npc, env: &mut Style116Env<'_>) {
    let (w, h) = npc.size();
    let center = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
    let mut on_surface = false; // flag
                                // `:47521-47545` — the water line.
    let tx = center.x as i32 >> 4;
    let ty = center.y as i32 >> 4;
    if let Some(water_line) = get_water_line(env.world, tx, ty) {
        let bottom_minus_1 = npc.y + h - 1.0; // num
        if center.y > water_line {
            npc.vy -= 0.8;
            if npc.vy < -4.0 {
                npc.vy = -4.0;
            }
            if bottom_minus_1 + npc.vy < water_line {
                npc.vy = water_line - bottom_minus_1;
            }
        } else {
            npc.vy = npc.vy.min(water_line - bottom_minus_1);
            on_surface = true;
        }
    } else if npc.wet {
        npc.vy -= 0.3;
    }
    // `:47546-47549` — the state lock.
    if npc.ai[0] as i32 != 0 {
        return;
    }
    // `:47550-47555` — the counter + the damps.
    npc.ai[1] += 1.0;
    npc.vx *= 0.9;
    if npc.vy == 0.0 {
        npc.vx *= 0.6;
    }
    // `:47556-47562` — the flags + the DOUBLE timer roll (the C# `|`
    // is non-short-circuit; both sides are pure reads).
    let in_water = npc.wet || on_surface; // flag2
    let can_hop = in_water || npc.vy == 0.0; // flag3
    let mut threshold = env.rng.next_range(120, 241); // num2
    if !in_water {
        threshold = env.rng.next_range(60, 241);
    }
    if !can_hop || npc.ai[1] < threshold as f32 {
        return;
    }
    // `:47567-47577` — the hop.
    npc.ai[1] = 0.0;
    npc.vx = env.rng.next_float_direction() * 5.0;
    npc.net_update = true;
    if !in_water {
        if npc.vy == 0.0 {
            npc.vy = -2.0;
        }
        npc.ai[1] = 60.0;
    }
}

/// `NPC.FindFrame` cases 612/613 (`NPC.cs:69773-69834`) — the LIVE
/// pair (the bestiary-dummy arm is the documented skip). `frame_row`
/// is `frame.Y / num`; the `Next(180)` idle-freeze draw happens ONLY
/// at row 0 and is consumed here (the rng parameter). The
/// `spriteDirection = Sign(vx)` write is the caller's.
pub fn strider_find_frame(
    frame_row: u32,
    frame_counter: f32,
    vx: f32,
    vy: f32,
    ai1: f32,
    rng: &mut UnifiedRandom,
) -> (u32, f32) {
    // `:69787-69795` — the idle freeze (the draw stays).
    let mut advance = true; // flag13
    if frame_row == 0 && rng.next_max(180) != 0 {
        advance = false;
    }
    let mut counter = frame_counter;
    if advance {
        counter += 1.0;
    }
    let mut row = frame_row;
    // `:69800-69823` — the rate + the jump snaps.
    let mut rate = 10; // num286
    let fast = vx.abs() > 1.0; // flag14
    if ai1 == 1.0 {
        counter = 0.0;
        row = 4;
    }
    if vy.abs() > 0.1 || fast {
        counter = 0.0;
        row = 4;
    } else if row == 0 {
        rate = 2;
    }
    if row == 4 {
        rate = 60;
        if !fast {
            rate = 2;
        }
    }
    // `:69824-69832` — the advance + the 5-row wrap.
    if counter >= rate as f32 {
        counter = 0.0;
        row += 1;
        if row >= 5 {
            row = 0;
        }
    }
    (row, counter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A 612-shaped NPC (the style73 placeholder convention).
    fn strider(x: f32, y: f32) -> Npc {
        let d = &WATER_STRIDER;
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

    /// `NPC.cs:16860-16879` — the shared row, field by field.
    #[test]
    fn w4_d33_s116_the_defaults_row() {
        let d = &WATER_STRIDER;
        assert_eq!((d.width, d.height), (10, 10));
        assert_eq!(d.ai_style, 116);
        assert_eq!((d.damage, d.defense, d.life_max), (0, 0, 5));
        assert_eq!(d.hit_sound, (3, 1));
        assert_eq!(d.death_sound, (4, 1));
        assert_eq!(d.npc_slots, 0.25);
        assert_eq!(d.catch_item_base, 4418);
        assert_eq!(d.gold_rarity, 3);
        assert_eq!(d.water_movement_speed, 1.0);
        assert_eq!(d.lava_movement_speed, 1.0);
        assert_eq!(d.honey_movement_speed, 1.0);
        assert_eq!(WATER_STRIDER_FRAME_COUNT, 5);
    }

    /// `:47546-47549` — the state lock returns before the counter and
    /// the rolls (the water-line lift still ran).
    #[test]
    fn w4_d33_s116_the_state_lock() {
        let w = air();
        let mut rng = UnifiedRandom::new(116);
        let mut n = strider(100.0, 100.0);
        n.ai[0] = 1.0;
        n.wet = true;
        let mut env = Style116Env {
            world: &w,
            rng: &mut rng,
        };
        style116_step(&mut n, &mut env);
        assert_eq!(n.vy, -0.3, "the wet lift ran");
        assert_eq!(n.ai[1], 0.0, "no counter tick");
        assert_eq!(rng.peek(), UnifiedRandom::new(116).peek(), "no draws");
    }

    /// `:47556-47562` — the DOUBLE timer roll: the dry grounded hopper
    /// draws Next(120,241) THEN Next(60,241) (both stay); the wet one
    /// draws only the first.
    #[test]
    fn w4_d33_s116_the_double_timer_roll() {
        let w = air();
        let mut rng = UnifiedRandom::new(116);
        let mut oracle = UnifiedRandom::new(116);
        let mut n = strider(100.0, 100.0);
        n.vy = 0.0; // grounded, dry
        let mut env = Style116Env {
            world: &w,
            rng: &mut rng,
        };
        style116_step(&mut n, &mut env);
        oracle.next_range(120, 241);
        oracle.next_range(60, 241);
        // The hop may fire (ai[1] 1 < 60 — never on tick 1); no third
        // draw either way.
        assert_eq!(rng.peek(), oracle.peek(), "the dry double roll");
        let mut rng = UnifiedRandom::new(116);
        let mut oracle = UnifiedRandom::new(116);
        let mut n = strider(100.0, 100.0);
        n.wet = true;
        let mut env = Style116Env {
            world: &w,
            rng: &mut rng,
        };
        style116_step(&mut n, &mut env);
        oracle.next_range(120, 241);
        assert_eq!(rng.peek(), oracle.peek(), "the wet single roll");
    }

    /// `:47567-47577` — the hop: the NextFloatDirection ×5 vx, the
    /// dry hop's -2 vy + ai[1]=60 cooldown; the wet hop keeps vy and
    /// ai[1]=0.
    #[test]
    fn w4_d33_s116_the_hop() {
        let w = air();
        let mut rng = UnifiedRandom::new(116);
        let mut oracle = UnifiedRandom::new(116);
        let mut n = strider(100.0, 100.0);
        n.vy = 0.0;
        n.ai[1] = 300.0; // past any threshold
        let mut env = Style116Env {
            world: &w,
            rng: &mut rng,
        };
        style116_step(&mut n, &mut env);
        oracle.next_range(120, 241);
        oracle.next_range(60, 241);
        let dir = oracle.next_float_direction();
        assert_eq!(rng.peek(), oracle.peek(), "the hop draw order");
        assert_eq!(n.vx, dir * 5.0);
        assert_eq!(n.vy, -2.0, "the dry hop lift");
        assert_eq!(n.ai[1], 60.0, "the cooldown");
        assert!(n.net_update);
        // The wet hop: no lift, no cooldown.
        let mut rng = UnifiedRandom::new(116);
        let mut n = strider(100.0, 100.0);
        n.wet = true;
        n.ai[1] = 300.0;
        let mut env = Style116Env {
            world: &w,
            rng: &mut rng,
        };
        style116_step(&mut n, &mut env);
        assert_eq!(n.ai[1], 0.0);
        // The wet lift ran first (-0.3), the hop keeps it.
        assert_eq!(n.vy, -0.3);
    }

    /// `:47521-47541` — the water-line ride: below the line the -0.8
    /// lift with the overshoot clamp; at/above it the Min snap + the
    /// on-surface flag (the single-roll path).
    #[test]
    fn w4_d33_s116_the_water_line_ride() {
        let mut w = air();
        // A full water tile at (6, 7): the centre (105, 117) → tile
        // (6, 7); GetWaterLine finds the line at row 7's top + the
        // liquid shortfall.
        let i = w.index(6, 7);
        w.tiles[i].liquid = 255;
        let mut rng = UnifiedRandom::new(116);
        let mut n = strider(100.0, 112.0); // centre y 117 > the line
        let line = get_water_line(&w, 6, 7).expect("the water line");
        let mut env = Style116Env {
            world: &w,
            rng: &mut rng,
        };
        style116_step(&mut n, &mut env);
        // The lift: vy = -0.8, then the overshoot clamp if it crosses.
        let bottom_minus_1 = 112.0 + 10.0 - 1.0;
        let want = if bottom_minus_1 - 0.8 < line {
            line - bottom_minus_1
        } else {
            -0.8
        };
        assert_eq!(n.vy, want);
    }

    /// `NPC.cs:69773-69834` — the idle freeze draw, the jump-row snap,
    /// the 5-row wrap.
    #[test]
    fn w4_d33_s116_the_find_frame() {
        let mut rng = UnifiedRandom::new(116);
        let mut oracle = UnifiedRandom::new(116);
        // Row 0 draws Next(180); a non-zero roll freezes the counter.
        let (row, counter) = strider_find_frame(0, 0.0, 0.0, 0.0, 0.0, &mut rng);
        let roll = oracle.next_max(180);
        assert_eq!(rng.peek(), oracle.peek(), "the idle draw");
        if roll != 0 {
            assert_eq!((row, counter), (0, 0.0), "the freeze");
        } else {
            assert_eq!((row, counter), (0, 1.0));
        }
        // The fast-move snap: row 4, counter 0, no idle draw (row != 0).
        let mut rng = UnifiedRandom::new(116);
        let (row, counter) = strider_find_frame(2, 5.0, 2.0, 0.0, 0.0, &mut rng);
        assert_eq!((row, counter), (4, 0.0));
        assert_eq!(
            rng.peek(),
            UnifiedRandom::new(116).peek(),
            "no idle draw off row 0"
        );
        // The ai[1]==1 snap.
        let mut rng = UnifiedRandom::new(116);
        let (row, _) = strider_find_frame(2, 5.0, 0.0, 0.0, 1.0, &mut rng);
        assert_eq!(row, 4);
    }

    /// The stream parity: 300 ticks match a cloned twin bit-for-bit.
    #[test]
    fn w4_d33_s116_the_run_is_twin_stable() {
        let w = air();
        let mut a = strider(280.0, 280.0);
        let mut b = a.clone();
        let mut rng_a = UnifiedRandom::new(612);
        let mut rng_b = UnifiedRandom::new(612);
        for tick in 0..300 {
            let mut env_a = Style116Env {
                world: &w,
                rng: &mut rng_a,
            };
            style116_step(&mut a, &mut env_a);
            let mut env_b = Style116Env {
                world: &w,
                rng: &mut rng_b,
            };
            style116_step(&mut b, &mut env_b);
            assert_eq!(rng_a.peek(), rng_b.peek(), "stream @{tick}");
            assert_eq!(a.vx.to_bits(), b.vx.to_bits(), "vx @{tick}");
            assert_eq!(a.ai[1].to_bits(), b.ai[1].to_bits(), "timer @{tick}");
        }
    }
}
