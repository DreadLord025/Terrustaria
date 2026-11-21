//! The aiStyle-121 family — the Queen Slime
//! (`NPC.AI_121_QueenSlime`, `NPC.cs:45202-45812`, dispatched at
//! `:41104-41107`). W4-d33, research/25 row 121, CODE-ONLY MODE (no
//! compiler invoked; the witness tests below are written, not run).
//!
//! ## Census
//!
//! The family has exactly ONE vanilla member: the QueenSlimeBoss 657
//! (`NPCID.QueenSlimeBoss`, `ID/NPCID.cs:11991`; `SetDefaults` row
//! `NPC.cs:17310-17325`). The census of `this.aiStyle = 121;` returns
//! `NPC.cs:17314` plus one projectile-space hit (`Projectile.cs:6741`,
//! out of NPC scope). The port's roster has no queen-slime kind — the
//! module lands UNWIRED (the `npc_ai_style73.rs` precedent).
//!
//! ## The machine
//!
//! The Hallow boss: a grounded hopper (phase 1) that turns into a
//! flying caster (phase 2, below half life). `ai[0]` is the state
//! (0 idle, 1 teleport-in, 2 teleport-out, 3 the hop chain, 4 the
//! smash, 5 the gel ring), `ai[3]` the off-screen teleport charge.
//!
//! The head (`:45204-45234`): the `localAI[0] == 0` spawn latch
//! (`ai[1] = -100`, `localAI[0] = lifeMax`, TargetClosest); the
//! constant `(1, 0.7, 0.9)` light; the dead / >500-tile retarget +
//! EncourageDespawn(10) + the away-facing.
//!
//! The teleport charge (`:45235-45277`): the alive, >10-timeLeft,
//! phase-1, `ai[3] >= 300`, grounded idle triggers the teleport-out
//! (state 2) — the non-client `FindTeleportSpot` (the King-Slime cache
//! port, the anti-cheese at `ai[3] >= 360` or >2000 px). The
//! unsighted / >320-px-vertical charge `ai[3] += 1.5` else the
//! count-down (the netUpdate on the zero-cross).
//!
//! The despawn-state latch (`:45278-45292`): `timeLeft <= 10` flips
//! to state 0 (phase 2) / 3 (phase 1).
//!
//! The phase-2 frame pulse (`:45295-45310`): `localAI[3]` 0..24, the
//! state-4/5 overrides.
//!
//! State 0 (`:45313-45385`): phase 2 flies ([`queen_fly_movement`]),
//! phase 1 damps the grounded vx; the attack pick past 60/120 ticks —
//! phase 2 the `Next(2)` smash/gel with the player-position override,
//! phase 1 the `Next(3)` hop/smash/gel.
//!
//! State 1 (`:45386-45413`): the teleport-in scale ramp (0.5→1 over
//! 30), the 10-dust type-4 burst (×2 velocity).
//!
//! State 2 (`:45414-45449`): the teleport-out scale ramp (1→0.5 over
//! 60), the 1258 gore at 60, the Bottom re-anchor to the cached spot
//! → state 1; the 10-dust type-4 burst (×0.5 velocity).
//!
//! State 3 (`:45450-45527`): the grounded hop chain — the ai[1] += 4
//! (+4 per life-third), the ai[2] 0→1→2→3 hop ladder (-8/-6/-13 vy,
//! the 4/4.5/3.5 vx), the air steer (the 3/7 getGood cap, the 0.2
//! accel / 0.93 damp).
//!
//! State 4 (`:45528-45662`): the smash — ai[2]==1 the grounded slam
//! (the 922 QueenSlimeSmash at 40 damage, the Item167 cue, the 20-dust
//! type-31 burst, the 30/10/0-tick fall ramp with the 14/15.99 cap);
//! else the fly-up (the 30-tick windup, the 60-tick → ai[2]=1 + vy -3,
//! the player-or-self -384-px aim at 20 speed).
//!
//! State 5 (`:45663-45740`): the gel ring — ai[2]==1 past 10 ticks
//! fires the 10/15 (6 phase-1) type-926 QueenSlimeGelAttack ring at 30
//! damage; else the 50-tick windup, the 4-dust type-31
//! CircularEdge(100,100) orbit, the phase-2 fly.
//!
//! The tail (`:45742-45811`): `dontTakeDamage = hide = flag`; the
//! scale re-box (the Bottom-anchored 114×100 × scale); the life<=0
//! RETURN; the `Next(360) == 0` roar (PlaySound 65); the client
//! RETURN; the phase-transition latch; the 2%/1.5%-life minion spawn
//! (`Next(1,3)` count, the 658/659/660 pick, the velocity + ai[0]
//! payload).
//!
//! [`queen_dust_color`] (`AI_121_QueenSlime_GetDustColor`,
//! `:45121-45124`): the TWO-NextFloat nested Lerp (the inner
//! (200,200,200)↔(255,80,255), the outer (0,160,255)↔inner).
//!
//! ## RNG order (by construction)
//!
//! The attack pick: Next(2)/Next(3). The dust colours: TWO NextFloat
//! each (the inner then the outer). The state-4 slam dusts: the
//! per-dust NextFloat/Next(8)/Next(2)/NextFloatDirection. The state-5
//! orbit: the per-dust CircularEdge NextFloat. The roar: Next(360).
//! The minion spawn: Next(1,3), then per-minion Next(w-32)/Next(h-32)/
//! Next(3)/Next(-15,16)/Next(-30,1)/Next(3). The teleport cache draws
//! on the shared king-slime port. TargetClosest is RNG-free.
//!
//! ## FindFrame case 657
//!
//! `NPC.cs:66848-66959`: the phase-2/noGravity-or-rising 4..7/20..23
//! cycles, the falling 8..10 cycle, the grounded state-5/4 row maps
//! and the 0..3/10..12 idle cycle. `Main.npcFrameCount[657] = 16` (the
//! sheet is 180-wide, the rows are the 24-frame strip).
//! [`queen_find_frame`] ports the machine (the style73 convention).
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::bow_launch::safe_normalize;
use crate::npc::Npc;
use crate::npc_ai_fishron::simple_fly_movement;
use crate::npc_ai_style24::find_closest_player;
use crate::npc_ai_style37::rotated_by;
use crate::npc_target::{target_closest, TargetCandidate};
use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// The type-657 `SetDefaults` row (`NPC.cs:17310-17325`), verbatim.
pub struct QueenSlimeDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit1`.
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath64` (`SoundID.cs:1378`).
    pub death_sound: (i32, i32),
    pub knock_back_resist: f32,
    pub value: f32,
    pub lava_immune: bool,
    pub boss: bool,
    pub npc_slots: f32,
}

