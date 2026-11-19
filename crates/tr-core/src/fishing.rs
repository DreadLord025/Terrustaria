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
//! Deviations (ANALYSIS "Fishing engine (M7-FISH-1)"): the chum buckets'
//! projectile side (`ChumBucketProjectileHelper`, the +11/+6/+3 inputs ride
//! the caller as a plain count) and `SpawnFakeFishInWater`'s sparkle/fake-fish
//! particle draws (`Projectile.cs:20477-20581` — the port skips the particle
//! layer AND its `Next(5)`/`Next(10)`/`NextFromList` draws, the standing
//! dust-skip convention) are recorded; the RNG rides the caller's stream
//! (`DropRng` — the app passes its `UpdateProjectiles`/`UpdatePlayers`
//! `UnifiedRandom`, the tests a `FastRandom`).
//!
//! S1-P-i35 census fixes (verbatim re-port against `Projectile.cs`):
//! - the height ladder was INVERTED below the surface (the old port mapped
//!   surface→3 / rock→2 and used a `(height-42)*0.99895` underworld edge);
//!   the C# vanilla ladder is `< surface*0.5 → 0`, `< surface → 1`,
//!   `< rockLayer → 2`, `< maxTilesY-300 → 3`, else 4 (`:20712-20731`), and
//!   the REMIX world swaps 2/3 with the 50 % cavern re-roll (`:20685-20711`).
//! - the luck rolls (`:20673-20683`) were missing — one `NextFloat` gate,
//!   a second `NextFloat` scale on a hit, BEFORE the height ladder.
//! - the bees-world honey nullification (`:20610-20613`, one `Next(2)`
//!   BEFORE the 75-tile gate) and the Truffle Worm duke arm (`:20620-20631`)
//!   were missing.
//! - the junk roll sits AFTER the height ladder in the C# (`:20732`), not
//!   before; the drop levels' rare/very-rare/legendary windows clamp at
//!   4/5/6 (`:21532-21543`), not only common/uncommon at 2/3.
//! - the full `SetFishingCheckResults` (`:20739-20802`), the quest-fish probe
//!   (`:21495-21510`), the bait consumption (`Player.cs:52449-52526`), the
//!   pull decision (`:52401-52447`) and the catch-stack rolls
//!   (`Projectile.cs:53907-53958`) are ported here (were missing).

use crate::loot::DropRng;
use crate::tile::{FLAG_LIQUID_HONEY, FLAG_LIQUID_LAVA};
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

/// `Player.GetFishingConditions` (`Player.cs:42176-42206`): the total power —
/// the bait + the pole + the skill + the situational buffs, times the
/// multiplier. The bait 2673 (the Truffle Worm, `:42181-42184`) and the
/// zero pole/bait (`:42185-42188`) return 0. `fishing_buff` is the C#'s
/// `FindBuffIndex(25) != -1` (`:42190` — Tipsy, the verbatim id; the fishing
/// POTION's +15 rides `fishing_skill` via `fishingSkill`, `:10148`).
pub fn final_fishing_level(
    pole_power: i32,
    bait_type: u16,
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
    // `:42181-42184`: the Truffle Worm returns BEFORE the zero-check — the
    // pole/bait types still ride the result struct, but the level is 0.
    if bait_type == 2673 {
        return 0;
    }
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
    // `GetFishingPondWidth` (`:21608-21620`): the liquid span on the row,
    // bounded by the solids and the 10-tile world margins. The C# tests the
    // CURRENT cell before stepping (so a dry bobber cell yields the empty
    // span — the old port probed the neighbour and slid into an adjacent
    // pond).
    let mut min_x = x;
    let mut max_x = x;
    while min_x > 10
        && world
            .tile(min_x, y)
            .is_some_and(|t| t.liquid > 0 && !solid(world, min_x, y))
    {
        min_x -= 1;
    }
    while max_x < world.width - 10
        && world
            .tile(max_x, y)
            .is_some_and(|t| t.liquid > 0 && !solid(world, max_x, y))
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
    /// `fisher.CanFishInLava` (`:20637`): the pole/bait/accessory OR.
    pub can_fish_in_lava: bool,
    /// `fisher.fishingLevel` after the chum/quality/luck adjustments.
    pub fishing_level: i32,
    /// `fisher.waterNeededToFish` after the atmosphere scaling.
    pub water_needed: i32,
    /// `fisher.waterQuality` after the quality flip (`1f - quality`).
    pub water_quality: f32,
    /// The junk roll inputs (`:20732`): both `Next(50) > level` rolls and
    /// the pond below the needed size.
    pub junk: bool,
    /// The depth ladder (`:20684-20731`): vanilla 0 = the space, 1 = the
    /// surface, 2 = the underground, 3 = the cavern, 4 = the underworld;
    /// the REMIX world swaps 2/3 and re-rolls the cavern to 1 at 50 %.
    pub height_level: i32,
    /// The rolled rarity tiers (`FishingCheck_RollDropLevels`, `:20733`).
    pub common: bool,
    pub uncommon: bool,
    pub rare: bool,
    pub very_rare: bool,
    pub legendary: bool,
    pub crate_hit: bool,
    /// The quest-fish probe's result (`FishingCheck_ProbeForQuestFish`,
    /// `:21495-21510`): the angler's current quest item, -1 when gated.
    pub quest_fish: i32,
}

/// The world/player inputs `TryBuildFishingContext` reads beyond the pond
/// (`Projectile.cs:20597-20737`). The zone flags ride [`FishingContext`]'s
/// player lanes; this struct carries the scalars the attempt builder needs.
#[derive(Clone, Copy, Debug, Default)]
pub struct AttemptInputs {
    /// `Main.player[owner].wet` (`:20599`) — the submerged-player gate.
    pub player_wet: bool,
    /// The bobber's `base.Center.Y` (`:20599`) — the submerged gate compares
    /// it against the mounted-center line.
    pub bobber_center_y: f32,
    /// `RotatedRelativePoint(MountedCenter).Y` (`:20599`) — the bobber must
    /// sit at or below this line while the player is wet.
    pub player_mounted_y: f32,
    /// `Main.notTheBeesWorld` (`:20610`, `:20868`).
    pub not_the_bees: bool,
    /// `Main.remixWorld` (`:20685`).
    pub remix: bool,
    /// `GetFishingConditions().FinalFishingLevel` (`:20632`).
    pub base_level: i32,
    /// `GetFishingConditions().BaitItemType` (`:20620`).
    pub bait_type: u16,
    /// `ItemID.Sets.CanFishInLava[PoleItemType] || IsLavaBait[BaitItemType]
    /// || accLavaFishing` (`:20637`).
    pub can_fish_in_lava: bool,
    /// `Main.player[owner].luck` (`:20673-20683`).
    pub luck: f32,
    /// `Main.player[owner].cratePotion` (`:21520`).
    pub crate_potion: bool,
    /// The chum count (`GetFishingPondState`'s out param — the port's caller
    /// feeds it; the chum projectile book is unported, so 0 in live play).
    pub chums_in_water: i32,
    /// `NPC.AnyNPCs(370)` (`:20623`) — a living Duke Fishron refuses the
    /// Truffle Worm's duke arm.
    pub duke_alive: bool,
    /// The quest probe's inputs (`:21495-21510`): the angler's current quest
    /// item, whether the player already holds it, whether an Angler (369)
    /// lives, and `Main.anglerQuestFinished`.
    pub quest_item: i32,
    pub player_has_quest_item: bool,
    pub angler_alive: bool,
    pub angler_quest_finished: bool,
}

