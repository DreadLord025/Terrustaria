//! The fishing engine — `Player.GetFishingConditions` (`Player.cs:42197-
//! 42300`), the bobber's bite accumulator (`Projectile.AI_061_FishingBobber`
//! `Projectile.cs:53820-53847`), the attempt builder (`Projectile.FishingCheck`
//! `:20583-20760` + `GetFishingPondState` `:21576-21623`) and the drop-level
//! rolls (`FishingCheck_RollDropLevels` `:21512-21545`).
//!
//! Source map (line numbers into `decompiled/Terraria/`):
//! - the power total (`:42202-42208`): bait + pole + `fishingSkill` + the
//!   buffs (the 25 fishing buff +5, `canFloatInWater && wet` +5, sitting +5),
//!   scaled by `Fishing_GetPowerMultiplier` (`:42233-42263`)
//! - the multiplier: the rain ×1.2, the background clouds ×1.1, the early/
//!   late daylight ×1.3, the midday ×0.8, the mid-night ×0.8, the moon
//!   phases (0 ×1.1, 1|7 ×1.05, 3|5 ×0.95, 4 ×0.9), the blood moon ×1.1
//! - the bite accumulator (`:53821-53847`): `Next(300) < level` adds
//!   `Next(1,3)`, then `+= level/30`, `+= Next(1,3)`, `Next(60)==0` adds 60;
//!   at `> 660` the reset fires `FishingCheck`
//! - the attempt gates (`:20583-20760`): the ≥75 water tile floor, the chum
//!   bonuses +11/+6/+3, `waterNeededToFish = 300 × atmo` (the height
//!   formula `:20710-20720`), the quality scaling and the depth ladder
//!   (`:20693-20712`)
//! - the drop levels (`:21512-21545`): `150/level`, `300/level`, `1050/
//!   level`, `2250/level`, `4500/level` with the mins 2/3 and the crate
//!   potion's +15 of 10
//!
//! Deviations (ANALYSIS "Fishing engine (M7-FISH-1)"): the drop TABLES
//! (`GameContent/FishDropRules/*`, FISH-2), the bobber projectile's flight
//! (aiStyle 61's states — the rod items and the water-line physics land
//! with the pole family), the quest fish and the chum buckets' projectile
//! side are follow-ups; the RNG rides the caller's stream.

use crate::tile::{FLAG_LIQUID_HONEY, FLAG_LIQUID_LAVA};
use crate::tile_light::FastRandom;
use crate::world::World;

/// `Fishing_GetPowerMultiplier` (`Player.cs:42233-42263`): the weather/time/
/// moon multiplier. `time` is `Main.time` (the day runs 0..54000, the night
/// 0..32400), `moon_phase` 0-7 (0 = full, 4 = new).
pub fn power_multiplier(
    rain: bool,
    cloud_bg_alpha: f32,
    day_time: bool,
    time: f64,
    moon_phase: i32,
    blood_moon: bool,
) -> f32 {
    let mut m = 1.0f32;
    if rain {
        m *= 1.2;
    }
    if cloud_bg_alpha > 0.0 {
        m *= 1.1;
    }
    if day_time && (time < 5400.0 || time > 48600.0) {
        m *= 1.3;
    }
    if day_time && time > 16200.0 && time < 37800.0 {
        m *= 0.8;
    }
    if !day_time && time > 6480.0 && time < 25920.0 {
        m *= 0.8;
    }
    if moon_phase == 0 {
        m *= 1.1;
    }
    if moon_phase == 1 || moon_phase == 7 {
        m *= 1.05;
    }
    if moon_phase == 3 || moon_phase == 5 {
        m *= 0.95;
    }
    if moon_phase == 4 {
        m *= 0.9;
    }
    if blood_moon {
        m *= 1.1;
    }
    m
}

/// `Player.GetFishingConditions` (`Player.cs:42197-42230`): the total power —
/// the bait + the pole + the skill + the situational buffs, times the
/// multiplier. The bait 2673 (the Lava fishing's "no bait" marker) and the
/// zero pole/bait return 0.
pub fn final_fishing_level(
    pole_power: i32,
    bait_power: i32,
    fishing_skill: i32,
    fishing_buff: bool,
    water_floating: bool,
    sitting: bool,
    rain: bool,
    cloud_bg_alpha: f32,
    day_time: bool,
    time: f64,
    moon_phase: i32,
    blood_moon: bool,
) -> i32 {
    if bait_power == 0 || pole_power == 0 {
        return 0;
    }
    let mut bonus = 0;
    if fishing_buff {
        bonus += 5;
    }
    if water_floating {
        bonus += 5;
    }
    if sitting {
        bonus += 5;
    }
    let total = bait_power + pole_power + fishing_skill + bonus;
    let mult = power_multiplier(rain, cloud_bg_alpha, day_time, time, moon_phase, blood_moon);
    (total as f32 * mult) as i32
}

/// `GetFishingPondState` (`Projectile.cs:21576-21623`): the pond scan — the
/// liquid span left/right along the bobber's row, then the columns down to
/// the floor. The honey pond counts ×1.5. The chums ride the caller (the
/// chum projectile book is FISH-2).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PondState {
    pub water_tiles: i32,
    pub in_lava: bool,
    pub in_honey: bool,
}

pub fn get_fishing_pond_state(world: &World, x: i32, y: i32) -> PondState {
    let mut state = PondState::default();
    // `GetFishingPondWidth` (`:21625-21642`): the liquid span on the row,
    // bounded by the solids and the 10-tile world margins.
    let mut min_x = x;
    let mut max_x = x;
    while min_x > 10
        && world
            .tile(min_x - 1, y)
            .is_some_and(|t| t.liquid > 0 && !solid(world, min_x - 1, y))
    {
        min_x -= 1;
    }
    while max_x < world.width - 10
        && world
            .tile(max_x + 1, y)
            .is_some_and(|t| t.liquid > 0 && !solid(world, max_x + 1, y))
    {
        max_x += 1;
    }
    for i in min_x..=max_x {
        let mut j = y;
        while let Some(t) = world.tile(i, j) {
            if t.liquid == 0 || solid(world, i, j) || j >= world.height - 10 {
                break;
            }
            state.water_tiles += 1;
            if t.flags & FLAG_LIQUID_LAVA != 0 {
                state.in_lava = true;
            } else if t.flags & FLAG_LIQUID_HONEY != 0 {
                state.in_honey = true;
            }
            j += 1;
        }
    }
    if state.in_honey {
        // `:21599-21601`: the honey pond counts ×1.5.
        state.water_tiles = (state.water_tiles as f64 * 1.5) as i32;
    }
    state
}

fn solid(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| t.active() && crate::tiledata::TILE_SOLID.has(t.kind) && t.flags & crate::tile::FLAG_INACTIVE == 0)
}

/// The `FishingAttempt` slice the engine carries (`Projectile.cs:20583+`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FishingAttempt {
    pub x: i32,
    pub y: i32,
    pub water_tiles_count: i32,
    pub in_lava: bool,
    pub in_honey: bool,
    /// `fisher.fishingLevel` after the chum/quality adjustments.
    pub fishing_level: i32,
    /// `fisher.waterNeededToFish` after the atmosphere scaling.
    pub water_needed: i32,
    /// `fisher.waterQuality` after the quality flip (`1f - quality`).
    pub water_quality: f32,
    /// The junk roll inputs (`:20743`): both `Next(50) > level` rolls and
    /// the pond below the needed size.
    pub junk: bool,
    /// The depth ladder (`:20693-20712`): 0 = the space, 1 = the surface,
    /// 3 = the underground, 2 = the cavern (the 50 % day re-roll to 1),
    /// 4 = the underworld.
    pub height_level: i32,
}

/// `FishingCheck`'s attempt builder (`:20583-20760`): the gates and the
/// level adjustments. `chums_in_water` feeds the +11/+6/+3 bonuses; the
/// roll rng rides the caller (the C# `Main.rand`).
pub fn try_build_fishing_attempt(
    world: &World,
    x: i32,
    y: i32,
    base_level: i32,
    chums_in_water: i32,
    rng: &mut FastRandom,
) -> Option<FishingAttempt> {
    let pond = get_fishing_pond_state(world, x, y);
    // `:20633-20636`: the hard 75-tile floor.
    if pond.water_tiles < 75 {
        return None;
    }
    if base_level == 0 {
        return None;
    }
    let mut level = base_level;
    // `:20653-20658`: the chum bonuses.
    if chums_in_water > 0 {
        level += 11;
    }
    if chums_in_water > 1 {
        level += 6;
    }
    if chums_in_water > 2 {
        level += 3;
    }
    // `:20665-20674`: `waterNeededToFish = 300 * atmo`, the atmosphere from
    // the height formula `num = maxTilesX / 4200`,
    // `atmo = (y - (60 + 10*num²)) / (worldSurface / 6)`, clamped 0.25..1.
    let num = world.width as f32 / 4200.0;
    let mut atmo = ((y as f32 - (60.0 + 10.0 * num * num)) / (world.world_surface as f32 / 6.0)) as f32;
    if atmo < 0.25 {
        atmo = 0.25;
    }
    if atmo > 1.0 {
        atmo = 1.0;
    }
    let water_needed = (300.0 * atmo) as i32;
    let mut quality = pond.water_tiles as f32 / water_needed as f32;
    if quality < 1.0 {
        level = (level as f32 * quality) as i32;
    }
    let quality = 1.0 - quality;
    // `:20743`: the junk roll — both `Next(50) > level` and the short pond.
    let junk = rng.next(50) > level
        && rng.next(50) > level
        && pond.water_tiles < water_needed;
    // The depth ladder (`:20683-20715`): the cavern's 50 % day re-roll rides
    // the caller's stream.
    let height_level = if y as f64 >= (world.height as f64 - 42.0) * 0.998_958_333_333_333_3 {
        4
    } else if y as f64 > world.rock_layer {
        2
    } else if y as f64 > world.world_surface {
        3
    } else if (y as f64) < world.world_surface * 0.5 {
        0
    } else {
        1
    };
    let height_level = if height_level == 2 && rng.next(2) == 0 {
        1
    } else {
        height_level
    };
    Some(FishingAttempt {
        x,
        y,
        water_tiles_count: pond.water_tiles,
        in_lava: pond.in_lava,
        in_honey: pond.in_honey,
        fishing_level: level,
        water_needed,
        water_quality: quality,
        junk,
        height_level,
    })
}

