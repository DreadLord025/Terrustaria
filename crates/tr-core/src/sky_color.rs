//! `Main.SetBackColor` (`Main.cs:63452-63924`) — the whole sky-colour chain in
//! XNA byte space: the day/night ramps, the blood-moon night, the rain and
//! graveyard dims, the four biome influences, the moon-phase light floor, the
//! solar-eclipse tint, the remix override and the lightning flash, ending in
//! `ModifyHorizonLight`. The pure outputs are [`BackColors`]; the caller
//! (`app/frame/draw.rs`) applies `ApplyColorOfTheSkiesToTiles` and
//! `UpdateAtmosphereTransparencyToSkyColor` after it, exactly like `Main.Draw`
//! (`Main.cs:61971-61983`).
//!
//! Deliberate remainders (documented in `docs/research/79-doc-sweep-sky.md`):
//! the Don't Starve hooks (`DontStarveSeed.cs:9-59`), the aurora tint
//! (`AuroraSky.ModifyTileColor`, `Main.cs:63922`) and the menu/server flat-35
//! night (`Main.cs:63560-63565`) have no live port input yet — the DontStarve
//! arms are coded behind the world's seed flag, the other two are noted at
//! their C# sites.

use crate::World;

/// `Main.SetBackColor`'s three outputs (`out sunColor, out moonColor` +
/// `Main.ColorOfTheSkies`), as sRGB bytes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BackColors {
    pub bg: [u8; 3],
    pub sun: [u8; 3],
    pub moon: [u8; 3],
    /// `Main.eclipseLight` after the step (`Main.cs:63845-63860`) — the caller
    /// stores it back (it decays across frames).
    pub eclipse_light: f32,
}

/// `InfoToSetBackColor` (`Main.cs:61971-61981`): the scene influences the draw
/// pass snapshots. The ratios are the raw `count / TileMax` floats — NOT the
/// zone thresholds (`SceneMetrics.cs:923-939`: Corruption/Crimson max 1000,
/// Jungle 700, Mushroom 160).
#[derive(Clone, Copy, Debug)]
pub struct BackColorInputs {
    pub time: f64,
    pub day: bool,
    /// `Main.moonPhase` (0 = full, `Enums/MoonPhase.cs`).
    pub moon_phase: u8,
    /// `Main.bloodMoon || SceneMetrics.BloodMoonMonolith` (`Main.cs:61979`).
    pub blood_moon: bool,
    pub eclipse: bool,
    /// The previous frame's `Main.eclipseLight`.
    pub eclipse_light: f32,
    /// `Main.cloudAlpha`.
    pub cloud_alpha: f32,
    /// `Main.atmo` (`UpdateAtmosphereTransparencyToSkyColor`).
    pub atmo: f32,
    /// `Main.GraveyardVisualIntensity`.
    pub graveyard: f32,
    /// `EvilTileCount / CorruptionTileMax` (1000).
    pub corruption: f32,
    /// `BloodTileCount / CrimsonTileMax` (1000).
    pub crimson: f32,
    /// `JungleTileCount / JungleTileMax` (700).
    pub jungle: f32,
    /// `Main.SmoothedMushroomLightInfluence`.
    pub mushroom: f32,
    /// `Main.lightning`.
    pub lightning: f32,
    /// `Main.remixWorld` (the in-game flag; `remixWorldGen` only matters while
    /// generating, which the port completes before the frame loop).
    pub remix: bool,
    /// `Main.dontStarveWorld`.
    pub dont_starve: bool,
    /// `Lighting.NotRetro` (`Lighting.cs:63-73`): mode is neither Retro nor
    /// Trippy.
    pub not_retro: bool,
}

/// The `(byte)` cast of a float expression: C# converts via `(int)` first
/// (truncate toward zero) and the byte cast wraps mod 256. Every ramp below
/// stays inside 0..255 by construction, so the wrap never fires.
#[inline]
fn b(v: f32) -> u8 {
    (v as i32) as u8
}