/// Why `TryBuildFishingContext` refused (`Projectile.cs:20597-20737`) — the
/// caller maps each to its `displayedFishingInfo` text / duke-arm side effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttemptRefusal {
    /// `:20599-20602` — the player is wet and the bobber is above the
    /// mounted-center line.
    PlayerSubmerged,
    /// `:20614-20618` — fewer than 75 water tiles ("NotEnoughWater").
    NotEnoughWater,
    /// `:20620-20631` — the bait is the Truffle Worm (2673); the duke arm
    /// (`ai[1]`/`localAI` writes) fires only when `duke_arm` is true.
    TruffleWorm { duke_arm: bool },
    /// `:20633-20636` — the final fishing level is 0.
    ZeroLevel,
}

/// `FishingCheck`'s attempt builder (`TryBuildFishingContext`,
/// `:20597-20737`): the gates, the level adjustments and the rolls, in the
/// C#'s exact draw order. `position_y` is the bobber's `base.position.Y`
/// (the atmo formula reads it, `:20653` — NOT the tile row). The roll rng
/// rides the caller (the C# `Main.rand` under the `UpdateProjectiles` swap).
///
/// Draw order (the RNG contract): bees honey `Next(2)` (`:20610`, only in a
/// bees world, BEFORE the 75 gate) → [truffle duke `Next(-180,-60)`
/// (`:20625`)] → luck gate `NextFloat` + scale `NextFloat` (`:20675-20682`)
/// → remix cavern `Next(2)` (`:20707`) → junk `Next(50)` ×2 (`:20732`,
/// short-circuit) → drop levels ×6 (`:21550-21573`) → `PrepareFishingContext`
/// (`:20857-20908`).
pub fn try_build_fishing_attempt(
    world: &World,
    x: i32,
    y: i32,
    position_y: f32,
    inputs: &AttemptInputs,
    zones: &mut FishingContext,
    rng: &mut dyn DropRng,
) -> Result<FishingAttempt, AttemptRefusal> {
    // `:20599-20602`: the submerged-player gate.
    if inputs.player_wet && !(inputs.bobber_center_y >= inputs.player_mounted_y) {
        return Err(AttemptRefusal::PlayerSubmerged);
    }
    let pond = get_fishing_pond_state(world, x, y);
    let mut in_honey = pond.in_honey;
    // `:20610-20613`: the bees-world honey nullification — one coin BEFORE
    // the 75-tile gate.
    if inputs.not_the_bees && rng.next_range(0, 2) == 0 {
        in_honey = false;
    }
    // `:20614-20618`: the hard 75-tile floor.
    if pond.water_tiles < 75 {
        return Err(AttemptRefusal::NotEnoughWater);
    }
    // `:20620-20631`: the Truffle Worm — the duke arm needs the ocean edge,
    // >1000 water tiles and no living Duke Fishron.
    if inputs.bait_type == 2673 {
        let duke_arm = (x < 380 || x > world.width - 380)
            && pond.water_tiles > 1000
            && !inputs.duke_alive;
        return Err(AttemptRefusal::TruffleWorm { duke_arm });
    }
    let mut level = inputs.base_level;
    // `:20633-20636`: the zero-level gate.
    if level == 0 {
        return Err(AttemptRefusal::ZeroLevel);
    }
    // `:20638-20649`: the chum bonuses.
    if inputs.chums_in_water > 0 {
        level += 11;
    }
    if inputs.chums_in_water > 1 {
        level += 6;
    }
    if inputs.chums_in_water > 2 {
        level += 3;
    }
    // `:20651-20662`: `waterNeededToFish = 300 * atmo`, the atmosphere from
    // the height formula `num = maxTilesX / 4200`,
    // `atmo = (float)((double)(position.Y/16f - (60f + 10f*num²)) /
    // (worldSurface / 6.0))`, clamped 0.25..1. The inner subtraction is f32
    // (the C# casts it to double only for the division), so the port keeps
    // it f32 then widens — byte-identical to the C# expression.
    let num = world.width as f32 / 4200.0;
    let inner = position_y / 16.0 - (60.0 + 10.0 * (num * num));
    let mut atmo = (inner as f64 / (world.world_surface / 6.0)) as f32;
    if (atmo as f64) < 0.25 {
        atmo = 0.25;
    }
    if atmo > 1.0 {
        atmo = 1.0;
    }
    let water_needed = (300.0f32 * atmo) as i32;
    // `:20663-20668`: the quality scaling and the flip.
    let mut quality = pond.water_tiles as f32 / water_needed as f32;
    if quality < 1.0 {
        level = (level as f32 * quality) as i32;
    }
    let quality = 1.0 - quality;
    // `:20673-20683`: the luck rolls — one gate draw, a second scale draw
    // only on a hit. Negative luck scales 0.9 - NextFloat*0.3, positive
    // 1.1 + NextFloat*0.3. ZERO luck still draws the else-gate ONCE
    // (`NextFloat() < 0.0` is always false, but the draw happens); only the
    // scale draw is conditional.
    if inputs.luck < 0.0 {
        if rng.next_float() < 0.0 - inputs.luck {
            level = (level as f64 * (0.9 - rng.next_float() as f64 * 0.3)) as i32;
        }
    } else if rng.next_float() < inputs.luck {
        level = (level as f64 * (1.1 + rng.next_float() as f64 * 0.3)) as i32;
    }
    // The depth ladder (`:20684-20731`). The REMIX world swaps the
    // underground/cavern bands and re-rolls the cavern to the surface at
    // 50 % (`:20707-20710`); the vanilla ladder has NO re-roll.
    let height_level = if inputs.remix {
        let h = if (y as f64) < world.world_surface * 0.5 {
            0
        } else if (y as f64) < world.world_surface {
            1
        } else if (y as f64) < world.rock_layer {
            3
        } else if y < world.height - 300 {
            2
        } else {
            4
        };
        if h == 2 && rng.next_range(0, 2) == 0 { 1 } else { h }
    } else if (y as f64) < world.world_surface * 0.5 {
        0
    } else if (y as f64) < world.world_surface {
        1
    } else if (y as f64) < world.rock_layer {
        2
    } else if y < world.height - 300 {
        3
    } else {
        4
    };
    // `:20732`: the junk roll — both `Next(50) > level` (short-circuit: the
    // second draws only when the first passed) and the short pond.
    let junk = rng.next_range(0, 50) > level
        && rng.next_range(0, 50) > level
        && pond.water_tiles < water_needed;
    // `:20733`: the drop-level rolls.
    let drops = roll_drop_levels(level, inputs.crate_potion, rng);
    // `:20734`: the quest-fish probe.
    let quest_fish = probe_quest_fish(inputs);
    // `:20735`: the context preparation (the zone lanes + their rolls).
    zones.height_level = height_level;
    zones.y = y;
    zones.x = x;
    zones.water_tiles_count = pond.water_tiles;
    zones.junk = junk;
    zones.crate_hit = drops.crate_hit;
    zones.common = drops.common;
    zones.uncommon = drops.uncommon;
    zones.rare = drops.rare;
    zones.very_rare = drops.very_rare;
    zones.legendary = drops.legendary;
    zones.in_lava = pond.in_lava;
    zones.in_honey = in_honey;
    zones.can_fish_in_lava = inputs.can_fish_in_lava;
    zones.quest_fish = quest_fish;
    zones.rock_layer = world.rock_layer as i32;
    zones.world_width = world.width;
    prepare_fishing_context(zones, rng);
    Ok(FishingAttempt {
        x,
        y,
        water_tiles_count: pond.water_tiles,
        in_lava: pond.in_lava,
        in_honey,
        can_fish_in_lava: inputs.can_fish_in_lava,
        fishing_level: level,
        water_needed,
        water_quality: quality,
        junk,
        height_level,
        common: drops.common,
        uncommon: drops.uncommon,
        rare: drops.rare,
        very_rare: drops.very_rare,
        legendary: drops.legendary,
        crate_hit: drops.crate_hit,
        quest_fish,
    })
}

