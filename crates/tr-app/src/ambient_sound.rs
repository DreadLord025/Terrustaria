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
//!   per-type variation. The table lives here and the caller rides the NPC
//!   update loop (`world_sim/npcs.rs`, the `NPC.cs:90323` position — after
//!   the AI's draws, per NPC); the legacy zombie-family voice resolution
//!   ([`critter_legacy_voice`]) consumes the same stream in the C#'s order.
//! * **Liquid-change sounds** (`WorldGen.cs:4632-4650`,
//!   `PlayLiquidChangeSound`) — the water/lava/honey merge cues fired from
//!   `Liquid.CreateLiquidMergeTile` (`Liquid.cs:1345-1347`). The cue table
//!   lives here; the tr-core liquid engine queues its merge events
//!   (`LiquidSystem::drain_merge_events`) and the liquid tick plays them.
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
/// loop's tracked style — `CreateTrackable("blizzard_strong_loop",
/// SoundType.Ambient).WithVolume(0.5f)`, one variation, uncapped (S1-E-23).
pub fn blizzard_strong_loop_style() -> crate::tracked_sound::TrackedStyle {
    crate::tracked_sound::TrackedStyle {
        cue: "blizzard_strong_loop",
        variations: 1,
        volume: 0.5,
        pitch_variance: 0.0,
        sound_type: crate::tracked_sound::SoundType::Ambient,
        max_tracked_instances: 0,
    }
}

/// `SoundID.BlizzardInsideBuildingLoop` (`SoundID.cs:1598`): the
/// inside-building loop's tracked style (`CreateTrackable("blizzard_
/// inside_building_loop", SoundType.Ambient)` — style volume 1).
pub fn blizzard_inside_loop_style() -> crate::tracked_sound::TrackedStyle {
    crate::tracked_sound::TrackedStyle {
        cue: "blizzard_inside_building_loop",
        variations: 1,
        volume: 1.0,
        pitch_variance: 0.0,
        sound_type: crate::tracked_sound::SoundType::Ambient,
        max_tracked_instances: 0,
    }
}

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

/// The `WorldGen.TileChangeType` (`WorldGen.cs:4656-4684`'s enum) merge
/// kinds — tr-core's liquid engine owns it (its merge events report the
/// kind, `Liquid.cs:1345`), this module maps it to the cue table.
pub use tr_core::liquid::TileChangeType;

/// One `PlayLiquidChangeSound` row (`WorldGen.cs:4632-4650`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LiquidChangeCue {
    /// The asset name: the `Sounds/Custom/<cue>.xnb` trackable name for the
    /// `Liquids*` rows (`SoundID.cs:1600-1602`, the trackable path
    /// `LegacySoundPlayer.cs:257`) or `Splash_4`'s `Sounds/Splash_4.xnb`
    /// (`:233`) for the shimmer arms.
    pub cue: &'static str,
    /// The `LegacySoundPlayer` sound type: 42 = the trackable case
    /// (`LegacySoundPlayer.cs:1382`), 19 = the splash case.
    pub sound_type: u32,
    /// `LegacySoundStyle.Variations` (`LegacySoundStyle.cs:14`): 1 for the
    /// shimmer arm, 3 for the `Liquids*` tracks (`SoundID.cs:1600-1602`).
    pub variations: u32,
    /// The per-case `num2` multiplier the `LegacySoundPlayer` applies:
    /// 1.0 for the `Liquids*` tracks, 0.75 for the shimmer arm — case 19's
    /// `num2 *= 0.75f` for variants 4/5 (`LegacySoundPlayer.cs:880-884`;
    /// `WorldGen.cs:4644`'s trailing `4` is the Style argument, not a
    /// volume scale).
    pub volume_scale: f32,
    /// The shimmer arm's case-19 pitch roll bounds (`Next(-20, 1)`,
    /// `LegacySoundPlayer.cs:883`); the `Liquids*` rows carry none (the
    /// style path's `PitchVariance` is the engine's documented gap).
    pub pitch_roll: Option<(i32, i32)>,
}