/// `NPC.cs:17310-17325` — `else if (this.type == 657)`.
pub const QUEEN_SLIME: QueenSlimeDefaults = QueenSlimeDefaults {
    width: 114,
    height: 100,
    ai_style: 121,
    damage: 60,
    defense: 26,
    life_max: 18000,
    hit_sound: (3, 1),
    death_sound: (4, 64),
    knock_back_resist: 0.0,
    value: 60000.0,
    lava_immune: true,
    boss: true,
    npc_slots: 5.0,
};

/// `Main.npcFrameCount[657]` = 16 (`Main.cs:67149`, the 650-659 row).
pub const QUEEN_SLIME_FRAME_COUNT: u32 = 16;

/// The per-tick environment.
pub struct Style121Env<'a> {
    pub world: &'a World,
    pub dust: &'a mut DustPool,
    pub dust_ctx: &'a SpawnCtx,
    pub gore: &'a mut GorePool,
    pub rng: &'a mut UnifiedRandom,
    /// The `Main.player[]` slots as `TargetClosest` sees them.
    pub players: &'a [TargetCandidate],
    /// `Main.netMode`.
    pub net_mode: i32,
    /// `Main.getGoodWorld` — the hop cap / smash ramp / gel count.
    pub get_good_world: bool,
    /// The current `noTileCollide` / `noGravity` (the `Npc` slot has no
    /// fields — the env carries them in, the outcome carries the last
    /// writes out).
    pub no_tile_collide: bool,
    pub no_gravity: bool,
    /// `Main.EverLastingTicker` for the selection's `targetSetFrame`.
    pub ticker: u64,
}

/// A `Projectile.NewProjectile` request — the smash (922) or the gel
/// ring (926).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QueenShot {
    pub projectile_type: i32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
}

/// A `NPC.NewNPC` request — the minion spawn (`:45801`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QueenMinionSpawn {
    /// 658 / 659 / 660 (the Blue/Pink/Purple minions).
    pub npc_type: i32,
    pub x: i32,
    pub y: i32,
    pub vx: f32,
    pub vy: f32,
    pub ai0: f32,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style121Outcome {
    /// The projectile requests in C# order.
    pub shots: Vec<QueenShot>,
    /// The minion spawn requests in C# order.
    pub minions: Vec<QueenMinionSpawn>,
    /// The `SoundEngine.PlaySound` cues — `(type, style, x, y)`:
    /// Item167 `(2, 167)` (the slam), the roar `(65, 1)`.
    pub sounds: Vec<(i32, i32, f32, f32)>,
    /// The constant `Lighting.AddLight(Center, 1, 0.7, 0.9)`
    /// (`:45217`).
    pub light: Option<(f32, f32, [f32; 3])>,
    /// The last `noTileCollide` / `noGravity` writes.
    pub no_tile_collide: bool,
    pub no_gravity: bool,
    /// `this.hide` (`:45742`) — the `Npc` slot has no field.
    pub hide: bool,
}

/// XNA `Vector2.Length()` — the f32 sum widened to f64 for the root.
fn length(v: Vec2) -> f32 {
    ((v.x * v.x + v.y * v.y) as f64).sqrt() as f32
}

/// XNA `Vector2.Normalize` — UNGUARDED (the style100 convention).
fn normalize_xna(v: Vec2) -> Vec2 {
    let n = 1.0 / ((v.x * v.x + v.y * v.y) as f64).sqrt() as f32;
    Vec2::new(v.x * n, v.y * n)
}

/// XNA `Color.Lerp` — the per-channel truncating Lerp (the dust_table
/// shape).
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let ch = |x: u8, y: u8| ((x as f32 + (y as f32 - x as f32) * t) as i32) as u8;
    Color::new(ch(a.r, b.r), ch(a.g, b.g), ch(a.b, b.b), ch(a.a, b.a))
}

/// `Utils.NextVector2Unit` (`Utils.cs:1261-1264`) — ONE NextFloat.
fn next_vector2_unit(rng: &mut UnifiedRandom) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32)
}

/// `Utils.NextVector2CircularEdge` (`Utils.cs:1271-1274`) — ONE
/// NextFloat, no radius scale.
fn next_vector2_circular_edge(rng: &mut UnifiedRandom, half_w: f32, half_h: f32) -> Vec2 {
    let unit = next_vector2_unit(rng);
    Vec2::new(unit.x * half_w, unit.y * half_h)
}

/// The target slot's candidate — the style16 `target_of` fallback.
fn target_of<'a>(npc: &Npc, players: &'a [TargetCandidate]) -> &'a TargetCandidate {
    let i = if npc.target >= 0 && (npc.target as usize) < players.len() {
        npc.target as usize
    } else {
        0
    };
    &players[i]
}

/// `AI_121_QueenSlime_GetDustColor` (`NPC.cs:45121-45124`): the
/// TWO-NextFloat nested Lerp — the inner (200,200,200)↔(255,80,255)
/// draws FIRST, then the outer (0,160,255)↔inner.
fn queen_dust_color(rng: &mut UnifiedRandom) -> Color {
    let inner = lerp_color(
        Color::new(200, 200, 200, 255),
        Color::new(255, 80, 255, 255),
        rng.next_float(),
    );
    lerp_color(Color::new(0, 160, 255, 255), inner, rng.next_float())
}

