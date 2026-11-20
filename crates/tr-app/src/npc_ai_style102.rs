//! The aiStyle-102 family — the Sand Elemental (`NPC.AI()`'s
//! `else if (this.aiStyle == 102)` arm, `NPC.cs:40147-40542`). W4-d33,
//! research/25 row 102, CODE-ONLY MODE (no compiler invoked; the
//! witness tests below are written, not run).
//!
//! ## Census
//!
//! The family has exactly ONE vanilla member: the SandElemental 541
//! (`NPCID.SandElemental`, `ID/NPCID.cs:11759`; `SetDefaults` row
//! `NPC.cs:15898-15911`). The census of `this.aiStyle = 102;` across the
//! decompiled tree returns four hits: `NPC.cs:15902` (the `type == 541`
//! branch) and `Projectile.cs:6146`/`:6234`/`:6243` (the projectile-space
//! 539/573/574 rows — a different machine, out of NPC scope). The port's
//! roster has no `NpcKind::SandElemental` — the module lands UNWIRED (the
//! `npc_ai_style73.rs` precedent), keyed on the vanilla `npc_type` until
//! the roster wave.
//!
//! ## The machine
//!
//! The arm is the generic aiStyle-26-style hover-walker skeleton with a
//! 541-only head (`:40168-40319`). The locals (`:40149-40167`): the
//! forward probe depth 4 (`num1378`), ceiling probe height 3
//! (`num1379`), foot probe depth 0 (`num1380` — 3 for 541), gravity 0.2
//! (`num1381` — 0.1 for 541) capped 2 (`num1382`), lift -0.2 (`num1383` —
//! -0.1 for 541) capped -4 (`num1384`), X top speed 2 (`num1385`), X
//! accel 0.1 (`num1386`), Y top speed 1 (`num1387`), Y accel 0.04
//! (`num1388`), the charge damp 0.96 (`num1389`). `flag111` is born
//! false and never set — the `if (!flag111)` stuck block ALWAYS runs.
//! `flag114` (the ceiling probe) is FALSE for 541 — the elemental never
//! ceiling-jumps; only the stuck-counter path sets `flag110`.
//!
//! The 541 head: `rotation = vx * 0.04`, `spriteDirection =
//! sign(direction)`; the life ratio `life / lifeMax` ramps the X speed
//! `2 + (1-r)*2` and accel `0.1 + (1-r)*0.02`, and below half life
//! `knockBackResist = 0` (the port's `kb_resist_override` seam, the
//! style39 precedent). The sand skirt (`:40183-40205`): position +=
//! netOffset around TWO `Next(2) != 0` dust gates — type 268 at Bottom,
//! position `Lerp(BottomLeft+(0,-12), BottomRight, NextFloat ×2) +
//! (-spriteDirection*10, -4)`; iteration 1 OVERWRITES the position with
//! `Bottom + RandomVector2(-6, 6)` (the two NextFloat draws STILL happen
//! first — the RNG order witness); colour `Color(222,108,48) * 0.7`
//! (the XNA truncating byte multiply), scale 0.8, `velocity.Y += -0.3 +
//! Max(vy*2, 0)`, `velocity.X += spriteDirection * 0.2`.
//!
//! The charge cycle (`:40206-40318`): `localAI[2] = 0`; a negative
//! `ai[0]` counts up (`Min(ai[0]+1, 0)`); a positive `ai[0]` disables
//! retargeting (`flag116 = false`), charges (`flag115 = true`) and
//! increments — at 135 it resets to -300 (+netUpdate). At exactly 54
//! on a non-client the sandnado placement runs (`:40223-40265`): the
//! target's centre lead `(vx*30, 0)`; if `Distance < 900`, up to 1000
//! `Next(px-30, px+31)` column rolls (min 10 tiles apart, max 3 marks),
//! each dropped by `ExpandVertically(col, py-20, 1, 51)` and gated by
//! `CanSpawnSandstormHostile((col, bottomY-15)*16, 15, 15)` — a type-658
//! `SandnadoHostileMark` (`Projectile.cs:7071-7081`, aiStyle 128) at
//! `(col*16, row*16)`, zero velocity/damage/knockback. Out of range:
//! `ai[0] = -200` (+netUpdate). The trail (`:40266-40309`): while
//! `0 < ai[0] < 114` and `t = (ai[0]-54)/30` is in `[0, 0.95)`, each
//! active 658 mark's centre rides the CatmullRom spline
//! `(mouth+(0,1500), mouth, mark, mark+(0,1500))` (mouth = `Center +
//! UnitX*direction*50 - UnitY*6`); 2/3/4 type-269 dusts per mark by the
//! `Distance(value3, value4)` 5/10 thresholds, position `Lerp(v3, v4,
//! i/count) + RandomVector2(-2, 2)`, `noLight`, `scale = 0.3 + t`. The
//! dead locals `vector216` (`:40221`), `vector219` (`:40266`) and
//! `num1397` (`:40286-40290`) are computed and never read — documented,
//! zero effect. `ai[0] == 0` arms the charge (`ai[0] = 1`, netUpdate).
//!
//! The stuck block (`:40320-40375`, the aiStyle-26 skeleton): justHit
//! zeroes `localAI[2]`; while non-negative, the ±16 px anchor window
//! (widened to ±40 when moving against the facing) counts stuck ticks —
//! 30 on the NARROW window flags the jump (`flag110`), 60 flips the
//! facing + vx, rests on -180 and clears collideX; outside the window
//! the anchor re-seeds. `flag116` gates `TargetClosest`. The cooldown
//! arm (`localAI[2] < 0`) counts up and faces the target.
//!
//! The movement tail (`:40376-40541`): the probe coordinates (the
//! INTEGER `width / 2`, the `(int)Bottom.Y / 16` cast-before-divide);
//! charging (`flag115`) damps the velocity ×0.96 and RETURNS. The
//! forward probe (4 rows, near-ground ≤ +1), the foot probe (`num1380`
//! rows — 3 for 541) and the (non-541) ceiling probe set the gravity /
//! lift / jump flags; `flag110` forces gravity. The collideX/Y bounces
//! (-0.4 / -0.25 with the ±1 floors), then the four accel arms. The
//! `:40523` clamp is the C#'s own BUG, ported verbatim: the upward Y
//! clamp writes `0f - num1385` (the X top speed), NOT `num1387`.
//!
//! ## RNG order (by construction)
//!
//! Per tick, in order: the two skirt `Next(2)` gates (each passing gate
//! draws the `NewDust` internals, then TWO `NextFloat`, then — gate 2
//! only — the `RandomVector2` TWO `NextDouble`); the placement rolls
//! (one `Next(px-30, px+31)` per attempt, ONLY at `ai[0] == 54` on a
//! non-client within 900 px — `ExpandVertically` and
//! `CanSpawnSandstormHostile` are draw-free); the trail dusts (per dust:
//! the `NewDust` internals then TWO `NextDouble`). `TargetClosest` is
//! RNG-free (`NPC.cs:77652-77781`); the probes, the stuck block, the
//! bounces and the accel arms draw NOTHING.
//!
//! ## FindFrame case 541
//!
//! `NPC.cs:69734-69772` over `Main.npcFrameCount[541] = 13`
//! (`Main.cs:67138`, the 540-549 row): a positive `ai[0]` picks the row
//! directly (the charge strip 5-8 cycling `n/8 % 4`, the 9 hold, the
//! 10-12 release `(n-114)/7`); otherwise the counter accumulates
//! `velocity.Length()*0.1 + 1` (the C# adds in DOUBLE) and advances the
//! 4-row walk at 8. [`elemental_find_frame`] ports it as the pure
//! function (the style73 `turret_find_frame` convention).
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use crate::projectile::catmull_rom;
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::stray::expand_vertically;
use tr_core::tiledata::TILE_SOLID;
use tr_core::{UnifiedRandom, World};

