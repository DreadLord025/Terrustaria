//! The SceneState ambience emitters — the sound half of the ROADMAP §5
//! "Ambience emitters" row (`docs/ROADMAP.md:206-263`, M6-AUDIO-1).
//!
//! The vanilla emitters and their port status:
//!
//! * **Blizzard loops** (`SceneState.cs:189-253`) — the two tracked ambient
//!   loops (`blizzard_strong_loop`/`blizzard_inside_building_loop`, gated by
//!   `ZoneRain && ZoneSnow` with the ±0.01 volume ramp and the
//!   cloudAlpha/outside-weather split). Ported here as
//!   [`blizzard_loop_volumes`]; the engine half lives in
//!   `audio.rs::set_ambient_loop`.
//! * **Critter sounds** (`NPC.cs:92244-92340`, `UpdateNPC_CritterSounds`) —
//!   the one-shot bird/frog/ambient cues with the type gates and the
//!   per-type variation. The table lives here; the caller waits for the
//!   critter NPC kinds (the port's roster has none — see `npc.rs`), so the
//!   table runs with the call sites pinning its inputs.
//! * **Liquid-change sounds** (`WorldGen.cs:4632-4650`,
//!   `PlayLiquidChangeSound`) — the water/lava/honey merge cues fired from
//!   `Liquid.CreateLiquidMergeTile` (`Liquid.cs:1355-1357`). The cue table
//!   lives here; the merge pass is not ported (`tr-core/liquid.rs` has the
//!   settle but no merge tiles), so no caller yet.
//! * **Thunder** (`Main.cs:58741-58766`) — the strike position and the
//!   `FocusHelper.UpdateBackgroundThunder` / `thunderSkipSound` gates; the
//!   position math and the gating table live here, the cue plays from the
//!   weather tick.
//!
//! All of these are ambient cues: they follow `Main.ambientVolume`
//! (`LegacySoundPlayer.DoesSoundScaleWithAmbientVolume` `:1440-1460` and
//! `ActiveSound.DetermineIntendedVolume`'s `SoundType.Ambient` case
//! `ActiveSound.cs:155-165`) and are silent in the menu. Determinism: none of
//! the emitters touch the world RNG; the rolls they make (the critter rolls,
//! the thunder slot/pitch) come from the owning system's own stream.

/// `SoundID.BlizzardStrongLoop` (`SoundID.cs:1599`): the outside blizzard
/// loop, `WithVolume(0.5f)`.
pub const BLIZZARD_STRONG_LOOP: &str = "blizzard_strong_loop";
/// `SoundID.BlizzardInsideBuildingLoop` (`SoundID.cs:1598`).
pub const BLIZZARD_INSIDE_LOOP: &str = "blizzard_inside_building_loop";

/// `Utils.MoveTowards(float original, float target, float amount)`
/// (`Utils.cs:1782-1793`) — the signed step with overshoot snap.
///
/// Kept here (rather than calling `tr_core::scene`'s identical helper) so the
/// emitter math stays a pure function of its inputs, like the vanilla
/// `SceneState` computes it from `metrics` alone.
pub fn move_towards(original: f32, target: f32, amount: f32) -> f32 {
    if original == target {
        return target;
    }
    let sign = (target - original).signum();
    let stepped = original + amount * sign;
    if (target - stepped).signum() != sign {
        return target;
    }
    stepped
}

/// The blizzard pair's per-frame volumes (`SceneState.cs:210-228`), reduced to
/// what the emitter needs. Returns `(strong, inside, volume)` — the
/// `_blizzardSoundVolume` is threaded through `volume` so the caller can hold
/// it as state.
///
/// `blizzard` = `metrics.ZoneRain && metrics.ZoneSnow` (`SceneState.cs:80`);
/// `cloud_alpha` = `Main.cloudAlpha` (`:211`'s `num11`);
/// `outside_weather_intensity` = `SceneState.outsideWeatherEffectIntensity`
/// (`:175`, the behind-backwall fade). Vanilla always ramps the volume and
/// only spells the split out while it is above 0 (`:214-228`); the zeros are
/// returned the same way.
pub fn blizzard_loop_volumes(
    blizzard: bool,
    cloud_alpha: f32,
    outside_weather_intensity: f32,
    volume: f32,
) -> (f32, f32, f32) {
    // `:210`: the ramp toward `flag ? 1 : 0` at 0.01 per update.
    let volume = move_towards(volume, if blizzard { 1.0 } else { 0.0 }, 0.01);
    if volume <= 0.0 {
        // `:229-247`: both instances mute and drop their slot ids.
        return (0.0, 0.0, volume);
    }
    // `:211`: `min(1, cloudAlpha * 2) * outsideWeatherEffectIntensity` splits
    // the pair — more outside weather pushes the strong track up.
    let num11 = (cloud_alpha * 2.0).min(1.0) * outside_weather_intensity;
    (num11 * volume, (1.0 - num11) * volume, volume)
}