/// `FishingCheck`'s hit roll (`:20587-20591`): `(level + 75) / 2` of 100.
pub fn fishing_check_roll(level: i32, rng: &mut FastRandom) -> bool {
    let num = (level + 75) / 2;
    rng.next(100) <= num
}

/// `AI_061`'s bite accumulator (`Projectile.cs:53821-53847`): one scan step;
/// returns the new timer value (the caller resets and fires `FishingCheck`
/// when it exceeds 660).
pub fn accumulate_bite(current: f32, level: i32, rng: &mut FastRandom) -> f32 {
    // `Main.rand.Next(1, 3)` = `Next(2) + 1`.
    let mut t = current;
    if rng.next(300) < level {
        t += (rng.next(2) + 1) as f32;
    }
    t += (level / 30) as f32;
    t += (rng.next(2) + 1) as f32;
    if rng.next(60) == 0 {
        t += 60.0;
    }
    t
}

/// The drop-level rolls (`FishingCheck_RollDropLevels`, `:21512-21545`):
/// `150/level` (common), `300/level` (uncommon), `1050/level` (rare),
/// `2250/level` (very rare), `4500/level` (legendary), `10 [+15 crate
/// potion]` (crate), with the mins 2/3.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DropLevels {
    pub common: bool,
    pub uncommon: bool,
    pub rare: bool,
    pub very_rare: bool,
    pub legendary: bool,
    pub crate_hit: bool,
}

pub fn roll_drop_levels(level: i32, crate_potion: bool, rng: &mut FastRandom) -> DropLevels {
    let mut nums = [
        150 / level.max(1),
        150 * 2 / level.max(1),
        150 * 7 / level.max(1),
        150 * 15 / level.max(1),
        150 * 30 / level.max(1),
    ];
    if nums[0] < 2 {
        nums[0] = 2;
    }
    if nums[1] < 3 {
        nums[1] = 3;
    }
    let mut crate_roll = 10;
    if crate_potion {
        crate_roll += 15;
    }
    DropLevels {
        common: rng.next(nums[0]) == 0,
        uncommon: rng.next(nums[1]) == 0,
        rare: rng.next(nums[2]) == 0,
        very_rare: rng.next(nums[3]) == 0,
        legendary: rng.next(nums[4]) == 0,
        // `:21571-21574`: `Next(100) < num6` — the percentage roll.
        crate_hit: rng.next(100) < crate_roll,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{FLAG_ACTIVE, Tile};

    fn world() -> World {
        let mut w = World::new("t".into(), 1200, 400);
        // world_surface defaults to height * 0.25 = 100; rock_layer to
        // height * 0.2.
        w.rock_layer = 120.0;
        w
    }

    fn put_water(w: &mut World, x: i32, y: i32, amount: u8) {
        let i = w.index(x, y);
        w.tiles[i].liquid = amount;
    }

    #[test]
    fn the_power_multiplier_windows_are_verbatim() {
        // `Player.cs:42233-42263`.
        let base = power_multiplier(false, 0.0, true, 27000.0, 2, false);
        assert!((base - 0.8).abs() < 1e-6, "the midday window");
        let early = power_multiplier(false, 0.0, true, 3000.0, 2, false);
        assert!((early - 1.3).abs() < 1e-6, "the early morning window");
        let night = power_multiplier(false, 0.0, false, 12000.0, 2, false);
        assert!((night - 0.8).abs() < 1e-6, "the mid-night window");
        let full_moon_rain = power_multiplier(true, 1.0, true, 3000.0, 0, true);
        assert!((full_moon_rain - 1.2 * 1.1 * 1.3 * 1.1 * 1.1).abs() < 1e-4);
        let new_moon = power_multiplier(false, 0.0, false, 3000.0, 4, false);
        assert!((new_moon - 0.9).abs() < 1e-6);
        let gibbous = power_multiplier(false, 0.0, false, 3000.0, 1, false);
        assert!((gibbous - 1.05).abs() < 1e-6);
    }

    #[test]
    fn the_final_level_sums_and_scales() {
        // `:42197-42230`: bait 50 + pole 30 + skill 20 + the fishing buff 5
        // = 105; the midday ×0.8 → 84.
        assert_eq!(
            final_fishing_level(30, 50, 20, true, false, false, false, 0.0, true, 27000.0, 2, false),
            84
        );
        // The zero bait or pole → 0.
        assert_eq!(final_fishing_level(0, 50, 20, false, false, false, false, 0.0, true, 0.0, 2, false), 0);
        assert_eq!(final_fishing_level(30, 0, 20, false, false, false, false, 0.0, true, 0.0, 2, false), 0);
        // The floating + sitting stack +10: 30 + 50 + 10 = 90, ×0.8 → 72.
        assert_eq!(
            final_fishing_level(30, 50, 0, false, true, true, false, 0.0, true, 27000.0, 2, false),
            72
        );
    }

    #[test]
    fn the_pond_scan_walks_the_span_and_counts_honey() {
        let mut w = world();
        for x in 20..32 {
            for y in 40..46 {
                put_water(&mut w, x, y, 255);
            }
        }
        // The banks.
        for y in 40..46 {
            let i = w.index(19, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
            let j = w.index(32, y);
            w.tiles[j] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[j].flags |= FLAG_ACTIVE;
        }
        let pond = get_fishing_pond_state(&w, 25, 40);
        assert_eq!(pond.water_tiles, 12 * 6);
        assert!(!pond.in_lava && !pond.in_honey);
        // The honey flags ×1.5.
        for x in 20..32 {
            for y in 40..46 {
                let i = w.index(x, y);
                w.tiles[i].flags |= crate::tile::FLAG_LIQUID_HONEY;
            }
        }
        let honey = get_fishing_pond_state(&w, 25, 40);
        assert!(honey.in_honey);
        assert_eq!(honey.water_tiles, (12 * 6) as f64 as i32 * 3 / 2);
    }

    #[test]
    fn the_attempt_gates_refuse_short_ponds() {
        let mut w = world();
        // A 10×7 pond = 70 tiles: below the 75 floor. The world surface
        // sits at height*0.25 = 100, so the pond row 60 is the surface
        // band (50..100 → heightLevel 1).
        for x in 20..30 {
            for y in 60..67 {
                put_water(&mut w, x, y, 255);
            }
        }
        let mut rng = FastRandom::new(1);
        assert!(try_build_fishing_attempt(&w, 25, 60, 50, 0, &mut rng).is_none());
        // Extend to 12×7 = 84 tiles: passes.
        for x in 30..32 {
            for y in 60..67 {
                put_water(&mut w, x, y, 255);
            }
        }
        let attempt = try_build_fishing_attempt(&w, 25, 60, 50, 0, &mut rng).unwrap();
        assert_eq!(attempt.water_tiles_count, 84);
        assert_eq!(attempt.height_level, 1, "the surface pond");
    }

    #[test]
    fn the_chums_and_quality_scale_the_level() {
        let mut w = world();
        // A 30×20 pond = 600 tiles ≥ the needed 300 (the atmo clamps to
        // 0.25 this high → 75 needed; 600/75 = 8 → quality flips past 1 →
        // no penalty).
        for x in 20..50 {
            for y in 40..60 {
                put_water(&mut w, x, y, 255);
            }
        }
        let mut rng = FastRandom::new(3);
        let attempt = try_build_fishing_attempt(&w, 30, 40, 100, 3, &mut rng).unwrap();
        // The chums +11 +6 +3 = +20.
        assert_eq!(attempt.fishing_level, 120);
        assert!(
            attempt.water_quality < 0.0,
            "the quality flips negative past the need"
        );
        // A small pond (10×10 = 100 ≥ 75) scales the level down: the
        // quality 100/75 = 1.33 → no penalty; shrink to 80 tiles for the
        // penalty: 80/75 = 1.06 — still above 1; the pond 78 → 1.04... the
        // penalty needs a pond BELOW the need: 74 tiles is under the gate,
        // so scale via a deep atmo: at y 80 the atmo = (80-60.8)/16.67 =
        // 1.15 → 1.0 → need 300 — the pond 100 → 0.33 → level ×0.33.
        let mut w2 = world();
        for x in 20..30 {
            for y in 80..90 {
                put_water(&mut w2, x, y, 255);
            }
        }
        let attempt2 = try_build_fishing_attempt(&w2, 25, 80, 100, 0, &mut rng).unwrap();
        assert!(
            attempt2.fishing_level < 100,
            "the quality penalty applies ({} < 100)",
            attempt2.fishing_level
        );
    }

    #[test]
    fn the_drop_levels_roll_with_the_mins() {
        // `:21512-21545`: at level 50 the windows are 3/6/21/45/90 and the
        // crate 10; a level-1 run clamps the common window to 2 and the
        // uncommon to 3.
        let mut rng = FastRandom::new(7);
        let mut hits = [0i32; 6];
        for _ in 0..3000 {
            let l = roll_drop_levels(50, false, &mut rng);
            hits[0] += l.common as i32;
            hits[1] += l.uncommon as i32;
            hits[2] += l.rare as i32;
            hits[3] += l.very_rare as i32;
            hits[4] += l.legendary as i32;
            hits[5] += l.crate_hit as i32;
        }
        // The expected frequencies over 3000 rolls: 1000/500/~143/~67/~33/
        // 300 — the windows clamp at 2/3 so nothing exceeds the caps.
        assert!(hits[0] > 700 && hits[0] < 1300, "common {}", hits[0]);
        assert!(hits[1] > 300 && hits[1] < 700, "uncommon {}", hits[1]);
        assert!(hits[5] > 150 && hits[5] < 450, "crate {}", hits[5]);
        // The crate potion widens the window (25 vs 10 → ~750/3000); a
        // different seed so the draw positions don't alias.
        let mut rng = FastRandom::new(8);
        let mut crate_hits = 0;
        for _ in 0..3000 {
            if roll_drop_levels(50, true, &mut rng).crate_hit {
                crate_hits += 1;
            }
        }
        // `Next(100) < 10` vs `< 25`: the potion triples the crate rate
        // (~750 vs ~300 over 3000).
        assert!(
            crate_hits > hits[5] + 200,
            "the potion widens the crate window ({} vs {})",
            crate_hits,
            hits[5]
        );
        assert!(hits[5] > 150, "the plain crate window ({})", hits[5]);
    }

    #[test]
    fn the_bite_accumulator_matches_the_verbatim_chain() {
        // `:53821-53847`: `Next(300) < level` adds `Next(1,3)`, then
        // `level/30`, `Next(1,3)`, and `Next(60)==0` adds 60.
        let mut rng = FastRandom::new(11);
        let t = accumulate_bite(0.0, 300, &mut rng);
        // level 300: the 300-window always hits (+1..2), +10, +1..2, the
        // 60-bonus 1/60 — the total is 12..74.
        assert!((12.0..=74.0).contains(&t), "t = {t}");
        // The low level never passes the 300 window.
        let mut rng = FastRandom::new(11);
        let t = accumulate_bite(0.0, 0, &mut rng);
        assert!((1.0..=63.0).contains(&t), "t = {t}");
    }

    #[test]
    fn the_fishing_check_rolls_half_the_level() {
        // `:20587-20591`: `(level + 75) / 2` of 100 — level 125 always hits.
        let mut rng = FastRandom::new(3);
        assert!(fishing_check_roll(125, &mut rng));
        assert!(fishing_check_roll(124, &mut rng));
    }
}

// --- Drop rules (M7-FISH-2, `GameContent/FishDropRules/*`) ---------------
//
// The catch side of the engine: `FishDropRuleList.TryGetItemDropType`
// (`FishDropRuleList.cs:10-21`) walks the registered rules in order and
// the first whose conditions and rarity match wins; stoppers (an empty
// item list) end the walk with no item (`FishDropRule.cs:15-51`). This
// half of the module ports the rule machinery (`FishDropRule.cs`,
// `AFishDropRulePopulator.cs`'s condition/rarity families `:41-172`),
// the full vanilla table (`GameContentFishDropPopulator.cs`'s 142
// invocations -> 158 rules, in `Populate()`'s execution order `:10-30`),
// the quest-fish gating (`FishingConditions.cs:5-27`), the context
// preparation (`Projectile.PrepareFishingContext` `:20857-20908`), the
// enemy roll (`FishingCheck_RollEnemySpawns` `:20804-20835`) and the
// lava fallback (`Projectile.cs:20774-20797`).
//
// Deviations (ANALYSIS "Fishing drop rules (M7-FISH-2)"): the bobber
// projectile's flight (aiStyle 61) and the chum-bucket projectile stay
// out until the pole family lands; the RNG rides the caller's stream,
// so a roll is fully deterministic per seed.

/// `AFishingCondition`'s registered families
/// (`AFishDropRulePopulator.cs:99-172`) plus the two quest-fish variants
/// (`FishingConditions.cs:5-27`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FishCondition {
    Height0,
    Height1,
    Height2,
    Height3,
    Height1And2,
    HeightUnder2,
    HeightAbove1,
    HeightAboveAnd1,
    HeightAbove2,
    UnderRockLayer,
    HardMode,
    EarlyMode,
    DidNotUseCombatBook,
    Junk,
    Crate,
    InLava,
    InHoney,
    CanFishInLava,
    AnyEnemies,
    Dungeon,
    Beach,
    Hallow,
    GlowingMushrooms,
    TrueDesert,
    TrueSnow,
    Remix,
    Corruption,
    Crimson,
    Jungle,
    Snow,
    Desert,
    RolledHallowDesert,
    OriginalOcean,
    RemixOcean,
    Ocean,
    Water1000,
    BloodMoon,
    /// `QuestFishCondition` — the roll only matches while the angler's
    /// current quest is this item.
    QuestFish(u16),
    /// `QuestFishConditionRemix` — the remix-world twin.
    QuestFishRemix(u16),
}

/// `FishRarityCondition`'s tiers (`AFishDropRulePopulator.cs:41-95`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FishRarity {
    Any,
    Legendary,
    VeryRare,
    Rare,
    Uncommon,
    Common,
    BombNotLegendaryNotVeryRareAndUncommon,
    UncommonOrCommon,
}

