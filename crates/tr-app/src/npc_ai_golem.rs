//! S1-D-3 (golem audit/gap-fill): the Golem family's missing core, as pure
//! functions cited against the C#. The family's machines live in
//! `npc.rs::update_golem_rng` (the kind-gated seam); this module carries the
//! numeric cores the v1 arm folded or invented:
//!
//! * the enrage balance gate (`GetMyBalance` solo 1 + the outside-temple x2),
//! * the body's charge-rate ladder / slam launch / flight caps,
//! * the attached head's snap tether (the v1 ran an invented accel model),
//! * the fists' anchor (the C# folds the body's FULL velocity, not just X),
//! * the free head's hover approach + the two cadence ladders,
//! * the two `Main.rand.Next` beam triggers (the v1 folded them to flat
//!   averages — the RNG-by-construction discipline restores the draws).
//!
//! Ids (`ID/NPCID.cs:11167-11175`): Golem 245 (aiStyle 45,
//! `AI_045_Golem` `NPC.cs:19505-19780`), GolemHead 246 (aiStyle 46,
//! `NPC.cs:31129-31346`), GolemFistLeft 247 / GolemFistRight 248 (aiStyle 47,
//! `AI_047_GolemFist` `NPC.cs:19231-19503`), GolemHeadFree 249 (aiStyle 48,
//! `NPC.cs:31351-31586`). `SetDefaults` rows: `NPC.cs:12163-12220`.

use tr_core::UnifiedRandom;

/// `NPC.GetMyBalance` (`NPC.cs:18483-18491`): solo (<= 1 scaled player)
/// returns 1f. The port is single-player, so the base is always 1.
pub const SOLO_BALANCE: f32 = 1.0;

/// The enrage gate, shared by all four arms: the body adds +2 on
/// `getGoodWorld` (`NPC.cs:19509-19512`), the head/fists/free head +3
/// (`:31132-31135`, `:19234-19237`, `:31355-31358`); then ALL FOUR double
/// the balance when the target stands outside BOTH the Lihzahrd temple
/// and the jungle, or above the world surface
/// (`:19513-19516`, `:31136-31139`, `:19238-19241`, `:31359-31362`):
/// `(!ZoneLihzhardTemple && !ZoneJungle) || Center.Y < worldSurface * 16`.
/// `get_good_world` has no port field yet — the callers pass false
/// (the documented getGood skip, ANALYSIS "The Golem family (S1-D-3)").
pub fn enrage_balance(
    get_good_world: bool,
    good_world_bonus: f32,
    in_temple: bool,
    in_jungle: bool,
    player_center_y: f32,
    world_surface_px: f32,
) -> f32 {
    let mut num = SOLO_BALANCE;
    if get_good_world {
        num += good_world_bonus;
    }
    if (!in_temple && !in_jungle) || player_center_y < world_surface_px {
        num *= 2.0;
    }
    num
}

/// The body's grounded charge rate (`NPC.cs:19643-19676`): the base 1;
/// while `ai[1] > 0` the ladder adds +2 per DEAD part (the left fist 247
/// `flag2`, the right fist 248 `flag3`, the head 246 `flag` — the scan
/// `:19548-19566`), +1 damaged, +4 under half, +8 under a third (the int
/// divisions `lifeMax / 2`, `lifeMax / 3`), then `*= num` and the
/// getGood +100. The v1 ran a flat +1 — the whole ladder was missing.
pub fn body_charge_rate(
    windup_positive: bool,
    head_alive: bool,
    left_fist_alive: bool,
    right_fist_alive: bool,
    hp: i32,
    max_hp: i32,
    num: f32,
    get_good_world: bool,
) -> f32 {
    let mut rate = 1.0;
    if windup_positive {
        if !left_fist_alive {
            rate += 2.0;
        }
        if !right_fist_alive {
            rate += 2.0;
        }
        if !head_alive {
            rate += 2.0;
        }
        if hp < max_hp {
            rate += 1.0;
        }
        if hp < max_hp / 2 {
            rate += 4.0;
        }
        if hp < max_hp / 3 {
            rate += 8.0;
        }
        rate *= num;
        if get_good_world {
            rate += 100.0;
        }
    }
    rate
}

/// The body's slam launch (`NPC.cs:19686-19698`): `velocity.X = 4 *
/// direction`; the damaged body jumps at `-12.1 * (num + 9) / 10` floored
/// at -19.1, the healthy body at the flat -12.1. The v1 hardcoded num = 1.
pub fn body_slam_vy(hp: i32, max_hp: i32, num: f32) -> f32 {
    if hp < max_hp {
        (-12.1 * (num + 9.0) / 10.0).max(-19.1)
    } else {
        -12.1
    }
}

/// The body's flight numbers (`NPC.cs:19727-19763`): the over-player sink
/// `0.2 * (num + 1) / 2` and the horizontal cap `num7 = 3 (+1 damaged,
/// +1 half, +1 quarter) * (num + 1) / 2`. The v1 hardcoded num = 1.
pub fn body_flight_sink(num: f32) -> f32 {
    0.2 * (num + 1.0) / 2.0
}