/// `TileChangeType` (the liquid merge kinds `WorldGen.GetLiquidChangeType`
/// can return).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileChangeType {
    /// water + lava.
    LavaWater,
    /// honey + water.
    HoneyWater,
    /// honey + lava.
    HoneyLava,
    /// shimmer + water.
    ShimmerWater,
    /// shimmer + lava.
    ShimmerLava,
    /// shimmer + honey.
    ShimmerHoney,
}

/// `WorldGen.GetLiquidChangeType`: the two liquid types (0 water, 1 lava,
/// 2 honey, 3 shimmer) → the merge kind.
pub fn get_liquid_change_type(liquid_type: i32, other_liquid_type: i32) -> Option<TileChangeType> {
    let kind = match (liquid_type, other_liquid_type) {
        (0, 1) | (1, 0) => TileChangeType::LavaWater,
        (0, 2) | (2, 0) => TileChangeType::HoneyWater,
        (1, 2) | (2, 1) => TileChangeType::HoneyLava,
        (0, 3) | (3, 0) => TileChangeType::ShimmerWater,
        (1, 3) | (3, 1) => TileChangeType::ShimmerLava,
        (2, 3) | (3, 2) => TileChangeType::ShimmerHoney,
        _ => return None,
    };
    Some(kind)
}

/// One `PlayLiquidChangeSound` row (`WorldGen.cs:4632-4650`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LiquidChangeCue {
    /// The asset name: the `Sounds/Custom/<cue>.xnb` trackable name for the
    /// `Liquids*` rows (`SoundID.cs:1600-1602`, the trackable path
    /// `LegacySoundPlayer.cs:257`) or `Splash_4`'s `Sounds/Splash_4.xnb`
    /// (`:233`) for the shimmer arms.
    pub cue: &'static str,
    /// The `LegacySoundPlayer` sound type: 42 = the trackable case
    /// (`LegacySoundPlayer.cs:1382`).
    pub sound_type: u32,
    /// `LegacySoundStyle.Variations` (`LegacySoundStyle.cs:14`): 1 for the
    /// shimmer arms, 3 for the `Liquids*` tracks (`SoundID.cs:1600-1602`).
    pub variations: u32,
    /// `PlaySound`'s `volumeScale` argument (`WorldGen.cs:4644`: the shimmer
    /// arms pass 4).
    pub volume_scale: f32,
}

/// `WorldGen.PlayLiquidChangeSound`'s merge table: the honey/lava pairs use
/// `SoundID.Liquids*` (three variations each, `SoundID.cs:1600-1602`'s
/// `CreateTrackable(name, 3, SoundType.Ambient)`), the three shimmer arms
/// share the legacy `PlaySound(19, x, y, 4)` (`WorldGen.cs:4644-4650`).
pub fn liquid_change_cue(event: TileChangeType) -> Option<LiquidChangeCue> {
    let cue = match event {
        TileChangeType::LavaWater => LiquidChangeCue {
            cue: "liquids_water_lava",
            sound_type: 42,
            variations: 3,
            volume_scale: 1.0,
        },
        TileChangeType::HoneyWater => LiquidChangeCue {
            cue: "liquids_honey_water",
            sound_type: 42,
            variations: 3,
            volume_scale: 1.0,
        },
        TileChangeType::HoneyLava => LiquidChangeCue {
            cue: "liquids_honey_lava",
            sound_type: 42,
            variations: 3,
            volume_scale: 1.0,
        },
        TileChangeType::ShimmerWater
        | TileChangeType::ShimmerLava
        | TileChangeType::ShimmerHoney => LiquidChangeCue {
            // `PlaySound(19, x, y, 4)` (`WorldGen.cs:4644-4650`) → case 19 →
            // `SoundSplash[4]` (`LegacySoundPlayer.cs:871`, the Style doubles
            // as the splash variant). The 4/5 variants cut the volume ×0.75
            // and jitter the pitch (`:878-884`) — the caller's seam.
            cue: "Splash_4",
            sound_type: 19,
            variations: 1,
            volume_scale: 4.0,
        },
    };
    Some(cue)
}

