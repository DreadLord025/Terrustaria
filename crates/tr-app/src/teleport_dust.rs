//! W4-g11 union-exec (research/116 keep-one) — the surviving
//! `Main.TeleportEffect` body (`Main.cs:13233-13464`), adapted per-hunk
//! from W4-g61's `npc_spawn_visuals.rs::teleport_effect` (`a9628928`,
//! bodies `6345f7b7`/`8d23e298`): the widest ELEVEN style arms
//! (0/1/2/3/4/5/7/9/10/11/13; 6/8/12 have no C# arm), the canonical
//! `ui::color::hsl_to_rgb` (the C#-exact f32→f64 widen + banker's
//! rounding + i64 byte wrap), the sounds AT the C# call position through
//! the callback, the faithful style-10 spawned-slot clone guard.
//!
//! The g11 seams stay canonical (`main_dust.rs`): the App-level
//! `teleport_effect` dispatcher builds the env from the disjoint App
//! fields + the caller's pass stream and resolves the Audio façade; the
//! Entry-OLD → move → Exit-NEW replays (`player_teleport`/`npc_teleport`)
//! and the queue/pad drains are unchanged in shape.
//!
//! Adaptations from g61's module (per-hunk, research/116 §3):
//! * [`TeleportDustEnv`] is g61's `SpawnVisualsEnv` MINUS the gore pool —
//!   no teleport arm spawns gores; the RNG parameter stays generic over
//!   [`DustRng`] so the recording witnesses ride the same internals.
//! * The three pool helpers (`new_dust_perfect`/`new_dust_direct`/
//!   `clone_dust`) take g61's `DustRng` generalisation
//!   (`particles.rs`, behaviour-identical — `UnifiedRandom` implements
//!   the trait).
//! * g61's env-level `npc_teleport` driver is RETIRED here — g11's
//!   App-level `main_dust::npc_teleport` replay supersedes it (it carries
//!   `other_position` per research/116 gap 2 and the `Npc` teleport
//!   fields instead of g61's `(f32, i32, i32)` triple).
//! * [`update_npc_teleport_visuals`] takes the g11 `Npc` fields
//!   (`npc.rs`: `teleport_time`/`teleport_style_id`/
//!   `last_portal_color_index`) instead of the triple; WIRED PRE-AI in
//!   the `UpdateNPC` slot preamble (`NPC.cs:90322` — after the cast
//!   lights `:90320`, BEFORE `AI()` `:90423`; `world_sim/npcs.rs`, the
//!   g61b seam-4 order fold, research/116 §5) on the `UpdateNPCs`
//!   stream, fed `Main.timeForVisualEffects` as the `anim_time*60 %
//!   216000` stand-in (g61b `c289d64f`'s shape).
//!
//! RNG contract (the d50 `DustPool` contract, S1-E-1): every draw rides
//! the caller's ONE stream in C# evaluation order — argument draws
//! BEFORE the allocator internals, post-write draws after (the 6000
//! sentinel takes the writes, `Dust.cs:79`). The `side` parameter is
//! unread in the whole C# body — dropped; `otherPosition` feeds only
//! style 10 (Entry leg = newPos, Exit leg = the old position,
//! `Player.cs:37444`/`:37489`, `NPC.cs:81044`/`:81051`).
//!
//! Witnesses: g61's nine teleport witnesses transferred with rig
//! adaptation (the draw sequences are identical per research/116 §1).
//! WRITTEN NOT RUN (code-only mode).
#![allow(dead_code)] // the visuals arm's style-12 branch has no live producer yet (the shimmer NPC lane's); the witnesses pin it

use crate::npc::Npc;
use glam::Vec2;
use tr_core::particles::{Color, DustPool, DustRng, SpawnCtx};
use tr_core::World;

/// The per-call environment the teleport dust reads through `Main`'s
/// statics — g61's `SpawnVisualsEnv` narrowed to the dust half (the
/// teleport arms spawn no gores).
pub(crate) struct TeleportDustEnv<'a, R: DustRng> {
    /// The read-only world the allocator's tile probes read.
    pub world: &'a World,
    /// `Main.dust` — the shared 6001-slot pool (S1-E-1).
    pub dust: &'a mut DustPool,
    /// The `Main` statics `NewDust` reads (`Dust.cs:77-98`).
    pub dust_ctx: &'a SpawnCtx,
    /// The caller's pass stream (`Main.rand`'s per-pass analogue — the
    /// dispatcher passes `app.rngs.stream(key)`).
    pub rng: &'a mut R,
    /// `SoundEngine.PlaySound(LegacySoundStyle, x, y)` at the C# call
    /// position: `(sound_id, style, x, y)`. The App dispatcher resolves
    /// the Audio façade + its pitch draws here (the cue draws ride the
    /// SAME stream mid-switch — a deferred sounds vec would re-order).
    pub play_sound: &'a mut dyn FnMut(i32, i32, f32, f32),
}