/// The flight's horizontal speed cap (`NPC.cs:19742-19755`).
pub fn body_flight_cap(hp: i32, max_hp: i32, num: f32) -> f32 {
    let mut num7 = 3.0;
    if hp < max_hp {
        num7 += 1.0;
    }
    if hp < max_hp / 2 {
        num7 += 1.0;
    }
    if hp < max_hp / 4 {
        num7 += 1.0;
    }
    num7 * (num + 1.0) / 2.0
}

/// The attached head's tether (`NPC.cs:31146-31164`): the target is the
/// body's centre offset by `(-3 * scale, -57 * scale)`; inside 100 px the
/// velocity is the RAW delta (a one-tick snap) with rotation 0; beyond it
/// the velocity normalises to 100 and `rotation = velocity.X * 0.1`.
/// The v1 ran an invented `+9` accel with a +-7 clamp — a different
/// motion model entirely. Returns `(vx, vy, rotation)`.
pub fn head_tether_velocity(
    cx: f32,
    cy: f32,
    body_cx: f32,
    body_cy: f32,
    scale: f32,
) -> (f32, f32, f32) {
    let mut dx = body_cx - cx;
    let mut dy = body_cy - cy - 57.0 * scale;
    dx -= 3.0 * scale;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 100.0 {
        (dx, dy, 0.0)
    } else {
        let r = 100.0 / len;
        let vx = dx * r;
        let vy = dy * r;
        (vx, vy, vx * 0.1)
    }
}

/// The fists' tether anchor (`NPC.cs:19258-19259`): `body.Center +
/// body.velocity + (0, -9 * scale)`, then X += -84 (type 247, the LEFT
/// fist) or +78 (type 248). The v1 folded only `velocity.X` — the Y term
/// was missing (the World now carries `golem_vy`).
pub fn fist_anchor(
    body_cx: f32,
    body_cy: f32,
    body_vx: f32,
    body_vy: f32,
    left: bool,
    scale: f32,
) -> (f32, f32) {
    let x = body_cx + body_vx + if left { -84.0 } else { 78.0 } * scale;
    let y = body_cy + body_vy - 9.0 * scale;
    (x, y)
}

/// The fist's follow speed (`NPC.cs:19265-19282`): the base 14, +3 under
/// the own half, +3 under the own quarter, +8 while the body is damaged,
/// `*= (num + 3) / 4`, capped at 32.
pub fn fist_follow_speed(hp: i32, max_hp: i32, body_damaged: bool, num: f32) -> f32 {
    let mut num3 = 14.0;
    if hp < max_hp / 2 {
        num3 += 3.0;
    }
    if hp < max_hp / 4 {
        num3 += 3.0;
    }
    if body_damaged {
        num3 += 8.0;
    }
    num3 *= (num + 3.0) / 4.0;
    num3.min(32.0)
}

/// The fist's in-contact cadence step (`NPC.cs:19291-19304`): `num`, +num
/// under the own half, +num under the own quarter, +10*num while the body
/// is damaged.
pub fn fist_contact_step(hp: i32, max_hp: i32, body_damaged: bool, num: f32) -> f32 {
    let mut step = num;
    if hp < max_hp / 2 {
        step += num;
    }
    if hp < max_hp / 4 {
        step += num;
    }
    if body_damaged {
        step += 10.0 * num;
    }
    step
}

/// The fist's dash speed (`NPC.cs:19361-19378`): the base 12, +4 under the
/// own half, +4 under the own quarter, +10 while the body is damaged,
/// `*= (num + 3) / 4`, capped at 48.
pub fn fist_dash_speed(hp: i32, max_hp: i32, body_damaged: bool, num: f32) -> f32 {
    let mut num6 = 12.0;
    if hp < max_hp / 2 {
        num6 += 4.0;
    }
    if hp < max_hp / 4 {
        num6 += 4.0;
    }
    if body_damaged {
        num6 += 10.0;
    }
    num6 *= (num + 3.0) / 4.0;
    num6.min(48.0)
}

/// The free head's hover approach (`NPC.cs:31378-31417`): the desired
/// velocity aims at `player.Center - (0, 300)` normalised to 7; each axis
/// steps toward it by 0.05, with the DOUBLE add on a sign cross. The v1
/// added `aim * 0.05` directly (a spring, not the C#'s converge) and
/// clamped at +-7 — neither is in the source.
pub fn free_head_hover_velocity(
    vx: f32,
    vy: f32,
    cx: f32,
    cy: f32,
    pcx: f32,
    pcy: f32,
) -> (f32, f32) {
    let mut dx = pcx - cx;
    let mut dy = pcy - cy - 300.0;
    let r = 7.0 / (dx * dx + dy * dy).sqrt();
    dx *= r;
    dy *= r;
    let mut vx = vx;
    let mut vy = vy;
    if vx < dx {
        vx += 0.05;
        if vx < 0.0 && dx > 0.0 {
            vx += 0.05;
        }
    } else if vx > dx {
        vx -= 0.05;
        if vx > 0.0 && dx < 0.0 {
            vx -= 0.05;
        }
    }
    if vy < dy {
        vy += 0.05;
        if vy < 0.0 && dy > 0.0 {
            vy += 0.05;
        }
    } else if vy > dy {
        vy -= 0.05;
        if vy > 0.0 && dy < 0.0 {
            vy -= 0.05;
        }
    }
    (vx, vy)
}