/// `Main.cs:58747-58766`'s thunder strike: the position, the `Next(2)` side
/// flip and the world-surface clamp, as a pure function of the frame's
/// inputs so the weather tick can call it inside its own RNG stream.
///
/// `center` = `Main.SceneMetrics.Center`, `thunder_distance` =
/// `Main.thunderDistance`, `flip_left` = the `Main.rand.Next(2) == 0` draw,
/// `world_surface_px` = `Main.worldSurface * 16` (the `num2` clamp line).
pub fn thunder_strike_position(
    center: (f32, f32),
    thunder_distance: f32,
    flip_left: bool,
    world_surface_px: f32,
) -> (f32, f32) {
    let mut x = center.0 + thunder_distance * 15.0 * if flip_left { -1.0 } else { 1.0 };
    let mut y = center.1;
    if y > world_surface_px - 500.0 {
        y = world_surface_px - 500.0;
    }
    // The strike's `x`/`y` are `int`s at the PlaySound call (`:58761`), so
    // the fractional part is truncated.
    (x.trunc(), y.trunc())
}

/// `FocusHelper.UpdateBackgroundThunder` (`FocusHelper.cs:73-79`) combined
/// with the `!Main.thunderSkipSound` half of the gate (`Main.cs:58759`):
/// whether the strike's cue plays. `allow_unfocused` carries
/// `Main.SettingPlayWhenUnfocused` for the unfocused window (BG-20's
/// deviation pins it); `skip_sound` = `Main.thunderSkipSound`
/// (`NewLightning(skipSound:)`, `:58727`, only set true by the unported
/// `ParticleOrchestrator.cs:1043`).
pub fn thunder_plays_when_focused(allow_unfocused: bool, skip_sound: bool) -> bool {
    !skip_sound && allow_unfocused
}

/// The critter sound table (`NPC.cs:92244-92340`,
/// `UpdateNPC_CritterSounds`) — the only emitter that plays ambience from NPC
/// AI. Returns the legacy sound id and its `volumeScale` (the `Style`
/// argument doubles as the per-type variation for type 32,
/// `LegacySoundPlayer.cs:1125-1185`).
///
/// Gates, in the C#'s if/else-if order (a matching arm consumes its roll even
/// when the inner condition fails — the RNG order is part of the contract):
/// 611/689 night `Next(700)` → 46; 602-603 day `Next(1200)` → 44; 362-365 day
/// `Next(200)` → 30; 361/445/687 slow + (night or below surface) `Next(200)`
/// → 31; the 74/297/298/442/671-675 day `time < 18000` `Next(400)` family →
/// 32/67/68/69; 300/447/610 `Next(1800)` → 33. `roll(hi)` stands in for
/// `Main.rand.Next(hi)`.
pub fn critter_sound_cue(
    npc_type: i32,
    day_time: bool,
    // `Main.time` (the 18000 gate, `:92276`).
    clock: f64,
    // `Main.worldSurface * 16` for the 361/445/687 underground half.
    world_surface_px: f32,
    y: f32,
    // `Math.Abs(base.velocity.X)` (`:92273`).
    speed_x: f32,
    roll: &mut dyn FnMut(u32) -> u32,
) -> Option<(u32, f32)> {
    if (npc_type == 611 || npc_type == 689) && !day_time && roll(700) == 0 {
        return Some((46, 1.0));
    }
    if (602..=603).contains(&npc_type) && day_time && roll(1200) == 0 {
        return Some((44, 1.0));
    }
    if (362..=365).contains(&npc_type) {
        if day_time && roll(200) == 0 {
            return Some((30, 1.0));
        }
    } else if (npc_type == 361 || npc_type == 445 || npc_type == 687)
        && speed_x < 0.5
        && (!day_time || y > world_surface_px)
        && roll(200) == 0
    {
        return Some((31, 1.0));
    } else if matches!(npc_type, 74 | 297 | 298 | 442 | 671..=675) {
        if day_time && clock < 18000.0 && roll(400) == 0 {
            return Some(match npc_type {
                74 => {
                    if roll(3) != 0 {
                        (32, 14.0)
                    } else {
                        (32, 18.0)
                    }
                }
                297 => (32, 16.0),
                298 => {
                    if roll(3) != 0 {
                        (32, 17.0)
                    } else {
                        (32, 19.0)
                    }
                }
                674 | 675 => (67, 1.0),
                671 | 672 => (68, 1.0),
                673 => (69, 1.0),
                442 => match roll(5) {
                    0 => (32, 16.0),
                    1 => (32, 17.0),
                    2 => (32, 19.0),
                    3 => (32, 14.0),
                    _ => (32, 18.0),
                },
                _ => unreachable!(),
            });
        }
    } else if (npc_type == 300 || npc_type == 447 || npc_type == 610) && roll(1800) == 0 {
        return Some((33, 1.0));
    }
    None
}