impl FishRarity {
    /// The tier predicate over the rolled flags
    /// (`AFishDropRulePopulator.cs:61-93`).
    pub fn matches(self, c: &FishingContext) -> bool {
        match self {
            FishRarity::Any => true,
            FishRarity::Legendary => c.legendary,
            FishRarity::VeryRare => c.very_rare,
            FishRarity::Rare => c.rare,
            FishRarity::Uncommon => c.uncommon,
            FishRarity::Common => c.common,
            FishRarity::BombNotLegendaryNotVeryRareAndUncommon => {
                !c.legendary && !c.very_rare && c.uncommon
            }
            FishRarity::UncommonOrCommon => c.uncommon || c.common,
        }
    }
}

/// `FishingContext` (`GameContent/FishDropRules/FishingContext.cs`): the
/// fisher flags, the player zones and the rolled biome flags the
/// conditions read. The player/world fields are plain booleans so the
/// simulation stays decoupled from the host's `Player`/`World`.
#[derive(Clone, Debug, Default)]
pub struct FishingContext {
    // `FishingAttempt`'s rolled fields (`Projectile.cs`).
    pub height_level: i32,
    pub y: i32,
    pub x: i32,
    pub water_tiles_count: i32,
    pub junk: bool,
    pub crate_hit: bool,
    pub common: bool,
    pub uncommon: bool,
    pub rare: bool,
    pub very_rare: bool,
    pub legendary: bool,
    pub in_lava: bool,
    pub in_honey: bool,
    pub can_fish_in_lava: bool,
    /// The enemy roll's result (`FishingCheck_RollEnemySpawns`).
    pub rolled_enemy_spawn: i32,
    /// The angler's current quest item, `-1` when unavailable.
    pub quest_fish: i32,
    // The player's zones (`Player.Zone*`).
    pub zone_corrupt: bool,
    pub zone_crimson: bool,
    pub zone_jungle: bool,
    pub zone_snow: bool,
    pub zone_dungeon: bool,
    pub zone_desert: bool,
    pub zone_hallow: bool,
    pub zone_glowshroom: bool,
    pub zone_beach: bool,
    pub downed_boss3: bool,
    // The world's flags.
    pub hard_mode: bool,
    pub remix_world: bool,
    pub not_the_bees_world: bool,
    pub blood_moon: bool,
    pub rock_layer: i32,
    pub combat_book_was_used: bool,
    pub world_width: i32,
    // `PrepareFishingContext`'s rolled biome lanes.
    pub rolled_corruption: bool,
    pub rolled_crimson: bool,
    pub rolled_jungle: bool,
    pub rolled_snow: bool,
    pub rolled_desert: bool,
    pub rolled_infected_desert: bool,
    pub rolled_remix_ocean: bool,
}

/// `AFishDropRulePopulator.IsOriginalOcean` (`:267-274`): within 380 tiles
/// of either edge, height band <= 1, more than 1000 water tiles.
pub fn is_original_ocean(c: &FishingContext) -> bool {
    c.height_level <= 1
        && (c.x < 380 || c.x > c.world_width.saturating_sub(380))
        && c.water_tiles_count > 1000
}