/// `Main.GetMoonPhase()`'s minimal-light grade (`Main.cs:63792-63813`):
/// Full 19, three-quarters 17, half 15, quarter 13, new 11.
fn minimal_light(moon_phase: u8) -> u8 {
    match moon_phase & 7 {
        0 => 19,
        1 | 7 => 17,
        2 | 6 => 15,
        3 | 5 => 13,
        _ => 11,
    }
}

/// `DontStarveSeed.ModifyMinimumLightColorAtNight` (`DontStarveSeed.cs:22-49`):
/// every phase but full floors at 1, full at 45; a blood moon lifts to 35.
fn dont_starve_minimal_light(moon_phase: u8, blood_moon: bool) -> u8 {
    let m = if moon_phase & 7 == 0 { 45 } else { 1 };
    if blood_moon { m.max(35) } else { m }
}

/// `DontStarveSeed.ModifyNightColor` (`DontStarveSeed.cs:9-20`): off the full
/// moon the night darkens to black at midnight and back — a double
/// `Utils.Remap` lerp over the night fraction.
fn dont_starve_night(bg: [u8; 3], moon_phase: u8, time: f64) -> [u8; 3] {
    if moon_phase & 7 == 0 {
        return bg;
    }
    let remap = |v: f32, lo: f32, hi: f32| ((v - lo) / (hi - lo)).clamp(0.0, 1.0);
    let f = (time / 32400.0) as f32;
    let to_black = remap(f, 0.0, 0.5);
    let back = remap(f, 0.5, 1.0);
    std::array::from_fn(|c| {
        let darkened = bg[c] as f32 * (1.0 - to_black);
        b(darkened + (bg[c] as f32 - darkened) * back)
    })
}

/// `DontStarveSeed.FixBiomeDarkness` (`DontStarveSeed.cs:51-59`): the biome
/// tint can never BRIGHTEN a channel on the seed — each result clamps to the
/// pre-tint byte.
fn fix_biome_darkness(orig: [u8; 3], rgb: [i32; 3], dont_starve: bool) -> [u8; 3] {
    std::array::from_fn(|c| {
        let v = rgb[c];
        if dont_starve {
            (v.min(orig[c] as i32)) as u8
        } else {
            v as u8
        }
    })
}

