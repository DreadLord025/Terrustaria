//! The damage math pipeline — `Main.DamageVar` / `Main.CalculateDamageNPCsTake`
//! (`Main.cs:66118`, `:66147`), `NPC.checkArmorPenetration` (`NPC.cs:81148`)
//! and the `StrikeNPC_Inner` defense/crit line (`NPC.cs:81226`).
//!
//! Pure math over a [`DropRng`] stream; the callers own the combat entities.

use crate::loot::DropRng;

/// C# `Math.Round` — banker's rounding (midpoints to even). Rust's `f64::
/// round` is half-away-from-zero, which differs on exactly `.5` remainders —
/// `DamageVar`'s final `(int)Math.Round(num)` must match. Shared: the regen
/// engine's final life add rounds the same way (`Player.cs:19262`).
pub fn round_bankers(x: f64) -> i64 {
    let floor = x.floor();
    let diff = x - floor;
    if diff > 0.5 {
        (floor + 1.0) as i64
    } else if diff < 0.5 {
        floor as i64
    } else if (floor as i64) % 2 == 0 {
        floor as i64
    } else {
        (floor + 1.0) as i64
    }
}

/// `Main.DamageVar(dmg, luck)` (`Main.cs:66118-66134`): ±15 % variance with
/// the luck reroll (positive luck rerolls and keeps the higher, negative the
/// lower), rounded banker's.
pub fn damage_var(rng: &mut dyn DropRng, dmg: f32, luck: f32) -> i32 {
    let mut num = dmg * (1.0 + rng.next_range(-15, 16) as f32 * 0.01);
    if luck > 0.0 {
        if rng.next_float() < luck {
            let reroll = dmg * (1.0 + rng.next_range(-15, 16) as f32 * 0.01);
            if reroll > num {
                num = reroll;
            }
        }
    } else if luck < 0.0 && rng.next_float() < 0.0 - luck {
        let reroll = dmg * (1.0 + rng.next_range(-15, 16) as f32 * 0.01);
        if reroll < num {
            num = reroll;
        }
    }
    round_bankers(num as f64) as i32
}

/// `Main.CalculateDamageNPCsTake` (`Main.cs:66147-66154`): `Damage −
/// Defense * 0.5`, floored at 1.
pub fn calculate_damage_npcs_take(damage: i32, defense: i32) -> f64 {
    let num = damage as f64 - defense as f64 * 0.5;
    if num < 1.0 { 1.0 } else { num }
}

/// `NPC.checkArmorPenetration` (`NPC.cs:81148-81169`): the debuff bonuses
/// (ichor +15, broken armor +20, Betsy's curse +40) join the weapon's flat
/// penetration plus `defense * armorPenetrationPercent`; the result caps at
/// `defense / 2`.
pub fn check_armor_penetration(
    mut armor_penetration: i32,
    armor_penetration_percent: f32,
    defense: i32,
    ichor: bool,
    broken_armor: bool,
    betsys_curse: bool,
) -> i32 {
    if ichor {
        armor_penetration += 15;
    }
    if broken_armor {
        armor_penetration += 20;
    }
    if betsys_curse {
        armor_penetration += 40;
    }
    armor_penetration += (defense as f32 * armor_penetration_percent) as i32;
    if armor_penetration <= 0 || defense <= 0 {
        return 0;
    }
    if armor_penetration > defense {
        return defense / 2;
    }
    armor_penetration / 2
}

/// The `StrikeNPC_Inner` damage line (`NPC.cs:81226-81229`):
/// `max(1, CalculateDamageNPCsTake(Damage, defense) * (crit ? 2 : 1))`.
pub fn strike_npc_damage(damage: i32, defense: i32, crit: bool) -> i32 {
    let num = calculate_damage_npcs_take(damage, defense) * if crit { 2.0 } else { 1.0 };
    let num = num.max(1.0);
    num as i32
}