/// `FishingCheck_ProbeForQuestFish` (`Projectile.cs:21495-21510`): the
/// angler's current quest item, cleared to -1 when the player already holds
/// it, no Angler (369) lives, or the day's quest is finished.
pub fn probe_quest_fish(inputs: &AttemptInputs) -> i32 {
    let mut quest = inputs.quest_item;
    if inputs.player_has_quest_item {
        quest = -1;
    }
    if !inputs.angler_alive {
        quest = -1;
    }
    if inputs.angler_quest_finished {
        quest = -1;
    }
    quest
}

/// `FishingCheck`'s hit roll (`:20589-20591`): `(level + 75) / 2` of 100.
pub fn fishing_check_roll(level: i32, rng: &mut dyn DropRng) -> bool {
    let num = (level + 75) / 2;
    rng.next_range(0, 100) <= num
}

/// `AI_061`'s bite accumulator (`Projectile.cs:53827-53836`): one scan step;
/// returns the new timer value (the caller resets and fires `FishingCheck`
/// when it exceeds 660, `:53837-53841`).
pub fn accumulate_bite(current: f32, level: i32, rng: &mut dyn DropRng) -> f32 {
    // `Main.rand.Next(1, 3)` = `Next(2) + 1`.
    let mut t = current;
    if rng.next_range(0, 300) < level {
        t += rng.next_range(1, 3) as f32;
    }
    t += (level / 30) as f32;
    t += rng.next_range(1, 3) as f32;
    if rng.next_range(0, 60) == 0 {
        t += 60.0;
    }
    t
}

/// The drop-level rolls (`FishingCheck_RollDropLevels`, `:21512-21574`):
/// `150/level` (common), `300/level` (uncommon), `1050/level` (rare),
/// `2250/level` (very rare), `4500/level` (legendary), `10 [+15 crate
/// potion]` (crate), with the mins 2/3/4/5/6 (`:21524-21543`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DropLevels {
    pub common: bool,
    pub uncommon: bool,
    pub rare: bool,
    pub very_rare: bool,
    pub legendary: bool,
    pub crate_hit: bool,
}