/// `Main.SetBackColor` (`Main.cs:63452-63924`), verbatim in order. The horizon
/// blend (`:63921`) runs AFTER this in the caller — it needs the gradient
/// tables that live in `render::sky`.
pub fn set_back_color(i: &BackColorInputs) -> BackColors {
    let num = i.time;
    let mut bg = [255u8; 3];
    let mut sun = [255u8; 3];
    let mut moon = [255u8; 3];
    // --- the day ramps (`:63460-63492`). The first `if` is NOT chained to the
    // second pair, so a time inside both windows would take both (unreachable
    // on the 0..54000 day).
    if i.day {
        if num < 13500.0 {
            let t = (num / 13500.0) as f32;
            sun = [b(t * 200.0 + 55.0), b(t * 180.0 + 75.0), b(t * 250.0 + 5.0)];
            bg = [b(t * 230.0 + 25.0), b(t * 220.0 + 35.0), b(t * 220.0 + 35.0)];
        }
        if num > 45900.0 {
            let t = (1.0 - (num / 54000.0 - 0.85) * 6.666_666_666_666_667) as f32;
            sun = [b(t * 120.0 + 55.0), b(t * 100.0 + 25.0), b(t * 120.0 + 55.0)];
            bg = [b(t * 200.0 + 35.0), b(t * 85.0 + 35.0), b(t * 135.0 + 35.0)];
        } else if num > 37800.0 {
            let t = (1.0 - (num / 54000.0 - 0.7) * 6.666_666_666_666_667) as f32;
            sun = [b(t * 80.0 + 175.0), b(t * 130.0 + 125.0), b(t * 100.0 + 155.0)];
            bg = [b(t * 20.0 + 235.0), b(t * 135.0 + 120.0), b(t * 85.0 + 170.0)];
        }
    }
    // --- the night ramps (`:63493-63545`).
    if !i.day {
        if i.blood_moon {
            if num < 16200.0 {
                let t = (1.0 - num / 16200.0) as f32;
                moon = [b(t * 10.0 + 205.0), b(t * 170.0 + 55.0), b(t * 200.0 + 55.0)];
                bg = [b(40.0 - t * 40.0 + 35.0), b(t * 20.0 + 15.0), b(t * 20.0 + 15.0)];
            } else {
                let t = ((num / 32400.0 - 0.5) * 2.0) as f32;
                // `:63510-63515`: the C# writes the pale ramp and then
                // OVERWRITES it with the red one — the second triple wins.
                moon = [b(t * 50.0 + 205.0), b(t * 100.0 + 155.0), b(t * 100.0 + 155.0)];
                moon = [b(t * 10.0 + 205.0), b(t * 170.0 + 55.0), b(t * 200.0 + 55.0)];
                bg = [b(40.0 - t * 40.0 + 35.0), b(t * 20.0 + 15.0), b(t * 20.0 + 15.0)];
            }
        } else if num < 16200.0 {
            let t = (1.0 - num / 16200.0) as f32;
            moon = [b(t * 10.0 + 205.0), b(t * 70.0 + 155.0), b(t * 100.0 + 155.0)];
            bg = [b(t * 30.0 + 5.0), b(t * 30.0 + 5.0), b(t * 30.0 + 5.0)];
        } else {
            let t = ((num / 32400.0 - 0.5) * 2.0) as f32;
            moon = [b(t * 50.0 + 205.0), b(t * 100.0 + 155.0), b(t * 100.0 + 155.0)];
            bg = [b(t * 20.0 + 5.0), b(t * 30.0 + 5.0), b(t * 30.0 + 5.0)];
        }
        if i.dont_starve {
            bg = dont_starve_night(bg, i.moon_phase, num);
        }
    }
    // --- rain dim (`:63546-63552`), graveyard dim (`:63553-63559`).
    if i.cloud_alpha > 0.0 && !i.remix {
        let n = 1.0 - i.cloud_alpha * 0.9 * i.atmo;
        bg = bg.map(|c| b(c as f32 * n));
    }
    if i.graveyard > 0.0 && !i.remix {
        let n = 1.0 - i.graveyard * 0.6;
        bg = bg.map(|c| b(c as f32 * n));
    }
    // `:63560-63565`: the menu/server night flat 35 — the port's menus draw
    // their own backdrop, no in-world caller (documented remainder).
    // --- corruption (`:63566-63616`).
    if i.corruption > 0.0 {
        let n = i.corruption.min(1.0);
        let rgb = [
            bg[0] as i32 - (90.0 * n * (bg[0] as f32 / 255.0)) as i32,
            bg[1] as i32 - (140.0 * n * (bg[1] as f32 / 255.0)) as i32,
            bg[2] as i32 - (70.0 * n * (bg[2] as f32 / 255.0)) as i32,
        ]
        .map(|v| v.max(15));
        bg = fix_biome_darkness(bg, rgb, i.dont_starve);
        let rgb = [
            sun[0] as i32 - (100.0 * n * (sun[0] as f32 / 255.0)) as i32,
            sun[1] as i32 - (100.0 * n * (sun[1] as f32 / 255.0)) as i32,
            // `:63600`: the B term multiplies by 0f — a no-op subtraction.
            sun[2] as i32 - (0.0 * n * (sun[2] as f32 / 255.0)) as i32,
        ]
        .map(|v| v.max(15));
        sun = rgb.map(|v| v as u8);
    }
    // --- crimson (`:63617-63666`). `:63627`'s R term reads the G byte —
    // verbatim.
    if i.crimson > 0.0 {
        let n = i.crimson.min(1.0);
        let rgb = [
            bg[0] as i32 - (40.0 * n * (bg[1] as f32 / 255.0)) as i32,
            bg[1] as i32 - (110.0 * n * (bg[1] as f32 / 255.0)) as i32,
            bg[2] as i32 - (140.0 * n * (bg[2] as f32 / 255.0)) as i32,
        ]
        .map(|v| v.max(15));
        bg = fix_biome_darkness(bg, rgb, i.dont_starve);
        // `:63649-63650`: only G and B drop; R keeps its value.
        let rgb = [
            sun[0] as i32,
            sun[1] as i32 - (90.0 * n * (sun[1] as f32 / 255.0)) as i32,
            sun[2] as i32 - (110.0 * n * (sun[2] as f32 / 255.0)) as i32,
        ]
        .map(|v| v.max(15));
        sun = rgb.map(|v| v as u8);
    }
    // --- jungle (`:63667-63723`): R and B drop, G untouched (the >255 clamp
    // at `:63679` is dead on a byte input).
    if i.jungle > 0.0 {
        let n = i.jungle.min(1.0);
        let rgb = [
            (bg[0] as i32 - (40.0 * n * (bg[0] as f32 / 255.0)) as i32).clamp(15, 255),
            (bg[1] as i32).clamp(15, 255),
            (bg[2] as i32 - (70.0 * n * (bg[2] as f32 / 255.0)) as i32).max(15),
        ];
        bg = fix_biome_darkness(bg, rgb, i.dont_starve);
        let rgb = [
            sun[0] as i32 - (30.0 * n * (sun[0] as f32 / 255.0)) as i32,
            sun[1] as i32,
            sun[2] as i32 - (10.0 * n * (sun[2] as f32 / 255.0)) as i32,
        ]
        .map(|v| v.max(15));
        sun = rgb.map(|v| v as u8);
    }
    // --- mushroom (`:63724-63791`): NO 1.0 cap on the influence (the smoothed
    // value is already clamped upstream). The moon's G term reads moon.R and
    // the R term reads moon.G (`:63773-63774`) — verbatim cross-reads.
    if i.mushroom > 0.0 {
        let n = i.mushroom;
        let rgb = [
            bg[0] as i32 - (250.0 * n * (bg[0] as f32 / 255.0)) as i32,
            bg[1] as i32 - (250.0 * n * (bg[1] as f32 / 255.0)) as i32,
            bg[2] as i32 - (250.0 * n * (bg[2] as f32 / 255.0)) as i32,
        ]
        .map(|v| v.max(15));
        bg = fix_biome_darkness(bg, rgb, i.dont_starve);
        let rgb = [
            sun[0] as i32 - (30.0 * n * (sun[0] as f32 / 255.0)) as i32,
            sun[1] as i32 - (10.0 * n * (sun[1] as f32 / 255.0)) as i32,
            sun[2] as i32 - (10.0 * n * (sun[2] as f32 / 255.0)) as i32,
        ]
        .map(|v| v.max(15));
        sun = rgb.map(|v| v as u8);
        let rgb = [
            moon[0] as i32 - (170.0 * n * (moon[1] as f32 / 255.0)) as i32,
            moon[1] as i32 - (140.0 * n * (moon[0] as f32 / 255.0)) as i32,
            moon[2] as i32 - (190.0 * n * (moon[2] as f32 / 255.0)) as i32,
        ]
        .map(|v| v.max(15));
        moon = rgb.map(|v| v as u8);
    }
    // --- the moon-phase floor (`:63792-63829`), then the blood-moon floor 25
    // (`:63830-63844`).
    let floor = if i.dont_starve {
        dont_starve_minimal_light(i.moon_phase, i.blood_moon)
    } else {
        minimal_light(i.moon_phase)
    };
    bg = bg.map(|c| c.max(floor));
    if i.blood_moon {
        bg = bg.map(|c| c.max(25));
    }
    // --- the eclipse step (`:63845-63898`): the light rides the day clock
    // (time / 1242, capped at 1) and decays 0.01 per call otherwise.
    let mut eclipse_light = i.eclipse_light;
    if i.eclipse && i.day {
        eclipse_light = ((num / 1242.0) as f32).min(1.0);
    } else if eclipse_light > 0.0 {
        eclipse_light -= 0.01;
        if eclipse_light < 0.0 {
            eclipse_light = 0.0;
        }
    }
    if eclipse_light > 0.0 {
        let (nr, ng, nb) = (
            1.0 - 0.925 * eclipse_light,
            1.0 - 0.96 * eclipse_light,
            1.0 - eclipse_light,
        );
        bg = [
            b(bg[0] as f32 * nr).max(20),
            b(bg[1] as f32 * ng).max(10),
            b(bg[2] as f32 * nb),
        ];
        sun = [255, 127, 67];
        if !i.not_retro {
            bg = [bg[0].max(20), bg[1].max(14), bg[2].max(6)];
        }
    }
    // --- remix override (`:63899-63904`) and the lightning flash
    // (`:63905-63916`, `MathHelper.Lerp` toward white).
    if i.remix {
        bg = [1, 1, 1];
    }
    if i.lightning > 0.0 {
        bg = bg.map(|c| {
            let v = c as f32 / 255.0;
            b((v + (1.0 - v) * i.lightning) * 255.0)
        });
    }
    // `:63917-63920`: the moon is white unless the blood moon is up — the
    // night ramps above only survive through the blood-moon arm.
    if !i.blood_moon {
        moon = [255, 255, 255];
    }
    BackColors { bg, sun, moon, eclipse_light }
}