/// `WorldGen.PlayLiquidChangeSound`'s merge table: the honey/lava pairs use
/// `SoundID.Liquids*` (three variations each, `SoundID.cs:1600-1602`'s
/// `CreateTrackable(name, 3, SoundType.Ambient)`), the three shimmer arms
/// share the legacy `PlaySound(19, x, y, Style 4)` (`WorldGen.cs:4644-4650`)
/// — case 19's `SoundSplash[4]`.
pub fn liquid_change_cue(event: TileChangeType) -> Option<LiquidChangeCue> {
    let cue = match event {
        TileChangeType::LavaWater => LiquidChangeCue {
            cue: "liquids_water_lava",
            sound_type: 42,
            variations: 3,
            volume_scale: 1.0,
            pitch_roll: None,
        },
        TileChangeType::HoneyWater => LiquidChangeCue {
            cue: "liquids_honey_water",
            sound_type: 42,
            variations: 3,
            volume_scale: 1.0,
            pitch_roll: None,
        },
        TileChangeType::HoneyLava => LiquidChangeCue {
            cue: "liquids_honey_lava",
            sound_type: 42,
            variations: 3,
            volume_scale: 1.0,
            pitch_roll: None,
        },
        TileChangeType::ShimmerWater
        | TileChangeType::ShimmerLava
        | TileChangeType::ShimmerHoney => LiquidChangeCue {
            // `PlaySound(19, x, y, 4)` (`WorldGen.cs:4644-4650`) → case 19 →
            // `SoundSplash[4]` (`LegacySoundPlayer.cs:871`, the Style doubles
            // as the splash variant). Variant 4 cuts the volume ×0.75 and
            // jitters the pitch `Next(-20, 1)` (`:880-884`); the playing
            // guard (`:872-874`) and the 4↔5 cross-guard (`:896-909`) are
            // the caller's seam.
            cue: "Splash_4",
            sound_type: 19,
            variations: 1,
            volume_scale: 0.75,
            pitch_roll: Some((-20, 1)),
        },
        TileChangeType::None => return None,
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
/// 43): the slot pick re-rolls while the picked instance is still playing —
/// one initial `Next(6)` draw plus up to six re-rolls inside the loop — then
/// the pitch jitter `Next(-10, 11) * 0.01f` (`:471`), all resolved inside
/// the port's per-consumer stream like the vanilla's `Main.rand` draws.
///
/// `roll` stands in for `Main.rand.Next(min, max)` (the max exclusive), and
/// `is_playing(slot)` for `SoundInstanceThunder[slot].State == Playing` —
/// the engine's live instance table.
pub fn thunder_cue_pick(
    roll: &mut dyn FnMut(i32, i32) -> i32,
    is_playing: &dyn Fn(u32) -> bool,
) -> (&'static str, u32, f32) {
    // `Sounds/Thunder_0..5` (`LegacySoundPlayer.cs:182-187`).
    const THUNDER: [&str; 6] = [
        "Thunder_0",
        "Thunder_1",
        "Thunder_2",
        "Thunder_3",
        "Thunder_4",
        "Thunder_5",
    ];
    let mut slot = roll(0, 6).max(0) as u32;
    // `:456-464`: the re-roll loop — one pass per family entry, so at most
    // six re-rolls before the pick gives up and steals the slot (`:466-467`'s
    // Stop() — the engine's sink replacement).
    for _ in 0..6 {
        if !is_playing(slot) {
            break;
        }
        slot = roll(0, 6).max(0) as u32;
    }
    (
        THUNDER[slot as usize % 6],
        slot % 6,
        roll(-10, 11) as f32 * 0.01,
    )
}

/// One zombie-family voice (`LegacySoundPlayer.cs`'s `SoundZombie` bank):
/// what `SoundEngine.PlaySound(legacy id, x, y, style)` resolves to for the
/// critter table's cues. `SoundZombie[k]` loads `Sounds/Zombie_<k>`
/// (`LegacySoundPlayer.cs:214-222`, the 131-entry bank `:112`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CritterVoice {
    /// The `SoundInstanceZombie` slot the case guards/overwrites
    /// (`LegacySoundPlayer.cs:112`'s instance array — the engine's per-slot
    /// playing table).
    pub slot: usize,
    /// The `Sounds/Zombie_<slot>.xnb` asset (the bank's name convention,
    /// `:221`).
    pub asset: &'static str,
    /// The per-case `num2` multiplier (`Volume = num2 * mult`).
    pub volume_mult: f32,
    /// The per-case `Main.rand.Next(lo, hi) * 0.01f` pitch, in XNA octaves.
    pub pitch: f32,
}

/// The legacy zombie-family cases behind the critter table's ids
/// (`LegacySoundPlayer.cs:1051-1076`'s 44/46 and `:1099-1213`'s
/// 30/31/32/67/68/69/33), each consuming its `Main.rand` draws in the C#'s
/// order — slot picks, the rare-swap arms and the pitch — inside the port's
/// per-NPC stream. `roll(min, max)` stands in for `Main.rand.Next(min, max)`
/// (max exclusive, min may be negative); `is_playing(slot)` stands in for
/// `SoundInstanceZombie[slot].State == Playing` (the guard returns null =
/// silent, `:1064-1069` and friends). `style` carries the critter table's
/// `Style` argument (case 32's slot — the per-type variation, `:1133`).
pub fn critter_legacy_voice(
    legacy_id: u32,
    style: u32,
    roll: &mut dyn FnMut(i32, i32) -> i32,
    is_playing: &dyn Fn(usize) -> bool,
) -> Option<CritterVoice> {
    let voice = |slot: usize, volume_mult: f32, pitch: i32| CritterVoice {
        slot,
        asset: zombie_asset(slot),
        volume_mult,
        pitch: pitch as f32 * 0.01,
    };
    match legacy_id {
        // `:1051-1062`: `Next(106, 109)` slot pick, no guard (the old
        // instance is overwritten and keeps playing), ×0.2, `Next(-70, 1)`.
        44 => {
            let slot = roll(106, 109).max(0) as usize;
            Some(voice(slot, 0.2, roll(-70, 1)))
        }
        // `:1063-1076`: the 110/111 guards, then `Next(110, 112)` and the
        // rare `Next(300) == 0` swap arm (`Next(3) == 0 → 114`, else
        // `Next(2) != 0 → 112` else 113); ×0.9, `Next(-10, 11)`.
        46 => {
            if is_playing(110) || is_playing(111) {
                return None;
            }
            let mut slot = roll(110, 112).max(0) as usize;
            if roll(0, 300) == 0 {
                slot = if roll(0, 3) == 0 {
                    114
                } else if roll(0, 2) != 0 {
                    112
                } else {
                    113
                };
            }
            Some(voice(slot, 0.9, roll(-10, 11)))
        }
        // `:1099-1124`: `Next(10, 12)`, the rare `Next(300) == 0` arm
        // re-targets 12 behind the playing guard; ×0.75, and the pitch range
        // splits on the re-target (`Next(-70, 1)` vs `Next(-40, 21)`).
        30 => {
            let mut slot = roll(10, 12).max(0) as usize;
            if roll(0, 300) == 0 {
                if is_playing(12) {
                    return None;
                }
                slot = 12;
            }
            let pitch = if slot != 12 { roll(-70, 1) } else { roll(-40, 21) };
            Some(voice(slot, 0.75, pitch))
        }
        // `:1125-1132`: the owl's slot 13, no guard, ×0.35, `Next(-40, 21)`.
        31 => Some(voice(13, 0.35, roll(-40, 21))),
        // `:1133-1143`: the Style argument is the slot (the per-type
        // variation), the playing guard blocks a busy slot; ×0.15,
        // `Next(-70, 26)`.
        32 => {
            let slot = style as usize;
            if is_playing(slot) {
                return None;
            }
            Some(voice(slot, 0.15, roll(-70, 26)))
        }
        // `:1144-1155`: `Next(118, 121)` behind the guard; ×0.3,
        // `Next(-5, 6)`.
        67 => {
            let slot = roll(118, 121).max(0) as usize;
            if is_playing(slot) {
                return None;
            }
            Some(voice(slot, 0.3, roll(-5, 6)))
        }
        // `:1156-1167`: `Next(126, 129)` behind the guard; ×0.22,
        // `Next(-5, 6)`.
        68 => {
            let slot = roll(126, 129).max(0) as usize;
            if is_playing(slot) {
                return None;
            }
            Some(voice(slot, 0.22, roll(-5, 6)))
        }
        // `:1168-1179`: `Next(129, 131)` behind the guard; ×0.2,
        // `Next(-5, 6)`.
        69 => {
            let slot = roll(129, 131).max(0) as usize;
            if is_playing(slot) {
                return None;
            }
            Some(voice(slot, 0.2, roll(-5, 6)))
        }
        // `:1200-1213`: the slot-15 frog arm behind the guard; ×0.2,
        // `Next(-10, 31)`.
        33 => {
            if is_playing(15) {
                return None;
            }
            Some(voice(15, 0.2, roll(-10, 31)))
        }
        // S1-E-21e — case 45 (`:1059-1070`): the pinned slot 109 quack.
        // W4-c10 sweep census fix: the call site `NPC.cs:23678-23681`
        // (`ai[3] == 60 && Next(2) == 0`) sits inside the aiStyle-16
        // arm's `this.type == 615` block (`:23572`) — the DOLPHIN
        // (`NPCID.cs:11907`), NOT a duck: the waterfowl quack is case 30
        // (`UpdateNPC_CritterSounds` `:92257-92262`, the 362-365 row).
        // The Dolphin is unported; the arm stays table-complete. The
        // playing guard BEFORE the pitch draw, ×0.3, `Next(-10, 11)`.
        45 => {
            if is_playing(109) {
                return None;
            }
            Some(voice(109, 0.3, roll(-10, 11)))
        }
        // S1-E-21e — the fighter-moan cases (`AI_003_Fighters`,
        // `NPC.cs:57340-57367`). Case 14 (`:793-824`): NO pitch draw and no
        // playing guard — the style-542 slot 7, the 489/586 `Next(21, 24)`
        // pair and the default `Next(3)`, all ×0.4, the array entry
        // overwritten without a Stop (the old voice plays out).
        14 => {
            let slot = match style {
                542 => 7,
                489 | 586 => roll(21, 24).max(0) as usize,
                _ => roll(0, 3).max(0) as usize,
            };
            Some(CritterVoice {
                slot,
                asset: zombie_asset(slot),
                volume_mult: 0.4,
                pitch: 0.0,
            })
        }
        // Case 26 (`:969-978`): the mummy's `Next(3, 5)` slot pick, ×0.9,
        // one `Next(-10, 11)` pitch draw; no guard, overwrite without Stop.
        26 => {
            let slot = roll(3, 5).max(0) as usize;
            Some(voice(slot, 0.9, roll(-10, 11)))
        }
        // Case 29 (`:1008-1047`): the Style argument IS the slot; the
        // per-style volume arms, the playing guard returns null BEFORE the
        // pitch draw (`:1038-1041`), one `Next(-10, 11)` pitch (`:1045`).
        // The moan arms only name styles 6/7/8 (×1.0); the full switch is
        // ported for the later waves' callers.
        29 => {
            let mut mult = 1.0f32;
            let s = style as i32;
            if (24..=87).contains(&s) {
                mult *= 0.5;
            }
            if (88..=91).contains(&s) {
                mult *= 0.7;
            }
            if (93..=99).contains(&s) {
                mult *= 0.4;
            }
            if s == 92 {
                mult *= 0.5;
            }
            if s == 103 {
                mult *= 0.4;
            }
            if s == 104 {
                mult *= 0.55;
            }
            if s == 100 || s == 101 {
                mult *= 0.25;
            }
            if s == 102 {
                mult *= 0.4;
            }
            let slot = style as usize;
            if is_playing(slot) {
                return None;
            }
            Some(voice(slot, mult, roll(-10, 11)))
        }
        _ => None,
    }
}

/// The zombie bank's asset name (`LegacySoundPlayer.cs:221`'s
/// `Sounds/Zombie_ + k`, the 131-entry bank `:112`) — the slots the critter
/// table's cases can name.
fn zombie_asset(slot: usize) -> &'static str {
    match slot {
        10 => "Zombie_10",
        11 => "Zombie_11",
        12 => "Zombie_12",
        13 => "Zombie_13",
        14 => "Zombie_14",
        15 => "Zombie_15",
        16 => "Zombie_16",
        17 => "Zombie_17",
        18 => "Zombie_18",
        19 => "Zombie_19",
        106 => "Zombie_106",
        107 => "Zombie_107",
        108 => "Zombie_108",
        112 => "Zombie_112",
        113 => "Zombie_113",
        114 => "Zombie_114",
        118 => "Zombie_118",
        119 => "Zombie_119",
        120 => "Zombie_120",
        126 => "Zombie_126",
        127 => "Zombie_127",
        128 => "Zombie_128",
        129 => "Zombie_129",
        130 => "Zombie_130",
        // S1-E-21e — the fighter-moan slots: case 14's `Next(3)` default
        // (`LegacySoundPlayer.cs:815-821`), case 26's `Next(3, 5)` (`:971`),
        // case 14's style-542 slot 7 (`:796`) and 489/586's `Next(21, 24)`
        // (`:807`), case 29's Style-is-the-slot rows 6/7/8 (`:1038-1046`)
        // plus the full switch's slot 5 row, case 45's pinned slot 109
        // (`:1060`).
        0 => "Zombie_0",
        1 => "Zombie_1",
        2 => "Zombie_2",
        3 => "Zombie_3",
        4 => "Zombie_4",
        5 => "Zombie_5",
        6 => "Zombie_6",
        7 => "Zombie_7",
        8 => "Zombie_8",
        21 => "Zombie_21",
        22 => "Zombie_22",
        23 => "Zombie_23",
        109 => "Zombie_109",
        _ => "Zombie_0",
    }
}

