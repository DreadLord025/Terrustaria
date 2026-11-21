//! W4-d24 (aiStyle-22) — the ghost/hover family, `NPC.AI`'s
//! `else if (this.aiStyle == 22)` block (`NPC.cs:24666-25234`), ported
//! 1:1 as a PURE machine for the EIGHT unported members (code-only mode:
//! no compiler/tests were run on this branch; the witness tests below
//! are written, not executed).
//!
//! W4-d103 WG-POST RETIRE: verbatim copy of the g8 branch's d24 module
//! (`origin/task/W4-g8-npc-kind-wire-1`, the `94a6d4954` port + the
//! `82c93cec` batch wiring). On THIS branch the Wraith 82 is now a
//! `NpcKind` and the aiStyle axis' 22 arm routes it here (the SmashAltar
//! wraith-spawn activation); the other SEVEN members stay UNWIRED until
//! the g8 merge (their kinds + axis arms are the NPC lane's).
//!
//! ## Family census (`SetDefaults` `aiStyle = 22` rows, whole-file grep)
//!
//! TEN vanilla members. TWO are already ported and stay VERIFY-ONLY in
//! their own modules (this module does NOT re-port them):
//! * Poltergeist 330 (`NPC.cs:13188-13203`) → `npc_ai_poltergeist.rs`;
//! * Drippler 490 (`:14618-14631`) → `npc_ai_drippler.rs`.
//! The EIGHT this module ports (none has a `NpcKind` yet — UNWIRED, the
//! `npc_ai_style17.rs` precedent):
//! * Pixie 75 (`:9772-9785`: 20×20, dmg 55, def 20, 150 HP, kb 0.6,
//!   NPCHit5/NPCDeath7, value 350, noGravity);
//! * Wraith 82 (`:9852-9866`: 24×44, dmg 65, def 16, 160 HP, kb 0.7,
//!   NPCHit54/NPCDeath52, alpha 100, value 500, noGravity+noTileCollide);
//! * Gastropod 122 (`:10479-10492`: 20×20, dmg 60, def 22, 220 HP,
//!   kb 0.8, NPCHit1/NPCDeath1, value 600, noGravity);
//! * IceElemental 169 (`:11113-11127`: 18×40, dmg 55, def 20, 200 HP,
//!   kb 0.6, NPCHit5/NPCDeath7, value 1500, coldDamage, noGravity);
//! * FloatyGross 182 (`:11316-11331`: 24×62, dmg 65, def 18, 240 HP,
//!   kb 0.7, NPCHit1/NPCDeath2, alpha 100, value 500,
//!   noGravity+noTileCollide);
//! * Reaper 253 (`:15450-15465`: 24×44, dmg 80, def 22, 700 HP, kb 0.6,
//!   NPCHit49/NPCDeath51, alpha 100, value 1500, noGravity+noTileCollide);
//! * IchorSticker 268 (`:12470-12483`: 28×56, dmg 55, def 20, 340 HP,
//!   kb 0.6, NPCHit13/NPCDeath19, value 450, noGravity);
//! * Ghost 316 (`:12991-13006`: 24×44, dmg 15, def 4, 50 HP, kb 0.5,
//!   NPCHit1/NPCDeath6, alpha 100, value 90, noGravity+noTileCollide).
//!
//! ## The machine (the shared structure, verbatim)
//!
//! * the flee gates (`:24668-24681`): 330 outside the Pumpkin Moon /
//!   253 outside the Eclipse / 490 by day arm `flag31`.
//! * `justHit` zeroes `ai[2]` (`:24682-24685`).
//! * the Ghost 316 far-player arm (`:24686-24713`): the `ai[3]==1`
//!   fade (the one-shot NPCDeath55 cry, `Opacity -= 0.025`, the
//!   `active = false` at 0) vs the `EncourageDespawn(10)` retreat.
//! * the dwell/dash state (`:24714-24788`): the flee drift (the ONE
//!   `Next(-1,2)*1.5` when parked), the ±16/±40 anchor window with the
//!   60-tick flip into the −200 dash, the 253 +2 / 330 +0.1 / generic
//!   +1 dash creep and the face-away turn.
//! * the per-type offence (`:24789-24957`): the floor-probe column
//!   `num292/num293`, `num294` (3 default; 122→8, 75→4, 169→10,
//!   268→6/12 by the player's Y, 490→4+dist/70), `flag36` the
//!   firing-distance rect; the 122 PinkLaser 84 (the RotatedByRandom
//!   π/40 spin), the 75 sparkle dust + chirp, the 169 FrostBlast 128 +
//!   the aim rotation, the 268 GoldenShower 288 (the confused gate, the
//!   60+Next(60) period, the two jitters).
//! * the shared movement tails (`:24958-25233`): the 490 distance
//!   probe, the below-player floor scan (330 skips), the no-aggro
//!   directionY re-pick, the 169/268 roof scan (flag30), the gravity
//!   ladder (75/169 0.2/2, 490 0.03/0.75, generic 0.1/3 with the 316
//!   flee −0.05/6), the rise brakes, the 75 wet lift, the collideX/Y
//!   oldVelocity bounces (−0.4/−0.25 with the ±1 clamps), the num320
//!   X cap ladder (2 / 75→3 / 253→4 / 490→1.5 / 330→4 + the alpha-0 +
//!   re-aim/despawn + the 0.9 opposite damp) and the ±0.1/0.05 X + the
//!   1.5/1 Y ±0.04/0.05/0.03 approach tails.
//!
//! Seam: writes velocity, `ai[0..3]`, `localAI[1]`/`localAI[3]`,
//! `direction`/`directionY`, `spriteDirection`, `rotation`, `alpha`,
//! `collideX` (the dash clear), `netUpdate`; the Ghost `Opacity` rides
//! the caller's `&mut f32` (no `Npc` field yet). Does NOT integrate.
//! Shots → [`Ai22Effects::shots`] (`HostileShotAi`); sounds →
//! `sounds` (`(style, x, y)`); lights → `lights`; the Ghost
//! deactivation → `deactivate` (the `active = false` → hp-0 convention);
//! `EncourageDespawn(10)` → `encourage_despawn`. The dust runs IN-STEP
//! on the caller's pool + stream (the `npc_ai_style75` convention).
//! `Main.netMode != 1` folds true; `npcTypeNoAggro` rides the candidate.
//!
//! ## RNG census (in C# order, per tick)
//!
//! 1. the flee drift `Next(-1, 2)` (`:24718`) — only flag31 && vx==0;
//! 2. the 122 shot `RotatedByRandom(π/40)` = TWO `NextDouble`
//!    (`:24818`, `Utils.cs:1697-1700`) — only the ai[3]==32 shot tick;
//! 3. the 75 sparkle `Next(6)` (`:24848`) then the chirp `Next(40)`
//!    (`:24853`) — EVERY 75 tick, both drawn (the dust pool's internal
//!    draws ride between them when the 6-gate passes);
//! 4. the 169 frost `Next(3)` (`:24864`) — EVERY 169 tick (+ pool draws);
//! 5. the 268 period `Next(60)` (`:24938`) — EVERY non-confused 268 tick
//!    (drawn even when the gate fails); on a fire, `Next(-10,11)` then
//!    `Next(-30,21)` (`:24948-24949`) BEFORE the normalize.
//! `TargetClosest`, `AI_GlobalFiringDistanceCheck`, `Collision.CanHit`
//! draw nothing. The witnesses mirror each on a cloned stream.
#![allow(dead_code)] // the seven other members stay UNWIRED until the g8 merge (the Wraith 82 rides the axis' 22 arm on this branch)

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