/// The type-541 `SetDefaults` row (`NPC.cs:15898-15911`), verbatim.
pub struct SandElementalDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit23` (`SoundID.cs:1279`).
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath39` (`SoundID.cs:1353`).
    pub death_sound: (i32, i32),
    pub knock_back_resist: f32,
    /// `Item.buyPrice(0, 1, 50)` = 15000 copper (`Item.cs:49417-49420`).
    pub value: i32,
    pub rarity: i32,
}

/// `NPC.cs:15898-15911` — `else if (this.type == 541)`.
pub const SAND_ELEMENTAL: SandElementalDefaults = SandElementalDefaults {
    width: 30,
    height: 76,
    ai_style: 102,
    damage: 40,
    defense: 30,
    life_max: 5000,
    hit_sound: (3, 23),
    death_sound: (4, 39),
    knock_back_resist: 0.05,
    value: 15000,
    rarity: 2,
};

/// `Main.npcFrameCount[541]` = 13 (`Main.cs:67138`, the 540-549 row:
/// `7, 13, 4, …` — index 541 is the second entry).
pub const SAND_ELEMENTAL_FRAME_COUNT: u32 = 13;

/// The per-tick environment (the `Style101Env` shape).
pub struct Style102Env<'a> {
    pub world: &'a World,
    pub dust: &'a mut DustPool,
    pub dust_ctx: &'a SpawnCtx,
    pub rng: &'a mut UnifiedRandom,
    /// The `Main.player[]` slots as `TargetClosest` sees them (the
    /// `npc_target.rs` seam).
    pub players: &'a [TargetCandidate],
    /// `Main.player[j].velocity.X` per slot — the sandnado lead reads
    /// it (`:40226`); `TargetCandidate` carries no velocity, so it
    /// rides this parallel slice (the style14 `players_wet` precedent).
    pub players_vx: &'a [f32],
    /// The active type-658 marks' centres in `Main.projectile` slot
    /// order (`:40270-40277` scans the 1000 slots) — the wiring caller
    /// gathers them; the pure module never touches the projectile pool.
    pub marks: &'a [Vec2],
    /// `Main.netMode` — the placement fires on `!= 1` (`:40223`); the
    /// trail runs on clients too.
    pub net_mode: i32,
    /// `Main.EverLastingTicker` for the selection's `targetSetFrame`.
    pub ticker: u64,
}

/// The `Projectile.NewProjectile` request of the placement
/// (`NPC.cs:40257`): type 658 (`ProjectileID.SandnadoHostileMark`,
/// `ID/ProjectileID.cs:1870`), ZERO velocity, damage and knockback,
/// owner `Main.myPlayer`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SandnadoMark {
    /// `ProjectileID.SandnadoHostileMark` = 658.
    pub projectile_type: i32,
    /// `item2.X * 16` — the INTEGER tile product widened to px.
    pub x: f32,
    pub y: f32,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Default, Debug, PartialEq)]
pub struct Style102Outcome {
    /// The placement's marks (`:40255-40258`), in list order (the roll
    /// order). Empty on every other tick.
    pub shots: Vec<SandnadoMark>,
}

/// `Utils.RandomVector2(random, min, max)` (`Utils.cs:2431-2434`): TWO
/// `NextDouble` draws, X first — `(max-min) * (float)NextDouble() + min`
/// per axis (the double is cast to f32 BEFORE the multiply).
fn random_vector2(rng: &mut UnifiedRandom, min: f32, max: f32) -> Vec2 {
    let x = (max - min) * rng.next_double() as f32 + min;
    let y = (max - min) * rng.next_double() as f32 + min;
    Vec2::new(x, y)
}

/// XNA `Vector2.Distance(a, b)` — the f32 squared sum widened to f64
/// for the root (the style100 `sqrt_f32` shape).
fn distance(a: Vec2, b: Vec2) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    ((dx * dx + dy * dy) as f64).sqrt() as f32
}

/// `StrayMethods.CanSpawnSandstormHostile` (`StrayMethods.cs:47-63`):
/// the three columns `x-1..=x+1` each expand 15 up / 15 down; the free
/// span (`topY+1 .. bottomY-1`) must be ≥ 20 rows in EVERY column.
/// Draw-free.
fn can_spawn_sandstorm_hostile(
    world: &World,
    position: Vec2,
    expand_up: i32,
    expand_down: i32,
) -> bool {
    // `ToTileCoordinates` (`Utils.cs:1807-1810`): `(int)v >> 4`.
    let px = position.x as i32 >> 4;
    let py = position.y as i32 >> 4;
    for i in -1..=1 {
        let (top_y, bottom_y) = expand_vertically(world, px + i, py, expand_up, expand_down);
        if (bottom_y - 1) - (top_y + 1) < 20 {
            return false;
        }
    }
    true
}