/// S1-E-21e — the aiStyle-3 kinds whose port machine is the plain
/// `AI_003_Fighters` walker (`update_zombie`/`update_skeleton`) and whose
/// `ai[3]` slot is therefore free for the C#'s idle counter. The other
/// aiStyle-3 machines (the martian shooters, the scutlix rider, the vortex
/// rifleman, …) have claimed `ai[3]` for their own state in the port; their
/// C# idle-counter share lands with each machine's full migration. The
/// Headless Horseman/Hellhound walker stand-ins are aiStyle 26 in the C#
/// (`AI_026_Unicorns`), not AI_003 — excluded.
pub fn fighter_walker_kind(kind: crate::npc::NpcKind) -> bool {
    use crate::npc::NpcKind::*;
    matches!(
        kind,
        Zombie
            // S1-D-3z — the ten wave-A zombie variants + the five holiday
            // zombies ride the SAME plain `update_zombie` walker, so their
            // `ai[3]` is free for the idle counter and their C# moan arms
            // (`NPC.cs:57340` lists 132/161/186-189/200/223/319-321/331-332)
            // draw the family's one `Next(1000)` on the caller stream.
            | DoctorBones
            | TheGroom
            | BaldZombie
            | ZombieEskimo
            | PincushionZombie
            | SlimedZombie
            | SwampZombie
            | TwiggyZombie
            | FemaleZombie
            | ZombieRaincoat
            | ZombieDoctor
            | ZombieSuperman
            | ZombiePixie
            | ZombieXmas
            | ZombieSweater
            // S1-D-3z — the Blood Zombie (489) rides the same plain
            // walker; its moan is the 489/586 `Next(800)` arm
            // (`NPC.cs:57344`, `fighter_ambient_moans` below), NOT the
            // family's `Next(1000)` (489 is absent from the `:57340`
            // list) — the kind gate keeps the draw shape exact.
            | BloodZombie
            | Skeleton
            | UndeadMiner
            | GoblinThief
            | GoblinWarrior
            | GoblinPeon
            | PirateDeckhand
            | PirateCaptainsMate
            // W4-f42 — the desert aiStyle-3 walkers: the mummies (78/79/80/
            // 630) draw their own `Next(500)` moan (`NPC.cs:57348`,
            // `fighter_ambient_moans` below) and the WalkingAntlion (580)
            // draws nothing — both ride the same idle counter.
            | Mummy
            | DarkMummy
            | BloodMummy
            | LightMummy
            | WalkingAntlion
            | Scarecrow1
            | Scarecrow2
            | Scarecrow3
            | Scarecrow4
            | Scarecrow5
            | Scarecrow6
            | Scarecrow7
            | Scarecrow8
            | Scarecrow9
            | Scarecrow10
            | Splinterling
            // W4-d22 (style-3 axis): the shared core's newly claimed plain
            // walkers — their `ai[3]` is the C# idle counter now (the core
            // reads it at the `:57336` chase gate). None of these ids sits
            // in the moan type lists (`NPC.cs:57340-57364`), so the moan
            // emitter stays draw-free for them; the counter is the change.
            // The martian engineer/zapper/walker (386/389/520) and the
            // officer (383) are NOT here — their head machines own ai[2]/
            // ai[3] state (the deferred per-type ports).
            | GrayGrunt
            | PirateCaptainsMate
            | Scutlix
            | VortexSoldier
            | SolarSpearman
            // W4-d23 — the Vampire (159) rides the plain
            // `walker_step_fighter` machine (`update_skeleton`), so its
            // `ai[3]` is free for the idle counter; its moan is the
            // 159-only `Next(500)` arm (`NPC.cs:57352-57355`,
            // `fighter_ambient_moans`' case 29 style 7), NOT the
            // family's `Next(1000)` (159 is absent from the `:57340`
            // list) — the Blood Zombie's kind-gate precedent.
            | Vampire
    )
}

/// S1-E-21e — `AI_003_Fighters`' idle counter (`NPC.cs:57034-57145`): the
/// `num57` cap (60; the type-120 row's 180 — its `-120` teleport arm is
/// unported), the `flag9`/`flag10` type gates (`:57076-57118`), the
/// turnaround `flag7` (`:57122-57125`), the stuck-or-idle `ai[3]++` / the
/// moving `ai[3]--` (`:57126-57133`), the `num57 * 10` wrap (`:57134-57137`),
/// the `justHit` reset (`:57138-57141`) and the player-hitbox reset
/// (`:57146-57149` — the `Main.player[target].Hitbox.Intersects` read).
/// `x`/`old_x` are the C#'s `position.X`/`oldPosition.X` at AI time: the
/// previous tick's pre-integration stamp (`NPC.cs:90496`), which the port's
/// `old_x` still holds when this runs ahead of `update_rng`. The
/// `ai[3] == num57` `netUpdate` flag (`:57142-57145`) has no net seam here.
/// `just_hit` rides the port's established `hurt_cd > 0` stand-in
/// (`npc.rs:13420`) — the counter rests at 0 through the hurt cooldown
/// instead of the C#'s single-tick reset (ANALYSIS, S1-E-21e).
pub fn fighter_idle_counter(
    npc_type: i32,
    ai2: f32,
    ai3: f32,
    vx: f32,
    vy: f32,
    facing: i32,
    x: f32,
    old_x: f32,
    just_hit: bool,
    player_intersects: bool,
) -> f32 {
    // `int num57 = 60;` + the type-120 row (`NPC.cs:57034-57037`).
    let num57 = if npc_type == 120 { 180.0 } else { 60.0 };
    // `flag9` (`:57076-57081`): the sigil/sigil-adjacent types 425/471.
    let flag9 = npc_type == 425 || npc_type == 471;
    // `flag10` (`:57082-57118`): the busy-with-action types freeze the
    // counter while `ai[2] > 0`.
    let flag10 = !(matches!(
        npc_type,
        110 | 111
            | 206
            | 214
            | 215
            | 216
            | 291
            | 292
            | 293
            | 350
            | 379
            | 380
            | 381
            | 382
            | 409
            | 411
            | 424
            | 426
            | 466
            | 498
            | 499
            | 500
            | 501
            | 502
            | 503
            | 504
            | 505
            | 506
            | 520
    ) && ai2 > 0.0);
    let mut ai3 = ai3;
    if !flag9 && flag10 {
        // `flag7` (`:57122-57125`): grounded and moving against the facing.
        let flag7 = vy == 0.0 && ((vx > 0.0 && facing < 0) || (vx < 0.0 && facing > 0));
        // `:57126-57133` — the C#'s `|` is non-short-circuiting; both sides
        // are pure reads, so the port's `||` keeps the value.
        if x == old_x || ai3 >= num57 || flag7 {
            ai3 += 1.0;
        } else if vx.abs() > 0.9 && ai3 > 0.0 {
            ai3 -= 1.0;
        }
        if ai3 > num57 * 10.0 {
            ai3 = 0.0;
        }
        if just_hit {
            ai3 = 0.0;
        }
        if player_intersects {
            ai3 = 0.0;
        }
    }
    ai3
}

/// S1-E-21e — `DespawnEncouragement_AIStyle3_Fighters_NotDiscouraged`
/// (`NPC.cs:60300-60330`), the moan arms' gate (`:57336`), ported whole:
/// the long `if` condition (`:60302`) — the five live scene terms (eclipse,
/// day, statue, surface, graveyard), the event terms (snow-moon 343/350,
/// goblin invasion 26/27/28/111/471, don't-starve 163/164, pirate invasion
/// 212-216, martian invasion 381/382/383/385/386/389/391/520), the
/// unconditional exclusion list (31/294-296/47/67/77/78-80/630/110/120/
/// 168/181/185/198/199/206/217-220/239/243/254/255/257/258/291-293/379/
/// 380/464/470/424/73/624), the 631 `ai[2]` and 411 `ai[1]` arms — and the
/// switch (`:60304-60327`): the case-listed types (409/415/419/425/427-429/
/// 508/524-527/580/582) and 528/529/530/532 break to `return true`, every
/// other type returns `NPCID.Sets.CritterThatCanTurnOnPlayers[npcID]`
/// (`NPCID.cs:10436`: 645/639/644/642/643/641/640/299/539/538 — false for
/// the zombie 3 and the skeleton 21). A condition failure falls through to
/// `return true` (`:60329`) — the excluded moan types (the mummies 78-80/
/// 630, the face monster 181, 110, 31/77/291-296) are NOT discouraged even
/// on a non-eclipse day above the surface. `npcInstance` is never null at
/// the moan call site, so the null arms fold to the instance reads.
#[allow(clippy::too_many_arguments)]
pub fn fighter_not_discouraged(
    npc_type: i32,
    y: f64,
    world_surface_px: f64,
    eclipse: bool,
    day_time: bool,
    spawned_from_statue: bool,
    zone_graveyard: bool,
    snow_moon: bool,
    invasion_type: i32,
    dont_starve: bool,
    ai1: f32,
    ai2: f32,
) -> bool {
    // `:60302` — the whole condition, term for term.
    let cond = !eclipse
        && day_time
        && !spawned_from_statue
        && !(y > world_surface_px)
        && !zone_graveyard
        && (!snow_moon || (npc_type != 343 && npc_type != 350))
        && (invasion_type != 1 || !matches!(npc_type, 26 | 27 | 28 | 111 | 471))
        && (!dont_starve || (npc_type != 164 && npc_type != 163))
        && npc_type != 73
        && npc_type != 624
        && (npc_type != 631 || !(ai2 > 0.0))
        && (invasion_type != 3 || !(212..=216).contains(&npc_type))
        && (invasion_type != 4 || !matches!(npc_type, 381 | 382 | 383 | 385 | 386 | 389 | 391 | 520))
        && !matches!(
            npc_type,
            31 | 294
                | 295
                | 296
                | 47
                | 67
                | 77
                | 78
                | 79
                | 80
                | 630
                | 110
                | 120
                | 168
                | 181
                | 185
                | 198
                | 199
                | 206
                | 217
                | 218
                | 219
                | 220
                | 239
                | 243
                | 254
                | 255
                | 257
                | 258
                | 291
                | 292
                | 293
                | 379
                | 380
                | 464
                | 470
                | 424
        )
        && (npc_type != 411 || (!(ai1 >= 180.0) && !(ai1 < 90.0)));
    if cond {
        // `:60304-60327` — the switch: the case-listed types break to the
        // tail's `return true`; the default returns the critter set unless
        // the type is 528/529/530/532 (those break too).
        match npc_type {
            409 | 415 | 419 | 425 | 427 | 428 | 429 | 508 | 524 | 525 | 526 | 527 | 580 | 582 => {}
            _ => {
                if !matches!(npc_type, 528 | 529 | 530 | 532) {
                    // `NPCID.Sets.CritterThatCanTurnOnPlayers` (`NPCID.cs:10436`).
                    return matches!(npc_type, 645 | 639 | 644 | 642 | 643 | 641 | 640 | 299 | 539 | 538);
                }
            }
        }
    }
    // `:60329` — the early-exit path: NOT discouraged.
    true
}