impl FishCondition {
    /// The predicate (`AFishDropRulePopulator.cs` + `FishingConditions.cs`).
    pub fn matches(self, c: &FishingContext) -> bool {
        match self {
            FishCondition::Height0 => c.height_level == 0,
            FishCondition::Height1 => c.height_level == 1,
            FishCondition::Height2 => c.height_level == 2,
            FishCondition::Height3 => c.height_level == 3,
            FishCondition::Height1And2 => c.height_level == 1 || c.height_level == 2,
            FishCondition::HeightUnder2 => c.height_level < 2,
            FishCondition::HeightAbove1 => c.height_level > 1,
            FishCondition::HeightAboveAnd1 => c.height_level >= 1,
            FishCondition::HeightAbove2 => c.height_level > 2,
            FishCondition::UnderRockLayer => c.y >= c.rock_layer,
            FishCondition::HardMode => c.hard_mode,
            FishCondition::EarlyMode => !c.hard_mode,
            FishCondition::DidNotUseCombatBook => !c.combat_book_was_used,
            FishCondition::Junk => c.junk,
            FishCondition::Crate => c.crate_hit,
            FishCondition::InLava => c.in_lava,
            FishCondition::InHoney => c.in_honey,
            FishCondition::CanFishInLava => c.can_fish_in_lava,
            FishCondition::AnyEnemies => c.rolled_enemy_spawn > 0,
            FishCondition::Dungeon => c.zone_dungeon && c.downed_boss3,
            FishCondition::Beach => c.zone_beach,
            FishCondition::Hallow => c.zone_hallow,
            FishCondition::GlowingMushrooms => c.zone_glowshroom,
            FishCondition::TrueDesert => c.zone_desert,
            FishCondition::TrueSnow => c.zone_snow,
            FishCondition::Remix => c.remix_world,
            FishCondition::Corruption => c.rolled_corruption,
            FishCondition::Crimson => c.rolled_crimson,
            FishCondition::Jungle => c.rolled_jungle,
            FishCondition::Snow => c.rolled_snow,
            FishCondition::Desert => c.rolled_desert,
            FishCondition::RolledHallowDesert => c.rolled_infected_desert && c.zone_hallow,
            FishCondition::OriginalOcean => is_original_ocean(c),
            FishCondition::RemixOcean => c.rolled_remix_ocean,
            FishCondition::Ocean => c.rolled_remix_ocean || is_original_ocean(c),
            FishCondition::Water1000 => c.water_tiles_count > 1000,
            FishCondition::BloodMoon => c.blood_moon,
            FishCondition::QuestFish(id) => c.quest_fish == id as i32,
            FishCondition::QuestFishRemix(id) => c.quest_fish == id as i32 && c.remix_world,
        }
    }
}

/// `FishDropRule` (`FishDropRule.cs:5-51`).
#[derive(Clone, Debug)]
pub struct FishRule {
    pub items: &'static [u16],
    /// `ChanceNumerator`.
    pub num: i32,
    /// `ChanceDenominator`.
    pub den: i32,
    pub rarity: FishRarity,
    pub conditions: &'static [FishCondition],
}

impl FishRule {
    /// `FishDropRule.IsStopper` (`:15-29`): the empty item list is the
    /// stopper (a full-rate Any rule with items is a normal drop).
    pub fn is_stopper(&self) -> bool {
        self.items.is_empty()
    }

    /// `FishDropRule.Attempt` (`:31-51`) — the exact order: conditions,
    /// then the `Next(den) >= num` chance, then the rarity, then the item
    /// pick.
    pub fn attempt(&self, c: &FishingContext, rng: &mut FastRandom) -> Option<u16> {
        if !self.conditions.iter().all(|cond| cond.matches(c)) {
            return None;
        }
        if rng.next(self.den) >= self.num {
            return None;
        }
        if !self.rarity.matches(c) {
            return None;
        }
        if self.items.is_empty() {
            return Some(0);
        }
        // `Random.NextFromList` (`Utils.cs:1213-1215`): uniform, repeated
        // ids give weight.
        let pick = self.items[rng.next(self.items.len() as i32) as usize];
        Some(pick)
    }
}

/// `FishDropRuleList` (`FishDropRuleList.cs:10-21`).
#[derive(Clone, Debug)]
pub struct FishDropRuleList {
    pub rules: Vec<FishRule>,
}

impl FishDropRuleList {
    /// `TryGetItemDropType` (`:10-21`): first matching rule wins; a
    /// stopper returns `Some(0)` (no item).
    pub fn try_get_item_drop(&self, c: &FishingContext, rng: &mut FastRandom) -> Option<u16> {
        for rule in &self.rules {
            if let Some(item) = rule.attempt(c, rng) {
                return Some(item);
            }
        }
        None
    }

    /// `Add` (`FishDropRuleList.cs:48-60`).
    pub fn add(
        &mut self,
        rarity: FishRarity,
        num: i32,
        den: i32,
        items: &'static [u16],
        conditions: &'static [FishCondition],
    ) {
        assert!(den > 0, "ChanceDenominator must be > 0");
        self.rules.push(FishRule {
            items,
            num,
            den,
            rarity,
            conditions,
        });
    }
}
// The condition slices and the item groups the vanilla table registers
// (`GameContentFishDropPopulator.cs`, in `Populate()`'s execution order
// `:10-30`).