/// The target slot's candidate — the style16 `target_of` fallback
/// (`SetTargetTrackingValues`, `NPC.cs:77731-77734`).
fn target_of<'a>(npc: &Npc, players: &'a [TargetCandidate]) -> &'a TargetCandidate {
    let i = if npc.target >= 0 && (npc.target as usize) < players.len() {
        npc.target as usize
    } else {
        0
    };
    &players[i]
}

/// `(int)((px) / 16f)`-style probe cell: solid (`nactive() &&
/// tileSolid[type]`, NO solidTop term — `:40391`) OR any liquid. A null
/// tile is the C#'s fresh empty `Tile` — the port's `None` reads the
/// same (out-of-bounds included).
fn probe_blocked(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| (t.active() && !t.inactive() && TILE_SOLID.has(t.kind)) || t.liquid > 0)
}

/// The sandnado placement (`NPC.cs:40223-40265`) — the `ai[0] == 54 &&
/// netMode != 1` arm. Returns the marks; the out-of-range arm's
/// `ai[0] = -200` + netUpdate writes the NPC in place.
fn place_sandnados(npc: &mut Npc, env: &mut Style102Env<'_>, out: &mut Style102Outcome) {
    let p = target_of(npc, env.players);
    let pvx = if npc.target >= 0 && (npc.target as usize) < env.players_vx.len() {
        env.players_vx[npc.target as usize]
    } else {
        env.players_vx.first().copied().unwrap_or(0.0)
    };
    // `:40226` — the lead: the target's centre + (vx*30, 0).
    let lead = Vec2::new(
        p.x + (p.width / 2) as f32 + pvx * 30.0,
        p.y + (p.height / 2) as f32,
    );
    let (cx, cy) = npc.center();
    // `:40227` — base.Distance = Vector2.Distance(Center, lead).
    if distance(Vec2::new(cx, cy), lead) < 900.0 {
        // `:40229` — ToTileCoordinates: (int) >> 4.
        let ptx = lead.x as i32 >> 4;
        let pty = lead.y as i32 >> 4;
        let mut list: Vec<(i32, i32)> = Vec::new();
        // `:40230-40254` — up to 1000 rolls, max 3 marks.
        for _ in 0..1000 {
            if list.len() >= 3 {
                break;
            }
            let col = env.rng.next_range(ptx - 30, ptx + 30 + 1);
            // `:40238-40245` — the 10-tile separation rejection.
            if list.iter().any(|&(x, _)| (x - col).abs() < 10) {
                continue;
            }
            // `:40248` — drop to the floor (1 up, 51 down).
            let (_, bottom_y) = expand_vertically(env.world, col, pty - 20, 1, 51);
            // `:40249-40252` — the headroom gate at (col, bottomY-15).
            let spot = Vec2::new(col as f32, (bottom_y - 15) as f32) * 16.0;
            if can_spawn_sandstorm_hostile(env.world, spot, 15, 15) {
                list.push((col, bottom_y - 15));
            }
        }
        // `:40255-40258` — one 658 mark per accepted spot.
        for &(x, y) in &list {
            out.shots.push(SandnadoMark {
                projectile_type: 658,
                x: (x * 16) as f32,
                y: (y * 16) as f32,
            });
        }
    } else {
        // `:40260-40264` — out of range: the short rest.
        npc.ai[0] = -200.0;
        npc.net_update = true;
    }
}

/// The sandnado trail (`NPC.cs:40266-40309`) — the `0 < ai[0] < 114`
/// window. `mouth` is `vector217`; the dead `vector219` (`:40266`) and
/// `num1397` (`:40286-40290`) locals are documented omissions (computed,
/// never read).
fn sandnado_trail(npc: &Npc, env: &mut Style102Env<'_>, mouth: Vec2) {
    // `:40278-40279` — the spline's vertical handle + the phase.
    let handle = Vec2::new(0.0, 1500.0);
    let t = (npc.ai[0] - 54.0) / 30.0;
    if !(t < 0.95 && t >= 0.0) {
        return;
    }
    // `:40282-40307` — one spline per active mark (the env slice keeps
    // the C#'s projectile-slot order).
    for &mark in env.marks {
        let v3 = catmull_rom(mouth + handle, mouth, mark, mark + handle, t);
        let v4 = catmull_rom(mouth + handle, mouth, mark, mark + handle, t + 0.05);
        // `:40291-40299` — the 2/3/4 density by the arc step.
        let mut count = 2.0f32;
        if distance(v3, v4) > 5.0 {
            count = 3.0;
        }
        if distance(v3, v4) > 10.0 {
            count = 4.0;
        }
        // `:40300-40306` — the float-stepped dust line.
        let mut i = 0.0f32;
        while i < count {
            let idx = env.dust.new_dust(
                env.rng,
                env.dust_ctx,
                env.world,
                mouth,
                0,
                0,
                269,
                0.0,
                0.0,
                0,
                Color::default(),
                1.0,
            );
            {
                let d = env.dust.slot_mut(idx);
                let jitter = random_vector2(env.rng, -2.0, 2.0);
                d.position = v3.lerp(v4, i / count) + jitter;
                d.no_light = true;
                d.scale = 0.3 + t;
            }
            i += 1.0;
        }
    }
}

