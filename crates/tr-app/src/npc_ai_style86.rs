//! The aiStyle-86 family — the shadow-flame apparitions (`NPC.AI()`'s
//! `else if (this.aiStyle == 86)` arm, `NPC.cs:37221-37470`). W4-d32,
//! research/25 row 86 (MISSING → ported), CODE-ONLY MODE (no compiler
//! invoked; the witness tests below are written, not run).
//!
//! ## Census
//!
//! The census of `this.aiStyle = 86;` across the decompiled tree
//! returns exactly two NPC hits: `NPC.cs:15746` (the
//! ShadowFlameApparition 472, `ID/NPCID.cs:11617`, the Goblin
//! Summoner's cast spawn) and `NPC.cs:15772` (the
//! AncientCultistSquidhead 521, `ID/NPCID.cs:11719`, the cultist
//! ritual's squid). No dynamic `aiStyle` reassignment to 86 exists.
//! Neither kind has an `NpcKind` yet — the module lands UNWIRED (the
//! style73/style83 precedent); the machine keys on the vanilla
//! `npc_type` until the roster wave.
//!
//! ## The machine
//!
//! The head (`:37223-37233`): the alpha fade-in (−30/tick, floor 0),
//! `noGravity = true`, `noTileCollide = true`, `knockBackResist = 0f`
//! (returned in [`Style86Outcome::knock_back_resist`] — `Npc` has no
//! mutable kb field, the style74 precedent).
//!
//! The sibling repulsion (`:37234-37260`): every OTHER active
//! same-type NPC within 50 px of the centre pushes both bodies apart
//! along the normalized delta ×0.4 — the pool step mutates the
//! sibling's velocity too (the `style75_step` slot-aligned pool
//! precedent). The zero-delta guard (`:37246-37256`) tests the
//! NORMALIZED vector's components against 0 — a true zero delta
//! normalizes to NaN (XNA's unguarded `Normalize`), NaN != 0, so the
//! ±1 whoAmI tie-break only fires on an exact-zero normalized pair;
//! the port keeps the literal comparisons (the C# NaN propagation is
//! preserved, the `normalize_xna` style74 precedent).
//!
//! The 472 materialize (`:37261-37316`): while `localAI[0] < 120`,
//! the FIRST tick (`localAI[0] == 0`) plays `SoundID.Item8`
//! (`ID/SoundID.cs:1391`, (2, 8)), retargets, kicks `velocity.X` ±2
//! by `direction`, and spins the 20-dust shadow ring around
//! `Center - (0, 18)` under the netOffset round trip; then
//! `localAI[0]++` and the fade trickle — `(1 - localAI[0]/120) * 20`
//! gated `Next(5) == 0` body dusts (type 27, alpha 100, velocity
//! ×0.3 + body ×0.75, noGravity).
//!
//! The 521 materialize (`:37317-37342`): the same first-tick
//! sound/retarget/kick (NO ring), `localAI[0]++`, then TWO type-228
//! box dusts per fading tick (the ±10 box inflation, alpha 100,
//! scale 2, noGravity + noLight) — and the 521 tail
//! (`:37458-37469`) repeats ONE such dust EVERY tick, all states.
//!
//! The four-state hover (`:37343-37457`): state 0 latches
//! `ai[1] = direction`; state 1 strafes along `ai[1]` (the 472 row
//! 0.3 accel / 7 cap / 4 Y-clamp / 660 X-threshold / 4 Y-divisor; the
//! 521 row 0.7 / 14 / 6 / 500 / 3 — the Y-divisor jumps to 15 when
//! the raw Y delta exceeds the clamp), flipping to state 2 past the
//! X-threshold with the ±1 Y-side pick (`Center.Y + 20 > player Y`);
//! state 2 dives along `ai[1]` (0.4/0.95/5 vs 0.3/0.9/7), exiting to
//! state 3 when `velocity.X` enters (−1, 1); state 3 climbs back
//! (0.4 X / 0.2 Y / 5 / 0.95 vs 0.6 / 0.3 / 7 / 0.9), exiting to
//! state 0 when `velocity.Y` enters (−1, 1). Every exit retargets.
//!
//! ## RNG census (by construction)
//!
//! The arm's explicit `Main.rand` draws, in C# order:
//! 1. The 472 first-tick ring (`:37283-37292`), 20 iterations of
//!    FIVE draws each: `Next(-100, 101)` X, `Next(-100, 101)` Y,
//!    `Next(0, 100)` radius, `Next(50, 90)` speed, `Next(2)` the
//!    noGravity/scale pick — with the `Dust.NewDust` internal draws
//!    (`Dust.cs:75-256`, the real `DustPool::new_dust`) between the
//!    fourth and fifth of each iteration.
//! 2. The 472 fade trickle (`:37302-37314`): one `Next(5)` per
//!    iteration of the float-bounded loop (`num1205 < num1204`,
//!    strict), each success spawning one NewDust 27 (internal
//!    draws).
//! 3. The 521 fade pair + tail (`:37337`/`:37464`): NO explicit
//!    draws — only the NewDust 228 internals.
//!
//! The states draw NOTHING (`TargetClosest` is deterministic,
//! `npc_target.rs`). The witnesses isolate the explicit draws with
//! the menu ctx (NewDust refuses BEFORE any draw, `Dust.cs:77` — the
//! style74 `menu_ctx` precedent) and replay them against a mirrored
//! oracle stream; the full-pool witnesses pin the interleaved
//! stream on cloned twins.
//!
//! ## Seams (the wiring obligations)
//!
//! * Both kinds integrate raw (noGravity/noTileCollide every tick) —
//!   the caller's movement step after `AI()` (the style74 split).
//! * The pool's `types` slice is the slot-aligned vanilla type table
//!   (the wiring step supplies it from the roster; the fixtures pass
//!   literal rows). `active` ≈ `hp > 0` (the style75 `host_is`
//!   stand-in).
//! * `SoundID.Item8` is returned as [`Style86Outcome::sound_item8`]
//!   — the audio pass owns its draws.
//! * The `netOffset` round trips are verbatim (`npc.net_offset`).
//!
//! See docs/ANALYSIS.md "The shadow-flame apparitions (W4-d32
//! aiStyle-86, code-only)".
#![allow(dead_code)] // wiring deferred — the kinds + dispatch arm land with the roster wave

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// The type-472 `SetDefaults` row (`NPC.cs:15742-15755`), verbatim.
pub struct ShadowFlameDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit52` (`ID/SoundID.cs:1308`).
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath55` (`ID/SoundID.cs:1369`).
    pub death_sound: (i32, i32),
    pub knock_back_resist: f32,
    pub value: f32,
    pub npc_slots: f32,
}