const CONDS_LAVA: &[FishCondition] = &[FishCondition::InLava, FishCondition::CanFishInLava];
const CONDS_LAVA_HARD: &[FishCondition] = &[FishCondition::InLava, FishCondition::CanFishInLava, FishCondition::HardMode];
const CONDS_LAVA_EARLY: &[FishCondition] = &[FishCondition::InLava, FishCondition::CanFishInLava, FishCondition::EarlyMode];
const CONDS_LAVA_CRATE_HARD: &[FishCondition] = &[FishCondition::InLava, FishCondition::CanFishInLava, FishCondition::Crate, FishCondition::HardMode];
const CONDS_LAVA_CRATE_EARLY: &[FishCondition] = &[FishCondition::InLava, FishCondition::CanFishInLava, FishCondition::Crate, FishCondition::EarlyMode];
const CONDS_LAVA_ONLY: &[FishCondition] = &[FishCondition::InLava];
const CONDS_HONEY: &[FishCondition] = &[FishCondition::InHoney];
const CONDS_JUNK: &[FishCondition] = &[FishCondition::Junk];
const CONDS_ANY_ENEMIES: &[FishCondition] = &[FishCondition::AnyEnemies];
const CONDS_CRATE_DUNGEON_HARD: &[FishCondition] = &[FishCondition::Crate, FishCondition::Dungeon, FishCondition::HardMode];
const CONDS_CRATE_DUNGEON_EARLY: &[FishCondition] = &[FishCondition::Crate, FishCondition::Dungeon, FishCondition::EarlyMode];
const CONDS_CRATE_BEACH_HARD: &[FishCondition] = &[FishCondition::Crate, FishCondition::Beach, FishCondition::HardMode];
const CONDS_CRATE_BEACH_EARLY: &[FishCondition] = &[FishCondition::Crate, FishCondition::Beach, FishCondition::EarlyMode];
const CONDS_CRATE_CORRUPTION_HARD: &[FishCondition] = &[FishCondition::Crate, FishCondition::Corruption, FishCondition::HardMode];
const CONDS_CRATE_CORRUPTION_EARLY: &[FishCondition] = &[FishCondition::Crate, FishCondition::Corruption, FishCondition::EarlyMode];
const CONDS_CRATE_CRIMSON_HARD: &[FishCondition] = &[FishCondition::Crate, FishCondition::Crimson, FishCondition::HardMode];
const CONDS_CRATE_CRIMSON_EARLY: &[FishCondition] = &[FishCondition::Crate, FishCondition::Crimson, FishCondition::EarlyMode];
const CONDS_CRATE_HALLOW_HARD: &[FishCondition] = &[FishCondition::Crate, FishCondition::Hallow, FishCondition::HardMode];
const CONDS_CRATE_HALLOW_EARLY: &[FishCondition] = &[FishCondition::Crate, FishCondition::Hallow, FishCondition::EarlyMode];
const CONDS_CRATE_JUNGLE_HARD: &[FishCondition] = &[FishCondition::Crate, FishCondition::Jungle, FishCondition::HardMode];
const CONDS_CRATE_JUNGLE_EARLY: &[FishCondition] = &[FishCondition::Crate, FishCondition::Jungle, FishCondition::EarlyMode];
const CONDS_CRATE_SNOW_HARD: &[FishCondition] = &[FishCondition::Crate, FishCondition::Snow, FishCondition::HardMode];
const CONDS_CRATE_SNOW_EARLY: &[FishCondition] = &[FishCondition::Crate, FishCondition::Snow, FishCondition::EarlyMode];
const CONDS_CRATE_DESERT_HARD: &[FishCondition] = &[FishCondition::Crate, FishCondition::TrueDesert, FishCondition::HardMode];
const CONDS_CRATE_DESERT_EARLY: &[FishCondition] = &[FishCondition::Crate, FishCondition::TrueDesert, FishCondition::EarlyMode];
const CONDS_CRATE_H0_HARD: &[FishCondition] = &[FishCondition::Crate, FishCondition::Height0, FishCondition::HardMode];
const CONDS_CRATE_H0_EARLY: &[FishCondition] = &[FishCondition::Crate, FishCondition::Height0, FishCondition::EarlyMode];
const CONDS_CRATE_REMIX_H1_ROCK_HARD: &[FishCondition] = &[FishCondition::Crate, FishCondition::Remix, FishCondition::Height1, FishCondition::UnderRockLayer, FishCondition::HardMode];
const CONDS_CRATE_REMIX_H1_ROCK_EARLY: &[FishCondition] = &[FishCondition::Crate, FishCondition::Remix, FishCondition::Height1, FishCondition::UnderRockLayer, FishCondition::EarlyMode];
const CONDS_CRATE_HARD: &[FishCondition] = &[FishCondition::Crate, FishCondition::HardMode];
const CONDS_CRATE_EARLY: &[FishCondition] = &[FishCondition::Crate, FishCondition::EarlyMode];
const CONDS_CRATE: &[FishCondition] = &[FishCondition::Crate];
const CONDS_NONE: &[FishCondition] = &[];
const CONDS_BLOOD_NOBOOK: &[FishCondition] = &[FishCondition::BloodMoon, FishCondition::DidNotUseCombatBook];
const CONDS_BLOOD: &[FishCondition] = &[FishCondition::BloodMoon];
const CONDS_H3: &[FishCondition] = &[FishCondition::HeightAbove1];
const CONDS_H3_HARD: &[FishCondition] = &[FishCondition::HeightAbove1, FishCondition::HardMode];
const CONDS_H3_EARLY: &[FishCondition] = &[FishCondition::HeightAbove1, FishCondition::EarlyMode];
const CONDS_H1_AND_2: &[FishCondition] = &[FishCondition::Height1And2];
const CONDS_H1: &[FishCondition] = &[FishCondition::Height1];
const CONDS_H_ABOVE1: &[FishCondition] = &[FishCondition::HeightAbove1];
const CONDS_H_ABOVE2: &[FishCondition] = &[FishCondition::HeightAbove2];
const CONDS_WATER1000: &[FishCondition] = &[FishCondition::Water1000];
const CONDS_H_UNDER2: &[FishCondition] = &[FishCondition::HeightUnder2];
const CONDS_H0: &[FishCondition] = &[FishCondition::Height0];
const CONDS_OCEAN: &[FishCondition] = &[FishCondition::Ocean];
const CONDS_DESERT: &[FishCondition] = &[FishCondition::Desert];
const CONDS_JUNGLE_HARD: &[FishCondition] = &[FishCondition::Jungle, FishCondition::HardMode];
const CONDS_JUNGLE_H1: &[FishCondition] = &[FishCondition::Jungle, FishCondition::Height1];
const CONDS_JUNGLE: &[FishCondition] = &[FishCondition::Jungle];
const CONDS_JUNGLE_H_ABOVE1: &[FishCondition] = &[FishCondition::Jungle, FishCondition::HeightAbove1];
const CONDS_JUNGLE_H_ABOVE_AND1: &[FishCondition] = &[FishCondition::Jungle, FishCondition::HeightAboveAnd1];
const CONDS_SNOW: &[FishCondition] = &[FishCondition::Snow];
const CONDS_SNOW_H_ABOVE1: &[FishCondition] = &[FishCondition::Snow, FishCondition::HeightAbove1];
const CONDS_HALLOW: &[FishCondition] = &[FishCondition::Hallow];
const CONDS_HALLOW_HARD: &[FishCondition] = &[FishCondition::Hallow, FishCondition::HardMode];
const CONDS_HALLOW_HARD_H3_SNOW: &[FishCondition] = &[FishCondition::Hallow, FishCondition::HardMode, FishCondition::Height3, FishCondition::TrueSnow];
const CONDS_CRIMSON: &[FishCondition] = &[FishCondition::Crimson];
const CONDS_CRIMSON_HARD: &[FishCondition] = &[FishCondition::Crimson, FishCondition::HardMode];
const CONDS_CRIMSON_HARD_H3_SNOW: &[FishCondition] = &[FishCondition::Crimson, FishCondition::HardMode, FishCondition::Height3, FishCondition::TrueSnow];
const CONDS_CORRUPTION: &[FishCondition] = &[FishCondition::Corruption];
const CONDS_CORRUPTION_HARD: &[FishCondition] = &[FishCondition::Corruption, FishCondition::HardMode];
const CONDS_CORRUPTION_HARD_H3_SNOW: &[FishCondition] = &[FishCondition::Corruption, FishCondition::HardMode, FishCondition::Height3, FishCondition::TrueSnow];
const CONDS_DUNGEON: &[FishCondition] = &[FishCondition::Dungeon];
const CONDS_ROLLED_HALLOW_DESERT: &[FishCondition] = &[FishCondition::RolledHallowDesert];