pub fn roll_drop_levels(level: i32, crate_potion: bool, rng: &mut dyn DropRng) -> DropLevels {
    // `:21514-21518`: the five windows. The C# divides by the raw level
    // (a 0 level would throw there; the attempt builder gates it at
    // `:20633`), the port clamps the divisor at 1 for safety.
    let l = level.max(1);
    let mut nums = [150 / l, 150 * 2 / l, 150 * 7 / l, 150 * 15 / l, 150 * 30 / l];
    // `:21524-21543`: ALL five windows clamp (the old port clamped only
    // common/uncommon — rare/very-rare/legendary could roll `Next(0)`).
    if nums[0] < 2 {
        nums[0] = 2;
    }
    if nums[1] < 3 {
        nums[1] = 3;
    }
    if nums[2] < 4 {
        nums[2] = 4;
    }
    if nums[3] < 5 {
        nums[3] = 5;
    }
    if nums[4] < 6 {
        nums[4] = 6;
    }
    let mut crate_roll = 10;
    if crate_potion {
        crate_roll += 15;
    }
    DropLevels {
        common: rng.next_range(0, nums[0]) == 0,
        uncommon: rng.next_range(0, nums[1]) == 0,
        rare: rng.next_range(0, nums[2]) == 0,
        very_rare: rng.next_range(0, nums[3]) == 0,
        legendary: rng.next_range(0, nums[4]) == 0,
        // `:21570-21573`: `Next(100) < num6` — the percentage roll.
        crate_hit: rng.next_range(0, 100) < crate_roll,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{FLAG_ACTIVE, Tile};
    use crate::tile_light::FastRandom;

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

    /// A neutral attempt-input set: level 100, no bees/remix/luck, a plain
    /// bait, no quest. The bobber sits at the tile row's top edge.
    fn inputs(base_level: i32) -> AttemptInputs {
        AttemptInputs {
            base_level,
            bait_type: 2674, // the Journeyman Bait — a plain bait
            ..Default::default()
        }
    }

    fn zones() -> FishingContext {
        FishingContext::default()
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
        // `:42189-42204`: bait 50 + pole 30 + skill 20 + the Tipsy buff 5
        // = 105; the midday ×0.8 → 84.
        assert_eq!(
            final_fishing_level(30, 2675, 50, 20, true, false, false, false, 0.0, true, 27000.0, 2, false),
            84
        );
        // The zero bait or pole → 0.
        assert_eq!(final_fishing_level(0, 2675, 50, 20, false, false, false, false, 0.0, true, 0.0, 2, false), 0);
        assert_eq!(final_fishing_level(30, 2675, 0, 20, false, false, false, false, 0.0, true, 0.0, 2, false), 0);
        // The Truffle Worm (2673) returns 0 even with its 666 bait power
        // (`:42181-42184` — the early return precedes the zero-check).
        assert_eq!(
            final_fishing_level(30, 2673, 666, 20, true, false, false, false, 0.0, true, 27000.0, 2, false),
            0
        );
        // The floating + sitting stack +10: 30 + 50 + 10 = 90, ×0.8 → 72.
        assert_eq!(
            final_fishing_level(30, 2675, 50, 0, false, true, true, false, 0.0, true, 27000.0, 2, false),
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
        let mut z = zones();
        assert!(
            try_build_fishing_attempt(&w, 25, 60, 60.0 * 16.0, &inputs(50), &mut z, &mut rng)
                .is_err()
        );
        // Extend to 12×7 = 84 tiles: passes.
        for x in 30..32 {
            for y in 60..67 {
                put_water(&mut w, x, y, 255);
            }
        }
        let attempt =
            try_build_fishing_attempt(&w, 25, 60, 60.0 * 16.0, &inputs(50), &mut z, &mut rng)
                .unwrap();
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
        let mut z = zones();
        let mut ins = inputs(100);
        ins.chums_in_water = 3;
        let attempt =
            try_build_fishing_attempt(&w, 30, 40, 40.0 * 16.0, &ins, &mut z, &mut rng).unwrap();
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
        // so scale via a deep atmo: at row 80 the atmo = (80-60.8)/16.67 =
        // 1.15 → 1.0 → need 300 — the pond 100 → 0.33 → level ×0.33.
        let mut w2 = world();
        for x in 20..30 {
            for y in 80..90 {
                put_water(&mut w2, x, y, 255);
            }
        }
        let attempt2 =
            try_build_fishing_attempt(&w2, 25, 80, 80.0 * 16.0, &inputs(100), &mut z, &mut rng)
                .unwrap();
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

    // --- S1-P-i35 witnesses (WRITTEN NOT RUN — code-only mode) -----------

    /// A pond deep enough to pass every gate: 20×10 = 200 tiles at row 110
    /// (surface 100, rock 120 → the underground band, vanilla height 2).
    fn deep_pond(w: &mut World) {
        for x in 20..40 {
            for y in 110..120 {
                put_water(w, x, y, 255);
            }
        }
    }

    #[test]
    fn the_height_ladder_is_the_vanilla_ladder() {
        // `Projectile.cs:20712-20731`: < surface*0.5 → 0, < surface → 1,
        // < rockLayer → 2, < maxTilesY-300 → 3, else 4. The old port
        // inverted 2/3 and used the (height-42) underworld edge.
        let mut w = world();
        // world_surface 100, rock_layer 120, height 400 → the underworld
        // starts at row 100 (400-300).
        deep_pond(&mut w);
        let mut z = zones();
        let mut rng = FastRandom::new(1);
        // Row 40 → space band 0 (a pond anywhere; the scan reads the row).
        for x in 20..40 {
            for y in 40..50 {
                put_water(&mut w, x, y, 255);
            }
        }
        let a = try_build_fishing_attempt(&w, 30, 40, 640.0, &inputs(100), &mut z, &mut rng)
            .unwrap();
        assert_eq!(a.height_level, 0);
        let a = try_build_fishing_attempt(&w, 30, 110, 1760.0, &inputs(100), &mut z, &mut rng)
            .unwrap();
        assert_eq!(a.height_level, 2, "the underground band");
        // A cavern pond at row 95... row 95 is < surface → band 1; use a
        // taller world for the cavern/underworld split.
        let mut w2 = World::new("t".into(), 1200, 1200);
        w2.rock_layer = 400.0; // world_surface = 300
        for x in 20..40 {
            for y in 500..510 {
                put_water(&mut w2, x, y, 255);
            }
            for y in 950..960 {
                put_water(&mut w2, x, y, 255);
            }
        }
        let a = try_build_fishing_attempt(&w2, 30, 500, 8000.0, &inputs(100), &mut z, &mut rng)
            .unwrap();
        assert_eq!(a.height_level, 3, "the cavern band (< 1200-300)");
        let a = try_build_fishing_attempt(&w2, 30, 950, 15200.0, &inputs(100), &mut z, &mut rng)
            .unwrap();
        assert_eq!(a.height_level, 4, "the underworld band (>= 1200-300)");
    }

    #[test]
    fn the_remix_ladder_swaps_two_and_three() {
        // `:20685-20710`: the remix world's underground band is 3 and the
        // cavern 2, with the 50 % cavern re-roll to 1.
        let mut w = World::new("t".into(), 1200, 1200);
        w.rock_layer = 400.0;
        for x in 20..40 {
            for y in 350..360 {
                put_water(&mut w, x, y, 255);
            }
        }
        let mut ins = inputs(100);
        ins.remix = true;
        let mut z = zones();
        // Two seeds cover both sides of the Next(2) re-roll over repeats.
        let mut saw = [false; 5];
        for seed in 0..32 {
            let mut rng = FastRandom::new(seed);
            let a =
                try_build_fishing_attempt(&w, 30, 350, 5600.0, &ins, &mut z, &mut rng).unwrap();
            saw[a.height_level as usize] = true;
        }
        assert!(saw[3], "the remix underground band is 3");
        // Row 500 (>= rock 400, < 900) is the remix cavern 2, re-rolled to 1.
        for x in 20..40 {
            for y in 500..510 {
                put_water(&mut w, x, y, 255);
            }
        }
        let mut saw2 = [false; 5];
        for seed in 0..32 {
            let mut rng = FastRandom::new(seed);
            let a =
                try_build_fishing_attempt(&w, 30, 500, 8000.0, &ins, &mut z, &mut rng).unwrap();
            saw2[a.height_level as usize] = true;
        }
        assert!(saw2[1] && saw2[2], "the remix cavern re-rolls to 1");
        assert!(!saw2[3], "no vanilla band 2 leak in the remix cavern");
    }

    #[test]
    fn the_luck_rolls_scale_the_level() {
        // `:20673-20683`: one NextFloat gate, a second NextFloat scale on a
        // hit; ZERO luck still draws the else-gate once (the scale draw is
        // the conditional one). The deep pond (200 tiles, need 300 at row
        // 110) quality-scales 100 → 66 BEFORE the luck rolls.
        let mut w = world();
        deep_pond(&mut w);
        let mut z = zones();
        let mut ins = inputs(100);
        ins.luck = 1.0; // always hits the gate
        let mut rng = FastRandom::new(5);
        let a = try_build_fishing_attempt(&w, 30, 110, 1760.0, &ins, &mut z, &mut rng).unwrap();
        assert_eq!(a.water_needed, 300);
        assert!(
            (72..=92).contains(&a.fishing_level),
            "the positive luck scales 66 by 1.1..1.4 ({})",
            a.fishing_level
        );
        ins.luck = -1.0;
        let a = try_build_fishing_attempt(&w, 30, 110, 1760.0, &ins, &mut z, &mut rng).unwrap();
        assert!(
            (39..=59).contains(&a.fishing_level),
            "the negative luck scales 66 by 0.6..0.9 ({})",
            a.fishing_level
        );
        // Zero luck: the level is the raw quality-scaled value, untouched,
        // but the else-gate STILL draws once. Witness: a zero-luck run and a
        // positive-luck run that MISSES its gate (a tiny luck the first
        // NextFloat exceeds) consume the SAME draw count — both draw exactly
        // one gate float and no scale float — so their streams stay aligned.
        ins.luck = 0.0;
        let mut rng_zero = FastRandom::new(9);
        let a = try_build_fishing_attempt(&w, 30, 110, 1760.0, &ins, &mut z, &mut rng_zero)
            .unwrap();
        assert_eq!(a.fishing_level, 66);
        // luck = 1e-9: the gate NextFloat (>= 0) almost surely exceeds it, so
        // the scale draw is skipped — one gate draw, like zero luck.
        ins.luck = 1e-9;
        let mut rng_tiny = FastRandom::new(9);
        let a2 = try_build_fishing_attempt(&w, 30, 110, 1760.0, &ins, &mut z, &mut rng_tiny)
            .unwrap();
        assert_eq!(a2.fishing_level, 66, "the tiny-luck gate misses");
        assert_eq!(rng_zero.next_float(), rng_tiny.next_float());
    }

    #[test]
    fn the_bees_world_honey_coin_precedes_the_water_gate() {
        // `:20610-20613`: the Next(2) honey nullification draws BEFORE the
        // 75-tile floor — a refused short pond still advances the stream.
        let mut w = world();
        // 7×7 = 49 honey tiles → ×1.5 = 73: still under the floor.
        for x in 20..27 {
            for y in 110..117 {
                put_water(&mut w, x, y, 255);
                let i = w.index(x, y);
                w.tiles[i].flags |= FLAG_LIQUID_HONEY;
            }
        }
        let mut ins = inputs(100);
        ins.not_the_bees = true;
        let mut z = zones();
        let mut rng_a = FastRandom::new(2);
        let mut rng_b = FastRandom::new(2);
        let err =
            try_build_fishing_attempt(&w, 23, 110, 1760.0, &ins, &mut z, &mut rng_a).unwrap_err();
        assert_eq!(err, AttemptRefusal::NotEnoughWater);
        // The stream moved exactly one Next(2) draw.
        let _ = rng_b.next_range(0, 2);
        assert_eq!(rng_a.next_range(0, 2), rng_b.next_range(0, 2));
    }

    #[test]
    fn the_truffle_worm_refuses_with_the_duke_arm() {
        // `:20620-20631`: bait 2673 refuses; the duke arm needs the ocean
        // edge (<380 or >width-380), >1000 water tiles and no living duke.
        let mut w = World::new("t".into(), 1200, 400);
        w.rock_layer = 120.0;
        // A 60×20 = 1200-tile pond at the left ocean edge.
        for x in 20..80 {
            for y in 110..130 {
                put_water(&mut w, x, y, 255);
            }
        }
        let mut ins = inputs(100);
        ins.bait_type = 2673;
        let mut z = zones();
        let mut rng = FastRandom::new(1);
        let err = try_build_fishing_attempt(&w, 30, 110, 1760.0, &ins, &mut z, &mut rng)
            .unwrap_err();
        assert_eq!(err, AttemptRefusal::TruffleWorm { duke_arm: true });
        ins.duke_alive = true;
        let err = try_build_fishing_attempt(&w, 30, 110, 1760.0, &ins, &mut z, &mut rng)
            .unwrap_err();
        assert_eq!(err, AttemptRefusal::TruffleWorm { duke_arm: false });
        // Mid-map: no ocean edge (600 is neither <380 nor >820).
        ins.duke_alive = false;
        for x in 550..650 {
            for y in 110..130 {
                put_water(&mut w, x, y, 255);
            }
        }
        let err = try_build_fishing_attempt(&w, 600, 110, 1760.0, &ins, &mut z, &mut rng)
            .unwrap_err();
        assert_eq!(err, AttemptRefusal::TruffleWorm { duke_arm: false });
    }

    #[test]
    fn the_submerged_player_gate_refuses() {
        // `:20599-20602`: wet player + bobber above the mounted line.
        let mut w = world();
        deep_pond(&mut w);
        let mut ins = inputs(100);
        ins.player_wet = true;
        ins.player_mounted_y = 2000.0;
        ins.bobber_center_y = 1760.0;
        let mut z = zones();
        let mut rng = FastRandom::new(1);
        let err = try_build_fishing_attempt(&w, 30, 110, 1760.0, &ins, &mut z, &mut rng)
            .unwrap_err();
        assert_eq!(err, AttemptRefusal::PlayerSubmerged);
        ins.bobber_center_y = 2000.0;
        assert!(
            try_build_fishing_attempt(&w, 30, 110, 1760.0, &ins, &mut z, &mut rng).is_ok()
        );
    }

    #[test]
    fn the_drop_level_mins_clamp_all_five_windows() {
        // `:21524-21543`: at level 1000 every window clamps (2/3/4/5/6) —
        // the old port left rare/very-rare/legendary at Next(1)/Next(0).
        let mut rng = FastRandom::new(4);
        for _ in 0..100 {
            let l = roll_drop_levels(1000, false, &mut rng);
            // Windows 2/3/4/5/6: over 100 rolls the legendary hits ~17 —
            // the assertion is that NOTHING panics (a Next(0) would) and
            // the common rate is ~50 %.
            let _ = l;
        }
        let mut hits = 0;
        let mut rng = FastRandom::new(6);
        for _ in 0..1000 {
            if roll_drop_levels(1000, false, &mut rng).legendary {
                hits += 1;
            }
        }
        assert!(hits > 100 && hits < 250, "legendary 1/6 → ~167 ({hits})");
    }

    #[test]
    fn the_pond_width_stops_at_the_dry_cell() {
        // `:21612-21619`: the CURRENT cell is tested before stepping — a
        // dry bobber cell yields the empty span even beside a pond.
        let mut w = world();
        deep_pond(&mut w);
        // Column 45 is dry (the pond spans 20..40).
        let s = get_fishing_pond_state(&w, 45, 110);
        assert_eq!(s.water_tiles, 0, "the dry cell counts nothing");
        // Inside the pond the span walks to both banks.
        let s = get_fishing_pond_state(&w, 30, 110);
        assert_eq!(s.water_tiles, 200);
    }

    #[test]
    fn the_quest_probe_gates() {
        // `:21495-21510`.
        let mut ins = inputs(100);
        ins.quest_item = 2450;
        ins.angler_alive = true;
        assert_eq!(probe_quest_fish(&ins), 2450);
        ins.player_has_quest_item = true;
        assert_eq!(probe_quest_fish(&ins), -1);
        ins.player_has_quest_item = false;
        ins.angler_alive = false;
        assert_eq!(probe_quest_fish(&ins), -1);
        ins.angler_alive = true;
        ins.angler_quest_finished = true;
        assert_eq!(probe_quest_fish(&ins), -1);
    }

    #[test]
    fn the_pull_decision_draws_only_in_the_item_arm() {
        // `Player.cs:52401-52447`: the Next(7) draws only for a plain item
        // catch — the duke/enemy arms return before it. With the line
        // accessory the draw STILL fires (`:52438`'s && order), it just
        // never snaps.
        let mut rng_a = FastRandom::new(3);
        let mut rng_b = FastRandom::new(3);
        assert_eq!(
            pull_bobber_decision(2673, 100.0, false, &mut rng_a),
            PullOutcome::SpawnDuke
        );
        assert_eq!(
            pull_bobber_decision(2674, -618.0, false, &mut rng_a),
            PullOutcome::SpawnEnemy { npc: 618, y_offset: 64 }
        );
        // Neither drew: the streams still match a fresh one.
        assert_eq!(rng_a.next_range(0, 7), rng_b.next_range(0, 7));
        // The line accessory blocks the snap but NOT the draw: after 20
        // accessory pulls rng_a sits 20 Next(7) draws ahead of a fresh
        // stream, and every outcome is ReelIn.
        let mut rng_c = FastRandom::new(3);
        for _ in 0..20 {
            assert_eq!(
                pull_bobber_decision(2674, 100.0, true, &mut rng_a),
                PullOutcome::ReelIn
            );
            let _ = rng_c.next_range(0, 7);
        }
        assert_eq!(rng_a.next_range(0, 7), rng_c.next_range(0, 7));
    }

    #[test]
    fn the_bait_roll_gates() {
        // `Player.cs:52476-52524`.
        let mut rng = FastRandom::new(3);
        // The Truffle Worm always consumes (the NextFloat still draws).
        assert!(bait_consumed(2673, 666, false, 100.0, 1, &mut rng));
        // The gold worm's Next(20) overrides everything.
        let mut hits = 0;
        for _ in 0..2000 {
            if bait_consumed(2895, 50, false, 100.0, 1, &mut rng) {
                hits += 1;
            }
        }
        assert!(hits > 50 && hits < 160, "gold worm 1/20 → ~100 ({hits})");
        // A gray-rarity catch never consumes (bait 2674, rare -1).
        for _ in 0..50 {
            assert!(!bait_consumed(2674, 15, false, 100.0, -1, &mut rng));
        }
        // The -1 sentinel forces the consume.
        assert!(bait_consumed(2674, 15, false, -1.0, 0, &mut rng));
    }

    #[test]
    fn the_catch_stacks_roll_the_bomb_and_frost_windows() {
        // `Projectile.cs:53907-53958`.
        let mut rng = FastRandom::new(3);
        assert_eq!(catch_stack(2300, 100, &mut rng), 1, "plain fish stack 1");
        // The bomb fish at level 100: min = (5+3)/2 = 4, base = (10+6)/2 =
        // 8, four bonuses (Next(50)<100 and Next(100)<100 always +1,
        // Next(150)/Next(200) ~2/3+1/2) → 10..12, the final Next(4, num+1).
        for _ in 0..100 {
            let s = catch_stack(3196, 100, &mut rng);
            assert!((4..=13).contains(&s), "bomb stack {s}");
        }
        // The frost daggerfish at level 100: min = (25+15)/2 = 20, base =
        // (50+40)/2 = 45 +6 per bonus → up to 69, Next(20, num+1).
        for _ in 0..100 {
            let s = catch_stack(3197, 100, &mut rng);
            assert!((20..=70).contains(&s), "frost stack {s}");
        }
    }

    #[test]
    fn the_bite_kick_counts_down_and_resets() {
        // `Projectile.cs:53843-53860`.
        let mut rng = FastRandom::new(3);
        // A settled bobber gets a kick; the countdown advances 1..4.
        let (ai1, la1, kick, net) = bite_kick(-10.0, 2300.0, 0.0, false, &mut rng);
        assert!(kick.is_some());
        assert!((-9.0..=-6.0).contains(&ai1), "ai1 {ai1}");
        assert_eq!(la1, 2300.0);
        assert!(!net);
        // Crossing zero clamps and clears localAI[1].
        let (ai1, la1, _, net) = bite_kick(-1.0, 2300.0, 0.0, false, &mut rng);
        assert_eq!(ai1, 0.0);
        assert_eq!(la1, 0.0);
        assert!(net);
        // An airborne bobber gets no kick but still advances.
        let (_, _, kick, _) = bite_kick(-10.0, 0.0, 3.0, false, &mut rng);
        assert!(kick.is_none());
    }

    #[test]
    fn the_results_overwrite_the_item_arm_with_the_enemy_arm() {
        // `:20744-20773`: both arms draw Next(-240,-90); the enemy arm's
        // writes overwrite the item arm's (the verbatim quirk).
        let attempt = FishingAttempt {
            x: 30,
            y: 110,
            water_tiles_count: 200,
            in_lava: false,
            in_honey: false,
            can_fish_in_lava: false,
            fishing_level: 100,
            water_needed: 300,
            water_quality: 0.33,
            junk: false,
            height_level: 2,
            common: true,
            uncommon: false,
            rare: false,
            very_rare: false,
            legendary: false,
            crate_hit: false,
            quest_fish: -1,
        };
        let mut z = zones();
        z.height_level = 2;
        z.common = true;
        // A one-rule list that always drops item 2300.
        let rules = FishDropRuleList {
            rules: vec![FishRule {
                items: &[2300],
                num: 1,
                den: 1,
                rarity: FishRarity::Any,
                conditions: &[],
            }],
        };
        let mut rng = FastRandom::new(3);
        let r = set_fishing_check_results(
            &attempt, &z, &rules, 2674, 2289, false, true, false, false, false, false, &mut rng,
        );
        assert_eq!(r.rolled_item, 2300);
        assert_eq!(r.local_ai1, 2300.0);
        assert_eq!(r.local_ai2, 2674.0);
        assert!(r.net_update);
        assert!((-340.0..=-190.0).contains(&r.ai1), "ai1 {}", r.ai1);
        // The both-arms overwrite quirk with a scripted stream: the enemy
        // roll draws Next(6)==0 (hits), the slime gate Next(5)!=0 (skips),
        // the pre-hardmode pool Next(2) picks 587; the item rule draws
        // Next(1) then Next(1); then the item arm's Next(-240,-90) = -200
        // and the enemy arm's = -150 OVERWRITES it.
        struct Script(Vec<i32>, usize);
        impl DropRng for Script {
            fn next_range(&mut self, _min: i32, _max: i32) -> i32 {
                let v = self.0[self.1];
                self.1 += 1;
                v
            }
            fn next_float(&mut self) -> f32 {
                0.0
            }
        }
        let mut s = Script(vec![0, 1, 1, 0, 0, -200, -150], 0);
        let r = set_fishing_check_results(
            &attempt, &z, &rules, 2674, 2289, true, false, false, false, false, false, &mut s,
        );
        assert_eq!(r.rolled_enemy, 587);
        assert_eq!(r.rolled_item, 2300);
        assert_eq!(r.local_ai1, -587.0, "the enemy arm overwrites the item");
        assert_eq!(r.ai1, -250.0, "the SECOND delay draw wins (-150 - 100)");
        assert_eq!(s.1, 7, "exactly seven draws");
        // The lava fallback fires only when NEITHER arm landed.
        let mut attempt_lava = attempt;
        attempt_lava.in_lava = true;
        attempt_lava.can_fish_in_lava = true;
        let empty = FishDropRuleList { rules: vec![] };
        let mut s = Script(vec![1], 0); // Next(6)!=0 → no enemy
        let r = set_fishing_check_results(
            &attempt_lava, &z, &empty, 4849, 2422, true, false, false, false, true, true, &mut s,
        );
        assert_eq!(r.local_ai1, 480.0, "the 3-count lava fallback");
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
    pub fn attempt(&self, c: &FishingContext, rng: &mut dyn DropRng) -> Option<u16> {
        if !self.conditions.iter().all(|cond| cond.matches(c)) {
            return None;
        }
        if rng.next_range(0, self.den) >= self.num {
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
        let pick = self.items[rng.next_range(0, self.items.len() as i32) as usize];
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
    pub fn try_get_item_drop(&self, c: &FishingContext, rng: &mut dyn DropRng) -> Option<u16> {
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
pub fn prepare_fishing_context(c: &mut FishingContext, rng: &mut dyn DropRng) {
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
    if c.not_the_bees_world && !c.remix_world && rng.next_range(0, 2) == 0 {
        jungle = false;
    }
    // `:20872-20887` — the remix surface clears both evils; otherwise the
    // evil XOR keeps exactly one.
    if c.remix_world && c.height_level == 0 {
        corrupt = false;
        crimson = false;
    } else if corrupt && crimson {
        if rng.next_range(0, 2) == 0 {
            crimson = false;
        } else {
            corrupt = false;
        }
    }
    // `:20888-20891` — the snow/jungle tie-break drops the snow half.
    if snow && jungle && rng.next_range(0, 2) == 0 {
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
        && rng.next_range(0, 3) == 0;
    // `:20905` — the infected desert's coin.
    let infected_desert = desert && rng.next_range(0, 2) == 0;
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
    rng: &mut dyn DropRng,
) -> i32 {
    // `:20806` — the early-out.
    if in_lava || in_honey || !blood_moon || day_time {
        return 0;
    }
    let mut max_value = 6;
    if pole_item == 4325 {
        max_value = 3;
    }
    if rng.next_range(0, max_value) != 0 {
        return 0;
    }
    // `:20819-20833` — the spawn table.
    if !unlocked_slime_red_spawn && rng.next_range(0, 5) == 0 {
        return 682;
    }
    if hard_mode {
        let pool = [620u16, 621, 586, 587];
        let pick = pool[rng.next_range(0, pool.len() as i32) as usize];
        if rng.next_range(0, 10) == 0 {
            return 618;
        }
        return pick as i32;
    }
    let pool = [586u16, 587];
    pool[rng.next_range(0, pool.len() as i32) as usize] as i32
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

/// `SetFishingCheckResults`' outcome (`Projectile.cs:20739-20802`): the
/// bobber's post-bite state. The caller writes `ai[1]`, `localAI[1]`,
/// `localAI[2]` and `netUpdate` from it; the sonar popup/sound and the
/// lava-fishing achievement (19, `:20798-20801`) ride the caller.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FishingResults {
    /// The rolled item (`FishDropsDB.TryGetItemDropType`), 0 = none.
    pub rolled_item: u16,
    /// The rolled enemy (`FishingCheck_RollEnemySpawns`), 0 = none.
    pub rolled_enemy: i32,
    /// `ai[1]` — the bite countdown (`Next(-240,-90) - fishingLevel`).
    pub ai1: f32,
    /// `localAI[1]` — the item id, the negated enemy id, or the lava
    /// fallback's tick bonus.
    pub local_ai1: f32,
    /// `localAI[2]` — the bait type used.
    pub local_ai2: f32,
    /// A bite landed (item or enemy) — `netUpdate = true`.
    pub net_update: bool,
}

/// `SetFishingCheckResults` (`Projectile.cs:20739-20802`): the enemy roll,
/// the item roll, the bite-countdown writes and the lava fallback. Draw
/// order: enemy roll (`:20741`) → item roll (`:20742`) → the item arm's
/// `Next(-240,-90)` (`:20754`) → the enemy arm's `Next(-240,-90)` (`:20768`,
/// OVERWRITES the item arm when both landed — the verbatim quirk). The lava
/// fallback (`:20774-20797`) fires only when NEITHER landed.
pub fn set_fishing_check_results(
    attempt: &FishingAttempt,
    zones: &FishingContext,
    rules: &FishDropRuleList,
    bait_type: u16,
    pole_item: u16,
    blood_moon: bool,
    day_time: bool,
    hard_mode: bool,
    unlocked_slime_red_spawn: bool,
    is_lava_bait: bool,
    acc_lava_fishing: bool,
    rng: &mut dyn DropRng,
) -> FishingResults {
    // `:20741` — the enemy roll.
    let rolled_enemy = roll_enemy_spawns(
        attempt.in_lava,
        attempt.in_honey,
        blood_moon,
        day_time,
        pole_item,
        hard_mode,
        unlocked_slime_red_spawn,
        rng,
    );
    // `:20742` — the item roll (the AnyEnemies stopper reads the enemy).
    let mut ctx = zones.clone();
    ctx.rolled_enemy_spawn = rolled_enemy;
    let rolled_item = rules.try_get_item_drop(&ctx, rng).unwrap_or(0);
    let mut out = FishingResults {
        rolled_item,
        rolled_enemy,
        ..Default::default()
    };
    let mut flag = false;
    // `:20744-20759` — the item arm.
    if rolled_item > 0 {
        out.ai1 = rng.next_range(-240, -90) as f32 - attempt.fishing_level as f32;
        out.local_ai1 = rolled_item as f32;
        out.local_ai2 = bait_type as f32;
        out.net_update = true;
        flag = true;
    }
    // `:20760-20773` — the enemy arm (overwrites the item arm's writes).
    if rolled_enemy > 0 {
        out.ai1 = rng.next_range(-240, -90) as f32 - attempt.fishing_level as f32;
        out.local_ai1 = -rolled_enemy as f32;
        out.local_ai2 = bait_type as f32;
        out.net_update = true;
        flag = true;
    }
    // `:20774-20797` — the lava fallback (no bite, in lava).
    if !flag && attempt.in_lava {
        out.local_ai1 += lava_fallback_ticks(is_lava_bait, attempt.can_fish_in_lava, acc_lava_fishing);
    }
    out
}

/// `AI_061`'s negative-`ai[1]` bite kick (`Projectile.cs:53843-53860`): while
/// the countdown runs negative, a settled bobber (`velocity.Y == 0`, or
/// honey-wet with `|velocity.Y| <= 0.01`) gets a random kick and its wet
/// flags clear; then `ai[1] += Next(1,5)`, clamped to 0 with `localAI[1] = 0`
/// and `netUpdate` when it crosses. Returns `(ai1, local_ai1, kick, net)`.
pub fn bite_kick(
    ai1: f32,
    local_ai1: f32,
    velocity_y: f32,
    honey_wet: bool,
    rng: &mut dyn DropRng,
) -> (f32, f32, Option<(f32, f32)>, bool) {
    let mut kick = None;
    // `:53845-53852` — the kick (the wet-flag clears ride the caller).
    if velocity_y == 0.0 || (honey_wet && velocity_y.abs() <= 0.01) {
        let vy = rng.next_range(100, 500) as f32 * 0.015;
        let vx = rng.next_range(-100, 101) as f32 * 0.015;
        kick = Some((vx, vy));
    }
    // `:53853-53859` — the countdown advance.
    let mut ai1 = ai1 + rng.next_range(1, 5) as f32;
    let mut local_ai1 = local_ai1;
    let mut net = false;
    if ai1 >= 0.0 {
        ai1 = 0.0;
        local_ai1 = 0.0;
        net = true;
    }
    (ai1, local_ai1, kick, net)
}

/// `AI_061_FishingBobber_GetWaterLine` (`Projectile.cs:53863-53891`): the
/// liquid surface line at the bobber's centre column — the tile above, the
/// tile itself, then the tile below, each `Y*16 - liquid/16`; the fallback
/// is the bobber's bottom edge.
pub fn bobber_water_line(world: &World, x: i32, y: i32, position_y: f32, height: f32) -> f32 {
    let liquid = |tx: i32, ty: i32| world.tile(tx, ty).map(|t| t.liquid).unwrap_or(0);
    let above = liquid(x, y - 1);
    if above > 0 {
        return (y * 16) as f32 - (above / 16) as f32;
    }
    let here = liquid(x, y);
    if here > 0 {
        return ((y + 1) * 16) as f32 - (here / 16) as f32;
    }
    let below = liquid(x, y + 1);
    if below > 0 {
        return ((y + 2) * 16) as f32 - (below / 16) as f32;
    }
    position_y + height
}

/// `ItemCheck_CheckFishingBobber_PullBobber`'s decision
/// (`Player.cs:52401-52447`): what the reel-in does with a biting bobber.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PullOutcome {
    /// `:52403-52414` — the Truffle Worm: spawn Duke Fishron (370),
    /// `ai[0] = 2`.
    SpawnDuke,
    /// `:52415-52437` — the enemy: spawn it at the bobber (the Blood
    /// Nautilus 618 drops 64 px lower, `:52419-52422`); the red town slime
    /// 682 raises `unlockedSlimeRedSpawn` (`:52429-52432`). `ai[0] = 2`.
    SpawnEnemy { npc: i32, y_offset: i32 },
    /// `:52438-52441` — the 1-in-7 line snap (no High Test Fishing Line):
    /// `ai[0] = 2`, the catch is lost.
    LineSnap,
    /// `:52442-52445` — the catch rides home: `ai[1] = localAI[1]`.
    ReelIn,
}

/// The pull decision (`Player.cs:52401-52447`). The `Next(7)` draw fires
/// ONLY in the item branch (the duke/enemy arms return before it) — and it
/// draws BEFORE the `!accFishingLine` check (`:52438`'s `&&` order: the
/// accessory blocks the snap, not the draw).
pub fn pull_bobber_decision(
    bait_type_used: i32,
    local_ai1: f32,
    acc_fishing_line: bool,
    rng: &mut dyn DropRng,
) -> PullOutcome {
    if bait_type_used == 2673 {
        return PullOutcome::SpawnDuke;
    }
    if local_ai1 < 0.0 {
        let npc = (0.0 - local_ai1) as i32;
        let y_offset = if npc == 618 { 64 } else { 0 };
        return PullOutcome::SpawnEnemy { npc, y_offset };
    }
    if rng.next_range(0, 7) == 0 && !acc_fishing_line {
        return PullOutcome::LineSnap;
    }
    PullOutcome::ReelIn
}

/// `ItemCheck_CheckFishingBobber_ConsumeBait`'s consumption roll
/// (`Player.cs:52476-52524`): whether the bait stack decrements. Draw
/// order: the `NextFloat` conservation roll (`:52487`, ALWAYS drawn once a
/// bait slot was found) → the gold worm's `Next(20)` (`:52507`, only for
/// 2895). The `localAI[1] == -1` forced consume (`:52491-52494`), the
/// gray-rarity exemption (`:52495-52503`, `item.rare < 0`) and the Truffle
/// Worm's forced consume (`:52509-52512`) are pure gates. `bait_power` is
/// the bait item's `Item.bait`; `caught_rare` the caught item's `Item.rare`
/// (only read when `local_ai1 > 0`).
pub fn bait_consumed(
    bait_type: u16,
    bait_power: i32,
    acc_tackle_box: bool,
    local_ai1: f32,
    caught_rare: i32,
    rng: &mut dyn DropRng,
) -> bool {
    // `:52478-52486` — the conservation factor `1 + bait/6` (min 1), +1
    // with the Tackle Box.
    let mut num2 = 1.0 + bait_power as f32 / 6.0;
    if num2 < 1.0 {
        num2 = 1.0;
    }
    if acc_tackle_box {
        num2 += 1.0;
    }
    // `:52487-52490` — the roll (always drawn).
    let mut flag = rng.next_float() * num2 < 1.0;
    // `:52491-52494` — the -1 sentinel forces the consume.
    if local_ai1 == -1.0 {
        flag = true;
    }
    // `:52495-52503` — a gray-rarity catch never consumes.
    if local_ai1 > 0.0 && caught_rare < 0 {
        flag = false;
    }
    // `:52505-52508` — the gold worm's 1-in-20.
    if bait_type == 2895 {
        flag = rng.next_range(0, 20) == 0;
    }
    // `:52509-52512` — the Truffle Worm always consumes.
    if bait_type == 2673 {
        flag = true;
    }
    flag
}

/// `AI_061_FishingBobber_GiveItemToPlayer`'s stack roll
/// (`Projectile.cs:53907-53958`): the caught item's stack. The bomb fish
/// (3196) and the frost daggerfish (3197) roll a level-scaled stack — four
/// `Next(50/100/150/200) < level` bonus draws then `Next(min, max+1)`; every
/// other item stacks 1. Returns the stack (the caller spawns the item).
pub fn catch_stack(item_type: u16, level: i32, rng: &mut dyn DropRng) -> i32 {
    if item_type == 3196 {
        // `:53911-53933` — the bomb fish.
        let min_value = (level / 20 + 3) / 2;
        let mut num = (level / 10 + 6) / 2;
        if rng.next_range(0, 50) < level {
            num += 1;
        }
        if rng.next_range(0, 100) < level {
            num += 1;
        }
        if rng.next_range(0, 150) < level {
            num += 1;
        }
        if rng.next_range(0, 200) < level {
            num += 1;
        }
        return rng.next_range(min_value, num + 1);
    }
    if item_type == 3197 {
        // `:53934-53956` — the frost daggerfish (+6 per bonus).
        let min_value = (level / 4 + 15) / 2;
        let mut num = (level / 2 + 40) / 2;
        if rng.next_range(0, 50) < level {
            num += 6;
        }
        if rng.next_range(0, 100) < level {
            num += 6;
        }
        if rng.next_range(0, 150) < level {
            num += 6;
        }
        if rng.next_range(0, 200) < level {
            num += 6;
        }
        return rng.next_range(min_value, num + 1);
    }
    1
}

#[cfg(test)]
mod drop_rule_tests {
    use super::*;
    use crate::tile_light::FastRandom;

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