/// `Main.TeleportEffect` (`Main.cs:13233-13464`) — the teleport-in /
/// teleport-out burst family, all eleven C# arms. g61's body verbatim
/// (research/116 §1: draw-equivalent with every other lane on the shared
/// styles). Styles 9/10 `Inflate(15,15)` mutate the rect FIRST
/// (X−15, Y−15, W+30, H+30). `water_style` is the stored `Main.waterStyle`
/// static (`Main.cs:1105`, written by `DrawWaters` `:57475`) — the
/// dispatcher input, NOT an inline recompute (research/116 gap 4).
#[allow(clippy::too_many_arguments)]
pub(crate) fn teleport_effect<R: DustRng>(
    env: &mut TeleportDustEnv<'_, R>,
    mut rect: (i32, i32, i32, i32), // (X, Y, Width, Height)
    style: i32,
    extra_info: i32,
    dust_count_mult: f32,
    other_position: Vec2,
    water_style: i32,
) {
    match style {
        0 | 1 => {
            // `:13237-13274` — the wizard (0: type 159, Item6) and
            // rod-of-discord (1: type 164, Item8) arms share the shape:
            // the INTEGER `Width*Height/5` count ×mult, per dust the
            // allocator then the `Next(20,70)*0.01` scale ASSIGNMENT +
            // the j<10 / j<5 +0.25 bumps.
            let dust_type = if style == 0 { 159 } else { 164 };
            let sound_style = if style == 0 { 6 } else { 8 };
            (env.play_sound)(
                2,
                sound_style,
                (rect.0 + rect.2 / 2) as f32,
                (rect.1 + rect.3 / 2) as f32,
            );
            let count = ((rect.2 * rect.3 / 5) as f32 * dust_count_mult) as i32;
            for j in 0..count {
                let idx = env.dust.new_dust(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    Vec2::new(rect.0 as f32, rect.1 as f32),
                    rect.2,
                    rect.3,
                    dust_type,
                    0.0,
                    0.0,
                    0,
                    Color::default(),
                    1.0,
                );
                let mut scale = DustRng::next_range(env.rng, 20, 70) as f32 * 0.01;
                if j < 10 {
                    scale += 0.25;
                }
                if j < 5 {
                    scale += 0.25;
                }
                env.dust.slot_mut(idx).scale = scale;
            }
        }
        2 => {
            // `:13275-13283` — the Chaos-State ghost puff: 50×mult
            // type-58 dust, alpha 150, Color.GhostWhite (248,248,255,
            // 255), scale 1.2, velocity ×0.5. NO sound. Type 58 hits
            // no tweak arm (the 59..=65 band starts above it).
            let count = (50.0f32 * dust_count_mult) as i32;
            for _ in 0..count {
                let idx = env.dust.new_dust(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    Vec2::new(rect.0 as f32, rect.1 as f32),
                    rect.2,
                    rect.3,
                    58,
                    0.0,
                    0.0,
                    150,
                    Color::new(248, 248, 255, 255),
                    1.2,
                );
                env.dust.slot_mut(idx).velocity *= 0.5;
            }
        }
        3 => {
            // `:13284-13317` — the King-Slime-style teleport: 50×mult
            // type-180 dust, noGravity, then the FIVE `Next(3)==0`
            // ×0.75 rolls (each draws), two `Next(3)==0` ×2/×1.2
            // bumps, one `Next(2)==0` gate whose body draws
            // Next(75,100) fadeIn then Next(25,75) scale, and the
            // unconditional ×0.8 scale tail. Item6 first.
            (env.play_sound)(
                2,
                6,
                (rect.0 + rect.2 / 2) as f32,
                (rect.1 + rect.3 / 2) as f32,
            );
            let count = (50.0f32 * dust_count_mult) as i32;
            for _ in 0..count {
                let idx = env.dust.new_dust(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    Vec2::new(rect.0 as f32, rect.1 as f32),
                    rect.2,
                    rect.3,
                    180,
                    0.0,
                    0.0,
                    0,
                    Color::default(),
                    1.0,
                );
                env.dust.slot_mut(idx).no_gravity = true;
                for _ in 0..5 {
                    if DustRng::next_max(env.rng, 3) == 0 {
                        env.dust.slot_mut(idx).velocity *= 0.75;
                    }
                }
                for _ in 0..2 {
                    if DustRng::next_max(env.rng, 3) == 0 {
                        let d = env.dust.slot_mut(idx);
                        d.velocity *= 2.0;
                        d.scale *= 1.2;
                    }
                }
                if DustRng::next_max(env.rng, 2) == 0 {
                    let fade = DustRng::next_range(env.rng, 75, 100) as f32 * 0.01;
                    let scale = DustRng::next_range(env.rng, 25, 75) as f32 * 0.01;
                    let d = env.dust.slot_mut(idx);
                    d.fade_in = fade;
                    d.scale = scale;
                }
                env.dust.slot_mut(idx).scale *= 0.8;
            }
        }
        4 => {
            // `:13318-13333` — the portal pair: W*H/5×mult type-263
            // dust coloured by GetPortalColor(extraInfo), noLight +
            // noGravity + scale 1.2 + fadeIn 0.4, then the alpha-255
            // overwrite. Item8 first. Type 263 hits no tweak arm.
            (env.play_sound)(
                2,
                8,
                (rect.0 + rect.2 / 2) as f32,
                (rect.1 + rect.3 / 2) as f32,
            );
            let count = ((rect.2 * rect.3 / 5) as f32 * dust_count_mult) as i32;
            let mut color = portal_color(extra_info);
            color.a = 255; // `:13330`
            for _ in 0..count {
                let idx = env.dust.new_dust(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    Vec2::new(rect.0 as f32, rect.1 as f32),
                    rect.2,
                    rect.3,
                    263,
                    0.0,
                    0.0,
                    0,
                    Color::default(),
                    1.0,
                );
                let d = env.dust.slot_mut(idx);
                d.color = color;
                d.no_light = true;
                d.no_gravity = true;
                d.scale = 1.2;
                d.fade_in = 0.4;
            }
        }
        5 | 7 => {
            // `:13334-13369` — the water (5: dustWater(), sound 19)
            // and lava (7: type 35, Item8) columns: 100×mult / 50×mult
            // NewDustDirect over the height+24 box, velocity.Y ×0
            // then −3.5, velocity.X ×1.5, scale 0.8, alpha 130,
            // noGravity, fadeIn 1.2. The sound fires AFTER the loop
            // (both arms — `:13349`/`:13367`), at the rect Centre
            // (float halves), style 5's volume argument 0.
            let (dust_type, count) = if style == 5 {
                (
                    tr_core::particles::dust_water(water_style),
                    (100.0f32 * dust_count_mult) as i32,
                )
            } else {
                (35, (50.0f32 * dust_count_mult) as i32)
            };
            for _ in 0..count {
                let idx = env.dust.new_dust_direct(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    Vec2::new(rect.0 as f32, rect.1 as f32),
                    rect.2,
                    rect.3 + 24,
                    dust_type,
                    0.0,
                    0.0,
                    0,
                    Color::default(),
                    1.0,
                );
                let d = env.dust.slot_mut(idx);
                d.velocity.y *= 0.0;
                d.velocity.y -= 3.5;
                d.velocity.x *= 1.5;
                d.scale = 0.8;
                d.alpha = 130;
                d.no_gravity = true;
                d.fade_in = 1.2;
            }
            // `Rectangle.Center` is INTEGER halves (X + Width/2).
            let cx = (rect.0 + rect.2 / 2) as f32;
            let cy = (rect.1 + rect.3 / 2) as f32;
            if style == 5 {
                (env.play_sound)(19, 0, cx, cy); // PlaySound(19, .., 0)
            } else {
                (env.play_sound)(2, 8, cx, cy);
            }
        }
        9 => {
            // `:13370-13380` — the pylon hop: Inflate(15,15) FIRST,
            // then 100×mult SpawnInWorldDust(extraInfo, rect) calls;
            // Item6 after, at the INFLATED integer centre.
            rect = (rect.0 - 15, rect.1 - 15, rect.2 + 30, rect.3 + 30);
            let count = (100.0f32 * dust_count_mult) as i32;
            for _ in 0..count {
                pylon_spawn_in_world_dust(env, extra_info, rect);
            }
            (env.play_sound)(2, 6, (rect.0 + rect.2 / 2) as f32, (rect.1 + rect.3 / 2) as f32);
        }
        10 => {
            // `:13381-13418` — the Rod-of-Harmony shimmer: Inflate,
            // 60×mult, per dust the fadeIn/scale NextFloat pair FIRST
            // (`:13388-13389`), then the hsl colour NextFloat inside
            // the NewDustDirect argument (`:13390`, drawn before the
            // allocator), the Next(20,70) scale roll + the n<10/n<5
            // bumps (OVERWRITTEN below — the rolls still run), the
            // n < count×0.8 velocity lerp toward otherPosition with
            // its NextFloat, the noGravity/noLight flags, the scale/
            // fadeIn ASSIGNMENTS, and the dustIndex != 6000 clone
            // (CloneDust's own allocator draws + the ×0.65 pair + the
            // white colour). Item8 after.
            rect = (rect.0 - 15, rect.1 - 15, rect.2 + 30, rect.3 + 30);
            let count = (60.0f32 * dust_count_mult) as i32;
            let vector = other_position - Vec2::new(rect.0 as f32, rect.1 as f32);
            for n in 0..count {
                let fade_in = 0.4 + DustRng::next_float(env.rng);
                let scale = 0.4 + DustRng::next_float(env.rng);
                let hue = 0.66 + DustRng::next_float(env.rng) * 0.24;
                let rgb = crate::ui::color::hsl_to_rgb(hue as f64, 1.0, 0.5, 255);
                let idx = env.dust.new_dust_direct(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    Vec2::new(rect.0 as f32, rect.1 as f32),
                    rect.2,
                    rect.3,
                    267,
                    0.0,
                    0.0,
                    127,
                    Color::new(rgb[0], rgb[1], rgb[2], rgb[3]),
                    1.0,
                );
                let mut rolled = DustRng::next_range(env.rng, 20, 70) as f32 * 0.01;
                if n < 10 {
                    rolled += 0.25;
                }
                if n < 5 {
                    rolled += 0.25;
                }
                env.dust.slot_mut(idx).scale = rolled;
                if (n as f32) < (count as f32) * 0.8 {
                    let jitter = DustRng::next_float(env.rng);
                    env.dust.slot_mut(idx).velocity += vector * 0.1 * jitter;
                }
                {
                    let d = env.dust.slot_mut(idx);
                    d.no_gravity = true;
                    d.no_light = true;
                    d.scale = scale;
                    d.fade_in = fade_in;
                }
                // `:13408-13414` — the clone guard reads the SPAWNED
                // slot's dustIndex (the sentinel refuses).
                if env.dust.slot(idx).dust_index != tr_core::particles::DUST_SENTINEL {
                    let clone = env.dust.clone_dust(env.rng, env.dust_ctx, env.world, idx);
                    let d = env.dust.slot_mut(clone);
                    d.scale *= 0.65;
                    d.fade_in *= 0.65;
                    d.color = Color::WHITE;
                }
            }
            (env.play_sound)(2, 8, (rect.0 + rect.2 / 2) as f32, (rect.1 + rect.3 / 2) as f32);
        }
        11 => {
            // `:13419-13446` — the mystic-frog / shellphone hop: 50
            // dust (NO mult), per dust the `Next(4)` colour pick
            // FIRST, then the position's TWO NextFloats
            // (`Utils.NextVector2FromRectangle`, `Utils.cs:1256-1259`)
            // inside the NewDustPerfect argument, the allocator, then
            // noGravity / colour / velocity ×2 / scale
            // 0.8+NextFloat×0.6 / fadeIn 0.5. NO sound.
            for _ in 0..50 {
                let num4 = DustRng::next_max(env.rng, 4);
                let color = match num4 {
                    0 | 1 => Color::new(100, 255, 100, 255),
                    2 => Color::new(255, 255, 0, 255), // Color.Yellow
                    _ => Color::WHITE,                 // 3: Color.White
                };
                let px = rect.0 as f32 + DustRng::next_float(env.rng) * rect.2 as f32;
                let py = rect.1 as f32 + DustRng::next_float(env.rng) * rect.3 as f32;
                let idx = env.dust.new_dust_perfect(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    Vec2::new(px, py),
                    267,
                    None,
                    0,
                    Color::default(),
                    1.0,
                );
                let scale = 0.8 + DustRng::next_float(env.rng) * 0.6;
                let d = env.dust.slot_mut(idx);
                d.no_gravity = true;
                d.color = color;
                d.velocity *= 2.0;
                d.scale = scale;
                d.fade_in = 0.5;
            }
        }
        13 => {
            // `:13447-13462` — the frog's style-13 sparkle: Item8
            // first (integer Centre), then 21 dust (NO mult): the
            // position's two NextFloats, the `Next(1,3)` SCALE
            // argument (drawn before the allocator), a 2×2 type-27
            // box (alpha 100), velocity ×1.75, the i%3==0 ×1.5 bump,
            // noLightEmittance + noGravity.
            (env.play_sound)(2, 8, (rect.0 + rect.2 / 2) as f32, (rect.1 + rect.3 / 2) as f32);
            for i in 0..21 {
                let px = rect.0 as f32 + DustRng::next_float(env.rng) * rect.2 as f32;
                let py = rect.1 as f32 + DustRng::next_float(env.rng) * rect.3 as f32;
                let scale_arg = DustRng::next_range(env.rng, 1, 3) as f32;
                let idx = env.dust.new_dust(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    Vec2::new(px, py),
                    2,
                    2,
                    27,
                    0.0,
                    0.0,
                    100,
                    Color::default(),
                    scale_arg,
                );
                let d = env.dust.slot_mut(idx);
                d.velocity *= 1.75;
                if i % 3 == 0 {
                    d.velocity *= 1.5;
                }
                d.no_light_emittance = true;
                d.no_gravity = true;
            }
        }
        _ => {
            // Styles 6/8/12 have no C# arm — the switch falls through
            // with zero draws.
        }
    }
}