/// `NPC.StrikeNPC_Inner`'s knockback section (`NPC.cs:81387-81475`): the
/// `onFire2` x1.1 (`:81390-81393`), the soft caps 8/10/12/14/16
/// (`:81394-81413`), the crit x1.4 (`:81414-81417`), then the damage gate
/// `num4 = damage * 10 > lifeMax` (expert `* 15`, `:81418-81422`).
///
/// STRONG hit (`num4 > lifeMax`, `:81423-81462`): the X push-away
/// (`:81425-81448`) AND the Y pop (`:81453-81461`, `-num3 * 0.75`, `* 0.5`
/// for no-gravity flyers) — the Y pop lives INSIDE the gate, not outside it.
/// The type-185 x1.5 (`:81449-81452`) has no ported `NpcKind` (SnowFlinx,
/// `NPCID.cs:11047`) and is skipped.
///
/// WEAK hit (else, `:81463-81474`): the velocity is SET outright —
/// `Y = -num3 * 0.75 * knockBackResist` (no-gravity `* 0.5 * knockBackResist`)
/// and `X = num3 * hitDirection * knockBackResist` (the resist multiplies a
/// second time, verbatim). Returns the new velocity.
pub fn knock_back_npc(
    knock_back: f32,
    knock_back_resist: f32,
    on_fire2: bool,
    crit: bool,
    damage: i32,
    life_max: i32,
    velocity: (f32, f32),
    hit_direction: i32,
    no_gravity: bool,
    expert: bool,
) -> (f32, f32) {
    let (mut vx, mut vy) = velocity;
    if knock_back > 0.0 && knock_back_resist > 0.0 {
        let mut num3 = knock_back * knock_back_resist;
        // `NPC.cs:81390-81393` — the Cursed Inferno debuff lifts the impulse
        // by 10 % BEFORE the soft caps.
        if on_fire2 {
            num3 *= 1.1;
        }
        if num3 > 8.0 {
            num3 = 8.0 + (num3 - 8.0) * 0.9;
        }
        if num3 > 10.0 {
            num3 = 10.0 + (num3 - 10.0) * 0.8;
        }
        if num3 > 12.0 {
            num3 = 12.0 + (num3 - 12.0) * 0.7;
        }
        if num3 > 14.0 {
            num3 = 14.0 + (num3 - 14.0) * 0.6;
        }
        if num3 > 16.0 {
            num3 = 16.0;
        }
        if crit {
            num3 *= 1.4;
        }
        let num4 = damage * if expert { 15 } else { 10 };
        if num4 > life_max {
            // The X push-away (`NPC.cs:81425-81448`); the shape resists an
            // existing drift away.
            if hit_direction < 0 && vx > -num3 {
                if vx > 0.0 {
                    vx -= num3;
                }
                vx -= num3;
                if vx < -num3 {
                    vx = -num3;
                }
            } else if hit_direction > 0 && vx < num3 {
                if vx < 0.0 {
                    vx += num3;
                }
                vx += num3;
                if vx > num3 {
                    vx = num3;
                }
            }
            // The Y pop (`NPC.cs:81453-81461`) — INSIDE the strong-hit gate:
            // upward unless the target is a no-gravity flyer (pushed at 0.5).
            let pop = if no_gravity { num3 * -0.5 } else { num3 * -0.75 };
            if vy > pop {
                vy += pop;
                if vy < pop {
                    vy = pop;
                }
            }
        } else {
            // The weak-hit branch (`NPC.cs:81463-81474`): the velocity is set
            // outright, the resist multiplying a second time (verbatim).
            vy = if !no_gravity {
                (0.0 - num3) * 0.75 * knock_back_resist
            } else {
                (0.0 - num3) * 0.5 * knock_back_resist
            };
            vx = num3 * hit_direction as f32 * knock_back_resist;
        }
    }
    (vx, vy)
}

/// `Player.Hurt`'s immunity window in ticks (`Player.cs:38385`): PVP 8;
/// otherwise damage != 1 -> long-invincibility 80 : 40, exactly-1 damage ->
/// 40 : 20.
pub fn iframes_for(damage: i32, pvp: bool, long_invince: bool) -> i32 {
    if pvp {
        8
    } else if damage != 1 {
        if long_invince { 80 } else { 40 }
    } else if long_invince {
        40
    } else {
        20
    }
}

/// The crit roll (`Player.cs` `Main.rand.Next(1, 101) <= crit`, base crit 4
/// per class at `Player.cs:2430`): `crit` is the percentage chance.
pub fn crit_roll(rng: &mut dyn DropRng, crit: i32) -> bool {
    rng.next_range(1, 101) <= crit
}

