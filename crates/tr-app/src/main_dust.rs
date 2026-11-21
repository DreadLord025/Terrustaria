//! W4-g11 — the Main.cs `Dust.NewDust*` sites, batch 1 of 3 (the line-order
//! thirds split of `docs/research/64-dust-main-sites-claims.md`, W4-g12's
//! claim doc: g11 owns sites 1-8, lines 13088-13358; g12 sites 9-16; g13
//! sites 17-23). This batch: `Main.snowing()` (`Main.cs:13032-13116`, the
//! one type-76 site `:13088`) and `Main.TeleportEffect`'s styles 0/1/2/3/4/
//! 5/7 (`:13233-13369`, sites `:13243`/`:13262`/`:13280`/`:13290`/`:13324`/
//! `:13340`/`:13358`). Styles 10/11/13 (`:13390`/`:13438`/`:13452`) are
//! g12's; the `DrawNPCDirect`/`DrawNPCExtras`/late-file sites are g12/g13's.
//!
//! Contract (the S1-E-1 pool, d50's `DustPool`): every draw rides the
//! caller's stream in C# statement order — the explicit `Main.rand`
//! argument draws FIRST (C# evaluates arguments before the call), then the
//! `NewDust` internals roll inside `DustPool::new_dust` (`Dust.cs:75-256`),
//! then the post-spawn writes (which read `Main.dust[6000]` when the pool
//! refuses — the sentinel slot exists and takes the writes exactly like
//! the C#). One draw = one stream step; no new streams (S1-A-10).
//!
//! RNG slots (one per C# pass): `snowing` runs at `Main.cs:17238`, BEFORE
//! the `DoUpdateInWorld` swap (`:17732`) — the bare `Main.rand`, the port's
//! `NamedRngs::global()` stand-in. `TeleportEffect` is called from
//! `Player.Teleport` (`Player.cs:37444`/`:37489`, the `UpdatePlayers`
//! swap), `NPC.Teleport` (`NPC.cs:81044`/`:81051`, the caller's NPC pass)
//! and `Wiring.Teleport` (`Wiring.cs:3154`/`:3170`, the `UpdateWorldGen`
//! swap) — the caller passes its pass's stream key.
//!
//! Wiring status (W4-g11 wiring round — BOTH seams WIRED): the snow site —
//! `world_sim::scene_scan::update_scene` calls `snowing` at the `:17238`
//! slot (before `Sandstorm.EmitDust` `:17247`, on `NamedRngs::global()`),
//! and the `rain.rs` `Snowflake` vector stand-in is retired (keep-one). The
//! type-76 physics arm (`snowDust++`, the scale lerp — `Dust.cs:1745-1763`)
//! is d50's `dust_physics.rs` lane; until it lands the flakes ride the
//! generic `UpdateDust` move/kills, and the pool's generic draw pass is the
//! draw lane's. The `TeleportEffect` styles — `Player.Teleport`
//! (`player.rs::teleport_style_with_extra` queues the Entry/Exit pair + the
//! grants; `player_teleport` below replays Entry-at-OLD, move, Exit-at-NEW)
//! drained by the player tick (`world_sim/player.rs`, UpdatePlayers) and the
//! wiring pads (`world_sim/wiring.rs`, UpdateWorldGen); `NPC.Teleport`
//! (`npc_teleport` below) drained by the wiring pads in pool order. The
//! per-tick `UpdateTeleportVisuals` decay arms are the W4-e1/NPC dust lanes'.
//! `recall_teleport` runs its own recall dust, NOT `TeleportEffect` (the
//! mirror/recall cluster calls `Spawn`, not `Teleport`). The witnesses below
//! pin the ports + the wiring. WRITTEN NOT RUN (code-only mode).

// The entry points are staged (see the wiring note above) — the witnesses
// exercise them under `cfg(test)`; the lint stays quiet until the wiring.
#![allow(dead_code)]

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

use crate::app::{App, RngKey};

/// `Main.maxScreenW` under the HiDef profile (`Main.cs:3709` — the port
/// renders HiDef; `sand.rs`'s 1920 stand-in is its own recorded deviation).
const MAX_SCREEN_W: f32 = 4096.0;

/// `SceneMetrics.SnowTileMax` (`SceneMetrics.cs:932`).
const SNOW_TILE_MAX: f32 = 6000.0;

/// The `Main` statics `Main.snowing` reads (`Main.cs:13032-13116`), gathered
/// by the (future) wiring caller. `scaled_position`/`scaled_size` are
/// `Main.Camera.ScaledPosition`/`ScaledSize` — the world-space view rect
/// (the `scene_scan.rs` convention: `camera - screen*0.5/zoom`,
/// `screen/zoom`).
pub(crate) struct SnowingEnv {
    /// `Main.remixWorld` (`:13034`) — the REMIX seed flag.
    pub remix_world: bool,
    /// `Main.gamePaused` (`:13040`).
    pub game_paused: bool,
    /// `Main.SceneMetrics.SnowTileCount` (`:13040`/`:13044`).
    pub snow_tile_count: i32,
    /// `Main.SceneMetrics.SurfaceAtmospherics` (`:13040`).
    pub surface_atmospherics: bool,
    /// `Main.cloudAlpha` (`:13044-13046` passim).
    pub cloud_alpha: f32,
    /// `Main.Camera.ScaledSize` (`:13038`).
    pub scaled_size: Vec2,
    /// `Main.Camera.ScaledPosition` (`:13039`).
    pub scaled_position: Vec2,
    /// `Main.gfxQuality` (`:13057`).
    pub gfx_quality: f32,
    /// `NPC.IsADeerclopsNearScreen()` (`:13047`, `NPC.cs:44534-44537` —
    /// `npcsFoundForCheckActive[668]`, any live Deerclops).
    pub deerclops_near: bool,
    /// `Main.player[Main.myPlayer].velocity.Y` (`:13067-13070`).
    pub player_vy: f32,
    /// `Main.raining` (`:13092`).
    pub raining: bool,
    /// `Main.windSpeedCurrent` (`:13094`/`:13099`).
    pub wind_speed_current: f32,
}

