//! W4-d54 batch 4 — the Main.cs world dust sites 30-40: `Main.snowing()`
//! (`Main.cs:13032-13116`, site 13088) and `Main.TeleportEffect`'s ten
//! dust styles (`:13233-13463`, sites 13243/13262/13280/13290/13324/
//! 13340/13358/13390/13438/13452). Style 9 (`:13376`) routes to
//! `TeleportPylonsSystem.SpawnInWorldDust` — the pylon lane's, not a
//! Main.cs site (the census doc).
//!
//! RNG slots: `snowing()` runs in `Main.DoUpdate` (`:17238`) OUTSIDE
//! every `SwapRandom` block — the bare `Main.rand`, the port's
//! `NamedRngs::global`. `TeleportEffect` runs from `Player.Teleport`
//! (`Player.cs:37444`/`:37489`) under `UpdatePlayers`. Both bodies take
//! the stream as a parameter; the caller picks the slot.
//!
//! LATENT bodies (the claim doc): the port's snowfall is the rain.rs
//! `Snowflake` stand-in (the recorded M1.4-WEATHER-2 deviation — wiring
//! this body live would double-spawn and touch the frozen rain stream),
//! and `Player.Teleport`'s `TeleportEffect` call is the recall_teleport
//! module's cited follow-up (`recall_teleport.rs:36`/`:88`). The bodies
//! are ported verbatim ahead of those lanes; the sounds
//! (`PlaySound(Item6/Item8/19)`) stay with the callers, exactly as the
//! C# interleaves them (cited per style below).

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// `SceneMetrics.SnowTileMax` (`SceneMetrics.cs:46`, set 6000 at
/// `:932`).
const SNOW_TILE_MAX: f32 = 6000.0;

/// The `Main` statics `snowing()` reads, gathered by the caller (the
/// `ThunderInputs` precedent — the port keeps them on App/rain/wind).
#[derive(Clone, Copy)]
pub(crate) struct SnowingEnv {
    /// `Main.remixWorld` (`:13034` — the early return).
    pub remix_world: bool,
    /// `Main.Camera.ScaledPosition` / `ScaledSize` (`:13038-13039`).
    pub scaled_position: Vec2,
    pub scaled_size: Vec2,
    /// `Main.gamePaused` (`:13040`).
    pub game_paused: bool,
    /// `Main.SceneMetrics.SnowTileCount` / `SurfaceAtmospherics`.
    pub snow_tile_count: i32,
    pub surface_atmospherics: bool,
    /// `Main.cloudAlpha`.
    pub cloud_alpha: f32,
    /// `Main.maxScreenW` (`Main.cs:66715` — 1920).
    pub max_screen_w: i32,
    /// `NPC.IsADeerclopsNearScreen()` (`:13047`).
    pub deerclops_near: bool,
    /// `Main.snowDust` — the live type-76 census (`:13057`).
    pub snow_dust: i32,
    /// `Main.gfxQuality` (`:13057`).
    pub gfx_quality: f32,
    /// `Main.player[Main.myPlayer].velocity.Y` (`:13067`).
    pub player_vy: f32,
    /// `Main.raining` (`:13092`).
    pub raining: bool,
    /// `Main.windSpeedCurrent` (`:13094`).
    pub wind_speed_current: f32,
}