/// `Main.SmoothedMushroomLightInfluence`'s per-draw walk
/// (`Main.cs:61945-61970`): toward `MushroomTileCount / MushroomTileMax` (160)
/// at ±0.01 while any mushroom tile is in the scan, −0.02 otherwise, clamped
/// to 0..1.
pub fn smooth_mushroom_influence(current: f32, mushroom_tile_count: i32) -> f32 {
    let target = mushroom_tile_count as f32 / 160.0;
    let mut v = current;
    if target > 0.0 {
        if target > v {
            v += 0.01;
        }
        if target < v {
            v -= 0.01;
        }
    } else {
        v -= 0.02;
    }
    v.clamp(0.0, 1.0)
}

/// `Main.ApplyColorOfTheSkiesToTiles` (`Main.cs:63175-63183`): the integer
/// `(R + G + B + 7·channel) / 10` per channel. `SkyManager.ProcessTileColor`
/// (`:63182`) has no active sky in the port.
pub fn apply_sky_to_tiles(bg: [u8; 3]) -> [u8; 3] {
    let sum = bg[0] as u32 + bg[1] as u32 + bg[2] as u32;
    std::array::from_fn(|c| ((sum + 7 * bg[c] as u32) / 10) as u8)
}

/// The colour half of `Main.UpdateAtmosphereTransparencyToSkyColor`
/// (`Main.cs:63201-63209`): `ColorOfTheSkies *= atmo` per byte, black at
/// `atmo <= 0.01`. Runs AFTER the tile colour is taken (`Main.cs:61982-61983`),
/// so the underground tiles keep the undimmed light while the sky gradient
/// fades out with depth.
pub fn atmo_dim_sky(bg: [u8; 3], atmo: f32) -> [u8; 3] {
    if atmo as f64 <= 0.01 {
        return [0, 0, 0];
    }
    bg.map(|c| b(c as f32 * atmo))
}