/// `Main.snowing()` (`Main.cs:13032-13116`) — the ambient snowfall's dust
/// spawn, verbatim. Runs on the bare `Main.rand` (the caller passes
/// `NamedRngs::global()` — `:17238` sits before every `SwapRandom`).
///
/// `Main.snowDust` (`Main.cs:621`) is a Main static the pool does not carry
/// (d50's note: the 76 arm's `snowDust++` is cited and skipped); the budget
/// reads the live type-76 population instead — the `SandStormCount`
/// stand-in pattern (`sand.rs:115`, `Sandstorm.cs:166`). The per-iteration
/// `try/catch` (`:13055`/`:13112-13114`) has no port equivalent (no
/// exceptions); dropped with this note.
pub(crate) fn snowing(
    env: &SnowingEnv,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    pool: &mut DustPool,
) {
    // `:13034-13037` — the remix world has no snowfall.
    if env.remix_world {
        return;
    }
    let scaled_size = env.scaled_size;
    let scaled_position = env.scaled_position;
    // `:13040-13043`.
    if env.game_paused || env.snow_tile_count <= 0 || !env.surface_atmospherics {
        return;
    }
    // `:13044` — `Math.Pow` is the f64 pow (the float args promote).
    let mut num = ((env.snow_tile_count as f32 / SNOW_TILE_MAX) as f64)
        .powf((4.0 - env.cloud_alpha * 3.0) as f64) as f32;
    // `:13045` — the inner `(int)` truncates BEFORE the storm multiplier.
    let mut num2 = ((500.0 * (scaled_size.x / MAX_SCREEN_W)) as i32 as f32
        * (1.0 + 2.0 * env.cloud_alpha)) as i32;
    // `:13046`.
    let num3 = 1.0 + 50.0 * env.cloud_alpha;
    // `:13047-13052` — the Deerclops throttle.
    let flag = env.deerclops_near;
    if flag {
        num /= 20.0;
        num2 /= 3;
    }
    let mut i = 0;
    // `:13053` — `(float)i < num3`.
    while (i as f32) < num3 {
        i += 1;
        // `:13057-13060` — the budget gate BREAKS the loop (not continue).
        let snow_dust = crate::minecart::live_type(pool, 76) as f32;
        if !(snow_dust < num2 as f32 * (env.gfx_quality / 2.0 + 0.5) + num2 as f32 * 0.1) {
            break;
        }
        // `:13061-13064`.
        if !(rng.next_float() < num) {
            continue;
        }
        // `:13065-13066`.
        let mut num4 = rng.next_max(scaled_size.x as i32 + 1500) - 750;
        let mut num5 = scaled_position.y as i32 - rng.next_max(50);
        // `:13067-13070` — the falling player drags the spawn band up.
        if env.player_vy > 0.0 {
            num5 -= env.player_vy as i32;
        }
        // `:13071-13078` — the edge pushes; the second `Next(5)` rolls only
        // when the first refused (the C# `else if`).
        if rng.next_max(5) == 0 {
            num4 = rng.next_max(500) - 500;
        } else if rng.next_max(5) == 0 {
            num4 = rng.next_max(500) + scaled_size.x as i32;
        }
        // `:13079-13082` — off-screen x spreads y over the view; the
        // `(double)` products truncate (the f32→f64 promotion is exact).
        if num4 < 0 || num4 as f32 > scaled_size.x {
            num5 += rng.next_max(((scaled_size.y as f64) * 0.8) as i32)
                + ((scaled_size.y as f64) * 0.1) as i32;
        }
        // `:13083-13086` — the world-px offset + the open-air tile gate.
        num4 += scaled_position.x as i32;
        let num6 = num4 / 16;
        let num7 = num5 / 16;
        let open = world
            .tile(num6, num7)
            .is_some_and(|t| !(t.active() && !t.inactive()) && t.wall == 0);
        if !open {
            continue;
        }
        // `:13088` — site 1: the type-76 flake, 10×10 box, defaults.
        let num8 = pool.new_dust(
            rng,
            ctx,
            world,
            Vec2::new(num4 as f32, num5 as f32),
            10,
            10,
            76,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
        let d = pool.slot_mut(num8);
        // `:13089-13091`.
        d.scale += env.cloud_alpha * 0.2;
        d.velocity.y = 3.0 + rng.next_max(30) as f32 * 0.1;
        d.velocity.y *= d.scale;
        if !env.raining {
            // `:13094-13095` — the dry wind arm.
            d.velocity.x = env.wind_speed_current + rng.next_range(-10, 10) as f32 * 0.1;
            d.velocity.x += env.wind_speed_current * 15.0;
        } else {
            // `:13099-13100` — the storm arm: `Math.Sqrt(Math.Abs(w))`
            // promotes to f64; `Math.Sign` is the three-way sign (Rust's
            // `signum` maps +0 to +1 — the sqrt(0) product is 0 either way,
            // but the explicit match keeps the C# shape).
            let sign = if env.wind_speed_current > 0.0 {
                1.0
            } else if env.wind_speed_current < 0.0 {
                -1.0
            } else {
                0.0
            };
            d.velocity.x = (env.wind_speed_current.abs() as f64).sqrt() as f32
                * sign
                * (env.cloud_alpha + 0.5)
                * 10.0
                + rng.next_float() * 0.2
                - 0.1;
            d.velocity.y *= 0.5;
        }
        // `:13102-13108` — the shared tail (the SECOND `scale +=` is the
        // C#'s own double bump).
        d.velocity.y *= 1.0 + 0.3 * env.cloud_alpha;
        d.scale += env.cloud_alpha * 0.2;
        if flag {
            d.scale -= 0.5;
        }
        d.velocity *= 1.0 + env.cloud_alpha * 0.5;
    }
}

/// The `Microsoft.Xna.Framework.Rectangle` `TeleportEffect` takes — the
/// entity's `getRect()` (world px, integer box).
#[derive(Clone, Copy)]
pub(crate) struct EffectRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl EffectRect {
    /// `Rectangle.TopLeft()` (`:13324`/`:13336`/`:13354`).
    fn top_left(self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }

    /// `Rectangle.Center` — XNA's integer halves (`X + Width/2`).
    fn center(self) -> (i32, i32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }
}

/// `PortalHelper.GetPortalColor(int colorIndex)`
/// (`PortalHelper.cs:373-392`) — the single-player (`netMode == 0`) arm:
/// `colorIndex % 2` picks the exit blue `hsl(0.52, 1, 0.6)` or the entry
/// orange `hsl(0.12, 1, 0.5)`; `A = 66` (the caller's `color.A = 255`
/// overwrites it). The MP hue ladder is the server arm — dropped (SP port).
fn portal_color(color_index: i32) -> Color {
    let (h, l) = if color_index % 2 != 0 {
        (0.52, 0.6)
    } else {
        (0.12, 0.5)
    };
    let c = tr_core::tile_light::hsl_to_rgb(h, 1.0, l);
    Color::new(
        (c[0] * 255.0).round() as u8,
        (c[1] * 255.0).round() as u8,
        (c[2] * 255.0).round() as u8,
        66,
    )
}

/// One `SoundID.ItemN` style play on the caller's pass stream — the same
/// façade as the NPC sites (`Audio::play_npc_item_cue`: the audio-only
/// `GetRandomPitch` NextFloat, the attenuation gate, then case 2's one
/// `Main.rand` pitch draw). `Main.TeleportEffect`'s Item6/Item8 cues
/// (`Main.cs:13239`/`:13258`/`:13286`/`:13320`/`:13367`).
fn item_cue(app: &mut App, style: i32, x: f32, y: f32, key: RngKey) {
    let rng = app.rngs.stream(key);
    app.audio.play_npc_item_cue(style, x, y, rng);
}

/// `Main.TeleportEffect` (`Main.cs:13233-13464`) — THIS BATCH's styles
/// 0/1/2/3/4/5/7 (sites 2-8, `:13243`-`:13358`). Styles 10/11/13
/// (`:13390`/`:13438`/`:13452`) are W4-g12's; style 9's pylon dust is
/// `TeleportPylonsSystem.SpawnInWorldDust` (not a `Main.cs` `NewDust` site
/// — the pylon lane's); styles 6/8/12 have no C# arm (the switch falls
/// through). `side` is unread in the whole C# body — dropped;
/// `otherPosition` feeds only g12's style 10 — dropped here.
///
/// `key` is the calling pass's `SwapRandom` slot (one RNG slot per C#
/// pass): `Player.Teleport` → `UpdatePlayers` (`Main.cs:17754`),
/// `NPC.Teleport` → the NPC pass's stream, `Wiring.Teleport` →
/// `UpdateWorldGen`. The sounds ride the same stream (the C# draws them
/// from `Main.rand` mid-pass).
pub(crate) fn teleport_effect(
    app: &mut App,
    rect: EffectRect,
    style: i32,
    extra_info: i32,
    dust_count_mult: f32,
    key: RngKey,
) {
    match style {
        // `:13237-13255` — site 2 (`:13243`): the Rod-of-Discord-style
        // teleport-in sparkle, type 159.
        0 => {
            let (cx, cy) = rect.center();
            item_cue(app, 6, cx as f32, cy as f32, key);
            // `:13240` — the INTEGER `Width*Height/5` truncates first.
            let num2 = ((rect.width * rect.height / 5) as f32 * dust_count_mult) as i32;
            for j in 0..num2 {
                let idx = nd(app, rect.top_left(), rect.width, rect.height, 159, key);
                let rng = app.rngs.stream(key);
                let d = app.dust.slot_mut(idx);
                d.scale = rng.next_range(20, 70) as f32 * 0.01;
                if j < 10 {
                    d.scale += 0.25;
                }
                if j < 5 {
                    d.scale += 0.25;
                }
            }
        }
        // `:13256-13274` — site 3 (`:13262`): the recall sparkle, type 164.
        1 => {
            let (cx, cy) = rect.center();
            item_cue(app, 8, cx as f32, cy as f32, key);
            let num10 = ((rect.width * rect.height / 5) as f32 * dust_count_mult) as i32;
            for num11 in 0..num10 {
                let idx = nd(app, rect.top_left(), rect.width, rect.height, 164, key);
                let rng = app.rngs.stream(key);
                let d = app.dust.slot_mut(idx);
                d.scale = rng.next_range(20, 70) as f32 * 0.01;
                if num11 < 10 {
                    d.scale += 0.25;
                }
                if num11 < 5 {
                    d.scale += 0.25;
                }
            }
        }
        // `:13275-13283` — site 4 (`:13280`): the shellphone's 50
        // GhostWhite type-58 puffs (NO sound), each velocity ×0.5.
        2 => {
            let num15 = (50.0 * dust_count_mult) as i32;
            for _ in 0..num15 {
                let idx = nd_full(
                    app,
                    rect.top_left(),
                    rect.width,
                    rect.height,
                    58,
                    0.0,
                    0.0,
                    150,
                    Color::new(248, 248, 255, 255), // Color.GhostWhite
                    1.2,
                    key,
                );
                app.dust.slot_mut(idx).velocity *= 0.5;
            }
        }
        // `:13284-13317` — site 5 (`:13290`): the magic-mirror burst, 50
        // type-180 noGravity puffs with the five-roll shrink ladder.
        3 => {
            let (cx, cy) = rect.center();
            item_cue(app, 6, cx as f32, cy as f32, key);
            let num5 = (50.0 * dust_count_mult) as i32;
            for _ in 0..num5 {
                let idx = nd(app, rect.top_left(), rect.width, rect.height, 180, key);
                let rng = app.rngs.stream(key);
                let d = app.dust.slot_mut(idx);
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
        // `:13318-13333` — site 6 (`:13324`): the portal pair's type-263
        // glow, the portal colour + the fixed 1.2/0.4 shape, `A = 255`.
        4 => {
            let (cx, cy) = rect.center();
            item_cue(app, 8, cx as f32, cy as f32, key);
            let num17 = ((rect.width * rect.height / 5) as f32 * dust_count_mult) as i32;
            let color = portal_color(extra_info);
            for _ in 0..num17 {
                let idx = nd(app, rect.top_left(), rect.width, rect.height, 263, key);
                let d = app.dust.slot_mut(idx);
                d.color = color;
                d.no_light = true;
                d.no_gravity = true;
                d.scale = 1.2;
                d.fade_in = 0.4;
                d.color.a = u8::MAX;
            }
        }
        // `:13334-13351` — site 7 (`:13340`): the water-style splash column
        // (`Dust.dustWater()`, `Dust.cs:402-418`), height +24, then the
        // style-0 Splash cue AFTER the loop (`:13349`).
        5 => {
            let position = rect.top_left();
            let water_style = crate::biome::calculate_water_style(&app.world, app.camera) as i32;
            let ty = tr_core::particles::dust_water(water_style);
            let num8 = (100.0 * dust_count_mult) as i32;
            for _ in 0..num8 {
                let idx = nd_direct(app, position, rect.width, rect.height + 24, ty, key);
                let d = app.dust.slot_mut(idx);
                d.velocity.y *= 0.0;
                d.velocity.y -= 3.5;
                d.velocity.x *= 1.5;
                d.scale = 0.8;
                d.alpha = 130;
                d.no_gravity = true;
                d.fade_in = 1.2;
            }
            let (cx, cy) = rect.center();
            // `PlaySound(19, Center.ToVector2(), 0)` — the int overload's
            // style-0 splash (case 19, `LegacySoundPlayer.cs:871-888`: the
            // playing guard, then ONE `Next(-10, 11)` pitch draw).
            let rng = app.rngs.stream(key);
            app.audio.play_splash_sound(0, cx as f32, cy as f32, rng);
        }
        // `:13352-13369` — site 8 (`:13358`): the potion-of-return column,
        // type 35, the same shape as style 5, the Item8 cue AFTER the loop.
        7 => {
            let position2 = rect.top_left();
            let num19 = (50.0 * dust_count_mult) as i32;
            for _ in 0..num19 {
                let idx = nd_direct(app, position2, rect.width, rect.height + 24, 35, key);
                let d = app.dust.slot_mut(idx);
                d.velocity.y *= 0.0;
                d.velocity.y -= 3.5;
                d.velocity.x *= 1.5;
                d.scale = 0.8;
                d.alpha = 130;
                d.no_gravity = true;
                d.fade_in = 1.2;
            }
            let (cx, cy) = rect.center();
            item_cue(app, 8, cx as f32, cy as f32, key);
        }
        // Style 9 (`:13370-13380`) spawns through
        // `TeleportPylonsSystem.SpawnInWorldDust` — not a Main.cs NewDust
        // site (the pylon lane's). Styles 10/11/13 are W4-g12's sites
        // 9-11; 6/8/12 have no C# arm.
        _ => {}
    }
}

/// One `Dust.NewDust` with the C# defaults (SpeedX/Y 0, Alpha 0, default
/// colour, Scale 1) on the pass's stream — the sentinel included.
fn nd(app: &mut App, pos: Vec2, w: i32, h: i32, ty: i32, key: RngKey) -> usize {
    nd_full(app, pos, w, h, ty, 0.0, 0.0, 0, Color::default(), 1.0, key)
}

/// One `Dust.NewDust` with the full argument list on the pass's stream.
#[allow(clippy::too_many_arguments)]
fn nd_full(
    app: &mut App,
    pos: Vec2,
    w: i32,
    h: i32,
    ty: i32,
    sx: f32,
    sy: f32,
    alpha: i32,
    color: Color,
    scale: f32,
    key: RngKey,
) -> usize {
    let ctx = app.spawn_ctx();
    let rng = app.rngs.stream(key);
    app.dust
        .new_dust(rng, &ctx, &app.world, pos, w, h, ty, sx, sy, alpha, color, scale)
}

/// One `Dust.NewDustDirect` (`Dust.cs:65-73`) with the C# defaults on the
/// pass's stream.
fn nd_direct(app: &mut App, pos: Vec2, w: i32, h: i32, ty: i32, key: RngKey) -> usize {
    let ctx = app.spawn_ctx();
    let rng = app.rngs.stream(key);
    app.dust.new_dust_direct(
        rng, &ctx, &app.world, pos, w, h, ty, 0.0, 0.0, 0, Color::default(), 1.0,
    )
}

/// W4-g11 wiring — the `Player.Teleport` request (`Player.cs:37414-37496`):
/// the movement core queues it (`Player::teleport_style_with_extra`), the
/// drain replays the C# order — the Entry `TeleportEffect` at the OLD box
/// (`:37444`), the move (`:37446`), the Exit at the NEW box (`:37489`), the
/// `teleportTime`/`teleportStyle` grants (`:37490-37491`). `key` is the
/// CALLER's pass stream: `UpdatePlayers` for the player-tick callers
/// (`Main.cs:17754`), `UpdateWorldGen` for the wiring pads (`Wiring.cs:3154`
/// under `WorldGen.UpdateWorld`), `UpdateProjectiles` for the Queen-Slime
/// hook (`Projectile.cs:52775` under the projectile pass).
pub(crate) struct PlayerTeleportReq {
    pub old_x: f32,
    pub old_y: f32,
    pub new_x: f32,
    pub new_y: f32,
    pub style: i32,
    /// The Entry effect's `extraInfo2` (the OLD portal colour, `:37433-37436`).
    pub extra_info2: i32,
    /// The Exit effect's `extraInfo2` (the NEW portal colour, `:37475-37478`).
    pub extra_info: i32,
    /// `:37442` — `Clamp(1 - teleportTime*0.99, 0.01, 1)`.
    pub mult: f32,
    pub key: RngKey,
}

/// The `Player.Teleport` replay (W4-g11 wiring): the Entry effect at the OLD
/// box, the movement core, the Exit effect at the NEW box, then the grants.
/// The camera transition / map-refresh halves (`:37456-37474`) stay the
/// port's render seams (`world_sim::camera` owns the wiring-pad transition).
pub(crate) fn player_teleport(app: &mut App, req: PlayerTeleportReq) {
    let Some(p) = app.player.as_ref() else {
        return;
    };
    let (w, h) = (p.width as i32, p.height as i32);
    let old_rect = EffectRect { x: req.old_x as i32, y: req.old_y as i32, width: w, height: h };
    // `:37444` — the Entry effect BEFORE the position write.
    teleport_effect(app, old_rect, req.style, req.extra_info2, req.mult, req.key);
    // `:37446`/`:37455`/`:37488` — the move (no re-queue).
    if let Some(p) = app.player.as_mut() {
        p.teleport_move_core(req.new_x, req.new_y, req.style);
    }
    let new_rect = EffectRect { x: req.new_x as i32, y: req.new_y as i32, width: w, height: h };
    // `:37489` — the Exit effect AFTER the move.
    teleport_effect(app, new_rect, req.style, req.extra_info, req.mult, req.key);
    // `:37490-37491` — the grants.
    if let Some(p) = app.player.as_mut() {
        p.teleport_time = 1.0;
        p.teleport_style_id = req.style;
    }
}

/// W4-g11 wiring — the `NPC.Teleport` request (`NPC.cs:81035-81058`): the
/// wiring-pad drain queues it, the replay runs the Entry effect at the OLD
/// box (`:81044`), the position write (`:81045`), the Exit at the NEW box
/// (`:81051`), the grants (`:81052-81053`). `index` is the NPC's slot in
/// `App::npcs`; `key` the caller's pass stream (`UpdateWorldGen` for the
/// pads, `Wiring.cs:3170`).
pub(crate) struct NpcTeleportReq {
    pub index: usize,
    pub new_x: f32,
    pub new_y: f32,
    pub style: i32,
    pub extra_info: i32,
    pub key: RngKey,
}

/// The `NPC.Teleport` replay (W4-g11 wiring). The MP `SendData(65)`
/// (`:81054-81057`) is single-player dead.
pub(crate) fn npc_teleport(app: &mut App, req: NpcTeleportReq) {
    let Some(n) = app.npcs.get(req.index) else {
        return;
    };
    let (w, h) = n.size();
    let (w, h) = (w as i32, h as i32);
    // `:81037-81041` — the Entry reads the OLD portal colour.
    let extra_info2 = if req.style == 4 { n.last_portal_color_index } else { 0 };
    // `:81042` — `teleportTime > 0 ? 0.3 : 1`.
    let mult = if n.teleport_time > 0.0 { 0.3 } else { 1.0 };
    let old_rect = EffectRect { x: n.x as i32, y: n.y as i32, width: w, height: h };
    // `:81044` — the Entry effect BEFORE the position write.
    teleport_effect(app, old_rect, req.style, extra_info2, mult, req.key);
    // `:81045-81050` — the move + the portal hand-off.
    let Some(n) = app.npcs.get_mut(req.index) else {
        return;
    };
    n.x = req.new_x;
    n.y = req.new_y;
    let exit_extra = if req.style == 4 {
        n.last_portal_color_index = req.extra_info;
        n.last_portal_color_index
    } else {
        extra_info2
    };
    let new_rect = EffectRect { x: req.new_x as i32, y: req.new_y as i32, width: w, height: h };
    // `:81051` — the Exit effect AFTER the move.
    teleport_effect(app, new_rect, req.style, exit_extra, mult, req.key);
    // `:81052-81053` — the grants.
    if let Some(n) = app.npcs.get_mut(req.index) {
        n.teleport_time = 1.0;
        n.teleport_style_id = req.style;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::sound::Listener;
    use tr_core::particles::DUST_SENTINEL;

    const KEY: RngKey = RngKey::UpdatePlayers;

    /// The teleport rig (the d51 `rig` pattern): an empty world, the full
    /// 6000 dust budget (no pressure-band draws below slot 3000), the
    /// camera parked on the effect box, the pass stream pinned to `seed`,
    /// and the audio listener ON the box centre so the attenuation gate
    /// passes deterministically (the cue's one pitch draw then rides the
    /// pass stream, exactly like the C#).
    fn rig(seed: i32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = World::new("dust".into(), 64, 128);
        app.gfx_quality.max_dust_to_draw = 6000;
        app.camera = Vec2::new(320.0, 640.0);
        app.audio.set_listener(Listener {
            camera_center: (310.0, 621.0),
            sound_volume: 1.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        *app.rngs.stream(KEY) = UnifiedRandom::new(seed);
        app
    }

    fn rect() -> EffectRect {
        EffectRect { x: 300, y: 600, width: 20, height: 42 }
    }

    fn live(app: &App) -> Vec<&tr_core::particles::Dust> {
        app.dust.slots().iter().filter(|d| d.active).collect()
    }

    fn stream_of(app: &mut App) -> UnifiedRandom {
        app.rngs.stream(KEY).clone()
    }

    /// The six `NewDust` default-block draws (`Dust.cs:153-193`) for a box
    /// clamped to ≥5 — none of this batch's types (159/164/58/180/263/33/
    /// 35/76) rolls a per-type tweak draw beyond the shared block (33/35
    /// tweak without draws).
    fn default_block(rng: &mut UnifiedRandom, pos: Vec2, w: i32, h: i32) -> (Vec2, Vec2, f32) {
        let w = w.max(5);
        let h = h.max(5);
        let px = pos.x + rng.next_max(w - 4) as f32 + 4.0;
        let py = pos.y + rng.next_max(h - 4) as f32 + 4.0;
        let vx = rng.next_range(-20, 21) as f32 * 0.1;
        let vy = rng.next_range(-20, 21) as f32 * 0.1;
        let _ = rng.next_max(3); // frame row
        let scale = 1.0 + rng.next_range(-20, 21) as f32 * 0.01;
        (Vec2::new(px, py), Vec2::new(vx, vy), scale)
    }

    /// The Item6/Item8 cue's one case-2 pitch draw (`sound.rs::
    /// item_use_case2`'s default arm — styles 6/8 hit no remap/volume arm).
    fn item_pitch(rng: &mut UnifiedRandom) {
        let _ = rng.next_range(-6, 7);
    }

    /// Sites 2/3 — styles 0/1: the Item6/Item8 cue FIRST (one pitch draw),
    /// then `Width*Height/5 * mult` puffs, each the six default draws + the
    /// `Next(20,70)` scale roll; j<10 adds 0.25, j<5 adds another 0.25.
    #[test]
    fn w4_g11_styles_0_and_1_roll_the_scale_ladder_after_the_cue() {
        for (style, ty) in [(0, 159), (1, 164)] {
            let mut app = rig(0x5EED);
            // 20*42/5 = 168; ×0.05 → 8.4 → 8 puffs (j<5 and j<10 both fire
            // on the first five).
            teleport_effect(&mut app, rect(), style, 0, 0.05, KEY);
            let slots = live(&app);
            assert_eq!(slots.len(), 8);
            let mut rng = UnifiedRandom::new(0x5EED);
            item_pitch(&mut rng); // the cue precedes the loop (`:13239`)
            for (j, d) in slots.iter().enumerate() {
                assert_eq!(d.dust_type, ty);
                let (p, v, _) = default_block(&mut rng, rect().top_left(), 20, 42);
                assert_eq!(d.position, p);
                assert_eq!(d.velocity, v);
                let mut scale = rng.next_range(20, 70) as f32 * 0.01;
                if j < 10 {
                    scale += 0.25;
                }
                if j < 5 {
                    scale += 0.25;
                }
                assert_eq!(d.scale, scale, "the roll OVERWRITES the spawn scale");
            }
            assert_eq!(stream_of(&mut app), rng);
        }
    }

    /// Site 4 — style 2: NO sound, 50×mult GhostWhite type-58 puffs
    /// (alpha 150, scale 1.2), each velocity ×0.5.
    #[test]
    fn w4_g11_style_2_is_the_soundless_ghostwhite_burst() {
        let mut app = rig(7);
        teleport_effect(&mut app, rect(), 2, 0, 0.1, KEY); // 5 puffs
        let slots = live(&app);
        assert_eq!(slots.len(), 5);
        let mut rng = UnifiedRandom::new(7);
        for d in &slots {
            assert_eq!(d.dust_type, 58);
            assert_eq!(d.alpha, 150);
            assert_eq!(d.color, Color::new(248, 248, 255, 255));
            let (p, v, s) = default_block(&mut rng, rect().top_left(), 20, 42);
            assert_eq!(d.position, p);
            assert_eq!(d.velocity, v * 0.5);
            assert_eq!(d.scale, s * 1.2);
        }
        // No cue: exactly 5×6 draws, nothing else.
        assert_eq!(stream_of(&mut app), rng);
    }

    /// Site 5 — style 3: the Item6 cue, then 50×mult type-180 noGravity
    /// puffs with the five-roll ×0.75 ladder, the two ×2/×1.2 gates, the
    /// `Next(2)` fadeIn/scale re-roll and the final ×0.8.
    #[test]
    fn w4_g11_style_3_mirrors_the_full_shrink_ladder() {
        let mut app = rig(0xC0FFEE);
        teleport_effect(&mut app, rect(), 3, 0, 0.1, KEY); // 5 puffs
        let slots = live(&app);
        assert_eq!(slots.len(), 5);
        let mut rng = UnifiedRandom::new(0xC0FFEE);
        item_pitch(&mut rng);
        for d in &slots {
            assert_eq!(d.dust_type, 180);
            assert!(d.no_gravity);
            let (p, mut v, mut s) = default_block(&mut rng, rect().top_left(), 20, 42);
            assert_eq!(d.position, p);
            for _ in 0..5 {
                if rng.next_max(3) == 0 {
                    v *= 0.75;
                }
            }
            if rng.next_max(3) == 0 {
                v *= 2.0;
                s *= 1.2;
            }
            if rng.next_max(3) == 0 {
                v *= 2.0;
                s *= 1.2;
            }
            let mut fade = 0.0;
            if rng.next_max(2) == 0 {
                fade = rng.next_range(75, 100) as f32 * 0.01;
                s = rng.next_range(25, 75) as f32 * 0.01;
            }
            s *= 0.8;
            assert_eq!(d.velocity, v);
            assert_eq!(d.scale, s);
            assert_eq!(d.fade_in, fade);
        }
        assert_eq!(stream_of(&mut app), rng);
    }

    /// Site 6 — style 4: the portal colour by `extraInfo % 2` (the entry
    /// orange is pinned byte-exact; the exit blue rides the ported
    /// `hsl_to_rgb`), `A = 255` overwrites the helper's 66, and the fixed
    /// 1.2/0.4/noLight/noGravity shape carries NO post-spawn draws.
    #[test]
    fn w4_g11_style_4_paints_the_portal_pair() {
        for extra in [0, 1] {
            let mut app = rig(11);
            teleport_effect(&mut app, rect(), 4, extra, 0.05, KEY); // 8 puffs
            let slots = live(&app);
            assert_eq!(slots.len(), 8);
            let expected = if extra % 2 != 0 {
                let c = tr_core::tile_light::hsl_to_rgb(0.52, 1.0, 0.6);
                Color::new(
                    (c[0] * 255.0).round() as u8,
                    (c[1] * 255.0).round() as u8,
                    (c[2] * 255.0).round() as u8,
                    255,
                )
            } else {
                // hsl(0.12, 1, 0.5): num2 = 1, t = 0 — r = hue2rgb(0.4533)
                // = 1 → 255, g = hue2rgb(0.12) = 0.72 → 184, b =
                // hue2rgb(-0.2133 → 0.7867) = 0 → 0 (banker's-rounded).
                Color::new(255, 184, 0, 255)
            };
            let mut rng = UnifiedRandom::new(11);
            item_pitch(&mut rng);
            for d in &slots {
                assert_eq!(d.dust_type, 263);
                assert_eq!(d.color, expected);
                assert!(d.no_light && d.no_gravity);
                assert_eq!(d.scale, 1.2);
                assert_eq!(d.fade_in, 0.4);
                let (p, v, _) = default_block(&mut rng, rect().top_left(), 20, 42);
                assert_eq!(d.position, p);
                assert_eq!(d.velocity, v);
            }
            assert_eq!(stream_of(&mut app), rng);
        }
    }

    /// Sites 7/8 — styles 5/7: the +24-height column (`dustWater()` /
    /// type 35), the `vy = vy*0 - 3.5` / `vx *= 1.5` shape, alpha 130,
    /// scale 0.8, fadeIn 1.2 — and the cue AFTER the loop (the style-0
    /// Splash's one `Next(-10,11)` for 5, the Item8 pitch for 7).
    #[test]
    fn w4_g11_styles_5_and_7_play_the_cue_after_the_column() {
        for style in [5, 7] {
            let mut app = rig(23);
            let mult = if style == 5 { 0.05 } else { 0.1 }; // 5 puffs each
            teleport_effect(&mut app, rect(), style, 0, mult, KEY);
            let slots = live(&app);
            assert_eq!(slots.len(), 5);
            let water_style =
                crate::biome::calculate_water_style(&app.world, app.camera) as i32;
            let ty = if style == 5 {
                tr_core::particles::dust_water(water_style)
            } else {
                35
            };
            let mut rng = UnifiedRandom::new(23);
            for d in &slots {
                assert_eq!(d.dust_type, ty);
                let (p, mut v, _) = default_block(&mut rng, rect().top_left(), 20, 66);
                // The 33/35 per-type tweaks (`Dust.cs:214-252`, no draws).
                if ty == 33 {
                    v *= 0.5;
                    v.y += 1.0;
                } else if ty == 35 {
                    v *= 0.1;
                    v.y = -0.5;
                }
                assert_eq!(d.position, p);
                v.y *= 0.0;
                v.y -= 3.5;
                v.x *= 1.5;
                assert_eq!(d.velocity, v);
                assert_eq!(d.scale, 0.8);
                assert_eq!(d.alpha, 130);
                assert!(d.no_gravity);
                assert_eq!(d.fade_in, 1.2);
            }
            // The AFTER-loop cue: one pitch draw on the same stream.
            if style == 5 {
                let _ = rng.next_range(-10, 11); // splash_case19 style 0
            } else {
                item_pitch(&mut rng); // Item8
            }
            assert_eq!(stream_of(&mut app), rng);
        }
    }

    /// The batch boundary: styles 6/8/12 have no C# arm, style 9 is the
    /// pylon lane's, and styles 10/11/13 are W4-g12's sites — this port
    /// must draw NOTHING for them (no dust, no stream step).
    #[test]
    fn w4_g11_the_g12_styles_stay_untouched() {
        for style in [6, 8, 9, 10, 11, 12, 13] {
            let mut app = rig(3);
            teleport_effect(&mut app, rect(), style, 0, 1.0, KEY);
            assert!(live(&app).is_empty(), "style {style} drew dust");
            assert_eq!(stream_of(&mut app), UnifiedRandom::new(3));
        }
    }

    /// The sentinel contract: a paused spawn ctx refuses every `NewDust`
    /// (no internals draws), but the post-spawn writes still land on
    /// `Main.dust[6000]` and the explicit scale rolls still ride the
    /// stream (the C# writes the sentinel blindly).
    #[test]
    fn w4_g11_a_refused_spawn_still_writes_the_sentinel() {
        let mut app = rig(5);
        app.game_paused = true;
        teleport_effect(&mut app, rect(), 0, 0, 0.05, KEY); // 8 puffs
        assert!(live(&app).is_empty());
        let mut rng = UnifiedRandom::new(5);
        item_pitch(&mut rng);
        let mut last_scale = 0.0;
        for j in 0..8 {
            let mut scale = rng.next_range(20, 70) as f32 * 0.01;
            if j < 10 {
                scale += 0.25;
            }
            if j < 5 {
                scale += 0.25;
            }
            last_scale = scale;
        }
        assert_eq!(app.dust.slot(DUST_SENTINEL).scale, last_scale);
        assert_eq!(stream_of(&mut app), rng);
    }

    // --- snowing (site 1) -------------------------------------------------

    /// The snowing rig: a big open world, the view parked far inside it so
    /// every rolled spawn lands on an open tile, the global stream pinned.
    fn snow_rig(seed: i32) -> (App, SnowingEnv) {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = World::new("snow".into(), 512, 512);
        app.gfx_quality.max_dust_to_draw = 6000;
        app.camera = Vec2::new(4096.0 + 640.0, 4096.0 + 360.0);
        *app.rngs.global() = UnifiedRandom::new(seed);
        let env = SnowingEnv {
            remix_world: false,
            game_paused: false,
            snow_tile_count: 6000,
            surface_atmospherics: true,
            cloud_alpha: 0.0,
            scaled_size: Vec2::new(1280.0, 720.0),
            scaled_position: Vec2::new(4096.0, 4096.0),
            gfx_quality: 0.0,
            deerclops_near: false,
            player_vy: 5.0,
            raining: false,
            wind_speed_current: 0.3,
        };
        (app, env)
    }

    /// The four early-return gates draw NOTHING (`:13034-13043`).
    #[test]
    fn w4_g11_snowing_gates_before_any_draw() {
        for gate in 0..4 {
            let (mut app, mut env) = snow_rig(1);
            match gate {
                0 => env.remix_world = true,
                1 => env.game_paused = true,
                2 => env.snow_tile_count = 0,
                _ => env.surface_atmospherics = false,
            }
            let ctx = app.spawn_ctx();
            snowing(&env, app.rngs.global(), &ctx, &app.world, &mut app.dust);
            assert!(live(&app).is_empty(), "gate {gate} drew dust");
            assert_eq!(app.rngs.global().clone(), UnifiedRandom::new(1));
        }
    }

    /// The `NewDust` off-screen cull (`Dust.cs:97-101`) for the snow rigs:
    /// screen_position (4096,4096), screen 1280×720, dCount 0 → margin 400,
    /// rect (3696,3696,2080,1520); XNA `Intersects` is strict `<`.
    fn snow_cull_passes(x: i32, y: i32) -> bool {
        3696 < x + 10 && x < 3696 + 2080 && 3696 < y + 10 && y < 3696 + 1520
    }

    /// Site 1, the dry-wind arm: cloudAlpha 0 → ONE iteration, the full
    /// snow count → the `NextFloat` gate always passes; the mirror walks
    /// every draw in C# order (the two `Next(5)` edge gates roll in `else
    /// if` order, the y-spread only off-screen) and pins the flake's
    /// position/velocity/scale — including the double `cloudAlpha` scale
    /// bump (0 here) and the `windSpeedCurrent * 15` dry tail. A culled
    /// spawn draws NO internals but the post-writes still roll onto the
    /// sentinel (`Dust.cs:79` — the C# writes `Main.dust[6000]` blindly).
    #[test]
    fn w4_g11_snowing_mirrors_the_dry_arm_draw_sequence() {
        let (mut app, env) = snow_rig(0x5EED);
        let ctx = app.spawn_ctx();
        snowing(&env, app.rngs.global(), &ctx, &app.world, &mut app.dust);
        let mut rng = UnifiedRandom::new(0x5EED);
        let _ = rng.next_float(); // the spawn gate (`:13061`)
        let mut num4 = rng.next_max(1280 + 1500) - 750;
        let mut num5 = 4096 - rng.next_max(50);
        num5 -= 5; // player_vy > 0 (`:13067-13070`)
        if rng.next_max(5) == 0 {
            num4 = rng.next_max(500) - 500;
        } else if rng.next_max(5) == 0 {
            num4 = rng.next_max(500) + 1280;
        }
        if num4 < 0 || num4 as f32 > 1280.0 {
            num5 += rng.next_max((720.0f64 * 0.8) as i32) + (720.0f64 * 0.1) as i32;
        }
        num4 += 4096;
        let passes = snow_cull_passes(num4, num5);
        let mut v = Vec2::ZERO;
        let mut s = 0.0;
        if passes {
            let (p, vel, scale) =
                default_block(&mut rng, Vec2::new(num4 as f32, num5 as f32), 10, 10);
            let d = live(&app)[0];
            assert_eq!(d.dust_type, 76);
            assert_eq!(d.position, p);
            v = vel;
            s = scale;
        } else {
            assert!(live(&app).is_empty(), "the cull refuses the spawn");
        }
        // `:13089-13091` — scale += 0 (cloudAlpha 0), the vy re-roll.
        v.y = 3.0 + rng.next_max(30) as f32 * 0.1;
        v.y *= s;
        // `:13094-13095` — the dry wind arm.
        v.x = 0.3 + rng.next_range(-10, 10) as f32 * 0.1;
        v.x += 0.3 * 15.0;
        // `:13102-13108` — the shared tail at cloudAlpha 0.
        v.y *= 1.0;
        if passes {
            let d = live(&app)[0];
            assert_eq!(d.velocity, v);
            assert_eq!(d.scale, s);
        } else {
            // The sentinel took the writes; its scale is the pool default
            // (0), so only the scale-independent x write is pinned.
            assert_eq!(app.dust.slot(DUST_SENTINEL).velocity.x, v.x);
        }
        assert_eq!(app.rngs.global().clone(), rng);
    }

    /// Site 1, the storm arm (`Main.raining`): the sqrt/sign wind formula
    /// (negative wind → negative carry), the vy ×0.5, the cloudAlpha
    /// double scale bump + the ×(1+0.3a)/×(1+0.5a) tails, and the
    /// Deerclops −0.5 scale cut. cloudAlpha 0.5 → num3 = 26 iterations;
    /// the mirror walks them all with a LOCAL budget counter (the pool's
    /// live count grows as the port spawns — the mirror must read it at
    /// the same moment). The deerclops gate is 0.05/iteration, so the
    /// seed sweep guarantees at least one spawn.
    #[test]
    fn w4_g11_snowing_mirrors_the_storm_arm_with_deerclops() {
        let mut any_spawn = false;
        for seed in [99i32, 7, 0x5EED, 12345] {
            let (mut app, mut env) = snow_rig(seed);
            env.cloud_alpha = 0.5;
            env.raining = true;
            env.wind_speed_current = -0.4;
            env.deerclops_near = true;
            let ctx = app.spawn_ctx();
            snowing(&env, app.rngs.global(), &ctx, &app.world, &mut app.dust);
            let mut rng = UnifiedRandom::new(seed);
            // num = (6000/6000)^(4-1.5) / 20 = 0.05; num2 = 156*2/3 = 104.
            let num = 1.0f32 / 20.0;
            let num2 = (((500.0 * (1280.0 / 4096.0)) as i32 as f32) * 2.0) as i32 / 3;
            let mut spawned = 0usize;
            let mut i = 0;
            while (i as f32) < 1.0 + 50.0 * 0.5 {
                i += 1;
                if !(spawned as f32 < num2 as f32 * 0.5 + num2 as f32 * 0.1) {
                    break;
                }
                if !(rng.next_float() < num) {
                    continue;
                }
                let mut num4 = rng.next_max(1280 + 1500) - 750;
                let mut num5 = 4096 - rng.next_max(50) - 5;
                if rng.next_max(5) == 0 {
                    num4 = rng.next_max(500) - 500;
                } else if rng.next_max(5) == 0 {
                    num4 = rng.next_max(500) + 1280;
                }
                if num4 < 0 || num4 as f32 > 1280.0 {
                    num5 += rng.next_max((720.0f64 * 0.8) as i32) + (720.0f64 * 0.1) as i32;
                }
                num4 += 4096;
                let mut v = Vec2::ZERO;
                let mut s = 0.0;
                let mut p = Vec2::ZERO;
                if snow_cull_passes(num4, num5) {
                    let (pos, vel, scale) =
                        default_block(&mut rng, Vec2::new(num4 as f32, num5 as f32), 10, 10);
                    p = pos;
                    v = vel;
                    s = scale;
                    let d = app.dust.slot(spawned);
                    assert!(d.active);
                    assert_eq!(d.dust_type, 76);
                    assert_eq!(d.position, p);
                    spawned += 1;
                }
                s += 0.5 * 0.2;
                v.y = 3.0 + rng.next_max(30) as f32 * 0.1;
                v.y *= s;
                v.x = (0.4f64.sqrt() as f32) * -1.0 * (0.5 + 0.5) * 10.0
                    + rng.next_float() * 0.2
                    - 0.1;
                v.y *= 0.5;
                v.y *= 1.0 + 0.3 * 0.5;
                s += 0.5 * 0.2;
                s -= 0.5; // the Deerclops cut
                v *= 1.0 + 0.5 * 0.5;
                if snow_cull_passes(num4, num5) {
                    let d = app.dust.slot(spawned - 1);
                    assert_eq!(d.velocity, v);
                    assert_eq!(d.scale, s);
                }
            }
            assert_eq!(live(&app).len(), spawned);
            assert_eq!(app.rngs.global().clone(), rng);
            any_spawn |= spawned > 0;
        }
        assert!(any_spawn, "the seed sweep must pass the 0.05 gate");
    }

    /// The budget gate BREAKS before any draw: with the Deerclops throttle
    /// at cloudAlpha 0, num2 = 156/3 = 52 → cap 52×0.5 + 52×0.1 = 31.2; a
    /// pool already over it spends nothing (the `Main.snowDust` stand-in
    /// reads the live 76 population, the `SandStormCount` pattern).
    #[test]
    fn w4_g11_snowing_breaks_on_the_live_76_budget() {
        let (mut app, mut env) = snow_rig(1);
        env.deerclops_near = true; // num2 = 156/3 = 52 → cap 31.2
        for slot in app.dust.slots_mut()[..40].iter_mut() {
            slot.active = true;
            slot.dust_type = 76;
        }
        let ctx = app.spawn_ctx();
        snowing(&env, app.rngs.global(), &ctx, &app.world, &mut app.dust);
        assert_eq!(crate::minecart::live_type(&app.dust, 76), 40);
        assert_eq!(app.rngs.global().clone(), UnifiedRandom::new(1));
    }

    // --- W4-g11 wiring: the Teleport replays -------------------------------

    /// The player rig: the teleport rig + a player parked on the OLD box
    /// (300, 600), the listener on its centre so both effect cues pass the
    /// attenuation gate.
    fn player_rig(seed: i32) -> App {
        let mut app = rig(seed);
        let mut p = crate::player::Player::spawn(&app.world);
        p.x = 300.0;
        p.y = 600.0;
        app.player = Some(p);
        app
    }

    /// The `Player.Teleport` replay (`Player.cs:37414-37496`): the Entry
    /// effect at the OLD box BEFORE the move, the Exit at the NEW box AFTER,
    /// both on the caller's stream in C# order — style 1's two Item8 cues +
    /// two 8-puff bursts (mult 0.05), the full stream mirror walks them.
    #[test]
    fn w4_g11_player_teleport_replays_entry_then_exit() {
        let mut app = player_rig(0x5EED);
        player_teleport(
            &mut app,
            PlayerTeleportReq {
                old_x: 300.0,
                old_y: 600.0,
                new_x: 500.0,
                new_y: 700.0,
                style: 1,
                extra_info2: 0,
                extra_info: 0,
                mult: 0.05,
                key: KEY,
            },
        );
        // The move + the grants (`:37446`/`:37490-37491`).
        let p = app.player.as_ref().unwrap();
        assert_eq!((p.x, p.y), (500.0, 700.0));
        assert_eq!(p.teleport_time, 1.0);
        assert_eq!(p.teleport_style_id, 1);
        // 8 puffs at the OLD box + 8 at the NEW (`20*42/5*0.05 = 8.4 → 8`).
        let slots = live(&app);
        assert_eq!(slots.len(), 16);
        let mut rng = UnifiedRandom::new(0x5EED);
        for (burst, origin) in [(0usize, Vec2::new(300.0, 600.0)), (8, Vec2::new(500.0, 700.0))] {
            item_pitch(&mut rng); // the Item8 cue precedes each burst
            for j in 0..8 {
                let d = slots[burst + j];
                assert_eq!(d.dust_type, 164);
                let (pos, v, _) = default_block(&mut rng, origin, 20, 42);
                assert_eq!(d.position, pos);
                assert_eq!(d.velocity, v);
                let mut scale = rng.next_range(20, 70) as f32 * 0.01;
                if j < 10 {
                    scale += 0.25;
                }
                if j < 5 {
                    scale += 0.25;
                }
                assert_eq!(d.scale, scale);
            }
        }
        assert_eq!(stream_of(&mut app), rng);
    }

    /// The queue contract: `teleport_style` captures the OLD box + the
    /// `Clamp(1 - teleportTime*0.99, 0.01, 1)` mult at the call moment, moves
    /// the player, grants `teleportTime = 1`/`teleportStyle`, and the take
    /// drains once (the `pending_force_clear` convention).
    #[test]
    fn w4_g11_teleport_style_queues_the_effect_pair() {
        let mut app = player_rig(3);
        let p = app.player.as_mut().unwrap();
        p.teleport_time = 0.5; // mult = 1 - 0.495 = 0.505
        p.teleport_style(500.0, 700.0, 1);
        assert_eq!((p.x, p.y), (500.0, 700.0), "the move ran inline");
        assert_eq!(p.teleport_time, 1.0);
        assert_eq!(p.teleport_style_id, 1);
        let req = p.take_teleport_effect().expect("the pair queued");
        assert_eq!((req.0, req.1, req.2, req.3), (300.0, 600.0, 500.0, 700.0));
        assert_eq!((req.4, req.5, req.6), (1, 0, 0));
        assert!((req.7 - 0.505).abs() < 1e-6, "mult = 1 - 0.5*0.99: {}", req.7);
        assert!(p.take_teleport_effect().is_none(), "drains once");
        // A second teleport clamps the mult at 0.01 (teleportTime 1).
        p.teleport_style(600.0, 800.0, 2);
        let req = p.take_teleport_effect().unwrap();
        assert_eq!(req.7, 0.01);
    }

    /// The `NPC.Teleport` replay (`NPC.cs:81035-81058`): the Entry at the OLD
    /// box, the move, the Exit at the NEW box, the grants — style 0's two
    /// Item6 cues + two bursts. The NPC box is its `size()` (the Nurse 20×42
    /// stand-in is the rig's; the mirror reads the live size).
    #[test]
    fn w4_g11_npc_teleport_replays_entry_then_exit() {
        let mut app = rig(0xC0FFEE);
        let mut n = crate::npc::Npc::new(crate::npc::NpcKind::Nurse, 300.0, 600.0, 1, 0.0);
        n.hp = n.max_hp;
        app.npcs.push(n);
        let (w, h) = app.npcs[0].size();
        let (wi, hi) = (w as i32, h as i32);
        npc_teleport(
            &mut app,
            NpcTeleportReq {
                index: 0,
                new_x: 500.0,
                new_y: 700.0,
                style: 0,
                extra_info: 0,
                key: KEY,
            },
        );
        assert_eq!((app.npcs[0].x, app.npcs[0].y), (500.0, 700.0));
        assert_eq!(app.npcs[0].teleport_time, 1.0);
        assert_eq!(app.npcs[0].teleport_style_id, 0);
        // mult 1 (teleportTime was 0): `w*h/5` puffs per burst.
        let per_burst = (wi * hi / 5) as usize;
        let slots = live(&app);
        assert_eq!(slots.len(), per_burst * 2);
        let mut rng = UnifiedRandom::new(0xC0FFEE);
        for (burst, origin) in
            [(0usize, Vec2::new(300.0, 600.0)), (per_burst, Vec2::new(500.0, 700.0))]
        {
            item_pitch(&mut rng); // the Item6 cue precedes each burst
            for j in 0..per_burst {
                let d = slots[burst + j];
                assert_eq!(d.dust_type, 159);
                let (pos, v, _) = default_block(&mut rng, origin, wi, hi);
                assert_eq!(d.position, pos);
                assert_eq!(d.velocity, v);
                let mut scale = rng.next_range(20, 70) as f32 * 0.01;
                if j < 10 {
                    scale += 0.25;
                }
                if j < 5 {
                    scale += 0.25;
                }
                assert_eq!(d.scale, scale);
            }
        }
        assert_eq!(stream_of(&mut app), rng);
        // A re-teleport inside the window reads mult 0.3 (`:81042`).
        let before = app.rngs.stream(KEY).clone();
        npc_teleport(
            &mut app,
            NpcTeleportReq { index: 0, new_x: 500.0, new_y: 700.0, style: 0, extra_info: 0, key: KEY },
        );
        let expected = ((wi * hi / 5) as f32 * 0.3) as usize;
        assert_eq!(live(&app).len() - per_burst * 2, expected * 2);
        assert_ne!(app.rngs.stream(KEY).clone(), before);
    }

    /// The caller-key contract: the wiring pads replay on `UpdateWorldGen`,
    /// the player tick on `UpdatePlayers` — the OTHER streams stay untouched
    /// (one RNG slot per C# pass, S1-A-10).
    #[test]
    fn w4_g11_the_replay_rides_only_the_callers_stream() {
        let mut app = player_rig(9);
        // The global baseline BEFORE any read (the rig's spawn_ctx does not
        // draw; the replay must not touch the bare stream either).
        let global_before = app.rngs.global().clone();
        let untouched: Vec<_> = crate::app::RngKey::ALL
            .iter()
            .copied()
            .filter(|k| *k != crate::app::RngKey::UpdateWorldGen)
            .map(|k| (k, app.rngs.stream(k).clone()))
            .collect();
        player_teleport(
            &mut app,
            PlayerTeleportReq {
                old_x: 300.0,
                old_y: 600.0,
                new_x: 500.0,
                new_y: 700.0,
                style: 1,
                extra_info2: 0,
                extra_info: 0,
                mult: 0.05,
                key: crate::app::RngKey::UpdateWorldGen,
            },
        );
        assert!(!live(&app).is_empty(), "the replay drew on UpdateWorldGen");
        for (k, before) in untouched {
            assert_eq!(app.rngs.stream(k).clone(), before, "{k:?} moved");
        }
        assert_eq!(app.rngs.global().clone(), global_before);
    }
}