/// `Projectile.Damage_PVE_Inner`'s per-hit crit block (`Projectile.cs:12883-
/// 12917`), run on the `UpdateProjectiles` stream at HIT time (NOT at spawn).
/// Each set class flag rolls an independent `Next(1, 101) <= classCrit +
/// bonusCritChance`; a hit on ANY flag crits. The melee/ranged/magic arms
/// fire in that order, each consuming its draw only when its flag is set
/// (`:12890`/`:12894`/`:12898`). `TryConsumingTimerCrit` (`:12902`) and the
/// monk-set / sentry-backpack arms (`:12903-12916`) are unported accessories
/// (no port state) and consume nothing. The type-1130 force-false and the
/// tag's forced crit are applied by the caller around this block.
pub fn projectile_hit_crit(
    rng: &mut dyn DropRng,
    melee: bool,
    ranged: bool,
    magic: bool,
    melee_crit: i32,
    ranged_crit: i32,
    magic_crit: i32,
    bonus_crit_chance: i32,
) -> bool {
    let mut crit = false;
    let bonus = if bonus_crit_chance > 0 { bonus_crit_chance } else { 0 };
    if melee && rng.next_range(1, 101) <= melee_crit + bonus {
        crit = true;
    }
    if ranged && rng.next_range(1, 101) <= ranged_crit + bonus {
        crit = true;
    }
    if magic && rng.next_range(1, 101) <= magic_crit + bonus {
        crit = true;
    }
    crit
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile_light::FastRandom;

    #[test]
    fn damage_var_stays_inside_the_15_percent_band() {
        // Main.cs:66121: dmg * (1 + Next(-15, 16) * 0.01) — ±15 %.
        for seed in 0..40 {
            let mut rng = FastRandom::new(seed);
            for dmg in [1.0f32, 7.0, 15.0, 60.0] {
                let v = damage_var(&mut rng, dmg, 0.0);
                let lo = (dmg * 0.85).floor() as i32 - 1;
                let hi = (dmg * 1.15).ceil() as i32 + 1;
                assert!((lo..=hi).contains(&v), "seed {seed} dmg {dmg} -> {v}");
            }
        }
    }

    #[test]
    fn damage_var_is_deterministic_per_seed() {
        // Same seed -> the same rolls (the app drives it from the item stream).
        let mut a = FastRandom::new(7);
        let mut b = FastRandom::new(7);
        for _ in 0..20 {
            assert_eq!(damage_var(&mut a, 25.0, 0.0), damage_var(&mut b, 25.0, 0.0));
        }
    }

    #[test]
    fn positive_luck_rerolls_keep_the_higher() {
        // The reroll branch can only raise the result: across many seeds the
        // luck-1 distribution never lands below the no-luck minimum for the
        // same first roll.
        let mut rng = FastRandom::new(9);
        for _ in 0..100 {
            let v = damage_var(&mut rng, 100.0, 1.0);
            assert!((85..=115).contains(&v), "v = {v}");
        }
    }

    #[test]
    fn bankers_rounding_matches_the_c_sharp_midpoints() {
        // Math.Round is to-even: 0.5 -> 0, 1.5 -> 2, 2.5 -> 2, -0.5 -> 0.
        assert_eq!(round_bankers(0.5), 0);
        assert_eq!(round_bankers(1.5), 2);
        assert_eq!(round_bankers(2.5), 2);
        assert_eq!(round_bankers(-0.5), 0);
        assert_eq!(round_bankers(-1.5), -2);
        assert_eq!(round_bankers(20.5), 20);
        // Non-midpoints behave like normal rounding.
        assert_eq!(round_bankers(20.7), 21);
        assert_eq!(round_bankers(20.2), 20);
    }

    #[test]
    fn npc_damage_takes_defense_halved_with_a_floor_of_one() {
        // Main.cs:66147: Damage - Defense * 0.5, min 1.
        assert_eq!(calculate_damage_npcs_take(25, 2), 24.0);
        assert_eq!(calculate_damage_npcs_take(10, 20), 1.0);
        assert_eq!(calculate_damage_npcs_take(1, 100), 1.0);
        // Odd defense halves down (x0.5 of an odd int ends in .5, kept in the
        // f64 result — StrikeNPC rounds only after the crit multiply).
        assert_eq!(calculate_damage_npcs_take(25, 7), 21.5);
    }

    #[test]
    fn strike_npc_line_applies_the_crit_after_defense() {
        // NPC.cs:81226: the crit x2 multiplies the post-defense value.
        assert_eq!(strike_npc_damage(25, 2, false), 24);
        assert_eq!(strike_npc_damage(25, 2, true), 48);
        // The floor holds through the crit path.
        assert_eq!(strike_npc_damage(1, 100, true), 2);
    }

    #[test]
    fn armor_penetration_caps_at_half_defense() {
        // NPC.cs:81148-81169: debuffs join the flat pen, the result caps at
        // defense / 2, and no defense means no penetration.
        assert_eq!(check_armor_penetration(0, 0.0, 20, false, false, false), 0);
        assert_eq!(check_armor_penetration(10, 0.0, 20, false, false, false), 5);
        assert_eq!(
            check_armor_penetration(50, 0.0, 20, false, false, false),
            10
        );
        // The percent bonus lifts ap to 10, and the whole result halves:
        // 10 / 2 = 5.
        assert_eq!(check_armor_penetration(0, 0.5, 20, false, false, false), 5);
        // Ichor lifts ap to 15 -> halved: 7. Broken armor vs defense 10:
        // ap 20 > 10 -> the defense/2 cap: 5.
        assert_eq!(check_armor_penetration(0, 0.0, 20, true, false, false), 7);
        assert_eq!(check_armor_penetration(0, 0.0, 10, false, true, false), 5);
        assert_eq!(check_armor_penetration(0, 0.0, 0, true, true, true), 0);
    }

    #[test]
    fn crit_roll_is_the_1_to_100_le_crit_gate() {
        // crit 0 (the ported items) never crits; crit 100 always does.
        let mut rng = FastRandom::new(1);
        for _ in 0..50 {
            assert!(!crit_roll(&mut rng, 0));
        }
        let mut rng = FastRandom::new(1);
        for _ in 0..50 {
            assert!(crit_roll(&mut rng, 100));
        }
    }

    #[test]
    fn knockback_weak_hit_sets_velocity_outright() {
        // NPC.cs:81463-81474 — when `damage * 10 <= lifeMax` the velocity is
        // SET, not nudged: X = num3 * hitDirection * resist, Y = -num3 * 0.75
        // * resist (no-gravity 0.5). The resist multiplies a SECOND time.
        // kb 5, resist 0.8 -> num3 = 4; damage 2 -> num4 = 20 <= lifeMax 100.
        let (vx, vy) = knock_back_npc(5.0, 0.8, false, false, 2, 100, (3.0, 1.0), 1, false, false);
        assert!((vx - 4.0 * 1.0 * 0.8).abs() < 1e-5, "vx {vx}");
        assert!((vy - (-4.0 * 0.75 * 0.8)).abs() < 1e-5, "vy {vy}");
        // The no-gravity flyer's weak branch uses 0.5 (`:81471`).
        let (_, vy) = knock_back_npc(5.0, 0.8, false, false, 2, 100, (3.0, 1.0), 1, true, false);
        assert!((vy - (-4.0 * 0.5 * 0.8)).abs() < 1e-5, "vy {vy}");
        // The hit direction flips the X sign (`:81473`).
        let (vx, _) = knock_back_npc(5.0, 0.8, false, false, 2, 100, (3.0, 1.0), -1, false, false);
        assert!((vx - (-4.0 * 0.8)).abs() < 1e-5, "vx {vx}");
    }

    #[test]
    fn knockback_strong_hit_pops_y_inside_the_gate() {
        // NPC.cs:81423-81461 — the Y pop runs only when `num4 > lifeMax`.
        // kb 5, resist 1 -> num3 = 5; damage 20 -> num4 = 200 > lifeMax 100.
        // Starting vy 0 > pop -3.75, so vy += -3.75 -> -3.75.
        let (vx, vy) = knock_back_npc(5.0, 1.0, false, false, 20, 100, (0.0, 0.0), 1, false, false);
        assert!((vx - 5.0).abs() < 1e-5, "vx {vx}");
        assert!((vy - (-5.0 * 0.75)).abs() < 1e-5, "vy {vy}");
        // A WEAK hit with the same velocity must NOT pop Y (the gate): the
        // old port popped unconditionally — this pins the fix.
        let (_, vy) = knock_back_npc(5.0, 1.0, false, false, 2, 100, (0.0, 0.0), 1, false, false);
        // Weak branch sets vy = -5 * 0.75 * 1 = -3.75 too, but via the SET
        // path; the distinguishing case is a falling target (vy > 0): the
        // strong path ADDS the pop, the weak path OVERWRITES.
        let (_, vy_strong) =
            knock_back_npc(5.0, 1.0, false, false, 20, 100, (0.0, 2.0), 1, false, false);
        let (_, vy_weak) =
            knock_back_npc(5.0, 1.0, false, false, 2, 100, (0.0, 2.0), 1, false, false);
        assert!((vy_strong - (2.0 - 3.75)).abs() < 1e-5, "strong adds: {vy_strong}");
        assert!((vy_weak - (-3.75)).abs() < 1e-5, "weak overwrites: {vy_weak}");
    }

    #[test]
    fn knockback_on_fire2_lifts_the_impulse_ten_percent() {
        // NPC.cs:81390-81393 — onFire2 multiplies num3 by 1.1 BEFORE the caps.
        // kb 5, resist 1 -> num3 = 5 -> 5.5; weak hit sets vy = -5.5*0.75.
        let (_, vy) = knock_back_npc(5.0, 1.0, true, false, 2, 100, (0.0, 0.0), 1, false, false);
        assert!((vy - (-5.0 * 1.1 * 0.75)).abs() < 1e-4, "vy {vy}");
        // Without the debuff the impulse is the plain 5.
        let (_, vy_plain) =
            knock_back_npc(5.0, 1.0, false, false, 2, 100, (0.0, 0.0), 1, false, false);
        assert!((vy_plain - (-5.0 * 0.75)).abs() < 1e-5, "vy {vy_plain}");
    }

    #[test]
    fn knockback_crit_multiplies_after_the_soft_caps() {
        // NPC.cs:81414-81417 — the crit x1.4 lands after the cap chain. A
        // capped impulse (num3 = 16) crits to 22.4, NOT capped again.
        // kb 30, resist 1 -> raw 30 -> caps to 16; weak hit vy = -16*1.4*0.75.
        let (_, vy) = knock_back_npc(30.0, 1.0, false, true, 2, 100, (0.0, 0.0), 1, false, false);
        assert!((vy - (-16.0 * 1.4 * 0.75)).abs() < 1e-4, "vy {vy}");
    }

    #[test]
    fn knockback_expert_uses_the_fifteen_times_gate() {
        // NPC.cs:81418-81422 — expert mode reads `num * 15`, classic `* 10`.
        // damage 8: classic num4 = 80 <= lifeMax 100 (weak), expert 120 > 100
        // (strong). The strong path adds the pop to vy 2; the weak overwrites.
        let (_, vy_classic) =
            knock_back_npc(5.0, 1.0, false, false, 8, 100, (0.0, 2.0), 1, false, false);
        let (_, vy_expert) =
            knock_back_npc(5.0, 1.0, false, false, 8, 100, (0.0, 2.0), 1, false, true);
        assert!((vy_classic - (-3.75)).abs() < 1e-5, "classic weak: {vy_classic}");
        assert!((vy_expert - (2.0 - 3.75)).abs() < 1e-5, "expert strong: {vy_expert}");
    }

    #[test]
    fn projectile_hit_crit_rolls_one_draw_per_set_flag() {
        // Projectile.cs:12890-12898 — each set class flag rolls an independent
        // Next(1, 101); a hit on ANY flag crits. The draw count is the number
        // of set flags, in melee/ranged/magic order.
        // No flags: no draw, no crit.
        let mut rng = FastRandom::new(3);
        let before = rng.seed;
        assert!(!projectile_hit_crit(&mut rng, false, false, false, 100, 100, 100, 0));
        assert_eq!(rng.seed, before, "no flag spends no draw");
        // crit 100 on the one set flag always crits; the other flags' crits
        // are irrelevant (their draws are not spent).
        let mut rng = FastRandom::new(3);
        assert!(projectile_hit_crit(&mut rng, true, false, false, 100, 0, 0, 0));
        // crit 0 never crits even with the flag set.
        let mut rng = FastRandom::new(3);
        for _ in 0..30 {
            assert!(!projectile_hit_crit(&mut rng, false, true, false, 0, 0, 0, 0));
        }
        // The bonus adds to the class crit (`:12886-12889`): a 0 class crit
        // with +100 bonus always crits.
        let mut rng = FastRandom::new(3);
        assert!(projectile_hit_crit(&mut rng, false, false, true, 0, 0, 0, 100));
        // A negative bonus is clamped to 0 (`:12886`'s `> 0` gate).
        let mut rng = FastRandom::new(3);
        for _ in 0..30 {
            assert!(!projectile_hit_crit(&mut rng, true, false, false, 0, 0, 0, -50));
        }
    }

    #[test]
    fn projectile_hit_crit_draw_count_matches_the_flag_count() {
        // The stream advance is exactly one Next(1, 101) per set flag — the
        // RNG-order invariant (the crit rides the strike stream before
        // DamageVar). Replay the same draws against a fresh stream.
        for (melee, ranged, magic) in [
            (true, false, false),
            (false, true, false),
            (false, false, true),
            (true, true, false),
            (true, true, true),
        ] {
            let mut a = FastRandom::new(11);
            let got = projectile_hit_crit(&mut a, melee, ranged, magic, 50, 50, 50, 0);
            let mut b = FastRandom::new(11);
            let mut want = false;
            if melee && b.next_range(1, 101) <= 50 {
                want = true;
            }
            if ranged && b.next_range(1, 101) <= 50 {
                want = true;
            }
            if magic && b.next_range(1, 101) <= 50 {
                want = true;
            }
            assert_eq!(got, want, "flags {melee}/{ranged}/{magic}");
            assert_eq!(a.seed, b.seed, "the draw count matches the flag count");
        }
    }
}