/// `PortalHelper.GetPortalColor(int colorIndex)`
/// (`PortalHelper.cs:373-392`), the `netMode == 0` single-player arm:
/// `colorIndex / 2` is the player (unread here), `colorIndex % 2` the
/// portal — 0 → hsl(0.12, 1, 0.5), else hsl(0.52, 1, 0.6); alpha 66.
/// The canonical `ui::color::hsl_to_rgb` (research/116 §1 style-4 row).
pub(crate) fn portal_color(color_index: i32) -> Color {
    let rgb = if color_index % 2 != 0 {
        crate::ui::color::hsl_to_rgb(0.52, 1.0, 0.6, 66)
    } else {
        crate::ui::color::hsl_to_rgb(0.12, 1.0, 0.5, 66)
    };
    Color::new(rgb[0], rgb[1], rgb[2], 66)
}

/// `TeleportPylonsSystem.SpawnInWorldDust`
/// (`TeleportPylonsSystem.cs:327-393`): the per-pylon-type colour
/// (the `TeleportPylonType` byte enum, `TeleportPylonType.cs:3-17`),
/// one type-43 dust (alpha 254, scale 0.5), velocity ×0.1 then
/// Y −0.2. The float-ctor colour `new Color(r,g,b,1f)` multiplies by
/// 255 and truncates (`(byte)(v*255)`); the Snow/Mushroom 1.2 blue
/// clamps at 255 (XNA's float ctor clamps to [0,1]).
fn pylon_spawn_in_world_dust<R: DustRng>(
    env: &mut TeleportDustEnv<'_, R>,
    tile_style: i32,
    rect: (i32, i32, i32, i32),
) {
    // `:329-389` — the colour table, default (1,1,1).
    let (r, g, b) = match tile_style {
        0 | 9 | 10 => (0.05, 0.8, 0.3),  // SurfacePurity/Underworld/Shimmer
        1 => (0.7, 0.8, 0.05),          // Jungle
        2 => (0.5, 0.3, 0.7),           // Hallow
        3 => (0.4, 0.4, 0.6),           // Underground
        4 => (0.2, 0.2, 0.95),          // Beach
        5 => (0.85, 0.45, 0.1),         // Desert
        6 => (1.0, 1.0, 1.2),           // Snow
        7 => (0.4, 0.7, 1.2),           // GlowingMushroom
        8 => (0.7, 0.7, 0.7),           // Victory
        _ => (1.0, 1.0, 1.0),
    };
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0) as u8;
    // `:390` — NewDust(TopLeft, W, H, 43, 0, 0, 254, color, 0.5).
    let idx = env.dust.new_dust(
        env.rng,
        env.dust_ctx,
        env.world,
        Vec2::new(rect.0 as f32, rect.1 as f32),
        rect.2,
        rect.3,
        43,
        0.0,
        0.0,
        254,
        Color::new(byte(r), byte(g), byte(b), 255),
        0.5,
    );
    // `:391-392`.
    let d = env.dust.slot_mut(idx);
    d.velocity *= 0.1;
    d.velocity.y -= 0.2;
}