/// The per-thunder-strike cue family (`LegacySoundPlayer.cs:454-476`'s case
/// 43): the slot pick and the pitch jitter, resolved inside the port's
/// per-consumer stream like the vanilla's `Main.rand` draws. `slot_roll`
/// stands in for `Main.rand.Next(SoundThunder.Length)`, `pitch_roll` for
/// `Main.rand.Next(-10, 11)`.
pub fn thunder_cue_slot(slot_roll: u32, pitch_roll: i32) -> (&'static str, f32) {
    // `Sounds/Thunder_0..5` (`LegacySoundPlayer.cs:182-187`).
    const THUNDER: [&str; 6] = [
        "Thunder_0",
        "Thunder_1",
        "Thunder_2",
        "Thunder_3",
        "Thunder_4",
        "Thunder_5",
    ];
    (THUNDER[(slot_roll % 6) as usize], pitch_roll as f32 * 0.01)
}

/// `Main.cs:58741-58759`'s thunder strike tick (the `currentDayRateIteration
/// == 0` arm of `UpdateWeather`): decrements `thunder_delay` and, on the tick
/// it reaches 0, returns the cue position — or `None` when the focus/skip gate
/// (`Main.cs:58759`) blocks it. The flash math stays in `rain.rs`.
pub fn thunder_strike(
    thunder_delay: &mut i32,
    thunder_distance: i32,
    scene_center: (f32, f32),
    world_surface_px: f32,
    weather: &crate::rain::ThunderInputs,
    roll: &mut dyn FnMut(u32) -> u32,
) -> Option<(f32, f32)> {
    if *thunder_delay <= 0 {
        return None;
    }
    *thunder_delay -= 1;
    if *thunder_delay != 0 {
        return None;
    }
    let flip_left = roll(2) == 0;
    let (x, y) = thunder_strike_position(
        scene_center,
        thunder_distance as f32,
        flip_left,
        world_surface_px,
    );
    // The gate is `FocusHelper.UpdateBackgroundThunder && !thunderSkipSound`
    // (`Main.cs:58759`); the port's unfocused-window handling is pinned off
    // (BG-20's deviation), so the focus half reduces to the focused default.
    if !thunder_plays_when_focused(true, weather.thunder_skip_sound) {
        return None;
    }
    Some((x, y))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A deterministic stand-in for the per-consumer `Main.rand`: call-index
    /// based, so a gate can force an exact roll to 0.
    struct Rolls {
        calls: u32,
        zeros: Vec<u32>,
    }

    impl Rolls {
        fn new() -> Self {
            Self {
                calls: 0,
                zeros: Vec::new(),
            }
        }
        fn with_zero(at: u32) -> Self {
            Self {
                calls: 0,
                zeros: vec![at],
            }
        }
    }

    fn roll(state: &mut Rolls) -> impl FnMut(u32) -> u32 + '_ {
        move |hi: u32| {
            let call = state.calls;
            state.calls += 1;
            if state.zeros.contains(&call) {
                0
            } else {
                1 % hi.max(1)
            }
        }
    }

    #[test]
    fn move_towards_steps_by_the_amount_and_snaps_past_the_target() {
        assert_eq!(move_towards(0.0, 1.0, 0.01), 0.01);
        assert_eq!(move_towards(0.0, 1.0, 0.3), 0.3);
        assert_eq!(move_towards(0.0, 1.0, 2.0), 1.0);
        assert_eq!(move_towards(1.0, 0.0, 0.01), 0.99);
        assert_eq!(move_towards(1.0, 0.0, 2.0), 0.0);
        assert_eq!(move_towards(0.5, 0.5, 0.1), 0.5);
    }

    #[test]
    fn blizzard_volumes_ramp_and_split() {
        // 100 frames of ramp reach the target exactly.
        let mut volume = 0.0f32;
        for _ in 0..100 {
            let (_, _, v) = blizzard_loop_volumes(true, 0.5, 1.0, volume);
            volume = v;
        }
        assert!((volume - 1.0).abs() < 1e-6);
        // cloudAlpha*2 = 1.0 → the strong track owns the split at full ramp.
        let (strong, inside, v) = blizzard_loop_volumes(true, 0.5, 1.0, 1.0);
        assert_eq!((strong, inside, v), (1.0, 0.0, 1.0));
        // Half ramp, half clouds: the ramp runs first (`:210` then `:211`),
        // so 0.5 → 0.51 and the split 0.5 gives 0.255 strong / 0.245 inside.
        let (strong, inside, ramped) = blizzard_loop_volumes(true, 0.25, 1.0, 0.5);
        assert!((ramped - 0.51).abs() < 1e-6, "ramped {ramped}");
        assert!((strong - 0.51 * 0.5).abs() < 1e-6, "strong {strong}");
        assert!((inside - 0.51 * 0.5).abs() < 1e-6, "inside {inside}");
        // The outside-weather intensity scales the split (behind a backwall).
        let (strong, inside, _) = blizzard_loop_volumes(true, 0.5, 0.4, 1.0);
        assert!((strong - 0.4).abs() < 1e-6);
        assert!((inside - 0.6).abs() < 1e-6);
        // Off: the ramp walks down and the pair keeps playing through the
        // 100-frame fade (`:210-227`); the mute lands only at volume 0.
        let (strong, inside, v) = blizzard_loop_volumes(false, 0.5, 1.0, 1.0);
        assert!((v - 0.99).abs() < 1e-6, "ramped {v}");
        assert!((strong - 0.99).abs() < 1e-6 && inside == 0.0);
        let mut volume = 1.0f32;
        for _ in 0..100 {
            let (strong, inside, v) = blizzard_loop_volumes(false, 0.5, 1.0, volume);
            volume = v;
            if volume <= 0.0 {
                assert_eq!((strong, inside), (0.0, 0.0));
            }
        }
        // The ±0.01 steps land on 0 (up to float error), like the C# ramp.
        assert!(volume.abs() < 1e-3, "volume {volume}");
    }

    #[test]
    fn liquid_change_types_and_cues() {
        assert_eq!(get_liquid_change_type(0, 1), Some(TileChangeType::LavaWater));
        assert_eq!(get_liquid_change_type(1, 0), Some(TileChangeType::LavaWater));
        assert_eq!(get_liquid_change_type(0, 2), Some(TileChangeType::HoneyWater));
        assert_eq!(get_liquid_change_type(1, 2), Some(TileChangeType::HoneyLava));
        assert_eq!(get_liquid_change_type(0, 3), Some(TileChangeType::ShimmerWater));
        assert_eq!(get_liquid_change_type(3, 2), Some(TileChangeType::ShimmerHoney));
        assert_eq!(get_liquid_change_type(0, 0), None);
        let lava_water = liquid_change_cue(TileChangeType::LavaWater).unwrap();
        assert_eq!(lava_water.cue, "liquids_water_lava");
        assert_eq!(lava_water.sound_type, 42);
        assert_eq!(lava_water.variations, 3);
        let shimmer = liquid_change_cue(TileChangeType::ShimmerWater).unwrap();
        assert_eq!(shimmer.cue, "Splash_4");
        assert_eq!(shimmer.sound_type, 19);
        assert_eq!(shimmer.variations, 1);
        assert_eq!(shimmer.volume_scale, 4.0);
    }

    #[test]
    fn thunder_position_flips_and_clamps_y() {
        // thunderDistance 2 → ±30 px around the centre, y untouched high up.
        let (x, y) = thunder_strike_position((100.0, 100.0), 2.0, false, 5000.0);
        assert_eq!((x, y), (130.0, 100.0));
        let (x, y) = thunder_strike_position((100.0, 100.0), 2.0, true, 5000.0);
        assert_eq!((x, y), (70.0, 100.0));
        // Deep in the world the strike is pulled to worldSurface*16 - 500.
        let (x, y) = thunder_strike_position((100.0, 9999.0), 0.0, false, 5000.0);
        assert_eq!((x, y), (100.0, 4500.0));
    }

    #[test]
    fn thunder_skip_sound_blocks_the_cue() {
        assert!(thunder_plays_when_focused(true, false));
        assert!(!thunder_plays_when_focused(true, true));
        assert!(!thunder_plays_when_focused(false, false));
    }

    #[test]
    fn thunder_strike_tick_returns_the_position_once() {
        let weather = crate::rain::ThunderInputs {
            thunder_skip_sound: false,
            ..Default::default()
        };
        let mut delay = 2;
        let mut rolls = Rolls::new();
        // First tick: 2 → 1, no strike.
        assert_eq!(
            thunder_strike(&mut delay, 2, (0.0, 0.0), 5000.0, &weather, &mut roll(&mut rolls)),
            None
        );
        assert_eq!(delay, 1);
        // Second tick: 1 → 0, the strike fires.
        let hit = thunder_strike(&mut delay, 2, (0.0, 0.0), 5000.0, &weather, &mut roll(&mut rolls));
        assert!(hit.is_some());
        assert_eq!(delay, 0);
        // And a delay of 0 never strikes again.
        assert_eq!(
            thunder_strike(&mut delay, 2, (0.0, 0.0), 5000.0, &weather, &mut roll(&mut rolls)),
            None
        );
    }

    #[test]
    fn thunder_cue_slot_wraps_the_six_family() {
        for slot in 0..12u32 {
            let (cue, _) = thunder_cue_slot(slot, 0);
            assert_eq!(cue, format!("Thunder_{}", slot % 6));
        }
        let (_, pitch) = thunder_cue_slot(0, -7);
        assert!((pitch + 0.07).abs() < 1e-6);
    }

    #[test]
    fn critter_gates_fire_only_through_their_rolls() {
        // A firefly-family NPC (674) on a clear day rolls 400 → 67.
        let mut state = Rolls::with_zero(0);
        let gate = critter_sound_cue(674, true, 1000.0, 3000.0, 3000.0, 0.0, &mut roll(&mut state));
        assert_eq!(gate, Some((67, 1.0)));
        // The same NPC at night never fires (the day gate comes first).
        let mut state = Rolls::new();
        assert_eq!(
            critter_sound_cue(674, false, 1000.0, 3000.0, 3000.0, 0.0, &mut roll(&mut state)),
            None
        );
        // Night crawler family (611) fires 46 at night behind Next(700).
        let mut state = Rolls::with_zero(0);
        let gate = critter_sound_cue(611, false, 1000.0, 300.0, 300.0, 0.0, &mut roll(&mut state));
        assert_eq!(gate, Some((46, 1.0)));
        // An NPC outside every table is silent.
        let mut state = Rolls::new();
        assert_eq!(
            critter_sound_cue(4, true, 1000.0, 3000.0, 3000.0, 0.0, &mut roll(&mut state)),
            None
        );
    }

    #[test]
    fn critter_roll_is_consumed_even_when_the_roll_fails() {
        // A 362..365 bird BY DAY with a failing Next(200): the C#'s
        // `Main.dayTime && Main.rand.Next(200) == 0` still draws the roll
        // (`NPC.cs:92267`) and the branch ends the chain — the stream moves
        // by one. The port's closure counts the call.
        let mut calls = 0u32;
        let mut roll = |hi: u32| {
            calls += 1;
            1 % hi.max(1)
        };
        assert_eq!(
            critter_sound_cue(363, true, 1000.0, 3000.0, 3000.0, 0.0, &mut roll),
            None
        );
        assert_eq!(calls, 1);
        // At night the day gate short-circuits first — the C# draws nothing.
        let mut calls = 0u32;
        let mut roll = |hi: u32| {
            calls += 1;
            1 % hi.max(1)
        };
        assert_eq!(
            critter_sound_cue(363, false, 1000.0, 3000.0, 3000.0, 0.0, &mut roll),
            None
        );
        assert_eq!(calls, 0);
    }
}