/// One fixed step of the aiStyle-102 arm (`NPC.cs:40147-40542`), in C#
/// order. `npc_type` is the vanilla id (541) until the roster kind
/// lands (the style100 convention). The caller owns the movement
/// integration after `AI()` and drains [`Style102Outcome`].
pub fn style102_step(npc: &mut Npc, npc_type: i32, env: &mut Style102Env<'_>) -> Style102Outcome {
    let mut out = Style102Outcome::default();
    // `:40149-40167` — the C# locals, verbatim. flag111 is born false
    // and never set (the stuck block always runs); flag114 (the ceiling
    // probe) is 541-gated off below.
    let mut jump = false; // flag110
    let mut gravity = true; // flag112
    let mut ground_near = false; // flag113
    let forward_depth = 4; // num1378
    let ceiling_height = 3; // num1379
    let mut foot_depth = 0; // num1380
    let mut fall_accel = 0.2f32; // num1381
    let fall_cap = 2.0f32; // num1382
    let mut rise_accel = -0.2f32; // num1383
    let rise_cap = -4.0f32; // num1384
    let mut ceiling_probe = true; // flag114
    let mut top_speed_x = 2.0f32; // num1385
    let mut accel_x = 0.1f32; // num1386
    let top_speed_y = 1.0f32; // num1387
    let accel_y = 0.04f32; // num1388
    let mut charging = false; // flag115
    let charge_damp = 0.96f32; // num1389
    let mut retarget = true; // flag116
    let (w, h) = npc.size();
    if npc_type == 541 {
        // `:40170-40182` — the 541 tuning + the life ramp.
        ceiling_probe = false;
        npc.rotation = npc.vx * 0.04;
        npc.sprite_dir = if npc.facing > 0 { 1 } else { -1 };
        foot_depth = 3;
        rise_accel = -0.1;
        fall_accel = 0.1;
        let ratio = npc.hp as f32 / npc.max_hp as f32;
        top_speed_x += (1.0 - ratio) * 2.0;
        accel_x += (1.0 - ratio) * 0.02;
        if ratio < 0.5 {
            // The `knockBackResist = 0` write — the kb_resist_override
            // seam (the style39 precedent; never cleared, as in C#).
            npc.kb_resist_override = Some(0.0);
        }
        // `:40183-40205` — the sand skirt under the netOffset shift.
        npc.x += npc.net_offset.x;
        npc.y += npc.net_offset.y;
        let bottom_left = Vec2::new(npc.x, npc.y + h) + Vec2::new(0.0, -12.0);
        let bottom_right = Vec2::new(npc.x + w, npc.y + h);
        let push = Vec2::new(-npc.sprite_dir as f32 * 10.0, -4.0);
        // `new Color(222, 108, 48) * 0.7f` — the XNA truncating byte
        // multiply (`ui::color::mul`): (155, 75, 33, 178).
        let c = crate::ui::color::mul([222, 108, 48, 255], 0.7);
        let color = Color::new(c[0], c[1], c[2], c[3]);
        let lift = -0.3 + (npc.vy * 2.0).max(0.0);
        let bottom = Vec2::new(npc.x + w / 2.0, npc.y + h);
        for iter in 0..2 {
            if env.rng.next_max(2) != 0 {
                let idx = env.dust.new_dust(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    bottom,
                    0,
                    0,
                    268,
                    0.0,
                    0.0,
                    0,
                    Color::default(),
                    1.0,
                );
                {
                    let d = env.dust.slot_mut(idx);
                    // `:40194` — the Lerp re-seed draws TWO NextFloat
                    // (X first) on BOTH iterations, even when the
                    // iteration-1 overwrite below discards them.
                    let fx = env.rng.next_float();
                    let fy = env.rng.next_float();
                    d.position = Vec2::new(
                        bottom_left.x + (bottom_right.x - bottom_left.x) * fx,
                        bottom_left.y + (bottom_right.y - bottom_left.y) * fy,
                    ) + push;
                    if iter == 1 {
                        // `:40197` — the overwrite: TWO NextDouble.
                        d.position = bottom + random_vector2(env.rng, -6.0, 6.0);
                    }
                    d.color = color;
                    d.scale = 0.8;
                    d.velocity.y += lift;
                    d.velocity.x += npc.sprite_dir as f32 * 0.2;
                }
            }
        }
        npc.x -= npc.net_offset.x;
        npc.y -= npc.net_offset.y;
        // `:40206-40210` — the stuck reset + the negative count-up.
        npc.local_ai[2] = 0.0;
        if npc.ai[0] < 0.0 {
            npc.ai[0] = (npc.ai[0] + 1.0).min(0.0);
        }
        // `:40211-40312` — the charge arm.
        if npc.ai[0] > 0.0 {
            retarget = false;
            charging = true;
            npc.ai[0] += 1.0;
            if npc.ai[0] >= 135.0 {
                npc.ai[0] = -300.0;
                npc.net_update = true;
            }
            // `:40221` — vector216 (Center + UnitX*direction*200) is
            // computed and NEVER read — the documented dead local.
            // `:40222` — vector217, the trail's mouth.
            let (cx, cy) = npc.center();
            let mouth = Vec2::new(cx + npc.facing as f32 * 50.0, cy - 6.0);
            if npc.ai[0] == 54.0 && env.net_mode != 1 {
                place_sandnados(npc, env, &mut out);
            }
            if npc.ai[0] < 114.0 && npc.ai[0] > 0.0 {
                sandnado_trail(npc, env, mouth);
            }
            // `:40310-40311` — `_ = this.ai[0]; _ = 100f;` decompiler
            // residue, no effect.
        }
        // `:40313-40318` — the zero latch arms the charge.
        if npc.ai[0] == 0.0 {
            npc.ai[0] = 1.0;
            npc.net_update = true;
            charging = true;
        }
    }
    // `:40320-40323` — justHit clears the stuck counter.
    if npc.just_hit {
        npc.local_ai[2] = 0.0;
    }
    // `:40324-40375` — the stuck block (flag111 is always false).
    if npc.local_ai[2] >= 0.0 {
        let mut window = 16.0f32; // num1400
        let mut near_x = false; // flag118
        let mut near_y = false; // flag119
        if npc.x > npc.local_ai[0] - window && npc.x < npc.local_ai[0] + window {
            near_x = true;
        } else if (npc.vx < 0.0 && npc.facing > 0) || (npc.vx > 0.0 && npc.facing < 0) {
            near_x = true;
            window += 24.0;
        }
        if npc.y > npc.local_ai[1] - window && npc.y < npc.local_ai[1] + window {
            near_y = true;
        }
        if near_x & near_y {
            npc.local_ai[2] += 1.0;
            if npc.local_ai[2] >= 30.0 && window == 16.0 {
                jump = true;
            }
            if npc.local_ai[2] >= 60.0 {
                npc.local_ai[2] = -180.0;
                npc.facing *= -1;
                npc.vx *= -1.0;
                npc.collide_x = false;
            }
        } else {
            npc.local_ai[0] = npc.x;
            npc.local_ai[1] = npc.y;
            npc.local_ai[2] = 0.0;
        }
        if retarget && !env.players.is_empty() {
            // `:40367` — TargetClosest() defaults faceTarget = true;
            // aiStyle 102 (not the fighter 3, the centre rule applies).
            target_closest(npc, env.world, env.players, true, 102, env.ticker);
        }
    } else {
        // `:40370-40374` — the cooldown: count up, face the target.
        npc.local_ai[2] += 1.0;
        let p = target_of(npc, env.players);
        let (cx, _) = npc.center();
        let face = if p.x + (p.width / 2) as f32 > cx {
            1
        } else {
            -1
        };
        npc.facing = face;
    }
    // `:40376-40379` — the probe coordinates. `width / 2` is INTEGER;
    // num1403/num1404 cast BEFORE the /16 (truncation, not the f32 div).
    let col = ((npc.x + (w as i32 / 2) as f32) / 16.0) as i32 + npc.facing * 2;
    let row = ((npc.y + h) / 16.0) as i32;
    let foot_row = (npc.y + h) as i32 / 16;
    let foot_col = (npc.x + w / 2.0) as i32 / 16;
    // `:40380-40384` — the charge damp + early return.
    if charging {
        npc.vx *= charge_damp;
        npc.vy *= charge_damp;
        return out;
    }
    // `:40385-40400` — the forward probe (4 rows).
    for r in row..row + forward_depth {
        if probe_blocked(env.world, col, r) {
            if r <= row + 1 {
                ground_near = true;
            }
            gravity = false;
            break;
        }
    }
    // `:40401-40413` — the foot probe (num1380 rows; 3 for 541).
    for r in foot_row..foot_row + foot_depth {
        if probe_blocked(env.world, foot_col, r) {
            ground_near = true;
            gravity = false;
            break;
        }
    }
    // `:40414-40429` — the ceiling probe (non-541 only).
    if ceiling_probe {
        for r in row - ceiling_height..row {
            if probe_blocked(env.world, col, r) {
                ground_near = false;
                jump = true;
                break;
            }
        }
    }
    // `:40430-40434` — the jump forces gravity.
    if jump {
        ground_near = false;
        gravity = true;
    }
    // `:40435-40453` — gravity vs lift.
    if gravity {
        npc.vy += fall_accel;
        if npc.vy > fall_cap {
            npc.vy = fall_cap;
        }
    } else {
        if (npc.direction_y < 0 && npc.vy > 0.0) | ground_near {
            npc.vy += rise_accel;
        }
        if npc.vy < rise_cap {
            npc.vy = rise_cap;
        }
    }
    // `:40454-40465` — the collideX bounce.
    if npc.collide_x {
        npc.vx = npc.old_vx * -0.4;
        if npc.facing == -1 && npc.vx > 0.0 && npc.vx < 1.0 {
            npc.vx = 1.0;
        }
        if npc.facing == 1 && npc.vx < 0.0 && npc.vx > -1.0 {
            npc.vx = -1.0;
        }
    }
    // `:40466-40477` — the collideY bounce.
    if npc.collide_y {
        npc.vy = npc.old_vy * -0.25;
        if npc.vy > 0.0 && npc.vy < 1.0 {
            npc.vy = 1.0;
        }
        if npc.vy < 0.0 && npc.vy > -1.0 {
            npc.vy = -1.0;
        }
    }
    // `:40478-40509` — the X accel arms.
    if npc.facing == -1 && npc.vx > 0.0 - top_speed_x {
        npc.vx -= accel_x;
        if npc.vx > top_speed_x {
            npc.vx -= accel_x;
        } else if npc.vx > 0.0 {
            npc.vx += accel_x / 2.0;
        }
        if npc.vx < 0.0 - top_speed_x {
            npc.vx = 0.0 - top_speed_x;
        }
    } else if npc.facing == 1 && npc.vx < top_speed_x {
        npc.vx += accel_x;
        if npc.vx < 0.0 - top_speed_x {
            npc.vx += accel_x;
        } else if npc.vx < 0.0 {
            npc.vx -= accel_x / 2.0;
        }
        if npc.vx > top_speed_x {
            npc.vx = top_speed_x;
        }
    }
    // `:40510-40541` — the Y accel arms. The `:40523` clamp writes
    // `0f - num1385` (the X top speed) — the C#'s own bug, VERBATIM.
    if npc.direction_y == -1 && npc.vy > 0.0 - top_speed_y {
        npc.vy -= accel_y;
        if npc.vy > top_speed_y {
            npc.vy -= accel_y * 1.25;
        } else if npc.vy > 0.0 {
            npc.vy += accel_y * 0.75;
        }
        if npc.vy < 0.0 - top_speed_y {
            npc.vy = 0.0 - top_speed_x; // :40523 — NOT top_speed_y.
        }
    } else if npc.direction_y == 1 && npc.vy < top_speed_y {
        npc.vy += accel_y;
        if npc.vy < 0.0 - top_speed_y {
            npc.vy += accel_y * 1.25;
        } else if npc.vy < 0.0 {
            npc.vy -= accel_y * 0.75;
        }
        if npc.vy > top_speed_y {
            npc.vy = top_speed_y;
        }
    }
    out
}