/// The `InfoToSetBackColor` snapshot from the live world/scene state
/// (`Main.cs:61971-61981`). `time` is the caller's (quantised) clock.
pub fn inputs_from(
    world: &World,
    metrics: &crate::scene::SceneMetrics,
    graveyard: f32,
    mushroom_influence: f32,
    eclipse_light: f32,
    cloud_alpha: f32,
    atmo: f32,
    lightning: f32,
    time: f64,
    day: bool,
    not_retro: bool,
) -> BackColorInputs {
    BackColorInputs {
        time,
        day,
        moon_phase: world.moon_phase,
        blood_moon: world.blood_moon || metrics.blood_moon_monolith,
        eclipse: world.eclipse,
        eclipse_light,
        cloud_alpha,
        atmo,
        graveyard,
        corruption: metrics.evil_tile_count as f32 / 1000.0,
        crimson: metrics.blood_tile_count as f32 / 1000.0,
        jungle: metrics.jungle_tile_count as f32 / 700.0,
        mushroom: mushroom_influence,
        lightning,
        remix: world.seeds.contains(crate::seeds::SeedFlags::REMIX),
        dont_starve: world.seeds.contains(crate::seeds::SeedFlags::DONT_STARVE),
        not_retro,
    }
}

// ---------------------------------------------------------------------------
// Witnesses — WRITTEN NOT RUN (code-only lane). Each expectation is hand-
// derived from the cited C# lines; run with `cargo test -p tr-core sky_color`.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> BackColorInputs {
        BackColorInputs {
            time: 27000.0,
            day: true,
            moon_phase: 0,
            blood_moon: false,
            eclipse: false,
            eclipse_light: 0.0,
            cloud_alpha: 0.0,
            atmo: 1.0,
            graveyard: 0.0,
            corruption: 0.0,
            crimson: 0.0,
            jungle: 0.0,
            mushroom: 0.0,
            lightning: 0.0,
            remix: false,
            dont_starve: false,
            not_retro: true,
        }
    }

    #[test]
    fn noon_is_white_everywhere() {
        let c = set_back_color(&base());
        assert_eq!(c.bg, [255, 255, 255]);
        assert_eq!(c.sun, [255, 255, 255]);
        assert_eq!(c.moon, [255, 255, 255]);
    }

    #[test]
    fn dawn_ramp_truncates_like_the_byte_cast() {
        // t = 0 (`:63462-63470`): bg (25,35,35), sun (55,75,5).
        let mut i = base();
        i.time = 0.0;
        let c = set_back_color(&i);
        assert_eq!(c.bg, [25, 35, 35]);
        assert_eq!(c.sun, [55, 75, 5]);
        // t = 0.5 (time 6750): bg (140,145,145) — 0.5*230+25 = 140 exactly.
        i.time = 6750.0;
        assert_eq!(set_back_color(&i).bg, [140, 145, 145]);
    }

    #[test]
    fn midnight_plain_night_floors_at_the_phase_minimum() {
        // time 16200, plain night (`:63531-63539`): t = 0 → (5,5,5), lifted
        // to the full-moon floor 19 (`:63810-63811`).
        let mut i = base();
        i.day = false;
        i.time = 16200.0;
        let c = set_back_color(&i);
        assert_eq!(c.bg, [19, 19, 19]);
        // The moon is WHITE outside a blood moon (`:63917-63920`).
        assert_eq!(c.moon, [255, 255, 255]);
        // New moon (phase 4) floors at 11.
        i.moon_phase = 4;
        assert_eq!(set_back_color(&i).bg, [11, 11, 11]);
    }

    #[test]
    fn blood_moon_midnight_keeps_the_red_ramp_and_floor_25() {
        // `:63507-63519` at t = 0: bg (75,15,15) — R = 40-0+35; the floor 25
        // (`:63830-63844`) lifts G/B to 25. The moon keeps the red ramp
        // (205,55,55) — the second write wins (`:63513-63515`).
        let mut i = base();
        i.day = false;
        i.time = 16200.0;
        i.blood_moon = true;
        let c = set_back_color(&i);
        assert_eq!(c.bg, [75, 25, 25]);
        assert_eq!(c.moon, [205, 55, 55]);
    }

    #[test]
    fn blood_moon_late_night_overwrite_is_the_red_triple() {
        // time 32400 → t = 1 (`:63509`): the pale triple (255,255,255) is
        // overwritten by (215,225,255); bg R = 40-40+35 = 35, G/B 35.
        let mut i = base();
        i.day = false;
        i.time = 32400.0;
        i.blood_moon = true;
        let c = set_back_color(&i);
        assert_eq!(c.moon, [215, 225, 255]);
        assert_eq!(c.bg, [35, 35, 35]);
    }

    #[test]
    fn rain_dim_multiplies_the_byte_then_truncates() {
        // Noon white, cloudAlpha 0.5, atmo 1 (`:63546-63552`):
        // n = 1 - 0.45 = 0.55 → (byte)(255 * 0.55) = 140.
        let mut i = base();
        i.cloud_alpha = 0.5;
        assert_eq!(set_back_color(&i).bg, [140, 140, 140]);
        // atmo 0.5 halves the dim term: n = 1 - 0.225 = 0.775 → 197.
        i.atmo = 0.5;
        assert_eq!(set_back_color(&i).bg, [197, 197, 197]);
    }

    #[test]
    fn graveyard_dim_stacks_after_the_rain() {
        // cloudAlpha 0 (no rain), graveyard 0.5 (`:63553-63559`):
        // n = 1 - 0.3 = 0.7 → 178.
        let mut i = base();
        i.graveyard = 0.5;
        assert_eq!(set_back_color(&i).bg, [178, 178, 178]);
    }

    #[test]
    fn corruption_tint_hits_bg_and_sun_with_the_15_floor() {
        // Noon, influence 1 (`:63566-63616`): bg 255 - (int)(90/140/70) =
        // (165,115,185); sun (155,155,255) — the B term multiplies by 0.
        let mut i = base();
        i.corruption = 1.0;
        let c = set_back_color(&i);
        assert_eq!(c.bg, [165, 115, 185]);
        assert_eq!(c.sun, [155, 155, 255]);
        // Influence > 1 caps at 1 (`:63569-63572`).
        i.corruption = 2.0;
        assert_eq!(set_back_color(&i).bg, [165, 115, 185]);
    }

    #[test]
    fn crimson_r_term_reads_the_green_byte() {
        // `:63627`: R -= 40 * n * G/255. On a (100,200,255) sky at n = 1:
        // R = 100 - 31 = 69, G = 200 - 86 = 114, B = 255 - 140 = 115.
        let mut i = base();
        i.time = 0.0; // bg (25,35,35) — too dark to show the cross-read
        i.crimson = 1.0;
        let dark = set_back_color(&i);
        // R = 25 - (int)(40*35/255) = 25 - 5 = 20; G = 35 - (int)(110*35/255)
        // = 35 - 15 = 20; B = 35 - (int)(140*35/255) = 35 - 19 = 16 — then
        // the unconditional full-moon floor 19 (`:63818-63829`) lifts B.
        assert_eq!(dark.bg, [20, 20, 19]);
    }

    #[test]
    fn jungle_leaves_green_alone() {
        // `:63677-63678`: only R and B drop. Dawn bg (25,35,35) at n = 1:
        // R = 25 - 3 = 22, G = 35, B = 35 - 9 = 26.
        let mut i = base();
        i.time = 0.0;
        i.jungle = 1.0;
        assert_eq!(set_back_color(&i).bg, [22, 35, 26]);
    }

    #[test]
    fn mushroom_cross_reads_the_moon_channels() {
        // Blood-moon midnight, influence 1 (`:63770-63775`): moon (205,55,55)
        // → G -= 140*205/255 = 112 → floor 15; R -= 170*55/255 = 36 → 169;
        // B -= 190*55/255 = 40 → 15. bg (75,25,25) → all floor 15, then the
        // blood floor lifts to 25.
        let mut i = base();
        i.day = false;
        i.time = 16200.0;
        i.blood_moon = true;
        i.mushroom = 1.0;
        let c = set_back_color(&i);
        assert_eq!(c.moon, [169, 15, 15]);
        assert_eq!(c.bg, [25, 25, 25]);
    }

    #[test]
    fn eclipse_rides_the_clock_and_decays() {
        // `:63845-63851`: time 621.0001 → light 0.50000008 (f32). The DAWN
        // ramp also runs (`:63462-63470`): t = 621.0001/13500 → bg (35,45,45).
        // The eclipse factors 0.5375/0.52/0.5 give (int) 18/23/22; R floors
        // at 20. sun (255,127,67).
        let mut i = base();
        i.eclipse = true;
        i.time = 621.0001;
        let c = set_back_color(&i);
        assert!((c.eclipse_light - 0.5).abs() < 1e-6);
        assert_eq!(c.bg, [20, 23, 22]);
        assert_eq!(c.sun, [255, 127, 67]);
        // Not retro (`:63883-63897`): G floors at 14, B at 6 — on the dawn
        // ramp. The stored light decays 0.5 → 0.49 (`:63853-63859`); factors
        // 0.54675/0.5296/0.51 on bg (25,35,35) give (int) 13/18/17 → R floors
        // at 20 (both the `:63875` and the retro `:63885` arms).
        let mut j = base();
        j.eclipse_light = 0.5;
        j.not_retro = false;
        j.time = 0.0;
        let d = set_back_color(&j);
        assert!((d.eclipse_light - 0.49).abs() < 1e-6);
        assert_eq!(d.bg, [20, 18, 17]);
    }

    #[test]
    fn remix_flattens_to_one_and_lightning_lerps_to_white() {
        let mut i = base();
        i.remix = true;
        assert_eq!(set_back_color(&i).bg, [1, 1, 1]);
        // Lightning 1.0 on the remix 1 (`:63905-63916`): the lerp lands
        // exactly on white.
        i.lightning = 1.0;
        assert_eq!(set_back_color(&i).bg, [255, 255, 255]);
        // Lightning 0.33 on the dawn ramp bg (25,35,35) (`:63905-63916`):
        // R: 25/255 = 0.098039 → lerp → 0.395686 → ×255 = 100.9 → 100;
        // G/B: 35/255 = 0.137255 → 0.421961 → 107.6 → 107.
        i.remix = false;
        i.time = 0.0;
        i.lightning = 0.33;
        assert_eq!(set_back_color(&i).bg, [100, 107, 107]);
    }

    #[test]
    fn dont_starve_floor_and_night_black() {
        // Full moon floors at 45 (`DontStarveSeed.cs:41-43`); midnight bg
        // (5,5,5) → 45 before the night-black lerp (full moon skips it).
        let mut i = base();
        i.day = false;
        i.time = 16200.0;
        i.dont_starve = true;
        assert_eq!(set_back_color(&i).bg, [45, 45, 45]);
        // New moon: floor 1, and the midnight lerp is fully black
        // (`ModifyNightColor` amount 1 at f = 0.5, back 0).
        i.moon_phase = 4;
        assert_eq!(set_back_color(&i).bg, [1, 1, 1]);
    }

    #[test]
    fn tile_colour_is_the_integer_seven_tenths_blend() {
        // `:63179-63181`: (R+G+B+7R)/10 integer division.
        assert_eq!(apply_sky_to_tiles([255, 255, 255]), [255, 255, 255]);
        assert_eq!(apply_sky_to_tiles([19, 19, 19]), [19, 19, 19]);
        // (100,50,25): sum 175 → R (175+700)/10 = 87, G 52, B 35.
        assert_eq!(apply_sky_to_tiles([100, 50, 25]), [87, 52, 35]);
    }

    #[test]
    fn atmo_dim_blacks_out_below_the_threshold() {
        assert_eq!(atmo_dim_sky([100, 100, 100], 0.01), [0, 0, 0]);
        assert_eq!(atmo_dim_sky([100, 100, 100], 0.0), [0, 0, 0]);
        assert_eq!(atmo_dim_sky([100, 200, 255], 0.5), [50, 100, 127]);
        assert_eq!(atmo_dim_sky([100, 200, 255], 1.0), [100, 200, 255]);
    }

    #[test]
    fn mushroom_influence_walks_at_the_cadence() {
        // `:61947-61961`: +0.01 toward a positive target, −0.02 to zero.
        assert!((smooth_mushroom_influence(0.0, 160) - 0.01).abs() < 1e-6);
        assert!((smooth_mushroom_influence(0.5, 0) - 0.48).abs() < 1e-6);
        // The clamp at 1 (`:61966-61969`): target 160/160 = 1 from 0.999.
        assert_eq!(smooth_mushroom_influence(0.999, 160), 1.0);
        // Below zero clamps (`:61962-61965`).
        assert_eq!(smooth_mushroom_influence(0.01, 0), 0.0);
    }
}