/// The free head's fireball cadence step (`NPC.cs:31418-31435`):
/// `num716 = (num + 4) / 5`, extra adds while the BODY's life is under
/// 80 %, 60 %, 20 % and 10 % (the double comparisons). The v1 carried
/// only the 10 % tier. R23 G2: the C# adds each tier to `ai[1]`
/// SEQUENTIALLY (`this.ai[1] += num716` five times, each a rounded f32
/// add) — the fn takes the cadence accumulator and returns it updated,
/// NOT a pre-summed delta (a different rounding path over long fights).
pub fn free_head_fireball_step(cadence: f32, num: f32, body_hp: i32, body_max_hp: i32) -> f32 {
    let step = (num + 4.0) / 5.0;
    let mut ai1 = cadence;
    ai1 += step;
    // W1-6: the C# compares in DOUBLE (`:31420-31434`,
    // `(double)life < (double)lifeMax * 0.8` …) — f64 here, not f32.
    let (life, max) = (body_hp as f64, body_max_hp as f64);
    if life < max * 0.8 {
        ai1 += step;
    }
    if life < max * 0.6 {
        ai1 += step;
    }
    if life < max * 0.2 {
        ai1 += step;
    }
    if life < max * 0.1 {
        ai1 += step;
    }
    ai1
}

/// The free head's eye-beam cadence step (`NPC.cs:31467-31505`): `num`,
/// extra adds while the BODY's life is under `lifeMax / 1.25` and
/// `/ 1.5` (DOUBLE divisions) then `/ 2`, `/ 3`, `/ 4`, `/ 5`, `/ 6`
/// (INTEGER divisions), and `+= num * 10` while the BODY cannot hit the
/// player (`flag54`, `:31497-31505`). The v1 ran a flat +1. R23 G2: the
/// C# adds each tier to `ai[2]` SEQUENTIALLY (`this.ai[2] += num725`
/// eight times + the flag54 `+= num725 * 10f`, each a rounded f32 add)
/// — the fn takes the cadence accumulator and returns it updated, NOT
/// a pre-summed delta (a different rounding path over long fights).
pub fn free_head_barrage_step(
    cadence: f32,
    num: f32,
    body_hp: i32,
    body_max_hp: i32,
    body_line_blocked: bool,
) -> f32 {
    let mut ai2 = cadence;
    ai2 += num;
    // W1-6: the /1.25 and /1.5 tiers compare in DOUBLE (`:31469-31475`);
    // the /2../6 tiers below stay INTEGER (`:31477-31491`).
    let (life, max) = (body_hp as f64, body_max_hp as f64);
    if life < max / 1.25 {
        ai2 += num;
    }
    if life < max / 1.5 {
        ai2 += num;
    }
    if body_hp < body_max_hp / 2 {
        ai2 += num;
    }
    if body_hp < body_max_hp / 3 {
        ai2 += num;
    }
    if body_hp < body_max_hp / 4 {
        ai2 += num;
    }
    if body_hp < body_max_hp / 5 {
        ai2 += num;
    }
    if body_hp < body_max_hp / 6 {
        ai2 += num;
    }
    if body_line_blocked {
        ai2 += num * 10.0;
    }
    ai2
}

/// The free head's per-beam damage/speed tiers (`NPC.cs:31521-31548`):
/// the base 24 dmg / 11 speed, +1 dmg / +0.25 speed at each of the BODY's
/// 50/40/30/20/10 % thresholds; while the body's line is blocked
/// (`flag54`) the damage truncates `(int)(dmg * 1.5)` and the speed
/// `*= 2.5` (`:31551-31554`).
pub fn free_head_beam_tier(body_hp: i32, body_max_hp: i32, body_line_blocked: bool) -> (i32, f32) {
    let mut dmg = 24;
    let mut speed = 11.0f32;
    // W1-6: the C# tier compares are DOUBLE (`:31524-31547`,
    // `(double)life < (double)lifeMax * 0.5` …) — f64 here, not f32.
    let (life, max) = (body_hp as f64, body_max_hp as f64);
    for threshold in [0.5, 0.4, 0.3, 0.2, 0.1] {
        if life < max * threshold {
            dmg += 1;
            speed += 0.25;
        }
    }
    if body_line_blocked {
        // `:31553`: `(int)((double)num728 * 1.5)` — the double multiply.
        dmg = (dmg as f64 * 1.5) as i32;
        speed *= 2.5;
    }
    (dmg, speed)
}