use crate::npc::{HostileShotAi, Npc};
use crate::npc_target::{target_closest, TargetCandidate};

/// One member's `SetDefaults` row (the census above).
pub struct GhostDefaults {
    pub ty: i32,
    pub width: i32,
    pub height: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    pub knock_back_resist: f32,
    pub hit_sound: (i32, i32),
    pub death_sound: (i32, i32),
    pub alpha: i32,
    pub value: f32,
    pub no_tile_collide: bool,
}

/// The eight rows, verbatim (all noGravity; the citations in the header).
pub const GHOST_ROWS: [GhostDefaults; 8] = [
    GhostDefaults { ty: 75, width: 20, height: 20, damage: 55, defense: 20, life_max: 150, knock_back_resist: 0.6, hit_sound: (3, 5), death_sound: (4, 7), alpha: 0, value: 350.0, no_tile_collide: false },
    GhostDefaults { ty: 82, width: 24, height: 44, damage: 65, defense: 16, life_max: 160, knock_back_resist: 0.7, hit_sound: (3, 54), death_sound: (4, 52), alpha: 100, value: 500.0, no_tile_collide: true },
    GhostDefaults { ty: 122, width: 20, height: 20, damage: 60, defense: 22, life_max: 220, knock_back_resist: 0.8, hit_sound: (3, 1), death_sound: (4, 1), alpha: 0, value: 600.0, no_tile_collide: false },
    GhostDefaults { ty: 169, width: 18, height: 40, damage: 55, defense: 20, life_max: 200, knock_back_resist: 0.6, hit_sound: (3, 5), death_sound: (4, 7), alpha: 0, value: 1500.0, no_tile_collide: false },
    GhostDefaults { ty: 182, width: 24, height: 62, damage: 65, defense: 18, life_max: 240, knock_back_resist: 0.7, hit_sound: (3, 1), death_sound: (4, 2), alpha: 100, value: 500.0, no_tile_collide: true },
    GhostDefaults { ty: 253, width: 24, height: 44, damage: 80, defense: 22, life_max: 700, knock_back_resist: 0.6, hit_sound: (3, 49), death_sound: (4, 51), alpha: 100, value: 1500.0, no_tile_collide: true },
    GhostDefaults { ty: 268, width: 28, height: 56, damage: 55, defense: 20, life_max: 340, knock_back_resist: 0.6, hit_sound: (3, 13), death_sound: (4, 19), alpha: 0, value: 450.0, no_tile_collide: false },
    GhostDefaults { ty: 316, width: 24, height: 44, damage: 15, defense: 4, life_max: 50, knock_back_resist: 0.5, hit_sound: (3, 1), death_sound: (4, 6), alpha: 100, value: 90.0, no_tile_collide: true },
];