/// `AI_121_QueenSlime_FlyMovement` (`NPC.cs:45126-45200`): the phase-2
/// fly — the noTileCollide/noGravity set, the TargetClosest, the
/// sighted hover-250 / unsighted 16-row ceiling scan / despawn flee,
/// the 40-px X-hold, the 100-px double-accel, the 40/80 speed bands,
/// the SimpleFlyMovement, the ±0.5 rotation clamp.
fn queen_fly_movement(npc: &mut Npc, env: &mut Style121Env<'_>, out: &mut Style121Outcome) {
    out.no_tile_collide = true;
    out.no_gravity = true;
    let max_speed = 12.0f32; // num
    let mut accel = 0.085f32; // num2
    let hover = 250.0f32; // num3
    if !env.players.is_empty() {
        target_closest(npc, env.world, env.players, true, 121, env.ticker);
    }
    let (w, h) = npc.size();
    let center = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
    let p = target_of(npc, env.players);
    let pc = Vec2::new(p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32);
    let mut desired = center; // desiredVelocity (the C# inits to Center)
    if npc.despawn_timer > 10 {
        let sighted = crate::projectile::can_hit(
            env.world,
            npc.x,
            npc.y,
            w,
            h,
            p.x,
            p.y,
            p.width as f32,
            p.height as f32,
        );
        if !sighted {
            let mut found = false;
            for i in 0..16 {
                let rise = 16 * i;
                let probe = pc + Vec2::new(0.0, -rise as f32);
                let tx = probe.x as i32 >> 4;
                let ty = probe.y as i32 >> 4;
                let solid = env
                    .world
                    .tile(tx, ty)
                    .is_some_and(|t| crate::collision::solid_or_sloped_tile(t, false));
                if solid {
                    desired = pc + Vec2::new(0.0, -rise as f32 + 16.0) - center;
                    found = true;
                    break;
                }
            }
            if !found {
                desired = pc - center;
            }
        } else {
            desired = pc + Vec2::new(0.0, -hover) - center;
        }
    } else {
        desired = center + Vec2::new(500.0 * npc.facing as f32, -hover) - center;
    }
    let dist = length(desired); // num5
    if desired.x.abs() < 40.0 {
        desired.x = npc.vx;
    }
    let band = max_speed - 2.0; // num6
    if dist > 100.0
        && ((npc.vx < 0.0 - band && desired.x > 0.0) || (npc.vx > band && desired.x < 0.0))
    {
        accel *= 2.0;
    }
    if dist < 40.0 {
        desired = Vec2::new(npc.vx, npc.vy);
    } else if dist < 80.0 {
        desired = normalize_xna(desired) * (max_speed * 0.65);
    } else {
        desired = normalize_xna(desired) * max_speed;
    }
    simple_fly_movement(&mut npc.vx, &mut npc.vy, desired.x, desired.y, accel);
    npc.rotation = npc.vx * 0.1;
    if npc.rotation > 0.5 {
        npc.rotation = 0.5;
    }
    if npc.rotation < -0.5 {
        npc.rotation = -0.5;
    }
}

