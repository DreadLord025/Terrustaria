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