/// `NPC.UpdateNPC_TeleportVisuals` (`NPC.cs:92194-92241`) — the
/// per-tick teleport fade, called from `UpdateNPC` (`:90322`) after the
/// cast lights (`:90320`) and BEFORE `AI()` (`:90423`), on the
/// `UpdateNPCs` stream. WIRED PRE-AI in the `world_sim/npcs.rs` slot
/// preamble (the g61b seam-4 order fold, research/116 §5). The teleport
/// fields are the g11
/// `Npc` fields (research/116 §2 — they replace g61's triple);
/// `time_for_visual_effects` is `Main.timeForVisualEffects`
/// (`Main.cs:1181`, a double).
///
/// * Style 0 (`:92200-92209`): the `Next(100)` gate against
///   `100×time×2`, then ONE type-159 dust at the getRect box with
///   scale = time×1.5, noGravity, velocity ×1.1. NO time decrement
///   inside the arm — only the shared −0.005 tail.
/// * Style 4 (`:92210-92222`): time −0.02 FIRST, the `Next(100)` gate
///   against `100×time` (the DECREMENTED value), then a type-263 dust
///   coloured `GetPortalColor(lastPortalColorIndex)` — alpha stays 66
///   (no overwrite here, unlike the TeleportEffect burst) — noLight,
///   noGravity, scale 1.2, fadeIn 0.4.
/// * Style 12 (`:92223-92235`): the same shape, colour
///   `hslToRgb((float)(timeForVisualEffects/60.0) % 1f, 1, 0.75, 0)`
///   — alpha 0.
/// * Style 13 (`:92236-92239`): time = 0, no dust.
/// * The tail (`:92240`): `time = max(0, time − 0.005)` runs for EVERY
///   style that entered (the `time > 0` head gate `:92196-92199`
///   returns before it otherwise).
pub(crate) fn update_npc_teleport_visuals<R: DustRng>(
    env: &mut TeleportDustEnv<'_, R>,
    npc: &mut Npc,
    time_for_visual_effects: f64,
) {
    // `:92196-92199` — the head gate: no draws, no decrement.
    if !(npc.teleport_time > 0.0) {
        return;
    }
    let pos = Vec2::new(npc.x, npc.y);
    let (wf, hf) = npc.size();
    let (w, h) = (wf as i32, hf as i32);
    match npc.teleport_style_id {
        0 => {
            // `:92202` — the gate compares the int roll as float.
            if (DustRng::next_max(env.rng, 100) as f32) <= 100.0 * npc.teleport_time * 2.0 {
                // `:92204` — the box origin is getRect().X/Y: the
                // INTEGER-cast position (`NPC.cs:90027`).
                let rect_pos = Vec2::new((npc.x as i32) as f32, (npc.y as i32) as f32);
                let idx = env.dust.new_dust(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    rect_pos,
                    w,
                    h,
                    159,
                    0.0,
                    0.0,
                    0,
                    Color::default(),
                    1.0,
                );
                let d = env.dust.slot_mut(idx);
                d.scale = npc.teleport_time * 1.5;
                d.no_gravity = true;
                d.velocity *= 1.1;
            }
        }
        4 => {
            // `:92212-92213` — decrement BEFORE the gate.
            npc.teleport_time -= 0.02;
            if (DustRng::next_max(env.rng, 100) as f32) <= 100.0 * npc.teleport_time {
                let idx = env.dust.new_dust(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    pos,
                    w,
                    h,
                    263,
                    0.0,
                    0.0,
                    0,
                    Color::default(),
                    1.0,
                );
                let d = env.dust.slot_mut(idx);
                d.color = portal_color(npc.last_portal_color_index);
                d.no_light = true;
                d.no_gravity = true;
                d.scale = 1.2;
                d.fade_in = 0.4;
            }
        }
        12 => {
            // `:92225-92226`.
            npc.teleport_time -= 0.02;
            if (DustRng::next_max(env.rng, 100) as f32) <= 100.0 * npc.teleport_time {
                let hue = ((time_for_visual_effects / 60.0) as f32) % 1.0;
                let rgb = crate::ui::color::hsl_to_rgb(hue as f64, 1.0, 0.75, 0);
                let idx = env.dust.new_dust(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    pos,
                    w,
                    h,
                    263,
                    0.0,
                    0.0,
                    0,
                    Color::default(),
                    1.0,
                );
                let d = env.dust.slot_mut(idx);
                d.color = Color::new(rgb[0], rgb[1], rgb[2], rgb[3]);
                d.no_light = true;
                d.no_gravity = true;
                d.scale = 1.2;
                d.fade_in = 0.4;
            }
        }
        13 => {
            // `:92238` — the frog's instant clear.
            npc.teleport_time = 0.0;
        }
        _ => {}
    }
    // `:92240` — the shared decay tail (Math.Max(0f, ..)).
    npc.teleport_time = (npc.teleport_time - 0.005).max(0.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::particles::DustRng;
    use tr_core::UnifiedRandom;

    /// One recorded stream call — the exact RNG-slot witness carrier
    /// (g61's `RecordingRng`, adapted: no `GoreRng` impl — the teleport
    /// env has no gore pool).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Draw {
        Max(i32),
        Range(i32, i32),
        Float,
    }

    /// The recording seam: forwards every draw to a real
    /// `UnifiedRandom` and logs the call shape, so a witness pins the
    /// C#'s draw ORDER without hand-tracing values.
    struct RecordingRng {
        inner: UnifiedRandom,
        log: Vec<Draw>,
    }

    impl RecordingRng {
        fn new(seed: i32) -> Self {
            Self {
                inner: UnifiedRandom::new(seed),
                log: Vec::new(),
            }
        }
    }

    impl DustRng for RecordingRng {
        fn next_max(&mut self, max: i32) -> i32 {
            self.log.push(Draw::Max(max));
            self.inner.next_max(max)
        }
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.log.push(Draw::Range(min, max));
            self.inner.next_range(min, max)
        }
        fn next_float(&mut self) -> f32 {
            self.log.push(Draw::Float);
            self.inner.next_float()
        }
    }

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    /// On-screen ctx: the (100,100) box sits inside the 400-px cull
    /// margin; maxDustToDraw 6000 puts slot 0 in the low band (no gate
    /// draw) — the style56 witness convention.
    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
        }
    }

    /// The six `NewDust` default-block draws for a w×h box
    /// (`Dust.cs:170-190`): pos X/Y, vel X/Y, frame row, scale.
    fn dust_block(w: i32, h: i32) -> Vec<Draw> {
        let nw = w.max(5) - 4;
        let nh = h.max(5) - 4;
        vec![
            Draw::Max(nw),
            Draw::Max(nh),
            Draw::Range(-20, 21),
            Draw::Range(-20, 21),
            Draw::Max(3),
            Draw::Range(-20, 21),
        ]
    }

    /// g61 `w4_g61_teleport_style0_count_scale_bumps_and_slots`
    /// (union-adapted): `Main.cs:13240-13252` — styles 0/1: the INTEGER
    /// W*H/5 count, per dust the block then the Next(20,70) scale roll;
    /// the j<10 / j<5 bumps stack (+0.5 for the first five).
    #[test]
    fn w4_g11u_style0_count_scale_bumps_and_slots() {
        let w = air();
        let mut dust = DustPool::new();
        let c = ctx();
        let mut rng = RecordingRng::new(29);
        let mut sounds = Vec::new();
        let mut env = TeleportDustEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            rng: &mut rng,
            play_sound: &mut |a, b, x, y| sounds.push((a, b, x, y)),
        };
        // 20×40 → 800/5 = 160 dust; mult 1.
        teleport_effect(&mut env, (100, 100, 20, 40), 0, 0, 1.0, Vec2::ZERO, 0);
        let mut expected = Vec::new();
        for _ in 0..160 {
            expected.extend(dust_block(20, 40));
            expected.push(Draw::Range(20, 70));
        }
        assert_eq!(rng.log, expected);
        assert_eq!(sounds, vec![(2, 6, 110.0, 120.0)]);
        let mut replay = UnifiedRandom::new(29);
        for j in 0..160 {
            // consume the block draws (shapes pinned by the log above)
            let _ = replay.next_max(16);
            let _ = replay.next_max(36);
            let _ = replay.next_range(-20, 21);
            let _ = replay.next_range(-20, 21);
            let _ = replay.next_max(3);
            let _ = replay.next_range(-20, 21);
            let mut want = replay.next_range(20, 70) as f32 * 0.01;
            if j < 10 {
                want += 0.25;
            }
            if j < 5 {
                want += 0.25;
            }
            let d = dust.slot(j);
            assert_eq!(d.dust_type, 159);
            assert_eq!(d.scale, want);
        }
        // The 0.3 re-teleport mult truncates: 160×0.3 = 48.
        assert_eq!((160.0f32 * 0.3) as i32, 48);
    }

    /// g61 `w4_g61_teleport_style3_gate_chain_and_scale_tail`
    /// (union-adapted): `Main.cs:13284-13317` — per dust the block +
    /// five Next(3) + two Next(3) + one Next(2), and the Next(2) body's
    /// Next(75,100)/Next(25,75) pair only when the gate opens; the
    /// unconditional ×0.8 scale tail.
    #[test]
    fn w4_g11u_style3_gate_chain_and_scale_tail() {
        let w = air();
        let mut dust = DustPool::new();
        let c = ctx();
        let mut rng = RecordingRng::new(31);
        let mut env = TeleportDustEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            rng: &mut rng,
            play_sound: &mut |_, _, _, _| {},
        };
        // mult 0.1 → 5 dust (keeps the witness log readable).
        teleport_effect(&mut env, (100, 100, 20, 40), 3, 0, 0.1, Vec2::ZERO, 0);
        let mut replay = UnifiedRandom::new(31);
        let mut expected = Vec::new();
        let mut gates_opened = 0;
        for i in 0..5 {
            expected.extend(dust_block(20, 40));
            // block replay
            let _ = replay.next_max(16);
            let _ = replay.next_max(36);
            let mut vx = replay.next_range(-20, 21) as f32 * 0.1;
            let mut vy = replay.next_range(-20, 21) as f32 * 0.1;
            let _ = replay.next_max(3);
            let mut scale = 1.0 + replay.next_range(-20, 21) as f32 * 0.01;
            for _ in 0..5 {
                expected.push(Draw::Max(3));
                if replay.next_max(3) == 0 {
                    vx *= 0.75;
                    vy *= 0.75;
                }
            }
            for _ in 0..2 {
                expected.push(Draw::Max(3));
                if replay.next_max(3) == 0 {
                    vx *= 2.0;
                    vy *= 2.0;
                    scale *= 1.2;
                }
            }
            expected.push(Draw::Max(2));
            let mut fade = 0.0f32;
            if replay.next_max(2) == 0 {
                gates_opened += 1;
                expected.push(Draw::Range(75, 100));
                expected.push(Draw::Range(25, 75));
                fade = replay.next_range(75, 100) as f32 * 0.01;
                scale = replay.next_range(25, 75) as f32 * 0.01;
            }
            scale *= 0.8;
            let d = dust.slot(i);
            assert_eq!(d.dust_type, 180);
            assert!(d.no_gravity);
            assert_eq!(d.velocity.x, vx);
            assert_eq!(d.velocity.y, vy);
            assert_eq!(d.scale, scale);
            assert_eq!(d.fade_in, fade);
        }
        assert_eq!(rng.log, expected);
        let _ = gates_opened;
    }

    /// g61 `w4_g61_teleport_style4_portal_colour_and_alpha`
    /// (union-adapted): `Main.cs:13318-13333` + `PortalHelper.cs:378-392`
    /// — the portal colour by index parity, alpha 66 → 255 overwrite,
    /// the fixed scale/fadeIn, no allocator-side draws beyond the block.
    #[test]
    fn w4_g11u_style4_portal_colour_and_alpha() {
        let w = air();
        let mut dust = DustPool::new();
        let c = ctx();
        let mut rng = RecordingRng::new(37);
        let mut env = TeleportDustEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            rng: &mut rng,
            play_sound: &mut |_, _, _, _| {},
        };
        // 10×10 → 20 dust; extraInfo 3 → portal 1 (the 0.52 hue).
        teleport_effect(&mut env, (100, 100, 10, 10), 4, 3, 1.0, Vec2::ZERO, 0);
        let mut expected = Vec::new();
        for _ in 0..20 {
            expected.extend(dust_block(10, 10));
        }
        assert_eq!(rng.log, expected, "the colour is computed, not rolled");
        let want = crate::ui::color::hsl_to_rgb(0.52, 1.0, 0.6, 66);
        for i in 0..20 {
            let d = dust.slot(i);
            assert_eq!(d.dust_type, 263);
            assert_eq!((d.color.r, d.color.g, d.color.b), (want[0], want[1], want[2]));
            assert_eq!(d.color.a, 255, ":13330 overwrites the 66");
            assert!(d.no_light && d.no_gravity);
            assert_eq!(d.scale, 1.2);
            assert_eq!(d.fade_in, 0.4);
        }
        // Parity: index 2 → portal 0 (the 0.12 hue).
        let p0 = portal_color(2);
        let w0 = crate::ui::color::hsl_to_rgb(0.12, 1.0, 0.5, 66);
        assert_eq!((p0.r, p0.g, p0.b, p0.a), (w0[0], w0[1], w0[2], 66));
    }

    /// g61 `w4_g61_teleport_style9_inflate_and_pylon_colour`
    /// (union-adapted): `Main.cs:13370-13380` +
    /// `TeleportPylonsSystem.cs:327-393` — style 9 inflates FIRST (the
    /// dust box AND the sound centre read the inflated rect), 100×mult
    /// type-43 dust with the per-type colour and the ×0.1 / Y−0.2
    /// velocity tail.
    #[test]
    fn w4_g11u_style9_inflate_and_pylon_colour() {
        let w = air();
        let mut dust = DustPool::new();
        let c = ctx();
        let mut rng = RecordingRng::new(41);
        let mut sounds = Vec::new();
        let mut env = TeleportDustEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            rng: &mut rng,
            play_sound: &mut |a, b, x, y| sounds.push((a, b, x, y)),
        };
        // mult 0.05 → 5 dust; style 6 (Snow) — the 1.2 blue clamps.
        teleport_effect(&mut env, (100, 100, 20, 40), 9, 6, 0.05, Vec2::ZERO, 0);
        let mut expected = Vec::new();
        for _ in 0..5 {
            expected.extend(dust_block(50, 70)); // the inflated box
        }
        assert_eq!(rng.log, expected);
        // inflated centre: (100−15 + 50/2, 100−15 + 70/2) = (110, 120).
        assert_eq!(sounds, vec![(2, 6, 110.0, 120.0)]);
        for i in 0..5 {
            let d = dust.slot(i);
            assert_eq!(d.dust_type, 43);
            assert_eq!(d.alpha, 254);
            assert_eq!((d.color.r, d.color.g, d.color.b, d.color.a), (255, 255, 255, 255));
            assert!(d.position.x >= 85.0 + 4.0 && d.position.x <= 85.0 + 49.0);
        }
    }

    /// g61 `w4_g61_teleport_style10_float_triple_clone_and_overwrites`
    /// (union-adapted): `Main.cs:13381-13418` — the fadeIn/scale/hue
    /// NextFloat triple BEFORE the allocator, the overwritten scale
    /// roll still drawn, the n<80% lerp gate, and the clone's own
    /// 0×0 block + ×0.65 pair + white colour.
    #[test]
    fn w4_g11u_style10_float_triple_clone_and_overwrites() {
        let w = air();
        let mut dust = DustPool::new();
        let c = ctx();
        let mut rng = RecordingRng::new(43);
        let mut env = TeleportDustEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            rng: &mut rng,
            play_sound: &mut |_, _, _, _| {},
        };
        // mult 0.05 → 3 dust (all n < 2.4 → the lerp gate opens).
        teleport_effect(&mut env, (100, 100, 20, 40), 10, 0, 0.05, Vec2::new(500.0, 300.0), 0);
        let mut expected = Vec::new();
        for _ in 0..3 {
            expected.push(Draw::Float); // fadeIn
            expected.push(Draw::Float); // scale
            expected.push(Draw::Float); // hue
            expected.extend(dust_block(50, 70));
            expected.push(Draw::Range(20, 70)); // the overwritten roll
            expected.push(Draw::Float); // the lerp jitter
            expected.extend(dust_block(0, 0)); // CloneDust's spawn
        }
        assert_eq!(rng.log, expected);
        // slot pairs: (primary, clone) — the clone is white, ×0.65.
        for pair in 0..3 {
            let p = dust.slot(pair * 2);
            let cl = dust.slot(pair * 2 + 1);
            assert_eq!(p.dust_type, 267);
            assert_eq!(p.alpha, 127);
            assert!(p.no_gravity && p.no_light);
            assert!(p.scale >= 0.4 && p.scale < 1.4);
            assert!(p.fade_in >= 0.4 && p.fade_in < 1.4);
            assert_eq!(cl.color, Color::WHITE);
            assert_eq!(cl.scale, p.scale * 0.65);
            assert_eq!(cl.fade_in, p.fade_in * 0.65);
            assert_eq!(cl.position, p.position);
        }
    }

    /// g61 `w4_g61_teleport_style11_colour_pick_and_position_floats`
    /// (union-adapted): `Main.cs:13419-13446` — per dust the Next(4)
    /// colour pick, the two position NextFloats, the 0×0 block, the
    /// scale NextFloat; the colour table and the ×2 velocity.
    #[test]
    fn w4_g11u_style11_colour_pick_and_position_floats() {
        let w = air();
        let mut dust = DustPool::new();
        let c = ctx();
        let mut rng = RecordingRng::new(53);
        let mut sounds = Vec::new();
        let mut env = TeleportDustEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            rng: &mut rng,
            play_sound: &mut |a, b, x, y| sounds.push((a, b, x, y)),
        };
        teleport_effect(&mut env, (100, 100, 20, 40), 11, 0, 1.0, Vec2::ZERO, 0);
        let mut expected = Vec::new();
        for _ in 0..50 {
            expected.push(Draw::Max(4));
            expected.push(Draw::Float);
            expected.push(Draw::Float);
            expected.extend(dust_block(0, 0));
            expected.push(Draw::Float);
        }
        assert_eq!(rng.log, expected);
        assert!(sounds.is_empty(), "style 11 plays no sound");
        let mut replay = UnifiedRandom::new(53);
        for i in 0..50 {
            let pick = replay.next_max(4);
            let px = 100.0 + replay.next_float() * 20.0;
            let py = 100.0 + replay.next_float() * 40.0;
            let _ = replay.next_max(1);
            let _ = replay.next_max(1);
            let vx = replay.next_range(-20, 21) as f32 * 0.1;
            let vy = replay.next_range(-20, 21) as f32 * 0.1;
            let _ = replay.next_max(3);
            let _ = replay.next_range(-20, 21);
            let scale = 0.8 + replay.next_float() * 0.6;
            let d = dust.slot(i);
            assert_eq!(d.dust_type, 267);
            assert_eq!(d.position, Vec2::new(px, py), "Perfect overwrites");
            let want = match pick {
                0 | 1 => Color::new(100, 255, 100, 255),
                2 => Color::new(255, 255, 0, 255),
                _ => Color::WHITE,
            };
            assert_eq!(d.color, want);
            assert_eq!(d.velocity, Vec2::new(vx * 2.0, vy * 2.0));
            assert_eq!(d.scale, scale);
            assert_eq!(d.fade_in, 0.5);
            assert!(d.no_gravity);
        }
    }

    /// g61 `w4_g61_teleport_visuals_gates_decrements_and_tail`
    /// (union-adapted to the g11 `Npc` fields): `NPC.cs:92200-92240` —
    /// style 0's ×2 gate with NO in-arm decrement, style 4's
    /// decrement-BEFORE-gate, the shared −0.005 tail, style 13's instant
    /// clear, and the time<=0 head gate drawing nothing.
    #[test]
    fn w4_g11u_visuals_gates_decrements_and_tail() {
        let w = air();
        // Style 0 at time 0.001: the gate bound is 0.2 — only a 0 roll
        // opens; either way NO in-arm decrement, then the −0.005 tail.
        {
            let mut npc = Npc::new(NpcKind::Creeper, 100.0, 100.0, 1, 0.0);
            npc.size_override = Some((20.0, 20.0));
            npc.teleport_time = 0.001;
            npc.teleport_style_id = 0;
            let mut dust = DustPool::new();
            let c = ctx();
            let mut rng = RecordingRng::new(59);
            let mut env = TeleportDustEnv {
                world: &w,
                dust: &mut dust,
                dust_ctx: &c,
                rng: &mut rng,
                play_sound: &mut |_, _, _, _| {},
            };
            update_npc_teleport_visuals(&mut env, &mut npc, 0.0);
            let gate = UnifiedRandom::new(59).next_max(100);
            let mut expected = vec![Draw::Max(100)];
            if (gate as f32) <= 0.2 {
                expected.extend(dust_block(20, 20));
            }
            assert_eq!(rng.log, expected);
            assert_eq!(npc.teleport_time, 0.0, "0.001 − 0.005 floors at 0");
        }
        // Style 4 at time 0.5: −0.02 first → gate bound 48; tail →
        // 0.475. The dust colour keeps alpha 66 (no overwrite).
        {
            let mut npc = Npc::new(NpcKind::Creeper, 100.0, 100.0, 1, 0.0);
            npc.size_override = Some((20.0, 20.0));
            npc.teleport_time = 0.5;
            npc.teleport_style_id = 4;
            npc.last_portal_color_index = 1;
            let mut dust = DustPool::new();
            let c = ctx();
            let mut rng = RecordingRng::new(61);
            let mut env = TeleportDustEnv {
                world: &w,
                dust: &mut dust,
                dust_ctx: &c,
                rng: &mut rng,
                play_sound: &mut |_, _, _, _| {},
            };
            update_npc_teleport_visuals(&mut env, &mut npc, 0.0);
            let gate = UnifiedRandom::new(61).next_max(100);
            let mut expected = vec![Draw::Max(100)];
            if (gate as f32) <= 48.0 {
                expected.extend(dust_block(20, 20));
                let d = dust.slot(0);
                assert_eq!(d.dust_type, 263);
                assert_eq!(d.color.a, 66, "the fade keeps the portal alpha");
                assert_eq!(d.scale, 1.2);
                assert_eq!(d.fade_in, 0.4);
            }
            assert_eq!(rng.log, expected);
            assert_eq!(npc.teleport_time, 0.5 - 0.02 - 0.005);
        }
        // Style 13: instant clear, zero draws, the tail keeps 0.
        {
            let mut npc = Npc::new(NpcKind::Creeper, 100.0, 100.0, 1, 0.0);
            npc.size_override = Some((20.0, 20.0));
            npc.teleport_time = 0.9;
            npc.teleport_style_id = 13;
            let mut dust = DustPool::new();
            let c = ctx();
            let mut rng = RecordingRng::new(67);
            let mut env = TeleportDustEnv {
                world: &w,
                dust: &mut dust,
                dust_ctx: &c,
                rng: &mut rng,
                play_sound: &mut |_, _, _, _| {},
            };
            update_npc_teleport_visuals(&mut env, &mut npc, 0.0);
            assert!(rng.log.is_empty());
            assert_eq!(npc.teleport_time, 0.0);
        }
        // time <= 0: the head gate returns before any draw/decrement.
        {
            let mut npc = Npc::new(NpcKind::Creeper, 100.0, 100.0, 1, 0.0);
            npc.size_override = Some((20.0, 20.0));
            npc.teleport_time = 0.0;
            npc.teleport_style_id = 0;
            let mut dust = DustPool::new();
            let c = ctx();
            let mut rng = RecordingRng::new(71);
            let mut env = TeleportDustEnv {
                world: &w,
                dust: &mut dust,
                dust_ctx: &c,
                rng: &mut rng,
                play_sound: &mut |_, _, _, _| {},
            };
            update_npc_teleport_visuals(&mut env, &mut npc, 0.0);
            assert!(rng.log.is_empty());
            assert_eq!(npc.teleport_time, 0.0);
        }
    }

    /// g61 `w4_g61_teleport_visuals_style12_clock_hue` (union-adapted):
    /// `NPC.cs:92223-92235` — style 12's colour is the visual-effects
    /// clock hue at luminosity 0.75, alpha 0.
    #[test]
    fn w4_g11u_visuals_style12_clock_hue() {
        let w = air();
        let mut dust = DustPool::new();
        let c = ctx();
        let mut npc = Npc::new(NpcKind::Creeper, 100.0, 100.0, 1, 0.0);
        npc.size_override = Some((20.0, 20.0));
        // Synthetic time 1.03 → decrement 1.01 → gate bound 101: the
        // Next(100) roll ALWAYS opens, so the colour check is
        // unconditional (at the in-game 1.0 a 99 roll would close it).
        npc.teleport_time = 1.03;
        npc.teleport_style_id = 12;
        let mut rng = RecordingRng::new(73);
        let mut env = TeleportDustEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            rng: &mut rng,
            play_sound: &mut |_, _, _, _| {},
        };
        // clock 90 → 90/60 = 1.5 → %1 = 0.5.
        update_npc_teleport_visuals(&mut env, &mut npc, 90.0);
        let want = crate::ui::color::hsl_to_rgb(0.5, 1.0, 0.75, 0);
        let d = dust.slot(0);
        assert_eq!(d.dust_type, 263);
        assert_eq!((d.color.r, d.color.g, d.color.b, d.color.a), (want[0], want[1], want[2], 0));
        assert_eq!(npc.teleport_time, 1.03 - 0.02 - 0.005);
    }
}