/// One fixed step of the aiStyle-121 arm (`NPC.cs:45202-45812`), in C#
/// order. The caller owns the movement integration and drains
/// [`Style121Outcome`].
pub fn style121_step(npc: &mut Npc, env: &mut Style121Env<'_>) -> Style121Outcome {
    let mut out = Style121Outcome::default();
    let smash_damage = 40; // num2
    let gel_damage = 30; // num
    let mut scale = 1.0f32; // num3
    let mut hide = false; // flag
    let phase2 = npc.hp <= npc.max_hp / 2; // flag2
    let (w, h) = npc.size();
    let center = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
    // `:45210-45216` — the spawn latch.
    if npc.local_ai[0] == 0.0 {
        npc.ai[1] = -100.0;
        npc.local_ai[0] = npc.max_hp as f32;
        if !env.players.is_empty() {
            target_closest(npc, env.world, env.players, true, 121, env.ticker);
        }
        npc.net_update = true;
    }
    // `:45217` — the constant light.
    out.light = Some((center.x, center.y, [1.0, 0.7, 0.9]));
    // `:45218-45234` — the retarget + despawn.
    let range = 500; // num4
    {
        let p = target_of(npc, env.players);
        let pcx = p.x + (p.width / 2) as f32;
        if p.dead || (center.x - pcx).abs() / 16.0 > range as f32 {
            if !env.players.is_empty() {
                target_closest(npc, env.world, env.players, true, 121, env.ticker);
            }
            let p = target_of(npc, env.players);
            let pcx = p.x + (p.width / 2) as f32;
            if p.dead || (center.x - pcx).abs() / 16.0 > range as f32 {
                npc.encourage_despawn(10);
                npc.facing = if pcx < center.x { 1 } else { -1 };
            }
        }
    }
    // `:45235-45260` — the teleport-out trigger.
    {
        let p = target_of(npc, env.players);
        if !p.dead
            && npc.despawn_timer > 10
            && !phase2
            && npc.ai[3] >= 300.0
            && npc.ai[0] == 0.0
            && npc.vy == 0.0
        {
            npc.ai[0] = 2.0;
            npc.ai[1] = 0.0;
            if env.net_mode != 1 {
                npc.net_update = true;
                target_closest(npc, env.world, env.players, false, 121, env.ticker);
                let p = target_of(npc, env.players);
                let pc = Vec2::new(p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32);
                let to_player = pc - center;
                let mut anti_cheese = false;
                if npc.ai[3] >= 360.0 || length(to_player) > 2000.0 {
                    if npc.ai[3] >= 360.0 {
                        npc.ai[3] = 360.0;
                    }
                    anti_cheese = true;
                }
                // The King-Slime teleport cache (the shared candidate
                // core). The C# returns false on the anti-cheese /
                // no-cache fall back to the player's Bottom; the port's
                // helper folds both into the Bottom return — the
                // ai[3]=0 reset on the false branch is the wiring's
                // (the helper cannot signal it; documented).
                let p = target_of(npc, env.players);
                let pc = Vec2::new(p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32);
                let p_bottom = (pc.x, p.y + p.height as f32);
                let spot = crate::npc::king_slime_find_teleport_spot_at(
                    env.world,
                    pc.x,
                    pc.y,
                    p_bottom,
                    anti_cheese,
                    h,
                    env.rng,
                );
                npc.local_ai[1] = spot.0;
                npc.local_ai[2] = spot.1;
            }
        }
    }
    // `:45261-45277` — the teleport charge.
    {
        let p = target_of(npc, env.players);
        let pc = Vec2::new(p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32);
        let p_bottom = p.y + p.height as f32;
        let sighted = crate::projectile::can_hit_line(
            env.world, center.x, center.y, 0.0, 0.0, pc.x, pc.y, 0.0, 0.0,
        );
        if !phase2 && (!sighted || (npc.y - p_bottom).abs() > 320.0) {
            npc.ai[3] += 1.5;
        } else {
            let prev = npc.ai[3];
            npc.ai[3] -= 1.0;
            if npc.ai[3] < 0.0 {
                if env.net_mode != 1 && prev > 0.0 {
                    npc.net_update = true;
                }
                npc.ai[3] = 0.0;
            }
        }
    }
    // `:45278-45292` — the despawn-state latch.
    if npc.despawn_timer <= 10 && ((phase2 && npc.ai[0] != 0.0) || (!phase2 && npc.ai[0] != 3.0)) {
        npc.ai[0] = if phase2 { 0.0 } else { 3.0 };
        npc.ai[1] = 0.0;
        npc.ai[2] = 0.0;
        npc.ai[3] = 0.0;
        npc.net_update = true;
    }
    // `:45293-45294` — the per-tick collision reset.
    out.no_tile_collide = false;
    out.no_gravity = false;
    // `:45295-45310` — the phase-2 frame pulse.
    if phase2 {
        npc.local_ai[3] += 1.0;
        if npc.local_ai[3] >= 24.0 {
            npc.local_ai[3] = 0.0;
        }
        if npc.ai[0] == 4.0 && npc.ai[2] == 1.0 {
            npc.local_ai[3] = 6.0;
        }
        if npc.ai[0] == 5.0 && npc.ai[2] != 1.0 {
            npc.local_ai[3] = 7.0;
        }
    }
    match npc.ai[0] as i32 {
        0 => {
            // `:45313-45385` — the idle.
            if phase2 {
                queen_fly_movement(npc, env, &mut out);
            } else {
                out.no_tile_collide = false;
                out.no_gravity = false;
                if npc.vy == 0.0 {
                    npc.vx *= 0.8;
                    if (npc.vx as f64) > -0.1 && (npc.vx as f64) < 0.1 {
                        npc.vx = 0.0;
                    }
                }
            }
            if npc.despawn_timer <= 10 || (!phase2 && npc.vy != 0.0) {
                // break
            } else {
                npc.ai[1] += 1.0;
                let mut wait = 60; // num10
                if phase2 {
                    wait = 120;
                }
                if npc.ai[1] > wait as f32 {
                    npc.ai[1] = 0.0;
                    if phase2 {
                        let p = target_of(npc, env.players);
                        npc.ai[0] = if env.rng.next_max(2) != 1 { 4.0 } else { 5.0 };
                        if npc.ai[0] == 4.0 {
                            npc.ai[2] = 1.0;
                            let pc =
                                Vec2::new(p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32);
                            let p_bottom = p.y + p.height as f32;
                            let bottom = npc.y + h;
                            if p.active
                                && !p.dead
                                && (p_bottom < bottom || (pc.x - center.x).abs() > 250.0)
                            {
                                npc.ai[0] = 5.0;
                                npc.ai[2] = 0.0;
                            }
                        }
                    } else {
                        npc.ai[0] = match env.rng.next_max(3) {
                            1 => 4.0,
                            2 => 5.0,
                            _ => 3.0,
                        };
                    }
                    npc.net_update = true;
                }
            }
        }
        1 => {
            // `:45386-45413` — the teleport-in.
            npc.rotation = 0.0;
            npc.ai[1] += 1.0;
            scale = 0.5 + (npc.ai[1] / 30.0).clamp(0.0, 1.0) * 0.5;
            if npc.ai[1] >= 30.0 && env.net_mode != 1 {
                npc.ai[0] = 0.0;
                npc.ai[1] = 0.0;
                npc.net_update = true;
                if !env.players.is_empty() {
                    target_closest(npc, env.world, env.players, true, 121, env.ticker);
                }
            }
            if env.net_mode == 1 && npc.ai[1] >= 60.0 {
                npc.ai[0] = 0.0;
                npc.ai[1] = 0.0;
                if !env.players.is_empty() {
                    target_closest(npc, env.world, env.players, true, 121, env.ticker);
                }
            }
            let mut color = queen_dust_color(env.rng);
            color.a = 150;
            for _ in 0..10 {
                let idx = env.dust.new_dust(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    Vec2::new(npc.x - 20.0, npc.y),
                    w as i32 + 40,
                    h as i32,
                    4,
                    npc.vx,
                    npc.vy,
                    50,
                    color,
                    1.5,
                );
                let d = env.dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 2.0;
            }
        }
        2 => {
            // `:45414-45449` — the teleport-out.
            npc.rotation = 0.0;
            npc.ai[1] += 1.0;
            scale = 0.5 + ((60.0 - npc.ai[1]) / 60.0).clamp(0.0, 1.0) * 0.5;
            if npc.ai[1] >= 60.0 {
                hide = true;
            }
            if npc.ai[1] == 60.0 {
                // Gore.NewGore(Center + (-40, -h/2), velocity, 1258).
                env.gore.new_gore(
                    env.rng,
                    env.world,
                    center + Vec2::new(-40.0, -h / 2.0),
                    Vec2::new(npc.vx, npc.vy),
                    1258,
                    1.0,
                );
            }
            if npc.ai[1] >= 60.0 && env.net_mode != 1 {
                // base.Bottom = (localAI[1], localAI[2]).
                npc.x = npc.local_ai[1] - w / 2.0;
                npc.y = npc.local_ai[2] - h;
                npc.ai[0] = 1.0;
                npc.ai[1] = 0.0;
                npc.net_update = true;
            }
            if env.net_mode == 1 && npc.ai[1] >= 120.0 {
                npc.ai[0] = 1.0;
                npc.ai[1] = 0.0;
            }
            if !hide {
                let mut color = queen_dust_color(env.rng);
                color.a = 150;
                for _ in 0..10 {
                    let idx = env.dust.new_dust(
                        env.rng,
                        env.dust_ctx,
                        env.world,
                        Vec2::new(npc.x - 20.0, npc.y),
                        w as i32 + 40,
                        h as i32,
                        4,
                        npc.vx,
                        npc.vy,
                        50,
                        color,
                        1.5,
                    );
                    let d = env.dust.slot_mut(idx);
                    d.no_gravity = true;
                    d.velocity *= 0.5;
                }
            }
        }
        3 => {
            // `:45450-45527` — the hop chain.
            npc.rotation = 0.0;
            if npc.vy == 0.0 {
                npc.vx *= 0.8;
                if (npc.vx as f64) > -0.1 && (npc.vx as f64) < 0.1 {
                    npc.vx = 0.0;
                }
                npc.ai[1] += 4.0;
                if (npc.hp as f64) < (npc.max_hp as f64) * 0.66 {
                    npc.ai[1] += 4.0;
                }
                if (npc.hp as f64) < (npc.max_hp as f64) * 0.33 {
                    npc.ai[1] += 4.0;
                }
                if npc.ai[1] >= 0.0 {
                    npc.net_update = true;
                    if !env.players.is_empty() {
                        target_closest(npc, env.world, env.players, true, 121, env.ticker);
                    }
                    if npc.ai[2] == 3.0 {
                        npc.vy = -13.0;
                        npc.vx += 3.5 * npc.facing as f32;
                        npc.ai[1] = 0.0;
                        npc.ai[2] = 0.0;
                        if npc.despawn_timer > 10 {
                            npc.ai[0] = 0.0;
                        } else {
                            npc.ai[1] = -60.0;
                        }
                    } else if npc.ai[2] == 2.0 {
                        npc.vy = -6.0;
                        npc.vx += 4.5 * npc.facing as f32;
                        npc.ai[1] = -40.0;
                        npc.ai[2] += 1.0;
                    } else {
                        npc.vy = -8.0;
                        npc.vx += 4.0 * npc.facing as f32;
                        npc.ai[1] = -40.0;
                        npc.ai[2] += 1.0;
                    }
                }
            } else {
                if npc.target >= 255 {
                    // break
                } else {
                    let mut cap = 3.0f32; // num11
                    if env.get_good_world {
                        cap = 7.0;
                    }
                    if (npc.facing == 1 && npc.vx < cap) || (npc.facing == -1 && npc.vx > 0.0 - cap)
                    {
                        if (npc.facing == -1 && (npc.vx as f64) < 0.1)
                            || (npc.facing == 1 && (npc.vx as f64) > -0.1)
                        {
                            npc.vx += 0.2 * npc.facing as f32;
                        } else {
                            npc.vx *= 0.93;
                        }
                    }
                }
            }
        }
        4 => {
            // `:45528-45662` — the smash.
            npc.rotation *= 0.9;
            out.no_tile_collide = true;
            out.no_gravity = true;
            if npc.ai[2] == 1.0 {
                out.no_tile_collide = false;
                out.no_gravity = false;
                let mut ramp = 30; // num12
                if phase2 {
                    ramp = 10;
                }
                if env.get_good_world {
                    ramp = 0;
                }
                if npc.vy == 0.0 {
                    npc.ai[0] = 0.0;
                    npc.ai[1] = 0.0;
                    npc.ai[2] = 0.0;
                    npc.net_update = true;
                    out.sounds.push((2, 167, center.x, center.y));
                    if env.net_mode != 1 {
                        let bottom = Vec2::new(npc.x + w / 2.0, npc.y + h);
                        out.shots.push(QueenShot {
                            projectile_type: 922,
                            x: bottom.x,
                            y: bottom.y,
                            vx: 0.0,
                            vy: 0.0,
                            damage: smash_damage,
                        });
                    }
                    for _ in 0..20 {
                        let color = queen_dust_color(env.rng);
                        let idx = env.dust.new_dust(
                            env.rng,
                            env.dust_ctx,
                            env.world,
                            Vec2::new(npc.x + w / 2.0 - (w as i32 / 2) as f32, npc.y + h - 30.0),
                            w as i32,
                            30,
                            31,
                            npc.vx,
                            npc.vy,
                            40,
                            color,
                            1.0,
                        );
                        let d = env.dust.slot_mut(idx);
                        d.no_gravity = true;
                        d.velocity.y = -5.0 + env.rng.next_float() * -3.0;
                        d.velocity.x *= 7.0;
                    }
                } else if npc.ai[1] >= ramp as f32 {
                    for _ in 0..4 {
                        let spot = Vec2::new(
                            npc.x + w / 2.0 - env.rng.next_float_direction() * 16.0,
                            npc.y + h - env.rng.next_max(8) as f32,
                        );
                        let color = queen_dust_color(env.rng);
                        let idx = env.dust.new_dust(
                            env.rng,
                            env.dust_ctx,
                            env.world,
                            spot,
                            2,
                            2,
                            31,
                            npc.vx,
                            npc.vy,
                            40,
                            color,
                            1.4,
                        );
                        let d = env.dust.slot_mut(idx);
                        d.position = spot;
                        d.no_gravity = true;
                        d.velocity.y = npc.vy * 0.9;
                        d.velocity.x = (if env.rng.next_max(2) == 0 {
                            -10.0
                        } else {
                            10.0
                        }) + env.rng.next_float_direction() * 3.0;
                    }
                }
                npc.vx *= 0.8;
                let prev = npc.ai[1]; // num15
                npc.ai[1] += 1.0;
                if npc.ai[1] >= ramp as f32 {
                    if prev < ramp as f32 {
                        npc.net_update = true;
                    }
                    if phase2 && npc.ai[1] > (ramp + 120) as f32 {
                        npc.ai[0] = 0.0;
                        npc.ai[1] = 0.0;
                        npc.ai[2] = 0.0;
                        npc.vy *= 0.8;
                        npc.net_update = true;
                    } else {
                        npc.vy += 1.0;
                        let mut fall_cap = 14.0f32; // num16
                        if env.get_good_world {
                            npc.vy += 1.0;
                            fall_cap = 15.99;
                        }
                        if npc.vy == 0.0 {
                            npc.vy = 0.01;
                        }
                        if npc.vy >= fall_cap {
                            npc.vy = fall_cap;
                        }
                    }
                } else {
                    npc.vy *= 0.8;
                }
            } else {
                if env.net_mode != 1 && npc.ai[1] == 0.0 {
                    if !env.players.is_empty() {
                        target_closest(npc, env.world, env.players, true, 121, env.ticker);
                    }
                    npc.net_update = true;
                }
                npc.ai[1] += 1.0;
                if npc.ai[1] >= 30.0 {
                    if npc.ai[1] >= 60.0 {
                        npc.ai[1] = 60.0;
                        if env.net_mode != 1 {
                            npc.ai[1] = 0.0;
                            npc.ai[2] = 1.0;
                            npc.vy = -3.0;
                            npc.net_update = true;
                        }
                    }
                    let p = target_of(npc, env.players);
                    let pc = Vec2::new(p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32);
                    let mut aim = center;
                    if !p.dead && p.active && (center.x - pc.x).abs() / 16.0 <= range as f32 {
                        aim = pc;
                    }
                    aim.y -= 384.0;
                    if npc.vy == 0.0 {
                        let v = safe_normalize(aim - center, Vec2::ZERO) * 20.0;
                        npc.vx = v.x;
                        npc.vy = v.y;
                    } else {
                        npc.vy *= 0.95;
                    }
                }
            }
        }
        5 => {
            // `:45663-45740` — the gel ring.
            npc.rotation *= 0.9;
            out.no_tile_collide = true;
            out.no_gravity = true;
            if phase2 {
                npc.ai[3] = 0.0;
            }
            if npc.ai[2] == 1.0 {
                npc.ai[1] += 1.0;
                if npc.ai[1] >= 10.0 {
                    if env.net_mode != 1 {
                        let mut ring = 10; // num6
                        if env.get_good_world {
                            ring = 15;
                        }
                        let mut count = ring; // num7
                        if !phase2 {
                            count = 6;
                        }
                        for i in 0..count {
                            let angle =
                                (-i as f32 * (std::f32::consts::PI * 2.0) / ring as f32) as f64;
                            let (vx, vy) = rotated_by(9.0, 0.0, angle, 0.0, 0.0);
                            out.shots.push(QueenShot {
                                projectile_type: 926,
                                x: center.x,
                                y: center.y,
                                vx,
                                vy,
                                damage: gel_damage,
                            });
                        }
                    }
                    npc.ai[0] = 0.0;
                    npc.ai[1] = 0.0;
                    npc.ai[2] = 0.0;
                    npc.net_update = true;
                }
            } else {
                if env.net_mode != 1 && npc.ai[1] == 0.0 {
                    if !env.players.is_empty() {
                        target_closest(npc, env.world, env.players, true, 121, env.ticker);
                    }
                    npc.net_update = true;
                }
                npc.ai[1] += 1.0;
                if npc.ai[1] >= 50.0 {
                    npc.ai[1] = 50.0;
                    if env.net_mode != 1 {
                        npc.ai[1] = 0.0;
                        npc.ai[2] = 1.0;
                        npc.net_update = true;
                    }
                }
                let orbit = 100.0f32; // num8
                for _ in 0..4 {
                    let mut spot = center + next_vector2_circular_edge(env.rng, orbit, orbit);
                    if !phase2 {
                        spot += Vec2::new(0.0, 20.0);
                    }
                    let vel = safe_normalize(spot - center, Vec2::ZERO) * -8.0;
                    let color = queen_dust_color(env.rng);
                    let idx = env.dust.new_dust(
                        env.rng,
                        env.dust_ctx,
                        env.world,
                        spot,
                        2,
                        2,
                        31,
                        vel.x,
                        vel.y,
                        40,
                        color,
                        1.8,
                    );
                    let d = env.dust.slot_mut(idx);
                    d.position = spot;
                    d.no_gravity = true;
                    d.alpha = 250;
                    d.velocity = vel;
                    // `customData = this` — the dropped field.
                }
                if phase2 {
                    queen_fly_movement(npc, env, &mut out);
                }
            }
        }
        _ => {}
    }
    // `:45742` — the hide/dontTakeDamage pair.
    npc.dont_take_damage = hide;
    out.hide = hide;
    // `:45743-45752` — the scale re-box (the Bottom anchor).
    if scale != npc.scale {
        npc.x += (w as i32 / 2) as f32;
        npc.y += h;
        npc.scale = scale;
        let new_w = (114.0 * scale) as i32;
        let new_h = (100.0 * scale) as i32;
        npc.size_override = Some((new_w as f32, new_h as f32));
        npc.x -= (new_w / 2) as f32;
        npc.y -= new_h as f32;
    }
    // `:45753-45756` — the life<=0 return.
    if npc.hp <= 0 {
        return out;
    }
    // `:45757-45760` — the roar.
    if env.rng.next_max(360) == 0 {
        out.sounds.push((65, 1, npc.x, npc.y));
    }
    // `:45761-45764` — the client return.
    if env.net_mode == 1 {
        return out;
    }
    // `:45765-45772` — the phase-transition latch.
    if npc.local_ai[0] >= (npc.max_hp / 2) as f32 && npc.hp < npc.max_hp / 2 {
        npc.local_ai[0] = npc.hp as f32;
        npc.ai[0] = 0.0;
        npc.ai[1] = 0.0;
        npc.ai[2] = 0.0;
        npc.net_update = true;
    }
    // `:45773-45811` — the minion spawn.
    let mut threshold = (npc.max_hp as f32 * 0.02) as i32; // num19
    if phase2 {
        threshold = (npc.max_hp as f32 * 0.015) as i32;
    }
    if (npc.hp + threshold) as f32 >= npc.local_ai[0] {
        return out;
    }
    npc.local_ai[0] = npc.hp as f32;
    let count = env.rng.next_range(1, 3); // num20
    let (cw, ch) = npc.size();
    for _ in 0..count {
        let x = (npc.x + env.rng.next_max(cw as i32 - 32) as f32) as i32;
        let y = (npc.y + env.rng.next_max(ch as i32 - 32) as f32) as i32;
        let ty = match env.rng.next_max(3) {
            1 => 659,
            2 => 660,
            _ => 658,
        };
        out.minions.push(QueenMinionSpawn {
            npc_type: ty,
            x,
            y,
            vx: env.rng.next_range(-15, 16) as f32 * 0.1,
            vy: env.rng.next_range(-30, 1) as f32 * 0.1,
            ai0: (-500 * env.rng.next_max(3)) as f32,
        });
    }
    out
}