/// The vanilla fish-drop table: `GameContentFishDropPopulator.cs`'s 142
/// invocations (158 rules, the 16 `AddWithHardmode` pairs counted twice) in
/// `Populate()`'s execution order — stoppers first, then Lava, Honey, Junk,
/// Crate, Rare, Remix, Dungeon, Corruption, Crimson, Hallow,
/// GlowingMushrooms, Snow, Jungle, Ocean, Desert, FloatingIsland and the
/// Surface fallbacks.
pub fn vanilla_drop_rules() -> FishDropRuleList {
    use FishRarity::*;
    let mut list = FishDropRuleList { rules: Vec::new() };
    // `Populate()` `:12` — the enemy roll stops the item pass.
    list.add(Any, 1, 1, &[], CONDS_ANY_ENEMIES);
    // LavaDrops `:239-247`.
    list.add(Any, 1, 6, &[4877], CONDS_LAVA_CRATE_EARLY);
    list.add(Any, 1, 6, &[4878], CONDS_LAVA_CRATE_HARD);
    list.add(Legendary, 1, 3, &[4819, 4820, 4872, 2331], CONDS_LAVA_HARD);
    list.add(Legendary, 1, 3, &[4819, 4820, 4872], CONDS_LAVA_EARLY);
    list.add(VeryRare, 1, 1, &[2312], CONDS_LAVA);
    list.add(Rare, 1, 1, &[2315], CONDS_LAVA);
    list.add(Any, 1, 1, &[], CONDS_LAVA_ONLY);
    // HoneyDrops `:231-236`.
    list.add(Rare, 1, 1, &[2314], CONDS_HONEY);
    list.add(Uncommon, 1, 2, &[2314], CONDS_HONEY);
    list.add(Uncommon, 1, 1, &[2451], &[FishCondition::QuestFish(2451), FishCondition::InHoney]);
    list.add(Any, 1, 1, &[], CONDS_HONEY);
    // JunkDrops `:224-228`.
    list.add(Any, 1, 8, &[5275], CONDS_JUNK);
    list.add(Any, 1, 1, &[2337, 2338, 2339], CONDS_JUNK);
    list.add(Any, 1, 1, &[], CONDS_JUNK);
    // CrateDrops `:204-221` (16 `AddWithHardmode` pairs).
    list.add(Rare, 1, 1, &[3205], CONDS_CRATE_DUNGEON_EARLY);
    list.add(Rare, 1, 1, &[3984], CONDS_CRATE_DUNGEON_HARD);
    list.add(Rare, 1, 1, &[5002], CONDS_CRATE_BEACH_EARLY);
    list.add(Rare, 1, 1, &[5003], CONDS_CRATE_BEACH_HARD);
    list.add(Rare, 1, 1, &[3203], CONDS_CRATE_CORRUPTION_EARLY);
    list.add(Rare, 1, 1, &[3982], CONDS_CRATE_CORRUPTION_HARD);
    list.add(Rare, 1, 1, &[3204], CONDS_CRATE_CRIMSON_EARLY);
    list.add(Rare, 1, 1, &[3983], CONDS_CRATE_CRIMSON_HARD);
    list.add(Rare, 1, 1, &[3207], CONDS_CRATE_HALLOW_EARLY);
    list.add(Rare, 1, 1, &[3986], CONDS_CRATE_HALLOW_HARD);
    list.add(Rare, 1, 1, &[3208], CONDS_CRATE_JUNGLE_EARLY);
    list.add(Rare, 1, 1, &[3987], CONDS_CRATE_JUNGLE_HARD);
    list.add(Rare, 1, 1, &[4405], CONDS_CRATE_SNOW_EARLY);
    list.add(Rare, 1, 1, &[4406], CONDS_CRATE_SNOW_HARD);
    list.add(Rare, 1, 1, &[4407], CONDS_CRATE_DESERT_EARLY);
    list.add(Rare, 1, 1, &[4408], CONDS_CRATE_DESERT_HARD);
    list.add(Rare, 1, 1, &[3206], CONDS_CRATE_H0_EARLY);
    list.add(Rare, 1, 1, &[3985], CONDS_CRATE_H0_HARD);
    list.add(Rare, 1, 1, &[5002], CONDS_CRATE_REMIX_H1_ROCK_EARLY);
    list.add(Rare, 1, 1, &[5003], CONDS_CRATE_REMIX_H1_ROCK_HARD);
    list.add(Legendary, 1, 1, &[2336], CONDS_CRATE_EARLY);
    list.add(Legendary, 1, 1, &[3981], CONDS_CRATE_HARD);
    list.add(VeryRare, 1, 1, &[2336], CONDS_CRATE_EARLY);
    list.add(VeryRare, 1, 1, &[3981], CONDS_CRATE_HARD);
    list.add(Rare, 1, 1, &[2335], CONDS_CRATE_EARLY);
    list.add(Rare, 1, 1, &[3980], CONDS_CRATE_HARD);
    list.add(Uncommon, 1, 1, &[2335], CONDS_CRATE_EARLY);
    list.add(Uncommon, 1, 1, &[3980], CONDS_CRATE_HARD);
    list.add(Any, 1, 1, &[2334], CONDS_CRATE_EARLY);
    list.add(Any, 1, 1, &[3979], CONDS_CRATE_HARD);
    list.add(Any, 1, 1, &[], CONDS_CRATE);
    // RareDrops `:194-201` (the unconditional tier rolls).
    list.add(Legendary, 1, 2, &[4382], CONDS_BLOOD_NOBOOK);
    list.add(Legendary, 1, 2, &[5240], CONDS_BLOOD);
    list.add(Legendary, 1, 5, &[2423], CONDS_NONE);
    list.add(Legendary, 1, 5, &[3225], CONDS_NONE);
    list.add(Legendary, 1, 10, &[2420], CONDS_NONE);
    list.add(BombNotLegendaryNotVeryRareAndUncommon, 1, 5, &[3196], CONDS_NONE);
    // RemixDrops `:32-43`.
    const REMIX_QUEST: [FishCondition; 1] = [FishCondition::QuestFishRemix(2461)];
    list.add(Uncommon, 1, 1, &[2461], &REMIX_QUEST);
    const REMIX_QUEST_2458: [FishCondition; 1] = [FishCondition::QuestFishRemix(2458)];
    list.add(Uncommon, 1, 1, &[2458], &REMIX_QUEST_2458);
    const REMIX_QUEST_2459: [FishCondition; 1] = [FishCondition::QuestFishRemix(2459)];
    list.add(Uncommon, 1, 1, &[2459], &REMIX_QUEST_2459);
    const REMIX_QUEST_2479: [FishCondition; 1] = [FishCondition::QuestFishRemix(2479)];
    list.add(Uncommon, 1, 1, &[2479], &REMIX_QUEST_2479);
    const REMIX_QUEST_2456: [FishCondition; 1] = [FishCondition::QuestFishRemix(2456)];
    list.add(Uncommon, 1, 1, &[2456], &REMIX_QUEST_2456);
    const REMIX_QUEST_2474: [FishCondition; 1] = [FishCondition::QuestFishRemix(2474)];
    list.add(Uncommon, 1, 1, &[2474], &REMIX_QUEST_2474);
    const REMIX_QUEST_2478: [FishCondition; 1] = [FishCondition::QuestFishRemix(2478)];
    list.add(Uncommon, 1, 1, &[2478], &REMIX_QUEST_2478);
    const REMIX_QUEST_2450: [FishCondition; 1] = [FishCondition::QuestFishRemix(2450)];
    list.add(Uncommon, 1, 1, &[2450], &REMIX_QUEST_2450);
    const REMIX_QUEST_2464: [FishCondition; 1] = [FishCondition::QuestFishRemix(2464)];
    list.add(Uncommon, 1, 1, &[2464], &REMIX_QUEST_2464);
    const REMIX_QUEST_2469: [FishCondition; 1] = [FishCondition::QuestFishRemix(2469)];
    list.add(Uncommon, 1, 1, &[2469], &REMIX_QUEST_2469);
    // DungeonDrops `:188-191`.
    list.add(VeryRare, 1, 12, &[3000], CONDS_DUNGEON);
    list.add(VeryRare, 1, 12, &[2999], CONDS_DUNGEON);
    // CorruptionDrops `:176-185`.
    list.add(Legendary, 2, 3, &[2429], CONDS_CORRUPTION_HARD_H3_SNOW);
    list.add(Legendary, 1, 2, &[3210], CONDS_CORRUPTION_HARD);
    list.add(Rare, 1, 1, &[2330], CONDS_CORRUPTION);
    list.add(Uncommon, 1, 1, &[2454], &[FishCondition::QuestFish(2454), FishCondition::Corruption]);
    list.add(Uncommon, 1, 1, &[2485], &[FishCondition::QuestFish(2485), FishCondition::Corruption]);
    list.add(Uncommon, 1, 1, &[2457], &[FishCondition::QuestFish(2457), FishCondition::Corruption]);
    list.add(Uncommon, 1, 1, &[2318], CONDS_CORRUPTION);
    // CrimsonDrops `:165-173`.
    list.add(Legendary, 2, 3, &[2429], CONDS_CRIMSON_HARD_H3_SNOW);
    list.add(Legendary, 1, 2, &[3211], CONDS_CRIMSON_HARD);
    list.add(Uncommon, 1, 1, &[2477], &[FishCondition::QuestFish(2477), FishCondition::Crimson]);
    list.add(Uncommon, 1, 1, &[2463], &[FishCondition::QuestFish(2463), FishCondition::Crimson]);
    list.add(Uncommon, 1, 1, &[2319], CONDS_CRIMSON);
    list.add(Common, 1, 1, &[2305], CONDS_CRIMSON);
    // HallowedDrops `:144-162`.
    list.add(Legendary, 1, 1, &[5490], CONDS_ROLLED_HALLOW_DESERT);
    list.add(Uncommon, 1, 1, &[4393], &[FishCondition::QuestFish(4393), FishCondition::RolledHallowDesert]);
    list.add(Uncommon, 1, 1, &[4394], &[FishCondition::QuestFish(4394), FishCondition::RolledHallowDesert]);
    list.add(Uncommon, 1, 1, &[4410], CONDS_ROLLED_HALLOW_DESERT);
    list.add(Any, 1, 3, &[4402], CONDS_ROLLED_HALLOW_DESERT);
    list.add(Any, 1, 1, &[4401], CONDS_ROLLED_HALLOW_DESERT);
    list.add(Legendary, 2, 3, &[2429], CONDS_HALLOW_HARD_H3_SNOW);
    list.add(Legendary, 1, 2, &[3209], CONDS_HALLOW_HARD);
    list.add(Legendary, 2, 3, &[5274], CONDS_HALLOW_HARD);
    list.add(VeryRare, 1, 1, &[2317], &[FishCondition::Hallow, FishCondition::HeightAbove1]);
    list.add(Uncommon, 1, 1, &[2465], &[FishCondition::QuestFish(2465), FishCondition::Hallow, FishCondition::HeightAbove1]);
    list.add(Uncommon, 1, 1, &[2468], &[FishCondition::QuestFish(2468), FishCondition::Hallow, FishCondition::HeightUnder2]);
    list.add(Rare, 1, 1, &[2310], &[FishCondition::Hallow]);
    list.add(Uncommon, 1, 1, &[2471], &[FishCondition::QuestFish(2471), FishCondition::Hallow]);
    list.add(Uncommon, 1, 1, &[2307], &[FishCondition::Hallow]);
    // GlowingMushroomsDrops `:139-141`.
    list.add(Uncommon, 1, 1, &[2475], &[FishCondition::QuestFish(2475), FishCondition::GlowingMushrooms]);
    // SnowDrops `:125-136`.
    list.add(Uncommon, 1, 1, &[2467], &[FishCondition::QuestFish(2467), FishCondition::Snow, FishCondition::HeightUnder2]);
    list.add(Uncommon, 1, 1, &[2470], &[FishCondition::QuestFish(2470), FishCondition::Snow, FishCondition::Height1]);
    list.add(Uncommon, 1, 1, &[2484], &[FishCondition::QuestFish(2484), FishCondition::Snow, FishCondition::HeightAbove1]);
    list.add(Uncommon, 1, 1, &[2466], &[FishCondition::QuestFish(2466), FishCondition::Snow, FishCondition::HeightAbove1]);
    list.add(Common, 1, 12, &[3197], &[FishCondition::Snow]);
    list.add(Uncommon, 1, 6, &[3197], &[FishCondition::Snow]);
    list.add(Uncommon, 1, 1, &[2306], &[FishCondition::Snow]);
    list.add(Common, 1, 1, &[2299], &[FishCondition::Snow]);
    list.add(Any, 1, 3, &[2309], &[FishCondition::Snow, FishCondition::HeightAbove1]);
    // JungleDrops `:111-122`.
    list.add(Legendary, 2, 3, &[5463], &[FishCondition::Jungle, FishCondition::HardMode]);
    list.add(Legendary, 2, 3, &[5634], &[FishCondition::Jungle]);
    list.add(Uncommon, 1, 1, &[2452], &[FishCondition::QuestFish(2452), FishCondition::Jungle, FishCondition::Height1]);
    list.add(Uncommon, 1, 1, &[2483], &[FishCondition::QuestFish(2483), FishCondition::Jungle, FishCondition::Height1]);
    list.add(Uncommon, 1, 1, &[2488], &[FishCondition::QuestFish(2488), FishCondition::Jungle, FishCondition::Height1]);
    list.add(Uncommon, 1, 1, &[2486], &[FishCondition::QuestFish(2486), FishCondition::Jungle, FishCondition::HeightAboveAnd1]);
    list.add(Uncommon, 1, 1, &[2311], &[FishCondition::Jungle, FishCondition::HeightAbove1]);
    list.add(Uncommon, 1, 1, &[2313], &[FishCondition::Jungle]);
    list.add(Common, 1, 1, &[2302], &[FishCondition::Jungle]);
    // OceanDrops `:95-108`.
    list.add(VeryRare, 1, 2, &[2341], &[FishCondition::Ocean]);
    list.add(VeryRare, 1, 1, &[2342], &[FishCondition::Ocean]);
    list.add(Rare, 1, 5, &[2438], &[FishCondition::Ocean]);
    list.add(Rare, 1, 3, &[2332], &[FishCondition::Ocean]);
    list.add(Uncommon, 1, 1, &[2480], &[FishCondition::QuestFish(2480), FishCondition::Ocean]);
    list.add(Uncommon, 1, 1, &[2481], &[FishCondition::QuestFish(2481), FishCondition::Ocean]);
    list.add(Uncommon, 1, 1, &[2316], &[FishCondition::Ocean]);
    list.add(Common, 1, 2, &[2301], &[FishCondition::Ocean]);
    list.add(Common, 1, 1, &[2300], &[FishCondition::Ocean]);
    list.add(Any, 1, 1, &[2297], &[FishCondition::Ocean]);
    list.add(Any, 1, 1, &[], &[FishCondition::Ocean]);
    // DesertDrops `:84-92`.
    list.add(Legendary, 1, 3, &[5490], &[FishCondition::Desert]);
    list.add(Uncommon, 1, 1, &[4393], &[FishCondition::QuestFish(4393), FishCondition::Desert]);
    list.add(Uncommon, 1, 1, &[4394], &[FishCondition::QuestFish(4394), FishCondition::Desert]);
    list.add(Uncommon, 1, 1, &[4410], &[FishCondition::Desert]);
    list.add(Any, 1, 3, &[4402], &[FishCondition::Desert]);
    list.add(Any, 1, 1, &[4401], &[FishCondition::Desert]);
    // FloatingIslandDrops `:73-81`.
    list.add(Uncommon, 1, 1, &[2461], &[FishCondition::QuestFish(2461), FishCondition::HeightUnder2]);
    list.add(Uncommon, 1, 1, &[2453], &[FishCondition::QuestFish(2453), FishCondition::Height0]);
    list.add(Uncommon, 1, 1, &[2473], &[FishCondition::QuestFish(2473), FishCondition::Height0]);
    list.add(Uncommon, 1, 1, &[2476], &[FishCondition::QuestFish(2476), FishCondition::Height0]);
    list.add(Uncommon, 1, 1, &[2458], &[FishCondition::QuestFish(2458), FishCondition::HeightUnder2]);
    list.add(Uncommon, 1, 1, &[2459], &[FishCondition::QuestFish(2459), FishCondition::HeightUnder2]);
    list.add(Uncommon, 1, 1, &[2304], &[FishCondition::Height0]);
    // SurfaceDrops `:46-70` (the quest fish first, then the plain rows).
    list.add(Uncommon, 1, 1, &[2455], &[FishCondition::QuestFish(2455), FishCondition::Height1And2]);
    list.add(Uncommon, 1, 1, &[2479], &[FishCondition::QuestFish(2479), FishCondition::Height1]);
    list.add(Uncommon, 1, 1, &[2456], &[FishCondition::QuestFish(2456), FishCondition::Height1]);
    list.add(Uncommon, 1, 1, &[2474], &[FishCondition::QuestFish(2474), FishCondition::Height1]);
    list.add(Rare, 1, 10, &[2437], &[FishCondition::HeightAbove1, FishCondition::HardMode]);
    list.add(Rare, 1, 9, &[2436], &[FishCondition::HeightAbove1, FishCondition::HardMode]);
    list.add(Rare, 1, 5, &[2436], &[FishCondition::HeightAbove1, FishCondition::EarlyMode]);
    list.add(Legendary, 2, 3, &[2308], &[FishCondition::HeightAbove1]);
    list.add(VeryRare, 1, 2, &[2320], &[FishCondition::HeightAbove1]);
    list.add(Rare, 1, 1, &[2321], &[FishCondition::HeightAbove1]);
    list.add(Uncommon, 1, 1, &[2478], &[FishCondition::QuestFish(2478), FishCondition::HeightAbove1]);
    list.add(Uncommon, 1, 1, &[2450], &[FishCondition::QuestFish(2450), FishCondition::HeightAbove1]);
    list.add(Uncommon, 1, 1, &[2464], &[FishCondition::QuestFish(2464), FishCondition::HeightAbove1]);
    list.add(Uncommon, 1, 1, &[2469], &[FishCondition::QuestFish(2469), FishCondition::HeightAbove1]);
    list.add(Uncommon, 1, 1, &[2462], &[FishCondition::QuestFish(2462), FishCondition::HeightAbove2]);
    list.add(Uncommon, 1, 1, &[2482], &[FishCondition::QuestFish(2482), FishCondition::HeightAbove2]);
    list.add(Uncommon, 1, 1, &[2472], &[FishCondition::QuestFish(2472), FishCondition::HeightAbove2]);
    list.add(Uncommon, 1, 1, &[2460], &[FishCondition::QuestFish(2460), FishCondition::HeightAbove2]);
    list.add(Uncommon, 3, 4, &[2303], &[FishCondition::HeightAbove1]);
    list.add(UncommonOrCommon, 1, 4, &[2303, 2309, 2309, 2309], &[FishCondition::HeightAbove1]);
    list.add(Uncommon, 1, 1, &[2487], &[FishCondition::QuestFish(2487)]);
    list.add(Common, 1, 1, &[2298], &[FishCondition::Water1000]);
    list.add(Any, 1, 1, &[2290], &[]);
    list
}