/// The attached head's fireball cadence step (`NPC.cs:31174-31187`
/// phase 0 / `:31225-31234` phase 1): phase 0 is `1` base plus the
/// balance extras — `2 * (num - 1) / 3` inside the mouth window
/// (`ai[1] < 20 || > 280`), `(num - 1) / 2` outside; phase 1 is
/// `num688 = (num + 3) / 4` with the extra adds at the OWN 40 % / 20 %
/// life tiers. The v1 ran a flat +1/+2/+3 without the balance factor.
pub fn head_fireball_step(num: f32, phase1: bool, mouth_open: bool, hp: i32, max_hp: i32) -> f32 {
    if phase1 {
        let num688 = (num + 3.0) / 4.0;
        let mut total = num688;
        if (hp as f64) < (max_hp as f64) * 0.4 {
            total += num688;
        }
        if (hp as f64) < (max_hp as f64) * 0.2 {
            total += num688;
        }
        total
    } else {
        let mut total = 1.0;
        if mouth_open {
            total += 2.0 * (num - 1.0) / 3.0;
        } else {
            total += (num - 1.0) / 2.0;
        }
        total
    }
}

/// The attached head's beam cadence step (`NPC.cs:31261-31277`):
/// `num688 = (num + 3) / 4` base with the extra adds at the OWN
/// `lifeMax / 3`, `/ 4`, `/ 5` INTEGER tiers and the flat `+4` while
/// the head cannot hit the player. The v1 hardcoded the base 1.
pub fn head_beam_step(num: f32, hp: i32, max_hp: i32, head_line_blocked: bool) -> f32 {
    let num688 = (num + 3.0) / 4.0;
    let mut total = num688;
    if hp < max_hp / 3 {
        total += num688;
    }
    if hp < max_hp / 4 {
        total += num688;
    }
    if hp < max_hp / 5 {
        total += num688;
    }
    if head_line_blocked {
        total += 4.0;
    }
    total
}

/// The fist's state-3 free-flight hover (`NPC.cs:19451-19501`): the
/// desired velocity aims at the player's centre normalised to 12; each
/// axis steps toward it by 0.4 with the TRIPLE add on a sign cross
/// (`num10 * 2f` extra). No writer of `ai[0] = 3` exists in this
/// decompilation for 247/248 — the arm is reachable only by an external
/// write; the port ports it for completeness (ANALYSIS S1-D-3).
pub fn fist_hover_velocity(
    vx: f32,
    vy: f32,
    cx: f32,
    cy: f32,
    pcx: f32,
    pcy: f32,
) -> (f32, f32) {
    let mut dx = pcx - cx;
    let mut dy = pcy - cy;
    let r = 12.0 / (dx * dx + dy * dy).sqrt();
    dx *= r;
    dy *= r;
    let mut vx = vx;
    let mut vy = vy;
    if vx < dx {
        vx += 0.4;
        if vx < 0.0 && dx > 0.0 {
            vx += 0.4 * 2.0;
        }
    } else if vx > dx {
        vx -= 0.4;
        if vx > 0.0 && dx < 0.0 {
            vx -= 0.4 * 2.0;
        }
    }
    if vy < dy {
        vy += 0.4;
        if vy < 0.0 && dy > 0.0 {
            vy += 0.4 * 2.0;
        }
    } else if vy > dy {
        vy -= 0.4;
        if vy > 0.0 && dy < 0.0 {
            vy -= 0.4 * 2.0;
        }
    }
    (vx, vy)
}

/// The body's grounded gravity (`NPC.UpdateNPC_UpdateGravity`,
/// `NPC.cs:90878-90936`): the base `NPC.gravity = 0.3` scaled by the
/// depth factor `clamp((y/16 - (60 + 10 * (maxTilesX/4200)^2)) /
/// (worldSurface / 6), 0.25, 1)`; the fall caps at `maxFallSpeed = 10`.
/// The wet/honey/shimmer overrides need the wet flags the port's NPC
/// does not carry (documented). The v1 golem body never fell at all.
pub fn body_gravity(y: f32, max_tiles_x: i32, world_surface: f64) -> f32 {
    let num = max_tiles_x as f32 / 4200.0;
    let mut num2 = ((y / 16.0 - (60.0 + 10.0 * (num * num))) as f64 / (world_surface / 6.0)) as f32;
    if num2 < 0.25 {
        num2 = 0.25;
    }
    if num2 > 1.0 {
        num2 = 1.0;
    }
    0.3 * num2
}

/// The attached head's eye-beam trigger (`NPC.cs:31278`):
/// `ai[2] > 60 + Main.rand.Next(600)`. The v1 folded this to a flat 300
/// average; the draw is restored (one `Next(600)` per trigger check —
/// the C# evaluates the right side EVERY tick while phase 1 runs).
pub fn head_beam_trigger(rng: &mut UnifiedRandom) -> f32 {
    (60 + rng.next_max(600)) as f32
}

/// The free head's barrage trigger (`NPC.cs:31506`):
/// `ai[2] > 100 + Main.rand.Next(4800)` — one draw per tick, every tick.
/// The v1 folded this to a flat 1500 average.
pub fn free_head_barrage_trigger(rng: &mut UnifiedRandom) -> f32 {
    (100 + rng.next_max(4800)) as f32
}