/// `NPC.FindFrame` case 657 (`NPC.cs:66848-66959`) — the frame machine
/// (the style73 convention). `frame_row` is `frame.Y / num`; `phase2`
/// is `life <= lifeMax/2`; the noGravity flag rides the env. Returns
/// `(row, counter)`.
pub fn queen_find_frame(
    frame_row: u32,
    frame_counter: f32,
    phase2: bool,
    no_gravity: bool,
    vy: f32,
    ai0: f32,
    ai1: f32,
) -> (u32, f32) {
    let mut row = frame_row as i32;
    let mut counter = frame_counter;
    if (phase2 && no_gravity) || vy < 0.0 {
        // `:66853-66882` — the rising/phase-2 cycles.
        if !(20..=23).contains(&row) {
            if !(4..=7).contains(&row) {
                row = 4;
                counter = -1.0;
            }
            counter += 1.0;
            if counter >= 4.0 {
                counter = 0.0;
                row += 1;
                if row >= 7 {
                    row = if !phase2 { 7 } else { 22 };
                }
            }
        } else {
            counter += 1.0;
            if counter >= 5.0 {
                counter = 0.0;
                row += 1;
                if row >= 24 {
                    row = 20;
                }
            }
        }
    } else if vy > 0.0 {
        // `:66883-66900` — the falling cycle.
        if !(8..=10).contains(&row) {
            row = 8;
            counter = -1.0;
        }
        counter += 1.0;
        if counter >= 8.0 {
            counter = 0.0;
            row += 1;
            if row >= 10 {
                row = 10;
            }
        }
    } else {
        // `:66901-66958` — the grounded arm.
        if vy != 0.0 {
            return (row as u32, counter);
        }
        if ai0 == 5.0 {
            counter = 0.0;
            row = match (ai1 as i32 / 3) % 3 {
                1 => 14,
                2 => 15,
                _ => 13,
            };
        } else if ai0 == 4.0 {
            counter = 0.0;
            row = match ai1 as i32 / 15 {
                1 => 11,
                2 | 3 => 10,
                _ => 12,
            };
        } else {
            let in_idle = (10..=12).contains(&row); // flag12
            let mut rate = 10; // num285
            if in_idle {
                rate = 6;
            }
            if !in_idle && row >= 4 {
                row = 0;
                counter = -1.0;
            }
            counter += 1.0;
            if counter >= rate as f32 {
                counter = 0.0;
                row += 1;
                if (!in_idle || row == 13) && row >= 4 {
                    row = 0;
                }
            }
        }
    }
    (row as u32, counter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A 657-shaped NPC (the style73 placeholder convention).
    fn queen(x: f32, y: f32) -> Npc {
        let d = &QUEEN_SLIME;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n.facing = 1;
        n.target = 0;
        n.local_ai[0] = d.life_max as f32; // skip the spawn latch
        n.despawn_timer = 750;
        n.scale = 1.0;
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

    fn menu_ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: true,
            ..spawn_ctx()
        }
    }

    /// `NPC.cs:17310-17325` — the SetDefaults row, field by field.
    #[test]
    fn w4_d33_s121_the_defaults_row() {
        let d = &QUEEN_SLIME;
        assert_eq!((d.width, d.height), (114, 100));
        assert_eq!(d.ai_style, 121);
        assert_eq!((d.damage, d.defense, d.life_max), (60, 26, 18000));
        assert_eq!(d.hit_sound, (3, 1));
        assert_eq!(d.death_sound, (4, 64));
        assert_eq!(d.knock_back_resist, 0.0);
        assert_eq!(d.value, 60000.0);
        assert!(d.lava_immune && d.boss);
        assert_eq!(d.npc_slots, 5.0);
        assert_eq!(QUEEN_SLIME_FRAME_COUNT, 16);
    }

    /// `:45210-45216` — the spawn latch: ai[1]=-100, localAI[0]=lifeMax.
    #[test]
    fn w4_d33_s121_the_spawn_latch() {
        let w = air();
        let mut pool = DustPool::new();
        let mut gore = GorePool::new();
        let c = menu_ctx();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(121);
        let mut n = queen(100.0, 100.0);
        n.local_ai[0] = 0.0;
        let mut env = Style121Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            gore: &mut gore,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            get_good_world: false,
            no_tile_collide: false,
            no_gravity: false,
            ticker: 7,
        };
        let _ = style121_step(&mut n, &mut env);
        assert_eq!(n.ai[1], -100.0);
        assert_eq!(n.local_ai[0], 18000.0);
        assert!(n.net_update);
    }

    /// `:45336-45382` — the phase-1 attack pick: Next(3) → 3/4/5.
    #[test]
    fn w4_d33_s121_the_phase1_attack_pick() {
        let w = air();
        let mut pool = DustPool::new();
        let mut gore = GorePool::new();
        let c = menu_ctx();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(121);
        let mut oracle = UnifiedRandom::new(121);
        let mut n = queen(100.0, 100.0);
        n.ai[0] = 0.0;
        n.ai[1] = 60.0; // ++ → 61 > 60
        n.vy = 0.0;
        let mut env = Style121Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            gore: &mut gore,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            get_good_world: false,
            no_tile_collide: false,
            no_gravity: false,
            ticker: 7,
        };
        let _ = style121_step(&mut n, &mut env);
        let pick = oracle.next_max(3);
        let want = match pick {
            1 => 4.0,
            2 => 5.0,
            _ => 3.0,
        };
        assert_eq!(n.ai[0], want);
        assert_eq!(n.ai[1], 0.0);
        // The roar draw follows (life > 0).
        oracle.next_max(360);
        assert_eq!(rng.peek(), oracle.peek(), "the pick + roar order");
    }

    /// `:45691-45695` — the gel ring: the 10/15 (6 phase-1) type-926
    /// shots at 30 damage, the rotated (9,0) velocities.
    #[test]
    fn w4_d33_s121_the_gel_ring() {
        let w = air();
        let mut pool = DustPool::new();
        let mut gore = GorePool::new();
        let c = menu_ctx();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(121);
        let mut n = queen(100.0, 100.0);
        n.ai[0] = 5.0;
        n.ai[2] = 1.0;
        n.ai[1] = 9.0; // ++ → 10
        let mut env = Style121Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            gore: &mut gore,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            get_good_world: false,
            no_tile_collide: false,
            no_gravity: false,
            ticker: 7,
        };
        let out = style121_step(&mut n, &mut env);
        // Phase 1 (full life) → 6 shots.
        assert_eq!(out.shots.len(), 6);
        assert!(out
            .shots
            .iter()
            .all(|s| s.projectile_type == 926 && s.damage == 30));
        // The first shot is (9, 0) (i=0, no rotation).
        assert!((out.shots[0].vx - 9.0).abs() < 1e-4);
        assert!(out.shots[0].vy.abs() < 1e-4);
        assert_eq!(n.ai[0], 0.0);
    }

    /// `:45552-45562` — the smash slam: the 922 shot at 40 damage, the
    /// Item167 cue, the state-0 reset.
    #[test]
    fn w4_d33_s121_the_smash_slam() {
        let w = air();
        let mut pool = DustPool::new();
        let mut gore = GorePool::new();
        let c = menu_ctx();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(121);
        let mut n = queen(100.0, 100.0);
        n.ai[0] = 4.0;
        n.ai[2] = 1.0;
        n.vy = 0.0;
        let mut env = Style121Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            gore: &mut gore,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            get_good_world: false,
            no_tile_collide: false,
            no_gravity: false,
            ticker: 7,
        };
        let out = style121_step(&mut n, &mut env);
        assert!(out
            .shots
            .iter()
            .any(|s| s.projectile_type == 922 && s.damage == 40));
        assert!(out.sounds.iter().any(|&(t, s, _, _)| (t, s) == (2, 167)));
        assert_eq!(n.ai[0], 0.0);
    }

    /// `:45121-45124` — the dust colour: TWO NextFloat (inner then
    /// outer).
    #[test]
    fn w4_d33_s121_the_dust_color_draws() {
        let mut rng = UnifiedRandom::new(121);
        let mut oracle = UnifiedRandom::new(121);
        let _ = queen_dust_color(&mut rng);
        oracle.next_float(); // inner
        oracle.next_float(); // outer
        assert_eq!(rng.peek(), oracle.peek(), "the two-float colour");
    }

    /// `:45765-45772` — the phase-transition latch at half life.
    #[test]
    fn w4_d33_s121_the_phase_transition() {
        let w = air();
        let mut pool = DustPool::new();
        let mut gore = GorePool::new();
        let c = menu_ctx();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(121);
        let mut n = queen(100.0, 100.0);
        n.hp = 8999; // just below half
        n.local_ai[0] = 18000.0; // >= half
        n.ai[0] = 3.0;
        let mut env = Style121Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            gore: &mut gore,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            get_good_world: false,
            no_tile_collide: false,
            no_gravity: false,
            ticker: 7,
        };
        let _ = style121_step(&mut n, &mut env);
        assert_eq!(n.ai[0], 0.0, "the phase-2 reset");
        assert_eq!(n.local_ai[0], 8999.0);
        assert!(n.net_update);
    }

    /// `NPC.cs:66848-66959` — the frame machine: the rising cycle, the
    /// falling cycle, the grounded state maps.
    #[test]
    fn w4_d33_s121_the_find_frame() {
        // Rising (vy<0), row out of band → 4, counter -1 then ++ → 0.
        let (row, counter) = queen_find_frame(0, 0.0, false, false, -1.0, 0.0, 0.0);
        assert_eq!(row, 4);
        assert_eq!(counter, 0.0);
        // Falling (vy>0), row out of band → 8.
        let (row, _) = queen_find_frame(0, 0.0, false, false, 1.0, 0.0, 0.0);
        assert_eq!(row, 8);
        // Grounded state 5: the ai[1]/3%3 map.
        assert_eq!(queen_find_frame(0, 0.0, false, false, 0.0, 5.0, 3.0).0, 14);
        assert_eq!(queen_find_frame(0, 0.0, false, false, 0.0, 5.0, 6.0).0, 15);
        assert_eq!(queen_find_frame(0, 0.0, false, false, 0.0, 5.0, 0.0).0, 13);
        // Grounded state 4: the ai[1]/15 map.
        assert_eq!(queen_find_frame(0, 0.0, false, false, 0.0, 4.0, 15.0).0, 11);
        assert_eq!(queen_find_frame(0, 0.0, false, false, 0.0, 4.0, 30.0).0, 10);
        assert_eq!(queen_find_frame(0, 0.0, false, false, 0.0, 4.0, 0.0).0, 12);
    }

    /// The full-pool stream parity: 40 ticks on the REAL dust ctx must
    /// match a cloned twin bit-for-bit (the style101 twin precedent).
    #[test]
    fn w4_d33_s121_the_full_pool_run_is_twin_stable() {
        let w = air();
        let c = spawn_ctx();
        let players = [cand(600.0, 300.0)];
        let mut a = queen(280.0, 280.0);
        let mut b = a.clone();
        let mut pool_a = DustPool::new();
        let mut pool_b = DustPool::new();
        let mut gore_a = GorePool::new();
        let mut gore_b = GorePool::new();
        let mut rng_a = UnifiedRandom::new(657);
        let mut rng_b = UnifiedRandom::new(657);
        for tick in 0..40 {
            let mut env_a = Style121Env {
                world: &w,
                dust: &mut pool_a,
                dust_ctx: &c,
                gore: &mut gore_a,
                rng: &mut rng_a,
                players: &players,
                net_mode: 0,
                get_good_world: false,
                no_tile_collide: false,
                no_gravity: false,
                ticker: 7,
            };
            let _ = style121_step(&mut a, &mut env_a);
            let mut env_b = Style121Env {
                world: &w,
                dust: &mut pool_b,
                dust_ctx: &c,
                gore: &mut gore_b,
                rng: &mut rng_b,
                players: &players,
                net_mode: 0,
                get_good_world: false,
                no_tile_collide: false,
                no_gravity: false,
                ticker: 7,
            };
            let _ = style121_step(&mut b, &mut env_b);
            assert_eq!(rng_a.peek(), rng_b.peek(), "stream @{tick}");
            assert_eq!(a.vx.to_bits(), b.vx.to_bits(), "vx @{tick}");
            assert_eq!(a.ai[0].to_bits(), b.ai[0].to_bits(), "state @{tick}");
        }
    }
}