/// The per-tick context (the `npc_ai_style16.rs::Ai16Ctx` shape).
pub struct Ai22Ctx<'a> {
    pub players: &'a [TargetCandidate],
    /// `NPC.justHit` — the one-tick strike flag (`Npc::just_hit`).
    pub just_hit: bool,
    /// `NPC.confused` — the 268 shot gate (`:24930`); the port folds the
    /// Confused (31) buff (the `npc.rs` Gangsta convention).
    pub confused: bool,
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp.
    pub ticker: u64,
}

/// The side effects beyond the NPC state.
#[derive(Clone, Debug, Default)]
pub struct Ai22Effects {
    pub shots: Vec<HostileShotAi>,
    /// `(style, x, y)` — the 316 NPCDeath55 (55) / the 75 chirp (27).
    pub sounds: Vec<(i32, f32, f32)>,
    /// `(tileX, tileY, [r,g,b])` — the 122 / 169 glows.
    pub lights: Vec<(i32, i32, [f32; 3])>,
    /// `EncourageDespawn(10)` (`:24708`/`:25158`).
    pub encourage_despawn: Option<i32>,
    /// The Ghost's `active = false` (`:24699`) — the hp-0 convention.
    pub deactivate: bool,
}

fn target_of<'a>(npc: &Npc, players: &'a [TargetCandidate]) -> &'a TargetCandidate {
    let i = if npc.target >= 0 && (npc.target as usize) < players.len() {
        npc.target as usize
    } else {
        0
    };
    &players[i]
}