/// `NPC.FindFrame` case 541 (`NPC.cs:69734-69772`) — one tick, as a
/// pure function over `(frame_row, frameCounter)` (the style73
/// `turret_find_frame` convention). `speed_len` is
/// `base.velocity.Length()`; the idle counter adds in DOUBLE (`:69761`)
/// and the 4-row walk wraps at `num * 4`. The charge arm leaves the
/// counter UNTOUCHED (the C# writes only `frame.Y`).
pub fn elemental_find_frame(
    frame_row: u32,
    frame_counter: f32,
    ai0: f32,
    speed_len: f32,
) -> (u32, f32) {
    if ai0 > 0.0 {
        // `:69735-69758` — the charge strip.
        let row = if ai0 < 6.0 {
            4
        } else if ai0 < 105.0 {
            // `(int)(ai0 / 8f % 4f + 5f)` — the f32 modulo then truncate.
            ((ai0 / 8.0) % 4.0 + 5.0) as u32
        } else if ai0 < 114.0 {
            9
        } else if ai0 < 135.0 {
            // `(int)((ai0 - 99f - 15f) / 7f + 10f)`.
            ((ai0 - 99.0 - 15.0) / 7.0 + 10.0) as u32
        } else {
            1 // `frame.Y = num` — row ONE, not zero.
        };
        (row, frame_counter)
    } else {
        // `:69759-69771` — the idle walk: the DOUBLE accumulation.
        let mut counter = frame_counter as f64 + (speed_len * 0.1) as f64 + 1.0;
        let mut row = frame_row;
        if counter >= 8.0 {
            row += 1;
            counter = 0.0;
        }
        if row >= 4 {
            row = 0;
        }
        (row, counter as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// A 541-shaped NPC (the style73 placeholder convention).
    fn elemental(x: f32, y: f32) -> Npc {
        let d = &SAND_ELEMENTAL;
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

    /// A flat dirt floor at row 120 (the stray.rs test shape).
    fn flat() -> World {
        let mut w = air();
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w
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

    fn menu_ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: true,
            ..spawn_ctx()
        }
    }

    /// `NPC.cs:15898-15911` — the SetDefaults row, field by field.
    #[test]
    fn w4_d33_s102_the_defaults_row() {
        let d = &SAND_ELEMENTAL;
        assert_eq!((d.width, d.height), (30, 76));
        assert_eq!(d.ai_style, 102);
        assert_eq!((d.damage, d.defense, d.life_max), (40, 30, 5000));
        assert_eq!(d.hit_sound, (3, 23));
        assert_eq!(d.death_sound, (4, 39));
        assert_eq!(d.knock_back_resist, 0.05);
        assert_eq!(d.value, 15000, "buyPrice(0, 1, 50)");
        assert_eq!(d.rarity, 2);
        assert_eq!(SAND_ELEMENTAL_FRAME_COUNT, 13);
    }

    /// `:40170-40182` — the 541 tuning: rotation = vx*0.04,
    /// spriteDirection = sign(direction); the life ramp lifts the X
    /// speed/accel and the below-half kb write.
    #[test]
    fn w4_d33_s102_the_life_ramp() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(100000.0, 100000.0)];
        let vx = [0.0f32];
        let mut rng = UnifiedRandom::new(102);
        let mut n = elemental(100.0, 100.0);
        n.vx = 1.0;
        n.hp = 2500; // exactly half — NOT below: no kb write yet
        n.ai[0] = -5.0; // no charge — the accel tail runs
        n.local_ai[2] = -5.0; // the cooldown arm — no retarget writes
        let mut env = Style102Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            players_vx: &vx,
            marks: &[],
            net_mode: 0,
            ticker: 7,
        };
        let _ = style102_step(&mut n, 541, &mut env);
        assert_eq!(n.rotation, 0.04);
        assert_eq!(n.sprite_dir, 1);
        assert_eq!(n.kb_resist_override, None, "half is not below half");
        // The X accel arm ran with 2 + 0.5*2 = 3 / 0.1 + 0.5*0.02 = 0.11.
        assert!((n.vx - 1.11).abs() < 1e-5, "vx {}", n.vx);
        let mut n = elemental(100.0, 100.0);
        n.hp = 2499;
        n.ai[0] = -5.0;
        n.local_ai[2] = -5.0;
        let mut env = Style102Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            players_vx: &vx,
            marks: &[],
            net_mode: 0,
            ticker: 7,
        };
        let _ = style102_step(&mut n, 541, &mut env);
        assert_eq!(n.kb_resist_override, Some(0.0), "below half zeroes kb");
    }

    /// `:40206-40218` + `:40313-40318` + `:40380-40384` — the charge
    /// latches: ai0=0 arms 1 + netUpdate; the negative count-up clamps
    /// at 0 (then arms the same tick); 134 → 135 → -300; the charge
    /// damps ×0.96 and RETURNS before gravity.
    #[test]
    fn w4_d33_s102_the_charge_latches() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(100000.0, 100000.0)];
        let vx = [0.0f32];
        let mut rng = UnifiedRandom::new(102);
        let mut env = Style102Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            players_vx: &vx,
            marks: &[],
            net_mode: 0,
            ticker: 7,
        };
        let mut n = elemental(100.0, 100.0);
        n.vx = 1.0;
        n.vy = 1.0;
        let _ = style102_step(&mut n, 541, &mut env);
        assert_eq!(n.ai[0], 1.0);
        assert!(n.net_update);
        assert!(
            (n.vx - 0.96).abs() < 1e-6 && (n.vy - 0.96).abs() < 1e-6,
            "the damp"
        );
        let mut n = elemental(100.0, 100.0);
        n.ai[0] = -1.0;
        let _ = style102_step(&mut n, 541, &mut env);
        assert_eq!(n.ai[0], 1.0, "-1 → 0 → the zero latch arms 1");
        let mut n = elemental(100.0, 100.0);
        n.ai[0] = 134.0;
        n.net_update = false;
        let _ = style102_step(&mut n, 541, &mut env);
        assert_eq!(n.ai[0], -300.0);
        assert!(n.net_update);
    }

    /// `:40326-40368` — the stuck counter: the anchor window counts,
    /// 60 flips the facing + vx and rests on -180; justHit zeroes it.
    /// The raw-counter cases ride a non-541 type (the 541 head zeroes
    /// localAI[2] first — the first case pins that order) and an EMPTY
    /// player slice so TargetClosest cannot re-face after the flip.
    #[test]
    fn w4_d33_s102_the_stuck_flip_and_just_hit() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players: [TargetCandidate; 0] = [];
        let vx: [f32; 0] = [];
        let mut rng = UnifiedRandom::new(102);
        let mut env = Style102Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            players_vx: &vx,
            marks: &[],
            net_mode: 0,
            ticker: 7,
        };
        let mut n = elemental(100.0, 100.0);
        n.ai[0] = 10.0; // charging — no retarget, no gravity tail
        n.local_ai[0] = 100.0;
        n.local_ai[1] = 100.0;
        n.local_ai[2] = 59.0;
        n.vx = 2.0;
        n.collide_x = true;
        let _ = style102_step(&mut n, 541, &mut env);
        // The head zeroed localAI[2] (:40206); the stuck block then
        // counts it back to 1 — the flip does NOT fire on this tick.
        assert_eq!(n.local_ai[2], 1.0, "the head zeroes it before the block");
        // Drive the non-541 skeleton for the raw counter (the head's
        // reset is 541-only).
        let mut n = elemental(100.0, 100.0);
        n.local_ai[0] = 100.0;
        n.local_ai[1] = 100.0;
        n.local_ai[2] = 59.0;
        n.vx = 2.0;
        n.collide_x = true;
        let _ = style102_step(&mut n, 542, &mut env);
        assert_eq!(n.local_ai[2], -180.0);
        assert_eq!(n.facing, -1);
        // The flip's vx = -2, then the facing -1 accel arm: -2 > -2 is
        // false — the tail leaves it (the collideX reset already ran).
        assert_eq!(n.vx, -2.0);
        assert!(!n.collide_x);
        let mut n = elemental(100.0, 100.0);
        n.local_ai[2] = 25.0;
        n.just_hit = true;
        let _ = style102_step(&mut n, 542, &mut env);
        assert!(n.local_ai[2] <= 1.0, "justHit zeroed the counter");
    }

    /// `:40189-40204` — the skirt draw census under the menu ctx: two
    /// Next(2) gates; a passing gate 0 draws TWO NextFloat; a passing
    /// gate 1 draws TWO NextFloat THEN TWO NextDouble (the overwrite
    /// does not skip the Lerp draws). The mirrored oracle replays it.
    #[test]
    fn w4_d33_s102_the_skirt_draws_in_order() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(100000.0, 100000.0)];
        let vx = [0.0f32];
        let mut rng = UnifiedRandom::new(102);
        let mut oracle = UnifiedRandom::new(102);
        let mut n = elemental(100.0, 100.0);
        let mut env = Style102Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            players_vx: &vx,
            marks: &[],
            net_mode: 0,
            ticker: 7,
        };
        let _ = style102_step(&mut n, 541, &mut env);
        for iter in 0..2 {
            if oracle.next_max(2) != 0 {
                oracle.next_float();
                oracle.next_float();
                if iter == 1 {
                    oracle.next_double();
                    oracle.next_double();
                }
            }
        }
        assert_eq!(rng.peek(), oracle.peek(), "the skirt draw order");
    }

    /// `:40223-40258` — the placement at ai0 == 54: the roll oracle
    /// (one Next(px-30, px+31) per attempt until 3 marks), the type-658
    /// zero-velocity marks at tile*16, and the client gate.
    #[test]
    fn w4_d33_s102_the_sandnado_placement() {
        let w = flat();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        // The player stands on the floor near the elemental.
        let players = [cand(300.0, 120.0 * 16.0 - 42.0)];
        let vx = [0.0f32];
        let mut rng = UnifiedRandom::new(102);
        let mut oracle = UnifiedRandom::new(102);
        let mut n = elemental(280.0, 120.0 * 16.0 - 76.0);
        n.ai[0] = 53.0;
        let mut env = Style102Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            players_vx: &vx,
            marks: &[],
            net_mode: 0,
            ticker: 7,
        };
        let out = style102_step(&mut n, 541, &mut env);
        assert_eq!(n.ai[0], 54.0);
        assert!(out.shots.iter().all(|s| s.projectile_type == 658));
        assert!(out.shots.len() <= 3);
        for s in &out.shots {
            assert_eq!(s.x % 16.0, 0.0);
            assert_eq!(s.y % 16.0, 0.0);
        }
        // The oracle: the skirt gates first, then the placement rolls.
        for iter in 0..2 {
            if oracle.next_max(2) != 0 {
                oracle.next_float();
                oracle.next_float();
                if iter == 1 {
                    oracle.next_double();
                    oracle.next_double();
                }
            }
        }
        let ptx = (300.0f32 + 10.0) as i32 >> 4;
        let mut accepted: Vec<i32> = Vec::new();
        for _ in 0..1000 {
            if accepted.len() >= 3 {
                break;
            }
            let col = oracle.next_range(ptx - 30, ptx + 30 + 1);
            if accepted.iter().any(|&x| (x - col).abs() < 10) {
                continue;
            }
            let (_, bottom_y) =
                expand_vertically(&w, col, ((120.0 * 16.0 - 21.0) as i32 >> 4) - 20, 1, 51);
            let spot = Vec2::new(col as f32, (bottom_y - 15) as f32) * 16.0;
            if can_spawn_sandstorm_hostile(&w, spot, 15, 15) {
                accepted.push(col);
            }
        }
        assert_eq!(out.shots.len(), accepted.len(), "the roll oracle count");
        assert_eq!(rng.peek(), oracle.peek(), "the placement draw order");
        // The client never fires.
        let mut n = elemental(280.0, 120.0 * 16.0 - 76.0);
        n.ai[0] = 53.0;
        env.net_mode = 1;
        let out = style102_step(&mut n, 541, &mut env);
        assert!(out.shots.is_empty());
        assert_eq!(n.ai[0], 54.0, "no -200 rest on the client");
    }

    /// `:40227` + `:40260-40264` — out of the 900 px range the arm
    /// rests: ai0 = -200 + netUpdate, no rolls drawn.
    #[test]
    fn w4_d33_s102_the_out_of_range_rest() {
        let w = flat();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(100000.0, 100000.0)];
        let vx = [0.0f32];
        let mut rng = UnifiedRandom::new(102);
        let mut n = elemental(280.0, 120.0 * 16.0 - 76.0);
        n.ai[0] = 53.0;
        n.net_update = false;
        let mut env = Style102Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            players_vx: &vx,
            marks: &[],
            net_mode: 0,
            ticker: 7,
        };
        let out = style102_step(&mut n, 541, &mut env);
        assert_eq!(n.ai[0], -200.0);
        assert!(n.net_update);
        assert!(out.shots.is_empty());
    }

    /// `:40280-40306` — the trail: at t=0 the 1500-px vertical handle
    /// lifts v4 ~35.6 px above v3 (the arc step > 10) → FOUR dusts per
    /// mark, each drawing TWO NextDouble after the (menu-ctx no-op)
    /// NewDust. netMode 1 keeps the placement rolls out of the stream.
    #[test]
    fn w4_d33_s102_the_trail_draws_four_per_mark_at_t0() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(100000.0, 100000.0)];
        let vx = [0.0f32];
        let mut rng = UnifiedRandom::new(102);
        let mut oracle = UnifiedRandom::new(102);
        let mut n = elemental(100.0, 100.0);
        n.ai[0] = 53.0; // ++ → 54 → t = 0
        let (cx, cy) = n.center();
        let mouth = Vec2::new(cx + 50.0, cy - 6.0);
        // v3 = mouth; v4 rides the 1500-px handle ~35.6 px up → the 4-band.
        let marks = [mouth + Vec2::new(200.0, 0.0), mouth + Vec2::new(200.0, 0.0)];
        let mut env = Style102Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            players_vx: &vx,
            marks: &marks,
            net_mode: 1,
            ticker: 7,
        };
        let _ = style102_step(&mut n, 541, &mut env);
        for iter in 0..2 {
            if oracle.next_max(2) != 0 {
                oracle.next_float();
                oracle.next_float();
                if iter == 1 {
                    oracle.next_double();
                    oracle.next_double();
                }
            }
        }
        for _ in 0..(4 * marks.len()) {
            oracle.next_double();
            oracle.next_double();
        }
        assert_eq!(rng.peek(), oracle.peek(), "4 dusts × 2 marks × 2 draws");
    }

    /// `:40435-40453` + `:40510-40524` — the :40523 clamp BUG: the
    /// upward Y clamp writes -num1385 (the X speed), not -num1387.
    /// Ground under the feet (flag113), vy = -0.88, directionY = -1:
    /// lift → -0.98, accel → -1.02 < -1 → clamp to -2 (full life). The
    /// EMPTY player slice keeps TargetClosest from re-facing directionY
    /// (the style24 select_target gate).
    #[test]
    fn w4_d33_s102_the_40523_clamp_uses_the_x_speed() {
        let mut w = air();
        // Solid under the foot column only (the forward probe stays air).
        let n0 = elemental(100.0, 100.0);
        let (ww, hh) = n0.size();
        let foot_col = (n0.x + ww / 2.0) as i32 / 16;
        let foot_row = (n0.y + hh) as i32 / 16;
        let i = w.index(foot_col, foot_row);
        w.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players: [TargetCandidate; 0] = [];
        let vx: [f32; 0] = [];
        let mut rng = UnifiedRandom::new(102);
        let mut n = elemental(100.0, 100.0);
        n.ai[0] = -5.0; // no charge — the tail runs
        n.vy = -0.88;
        n.direction_y = -1;
        let mut env = Style102Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            players_vx: &vx,
            marks: &[],
            net_mode: 0,
            ticker: 7,
        };
        let _ = style102_step(&mut n, 541, &mut env);
        assert_eq!(n.vy, -2.0, "the clamp wrote -num1385, not -num1387");
        // Half life: num1385 = 3 → the clamp lands on -3.
        let mut n = elemental(100.0, 100.0);
        n.ai[0] = -5.0;
        n.vy = -0.88;
        n.direction_y = -1;
        n.hp = 2500;
        let _ = style102_step(&mut n, 541, &mut env);
        assert_eq!(n.vy, -3.0, "the life-ramped X speed rides the bug");
    }

    /// `NPC.cs:69734-69772` — the FindFrame rows: the charge strip, the
    /// release ladder, the ≥135 row-ONE fallback, and the idle 4-row
    /// wrap with the double accumulation.
    #[test]
    fn w4_d33_s102_the_find_frame_rows() {
        assert_eq!(
            elemental_find_frame(0, 3.0, 5.0, 0.0),
            (4, 3.0),
            "<6 → row 4, counter untouched"
        );
        assert_eq!(elemental_find_frame(0, 0.0, 8.0, 0.0).0, 6, "8/8%4+5");
        assert_eq!(
            elemental_find_frame(0, 0.0, 104.0, 0.0).0,
            6,
            "104/8=13%4=1+5"
        );
        assert_eq!(elemental_find_frame(0, 0.0, 105.0, 0.0).0, 9);
        assert_eq!(
            elemental_find_frame(0, 0.0, 114.0, 0.0).0,
            10,
            "(114-114)/7+10"
        );
        assert_eq!(
            elemental_find_frame(0, 0.0, 134.0, 0.0).0,
            12,
            "(20)/7→2+10"
        );
        assert_eq!(
            elemental_find_frame(0, 0.0, 135.0, 0.0).0,
            1,
            "frame.Y = num — row ONE"
        );
        // The idle walk: len 0 → +1/tick; 7 → 8 advances and zeroes.
        assert_eq!(elemental_find_frame(0, 6.0, 0.0, 0.0), (0, 7.0));
        assert_eq!(elemental_find_frame(0, 7.0, 0.0, 0.0), (1, 0.0));
        assert_eq!(
            elemental_find_frame(3, 7.0, 0.0, 0.0),
            (0, 0.0),
            "the 4-row wrap"
        );
        // len 10 → +2/tick (10*0.1 + 1).
        assert_eq!(elemental_find_frame(0, 6.0, -1.0, 10.0), (1, 0.0));
    }

    /// The full-pool stream parity: 30 ticks on the REAL dust ctx must
    /// match a cloned twin bit-for-bit (the style101 twin precedent).
    #[test]
    fn w4_d33_s102_the_full_pool_run_is_twin_stable() {
        let w = flat();
        let c = spawn_ctx();
        let players = [cand(300.0, 120.0 * 16.0 - 42.0)];
        let vx = [0.5f32];
        let mut a = elemental(280.0, 120.0 * 16.0 - 76.0);
        let mut b = a.clone();
        let mut pool_a = DustPool::new();
        let mut pool_b = DustPool::new();
        let mut rng_a = UnifiedRandom::new(541);
        let mut rng_b = UnifiedRandom::new(541);
        for tick in 0..30 {
            let mut env_a = Style102Env {
                world: &w,
                dust: &mut pool_a,
                dust_ctx: &c,
                rng: &mut rng_a,
                players: &players,
                players_vx: &vx,
                marks: &[],
                net_mode: 0,
                ticker: 7,
            };
            let _ = style102_step(&mut a, 541, &mut env_a);
            let mut env_b = Style102Env {
                world: &w,
                dust: &mut pool_b,
                dust_ctx: &c,
                rng: &mut rng_b,
                players: &players,
                players_vx: &vx,
                marks: &[],
                net_mode: 0,
                ticker: 7,
            };
            let _ = style102_step(&mut b, 541, &mut env_b);
            assert_eq!(rng_a.peek(), rng_b.peek(), "stream @{tick}");
            assert_eq!(a.vx.to_bits(), b.vx.to_bits(), "vx @{tick}");
            assert_eq!(a.ai[0].to_bits(), b.ai[0].to_bits(), "ai0 @{tick}");
        }
    }
}