/// `Projectile.PrepareFishingContext` (`:20857-20908`): the zone reads,
/// the dungeon gate, the jungle's 50% nullification, the remix surface
/// rule, the evil XOR, the snow/jungle tie-break, the dungeon's desert
/// clear, the remix-ocean roll and the infected-desert roll. Mutates the
/// context's rolled lanes.
pub fn prepare_fishing_context(c: &mut FishingContext, rng: &mut FastRandom) {
    let mut corrupt = c.zone_corrupt;
    let mut crimson = c.zone_crimson;
    let mut jungle = c.zone_jungle;
    let mut snow = c.zone_snow;
    let mut dungeon = c.zone_dungeon;
    let mut desert = c.zone_desert;
    // `:20864-20867` — the dungeon needs Skeletron.
    if !c.downed_boss3 {
        dungeon = false;
    }
    // `:20868-20871` — the jungle's coin nullification.
    if c.not_the_bees_world && !c.remix_world && rng.next(2) == 0 {
        jungle = false;
    }
    // `:20872-20887` — the remix surface clears both evils; otherwise the
    // evil XOR keeps exactly one.
    if c.remix_world && c.height_level == 0 {
        corrupt = false;
        crimson = false;
    } else if corrupt && crimson {
        if rng.next(2) == 0 {
            crimson = false;
        } else {
            corrupt = false;
        }
    }
    // `:20888-20891` — the snow/jungle tie-break drops the snow half.
    if snow && jungle && rng.next(2) == 0 {
        snow = false;
    }
    // `:20892-20896` — the dungeon clears the desert.
    if dungeon {
        desert = false;
    }
    // `:20897` — the remix ocean's 1-in-3 roll under the rock layer.
    let remix_ocean = c.remix_world
        && c.height_level == 1
        && c.y >= c.rock_layer
        && rng.next(3) == 0;
    // `:20905` — the infected desert's coin.
    let infected_desert = desert && rng.next(2) == 0;
    c.rolled_corruption = corrupt;
    c.rolled_crimson = crimson;
    c.rolled_jungle = jungle;
    c.rolled_snow = snow;
    c.rolled_desert = desert;
    c.rolled_infected_desert = infected_desert;
    c.rolled_remix_ocean = remix_ocean;
}

/// `FishingCheck_RollEnemySpawns` (`Projectile.cs:20804-20835`): the
/// blood-moon spawn roll — the early-out (lava/honey/day), the `Next(6)`
/// gate (3 with the BloodRod), then the red-slime / hardmode / classic
/// tables.
pub fn roll_enemy_spawns(
    in_lava: bool,
    in_honey: bool,
    blood_moon: bool,
    day_time: bool,
    pole_item: u16,
    hard_mode: bool,
    unlocked_slime_red_spawn: bool,
    rng: &mut FastRandom,
) -> i32 {
    // `:20806` — the early-out.
    if in_lava || in_honey || !blood_moon || day_time {
        return 0;
    }
    let mut max_value = 6;
    if pole_item == 4325 {
        max_value = 3;
    }
    if rng.next(max_value) != 0 {
        return 0;
    }
    // `:20819-20833` — the spawn table.
    if !unlocked_slime_red_spawn && rng.next(5) == 0 {
        return 682;
    }
    if hard_mode {
        let pool = [620u16, 621, 586, 587];
        let pick = pool[rng.next(pool.len() as i32) as usize];
        if rng.next(10) == 0 {
            return 618;
        }
        return pick as i32;
    }
    let pool = [586u16, 587];
    pool[rng.next(pool.len() as i32) as usize] as i32
}