/// `Utils.RotatedByRandom(maxRadians)` (`Utils.cs:1697-1700`): TWO
/// `NextDouble`, left operand first (the `bow_launch.rs` transcription).
fn rotated_by_random(v: Vec2, max_radians: f64, rng: &mut UnifiedRandom) -> Vec2 {
    let angle = rng.next_double() * max_radians - rng.next_double() * max_radians;
    let (s, c) = (angle.sin() as f32, angle.cos() as f32);
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

/// `Utils.ToDirectionInt` (`Utils.cs:2010-2017`): bool → ±1.
fn to_direction_int(b: bool) -> i32 {
    if b { 1 } else { -1 }
}

/// The aiStyle-22 arm (`NPC.cs:24666-25234`) for one of the EIGHT
/// unported types, one tick. `opacity` is the Ghost's `NPC.Opacity`
/// (the caller owns the field until the kind lands). The dust runs
/// in-step on the caller's pool + stream.
pub fn ai_022_arm(
    ty: i32,
    npc: &mut Npc,
    world: &World,
    ctx: &Ai22Ctx,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    opacity: &mut f32,
) -> Ai22Effects {
    let mut fx = Ai22Effects::default();
    let (w, h) = npc.size();
    let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
    // `:24668-24681` — the flee gates (330/490 are the other modules').
    let mut flag30 = false;
    let mut flag31 = false;
    if ty == 253 && !world.eclipse {
        flag31 = true;
    }
    // `:24682-24685` — justHit zeroes the dwell/dash counter.
    if ctx.just_hit {
        npc.ai[2] = 0.0;
    }
    // `:24686-24713` — the Ghost 316 far-player arm.
    if ty == 316 {
        let p = target_of(npc, ctx.players);
        let (pcx, pcy) = (p.x + p.width as f32 * 0.5, p.y + p.height as f32 * 0.5);
        let dist = ((pcx - cx).powi(2) + (pcy - cy).powi(2)).sqrt();
        if p.dead || dist > 3000.0 {
            if npc.ai[3] == 1.0 {
                if npc.local_ai[3] == 0.0 {
                    npc.local_ai[3] = 1.0;
                    // `:24693` — NPCDeath55 at the truncated centre.
                    fx.sounds.push((55, cx, cy));
                }
                flag31 = true;
                *opacity -= 0.025;
                if *opacity <= 0.0 {
                    fx.deactivate = true;
                    npc.net_update = true;
                }
            } else {
                target_closest(npc, world, ctx.players, true, 22, ctx.ticker);
                let p = target_of(npc, ctx.players);
                let (pcx, pcy) = (p.x + p.width as f32 * 0.5, p.y + p.height as f32 * 0.5);
                let dist = ((pcx - cx).powi(2) + (pcy - cy).powi(2)).sqrt();
                if p.dead || dist > 3000.0 {
                    fx.encourage_despawn = Some(10);
                    flag30 = true;
                    flag31 = true;
                }
            }
        }
    }
    // `:24714-24788` — the dwell/dash state.
    if flag31 {
        if npc.vx == 0.0 {
            npc.vx = rng.next_range(-1, 2) as f32 * 1.5;
            npc.net_update = true;
        }
    } else if npc.ai[2] >= 0.0 {
        // `:24724-24762` — the ±16/±40 anchor window.
        let mut num291 = 16.0;
        let mut flag32 = false;
        let mut flag33 = false;
        if npc.x > npc.ai[0] - num291 && npc.x < npc.ai[0] + num291 {
            flag32 = true;
        } else if (npc.vx < 0.0 && npc.facing > 0) || (npc.vx > 0.0 && npc.facing < 0) {
            flag32 = true;
        }
        num291 += 24.0;
        if npc.y > npc.ai[1] - num291 && npc.y < npc.ai[1] + num291 {
            flag33 = true;
        }
        if flag32 && flag33 {
            npc.ai[2] += 1.0;
            // `:24743-24746` — dead C# (num291 is 40 here); carried.
            if npc.ai[2] >= 30.0 && num291 == 16.0 {
                flag30 = true;
            }
            if npc.ai[2] >= 60.0 {
                npc.ai[2] = -200.0;
                npc.facing *= -1;
                npc.vx *= -1.0;
                npc.collide_x = false;
                npc.net_update = true;
            }
        } else {
            npc.ai[0] = npc.x;
            npc.ai[1] = npc.y;
            npc.ai[2] = 0.0;
            npc.net_update = true;
        }
        target_closest(npc, world, ctx.players, true, 22, ctx.ticker);
    } else if ty == 253 {
        // `:24765-24769` — the Reaper's +2 dash creep.
        target_closest(npc, world, ctx.players, true, 22, ctx.ticker);
        npc.ai[2] += 2.0;
    } else {
        // `:24770-24788` — the generic +1 creep (330's +0.1 is the
        // other module's) + the face-away turn.
        npc.ai[2] += 1.0;
        let p = target_of(npc, ctx.players);
        let pcx = p.x + p.width as f32 * 0.5;
        npc.facing = if pcx > cx { -1 } else { 1 };
    }
    // `:24789-24795` — the floor-probe column + the firing-distance rect.
    let num292 = (cx / 16.0) as i32 + npc.facing * 2;
    let num293 = ((npc.y + h) / 16.0) as i32;
    let mut flag34 = true;
    let mut flag35 = false;
    let mut num294 = 3;
    let p6 = target_of(npc, ctx.players);
    let (p6cx, p6cy) = (p6.x + p6.width as f32 * 0.5, p6.y + p6.height as f32 * 0.5);
    let flag36 = crate::npc_ai_style14::global_firing_distance_check(cx, cy, p6cx, p6cy);
    // `:24796-24843` — the Gastropod 122.
    if ty == 122 {
        // `:24801` — the glow at the truncated centre (netOffset ride).
        let lx = (cx + npc.net_offset.x) as i32 / 16;
        let ly = (cy + npc.net_offset.y) as i32 / 16;
        fx.lights.push((lx, ly, [0.4, 0.0, 0.25]));
        if ctx.just_hit {
            npc.ai[3] = 0.0;
            npc.local_ai[1] = 0.0;
        }
        if npc.ai[3] == 32.0 && !p6.npc_type_no_aggro {
            let num295 = 7.0f32;
            let (vx28, vy28) = (npc.x + w * 0.5, npc.y + h * 0.5);
            let num296 = p6.x + (p6.width / 2) as f32 - vx28;
            let num297 = p6.y + (p6.height / 2) as f32 - vy28;
            let num298 = num295 / (num296 * num296 + num297 * num297).sqrt();
            let v29 = rotated_by_random(
                Vec2::new(num296 * num298, num297 * num298),
                0.07853981852531433,
                rng,
            );
            fx.shots.push(HostileShotAi {
                kind: 84, x: vx28, y: vy28, vx: v29.x, vy: v29.y,
                damage: 25, ai0: 0.0, ai1: 0.0, scale: None,
            });
        }
        num294 = 8;
        if npc.ai[3] > 0.0 {
            npc.ai[3] += 1.0;
            if npc.ai[3] >= 64.0 {
                npc.ai[3] = 0.0;
            }
        }
        if npc.ai[3] == 0.0 {
            npc.local_ai[1] += 1.0;
            if npc.local_ai[1] > 120.0 {
                npc.local_ai[1] = 0.0;
                if flag36
                    && crate::projectile::can_hit(
                        world, npc.x, npc.y, w, h, p6.x, p6.y, p6.width as f32, p6.height as f32,
                    )
                    && !p6.npc_type_no_aggro
                {
                    npc.ai[3] = 1.0;
                    npc.net_update = true;
                }
            }
        }
    } else if ty == 75 {
        // `:24844-24858` — the Pixie sparkle + chirp.
        num294 = 4;
        let pos = Vec2::new(npc.x + npc.net_offset.x, npc.y + npc.net_offset.y);
        if rng.next_max(6) == 0 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 55,
                0.0, 0.0, 200, Color::new(0, 0, 0, 0), 1.0,
            );
            dust.slot_mut(idx).velocity *= 0.3;
        }
        if rng.next_max(40) == 0 {
            fx.sounds.push((27, pos.x, pos.y));
        }
    } else if ty == 169 {
        // `:24859-24925` — the Ice Elemental.
        let pos = Vec2::new(npc.x + npc.net_offset.x, npc.y + npc.net_offset.y);
        let lx = (npc.x + npc.net_offset.x + (w as i32 / 2) as f32) as i32 / 16;
        let ly = (npc.y + npc.net_offset.y + (h as i32 / 2) as f32) as i32 / 16;
        fx.lights.push((lx, ly, [0.0, 0.6, 0.75]));
        npc.alpha = 30;
        if rng.next_max(3) == 0 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 92,
                0.0, 0.0, 200, Color::default(), 1.0,
            );
            dust.slot_mut(idx).velocity *= 0.3;
            dust.slot_mut(idx).no_gravity = true;
        }
        if ctx.just_hit {
            npc.ai[3] = 0.0;
            npc.local_ai[1] = 0.0;
        }
        let num303 = 5.0f32;
        let (vx30, vy30) = (npc.x + w * 0.5, npc.y + h * 0.5);
        let mut num304 = p6.x + (p6.width / 2) as f32 - vx30;
        let mut num305 = p6.y + (p6.height / 2) as f32 - vy30;
        let num306 = num303 / (num304 * num304 + num305 * num305).sqrt();
        num304 *= num306;
        num305 *= num306;
        npc.facing = if num304 > 0.0 { 1 } else { -1 };
        npc.sprite_dir = npc.facing;
        npc.rotation = if npc.facing < 0 {
            (-num305).atan2(-num304)
        } else {
            num305.atan2(num304)
        };
        if npc.ai[3] == 16.0 {
            fx.shots.push(HostileShotAi {
                kind: 128, x: vx30, y: vy30, vx: num304, vy: num305,
                damage: 45, ai0: 0.0, ai1: 0.0, scale: None,
            });
        }
        num294 = 10;
        if npc.ai[3] > 0.0 {
            npc.ai[3] += 1.0;
            if npc.ai[3] >= 64.0 {
                npc.ai[3] = 0.0;
            }
        }
        if npc.ai[3] == 0.0 {
            npc.local_ai[1] += 1.0;
            if npc.local_ai[1] > 120.0
                && crate::projectile::can_hit(
                    world, npc.x, npc.y, w, h, p6.x, p6.y, p6.width as f32, p6.height as f32,
                )
            {
                npc.local_ai[1] = 0.0;
                npc.ai[3] = 1.0;
                npc.net_update = true;
            }
        }
    } else if ty == 268 {
        // `:24926-24957` — the Ichor Sticker.
        npc.rotation = npc.vx * 0.1;
        num294 = if !(p6cy < cy) { 6 } else { 12 };
        if !ctx.confused {
            npc.ai[3] += 1.0;
            if ctx.just_hit {
                npc.ai[3] = -45.0;
                npc.local_ai[1] = 0.0;
            }
            // `:24938` — the period roll is drawn EVERY tick.
            if npc.ai[3] >= (60 + rng.next_max(60)) as f32 {
                npc.ai[3] = 0.0;
                if flag36
                    && crate::projectile::can_hit(
                        world, npc.x, npc.y, w, h, p6.x, p6.y, p6.width as f32, p6.height as f32,
                    )
                {
                    let num309 = 10.0f32;
                    let (vx31, vy31) = (npc.x + w * 0.5 - 4.0, npc.y + h * 0.7);
                    let mut num310 = p6.x + (p6.width / 2) as f32 - vx31;
                    let num311 = num310.abs() * 0.1;
                    let mut num312 = p6.y + (p6.height / 2) as f32 - vy31 - num311;
                    num310 += rng.next_range(-10, 11) as f32;
                    num312 += rng.next_range(-30, 21) as f32;
                    let num313 = num309 / (num310 * num310 + num312 * num312).sqrt();
                    fx.shots.push(HostileShotAi {
                        kind: 288, x: vx31, y: vy31, vx: num310 * num313, vy: num312 * num313,
                        damage: 40, ai0: 0.0, ai1: 0.0, scale: None,
                    });
                }
            }
        }
    }
    // `:24971-24996` — the below-player floor scan (330 skips).
    if npc.y + h > p6.y {
        for num317 in num293..num293 + num294 {
            if let Some(t) = world.tile(num292, num317) {
                let solid = t.active() && !t.inactive() && tr_core::tiledata::TILE_SOLID.has(t.kind);
                if solid || t.liquid > 0 {
                    if num317 <= num293 + 1 {
                        flag35 = true;
                    }
                    flag34 = false;
                    break;
                }
            }
        }
    }
    // `:24997-25013` — the no-aggro directionY re-pick.
    if p6.npc_type_no_aggro {
        let mut flag37 = false;
        for num318 in num293..num293 + num294 - 2 {
            if let Some(t) = world.tile(num292, num318) {
                let solid = t.active() && !t.inactive() && tr_core::tiledata::TILE_SOLID.has(t.kind);
                if solid || t.liquid > 0 {
                    flag37 = true;
                    break;
                }
            }
        }
        npc.direction_y = to_direction_int(!flag37);
    }
    // `:25014-25029` — the 169/268 roof scan (flag30).
    if ty == 169 || ty == 268 {
        for num319 in num293 - 3..num293 {
            if let Some(t) = world.tile(num292, num319) {
                let solid = t.active()
                    && !t.inactive()
                    && tr_core::tiledata::TILE_SOLID.has(t.kind)
                    && !tr_core::tiledata::SET_PLATFORMS.has(t.kind);
                if solid || t.liquid > 0 {
                    flag35 = false;
                    flag30 = true;
                    break;
                }
            }
        }
    }
    // `:25030-25038` — flag30 forces the fall (268 gets the +2 kick).
    if flag30 {
        flag35 = false;
        flag34 = true;
        if ty == 268 {
            npc.vy += 2.0;
        }
    }
    // `:25039-25102` — the gravity / rise ladder.
    if flag34 {
        if ty == 75 || ty == 169 {
            npc.vy += 0.2;
            if npc.vy > 2.0 {
                npc.vy = 2.0;
            }
        } else {
            npc.vy += 0.1;
            if ty == 316 && flag31 {
                npc.vy -= 0.05;
                if npc.vy > 6.0 {
                    npc.vy = 6.0;
                }
            } else if npc.vy > 3.0 {
                npc.vy = 3.0;
            }
        }
    } else {
        if ty == 75 || ty == 169 {
            if (npc.direction_y < 0 && npc.vy > 0.0) || flag35 {
                npc.vy -= 0.2;
            }
        } else if npc.direction_y < 0 && npc.vy > 0.0 {
            npc.vy -= 0.1;
        }
        if npc.vy < -4.0 {
            npc.vy = -4.0;
        }
    }
    // `:25103-25110` — the Pixie wet lift.
    if ty == 75 && npc.wet {
        npc.vy -= 0.2;
        if npc.vy < -2.0 {
            npc.vy = -2.0;
        }
    }
    // `:25111-25134` — the collideX/Y oldVelocity bounces.
    if npc.collide_x {
        npc.vx = npc.old_vx * -0.4;
        if npc.facing == -1 && npc.vx > 0.0 && npc.vx < 1.0 {
            npc.vx = 1.0;
        }
        if npc.facing == 1 && npc.vx < 0.0 && npc.vx > -1.0 {
            npc.vx = -1.0;
        }
    }
    if npc.collide_y {
        npc.vy = npc.old_vy * -0.25;
        if npc.vy > 0.0 && npc.vy < 1.0 {
            npc.vy = 1.0;
        }
        if npc.vy < 0.0 && npc.vy > -1.0 {
            npc.vy = -1.0;
        }
    }
    // `:25135-25168` — the num320 X cap ladder (330's rows are the other
    // module's; carried here only for the shared structure).
    let mut num320 = 2.0f32;
    if ty == 75 {
        num320 = 3.0;
    }
    if ty == 253 {
        num320 = 4.0;
    }
    // `:25169-25200` — the X approach tail.
    if npc.facing == -1 && npc.vx > -num320 {
        npc.vx -= 0.1;
        if npc.vx > num320 {
            npc.vx -= 0.1;
        } else if npc.vx > 0.0 {
            npc.vx += 0.05;
        }
        if npc.vx < -num320 {
            npc.vx = -num320;
        }
    } else if npc.facing == 1 && npc.vx < num320 {
        npc.vx += 0.1;
        if npc.vx < -num320 {
            npc.vx += 0.1;
        } else if npc.vx < 0.0 {
            npc.vx -= 0.05;
        }
        if npc.vx > num320 {
            npc.vx = num320;
        }
    }
    // `:25201-25233` — the Y approach tail (the 490 cap is 1).
    let num320y = 1.5f32;
    if npc.direction_y == -1 && npc.vy > -num320y {
        npc.vy -= 0.04;
        if npc.vy > num320y {
            npc.vy -= 0.05;
        } else if npc.vy > 0.0 {
            npc.vy += 0.03;
        }
        if npc.vy < -num320y {
            npc.vy = -num320y;
        }
    } else if npc.direction_y == 1 && npc.vy < num320y {
        npc.vy += 0.04;
        if npc.vy < -num320y {
            npc.vy += 0.05;
        } else if npc.vy < 0.0 {
            npc.vy -= 0.03;
        }
        if npc.vy > num320y {
            npc.vy = num320y;
        }
    }
    fx
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN, NOT RUN (code-only mode). Each pins one
    //! C# branch against its citation (`NPC.cs:24666-25234`).
    use super::*;
    use crate::npc::NpcKind;

    fn air() -> World {
        World::new("t".into(), 200, 120)
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

    fn body(ty: i32, x: f32, y: f32) -> Npc {
        let row = GHOST_ROWS.iter().find(|r| r.ty == ty).unwrap();
        let mut n = Npc::new(NpcKind::DemonEye, x, y, 1, 0.0);
        n.size_override = Some((row.width as f32, row.height as f32));
        n.hp = row.life_max;
        n.max_hp = row.life_max;
        n
    }

    fn ctx<'a>(players: &'a [TargetCandidate]) -> Ai22Ctx<'a> {
        Ai22Ctx { players, just_hit: false, confused: false, ticker: 0 }
    }

    fn dust_ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(8000.0, 8000.0),
            max_dust_to_draw: 6000,
        }
    }

    /// The eight `SetDefaults` rows (the census).
    #[test]
    fn the_defaults_rows_match_the_census() {
        let want = [
            (75, 20, 20, 55, 20, 150, 0.6, (3, 5), (4, 7), 350.0),
            (82, 24, 44, 65, 16, 160, 0.7, (3, 54), (4, 52), 500.0),
            (122, 20, 20, 60, 22, 220, 0.8, (3, 1), (4, 1), 600.0),
            (169, 18, 40, 55, 20, 200, 0.6, (3, 5), (4, 7), 1500.0),
            (182, 24, 62, 65, 18, 240, 0.7, (3, 1), (4, 2), 500.0),
            (253, 24, 44, 80, 22, 700, 0.6, (3, 49), (4, 51), 1500.0),
            (268, 28, 56, 55, 20, 340, 0.6, (3, 13), (4, 19), 450.0),
            (316, 24, 44, 15, 4, 50, 0.5, (3, 1), (4, 6), 90.0),
        ];
        for (r, wnt) in GHOST_ROWS.iter().zip(want) {
            assert_eq!((r.ty, r.width, r.height), (wnt.0, wnt.1, wnt.2));
            assert_eq!((r.damage, r.defense, r.life_max), (wnt.3, wnt.4, wnt.5));
            assert_eq!(r.knock_back_resist, wnt.6);
            assert_eq!(r.hit_sound, wnt.7);
            assert_eq!(r.death_sound, wnt.8);
            assert_eq!(r.value, wnt.9);
        }
    }

    /// `:24714-24721` — the Reaper flee (no eclipse): a parked body
    /// draws ONE Next(-1,2)*1.5; a moving body draws nothing.
    #[test]
    fn the_reaper_flee_drift_draws_once() {
        let mut w = air();
        w.eclipse = false;
        let p = [cand(1400.0, 1000.0)];
        let mut n = body(253, 1000.0, 1000.0);
        n.vx = 0.0;
        let mut rng = UnifiedRandom::new(4);
        let mut mirror = UnifiedRandom::new(4);
        let want = mirror.next_range(-1, 2) as f32 * 1.5;
        let mut dust = DustPool::new();
        let mut op = 1.0;
        ai_022_arm(253, &mut n, &w, &ctx(&p), &mut rng, &mut dust, &dust_ctx(), &mut op);
        assert_eq!(n.vx, want);
        assert_eq!(rng, mirror, "exactly the one flee draw");
        // A moving body: no draw.
        let mut m = body(253, 1000.0, 1000.0);
        m.vx = 2.0;
        let mut rng2 = UnifiedRandom::new(4);
        let w2 = rng2.clone();
        ai_022_arm(253, &mut m, &w, &ctx(&p), &mut rng2, &mut dust, &dust_ctx(), &mut op);
        assert_eq!(rng2, w2, "the moving flee draws nothing");
    }

    /// `:24722-24764` — the dwell anchor: inside the window ai[2]
    /// counts; at 60 the dash flips (ai[2]=-200, facing/vx invert).
    #[test]
    fn the_dwell_anchor_flips_into_the_dash() {
        let w = air();
        let p = [cand(1400.0, 1000.0)];
        let mut n = body(82, 1000.0, 1000.0);
        n.ai[0] = 1000.0;
        n.ai[1] = 1000.0;
        n.ai[2] = 59.0;
        n.facing = 1;
        n.vx = 1.0;
        let mut rng = UnifiedRandom::new(4);
        let witness = rng.clone();
        let mut dust = DustPool::new();
        let mut op = 1.0;
        ai_022_arm(82, &mut n, &w, &ctx(&p), &mut rng, &mut dust, &dust_ctx(), &mut op);
        assert_eq!(n.ai[2], -200.0, "the 60-tick dash");
        assert_eq!(n.facing, -1);
        assert!(!n.collide_x);
        assert_eq!(rng, witness, "the dwell/dash draws nothing");
    }

    /// `:24809-24819` — the Gastropod shot: ai[3]==32 fires the PinkLaser
    /// 84 with the RotatedByRandom(π/40) TWO-NextDouble spin.
    #[test]
    fn the_gastropod_shot_spins_with_two_doubles() {
        let w = air();
        let p = [cand(1100.0, 1000.0)];
        let mut n = body(122, 1000.0, 1000.0);
        n.ai[3] = 32.0;
        n.target = 0;
        let mut rng = UnifiedRandom::new(8);
        let mut mirror = UnifiedRandom::new(8);
        let mut dust = DustPool::new();
        let mut op = 1.0;
        let fx = ai_022_arm(122, &mut n, &w, &ctx(&p), &mut rng, &mut dust, &dust_ctx(), &mut op);
        assert_eq!(fx.shots.len(), 1);
        assert_eq!(fx.shots[0].kind, 84);
        assert_eq!(fx.shots[0].damage, 25);
        // Mirror the two NextDouble draws.
        let a = mirror.next_double() * 0.07853981852531433
            - mirror.next_double() * 0.07853981852531433;
        let _ = a;
        assert_eq!(rng, mirror, "exactly the two NextDouble spins");
        assert_eq!(fx.lights.len(), 1, "the 122 glow");
    }

    /// `:24848-24856` — the Pixie gates: Next(6) then Next(40) EVERY
    /// tick, in that order (the pool draws ride between on a 6-pass).
    #[test]
    fn the_pixie_gates_draw_in_order() {
        let w = air();
        let p = [cand(1400.0, 1000.0)];
        let mut n = body(75, 1000.0, 1000.0);
        let mut rng = UnifiedRandom::new(12);
        let mut mirror = UnifiedRandom::new(12);
        let mut dust = DustPool::new();
        let mut mdust = DustPool::new();
        let dc = dust_ctx();
        let mut op = 1.0;
        let g6 = mirror.next_max(6);
        if g6 == 0 {
            let pos = Vec2::new(n.x, n.y);
            let idx = mdust.new_dust(&mut mirror, &dc, &w, pos, 20, 20, 55, 0.0, 0.0, 200, Color::new(0, 0, 0, 0), 1.0);
            mdust.slot_mut(idx).velocity *= 0.3;
        }
        let g40 = mirror.next_max(40);
        let fx = ai_022_arm(75, &mut n, &w, &ctx(&p), &mut rng, &mut dust, &dc, &mut op);
        assert_eq!(rng, mirror, "the 6-gate + pool + 40-gate order");
        assert_eq!(fx.sounds.len(), if g40 == 0 { 1 } else { 0 });
    }

    /// `:24938-24953` — the Ichor period: Next(60) EVERY non-confused
    /// tick; on a fire the two jitters draw BEFORE the normalize.
    #[test]
    fn the_ichor_period_and_jitters() {
        let w = air();
        let p = [cand(1100.0, 1000.0)];
        let mut n = body(268, 1000.0, 1000.0);
        n.ai[3] = 200.0; // past any 60+Next(60) threshold
        n.target = 0;
        let mut rng = UnifiedRandom::new(16);
        let mut mirror = UnifiedRandom::new(16);
        let mut dust = DustPool::new();
        let mut op = 1.0;
        let period = mirror.next_max(60);
        let fired = n.ai[3] >= (60 + period) as f32;
        let fx = ai_022_arm(268, &mut n, &w, &ctx(&p), &mut rng, &mut dust, &dust_ctx(), &mut op);
        if fired {
            // The CanHit gate passes in air; the two jitters draw.
            let _ = mirror.next_range(-10, 11);
            let _ = mirror.next_range(-30, 21);
            assert_eq!(fx.shots.len(), 1);
            assert_eq!(fx.shots[0].kind, 288);
        }
        assert_eq!(rng, mirror, "the period + jitter order");
        assert_eq!(n.ai[3], 0.0, "the fire resets the counter");
        // Confused: no draw at all.
        let mut m = body(268, 1000.0, 1000.0);
        m.ai[3] = 200.0;
        let mut rng2 = UnifiedRandom::new(16);
        let w2 = rng2.clone();
        let c = Ai22Ctx { players: &p, just_hit: false, confused: true, ticker: 0 };
        ai_022_arm(268, &mut m, &w, &c, &mut rng2, &mut dust, &dust_ctx(), &mut op);
        assert_eq!(rng2, w2, "the confused Ichor draws nothing");
    }

    /// `:24686-24701` — the Ghost fade: ai[3]==1 + far player → the
    /// one-shot NPCDeath55, the opacity decay, the deactivation at 0.
    #[test]
    fn the_ghost_fade_cries_once_and_deactivates() {
        let w = air();
        let p = [cand(9000.0, 1000.0)]; // > 3000 px away
        let mut n = body(316, 1000.0, 1000.0);
        n.ai[3] = 1.0;
        n.target = 0;
        let mut rng = UnifiedRandom::new(4);
        let witness = rng.clone();
        let mut dust = DustPool::new();
        let mut op = 0.02;
        let fx = ai_022_arm(316, &mut n, &w, &ctx(&p), &mut rng, &mut dust, &dust_ctx(), &mut op);
        assert_eq!(fx.sounds, vec![(55, 1012.0, 1022.0)], "the NPCDeath55 cry");
        assert_eq!(n.local_ai[3], 1.0, "the one-shot latch");
        assert!(op <= 0.0);
        assert!(fx.deactivate, "the opacity-0 deactivation");
        assert_eq!(rng, witness, "the fade draws nothing");
        // The second tick: no repeat cry.
        let fx2 = ai_022_arm(316, &mut n, &w, &ctx(&p), &mut rng, &mut dust, &dust_ctx(), &mut op);
        assert!(fx2.sounds.is_empty(), "the cry is one-shot");
    }

    /// The RNG census: the Wraith 82 / FloatyGross 182 (no per-type
    /// offence draws) consume NOTHING across the dwell/dash/movement.
    #[test]
    fn the_wraith_and_floaty_draw_no_rng() {
        let w = air();
        let p = [cand(1400.0, 1000.0)];
        let mut dust = DustPool::new();
        let mut op = 1.0;
        for ty in [82, 182] {
            let mut n = body(ty, 1000.0, 1000.0);
            let mut rng = UnifiedRandom::new(23);
            let witness = rng.clone();
            for _ in 0..200 {
                ai_022_arm(ty, &mut n, &w, &ctx(&p), &mut rng, &mut dust, &dust_ctx(), &mut op);
            }
            assert_eq!(rng, witness, "type {ty} draws nothing");
        }
    }
}
