//! The damage math pipeline — `Main.DamageVar` / `Main.CalculateDamageNPCsTake`
//! (`Main.cs:66118`, `:66147`), `NPC.checkArmorPenetration` (`NPC.cs:81148`)
//! and the `StrikeNPC_Inner` defense/crit line (`NPC.cs:81226`).
//!
//! Pure math over a [`DropRng`] stream; the callers own the combat entities.

use crate::loot::DropRng;

/// C# `Math.Round` — banker's rounding (midpoints to even). Rust's `f64::
/// round` is half-away-from-zero, which differs on exactly `.5` remainders —
/// `DamageVar`'s final `(int)Math.Round(num)` must match.
fn round_bankers(x: f64) -> i64 {
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

/// `NPC.StrikeNPC_Inner`'s knockback section (`NPC.cs:81389-81458`): the
/// soft caps 8/10/12/14/16, the crit x1.4, the damage gate
/// (`damage * 10 > lifeMax` — expert `* 15`) gating the X push, and the Y pop
/// (`-kb * 0.75`, `* 0.5` for no-gravity flyers). Returns the new velocity.
pub fn knock_back_npc(
    knock_back: f32,
    knock_back_resist: f32,
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
        // The X push-away runs only when the hit is strong for the target
        // (`num4 > lifeMax`); the shape resists an existing drift away.
        if num4 > life_max {
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
        }
        // The Y pop always runs (inside the kb/resist gate): upward unless the
        // target is a no-gravity flyer, which gets pushed down at 0.5.
        let pop = if no_gravity {
            num3 * -0.5
        } else {
            num3 * -0.75
        };
        if vy > pop {
            vy += pop;
            if vy < pop {
                vy = pop;
            }
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
}