/// `NPC.cs:15742-15755` — `else if (this.type == 472)`.
pub const SHADOW_FLAME_APPARITION: ShadowFlameDefaults = ShadowFlameDefaults {
    width: 40,
    height: 24,
    ai_style: 86,
    damage: 50,
    defense: 18,
    life_max: 180,
    hit_sound: (3, 52),
    death_sound: (4, 55),
    knock_back_resist: 0.0,
    value: 0.0,
    npc_slots: 0.1,
};

/// `NPC.cs:15768-15779` — `else if (this.type == 521)` (the
/// AncientCultistSquidhead; the row leaves value/npcSlots at the
/// head defaults).
pub const CULTIST_SQUIDHEAD: ShadowFlameDefaults = ShadowFlameDefaults {
    width: 60,
    height: 60,
    ai_style: 86,
    damage: 90,
    defense: 30,
    life_max: 2000,
    hit_sound: (3, 1),
    death_sound: (4, 6),
    knock_back_resist: 0.0,
    value: 0.0,
    npc_slots: 1.0,
};

/// The per-tick environment (the `Style74Env` shape).
pub struct Style86Env<'a> {
    /// The read-only world `TargetClosest` reads.
    pub world: &'a World,
    /// The `Main.player` slots as `TargetClosest` sees them.
    pub players: &'a [TargetCandidate],
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp
    /// (`NPC.cs:77780`).
    pub ticker: u64,
    /// `Main.dust` — the shared 6001-slot pool (`Dust.cs:75-256`).
    pub dust: &'a mut DustPool,
    /// The `Main` statics `NewDust` reads (`Dust.cs:77-98`).
    pub dust_ctx: &'a SpawnCtx,
    /// The `Main.rand` stream — every draw cited in the module header
    /// rides it.
    pub rng: &'a mut UnifiedRandom,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Default, Debug, PartialEq)]