/// The catch's lava fallback (`Projectile.cs:20774-20797`): the lava-capable
/// count across `IsLavaBait`, `CanFishInLava` and `accLavaFishing` — `>= 3`
/// adds 480 ticks to `localAI[1]`, `>= 2` adds 240.
pub fn lava_fallback_ticks(
    is_lava_bait: bool,
    can_fish_in_lava: bool,
    acc_lava_fishing: bool,
) -> f32 {
    let mut count = 0;
    if is_lava_bait {
        count += 1;
    }
    if can_fish_in_lava {
        count += 1;
    }
    if acc_lava_fishing {
        count += 1;
    }
    if count >= 3 {
        480.0
    } else if count >= 2 {
        240.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod drop_rule_tests {
    use super::*;

    fn base_ctx() -> FishingContext {
        FishingContext {
            world_width: 4200,
            ..Default::default()
        }
    }

    #[test]
    fn the_vanilla_table_matches_the_populator_rule_count() {
        // `GameContentFishDropPopulator.cs`: 142 invocations -> 158 rules
        // (16 `AddWithHardmode` pairs), the AnyEnemies stopper first.
        let rules = vanilla_drop_rules();
        assert_eq!(rules.rules.len(), 158);
        assert!(rules.rules[0].is_stopper());
        assert_eq!(
            rules.rules[0].conditions,
            &[FishCondition::AnyEnemies],
            "the first rule is the enemy stopper"
        );
        // Six stoppers in all: AnyEnemies, InLava, InHoney, Junk, Crate, Ocean.
        assert_eq!(rules.rules.iter().filter(|r| r.is_stopper()).count(), 6);
    }

    #[test]
    fn the_any_enemies_stopper_ends_the_roll_with_no_item() {
        // `FishDropRuleList.cs:10-21` — the first stopper wins with item 0.
        let rules = vanilla_drop_rules();
        let mut ctx = base_ctx();
        ctx.rolled_enemy_spawn = 586;
        ctx.uncommon = true;
        let mut rng = FastRandom::new(1);
        assert_eq!(rules.try_get_item_drop(&ctx, &mut rng), Some(0));
    }

    #[test]
    fn quest_fish_only_roll_while_the_quest_matches() {
        let rules = vanilla_drop_rules();
        // The Bunnyfish (2479) at Height1 with the Uncommon tier rolled.
        // The unconditional RareDrops rows (`:194-201`) still answer on
        // some seeds — that is the vanilla behaviour (the quest rows are a
        // fallback, not a guarantee).
        let mut ctx = base_ctx();
        ctx.height_level = 1;
        ctx.uncommon = true;
        ctx.quest_fish = 2479;
        let mut hits = 0;
        for seed in 0..40u64 {
            let mut rng = FastRandom::new(seed.wrapping_mul(1_000_003).wrapping_add(11));
            if rules.try_get_item_drop(&ctx, &mut rng) == Some(2479) {
                hits += 1;
            }
        }
        assert!(hits > 0, "the bunnyfish row answers while it is the quest");
        // Without the quest the bunnyfish row never answers.
        ctx.quest_fish = -1;
        for seed in 0..40u64 {
            let mut rng = FastRandom::new(seed.wrapping_mul(1_000_003).wrapping_add(11));
            assert_ne!(rules.try_get_item_drop(&ctx, &mut rng), Some(2479));
        }
    }

    #[test]
    fn remix_quest_fish_need_the_remix_world() {
        // The Fallen Starfish (2458) has both a plain HeightUnder2 quest
        // row and a remix-only row; at Height3 only the remix row can
        // answer, which isolates the remix gate.
        let rules = vanilla_drop_rules();
        let mut ctx = base_ctx();
        ctx.height_level = 3;
        ctx.uncommon = true;
        ctx.quest_fish = 2458;
        for seed in 0..10u64 {
            let mut rng = FastRandom::new(seed + 3);
            assert_ne!(rules.try_get_item_drop(&ctx, &mut rng), Some(2458));
        }
        ctx.remix_world = true;
        // The remix rows have no conditions, but the unconditional
        // RareDrops rows (`:194-201`) still answer on some seeds.
        let mut remix_hits = 0;
        for seed in 0..40u64 {
            let mut rng = FastRandom::new(seed.wrapping_mul(1_000_003).wrapping_add(5));
            if rules.try_get_item_drop(&ctx, &mut rng) == Some(2458) {
                remix_hits += 1;
            }
        }
        assert!(remix_hits > 0, "the remix row answers in a remix world");
    }

    #[test]
    fn the_roll_is_deterministic_per_seed() {
        let rules = vanilla_drop_rules();
        let mut ctx = base_ctx();
        ctx.height_level = 1;
        ctx.common = true;
        let mut a = FastRandom::new(42);
        let mut b = FastRandom::new(42);
        for _ in 0..50 {
            assert_eq!(
                rules.try_get_item_drop(&ctx, &mut a),
                rules.try_get_item_drop(&ctx, &mut b),
                "the same seed must roll identically"
            );
        }
    }

    #[test]
    fn the_ocean_rows_need_the_edge_water_and_the_shallow_band() {
        // `IsOriginalOcean` (`AFishDropRulePopulator.cs:267-274`).
        let mut ctx = base_ctx();
        ctx.x = 100;
        ctx.height_level = 1;
        ctx.water_tiles_count = 1500;
        assert!(is_original_ocean(&ctx));
        // Too deep.
        ctx.height_level = 2;
        assert!(!is_original_ocean(&ctx));
        // Too far from the edge.
        ctx.height_level = 1;
        ctx.x = 2000;
        assert!(!is_original_ocean(&ctx));
        // Too little water.
        ctx.x = 100;
        ctx.water_tiles_count = 900;
        assert!(!is_original_ocean(&ctx));
    }

    #[test]
    fn the_evil_biomes_xor_to_exactly_one() {
        // `Projectile.cs:20877-20887`.
        let mut keep_both = 0;
        let mut keep_corrupt = 0;
        let mut keep_crimson = 0;
        for seed in 0..40u64 {
            let mut ctx = base_ctx();
            ctx.zone_corrupt = true;
            ctx.zone_crimson = true;
            let mut rng = FastRandom::new(seed * 1_000_003 + 7);
            prepare_fishing_context(&mut ctx, &mut rng);
            assert_ne!(ctx.rolled_corruption, ctx.rolled_crimson, "the XOR keeps one");
            if ctx.rolled_corruption {
                keep_corrupt += 1;
            }
            if ctx.rolled_crimson {
                keep_crimson += 1;
            }
            keep_both += (ctx.rolled_corruption && ctx.rolled_crimson) as i32;
        }
        assert_eq!(keep_both, 0, "both lanes can never survive");
        assert!(keep_corrupt > 0 && keep_crimson > 0, "both sides of the coin appear");
    }

    #[test]
    fn prepare_gates_the_dungeon_and_clears_the_desert() {
        // `:20864-20867` + `:20892-20896`.
        let mut ctx = base_ctx();
        ctx.zone_dungeon = true;
        ctx.zone_desert = true;
        ctx.downed_boss3 = false;
        let mut rng = FastRandom::new(11);
        prepare_fishing_context(&mut ctx, &mut rng);
        // Without Skeletron the dungeon condition fails and the desert stays.
        assert!(!FishCondition::Dungeon.matches(&ctx));
        assert!(ctx.rolled_desert);
        // With Skeletron the dungeon clears the desert.
        ctx.downed_boss3 = true;
        let mut rng = FastRandom::new(11);
        prepare_fishing_context(&mut ctx, &mut rng);
        assert!(FishCondition::Dungeon.matches(&ctx));
        assert!(!ctx.rolled_desert);
    }

    #[test]
    fn the_enemy_roll_respects_the_blood_moon_night_gate() {
        let mut rng = FastRandom::new(5);
        // Day, or no blood moon — nothing spawns.
        assert_eq!(
            roll_enemy_spawns(false, false, true, true, 0, false, true, &mut rng),
            0
        );
        assert_eq!(
            roll_enemy_spawns(false, false, false, false, 0, false, true, &mut rng),
            0
        );
        // Honey and lava also end it.
        assert_eq!(
            roll_enemy_spawns(false, true, true, false, 0, false, true, &mut rng),
            0
        );
    }

    #[test]
    fn the_lava_fallback_counts_the_three_sources() {
        // `Projectile.cs:20789-20796`.
        assert_eq!(lava_fallback_ticks(false, false, false), 0.0);
        assert_eq!(lava_fallback_ticks(true, false, false), 0.0);
        assert_eq!(lava_fallback_ticks(true, true, false), 240.0);
        assert_eq!(lava_fallback_ticks(true, true, true), 480.0);
    }

    #[test]
    fn the_rarity_tiers_read_the_rolled_flags() {
        let mut ctx = base_ctx();
        ctx.uncommon = true;
        assert!(FishRarity::Uncommon.matches(&ctx));
        assert!(FishRarity::UncommonOrCommon.matches(&ctx));
        assert!(FishRarity::BombNotLegendaryNotVeryRareAndUncommon.matches(&ctx));
        assert!(!FishRarity::Legendary.matches(&ctx));
        assert!(!FishRarity::Common.matches(&ctx));
        ctx.legendary = true;
        assert!(
            !FishRarity::BombNotLegendaryNotVeryRareAndUncommon.matches(&ctx),
            "the legendary flag disables the bomb tier"
        );
    }
}