/// `Main.snowing()` (`Main.cs:13032-13116`): the ambient snowfall's
/// type-76 spawn. The whole body rides the bare `Main.rand` (the
/// caller's `global` stream). The per-flake post chain keeps the C#'s
/// exact write order — the scale gets `cloudAlpha*0.2` TWICE
/// (`:13089` and `:13103`), the velocity.Y is overwritten then scaled
/// by the scale, and the final `velocity *= 1 + cloudAlpha*0.5` hits
/// both axes.
pub(crate) fn snowing(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    env: &SnowingEnv,
) {
    if env.remix_world {
        return;
    }
    if env.game_paused || env.snow_tile_count <= 0 || !env.surface_atmospherics {
        return;
    }
    // `:13044-13046` — the spawn probability + the per-second budget.
    // `Math.Pow` is DOUBLE: the float base/exponent widen, the result
    // narrows back (`(float)Math.Pow(...)`).
    let mut num = ((env.snow_tile_count as f32 / SNOW_TILE_MAX) as f64)
        .powf((4.0f32 - env.cloud_alpha * 3.0) as f64) as f32;
    let mut num2 = ((500.0f32 * (env.scaled_size.x / env.max_screen_w as f32)) as i32 as f32
        * (1.0 + 2.0 * env.cloud_alpha)) as i32;
    let num3 = 1.0 + 50.0 * env.cloud_alpha;
    let flag = env.deerclops_near;
    if flag {
        num /= 20.0;
        num2 /= 3;
    }
    let mut i = 0.0f32;
    while i < num3 {
        // `:13057-13060` — the live-count cap BREAKS the loop.
        if !(env.snow_dust as f32
            < num2 as f32 * (env.gfx_quality / 2.0 + 0.5) + num2 as f32 * 0.1)
        {
            break;
        }
        // `:13061-13064` — the probability gate CONTINUES (the draw
        // still rides the stream).
        if !(rng.next_float() < num) {
            i += 1.0;
            continue;
        }
        // `:13065-13083` — the position rolls.
        let mut num4 = rng.next_max(env.scaled_size.x as i32 + 1500) - 750;
        let mut num5 = env.scaled_position.y as i32 - rng.next_max(50);
        if env.player_vy > 0.0 {
            num5 -= env.player_vy as i32;
        }
        if rng.next_max(5) == 0 {
            num4 = rng.next_max(500) - 500;
        } else if rng.next_max(5) == 0 {
            num4 = rng.next_max(500) + env.scaled_size.x as i32;
        }
        if num4 < 0 || num4 as f32 > env.scaled_size.x {
            num5 += rng.next_max((env.scaled_size.y as f64 * 0.8) as i32)
                + (env.scaled_size.y as f64 * 0.1) as i32;
        }
        num4 += env.scaled_position.x as i32;
        // `:13084-13087` — the open-air tile gate.
        let num6 = num4 / 16;
        let num7 = num5 / 16;
        let open = world
            .tile(num6, num7)
            .is_some_and(|t| !t.active() && t.wall == 0);
        if world.in_bounds(num6, num7) && open {
            // `:13088-13108` — the spawn + the post chain.
            let idx = dust.new_dust(
                rng, ctx, world, Vec2::new(num4 as f32, num5 as f32), 10, 10, 76,
                0.0, 0.0, 0, Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.scale += env.cloud_alpha * 0.2;
            d.velocity.y = 3.0 + rng.next_max(30) as f32 * 0.1;
            d.velocity.y *= d.scale;
            if !env.raining {
                d.velocity.x = env.wind_speed_current + rng.next_range(-10, 10) as f32 * 0.1;
                d.velocity.x += env.wind_speed_current * 15.0;
            } else {
                // `Math.Sqrt(Math.Abs(wind))` is double (`:13099`);
                // `Math.Sign(0f) == 0` — the sqrt(0) factor makes the
                // signum-vs-Sign zero edge observation-equal.
                d.velocity.x = ((env.wind_speed_current.abs() as f64).sqrt() as f32)
                    * env.wind_speed_current.signum()
                    * (env.cloud_alpha + 0.5)
                    * 10.0
                    + rng.next_float() * 0.2
                    - 0.1;
                d.velocity.y *= 0.5;
            }
            d.velocity.y *= 1.0 + 0.3 * env.cloud_alpha;
            d.scale += env.cloud_alpha * 0.2; // the SECOND add (`:13103`)
            if flag {
                d.scale -= 0.5;
            }
            d.velocity *= 1.0 + env.cloud_alpha * 0.5;
        }
        i += 1.0;
    }
}

/// `Main.hslToRgb` (`Main.cs:47890-47917`) as XNA bytes — the shared
/// `tile_light::hsl_to_rgb` normalises to floats, but the dust colour
/// needs the exact `(byte)Math.Round(c * 255.0)` (banker's rounding,
/// `hue2rgb` `:47919-47944`).
fn hsl_to_rgb_bytes(hue: f32, saturation: f32, luminosity: f32, a: u8) -> Color {
    let round = |v: f64| -> u8 { v.round_ties_even() as u8 };
    if saturation == 0.0 {
        let g = round(luminosity as f64 * 255.0);
        return Color::new(g, g, g, a);
    }
    let hue2rgb = |mut c: f64, t1: f64, t2: f64| -> f64 {
        if c < 0.0 {
            c += 1.0;
        }
        if c > 1.0 {
            c -= 1.0;
        }
        if 6.0 * c < 1.0 {
            return t1 + (t2 - t1) * 6.0 * c;
        }
        if 2.0 * c < 1.0 {
            return t2;
        }
        if 3.0 * c < 2.0 {
            return t1 + (t2 - t1) * (2.0 / 3.0 - c) * 6.0;
        }
        t1
    };
    let num = hue as f64;
    let l = luminosity as f64;
    let s = saturation as f64;
    let num2 = if !(l < 0.5) { l + s - l * s } else { l * (1.0 + s) };
    let t = 2.0 * l - num2;
    let r = round(hue2rgb(num + 1.0 / 3.0, t, num2) * 255.0);
    let g = round(hue2rgb(num, t, num2) * 255.0);
    let b = round(hue2rgb(num - 1.0 / 3.0, t, num2) * 255.0);
    Color::new(r, g, b, a)
}

/// `Utils.NextVector2FromRectangle` (`Utils.cs:1256-1259`) — two
/// `NextFloat` draws, X + f*width / Y + f*height.
fn next_vector2_from_rectangle(rng: &mut UnifiedRandom, x: f32, y: f32, w: f32, h: f32) -> Vec2 {
    let fx = rng.next_float();
    let fy = rng.next_float();
    Vec2::new(x + fx * w, y + fy * h)
}

/// `PortalHelper.GetPortalColor(colorIndex)`
/// (`GameContent/PortalHelper.cs`, the `netMode == 0` single-player
/// arm): `colorIndex / 2` = player, `% 2` = portal; portal 0 →
/// `hslToRgb(0.12, 1, 0.5)`, portal 1 → `hslToRgb(0.52, 1, 0.6)`;
/// alpha 66.
fn portal_color(color_index: i32) -> Color {
    let portal = color_index % 2;
    let mut c = if portal != 0 {
        hsl_to_rgb_bytes(0.52, 1.0, 0.6, 255)
    } else {
        hsl_to_rgb_bytes(0.12, 1.0, 0.5, 255)
    };
    c.a = 66;
    c
}

/// `Main.TeleportEffect(effectRect, Style, extraInfo, dustCountMult,
/// side, otherPosition)` (`Main.cs:13233-13463`) — the dust half of the
/// ten ported styles. `side` selects nothing in the dust arms (the C#
/// reads it only in the unported pylon style 9), so it is not a
/// parameter. The sounds stay with the caller (cited per style). The
/// rect is `(x, y, w, h)` in world px.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub(crate) fn teleport_effect(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    rect: (i32, i32, i32, i32),
    style: i32,
    extra_info: i32,
    dust_count_mult: f32,
    other_position: Vec2,
    water_style: i32,
) {
    let (rx, ry, rw, rh) = rect;
    let top_left = Vec2::new(rx as f32, ry as f32);
    match style {
        // `:13237-13255` — style 0: PlaySound(Item6) FIRST (the
        // caller's), then (w*h/5)*mult type-159 puffs; the scale is
        // OVERWRITTEN by Next(20,70)*0.01, +0.25 for j<10, +0.25 for
        // j<5.
        0 => {
            let count = ((rw * rh / 5) as f32 * dust_count_mult) as i32;
            for j in 0..count {
                let idx = dust.new_dust(
                    rng, ctx, world, top_left, rw, rh, 159, 0.0, 0.0, 0, Color::default(), 1.0,
                );
                let d = dust.slot_mut(idx);
                d.scale = rng.next_range(20, 70) as f32 * 0.01;
                if j < 10 {
                    d.scale += 0.25;
                }
                if j < 5 {
                    d.scale += 0.25;
                }
            }
        }
        // `:13256-13274` — style 1: PlaySound(Item8) first, the same
        // shape with type 164.
        1 => {
            let count = ((rw * rh / 5) as f32 * dust_count_mult) as i32;
            for j in 0..count {
                let idx = dust.new_dust(
                    rng, ctx, world, top_left, rw, rh, 164, 0.0, 0.0, 0, Color::default(), 1.0,
                );
                let d = dust.slot_mut(idx);
                d.scale = rng.next_range(20, 70) as f32 * 0.01;
                if j < 10 {
                    d.scale += 0.25;
                }
                if j < 5 {
                    d.scale += 0.25;
                }
            }
        }
        // `:13275-13283` — style 2: NO sound; 50*mult type-58
        // GhostWhite puffs (alpha 150, scale 1.2), velocity *= 0.5.
        2 => {
            let count = (50.0f32 * dust_count_mult) as i32;
            for _ in 0..count {
                let idx = dust.new_dust(
                    rng, ctx, world, top_left, rw, rh, 58, 0.0, 0.0, 150,
                    Color::new(248, 248, 255, 255), 1.2,
                );
                dust.slot_mut(idx).velocity *= 0.5;
            }
        }
        // `:13284-13317` — style 3: PlaySound(Item6) first; 50*mult
        // type-180 puffs, noGravity, then the five Next(3) damp rolls,
        // the two Next(3) double arms, the Next(2) fadeIn/scale
        // overwrite arm, and the final scale *= 0.8.
        3 => {
            let count = (50.0f32 * dust_count_mult) as i32;
            for _ in 0..count {
                let idx = dust.new_dust(
                    rng, ctx, world, top_left, rw, rh, 180, 0.0, 0.0, 0, Color::default(), 1.0,
                );
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                for _ in 0..5 {
                    if rng.next_max(3) == 0 {
                        d.velocity *= 0.75;
                    }
                }
                if rng.next_max(3) == 0 {
                    d.velocity *= 2.0;
                    d.scale *= 1.2;
                }
                if rng.next_max(3) == 0 {
                    d.velocity *= 2.0;
                    d.scale *= 1.2;
                }
                if rng.next_max(2) == 0 {
                    d.fade_in = rng.next_range(75, 100) as f32 * 0.01;
                    d.scale = rng.next_range(25, 75) as f32 * 0.01;
                }
                d.scale *= 0.8;
            }
        }
        // `:13318-13333` — style 4: PlaySound(Item8) first; (w*h/5)*
        // mult type-263 puffs, colour GetPortalColor(extraInfo) with
        // A forced 255, noLight + noGravity, scale 1.2, fadeIn 0.4.
        4 => {
            let count = ((rw * rh / 5) as f32 * dust_count_mult) as i32;
            for _ in 0..count {
                let idx = dust.new_dust(
                    rng, ctx, world, top_left, rw, rh, 263, 0.0, 0.0, 0, Color::default(), 1.0,
                );
                let d = dust.slot_mut(idx);
                d.color = portal_color(extra_info);
                d.no_light = true;
                d.no_gravity = true;
                d.scale = 1.2;
                d.fade_in = 0.4;
                d.color.a = 255;
            }
        }
        // `:13334-13351` — style 5: NO leading sound; 100*mult
        // dustWater() Direct puffs (box h+24), the velocity/scale/
        // alpha/fadeIn overwrite chain; PlaySound(19) AFTER the loop.
        5 => {
            let count = (100.0f32 * dust_count_mult) as i32;
            let ty = tr_core::particles::dust_water(water_style);
            for _ in 0..count {
                let idx = dust.new_dust_direct(
                    rng, ctx, world, top_left, rw, rh + 24, ty, 0.0, 0.0, 0,
                    Color::default(), 1.0,
                );
                let d = dust.slot_mut(idx);
                d.velocity.y *= 0.0;
                d.velocity.y -= 3.5;
                d.velocity.x *= 1.5;
                d.scale = 0.8;
                d.alpha = 130;
                d.no_gravity = true;
                d.fade_in = 1.2;
            }
        }
        // `:13352-13369` — style 7: the style-5 shape with type 35,
        // 50*mult; PlaySound(Item8) after the loop.
        7 => {
            let count = (50.0f32 * dust_count_mult) as i32;
            for _ in 0..count {
                let idx = dust.new_dust_direct(
                    rng, ctx, world, top_left, rw, rh + 24, 35, 0.0, 0.0, 0,
                    Color::default(), 1.0,
                );
                let d = dust.slot_mut(idx);
                d.velocity.y *= 0.0;
                d.velocity.y -= 3.5;
                d.velocity.x *= 1.5;
                d.scale = 0.8;
                d.alpha = 130;
                d.no_gravity = true;
                d.fade_in = 1.2;
            }
        }
        // `:13381-13418` — style 10: the rect INFLATES 15px per side
        // first; 60*mult type-267 hsl puffs. Per iteration the C#
        // evaluates fadeIn/scale locals FIRST (two NextFloat), then
        // the named-argument newColor (one NextFloat for the hue)
        // BEFORE the spawn. The scale ladder runs, the 80% velocity
        // push draws a NextFloat, then scale/fadeIn are OVERWRITTEN
        // by the locals; a non-sentinel spawn clones (CloneDust's own
        // NewDust block) with scale/fadeIn ×0.65 + white.
        // PlaySound(Item8) after the loop.
        10 => {
            let (rx, ry, rw, rh) = (rx - 15, ry - 15, rw + 30, rh + 30);
            let top_left = Vec2::new(rx as f32, ry as f32);
            let count = (60.0f32 * dust_count_mult) as i32;
            let vector = other_position - top_left;
            for n in 0..count {
                let fade_in = 0.4 + rng.next_float();
                let scale = 0.4 + rng.next_float();
                let hue = 0.66 + rng.next_float() * 0.24;
                let new_color = hsl_to_rgb_bytes(hue, 1.0, 0.5, 255);
                let idx = dust.new_dust_direct(
                    rng, ctx, world, top_left, rw, rh, 267, 0.0, 0.0, 127, new_color, 1.0,
                );
                let d = dust.slot_mut(idx);
                d.scale = rng.next_range(20, 70) as f32 * 0.01;
                if n < 10 {
                    d.scale += 0.25;
                }
                if n < 5 {
                    d.scale += 0.25;
                }
                if (n as f32) < count as f32 * 0.8 {
                    d.velocity += vector * 0.1 * rng.next_float();
                }
                d.no_gravity = true;
                d.no_light = true;
                d.scale = scale;
                d.fade_in = fade_in;
                if d.dust_index != 6000 {
                    let clone = dust.clone_dust(rng, ctx, world, idx);
                    let c = dust.slot_mut(clone);
                    c.scale *= 0.65;
                    c.fade_in *= 0.65;
                    c.color = Color::new(255, 255, 255, 255);
                }
            }
        }
        // `:13419-13446` — style 11: NO sound; 50 Perfect type-267
        // puffs at NextVector2FromRectangle (two NextFloat), the
        // Next(4) colour pick FIRST (Green default, 0/1 → (100,255,
        // 100), 2 → Yellow, 3 → White), velocity ×2, scale 0.8 +
        // NextFloat*0.6, fadeIn 0.5, noGravity.
        11 => {
            for _ in 0..50 {
                let num4 = rng.next_max(4);
                let color = match num4 {
                    0 | 1 => Color::new(100, 255, 100, 255),
                    2 => Color::new(255, 255, 0, 255),
                    3 => Color::new(255, 255, 255, 255),
                    _ => Color::new(0, 128, 0, 255), // Color.Green
                };
                let pos = next_vector2_from_rectangle(
                    rng, rx as f32, ry as f32, rw as f32, rh as f32,
                );
                let idx = dust.new_dust_perfect(
                    rng, ctx, world, pos, 267, None, 0, Color::default(), 1.0,
                );
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.color = color;
                d.velocity *= 2.0;
                d.scale = 0.8 + rng.next_float() * 0.6;
                d.fade_in = 0.5;
            }
        }
        // `:13447-13462` — style 13: PlaySound(Item8) first; 21
        // type-27 puffs at NextVector2FromRectangle (2×2 box), the
        // scale ARGUMENT Next(1,3) evaluates after the position
        // floats; velocity ×1.75, ×1.5 more on i%3==0,
        // noLightEmittance + noGravity.
        13 => {
            for i in 0..21 {
                let pos = next_vector2_from_rectangle(
                    rng, rx as f32, ry as f32, rw as f32, rh as f32,
                );
                let scale = rng.next_range(1, 3) as f32;
                let idx = dust.new_dust(
                    rng, ctx, world, pos, 2, 2, 27, 0.0, 0.0, 100, Color::default(), scale,
                );
                let d = dust.slot_mut(idx);
                d.velocity *= 1.75;
                if i % 3 == 0 {
                    d.velocity *= 1.5;
                }
                d.no_light_emittance = true;
                d.no_gravity = true;
            }
        }
        // Styles 6/8/12 and the default have no dust arms in the C#
        // switch; style 9 is the pylon lane's (module doc).
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    //! W4-d54 batch-4 dust witnesses — WRITTEN, NOT RUN (code-only).

    use super::*;

    fn dust_ctx(w: &World) -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(w.width as f32 * 16.0, w.height as f32 * 16.0),
            max_dust_to_draw: 6000,
        }
    }

    /// The default block + the per-type tweak family (`Dust.cs:214-
    /// 252`); none of the batch-4 types (76/159/164/58/180/263/33/35/
    /// 267/27) draws a tweak roll.
    fn block(r: &mut UnifiedRandom, box_w: i32, box_h: i32) {
        r.next_max(box_w.max(5) - 4);
        r.next_max(box_h.max(5) - 4);
        r.next_range(-20, 21);
        r.next_range(-20, 21);
        r.next_max(3);
        r.next_range(-20, 21);
    }

    fn drain(dust: &mut DustPool) {
        for s in dust.slots_mut() {
            s.active = false;
        }
    }

    fn env(snow_dust: i32) -> SnowingEnv {
        SnowingEnv {
            remix_world: false,
            scaled_position: Vec2::new(1000.0, 800.0),
            scaled_size: Vec2::new(1920.0, 1080.0),
            game_paused: false,
            snow_tile_count: 3000,
            surface_atmospherics: true,
            cloud_alpha: 0.5,
            max_screen_w: 1920,
            deerclops_near: false,
            snow_dust,
            gfx_quality: 1.0,
            player_vy: 0.0,
            raining: false,
            wind_speed_current: 0.2,
        }
    }

    /// `Main.cs:13053-13108` — the snowing draw census: per iteration
    /// the NextFloat gate, the two position rolls, the two Next(5)
    /// side gates (the second draws ONLY when the first fails), the
    /// off-screen Next, then on an open tile the 10×10 block + the
    /// Next(30) vy roll + the Next(-10,10) vx roll. The cap BREAK and
    /// the gate CONTINUE keep the C# shape.
    #[test]
    fn w4_d54_snowing_draws_match_the_oracle() {
        let w = World::new("t".into(), 400, 300);
        let ctx = dust_ctx(&w);
        for seed in [54, 1, 99] {
            for raining in [false, true] {
                let mut e = env(0);
                e.raining = raining;
                let mut rng = UnifiedRandom::new(seed);
                let mut probe = UnifiedRandom::new(seed);
                let mut dust = DustPool::new();
                snowing(&mut rng, &mut dust, &ctx, &w, &e);
                // Oracle replay.
                let num = ((e.snow_tile_count as f32 / SNOW_TILE_MAX) as f64)
                    .powf((4.0f32 - e.cloud_alpha * 3.0) as f64) as f32;
                let num2 = ((500.0f32 * (e.scaled_size.x / e.max_screen_w as f32)) as i32
                    as f32 * (1.0 + 2.0 * e.cloud_alpha)) as i32;
                let num3 = 1.0 + 50.0 * e.cloud_alpha;
                let mut i = 0.0f32;
                while i < num3 {
                    if !(e.snow_dust as f32
                        < num2 as f32 * (e.gfx_quality / 2.0 + 0.5) + num2 as f32 * 0.1)
                    {
                        break;
                    }
                    if !(probe.next_float() < num) {
                        i += 1.0;
                        continue;
                    }
                    let mut num4 = probe.next_max(e.scaled_size.x as i32 + 1500) - 750;
                    let mut num5 = e.scaled_position.y as i32 - probe.next_max(50);
                    if e.player_vy > 0.0 {
                        num5 -= e.player_vy as i32;
                    }
                    if probe.next_max(5) == 0 {
                        num4 = probe.next_max(500) - 500;
                    } else if probe.next_max(5) == 0 {
                        num4 = probe.next_max(500) + e.scaled_size.x as i32;
                    }
                    if num4 < 0 || num4 as f32 > e.scaled_size.x {
                        num5 += probe.next_max((e.scaled_size.y as f64 * 0.8) as i32)
                            + (e.scaled_size.y as f64 * 0.1) as i32;
                    }
                    num4 += e.scaled_position.x as i32;
                    let open = w
                        .tile(num4 / 16, num5 / 16)
                        .is_some_and(|t| !t.active() && t.wall == 0);
                    if w.in_bounds(num4 / 16, num5 / 16) && open {
                        block(&mut probe, 10, 10);
                        probe.next_max(30);
                        if !raining {
                            probe.next_range(-10, 10);
                        } else {
                            probe.next_float();
                        }
                    }
                    i += 1.0;
                }
                assert_eq!(rng.peek(), probe.peek(), "snow seed {seed} rain {raining}");
                drain(&mut dust);
            }
        }
    }

    /// The snowing gates: remix/paused/no-snow/no-atmo return with NO
    /// draws; the live-count cap breaks before the first gate draw.
    #[test]
    fn w4_d54_snowing_gates_draw_nothing() {
        let w = World::new("t".into(), 400, 300);
        let ctx = dust_ctx(&w);
        let mut dust = DustPool::new();
        for mut e in [env(0), env(0), env(0), env(0), env(100000)] {
            e.remix_world = true;
            let mut rng = UnifiedRandom::new(54);
            snowing(&mut rng, &mut dust, &ctx, &w, &e);
            assert_eq!(rng.peek(), UnifiedRandom::new(54).peek(), "remix");
            e.remix_world = false;
            e.game_paused = true;
            snowing(&mut rng, &mut dust, &ctx, &w, &e);
            assert_eq!(rng.peek(), UnifiedRandom::new(54).peek(), "paused");
            e.game_paused = false;
            e.snow_tile_count = 0;
            snowing(&mut rng, &mut dust, &ctx, &w, &e);
            assert_eq!(rng.peek(), UnifiedRandom::new(54).peek(), "no snow");
            e.snow_tile_count = 3000;
            e.surface_atmospherics = false;
            snowing(&mut rng, &mut dust, &ctx, &w, &e);
            assert_eq!(rng.peek(), UnifiedRandom::new(54).peek(), "no atmo");
        }
        // The cap break: snow_dust above the budget draws nothing.
        let e = env(100000);
        let mut rng = UnifiedRandom::new(54);
        snowing(&mut rng, &mut dust, &ctx, &w, &e);
        assert_eq!(rng.peek(), UnifiedRandom::new(54).peek(), "cap");
    }

    /// `Main.cs:13240-13253` — style 0's scale ladder: the Next(20,70)
    /// overwrite, the j<10 and j<5 bumps.
    #[test]
    fn w4_d54_teleport_style0_scale_ladder() {
        let w = World::new("t".into(), 400, 300);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        // 20x20 rect → 80 puffs.
        teleport_effect(&mut rng, &mut dust, &ctx, &w, (100, 100, 20, 20), 0, 0, 1.0, Vec2::ZERO, 0);
        let mut probe = UnifiedRandom::new(54);
        for j in 0..80 {
            block(&mut probe, 20, 20);
            let roll = probe.next_range(20, 70) as f32 * 0.01;
            let mut want = roll;
            if j < 10 {
                want += 0.25;
            }
            if j < 5 {
                want += 0.25;
            }
            let d = dust.slot(j as usize);
            assert_eq!(d.dust_type, 159);
            assert!((d.scale - want).abs() < 1e-6, "j {j}");
        }
        assert_eq!(rng.peek(), probe.peek());
    }

    /// `Main.cs:13284-13315` — style 3's full post census: 5 Next(3)
    /// damps, 2 Next(3) doubles, the Next(2) overwrite arm (2 more
    /// rolls on a pass), the final ×0.8.
    #[test]
    fn w4_d54_teleport_style3_draws_match_the_oracle() {
        let w = World::new("t".into(), 400, 300);
        let ctx = dust_ctx(&w);
        for seed in [54, 7] {
            let mut rng = UnifiedRandom::new(seed);
            let mut probe = UnifiedRandom::new(seed);
            let mut dust = DustPool::new();
            teleport_effect(&mut rng, &mut dust, &ctx, &w, (100, 100, 20, 20), 3, 0, 1.0, Vec2::ZERO, 0);
            for _ in 0..50 {
                block(&mut probe, 20, 20);
                for _ in 0..5 {
                    probe.next_max(3);
                }
                probe.next_max(3);
                probe.next_max(3);
                if probe.next_max(2) == 0 {
                    probe.next_range(75, 100);
                    probe.next_range(25, 75);
                }
            }
            assert_eq!(rng.peek(), probe.peek(), "style3 seed {seed}");
            drain(&mut dust);
        }
    }

    /// `Main.cs:13334-13366` — styles 5/7: the dustWater/35 Direct
    /// puffs (box h+24), the overwrite chain leaves vy=-3.5, vx =
    /// spawn-vx ×1.5 (after the per-type tweak), scale 0.8, alpha 130,
    /// fadeIn 1.2, noGravity.
    #[test]
    fn w4_d54_teleport_styles5_7_writes_and_draws() {
        let w = World::new("t".into(), 400, 300);
        let ctx = dust_ctx(&w);
        for (style, ty, count) in [(5, tr_core::particles::dust_water(0), 100), (7, 35, 50)] {
            let mut rng = UnifiedRandom::new(54);
            let mut probe = UnifiedRandom::new(54);
            let mut dust = DustPool::new();
            teleport_effect(&mut rng, &mut dust, &ctx, &w, (100, 100, 20, 20), style, 0, 1.0, Vec2::ZERO, 0);
            for _ in 0..count {
                block(&mut probe, 20, 44);
            }
            assert_eq!(rng.peek(), probe.peek(), "style {style}");
            // Replay the first spawn's vx: the default block's roll,
            // then the type tweak (33: ×0.5; 35: ×0.1), then ×1.5.
            let mut rp = UnifiedRandom::new(54);
            rp.next_max(16);
            rp.next_max(40);
            let vx = rp.next_range(-20, 21) as f32 * 0.1;
            let tweak = if ty == 35 { 0.1 } else { 0.5 };
            let d = dust.slot(0);
            assert_eq!(d.dust_type, ty);
            assert!((d.velocity.y - -3.5).abs() < 1e-6);
            assert!((d.velocity.x - vx * tweak * 1.5).abs() < 1e-6);
            assert!((d.scale - 0.8).abs() < 1e-6);
            assert_eq!(d.alpha, 130);
            assert!((d.fade_in - 1.2).abs() < 1e-6);
            assert!(d.no_gravity);
            drain(&mut dust);
        }
    }

    /// `Main.cs:13386-13414` — style 10's per-iteration census: the
    /// two local NextFloats, the hue NextFloat, the Direct block, the
    /// Next(20,70) scale roll, the 80% push NextFloat, then the
    /// CloneDust block (a 5×5-clamped NewDust of type 267).
    #[test]
    fn w4_d54_teleport_style10_draws_match_the_oracle() {
        let w = World::new("t".into(), 400, 300);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        teleport_effect(
            &mut rng, &mut dust, &ctx, &w, (100, 100, 20, 20), 10, 0, 1.0,
            Vec2::new(300.0, 200.0), 0,
        );
        let count = 60;
        for n in 0..count {
            probe.next_float(); // fadeIn local
            probe.next_float(); // scale local
            probe.next_float(); // hue
            block(&mut probe, 50, 50); // the inflated rect
            probe.next_range(20, 70);
            if (n as f32) < count as f32 * 0.8 {
                probe.next_float();
            }
            // CloneDust's own NewDust block (0×0 → 5-clamped, type 267).
            block(&mut probe, 0, 0);
        }
        assert_eq!(rng.peek(), probe.peek());
    }

    /// `Main.cs:13421-13443` — style 11: the Next(4) colour pick FIRST,
    /// then the two position NextFloats, then the Perfect block, then
    /// the scale NextFloat. The colour arms pinned.
    #[test]
    fn w4_d54_teleport_style11_draws_match_the_oracle() {
        let w = World::new("t".into(), 400, 300);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        teleport_effect(&mut rng, &mut dust, &ctx, &w, (100, 100, 20, 20), 11, 0, 1.0, Vec2::ZERO, 0);
        let mut saw = [false; 4];
        for _ in 0..50 {
            let pick = probe.next_max(4);
            saw[pick as usize] = true;
            probe.next_float();
            probe.next_float();
            block(&mut probe, 0, 0); // Perfect: 0×0 box
            probe.next_float(); // scale
        }
        assert_eq!(rng.peek(), probe.peek());
        assert!(saw.iter().all(|&s| s), "all four colour arms");
        // The colour writes landed.
        let d = dust.slot(0);
        assert_eq!(d.dust_type, 267);
        assert!(d.no_gravity);
        assert!((d.fade_in - 0.5).abs() < 1e-6);
    }

    /// `Main.cs:13450-13459` — style 13: the two position NextFloats,
    /// the Next(1,3) scale argument, the 2×2→5-clamped block; the
    /// velocity ×1.75 (×1.5 more on i%3==0), noLightEmittance +
    /// noGravity.
    #[test]
    fn w4_d54_teleport_style13_draws_match_the_oracle() {
        let w = World::new("t".into(), 400, 300);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        teleport_effect(&mut rng, &mut dust, &ctx, &w, (100, 100, 20, 20), 13, 0, 1.0, Vec2::ZERO, 0);
        for _ in 0..21 {
            probe.next_float();
            probe.next_float();
            probe.next_range(1, 3);
            block(&mut probe, 2, 2);
        }
        assert_eq!(rng.peek(), probe.peek());
        let d = dust.slot(0);
        assert_eq!(d.dust_type, 27);
        assert!(d.no_light_emittance && d.no_gravity);
        assert_eq!(d.alpha, 100);
    }

    /// `Main.hslToRgb` byte rounding + the portal colour arms.
    #[test]
    fn w4_d54_hsl_and_portal_colors() {
        // Pure red / green / blue at the primary hues.
        assert_eq!(hsl_to_rgb_bytes(0.0, 1.0, 0.5, 255), Color::new(255, 0, 0, 255));
        assert_eq!(hsl_to_rgb_bytes(1.0 / 3.0, 1.0, 0.5, 255), Color::new(0, 255, 0, 255));
        // The saturation-0 grey arm.
        assert_eq!(hsl_to_rgb_bytes(0.3, 0.0, 0.5, 255), Color::new(128, 128, 128, 255));
        // Portal 0 (orange-ish) vs portal 1 (cyan-ish), alpha 66.
        let p0 = portal_color(0);
        let p1 = portal_color(1);
        assert_eq!(p0.a, 66);
        assert_eq!(p1.a, 66);
        assert_eq!(p0, hsl_to_rgb_bytes(0.12, 1.0, 0.5, 66));
        assert_eq!(p1, hsl_to_rgb_bytes(0.52, 1.0, 0.6, 66));
        // The player index does not change the single-player colour.
        assert_eq!(portal_color(2), p0);
        assert_eq!(portal_color(3), p1);
    }
}