/// S1-E-21e — `AI_003_Fighters`' ambient moan arms (`NPC.cs:57336-57368`):
/// behind `ai[3] < num57 && DespawnEncouragement_AIStyle3_Fighters_
/// NotDiscouraged` (`:57336`) and `shimmerTransparency < 1f` (`:57338`),
/// seven independent `if` arms each roll `Main.rand.Next(N) == 0` on the
/// `UpdateNPCs` stream (`Main.cs:17767`) and play their legacy cue at the
/// NPC's position. The arms are NOT else-chained — the type-162
/// Frankenstein matches both the 1000-roll zombie arm and its own 500-roll
/// `PlaySound(29, …, 6)` arm, so both rolls are consumed in the C#'s order
/// and `play` fires per hit, interleaved with the later arms' rolls. The
/// caller's `play` is [`crate::audio::Audio::play_fighter_moan`] — the
/// legacy case's slot/pitch draws run inside it, AFTER the attenuation
/// gate (the `PlaySound` prologue, `LegacySoundPlayer.cs:360-431`).
///
/// `not_discouraged` is the caller's
/// [`fighter_not_discouraged`] — the full
/// `DespawnEncouragement_AIStyle3_Fighters_NotDiscouraged` port
/// (`NPC.cs:60300-60330`): the `:60302` condition's event/exclusion terms
/// are NOT vacuous for the moan type sets — the mummies 78-80/630, the
/// face monster 181, and 31/77/110/291-296 sit in the unconditional
/// exclusion list and reach `return true` (`:60329`) even on a day above
/// the surface; the non-excluded types (the zombie 3, the skeleton 21)
/// fall to the switch's default `CritterThatCanTurnOnPlayers` arm (false
/// for both), so for them the five live scene terms decide.
pub fn fighter_ambient_moans(
    npc_type: i32,
    idle_counter: f32,
    not_discouraged: bool,
    shimmer_transparency: f32,
    rng: &mut tr_core::UnifiedRandom,
    play: &mut dyn FnMut(u32, i32, &mut tr_core::UnifiedRandom),
) {
    // `:57336` — `num57` is 60 for every type the arms name (the 120 row's
    // 180 cannot reach an arm).
    if !(idle_counter < 60.0 && not_discouraged) {
        return;
    }
    // `:57338` — the shimmer gate wraps ALL seven arms.
    if !(shimmer_transparency < 1.0) {
        return;
    }
    // `:57340-57343` — the zombie/skeleton family's 1000-roll, case 14.
    if matches!(
        npc_type,
        3 | 691
            | 591
            | 590
            | 331
            | 332
            | 21
            | 31
            | 294
            | 295
            | 296
            | 77
            | 110
            | 132
            | 167
            | 161
            | 162
            | 186
            | 187
            | 188
            | 189
            | 197
            | 200
            | 201
            | 202
            | 203
            | 223
            | 291
            | 292
            | 293
            | 320
            | 321
            | 319
            | 481
            | 632
            | 635
    ) || (449..=452).contains(&npc_type)
    {
        if rng.next_max(1000) == 0 {
            play(14, 1, rng);
        }
    }
    // `:57344-57347` — the 489/586 pair's 800-roll, case 14 with the type
    // as the Style (the `Next(21, 24)` slot arm).
    if (npc_type == 489 || npc_type == 586) && rng.next_max(800) == 0 {
        play(14, npc_type, rng);
    }
    // `:57348-57351` — the mummy family's 500-roll, case 26.
    if matches!(npc_type, 78 | 79 | 80 | 630) && rng.next_max(500) == 0 {
        play(26, 1, rng);
    }
    // `:57352-57355` — the vampire's 500-roll, case 29 style 7.
    if npc_type == 159 && rng.next_max(500) == 0 {
        play(29, 7, rng);
    }
    // `:57356-57359` — Frankenstein's own 500-roll, case 29 style 6.
    if npc_type == 162 && rng.next_max(500) == 0 {
        play(29, 6, rng);
    }
    // `:57360-57363` — the face monster's 500-roll, case 29 style 8.
    if npc_type == 181 && rng.next_max(500) == 0 {
        play(29, 8, rng);
    }
    // `:57364-57367` — the armored-bones 269-280 range's 1000-roll, case 14.
    if (269..=280).contains(&npc_type) && rng.next_max(1000) == 0 {
        play(14, 1, rng);
    }
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

    /// A signed `Main.rand.Next(min, max)` stand-in over [`Rolls`]: the same
    /// call-index pattern, so a gate can force an exact draw to a value.
    fn signed_roll<'a>(
        state: &'a mut Rolls,
        forced: &'a dyn Fn(i32, i32, u32) -> Option<i32>,
    ) -> impl FnMut(i32, i32) -> i32 + 'a {
        move |lo: i32, hi: i32| {
            let call = state.calls;
            state.calls += 1;
            if let Some(v) = forced(lo, hi, call) {
                return v;
            }
            lo + (1 % (hi - lo).max(1)) as i32
        }
    }

    #[test]
    fn thunder_cue_pick_wraps_the_six_family() {
        // A free engine: one slot draw + the pitch draw, the slot cycles.
        for slot in 0..6u32 {
            let mut state = Rolls::new();
            let (cue, picked, _) = thunder_cue_pick(
                &mut signed_roll(&mut state, &|lo, hi, _| {
                    Some(lo + (slot as i32 % (hi - lo).max(1)))
                }),
                &|_| false,
            );
            assert_eq!(cue, format!("Thunder_{slot}"));
            assert_eq!(picked, slot);
            assert_eq!(state.calls, 2, "one slot draw + one pitch draw");
        }
        // The pitch jitter is `Next(-10, 11) * 0.01`.
        let mut state = Rolls::new();
        let (_, _, pitch) = thunder_cue_pick(
            &mut signed_roll(&mut state, &|lo, hi, call| {
                if call == 1 {
                    Some(-7)
                } else {
                    Some(lo)
                }
            }),
            &|_| false,
        );
        assert!((pitch + 0.07).abs() < 1e-6);
    }

    #[test]
    fn thunder_cue_pick_rerolls_while_the_slot_is_playing() {
        // `LegacySoundPlayer.cs:456-464`: every draw lands on a playing slot
        // for the first five re-rolls, then the engine frees up — the pick
        // takes the sixth draw's slot. 1 initial + 5 re-rolls + the pitch.
        let mut state = Rolls::new();
        let (cue, picked, _) = thunder_cue_pick(
            &mut signed_roll(&mut state, &|lo, hi, call| {
                Some(lo + ((call.min(4) as i32) % (hi - lo).max(1)))
            }),
            &|slot| slot < 4,
        );
        assert_eq!(cue, "Thunder_4");
        assert_eq!(picked, 4);
        assert_eq!(state.calls, 6, "1 initial + 4 re-rolls + the pitch");
        // All six slots busy for the whole loop: the pick stops after the
        // six re-rolls (1 + 6 slot draws) and steals the last pick's slot
        // (`:466-467`'s Stop()), then still draws the pitch.
        let mut state = Rolls::new();
        let (cue, picked, _) = thunder_cue_pick(
            &mut signed_roll(&mut state, &|lo, hi, call| {
                Some(lo + (((call % 6) as i32) % (hi - lo).max(1)))
            }),
            &|_| true,
        );
        assert_eq!(cue, "Thunder_0", "call 6 → (6 % 6) = 0");
        assert_eq!(picked, 0);
        assert_eq!(state.calls, 8, "7 slot draws + the pitch");
    }

    #[test]
    fn liquid_change_cues_match_the_case_tables() {
        // The trackable pairs: `Sounds/Custom/<name>_<0..2>`, volume 1, no
        // legacy pitch roll (`SoundID.cs:1600-1602`).
        let lava_water = liquid_change_cue(TileChangeType::LavaWater).unwrap();
        assert_eq!(lava_water.cue, "liquids_water_lava");
        assert_eq!(lava_water.sound_type, 42);
        assert_eq!(lava_water.variations, 3);
        assert_eq!(lava_water.volume_scale, 1.0);
        assert_eq!(lava_water.pitch_roll, None);
        // The shimmer arm: `PlaySound(19, x, y, Style 4)` → case 19's
        // `SoundSplash[4]`, volume ×0.75 and `Next(-20, 1)` pitch
        // (`LegacySoundPlayer.cs:871-884`).
        let shimmer = liquid_change_cue(TileChangeType::ShimmerWater).unwrap();
        assert_eq!(shimmer.cue, "Splash_4");
        assert_eq!(shimmer.sound_type, 19);
        assert_eq!(shimmer.variations, 1);
        assert_eq!(shimmer.volume_scale, 0.75);
        assert_eq!(shimmer.pitch_roll, Some((-20, 1)));
        assert!(liquid_change_cue(TileChangeType::None).is_none());
    }

    #[test]
    fn critter_voices_resolve_the_zombie_family_in_order() {
        // Case 44 (`:1051-1062`): `Next(106, 109)` → the slot asset, no
        // guard, ×0.2, `Next(-70, 1) * 0.01`.
        let mut state = Rolls::new();
        let voice = critter_legacy_voice(
            44,
            1,
            &mut signed_roll(&mut state, &|lo, hi, call| {
                if call == 0 {
                    Some(107)
                } else if call == 1 {
                    Some(-30)
                } else {
                    Some(lo)
                }
            }),
            &|_| unreachable!("case 44 has no guard"),
        )
        .unwrap();
        assert_eq!(voice.slot, 107);
        assert_eq!(voice.asset, "Zombie_107");
        assert_eq!(voice.volume_mult, 0.2);
        assert_eq!(voice.pitch, -30.0f32 * 0.01);
        assert_eq!(state.calls, 2);

        // Case 46 (`:1063-1076`): the 110/111 guards return null; then the
        // slot draw, the rare `Next(300)` swap (`Next(3) == 0 → 114`) and
        // the pitch — four draws in the C#'s order when the swap fires.
        let mut state = Rolls::new();
        assert!(
            critter_legacy_voice(46, 1, &mut signed_roll(&mut state, &|_, _, _| Some(0)), &|s| {
                s == 111
            })
            .is_none(),
            "a busy 111 blocks the whole cue"
        );
        let mut state = Rolls::new();
        let voice = critter_legacy_voice(
            46,
            1,
            &mut signed_roll(&mut state, &|lo, hi, call| match call {
                0 => Some(110), // Next(110, 112)
                1 => Some(0),   // Next(300) == 0 → swap
                2 => Some(1),   // Next(3) == 1 → the Next(2) arm
                3 => Some(1),   // Next(2) != 0 → 112
                _ => Some(-10), // the pitch
            }),
            // The guards run before the pick: a busy 110 or 111 blocks the
            // cue outright (`:1064-1069`) — the first block pins that. Here
            // both are free.
            &|_| false,
        )
        .unwrap();
        assert_eq!(voice.slot, 112);
        assert_eq!(voice.asset, "Zombie_112");
        assert_eq!(voice.volume_mult, 0.9);
        assert_eq!(state.calls, 5, "slot + Next(300) + Next(3) + Next(2) + pitch");

        // Case 32 (`:1133-1143`): the Style is the slot, behind the guard;
        // ×0.15, `Next(-70, 26)`.
        let mut state = Rolls::new();
        assert!(
            critter_legacy_voice(32, 17, &mut signed_roll(&mut state, &|_, _, _| Some(0)), &|s| s
                == 17)
            .is_none(),
            "a busy style slot blocks the cue"
        );
        let mut state = Rolls::new();
        let voice = critter_legacy_voice(
            32,
            18,
            &mut signed_roll(&mut state, &|lo, hi, call| {
                if call == 0 {
                    Some(26 - 1)
                } else {
                    Some(lo)
                }
            }),
            &|s| s == 14,
        )
        .unwrap();
        assert_eq!(voice.slot, 18);
        assert_eq!(voice.asset, "Zombie_18");
        assert_eq!(voice.volume_mult, 0.15);
        assert!((voice.pitch - 0.25).abs() < 1e-6);
        assert_eq!(state.calls, 1);

        // Case 30 (`:1099-1124`): `Next(10, 12)` without the rare arm keeps
        // the `Next(-70, 1)` pitch; ×0.75.
        let mut state = Rolls::new();
        let voice = critter_legacy_voice(
            30,
            1,
            &mut signed_roll(&mut state, &|lo, hi, call| match call {
                0 => Some(11),
                1 => Some(299), // Next(300) != 0 → no swap
                _ => Some(-70),
            }),
            &|s| s == 12,
        )
        .unwrap();
        assert_eq!(voice.slot, 11);
        assert_eq!(voice.asset, "Zombie_11");
        assert_eq!(voice.volume_mult, 0.75);
        assert!((voice.pitch + 0.7).abs() < 1e-6);

        // Case 33 (`:1200-1213`): slot 15 behind the guard; ×0.2,
        // `Next(-10, 31)`.
        let mut state = Rolls::new();
        let voice = critter_legacy_voice(
            33,
            1,
            &mut signed_roll(&mut state, &|lo, hi, call| {
                if call == 0 {
                    Some(31 - 1)
                } else {
                    Some(lo)
                }
            }),
            // The busy-15 block is the same shape the case-32 block pins.
            &|_| false,
        )
        .unwrap();
        assert_eq!(voice.slot, 15);
        assert_eq!(voice.volume_mult, 0.2);
        assert!((voice.pitch - 0.30).abs() < 1e-6);

        // An id outside the family resolves to nothing.
        let mut state = Rolls::new();
        assert!(
            critter_legacy_voice(19, 4, &mut signed_roll(&mut state, &|_, _, _| Some(0)), &|_| {
                false
            })
            .is_none()
        );
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

    /// W4-c10 sweep — the waterfowl ambient census
    /// (`NPC.UpdateNPC_CritterSounds`, `NPC.cs:92249-92340`): the ducks
    /// 362-365 quack case 30 by day behind `Next(200)` (`:92257-92262`),
    /// the seagulls 602/603 cry case 44 by day behind `Next(1200)`
    /// (`:92253-92256`), the grebes 608/609 match NO row — silent, zero
    /// draws. The hit/death rows are the SetDefaults NPCHit1/NPCDeath1
    /// (`:13621-13622` etc., landed phase 1); there is NO takeoff/
    /// landing cue (the TransformVisuals census: the pairs match no
    /// arm) and NO idle quack beyond these rolls.
    #[test]
    fn w4_c10_the_waterfowl_ambient_rows_census() {
        // The four ducks: day + Next(200) == 0 → case 30.
        for ty in [362, 363, 364, 365] {
            let mut state = Rolls::with_zero(0);
            let cue =
                critter_sound_cue(ty, true, 1000.0, 3000.0, 3000.0, 0.0, &mut roll(&mut state));
            assert_eq!(cue, Some((30, 1.0)), "type {ty} quacks (:92261)");
            assert_eq!(state.calls, 1, "type {ty}: one Next(200)");
            // Night: the day gate short-circuits — NO draw.
            let mut night = Rolls::new();
            assert_eq!(
                critter_sound_cue(ty, false, 1000.0, 3000.0, 3000.0, 0.0, &mut roll(&mut night)),
                None
            );
            assert_eq!(night.calls, 0, "type {ty}: the night gate draws nothing");
        }
        // The seagulls: day + Next(1200) == 0 → case 44 (the row sits
        // ABOVE the duck row, an independent if — `:92253`).
        for ty in [602, 603] {
            let mut state = Rolls::with_zero(0);
            let cue =
                critter_sound_cue(ty, true, 1000.0, 3000.0, 3000.0, 0.0, &mut roll(&mut state));
            assert_eq!(cue, Some((44, 1.0)), "type {ty} cries (:92255)");
            assert_eq!(state.calls, 1, "type {ty}: one Next(1200)");
        }
        // The grebes 608/609: no row in the whole chain — silent, and
        // the else-if ladder draws NOTHING for them (the 361/445/687
        // row's speed gate is on the else branch, `:92264`).
        for ty in [608, 609] {
            for day in [true, false] {
                let mut state = Rolls::new();
                assert_eq!(
                    critter_sound_cue(ty, day, 1000.0, 3000.0, 3000.0, 0.0, &mut roll(&mut state)),
                    None,
                    "type {ty} has no ambient row"
                );
                assert_eq!(state.calls, 0, "type {ty}: zero draws");
            }
        }
        // The failing-roll stream contract: a duck by day with a
        // failing Next(200) still consumes the draw (the `&&` draws
        // the right operand; the branch then ends the chain).
        let mut calls = 0u32;
        let mut fail = |hi: u32| {
            calls += 1;
            1 % hi.max(1)
        };
        assert_eq!(
            critter_sound_cue(362, true, 1000.0, 3000.0, 3000.0, 0.0, &mut fail),
            None
        );
        assert_eq!(calls, 1);
    }

    /// W4-c10 sweep — the waterfowl voice arms: case 30 (the duck
    /// quack, `LegacySoundPlayer.cs:1099-1124`) and case 44 (the
    /// seagull cry, `:1051-1062`) resolve the zombie bank in the C#
    /// draw order. Case 30: `Next(10, 12)` slot, `Next(300)` rare
    /// re-target (the playing guard on 12 blocks BEFORE any further
    /// draw), the pitch range splits on the re-target.
    #[test]
    fn w4_c10_the_quack_and_cry_voices_draw_in_order() {
        // Case 30 plain: slot 10, no re-target, pitch Next(-70, 1).
        let mut state = Rolls::new();
        let voice = critter_legacy_voice(
            30,
            1,
            &mut signed_roll(&mut state, &|lo, hi, call| match call {
                0 => Some(10),  // Next(10, 12)
                1 => Some(1),   // Next(300) != 0 — no re-target
                _ => Some(-40), // the pitch Next(-70, 1)
            }),
            &|_| unreachable!("the guard only runs on the re-target"),
        )
        .unwrap();
        assert_eq!(voice.slot, 10);
        assert_eq!(voice.asset, "Zombie_10");
        assert_eq!(voice.volume_mult, 0.75);
        assert_eq!(voice.pitch, -0.4);
        assert_eq!(state.calls, 3, "slot + Next(300) + pitch");
        // Case 30 re-target: Next(300) == 0, slot 12 free → the pitch
        // range flips to Next(-40, 21).
        let mut state = Rolls::new();
        let voice = critter_legacy_voice(
            30,
            1,
            &mut signed_roll(&mut state, &|lo, hi, call| match call {
                0 => Some(11),
                1 => Some(0), // the re-target fires
                _ => Some(5), // the pitch Next(-40, 21)
            }),
            &|_| false,
        )
        .unwrap();
        assert_eq!(voice.slot, 12);
        assert_eq!(voice.pitch, 0.05);
        assert_eq!(state.calls, 3);
        // Case 30 re-target blocked: slot 12 busy → None, NO pitch draw.
        let mut state = Rolls::new();
        assert!(
            critter_legacy_voice(
                30,
                1,
                &mut signed_roll(&mut state, &|_, _, call| match call {
                    0 => Some(10),
                    1 => Some(0),
                    _ => unreachable!("the guard precedes the pitch"),
                }),
                &|s| s == 12,
            )
            .is_none()
        );
        assert_eq!(state.calls, 2, "slot + Next(300), the guard blocks");
        // Case 44: Next(106, 109) + Next(-70, 1), no guard, ×0.2.
        let mut state = Rolls::new();
        let voice = critter_legacy_voice(
            44,
            1,
            &mut signed_roll(&mut state, &|_, _, call| match call {
                0 => Some(108),
                _ => Some(-55),
            }),
            &|_| unreachable!("case 44 has no guard"),
        )
        .unwrap();
        assert_eq!(voice.slot, 108);
        assert_eq!(voice.asset, "Zombie_108");
        assert_eq!(voice.volume_mult, 0.2);
        assert_eq!(voice.pitch, -0.55);
        assert_eq!(state.calls, 2);
    }

    /// S1-E-21e — the fighter-moan cases' draw contract
    /// (`LegacySoundPlayer.cs:793-824/:969-978/:1008-1050/:1059-1070`).
    #[test]
    fn s1_e_21e_the_moan_cases_draw_in_the_c_order() {
        // Case 14 default (`:817`): ONE `Next(3)` slot draw, NO pitch draw
        // (the case never sets `Pitch`), ×0.4, no guard — the busy slot is
        // overwritten without a Stop.
        let mut state = Rolls::new();
        let voice = critter_legacy_voice(
            14,
            1,
            &mut signed_roll(&mut state, &|lo, hi, _| Some(lo + (2 % (hi - lo).max(1)))),
            &|_| unreachable!("case 14 has no playing guard"),
        )
        .unwrap();
        assert_eq!(voice.slot, 2);
        assert_eq!(voice.asset, "Zombie_2");
        assert_eq!(voice.volume_mult, 0.4);
        assert_eq!(voice.pitch, 0.0, "case 14 draws no pitch");
        assert_eq!(state.calls, 1, "the slot draw only");

        // Case 14 style 542 (`:796-803`): slot 7 pinned, ZERO draws.
        let mut state = Rolls::new();
        let voice = critter_legacy_voice(
            14,
            542,
            &mut signed_roll(&mut state, &|_, _, _| unreachable!("style 542 draws nothing")),
            &|_| false,
        )
        .unwrap();
        assert_eq!(voice.slot, 7);
        assert_eq!(voice.asset, "Zombie_7");
        assert_eq!(state.calls, 0);

        // Case 14 styles 489/586 (`:805-813`): the `Next(21, 24)` slot arm,
        // still no pitch.
        for style in [489u32, 586] {
            let mut state = Rolls::new();
            let voice = critter_legacy_voice(
                14,
                style,
                &mut signed_roll(&mut state, &|lo, hi, _| {
                    Some(lo + (1 % (hi - lo).max(1)))
                }),
                &|_| false,
            )
            .unwrap();
            assert_eq!(voice.slot, 22);
            assert_eq!(voice.volume_mult, 0.4);
            assert_eq!(voice.pitch, 0.0);
            assert_eq!(state.calls, 1);
        }

        // Case 26 (`:969-978`): `Next(3, 5)` + one `Next(-10, 11)` pitch,
        // ×0.9, no guard.
        let mut state = Rolls::new();
        let voice = critter_legacy_voice(
            26,
            1,
            &mut signed_roll(&mut state, &|lo, hi, call| {
                if call == 0 {
                    Some(4)
                } else {
                    Some(-7)
                }
            }),
            &|_| unreachable!("case 26 has no playing guard"),
        )
        .unwrap();
        assert_eq!(voice.slot, 4);
        assert_eq!(voice.asset, "Zombie_4");
        assert_eq!(voice.volume_mult, 0.9);
        assert!((voice.pitch + 0.07).abs() < 1e-6);
        assert_eq!(state.calls, 2, "slot + pitch");

        // Case 29 (`:1008-1050`): the Style IS the slot; the playing guard
        // returns BEFORE the pitch draw (`:1041-1044`).
        let mut state = Rolls::new();
        assert!(
            critter_legacy_voice(29, 7, &mut signed_roll(&mut state, &|_, _, _| Some(0)), &|s| s
                == 7)
            .is_none(),
            "a busy style slot blocks the cue"
        );
        assert_eq!(state.calls, 0, "the guard precedes the pitch draw");
        // The moan arms' styles 6/7/8 keep mult 1.0; one pitch draw.
        for style in [6u32, 7, 8] {
            let mut state = Rolls::new();
            let voice = critter_legacy_voice(
                29,
                style,
                &mut signed_roll(&mut state, &|_, _, _| Some(5)),
                &|_| false,
            )
            .unwrap();
            assert_eq!(voice.slot, style as usize);
            assert_eq!(voice.volume_mult, 1.0);
            assert!((voice.pitch - 0.05).abs() < 1e-6);
            assert_eq!(state.calls, 1);
        }
        // The full switch's volume arms (`:1009-1040`): 24..87 ×0.5,
        // 88..91 ×0.7, 92 ×0.5, 93..99 ×0.4, 100/101 ×0.25, 102 ×0.4,
        // 103 ×0.4, 104 ×0.55.
        for (style, mult) in [
            (24u32, 0.5f32),
            (87, 0.5),
            (90, 0.7),
            (92, 0.5),
            (95, 0.4),
            (100, 0.25),
            (101, 0.25),
            (102, 0.4),
            (103, 0.4),
            (104, 0.55),
        ] {
            let mut state = Rolls::new();
            let voice = critter_legacy_voice(
                29,
                style,
                &mut signed_roll(&mut state, &|_, _, _| Some(0)),
                &|_| false,
            )
            .unwrap();
            assert_eq!(voice.volume_mult, mult, "style {style}");
        }

        // Case 45 (`:1059-1070`): slot 109 pinned, the guard before the
        // pitch draw, ×0.3, `Next(-10, 11)`.
        let mut state = Rolls::new();
        assert!(
            critter_legacy_voice(45, 1, &mut signed_roll(&mut state, &|_, _, _| Some(0)), &|s| s
                == 109)
            .is_none()
        );
        assert_eq!(state.calls, 0);
        let mut state = Rolls::new();
        let voice = critter_legacy_voice(
            45,
            1,
            &mut signed_roll(&mut state, &|_, _, _| Some(-10)),
            &|_| false,
        )
        .unwrap();
        assert_eq!(voice.slot, 109);
        assert_eq!(voice.asset, "Zombie_109");
        assert_eq!(voice.volume_mult, 0.3);
        assert!((voice.pitch + 0.10).abs() < 1e-6);
        assert_eq!(state.calls, 1);
    }

    /// S1-E-21e — `fighter_idle_counter`'s branches (`NPC.cs:57034-57149`).
    #[test]
    fn s1_e_21e_the_idle_counter_follows_the_branch_table() {
        // Stuck (`position.X == oldPosition.X`, `:57126`): the counter ticks
        // up even while moving fast.
        assert_eq!(
            fighter_idle_counter(3, 0.0, 5.0, 2.0, 0.0, 1, 100.0, 100.0, false, false),
            6.0
        );
        // Moving with the facing (`:57130-57132`): |vx| > 0.9 decrements.
        assert_eq!(
            fighter_idle_counter(3, 0.0, 5.0, 2.0, 0.0, 1, 100.0, 90.0, false, false),
            4.0
        );
        // A slow walk neither ticks nor decrements.
        assert_eq!(
            fighter_idle_counter(3, 0.0, 5.0, 0.5, 0.0, 1, 100.0, 90.0, false, false),
            5.0
        );
        // The turnaround `flag7` (`:57122-57125`): grounded, moving against
        // the facing — ticks up despite the motion.
        assert_eq!(
            fighter_idle_counter(3, 0.0, 5.0, 2.0, 0.0, -1, 100.0, 90.0, false, false),
            6.0
        );
        // At/over the cap (`ai[3] >= num57`) the counter keeps ticking up.
        assert_eq!(
            fighter_idle_counter(3, 0.0, 60.0, 2.0, 0.0, 1, 100.0, 90.0, false, false),
            61.0
        );
        // The `num57 * 10` wrap (`:57134-57137`): 601 → 0.
        assert_eq!(
            fighter_idle_counter(3, 0.0, 600.0, 0.0, 0.0, 1, 100.0, 100.0, false, false),
            0.0
        );
        // The `justHit` reset (`:57138-57141`) and the player-hitbox reset
        // (`:57146-57149`) both zero the counter.
        assert_eq!(
            fighter_idle_counter(3, 0.0, 5.0, 0.0, 0.0, 1, 100.0, 100.0, true, false),
            0.0
        );
        assert_eq!(
            fighter_idle_counter(3, 0.0, 5.0, 0.0, 0.0, 1, 100.0, 100.0, false, true),
            0.0
        );
        // `flag9` (the sigil types 425/471, `:57076-57081`): frozen.
        assert_eq!(
            fighter_idle_counter(425, 0.0, 5.0, 0.0, 0.0, 1, 100.0, 100.0, false, false),
            5.0
        );
        // `flag10` (the busy-with-action types, `:57082-57118`): frozen
        // while `ai[2] > 0`, live once it clears.
        assert_eq!(
            fighter_idle_counter(110, 3.0, 5.0, 0.0, 0.0, 1, 100.0, 100.0, false, false),
            5.0
        );
        assert_eq!(
            fighter_idle_counter(110, 0.0, 5.0, 0.0, 0.0, 1, 100.0, 100.0, false, false),
            6.0
        );
        // The type-120 row's 180 cap (`:57036-57038`): the wrap is 1800.
        assert_eq!(
            fighter_idle_counter(120, 0.0, 600.0, 0.0, 0.0, 1, 100.0, 100.0, false, false),
            601.0
        );
    }

    /// S1-E-21e — `fighter_ambient_moans`' roll consumption
    /// (`NPC.cs:57336-57367`): the seven arms are independent `if`s on the
    /// `UpdateNPCs` stream; a matching type consumes its roll even when the
    /// roll fails, and the type-162 Frankenstein matches TWO arms.
    #[test]
    fn s1_e_21e_the_moan_arms_roll_in_the_c_order() {
        use tr_core::UnifiedRandom;
        // Seed 1: `Next(1000)` fails — the zombie's arm consumes exactly
        // one draw and plays nothing.
        let mut rng = UnifiedRandom::new(1);
        let mut witness = UnifiedRandom::new(1);
        let _ = witness.next_max(1000);
        let mut plays: Vec<(u32, i32)> = Vec::new();
        fighter_ambient_moans(3, 0.0, true, 0.0, &mut rng, &mut |c, s, _| plays.push((c, s)));
        assert!(plays.is_empty());
        assert_eq!(rng.peek(), witness.peek(), "one failed roll consumed");

        // Seed 146: `Next(1000)` hits 0 — the zombie plays case 14 style 1;
        // the case's own draws run inside the play callback (the caller's
        // `Audio::play_fighter_moan`), not in the arm table.
        let mut rng = UnifiedRandom::new(146);
        let mut witness = UnifiedRandom::new(146);
        let _ = witness.next_max(1000);
        let mut plays: Vec<(u32, i32)> = Vec::new();
        fighter_ambient_moans(3, 0.0, true, 0.0, &mut rng, &mut |c, s, _| plays.push((c, s)));
        assert_eq!(plays, vec![(14, 1)]);
        assert_eq!(rng.peek(), witness.peek());

        // The 489/586 pair's 800-roll arm carries the TYPE as the Style
        // (the case-14 `Next(21, 24)` slot arm, `:57344-57347`).
        for npc_type in [489, 586] {
            let mut rng = UnifiedRandom::new(146);
            let mut witness = UnifiedRandom::new(146);
            let _ = witness.next_max(800);
            let mut plays: Vec<(u32, i32)> = Vec::new();
            fighter_ambient_moans(npc_type, 0.0, true, 0.0, &mut rng, &mut |c, s, _| {
                plays.push((c, s))
            });
            assert_eq!(plays, vec![(14, npc_type)]);
            assert_eq!(rng.peek(), witness.peek());
        }

        // The mummy family's 500-roll → case 26 (`:57348-57351`).
        for npc_type in [78, 79, 80, 630] {
            let mut rng = UnifiedRandom::new(146);
            let mut witness = UnifiedRandom::new(146);
            let _ = witness.next_max(500);
            let mut plays: Vec<(u32, i32)> = Vec::new();
            fighter_ambient_moans(npc_type, 0.0, true, 0.0, &mut rng, &mut |c, s, _| {
                plays.push((c, s))
            });
            assert_eq!(plays, vec![(26, 1)], "type {npc_type}");
            assert_eq!(rng.peek(), witness.peek());
        }

        // The vampire 159 → case 29 style 7, Frankenstein 162 → style 6,
        // the face monster 181 → style 8 (`:57352-57363`).
        for (npc_type, style) in [(159, 7), (181, 8)] {
            let mut rng = UnifiedRandom::new(146);
            let mut witness = UnifiedRandom::new(146);
            let _ = witness.next_max(500);
            let mut plays: Vec<(u32, i32)> = Vec::new();
            fighter_ambient_moans(npc_type, 0.0, true, 0.0, &mut rng, &mut |c, s, _| {
                plays.push((c, s))
            });
            assert_eq!(plays, vec![(29, style)], "type {npc_type}");
            assert_eq!(rng.peek(), witness.peek());
        }

        // The armored-bones 269-280 range's 1000-roll → case 14
        // (`:57364-57367`).
        for npc_type in [269, 280] {
            let mut rng = UnifiedRandom::new(146);
            let mut witness = UnifiedRandom::new(146);
            let _ = witness.next_max(1000);
            let mut plays: Vec<(u32, i32)> = Vec::new();
            fighter_ambient_moans(npc_type, 0.0, true, 0.0, &mut rng, &mut |c, s, _| {
                plays.push((c, s))
            });
            assert_eq!(plays, vec![(14, 1)], "type {npc_type}");
            assert_eq!(rng.peek(), witness.peek());
        }

        // The type-162 Frankenstein matches BOTH the zombie-family 1000
        // arm AND its own 500 arm — the arms are NOT else-chained. Seed
        // 34995 hits 0 on both: two plays, two rolls, in the C#'s order.
        let mut rng = UnifiedRandom::new(34995);
        let mut witness = UnifiedRandom::new(34995);
        let _ = witness.next_max(1000);
        let _ = witness.next_max(500);
        let mut plays: Vec<(u32, i32)> = Vec::new();
        fighter_ambient_moans(162, 0.0, true, 0.0, &mut rng, &mut |c, s, _| plays.push((c, s)));
        assert_eq!(plays, vec![(14, 1), (29, 6)]);
        assert_eq!(rng.peek(), witness.peek(), "both arms' rolls consumed");
        // Seed 580: the first arm fails, the second hits — the failed roll
        // is still consumed ahead of the second arm's draw.
        let mut rng = UnifiedRandom::new(580);
        let mut witness = UnifiedRandom::new(580);
        let _ = witness.next_max(1000);
        let _ = witness.next_max(500);
        let mut plays: Vec<(u32, i32)> = Vec::new();
        fighter_ambient_moans(162, 0.0, true, 0.0, &mut rng, &mut |c, s, _| plays.push((c, s)));
        assert_eq!(plays, vec![(29, 6)]);
        assert_eq!(rng.peek(), witness.peek());

        // The gates are draw-free: the idle counter at the cap, a
        // discouraged fighter, and a fully-shimmered NPC each consume
        // nothing (`:57336`/`:57338`).
        for (counter, not_discouraged, shimmer) in
            [(60.0, true, 0.0), (0.0, false, 0.0), (0.0, true, 1.0)]
        {
            let mut rng = UnifiedRandom::new(146);
            let peek = rng.peek();
            let mut plays: Vec<(u32, i32)> = Vec::new();
            fighter_ambient_moans(
                3,
                counter,
                not_discouraged,
                shimmer,
                &mut rng,
                &mut |c, s, _| plays.push((c, s)),
            );
            assert!(plays.is_empty());
            assert_eq!(rng.peek(), peek, "the gate is draw-free");
        }
        // A kind outside every arm (the skeleton 21 matches the first arm;
        // the scarecrow 305 matches none) draws nothing.
        let mut rng = UnifiedRandom::new(146);
        let peek = rng.peek();
        let mut plays: Vec<(u32, i32)> = Vec::new();
        fighter_ambient_moans(305, 0.0, true, 0.0, &mut rng, &mut |c, s, _| plays.push((c, s)));
        assert!(plays.is_empty());
        assert_eq!(rng.peek(), peek);
    }

    /// S1-D-3z — the fifteen zombie variants (the ten wave-A kinds + the
    /// five holiday kinds) are plain `update_zombie` walkers: their `ai[3]`
    /// is free for the idle counter (`fighter_walker_kind`) and the holiday
    /// types 319-321/331-332 sit in the C#'s zombie-family 1000-roll moan
    /// arm (`NPC.cs:57340`), consuming exactly ONE draw per qualifying
    /// tick on the caller stream.
    #[test]
    fn s1_d_3z_the_zombie_variants_are_walker_kinds_and_moan_once() {
        use crate::npc::NpcKind;
        use tr_core::UnifiedRandom;
        for kind in [
            NpcKind::Zombie,
            NpcKind::DoctorBones,
            NpcKind::TheGroom,
            NpcKind::BaldZombie,
            NpcKind::ZombieEskimo,
            NpcKind::PincushionZombie,
            NpcKind::SlimedZombie,
            NpcKind::SwampZombie,
            NpcKind::TwiggyZombie,
            NpcKind::FemaleZombie,
            NpcKind::ZombieRaincoat,
            NpcKind::ZombieDoctor,
            NpcKind::ZombieSuperman,
            NpcKind::ZombiePixie,
            NpcKind::ZombieXmas,
            NpcKind::ZombieSweater,
        ] {
            assert!(fighter_walker_kind(kind), "{kind:?} is a plain walker");
        }
        // Each holiday type draws the family's one `Next(1000)` (seed 146
        // hits 0 → case 14 style 1; seed 1 fails → still one draw).
        for npc_type in [319, 320, 321, 331, 332] {
            let mut rng = UnifiedRandom::new(146);
            let mut witness = UnifiedRandom::new(146);
            let _ = witness.next_max(1000);
            let mut plays: Vec<(u32, i32)> = Vec::new();
            fighter_ambient_moans(npc_type, 0.0, true, 0.0, &mut rng, &mut |c, s, _| {
                plays.push((c, s))
            });
            assert_eq!(plays, vec![(14, 1)], "type {npc_type}");
            assert_eq!(rng.peek(), witness.peek(), "type {npc_type}: one roll");

            let mut rng = UnifiedRandom::new(1);
            let mut witness = UnifiedRandom::new(1);
            let _ = witness.next_max(1000);
            let mut plays: Vec<(u32, i32)> = Vec::new();
            fighter_ambient_moans(npc_type, 0.0, true, 0.0, &mut rng, &mut |c, s, _| {
                plays.push((c, s))
            });
            assert!(plays.is_empty(), "type {npc_type}");
            assert_eq!(rng.peek(), witness.peek(), "type {npc_type}: failed roll consumed");
        }
    }

    /// S1-E-21e — `fighter_not_discouraged`'s table
    /// (`NPC.cs:60300-60330`): the `:60302` condition, the switch's
    /// case-list / critter-set default and the `:60329` early-exit tail.
    /// The excluded moan types are NOT discouraged even by day above the
    /// surface — the Lead-review witness.
    #[test]
    fn s1_e_21e_the_not_discouraged_gate_follows_the_c_table() {
        // A plain scene: no eclipse, no statue, above the surface (y 100 <
        // surface 800), no graveyard, no events, ai[1]/ai[2] at 0. Only
        // `day` varies unless a case overrides.
        let gate = |npc_type: i32, day: bool| {
            fighter_not_discouraged(
                npc_type, 100.0, 800.0, false, day, false, false, false, 0, false, 0.0, 0.0,
            )
        };
        // The zombie 3 / skeleton 21: the condition holds by day, the
        // switch's default returns CritterThatCanTurnOnPlayers (false for
        // both) — discouraged; night opens the gate.
        for t in [3, 21] {
            assert!(!gate(t, true), "type {t}: day above surface discourages");
            assert!(gate(t, false), "type {t}: night is not discouraged");
        }
        // The five live scene terms each open the gate by day (`:60302`).
        assert!(fighter_not_discouraged(3, 100.0, 800.0, true, true, false, false, false, 0, false, 0.0, 0.0)); // eclipse
        assert!(fighter_not_discouraged(3, 100.0, 800.0, false, true, true, false, false, 0, false, 0.0, 0.0)); // statue
        assert!(fighter_not_discouraged(3, 900.0, 800.0, false, true, false, false, false, 0, false, 0.0, 0.0)); // below surface
        assert!(fighter_not_discouraged(3, 100.0, 800.0, false, true, false, true, false, 0, false, 0.0, 0.0)); // graveyard
        // THE EXCLUDED MOAN TYPES (`:60302`'s unconditional list): the
        // condition fails, the tail returns TRUE — not discouraged even on
        // a plain day above the surface. The mummies 78-80/630 ride moan
        // arm 3, the face monster 181 arm 6, 110/31/77/291-296 arm 1.
        for t in [31, 77, 78, 79, 80, 110, 181, 291, 292, 293, 294, 295, 296, 630] {
            assert!(gate(t, true), "excluded type {t}: not discouraged by day");
        }
        // The event terms: each fails the condition for its own ids BY
        // DAY (where the plain scene otherwise discourages them — the
        // same ids without their event assert false below).
        assert!(fighter_not_discouraged(343, 100.0, 800.0, false, true, false, false, true, 0, false, 0.0, 0.0)); // snow moon
        assert!(fighter_not_discouraged(26, 100.0, 800.0, false, true, false, false, false, 1, false, 0.0, 0.0)); // goblin invasion
        assert!(fighter_not_discouraged(212, 100.0, 800.0, false, true, false, false, false, 3, false, 0.0, 0.0)); // pirate invasion
        assert!(fighter_not_discouraged(381, 100.0, 800.0, false, true, false, false, false, 4, false, 0.0, 0.0)); // martian invasion
        assert!(fighter_not_discouraged(163, 100.0, 800.0, false, true, false, false, false, 0, true, 0.0, 0.0)); // don't starve
        // …and the same ids WITHOUT their event fall to the default arm
        // (false — none is in the critter set): the day gate applies.
        for t in [26, 212, 381, 163, 343] {
            assert!(!gate(t, true), "type {t} without its event: day discourages");
        }
        // The 631 `ai[2]` arm and the 411 `ai[1]` band (`:60302`), by day
        // so the term is the only difference.
        assert!(fighter_not_discouraged(631, 100.0, 800.0, false, true, false, false, false, 0, false, 0.0, 1.0));
        assert!(!fighter_not_discouraged(631, 100.0, 800.0, false, true, false, false, false, 0, false, 0.0, 0.0));
        assert!(fighter_not_discouraged(411, 100.0, 800.0, false, true, false, false, false, 0, false, 200.0, 0.0)); // >= 180
        assert!(fighter_not_discouraged(411, 100.0, 800.0, false, true, false, false, false, 0, false, 50.0, 0.0)); // < 90
        assert!(!fighter_not_discouraged(411, 100.0, 800.0, false, true, false, false, false, 0, false, 100.0, 0.0)); // in-band
        // The switch's default: a critter-set member returns TRUE by day
        // (`NPCID.cs:10436`), the case-listed and 528-532 types break to
        // the tail's TRUE.
        assert!(gate(299, true), "the critter set returns true");
        assert!(gate(425, true), "the case list breaks to true");
        assert!(gate(528, true), "the 528-532 break arm");
    }
}