/// The blocked-line lead on the free head's beams (`NPC.cs:31555-31556`):
/// `target += player.velocity.X * Main.rand.NextFloat() * 50` then the Y
/// with a SECOND `NextFloat` — two draws per beam, X before Y, only while
/// `flag54`. Returns the (x, y) lead offsets.
pub fn free_head_blocked_lead(
    player_vx: f32,
    player_vy: f32,
    rng: &mut UnifiedRandom,
) -> (f32, f32) {
    let lx = player_vx * rng.next_float() * 50.0;
    let ly = player_vy * rng.next_float() * 50.0;
    (lx, ly)
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN, NOT RUN (the S1-D-3 golem round ran in
    //! code-only mode: no cargo/rustc invocation). Each pins one cited
    //! block against hand-computed C# values.
    use super::*;

    #[test]
    fn s1_d_3_golem_the_enrage_gate_doubles_outside_the_temple() {
        // `NPC.cs:19513-19516`: outside BOTH zones -> x2 even below the
        // surface; inside the temple (or the jungle) below the surface
        // -> the solo 1; above the surface -> x2 even in the temple.
        assert_eq!(enrage_balance(false, 2.0, false, false, 1600.0, 800.0), 2.0);
        assert_eq!(enrage_balance(false, 2.0, true, false, 1600.0, 800.0), 1.0);
        assert_eq!(enrage_balance(false, 2.0, false, true, 1600.0, 800.0), 1.0);
        assert_eq!(enrage_balance(false, 2.0, true, true, 700.0, 800.0), 2.0);
        // The getGood bonuses: +2 body (`:19511`), +3 parts (`:31134`).
        assert_eq!(enrage_balance(true, 2.0, true, false, 1600.0, 800.0), 3.0);
        assert_eq!(enrage_balance(true, 3.0, true, false, 1600.0, 800.0), 4.0);
    }

    #[test]
    fn s1_d_3_golem_the_charge_ladder_counts_the_dead_parts() {
        // `NPC.cs:19643-19676`: full hp, all parts alive, windup positive
        // -> the bare 1. All three parts dead -> 1 + 2 + 2 + 2 = 7.
        assert_eq!(body_charge_rate(true, true, true, true, 15000, 15000, 1.0, false), 1.0);
        assert_eq!(body_charge_rate(true, false, false, false, 15000, 15000, 1.0, false), 7.0);
        // Damaged under a third: 7 + 1 + 4 + 8 = 20, x2 enraged = 40.
        assert_eq!(body_charge_rate(true, false, false, false, 4000, 15000, 2.0, false), 40.0);
        // The windup at/below zero keeps the base 1 (`ai[1] > 0f` gate).
        assert_eq!(body_charge_rate(false, false, false, false, 4000, 15000, 2.0, false), 1.0);
        // The getGood +100 lands AFTER the num multiply (`:19670-19674`).
        assert_eq!(body_charge_rate(true, true, true, true, 15000, 15000, 2.0, true), 102.0);
    }

    #[test]
    fn s1_d_3_golem_the_slam_launch_scales_with_the_balance() {
        // `NPC.cs:19687-19698`: healthy -12.1; damaged solo
        // -12.1 * 10 / 10 = -12.1; enraged -12.1 * 11 / 10 = -13.31;
        // the -19.1 floor binds at num >= ~6.78.
        assert_eq!(body_slam_vy(15000, 15000, 2.0), -12.1);
        assert!((body_slam_vy(100, 15000, 1.0) - -12.1).abs() < 1e-4);
        assert!((body_slam_vy(100, 15000, 2.0) - -13.31).abs() < 1e-3);
        assert_eq!(body_slam_vy(100, 15000, 20.0), -19.1);
    }

    #[test]
    fn s1_d_3_golem_the_flight_caps_take_the_balance_factor() {
        // `NPC.cs:19729` / `:19742-19755`: solo sink 0.2, enraged 0.3;
        // the healthy solo cap 3, the damaged-under-quarter enraged cap
        // (3+1+1+1) * 3 / 2 = 9.
        assert!((body_flight_sink(1.0) - 0.2).abs() < 1e-6);
        assert!((body_flight_sink(2.0) - 0.3).abs() < 1e-6);
        assert_eq!(body_flight_cap(15000, 15000, 1.0), 3.0);
        assert_eq!(body_flight_cap(100, 15000, 2.0), 9.0);
    }

    #[test]
    fn s1_d_3_golem_the_head_tether_snaps_inside_the_hundred() {
        // `NPC.cs:31146-31164`: the target is body - (3, 57). A head 50 px
        // left of the target snaps with the raw delta and rotation 0.
        let (vx, vy, rot) = head_tether_velocity(1547.0, 1543.0, 1600.0, 1600.0, 1.0);
        assert!((vx - 50.0).abs() < 1e-4);
        assert!((vy - 0.0).abs() < 1e-4);
        assert_eq!(rot, 0.0);
        // 300 px out: the velocity normalises to 100, rotation vx * 0.1.
        let (vx2, _, rot2) = head_tether_velocity(1297.0, 1543.0, 1600.0, 1600.0, 1.0);
        assert!((vx2 - 100.0).abs() < 1e-3);
        assert!((rot2 - 10.0).abs() < 1e-3);
    }

    #[test]
    fn s1_d_3_golem_the_fist_anchor_folds_the_full_body_velocity() {
        // `NPC.cs:19258-19259`: Center + velocity + (0, -9), X -84 (247)
        // or +78 (248). The v1 dropped the velocity.Y term.
        let (lx, ly) = fist_anchor(1600.0, 1600.0, 4.0, -12.1, true, 1.0);
        assert!((lx - (1600.0 + 4.0 - 84.0)).abs() < 1e-4);
        assert!((ly - (1600.0 - 12.1 - 9.0)).abs() < 1e-4);
        let (rx, _) = fist_anchor(1600.0, 1600.0, 4.0, -12.1, false, 1.0);
        assert!((rx - (1600.0 + 4.0 + 78.0)).abs() < 1e-4);
    }

    #[test]
    fn s1_d_3_golem_the_fist_speeds_take_the_balance_factor() {
        // `NPC.cs:19265-19282` / `:19291-19304` / `:19361-19378`: solo
        // (num 1) the factor (1+3)/4 = 1 keeps the v1 numbers; the
        // enraged num 2 scales by 5/4 and the caps still bind.
        assert_eq!(fist_follow_speed(10000, 10000, false, 1.0), 14.0);
        assert_eq!(fist_follow_speed(10000, 10000, true, 1.0), 22.0);
        assert!((fist_follow_speed(10000, 10000, true, 2.0) - 27.5).abs() < 1e-4);
        assert_eq!(fist_follow_speed(100, 10000, true, 20.0), 32.0);
        assert_eq!(fist_contact_step(10000, 10000, true, 1.0), 11.0);
        assert_eq!(fist_contact_step(10000, 10000, true, 2.0), 22.0);
        assert_eq!(fist_dash_speed(10000, 10000, true, 1.0), 22.0);
        assert!((fist_dash_speed(10000, 10000, true, 2.0) - 27.5).abs() < 1e-4);
        assert_eq!(fist_dash_speed(100, 10000, true, 20.0), 48.0);
    }

    #[test]
    fn s1_d_3_golem_the_free_head_hover_converges_with_the_sign_double() {
        // `NPC.cs:31378-31417`: the desired velocity is 7 toward
        // (player - (0, 300)). Standing still directly under the target
        // column: dx 0, dy -7 -> vy steps -0.05; a sign cross doubles.
        let (vx, vy) = free_head_hover_velocity(0.0, 0.0, 1600.0, 1600.0, 1600.0, 1600.0);
        assert_eq!(vx, 0.0);
        assert!((vy - -0.05).abs() < 1e-6);
        // Moving down (+0.02) while the aim is up: -0.05 once = -0.03 —
        // the sign-cross double fires only while vy is STILL positive
        // after the step (`:31409`), and 0.02 - 0.05 already crossed.
        let (_, vy2) = free_head_hover_velocity(0.0, 0.02, 1600.0, 1600.0, 1600.0, 1600.0);
        assert!((vy2 - -0.03).abs() < 1e-6);
        // The true sign-cross double: +0.08 - 0.05 = +0.03 still > 0
        // while the aim is negative -> the extra -0.05 lands (-0.02).
        let (_, vy3) = free_head_hover_velocity(0.0, 0.08, 1600.0, 1600.0, 1600.0, 1600.0);
        assert!((vy3 - -0.02).abs() < 1e-6);
    }

    #[test]
    fn s1_d_3_golem_the_free_head_cadence_ladders() {
        // `NPC.cs:31418-31435`: solo step (1+4)/5 = 1; the full body adds
        // nothing; a body under 10 % adds all four -> 5. R23 G2: the fns
        // take the cadence accumulator (0 here = the per-tick delta).
        assert_eq!(free_head_fireball_step(0.0, 1.0, 15000, 15000), 1.0);
        assert_eq!(free_head_fireball_step(0.0, 1.0, 1000, 15000), 5.0);
        // Under 60 % only: 1 + 0.8 tier + 0.6 tier? 8000 < 12000 yes,
        // 8000 < 9000 yes, < 3000 no -> 3.
        assert_eq!(free_head_fireball_step(0.0, 1.0, 8000, 15000), 3.0);
        // `NPC.cs:31467-31505`: the barrage's seven tiers; a body at
        // 1/7 of max passes /1.25, /1.5, /2, /3, /4, /5, /6 -> 8; the
        // blocked line adds 10 -> 18.
        assert_eq!(free_head_barrage_step(0.0, 1.0, 15000, 15000, false), 1.0);
        assert_eq!(free_head_barrage_step(0.0, 1.0, 2000, 15000, false), 8.0);
        assert_eq!(free_head_barrage_step(0.0, 1.0, 2000, 15000, true), 18.0);
    }

    #[test]
    fn s1_d_3_golem_the_beam_tiers_and_the_blocked_escalation() {
        // `NPC.cs:31521-31554`: the full body 24/11; under 10 % 29/12.25;
        // blocked truncates (int)(24 * 1.5) = 36 and 11 * 2.5 = 27.5.
        assert_eq!(free_head_beam_tier(15000, 15000, false), (24, 11.0));
        assert_eq!(free_head_beam_tier(1000, 15000, false), (29, 12.25));
        assert_eq!(free_head_beam_tier(15000, 15000, true), (36, 27.5));
    }

    #[test]
    fn s1_d_3_golem_the_head_cadences_take_the_balance_factor() {
        // `NPC.cs:31174-31187` / `:31225-31234`: solo num 1 keeps the
        // v1's flat +1 (phase 0) and the (1+3)/4 = 1 base (phase 1);
        // enraged num 2 adds 2/3 mouth-open, 1/2 closed (phase 0) and
        // 5/4 per tier (phase 1).
        assert_eq!(head_fireball_step(1.0, false, true, 25000, 25000), 1.0);
        assert!((head_fireball_step(2.0, false, true, 25000, 25000) - (1.0 + 2.0 / 3.0)).abs() < 1e-5);
        assert_eq!(head_fireball_step(2.0, false, false, 25000, 25000), 1.5);
        assert_eq!(head_fireball_step(2.0, true, false, 25000, 25000), 1.25);
        // The under-20% head adds both tiers: 3 * 1.25 = 3.75.
        assert_eq!(head_fireball_step(2.0, true, false, 4000, 25000), 3.75);
        // `NPC.cs:31261-31277`: solo under a fifth + blocked -> 1*4 + 4.
        assert_eq!(head_beam_step(1.0, 4000, 25000, true), 8.0);
        assert_eq!(head_beam_step(2.0, 25000, 25000, false), 1.25);
    }

    #[test]
    fn s1_d_3_golem_the_fist_hover_triples_the_sign_cross() {
        // `NPC.cs:19451-19501`: the 12-speed aim, the 0.4 step, the
        // TRIPLE add on a sign cross (0.4 + 0.8).
        let (vx, _) = fist_hover_velocity(0.0, 0.0, 1600.0, 1600.0, 1600.0, 1600.0);
        assert_eq!(vx, 0.0); // dx == 0, vx == 0: neither branch runs.
        let (vx2, _) = fist_hover_velocity(-0.1, 0.0, 1600.0, 1600.0, 2600.0, 1600.0);
        // aim dx = 12; -0.1 + 0.4 = 0.3 stays > 0 -> no extra add.
        assert!((vx2 - 0.3).abs() < 1e-5);
        let (vx3, _) = fist_hover_velocity(-0.5, 0.0, 1600.0, 1600.0, 2600.0, 1600.0);
        // -0.5 + 0.4 = -0.1 < 0 while aim > 0 -> +0.8 more = 0.7.
        assert!((vx3 - 0.7).abs() < 1e-5);
    }

    #[test]
    fn s1_d_3_golem_the_body_gravity_depth_factor() {
        // `NPC.cs:90926-90936`: maxTilesX 4200 -> num 1 -> the offset 70;
        // worldSurface 600 -> divisor 100. y/16 = 70 -> factor 0.25
        // (the floor), y/16 = 170 -> 1.0 (the cap), midway 120 -> 0.5.
        assert_eq!(body_gravity(70.0 * 16.0, 4200, 600.0), 0.3 * 0.25);
        assert_eq!(body_gravity(170.0 * 16.0, 4200, 600.0), 0.3);
        assert!((body_gravity(120.0 * 16.0, 4200, 600.0) - 0.15).abs() < 1e-6);
    }

    #[test]
    fn s1_d_3_golem_the_rng_triggers_draw_once_per_check() {
        // `NPC.cs:31278` / `:31506`: the triggers are 60 + Next(600) and
        // 100 + Next(4800) — the bounds pin the ranges; two consecutive
        // draws from one stream differ (the v1's flat folds drew nothing).
        let mut rng = UnifiedRandom::new(12345);
        let a = head_beam_trigger(&mut rng);
        assert!((60.0..660.0).contains(&a));
        let b = free_head_barrage_trigger(&mut rng);
        assert!((100.0..4900.0).contains(&b));
        // The blocked lead draws X then Y (`:31555-31556`): with a zero
        // player velocity the offsets are zero but the draws still land.
        let (lx, ly) = free_head_blocked_lead(0.0, 0.0, &mut rng);
        assert_eq!((lx, ly), (0.0, 0.0));
        let (lx2, _) = free_head_blocked_lead(10.0, 0.0, &mut rng);
        assert!((0.0..500.0).contains(&lx2));
    }

    #[test]
    fn w1_6_the_free_head_tiers_compare_in_double() {
        // W1-6 (`NPC.cs:31420-31434` / `:31524-31547`): the tier compares
        // are `(double)life < (double)lifeMax * k` — at the family's
        // lifeMax values (15000/16000/25000) every threshold product is
        // exactly representable, so the strict `<` boundaries hold in
        // both widths; these pins catch a regression to a rounded f32
        // product on a future lifeMax where the widths diverge.
        // Fireball: exactly at 80 % (12000 of 15000) the tier does NOT
        // add (strict `<`); one hp below adds it.
        assert_eq!(free_head_fireball_step(0.0, 1.0, 12000, 15000), 1.0);
        assert_eq!(free_head_fireball_step(0.0, 1.0, 11999, 15000), 2.0);
        // Exactly at 10 % (1500 of 15000): the 80/60/20 tiers add, the
        // 10 % tier does not -> 4.
        assert_eq!(free_head_fireball_step(0.0, 1.0, 1500, 15000), 4.0);
        assert_eq!(free_head_fireball_step(0.0, 1.0, 1499, 15000), 5.0);
        // Beam tiers: exactly at 50 % (8000 of 16000) stays 24/11; one
        // below steps to 25/11.25; exactly at 10 % (1600) carries four
        // tiers (50/40/30/20) -> 28/12; one below adds the fifth.
        assert_eq!(free_head_beam_tier(8000, 16000, false), (24, 11.0));
        assert_eq!(free_head_beam_tier(7999, 16000, false), (25, 11.25));
        assert_eq!(free_head_beam_tier(1600, 16000, false), (28, 12.0));
        assert_eq!(free_head_beam_tier(1599, 16000, false), (29, 12.25));
        // The blocked escalation truncates `(int)((double)dmg * 1.5)`
        // (`:31553`): 25 * 1.5 = 37.5 -> 37 (never rounds up).
        assert_eq!(free_head_beam_tier(7999, 16000, true), (37, 28.125));
        // The barrage's DOUBLE /1.25 and /1.5 tiers (`:31469-31475`):
        // exactly at lifeMax/1.25 (12000 of 15000) neither adds; one
        // below adds /1.25 only (12000/1.5 = 10000 boundary is lower);
        // exactly at /1.5 (10000) adds /1.25 only; 9999 adds both. The
        // INTEGER tiers /2../6 (`:31477-31491`) stay integer: at 5000
        // of 15000, /2 = 7500, /3 = 5000 — `5000 < 5000` is false.
        assert_eq!(free_head_barrage_step(0.0, 1.0, 12000, 15000, false), 1.0);
        assert_eq!(free_head_barrage_step(0.0, 1.0, 11999, 15000, false), 2.0);
        assert_eq!(free_head_barrage_step(0.0, 1.0, 10000, 15000, false), 2.0);
        assert_eq!(free_head_barrage_step(0.0, 1.0, 9999, 15000, false), 3.0);
        // 5000 of 15000: /1.25, /1.5, /2 add (3); /3 is the exact
        // integer boundary (no); /4 = 3750, /5 = 3000, /6 = 2500 no.
        assert_eq!(free_head_barrage_step(0.0, 1.0, 5000, 15000, false), 4.0);
        // R23 G2: lifeMax 21000 at the 30 % beam tier — `21000.0f32 *
        // 0.3f32` rounds ABOVE 6300.0, so an f32 compare would take the
        // tier at life == 6300; the C#'s DOUBLE product is exactly 6300
        // and the strict `<` does NOT (`:31533-31537`). One below adds
        // it (50/40/30 -> 27 dmg / 11.75).
        assert_eq!(free_head_beam_tier(6300, 21000, false), (26, 11.5));
        assert_eq!(free_head_beam_tier(6299, 21000, false), (27, 11.75));
    }

    #[test]
    fn r23_g2_the_free_head_ladders_add_per_tier_sequentially() {
        // R23 G2 (`NPC.cs:31419-31434` / `:31468-31504`): the C# adds
        // each tier to ai[1]/ai[2] SEQUENTIALLY — five/eight rounded f32
        // adds, NOT one pre-summed `+=`. At a 2^24 accumulator the f32
        // integer grid turns 2-wide and the two paths diverge:
        // - fireball, solo num 1 (step exactly 1), body under 10 % (all
        //   four tiers): each `16777216 + 1 = 16777217` is a tie that
        //   rounds to EVEN -> stays 16777216, five times (the C# stalls
        //   here too). One pre-summed `+= 5` gives 16777221 -> rounds to
        //   16777220: DIFFERENT.
        // - barrage, num 2 (step exactly 2), body under 1/6 (all seven
        //   tiers): eight `+2` adds are exact on the 2-wide grid ->
        //   16777232; also pins that the accumulator argument is honoured.
        let base = 16_777_216.0f32; // 2^24
        assert_eq!(free_head_fireball_step(base, 1.0, 1000, 15000), base);
        assert_ne!(base + 5.0, base, "the pre-sum diverges at 2^24");
        assert_eq!(
            free_head_barrage_step(base, 2.0, 1000, 15000, false),
            base + 16.0
        );
    }
}