pub struct Style86Outcome {
    /// The head's `knockBackResist = 0f` write (`:37233`) — every
    /// tick, both kinds (the style74 kb seam).
    pub knock_back_resist: Option<f32>,
    /// The first-tick `SoundEngine.PlaySound(SoundID.Item8, Center)`
    /// request (`:37268` 472 / `:37321` 521) — (2, 8),
    /// `ID/SoundID.cs:1391`.
    pub sound_item8: bool,
}

/// `(float)Math.Sqrt(x*x + y*y)` — XNA `Vector2.Length()` (the f32
/// sum widened to f64 for the root, the style74 `sqrt_f32` shape).
fn sqrt_f32(x: f32, y: f32) -> f32 {
    ((x * x + y * y) as f64).sqrt() as f32
}

/// XNA `Vector2.Normalize()` — UNGUARDED: a zero vector gives NaN,
/// exactly like the C# (the style74 `normalize_xna` precedent).
fn normalize_xna(x: f32, y: f32) -> (f32, f32) {
    let num = 1.0f32 / ((x * x + y * y) as f64).sqrt() as f32;
    (x * num, y * num)
}

/// One fixed step of the aiStyle-86 arm (`NPC.cs:37221-37470`), in
/// C# order. `npcs` is the slot-aligned pool, `me` this NPC's slot
/// (`whoAmI`), `types` the slot-aligned vanilla type table (the
/// wiring seam — module docs). The caller owns the raw movement
/// integration after `AI()` and drains [`Style86Outcome`].
pub fn style86_step(
    npcs: &mut [Npc],
    me: usize,
    types: &[i32],
    env: &mut Style86Env<'_>,
) -> Style86Outcome {
    let mut out = Style86Outcome::default();
    let ty = types[me];
    // `:37223-37230` — the alpha fade-in (the C# int alpha; the port's
    // u8 saturates at 0 identically after the floor).
    if npcs[me].alpha > 0 {
        npcs[me].alpha = npcs[me].alpha.saturating_sub(30);
    }
    // `:37231-37233` — noGravity/noTileCollide are the caller's raw
    // integration (module docs); the kb write rides the outcome.
    out.knock_back_resist = Some(0.0);
    // `:37234-37260` — the sibling repulsion, slot order.
    let (cx, cy) = npcs[me].center();
    for j in 0..npcs.len() {
        if j == me || npcs[j].hp <= 0 || types[j] != ty {
            continue;
        }
        let (jx, jy) = npcs[j].center();
        let (mut dx, mut dy) = (jx - cx, jy - cy);
        if !(sqrt_f32(dx, dy) < 50.0) {
            continue;
        }
        let (nx, ny) = normalize_xna(dx, dy);
        dx = nx;
        dy = ny;
        // `:37246-37256` — the literal zero test on the NORMALIZED
        // pair (NaN fails it, exactly like the C#).
        if dx == 0.0 && dy == 0.0 {
            dx = if j > me { 1.0 } else { -1.0 };
        }
        dx *= 0.4;
        dy *= 0.4;
        npcs[me].vx -= dx;
        npcs[me].vy -= dy;
        npcs[j].vx += dx;
        npcs[j].vy += dy;
    }
    // `:37261-37316` — the 472 materialize.
    if ty == 472 {
        let num1201 = 120.0f32;
        if npcs[me].local_ai[0] < num1201 {
            if npcs[me].local_ai[0] == 0.0 {
                // `:37268-37277` — the cast sound, the retarget, the kick.
                out.sound_item8 = true;
                target_closest(&mut npcs[me], env.world, env.players, true, 86, env.ticker);
                if npcs[me].facing > 0 {
                    npcs[me].vx += 2.0;
                } else {
                    npcs[me].vx -= 2.0;
                }
                // `:37278-37298` — the 20-dust ring under the netOffset
                // round trip.
                npcs[me].x += npcs[me].net_offset.x;
                npcs[me].y += npcs[me].net_offset.y;
                for _ in 0..20 {
                    let (ccx, mut ccy) = npcs[me].center();
                    ccy -= 18.0;
                    let mut vx = env.rng.next_range(-100, 101) as f32;
                    let mut vy = env.rng.next_range(-100, 101) as f32;
                    let (nx, ny) = normalize_xna(vx, vy);
                    let r = env.rng.next_range(0, 100) as f32 * 0.1;
                    vx = nx * r;
                    vy = ny * r;
                    let px = ccx + vx;
                    let py = ccy + vy;
                    let (nx2, ny2) = normalize_xna(vx, vy);
                    let s = env.rng.next_range(50, 90) as f32 * 0.2;
                    vx = nx2 * s;
                    vy = ny2 * s;
                    let idx = env.dust.new_dust(
                        env.rng,
                        env.dust_ctx,
                        env.world,
                        Vec2::new(px, py),
                        1,
                        1,
                        27,
                        0.0,
                        0.0,
                        0,
                        Color::default(),
                        1.0,
                    );
                    {
                        let d = env.dust.slot_mut(idx);
                        d.velocity = Vec2::new(-vx * 0.3, -vy * 0.3);
                        d.alpha = 100;
                        if env.rng.next_max(2) == 0 {
                            d.no_gravity = true;
                            d.scale += 0.3;
                        }
                    }
                }
                npcs[me].x -= npcs[me].net_offset.x;
                npcs[me].y -= npcs[me].net_offset.y;
            }
            // `:37300-37314` — the counter bump, then the fade trickle.
            npcs[me].local_ai[0] += 1.0;
            let num1204 = (1.0 - npcs[me].local_ai[0] / num1201) * 20.0;
            let mut num1205 = 0i32;
            while (num1205 as f32) < num1204 {
                if env.rng.next_max(5) == 0 {
                    npcs[me].x += npcs[me].net_offset.x;
                    npcs[me].y += npcs[me].net_offset.y;
                    let (w, h) = npcs[me].size();
                    let idx = env.dust.new_dust(
                        env.rng,
                        env.dust_ctx,
                        env.world,
                        Vec2::new(npcs[me].x, npcs[me].y),
                        w as i32,
                        h as i32,
                        27,
                        0.0,
                        0.0,
                        0,
                        Color::default(),
                        1.0,
                    );
                    {
                        let d = env.dust.slot_mut(idx);
                        d.alpha = 100;
                        d.velocity *= 0.3;
                        d.velocity += Vec2::new(npcs[me].vx, npcs[me].vy) * 0.75;
                        d.no_gravity = true;
                    }
                    npcs[me].x -= npcs[me].net_offset.x;
                    npcs[me].y -= npcs[me].net_offset.y;
                }
                num1205 += 1;
            }
        }
    }
    // `:37317-37342` — the 521 materialize (no ring).
    if ty == 521 && npcs[me].local_ai[0] < 120.0 {
        if npcs[me].local_ai[0] == 0.0 {
            out.sound_item8 = true;
            target_closest(&mut npcs[me], env.world, env.players, true, 86, env.ticker);
            if npcs[me].facing > 0 {
                npcs[me].vx += 2.0;
            } else {
                npcs[me].vx -= 2.0;
            }
        }
        npcs[me].local_ai[0] += 1.0;
        let num1207 = 10.0f32;
        for _ in 0..2 {
            npcs[me].x += npcs[me].net_offset.x;
            npcs[me].y += npcs[me].net_offset.y;
            let (w, h) = npcs[me].size();
            let idx = env.dust.new_dust(
                env.rng,
                env.dust_ctx,
                env.world,
                Vec2::new(npcs[me].x - num1207, npcs[me].y - num1207),
                (w + num1207 * 2.0) as i32,
                (h + num1207 * 2.0) as i32,
                228,
                0.0,
                0.0,
                100,
                Color::default(),
                2.0,
            );
            {
                let d = env.dust.slot_mut(idx);
                d.no_gravity = true;
                d.no_light = true;
            }
            npcs[me].x -= npcs[me].net_offset.x;
            npcs[me].y -= npcs[me].net_offset.y;
        }
    }
    // `:37343-37457` — the four-state hover.
    let (pcx, pcy) = {
        let p = &env.players[npcs[me].target as usize];
        (
            p.x + (p.width / 2) as f32,
            p.y + (p.height / 2) as f32,
        )
    };
    let (cx, cy) = npcs[me].center();
    if npcs[me].ai[0] == 0.0 {
        // `:37343-37348`.
        target_closest(&mut npcs[me], env.world, env.players, true, 86, env.ticker);
        npcs[me].ai[0] = 1.0;
        npcs[me].ai[1] = npcs[me].facing as f32;
    } else if npcs[me].ai[0] == 1.0 {
        // `:37349-37401` — the strafe.
        target_closest(&mut npcs[me], env.world, env.players, true, 86, env.ticker);
        let (num1210, num1211, num1212, num1213, mut num1214) = if ty == 521 {
            (0.7f32, 14.0f32, 6.0f32, 500.0f32, 3.0f32)
        } else {
            (0.3, 7.0, 4.0, 660.0, 4.0)
        };
        npcs[me].vx += npcs[me].ai[1] * num1210;
        if npcs[me].vx > num1211 {
            npcs[me].vx = num1211;
        }
        if npcs[me].vx < 0.0 - num1211 {
            npcs[me].vx = 0.0 - num1211;
        }
        let mut num1215 = pcy - cy;
        if num1215.abs() > num1212 {
            num1214 = 15.0;
        }
        if num1215 > num1212 {
            num1215 = num1212;
        } else if num1215 < 0.0 - num1212 {
            num1215 = 0.0 - num1212;
        }
        npcs[me].vy = (npcs[me].vy * (num1214 - 1.0) + num1215) / num1214;
        if (npcs[me].ai[1] > 0.0 && pcx - cx < 0.0 - num1213)
            || (npcs[me].ai[1] < 0.0 && pcx - cx > num1213)
        {
            npcs[me].ai[0] = 2.0;
            npcs[me].ai[1] = if cy + 20.0 > pcy { -1.0 } else { 1.0 };
        }
    } else if npcs[me].ai[0] == 2.0 {
        // `:37402-37424` — the dive.
        let (num1216, num1217, num1218) = if ty == 521 {
            (0.3f32, 0.9f32, 7.0f32)
        } else {
            (0.4, 0.95, 5.0)
        };
        npcs[me].vy += npcs[me].ai[1] * num1216;
        if sqrt_f32(npcs[me].vx, npcs[me].vy) > num1218 {
            npcs[me].vx *= num1217;
            npcs[me].vy *= num1217;
        }
        if npcs[me].vx > -1.0 && npcs[me].vx < 1.0 {
            target_closest(&mut npcs[me], env.world, env.players, true, 86, env.ticker);
            npcs[me].ai[0] = 3.0;
            npcs[me].ai[1] = npcs[me].facing as f32;
        }
    } else if npcs[me].ai[0] == 3.0 {
        // `:37425-37457` — the climb-back.
        let (num1219, num1220, num1221, num1222) = if ty == 521 {
            (0.6f32, 0.3f32, 7.0f32, 0.9f32)
        } else {
            (0.4, 0.2, 5.0, 0.95)
        };
        npcs[me].vx += npcs[me].ai[1] * num1219;
        if cy > pcy {
            npcs[me].vy -= num1220;
        } else {
            npcs[me].vy += num1220;
        }
        if sqrt_f32(npcs[me].vx, npcs[me].vy) > num1221 {
            npcs[me].vx *= num1222;
            npcs[me].vy *= num1222;
        }
        if npcs[me].vy > -1.0 && npcs[me].vy < 1.0 {
            target_closest(&mut npcs[me], env.world, env.players, true, 86, env.ticker);
            npcs[me].ai[0] = 0.0;
            npcs[me].ai[1] = npcs[me].facing as f32;
        }
    }
    // `:37458-37469` — the 521 every-tick tail dust.
    if ty == 521 {
        let num1223 = 10.0f32;
        npcs[me].x += npcs[me].net_offset.x;
        npcs[me].y += npcs[me].net_offset.y;
        let (w, h) = npcs[me].size();
        let idx = env.dust.new_dust(
            env.rng,
            env.dust_ctx,
            env.world,
            Vec2::new(npcs[me].x - num1223, npcs[me].y - num1223),
            (w + num1223 * 2.0) as i32,
            (h + num1223 * 2.0) as i32,
            228,
            0.0,
            0.0,
            100,
            Color::default(),
            2.0,
        );
        {
            let d = env.dust.slot_mut(idx);
            d.no_gravity = true;
            d.no_light = true;
        }
        npcs[me].x -= npcs[me].net_offset.x;
        npcs[me].y -= npcs[me].net_offset.y;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A 472-shaped NPC: no kind yet, so the fixture rides a
    /// placeholder with the C# box through `size_override` (the
    /// style73 turret convention).
    fn apparition(x: f32, y: f32) -> Npc {
        let d = &SHADOW_FLAME_APPARITION;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n
    }

    fn squidhead(x: f32, y: f32) -> Npc {
        let d = &CULTIST_SQUIDHEAD;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n
    }

    fn cand(x: f32, y: f32) -> TargetCandidate {
        TargetCandidate {
            active: true,
            dead: false,
            ghost: false,
            x,
            y,
            width: 20,
            height: 42,
            aggro: 0,
            item_animation: 0,
            npc_type_no_aggro: false,
            tank_pet: None,
        }
    }

    fn air() -> World {
        World::new("t".into(), 400, 300)
    }

    fn spawn_ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
        }
    }

    /// The menu ctx: `NewDust` refuses BEFORE any draw (`Dust.cs:77`)
    /// — isolates the arm's EXPLICIT Next draws (the style74 fixture).
    fn menu_ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: true,
            ..spawn_ctx()
        }
    }

    /// `NPC.cs:15742-15755` / `:15768-15779` — the two SetDefaults
    /// rows, field by field.
    #[test]
    fn w4_d32_s86_the_two_defaults_rows() {
        let d = &SHADOW_FLAME_APPARITION;
        assert_eq!((d.width, d.height), (40, 24));
        assert_eq!(d.ai_style, 86);
        assert_eq!((d.damage, d.defense, d.life_max), (50, 18, 180));
        assert_eq!(d.hit_sound, (3, 52), "SoundID.NPCHit52");
        assert_eq!(d.death_sound, (4, 55), "SoundID.NPCDeath55");
        assert_eq!(d.knock_back_resist, 0.0);
        assert_eq!(d.value, 0.0);
        assert_eq!(d.npc_slots, 0.1);
        let s = &CULTIST_SQUIDHEAD;
        assert_eq!((s.width, s.height), (60, 60));
        assert_eq!(s.ai_style, 86);
        assert_eq!((s.damage, s.defense, s.life_max), (90, 30, 2000));
        assert_eq!(s.hit_sound, (3, 1));
        assert_eq!(s.death_sound, (4, 6));
        assert_eq!(s.knock_back_resist, 0.0);
    }

    /// `:37223-37233` — the alpha fade floors at 0 and the kb write
    /// fires every tick.
    #[test]
    fn w4_d32_s86_the_head_fades_alpha_and_zeroes_the_kb() {
        let w = air();
        let players = [cand(2000.0, 2000.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(7);
        let mut npcs = [apparition(100.0, 100.0)];
        npcs[0].alpha = 45;
        npcs[0].local_ai[0] = 120.0; // past the materialize
        npcs[0].ai[0] = 9.0; // no state arm
        let mut env = Style86Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
        };
        let out = style86_step(&mut npcs, 0, &[472], &mut env);
        assert_eq!(npcs[0].alpha, 15);
        assert_eq!(out.knock_back_resist, Some(0.0));
        let _ = style86_step(&mut npcs, 0, &[472], &mut env);
        assert_eq!(npcs[0].alpha, 0);
        let _ = style86_step(&mut npcs, 0, &[472], &mut env);
        assert_eq!(npcs[0].alpha, 0, "the floor holds");
    }

    /// `:37234-37260` — two same-type bodies within 50 px push apart
    /// symmetrically along the normalized delta ×0.4; a 521 sibling
    /// (different type) is ignored.
    #[test]
    fn w4_d32_s86_the_sibling_repulsion_is_symmetric_and_type_gated() {
        let w = air();
        let players = [cand(2000.0, 2000.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(7);
        let mut npcs = [apparition(100.0, 100.0), apparition(130.0, 100.0), squidhead(110.0, 100.0)];
        for n in npcs.iter_mut() {
            n.local_ai[0] = 120.0;
            n.ai[0] = 9.0;
        }
        let mut env = Style86Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
        };
        let _ = style86_step(&mut npcs, 0, &[472, 472, 521], &mut env);
        // The delta is (+30, 0) → normalized (1, 0) ×0.4.
        assert_eq!(npcs[0].vx, -0.4);
        assert_eq!(npcs[1].vx, 0.4);
        assert_eq!(npcs[2].vx, 0.0, "the 521 is a different type");
        assert_eq!(npcs[0].vy, 0.0);
    }

    /// `:37279-37297` + `:37302-37314` — the 472 FIRST tick's explicit
    /// draw census under the menu ctx: 20 ring iterations × 5 draws
    /// (X, Y, radius, speed, the Next(2) pick) + the fade trickle's
    /// 20 Next(5) gates (`localAI[0]` = 1 → num1204 = 19.8333, the
    /// float loop runs 0..=19). The mirrored oracle replays the
    /// exact sequence.
    #[test]
    fn w4_d32_s86_the_472_first_tick_consumes_the_ring_and_trickle_draws() {
        let w = air();
        let players = [cand(2000.0, 2000.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(86);
        let mut oracle = UnifiedRandom::new(86);
        let mut npcs = [apparition(100.0, 100.0)];
        let mut env = Style86Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
        };
        let out = style86_step(&mut npcs, 0, &[472], &mut env);
        assert!(out.sound_item8, "the cast sound fires once");
        assert_eq!(npcs[0].local_ai[0], 1.0);
        // The oracle: 20 × (X, Y, radius, speed, pick), then 20 gates.
        for _ in 0..20 {
            oracle.next_range(-100, 101);
            oracle.next_range(-100, 101);
            oracle.next_range(0, 100);
            oracle.next_range(50, 90);
            oracle.next_max(2);
        }
        for _ in 0..20 {
            oracle.next_max(5);
        }
        assert_eq!(rng.peek(), oracle.peek(), "the first-tick draw order");
        // The kick: facing +1 (the player is to the right) → +2.
        assert!(npcs[0].vx >= 2.0 - 0.4, "the ±2 direction kick");
    }

    /// `:37300-37314` — a mid-fade tick (localAI[0] = 60 → 61,
    /// num1204 = (1 − 61/120) × 20 = 9.8333 → 10 Next(5) gates, NO
    /// ring, NO sound).
    #[test]
    fn w4_d32_s86_the_472_fade_tick_draws_only_the_trickle_gates() {
        let w = air();
        let players = [cand(2000.0, 2000.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(86);
        let mut oracle = UnifiedRandom::new(86);
        let mut npcs = [apparition(100.0, 100.0)];
        npcs[0].local_ai[0] = 60.0;
        npcs[0].ai[0] = 9.0;
        let mut env = Style86Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
        };
        let out = style86_step(&mut npcs, 0, &[472], &mut env);
        assert!(!out.sound_item8);
        assert_eq!(npcs[0].local_ai[0], 61.0);
        for _ in 0..10 {
            oracle.next_max(5);
        }
        assert_eq!(rng.peek(), oracle.peek(), "the trickle gate count");
    }

    /// `:37317-37342` + `:37458-37469` — the 521 materialize draws NO
    /// explicit Next (the dusts are NewDust internals only): under
    /// the menu ctx the first tick's stream is untouched, the sound
    /// fires, the counter bumps; the tail runs every tick.
    #[test]
    fn w4_d32_s86_the_521_block_is_explicit_draw_free() {
        let w = air();
        let players = [cand(2000.0, 2000.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(86);
        let mut npcs = [squidhead(100.0, 100.0)];
        npcs[0].ai[0] = 9.0;
        let mut env = Style86Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
        };
        let out = style86_step(&mut npcs, 0, &[521], &mut env);
        assert!(out.sound_item8);
        assert_eq!(npcs[0].local_ai[0], 1.0);
        assert_eq!(rng.peek(), UnifiedRandom::new(86).peek(), "no explicit draws");
    }

    /// `:37343-37348` + `:37349-37401` — state 0 latches the side;
    /// state 1 accelerates along it, caps at 7 (472), and the Y
    /// smoothing uses the 4-divisor (15 past the clamp).
    #[test]
    fn w4_d32_s86_the_strafe_latches_accelerates_and_caps() {
        let w = air();
        let players = [cand(2000.0, 100.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(7);
        let mut npcs = [apparition(100.0, 100.0)];
        npcs[0].local_ai[0] = 120.0;
        let mut env = Style86Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
        };
        let _ = style86_step(&mut npcs, 0, &[472], &mut env);
        assert_eq!(npcs[0].ai[0], 1.0);
        assert_eq!(npcs[0].ai[1], npcs[0].facing as f32);
        for _ in 0..40 {
            let _ = style86_step(&mut npcs, 0, &[472], &mut env);
        }
        assert_eq!(npcs[0].vx, 7.0, "the 472 cap");
        // The Y delta is small (both at y≈100): the 4-divisor path.
        assert!(npcs[0].vy.abs() < 4.0);
    }

    /// `:37388-37400` — past the 660-px X-threshold the strafe flips
    /// to the dive with the ±1 Y-side pick (`Center.Y + 20 > pcy`).
    #[test]
    fn w4_d32_s86_the_strafe_flips_to_the_dive_past_the_threshold() {
        let w = air();
        // The player far LEFT of the apparition, BELOW it.
        let players = [cand(100.0, 400.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(7);
        let mut npcs = [apparition(1000.0, 100.0)];
        npcs[0].local_ai[0] = 120.0;
        npcs[0].ai[0] = 1.0;
        npcs[0].ai[1] = 1.0; // strafing right, player left → flip
        let mut env = Style86Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
        };
        let _ = style86_step(&mut npcs, 0, &[472], &mut env);
        assert_eq!(npcs[0].ai[0], 2.0);
        // `:37392-37399`: cy + 20 (132) > pcy (421) is FALSE → ai[1] = +1.
        assert_eq!(npcs[0].ai[1], 1.0, "the body above the player picks the +1 side");
    }

    /// `:37402-37457` — the dive/climb exits: state 2 exits to 3 on
    /// the (−1, 1) X band, state 3 exits to 0 on the (−1, 1) Y band.
    #[test]
    fn w4_d32_s86_the_dive_and_climb_exit_bands() {
        let w = air();
        let players = [cand(2000.0, 100.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(7);
        let mut npcs = [apparition(100.0, 100.0)];
        npcs[0].local_ai[0] = 120.0;
        npcs[0].ai[0] = 2.0;
        npcs[0].ai[1] = 1.0;
        npcs[0].vx = 0.5; // inside the band already
        let mut env = Style86Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
        };
        let _ = style86_step(&mut npcs, 0, &[472], &mut env);
        assert_eq!(npcs[0].ai[0], 3.0, "the X band exits the dive");
        npcs[0].vy = 0.5;
        npcs[0].vx = 3.0; // out of the X band, irrelevant in state 3
        let _ = style86_step(&mut npcs, 0, &[472], &mut env);
        assert_eq!(npcs[0].ai[0], 0.0, "the Y band exits the climb");
    }

    /// The state machine's zero-draw census: 600 ticks through the
    /// full 0→1→2→3 cycle (materialize done, menu ctx) consume NO
    /// explicit `Main.rand` — the states' only inputs are
    /// deterministic (`TargetClosest`, the arithmetic).
    #[test]
    fn w4_d32_s86_the_state_cycle_draws_nothing() {
        let w = air();
        let players = [cand(900.0, 300.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(521);
        let mut npcs = [apparition(100.0, 100.0)];
        npcs[0].local_ai[0] = 120.0;
        let mut env = Style86Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
        };
        for _ in 0..600 {
            let _ = style86_step(&mut npcs, 0, &[472], &mut env);
        }
        assert_eq!(rng.peek(), UnifiedRandom::new(521).peek(), "the states are draw-free");
    }

    /// The full-pool stream parity: the 472 first tick on the REAL
    /// dust ctx (the NewDust internals interleave) must match a
    /// cloned twin step bit-for-bit — the by-construction census's
    /// pooled pin (the style65 twin-pool precedent).
    #[test]
    fn w4_d32_s86_the_full_pool_first_tick_is_twin_stable() {
        let w = air();
        let players = [cand(2000.0, 2000.0)];
        let c = spawn_ctx();
        let mut npcs_a = [apparition(600.0, 600.0)];
        let mut npcs_b = npcs_a.clone();
        let mut pool_a = DustPool::new();
        let mut pool_b = DustPool::new();
        let mut rng_a = UnifiedRandom::new(472);
        let mut rng_b = UnifiedRandom::new(472);
        for tick in 0..30 {
            let mut env_a = Style86Env {
                world: &w,
                players: &players,
                ticker: tick,
                dust: &mut pool_a,
                dust_ctx: &c,
                rng: &mut rng_a,
            };
            let _ = style86_step(&mut npcs_a, 0, &[472], &mut env_a);
            let mut env_b = Style86Env {
                world: &w,
                players: &players,
                ticker: tick,
                dust: &mut pool_b,
                dust_ctx: &c,
                rng: &mut rng_b,
            };
            let _ = style86_step(&mut npcs_b, 0, &[472], &mut env_b);
            assert_eq!(rng_a.peek(), rng_b.peek(), "stream @{tick}");
            assert_eq!(npcs_a[0].vx.to_bits(), npcs_b[0].vx.to_bits(), "vx @{tick}");
            assert_eq!(npcs_a[0].ai[0].to_bits(), npcs_b[0].ai[0].to_bits(), "ai0 @{tick}");
        }
    }
}
