//! The aiStyle-111 family — the DD2 Lightning Bug
//! (`NPC.AI_111_DD2LightningBug`, `NPC.cs:66188-66393`, dispatched at
//! `:41064-41067`). W4-d33, research/25 row 111, CODE-ONLY MODE (no
//! compiler invoked; the witness tests below are written, not run).
//!
//! ## Census
//!
//! The family has exactly ONE vanilla member: the DD2LightningBugT3
//! 578 (`NPCID.DD2LightningBugT3`, `ID/NPCID.cs:11833`; `SetDefaults`
//! row `NPC.cs:16536-16553`). The census of `this.aiStyle = 111;`
//! across the decompiled tree returns exactly that one hit (`:16540`)
//! — no projectile-space user. The arm's `type == 576 || type == 577`
//! dust-gate branch (`:66279`) is DEAD for the style's own member (the
//! Ogres ride aiStyle 107) — ported verbatim behind the `npc_type`
//! parameter anyway. The port's roster has no lightning-bug kind — the
//! module lands UNWIRED (the `npc_ai_style73.rs` precedent).
//!
//! ## The machine
//!
//! The portal-spawned zapper: it materializes over 60 ticks, swarms
//! apart from its siblings, hovers at the target and fires jittered
//! bolts on a 5-tick charge / 30-tick rest cycle.
//!
//! The 578 head (`:66209-66257`): the tunables flip to speed 4, the
//! 20/20 approach blends, separation 0.1, bolt 682
//! (`ProjectileID.DD2LightningBugZap` — the C# writes the literal),
//! damage 50, charge 5 / rest 30, bolt speed 10, hover band 50; the
//! muzzle shifts `Center + (-spriteDirection*20, 10)`. Under the
//! netOffset shift: the `Next(15) == 0` trail dust (type 272,
//! `NewDustDirect` at position — X nudged `width/2` when facing -1 —
//! box `(width/2, height)`, scale 0.65, `vy -= 0.8`, the
//! `vx*direction > 0` flip, the `vy > 0` ×-0.5 flip) and the
//! `Next(80) == 0` spark pair (ONE `NextFloat` angle →
//! `ToRotationVector2`, TWO `NewDustPerfect` 272 at Center with the
//! ×1.1 / ×1.5 velocities, noGravity; `customData = this` is the
//! dropped field — the port's Dust has no customData, documented).
//!
//! The materialize (`:66258-66295`, 578-only): `localAI[1] == 0` sets
//! alpha 255; `== 30` plays the tracked portal-spawn cue (the audio
//! seam); `< 60` increments, `alpha -= 5` floored 0, and runs
//! `(int)localAI[1] / 10` dust iterations — each `Next(5) == 0` (1 for
//! the dead 576/577 branch) spawns a type-27 `NewDustDirect` at
//! position, box (width, height), `vx*1`, alpha 100, scale 0.55,
//! fadeIn 0.7, `velocity *= 0.1 * Size.Length()/2/20` then
//! `+= velocity`.
//!
//! The separation (`:66296-66320`, 578-only): every OTHER active
//! same-type NPC within Manhattan `width` pushes ±0.1 per axis (the
//! sign of the position difference; ties push +).
//!
//! The hover (`:66321-66363`): `rotation = |vx| * direction * 0.1`,
//! `spriteDirection = direction`; the aim
//! `SafeNormalize(targetCenter - muzzle, UnitY) * speed`; the
//! `CanHit(Center, 1, 1, targetCenter, 1, 1)` sight gate. The negative
//! `localAI[0]` counts up. `flag` is born TRUE and never cleared — the
//! `!flag` despawn arm (`:66331-66335`, `EncourageDespawn(10)`) is DEAD
//! code, documented. Out of 200 px or unsighted: the 20-blend approach.
//! Inside the 50-px hover band (`aim.Y < 50`): `vy -= 0.03`. Else the
//! charge: `velocity *= 0.96`; below speed 1 on a non-client
//! `localAI[0]++`; at 5 it resets to -30, faces the aim sign, and fires
//! the jittered bolt — `aim + RandomVector2(-25, 25)` (TWO NextDouble),
//! ×`1 + Next(-20,21)*0.005` per axis, `SafeNormalize(UnitY) * 10`,
//! ×`1 + Next(-20,21)*(1/160)` per axis — type 682, damage 50,
//! knockback 0, from the muzzle.
//!
//! The lift tail (`:66368-66392`): the target above OR the centre tile
//! solid (`WorldGen.SolidTile(Tile)` — the strict test, null = solid,
//! the `InWorld(x, y, 2)` gate) → `vy -= 0.2` capped -10.
//!
//! ## The target seam
//!
//! `NPCUtils.TargetClosestOldOnesInvasion` (`NPCUtils.cs:262-278`) is
//! the NEW targeting: `SearchForTarget(All, OnlyPlayersInCertainDistance
//! (Center, 200), OnlyCrystal)` — the nearest player within 200 px
//! (Distance − aggro, the +1000 no-aggro penalty when direction == 0)
//! versus the nearest active 548 crystal with
//! `!dontTakeDamageFromHostiles` (DistanceSquared); the crystal wins
//! ties (`npcDist < adjustedTank` is strict). The winner writes
//! `target` (the NPC index +300) / `targetRect` and faces via
//! `ShouldFaceTarget` (`NPC.cs:77607-77644`) + `FaceTarget`
//! (`:77646-77650`, the INTEGER rect-centre compare). `GetTargetData`
//! (`:6901-6912`) then reads the player or the +300 NPC slot; an
//! invalid target returns `default` — the (0,0) aim. The port models
//! the whole search over the env slices (the tank-pet steal is inert,
//! the `session_candidate` convention).
//!
//! ## RNG order (by construction)
//!
//! Per tick: `Next(15)` (passing: the NewDustDirect internals);
//! `Next(80)` (passing: ONE NextFloat, then TWO NewDustPerfect
//! internals); the materialize loop's `(int)localAI[1]/10` × `Next(5)`
//! (passing: the NewDustDirect internals); the bolt's TWO NextDouble +
//! FOUR Next(-20,21) in X,Y,X,Y order. The search, the separation, the
//! blends and the lift draw NOTHING.
//!
//! ## FindFrame case 578
//!
//! `NPC.cs:67875-67886`: `rotation = vx * 0.1` (the frame pass
//! OVERWRITES the AI's `|vx|*direction*0.1` — both ported, each in its
//! own pass); `++frameCounter >= 4` advances the row, wrapping at
//! `Main.npcFrameCount[578] = 4` (`Main.cs:67141`, the 570-579 row).
//! [`bug_find_frame`] ports the counter pair (the style73 convention).
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::bow_launch::safe_normalize;
use crate::npc::Npc;
use crate::npc_target::TargetCandidate;
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::tiledata::{TILE_SOLID, TILE_SOLID_TOP};
use tr_core::{UnifiedRandom, World};

/// The type-578 `SetDefaults` row (`NPC.cs:16536-16553`), verbatim.
/// The two sounds are TRACKABLE façades (`SoundID.cs:1647-1648`,
/// `CreateTrackable("dd2_lightning_bug_hurt"/"…_death", 3)`) — the
/// names ride the audio seam, not a (type, style) pair.
pub struct LightningBugDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    pub hit_sound: &'static str,
    pub death_sound: &'static str,
    pub knock_back_resist: f32,
    pub no_gravity: bool,
    pub no_tile_collide: bool,
    pub npc_slots: f32,
    pub lava_immune: bool,
    pub net_always: bool,
}

/// `NPC.cs:16536-16553` — `else if (this.type == 578)`.
/// `LazySetLiquidMovementDD2` is the liquid-movement preset the
/// wiring owns (the DD2 family convention).
pub const LIGHTNING_BUG: LightningBugDefaults = LightningBugDefaults {
    width: 30,
    height: 24,
    ai_style: 111,
    damage: 80,
    defense: 36,
    life_max: 500,
    hit_sound: "dd2_lightning_bug_hurt",
    death_sound: "dd2_lightning_bug_death",
    knock_back_resist: 0.8,
    no_gravity: true,
    no_tile_collide: true,
    npc_slots: 0.0,
    lava_immune: true,
    net_always: true,
};

/// `Main.npcFrameCount[578]` = 4 (`Main.cs:67141`, the 570-579 row:
/// `16, 16, 18, 18, 10, 10, 10, 10, 4, 1` — index 578 is the 9th).
pub const LIGHTNING_BUG_FRAME_COUNT: u32 = 4;

/// One `Main.npc[j]` slot as the separation scan and the crystal
/// search see it (`:66300`, `NPCUtils.cs:207-219`).
#[derive(Clone, Copy, Debug)]
pub struct BugNeighbor {
    /// `j != whoAmI` — the self skip, precomputed by the wiring.
    pub is_self: bool,
    pub active: bool,
    /// `Main.npc[j].type`.
    pub ty: i32,
    pub x: f32,
    pub y: f32,
    pub width: i32,
    pub height: i32,
    /// `dontTakeDamageFromHostiles` — the OnlyCrystal filter reads it
    /// (`NPCUtils.cs:15-22`).
    pub dont_take_damage_from_hostiles: bool,
}

/// The per-tick environment (the `Style102Env` shape).
pub struct Style111Env<'a> {
    pub world: &'a World,
    pub dust: &'a mut DustPool,
    pub dust_ctx: &'a SpawnCtx,
    pub rng: &'a mut UnifiedRandom,
    /// The `Main.player[]` slots as the search sees them.
    pub players: &'a [TargetCandidate],
    /// The NPC slots in `Main.npc` order, self included (`is_self`).
    pub neighbors: &'a [BugNeighbor],
    /// `Main.netMode` — the bolt fires on `!= 1` (`:66347`).
    pub net_mode: i32,
}

/// The `Projectile.NewProjectile` request of the charge
/// (`NPC.cs:66360`): type 682 (the C# literal — the DD2 lightning
/// zap), damage 50, knockback 0, owner `Main.myPlayer`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BugBolt {
    pub projectile_type: i32,
    /// The muzzle `vector3` — `Center + (-spriteDirection*20, 10)`.
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style111Outcome {
    /// The charge's bolt (`:66360`), else `None`.
    pub shot: Option<BugBolt>,
    /// `SoundEngine.PlayTrackedSound(SoundID.DD2_EtherianPortalSpawnEnemy,
    /// Center)` at `localAI[1] == 30` (`:66266`) — `(name, x, y)`; the
    /// audio family owns the trackable's draws.
    pub sounds: Vec<(&'static str, f32, f32)>,
}

/// XNA `Vector2.Distance` — the f32 squared sum widened to f64 for the
/// root (the style100 `sqrt_f32` shape).
fn distance(a: Vec2, b: Vec2) -> f32 {
    let d = a - b;
    ((d.x * d.x + d.y * d.y) as f64).sqrt() as f32
}

/// XNA `Vector2.DistanceSquared` — the f32 sum, NO root
/// (`NPCUtils.cs:212`).
fn distance_sq(a: Vec2, b: Vec2) -> f32 {
    let d = a - b;
    d.x * d.x + d.y * d.y
}

/// `Utils.RandomVector2` (`Utils.cs:2431-2434`): TWO NextDouble, X
/// first, the double cast to f32 BEFORE the multiply.
fn random_vector2(rng: &mut UnifiedRandom, min: f32, max: f32) -> Vec2 {
    let x = (max - min) * rng.next_double() as f32 + min;
    let y = (max - min) * rng.next_double() as f32 + min;
    Vec2::new(x, y)
}

/// `WorldGen.SolidTile(Tile)` (`WorldGen.cs:69957-69974`): the strict
/// test — null tile is SOLID (the port's out-of-bounds `None`), but
/// the caller's `tile != null` gate (`:66379`) refuses it first.
fn solid_tile_strict(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && !t.inactive()
        && TILE_SOLID.has(t.kind)
        && !TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
}

/// `NPCUtils.TargetClosestOldOnesInvasion` (`NPCUtils.cs:262-278`)
/// over the env slices: the 200-px player search (Distance − aggro,
/// the +1000 no-aggro penalty at direction 0; the tank-pet steal is
/// inert — `session_candidate` passes None) versus the OnlyCrystal
/// 548 scan (DistanceSquared). Writes `target`/`targetRect` and faces
/// via `ShouldFaceTarget` + `FaceTarget`. RNG-free.
fn target_closest_old_ones(npc: &mut Npc, env: &Style111Env<'_>) {
    let (w, h) = npc.size();
    let center = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
    // `NPCUtils.cs:205-220` — the NPC scan (OnlyCrystal: the active
    // 548 with !dontTakeDamageFromHostiles).
    let mut npc_dist_sq = f32::MAX;
    let mut nearest_npc: i32 = -1;
    for (i, n) in env.neighbors.iter().enumerate() {
        if n.active && !n.is_self && n.ty == 548 && !n.dont_take_damage_from_hostiles {
            let c = Vec2::new(n.x + n.width as f32 / 2.0, n.y + n.height as f32 / 2.0);
            let d = distance_sq(center, c);
            if d < npc_dist_sq {
                nearest_npc = i as i32;
                npc_dist_sq = d;
            }
        }
    }
    // `NPCUtils.cs:221-257` — the player scan (the 200-px filter, the
    // aggro offset, the direction-0 no-aggro penalty).
    let mut tank_adj = f32::MAX;
    let mut tank_dist = f32::MAX;
    let mut nearest_tank: i32 = -1;
    for (j, p) in env.players.iter().enumerate() {
        if !p.active || p.dead || p.ghost {
            continue;
        }
        let c = Vec2::new(p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32);
        let d = distance(center, c);
        if d > 200.0 {
            continue; // OnlyPlayersInCertainDistance (NPCUtils.cs:24-27)
        }
        let mut adj = d - p.aggro as f32;
        if p.npc_type_no_aggro && npc.facing == 0 {
            adj += 1000.0;
        }
        if adj < tank_adj {
            nearest_tank = j as i32;
            tank_adj = adj;
            tank_dist = d;
        }
        // The tank-pet steal (`:244-256`) is inert — the port's
        // candidates carry no pet (the session_candidate convention).
    }
    // `TargetSearchResults` ctor (`NPCUtils.cs:144-175`): the crystal
    // wins only on the STRICT `<`.
    let npc_dist = (npc_dist_sq as f64).sqrt() as f32;
    let (target, rect, is_npc) = if nearest_npc != -1 && nearest_tank != -1 {
        if npc_dist < tank_adj {
            let n = &env.neighbors[nearest_npc as usize];
            (
                nearest_npc + 300,
                (n.x as i32, n.y as i32, n.width, n.height),
                true,
            )
        } else {
            let p = &env.players[nearest_tank as usize];
            (
                nearest_tank,
                (p.x as i32, p.y as i32, p.width, p.height),
                false,
            )
        }
    } else if nearest_npc != -1 {
        let n = &env.neighbors[nearest_npc as usize];
        (
            nearest_npc + 300,
            (n.x as i32, n.y as i32, n.width, n.height),
            true,
        )
    } else if nearest_tank != -1 {
        let p = &env.players[nearest_tank as usize];
        (
            nearest_tank,
            (p.x as i32, p.y as i32, p.width, p.height),
            false,
        )
    } else {
        return; // FoundTarget false — nothing writes.
    };
    npc.target = target;
    npc.target_rect = rect;
    // `ShouldFaceTarget` (`NPC.cs:77607-77644`): the NPC target always
    // faces; the player arm runs the dead/no-aggro gate + the stealth
    // distance rule.
    let face = if is_npc {
        true
    } else {
        let p = &env.players[nearest_tank as usize];
        if p.dead || (p.npc_type_no_aggro && npc.facing != 0) {
            false
        } else {
            let stealth_band = (p.width + p.height + w as i32 + h as i32) as f32 / 4.0 + 800.0;
            if p.item_animation == 0 && p.aggro < 0 && tank_dist > stealth_band {
                if npc.old_target >= 0 {
                    npc.old_target >= 255
                } else {
                    true
                }
            } else {
                true
            }
        }
    };
    if face {
        // `FaceTarget` (`NPC.cs:77646-77650`): the INTEGER rect centre,
        // ties face +1.
        let rcx = (rect.0 + rect.2 / 2) as f32;
        let rcy = (rect.1 + rect.3 / 2) as f32;
        npc.facing = if rcx < center.x { -1 } else { 1 };
        npc.direction_y = if rcy < center.y { -1 } else { 1 };
    }
}

/// `NPC.GetTargetData` (`NPC.cs:6901-6912`) over the env slices: the
/// +300 NPC slot's centre, the player slot's centre, or the `default`
/// (0,0) when invalid (`HasValidTarget`, `:6657-6671` — 578 is in
/// `UsesNewTargeting`, `NPCID.cs:10434`).
fn target_center(npc: &Npc, env: &Style111Env<'_>) -> Vec2 {
    if npc.target >= 300 {
        let i = (npc.target - 300) as usize;
        if let Some(n) = env.neighbors.get(i) {
            if n.active {
                return Vec2::new(n.x + n.width as f32 / 2.0, n.y + n.height as f32 / 2.0);
            }
        }
        return Vec2::ZERO;
    }
    if npc.target >= 0 && (npc.target as usize) < env.players.len() {
        let p = &env.players[npc.target as usize];
        if p.active && !p.dead && !p.ghost {
            return Vec2::new(p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32);
        }
    }
    Vec2::ZERO
}

/// One fixed step of the aiStyle-111 arm (`NPC.cs:66188-66393`), in C#
/// order. `npc_type` is the vanilla id (578) until the roster kind
/// lands. The caller owns the movement integration and drains
/// [`Style111Outcome`].
pub fn style111_step(npc: &mut Npc, npc_type: i32, env: &mut Style111Env<'_>) -> Style111Outcome {
    let mut out = Style111Outcome::default();
    // `:66190-66204` — the C# locals, verbatim. `flag` is born true and
    // never cleared (the despawn arm is dead code); `flag2`/`flag3`
    // are the 578 gates.
    let flag = true;
    let mut speed = 7.0f32; // num
    let mut blend_far = 60.0f32; // num2 (the dead despawn blend)
    let mut blend_near = 60.0f32; // num3
    let charge_speed_gate = 1.0f32; // num4
    let hover_damp = 0.96f32; // num5
    let mut charge_ticks = 30; // num6
    let mut rest_ticks = 20; // num7
    let approach_range = 200.0f32; // num8
    let mut bolt_damage = 40; // num9
    let mut bolt_type = 438; // num10
    let mut bolt_speed = 10.0f32; // num11
    let mut hover_band = 100.0f32; // num12
    let mut separation = false; // flag2
    let mut sep_accel = 0.05f32; // num13
    let (w, h) = npc.size();
    let mut muzzle = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0); // center
    target_closest_old_ones(npc, env);
    let target = target_center(npc, env); // GetTargetData().Center
    let mut materialize = false; // flag3
    if npc_type == 578 {
        // `:66211-66223` — the 578 tuning.
        speed = 4.0;
        blend_far = 20.0;
        blend_near = 20.0;
        materialize = true;
        separation = true;
        sep_accel = 0.1;
        bolt_type = 682;
        bolt_damage = 50;
        bolt_speed = 10.0;
        hover_band = 50.0;
        charge_ticks = 5;
        rest_ticks = 30;
        muzzle += Vec2::new(-npc.sprite_dir as f32 * 20.0, 10.0);
        // `:66224-66256` — the two dust gates under the netOffset shift.
        npc.x += npc.net_offset.x;
        npc.y += npc.net_offset.y;
        if env.rng.next_max(15) == 0 {
            let mut pos = Vec2::new(npc.x, npc.y);
            if npc.facing == -1 {
                pos.x += (w as i32 / 2) as f32;
            }
            let idx = env.dust.new_dust_direct(
                env.rng,
                env.dust_ctx,
                env.world,
                pos,
                w as i32 / 2,
                h as i32,
                272,
                0.0,
                0.0,
                0,
                Color::default(),
                1.0,
            );
            let d = env.dust.slot_mut(idx);
            d.scale = 0.65;
            d.velocity.y -= 0.8;
            if d.velocity.x * npc.facing as f32 > 0.0 {
                d.velocity.x *= -1.0;
            }
            if d.velocity.y > 0.0 {
                d.velocity *= -0.5;
            }
        }
        if env.rng.next_max(80) == 0 {
            // ONE NextFloat angle, then the two perfect dusts.
            let a = env.rng.next_float() * (std::f32::consts::PI * 2.0);
            let spin = Vec2::new(a.cos(), a.sin());
            let c = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
            let i1 = env.dust.new_dust_perfect(
                env.rng,
                env.dust_ctx,
                env.world,
                c,
                272,
                Some(spin),
                0,
                Color::default(),
                1.0,
            );
            {
                let d = env.dust.slot_mut(i1);
                d.velocity *= 1.1;
                d.no_gravity = true;
                // `customData = this` — the dropped field (module docs).
            }
            let i2 = env.dust.new_dust_perfect(
                env.rng,
                env.dust_ctx,
                env.world,
                c,
                272,
                Some(spin),
                0,
                Color::default(),
                1.0,
            );
            {
                let d = env.dust.slot_mut(i2);
                d.velocity *= 1.5;
                d.no_gravity = true;
            }
        }
        npc.x -= npc.net_offset.x;
        npc.y -= npc.net_offset.y;
    }
    if materialize {
        // `:66260-66267` — the alpha latch + the tracked cue.
        if npc.local_ai[1] == 0.0 {
            npc.alpha = 255;
        }
        if npc.local_ai[1] == 30.0 {
            out.sounds.push((
                "dd2_etherian_portal_spawn_enemy",
                npc.x + w / 2.0,
                npc.y + h / 2.0,
            ));
        }
        // `:66268-66294` — the 60-tick ramp + the dust iterations.
        if npc.local_ai[1] < 60.0 {
            npc.local_ai[1] += 1.0;
            npc.alpha = npc.alpha.saturating_sub(5);
            let iterations = npc.local_ai[1] as i32 / 10; // num14
            let size_len = ((w * w + h * h) as f64).sqrt() as f32;
            let dust_scale = size_len / 2.0 / 20.0; // num15
            let mut max_value = 5;
            if npc_type == 576 || npc_type == 577 {
                max_value = 1; // the dead Ogre branch, verbatim
            }
            for _ in 0..iterations {
                if env.rng.next_max(max_value) == 0 {
                    let idx = env.dust.new_dust_direct(
                        env.rng,
                        env.dust_ctx,
                        env.world,
                        Vec2::new(npc.x, npc.y),
                        w as i32,
                        h as i32,
                        27,
                        npc.vx,
                        0.0,
                        100,
                        Color::default(),
                        1.0,
                    );
                    let d = env.dust.slot_mut(idx);
                    d.scale = 0.55;
                    d.fade_in = 0.7;
                    d.velocity *= 0.1 * dust_scale;
                    d.velocity += Vec2::new(npc.vx, npc.vy);
                }
            }
        }
    }
    if separation {
        // `:66298-66319` — the same-type Manhattan push.
        for n in env.neighbors {
            if !n.is_self
                && n.active
                && n.ty == npc_type
                && (npc.x - n.x).abs() + (npc.y - n.y).abs() < w
            {
                if npc.x < n.x {
                    npc.vx -= sep_accel;
                } else {
                    npc.vx += sep_accel;
                }
                if npc.y < n.y {
                    npc.vy -= sep_accel;
                } else {
                    npc.vy += sep_accel;
                }
            }
        }
    }
    // `:66321-66326` — the tilt, the facing, the aim, the sight.
    npc.rotation = npc.vx.abs() * npc.facing as f32 * 0.1;
    npc.sprite_dir = npc.facing;
    let aim_raw = target - muzzle; // vector4
    let mut aim = safe_normalize(aim_raw, Vec2::Y) * speed; // vector5
    let sighted = crate::projectile::can_hit(
        env.world,
        npc.x + w / 2.0,
        npc.y + h / 2.0,
        1.0,
        1.0,
        target.x,
        target.y,
        1.0,
        1.0,
    ); // flag4
       // `:66327-66330` — the rest count-up.
    if npc.local_ai[0] < 0.0 {
        npc.local_ai[0] += 1.0;
    }
    if !flag {
        // `:66331-66335` — DEAD: flag is never cleared (module docs).
        let v = Vec2::new(npc.vx, npc.vy);
        let blended = (v * (blend_far - 1.0) + aim) / blend_far;
        npc.vx = blended.x;
        npc.vy = blended.y;
        // EncourageDespawn(10) — the despawn seam, unreachable.
    } else if distance(muzzle, target) > approach_range || !sighted {
        // `:66336-66339` — the far approach blend.
        let v = Vec2::new(npc.vx, npc.vy);
        let blended = (v * (blend_near - 1.0) + aim) / blend_near;
        npc.vx = blended.x;
        npc.vy = blended.y;
    } else if aim_raw.y < hover_band {
        // `:66340-66343` — the hover lift.
        npc.vy -= 0.03;
    } else if npc.local_ai[0] >= 0.0 {
        // `:66344-66362` — the charge.
        npc.vx *= hover_damp;
        npc.vy *= hover_damp;
        let len = ((npc.vx * npc.vx + npc.vy * npc.vy) as f64).sqrt() as f32;
        if len < charge_speed_gate && env.net_mode != 1 {
            npc.local_ai[0] += 1.0;
            if npc.local_ai[0] >= charge_ticks as f32 {
                npc.local_ai[0] = -(rest_ticks as f32);
                npc.facing = if aim.x > 0.0 { 1 } else { -1 };
                npc.sprite_dir = npc.facing;
                // The jitter chain: RandomVector2 (2 NextDouble), the
                // 0.005 pair, the normalize, the 1/160 pair.
                let mut shot = aim_raw + random_vector2(env.rng, -25.0, 25.0);
                shot.x *= 1.0 + env.rng.next_range(-20, 21) as f32 * 0.005;
                shot.y *= 1.0 + env.rng.next_range(-20, 21) as f32 * 0.005;
                shot = safe_normalize(shot, Vec2::Y) * bolt_speed;
                shot.x *= 1.0 + env.rng.next_range(-20, 21) as f32 * (1.0 / 160.0);
                shot.y *= 1.0 + env.rng.next_range(-20, 21) as f32 * (1.0 / 160.0);
                out.shot = Some(BugBolt {
                    projectile_type: bolt_type,
                    x: muzzle.x,
                    y: muzzle.y,
                    vx: shot.x,
                    vy: shot.y,
                    damage: bolt_damage,
                });
            }
        }
    }
    if !flag {
        return out; // `:66364-66367` — dead, verbatim.
    }
    // `:66368-66392` — the lift tail.
    let mut lift = target.y < npc.y + h / 2.0; // flag5
    if !lift {
        let px = (npc.x + w / 2.0) as i32 >> 4;
        let py = (npc.y + h / 2.0) as i32 >> 4;
        // `WorldGen.InWorld(x, y, 2)` — the 2-tile border refusal.
        if px >= 2 && px < env.world.width - 2 && py >= 2 && py < env.world.height - 2 {
            if env.world.tile(px, py).is_some() && solid_tile_strict(env.world, px, py) {
                lift = true;
            }
        }
    }
    if lift {
        npc.vy -= 0.2;
        if npc.vy < -10.0 {
            npc.vy = -10.0;
        }
    }
    out
}

/// `NPC.FindFrame` case 578 (`NPC.cs:67875-67886`) — the counter pair
/// (the style73 convention). The frame pass ALSO writes
/// `rotation = vx * 0.1` (`:67876`), overwriting the AI's tilt — the
/// wiring caller applies it after this pair.
pub fn bug_find_frame(frame_row: u32, frame_counter: f32) -> (u32, f32) {
    let mut counter = frame_counter + 1.0; // `++this.frameCounter`
    let mut row = frame_row;
    if counter >= 4.0 {
        row += 1;
        counter = 0.0;
        if row >= LIGHTNING_BUG_FRAME_COUNT {
            row = 0;
        }
    }
    (row, counter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A 578-shaped NPC (the style73 placeholder convention).
    fn bug(x: f32, y: f32) -> Npc {
        let d = &LIGHTNING_BUG;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n.facing = 1;
        n.sprite_dir = 1;
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

    fn crystal(slot_self: bool, x: f32, y: f32) -> BugNeighbor {
        BugNeighbor {
            is_self: slot_self,
            active: true,
            ty: 548,
            x,
            y,
            width: 40,
            height: 40,
            dont_take_damage_from_hostiles: false,
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

    /// `NPC.cs:16536-16553` — the SetDefaults row, field by field.
    #[test]
    fn w4_d33_s111_the_defaults_row() {
        let d = &LIGHTNING_BUG;
        assert_eq!((d.width, d.height), (30, 24));
        assert_eq!(d.ai_style, 111);
        assert_eq!((d.damage, d.defense, d.life_max), (80, 36, 500));
        assert_eq!(d.hit_sound, "dd2_lightning_bug_hurt");
        assert_eq!(d.death_sound, "dd2_lightning_bug_death");
        assert_eq!(d.knock_back_resist, 0.8);
        assert!(d.no_gravity && d.no_tile_collide && d.lava_immune && d.net_always);
        assert_eq!(d.npc_slots, 0.0);
        assert_eq!(LIGHTNING_BUG_FRAME_COUNT, 4);
    }

    /// `:66260-66275` — the materialize ramp: tick 1 sets alpha 255
    /// then -5 → 250; localAI[1] counts to 60 and stops; the alpha
    /// floors at 0 (51 ticks of -5 from 255).
    #[test]
    fn w4_d33_s111_the_materialize_ramp() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(100000.0, 100000.0)];
        let neighbors: [BugNeighbor; 0] = [];
        let mut rng = UnifiedRandom::new(111);
        let mut n = bug(100.0, 100.0);
        let mut env = Style111Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            neighbors: &neighbors,
            net_mode: 0,
        };
        let _ = style111_step(&mut n, 578, &mut env);
        assert_eq!(n.local_ai[1], 1.0);
        assert_eq!(n.alpha, 250, "255 then the same-tick -5");
        for _ in 0..59 {
            let _ = style111_step(&mut n, 578, &mut env);
        }
        assert_eq!(n.local_ai[1], 60.0);
        assert_eq!(n.alpha, 0, "the floor");
        let _ = style111_step(&mut n, 578, &mut env);
        assert_eq!(n.local_ai[1], 60.0, "the ramp stops");
    }

    /// `NPCUtils.cs:262-278` — the crystal beats the player when its
    /// DistanceSquared-root is strictly under the player's adjusted
    /// distance; the target index is slot+300 and the aim reads the
    /// crystal's centre.
    #[test]
    fn w4_d33_s111_the_crystal_target_wins() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        // The player at 190 px (inside the 200 filter), the crystal at
        // 60 px — the crystal wins.
        let players = [cand(290.0, 100.0)];
        let neighbors = [crystal(true, 0.0, 0.0), crystal(false, 160.0, 100.0)];
        let mut rng = UnifiedRandom::new(111);
        let mut n = bug(100.0, 100.0);
        n.local_ai[1] = 60.0; // skip the materialize draws
        let mut env = Style111Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            neighbors: &neighbors,
            net_mode: 0,
        };
        let _ = style111_step(&mut n, 578, &mut env);
        assert_eq!(n.target, 301, "slot 1 + 300");
        assert_eq!(n.target_rect, (160, 100, 40, 40));
        // The player beyond 200 px is filtered — the crystal still wins.
        let players = [cand(100000.0, 100000.0)];
        let mut n = bug(100.0, 100.0);
        n.local_ai[1] = 60.0;
        let mut env = Style111Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            neighbors: &neighbors,
            net_mode: 0,
        };
        let _ = style111_step(&mut n, 578, &mut env);
        assert_eq!(n.target, 301);
    }

    /// `:66298-66319` — the separation: a same-type neighbour inside
    /// the Manhattan width pushes ±0.1 per axis away from it.
    #[test]
    fn w4_d33_s111_the_separation_push() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let players = [cand(100000.0, 100000.0)];
        let mut sib = crystal(false, 110.0, 105.0);
        sib.ty = 578;
        let neighbors = [crystal(true, 0.0, 0.0), sib];
        let mut rng = UnifiedRandom::new(111);
        let mut n = bug(100.0, 100.0);
        n.local_ai[1] = 60.0;
        // `TargetClosestOldOnesInvasion` (`NPCUtils.cs:269-277`) writes
        // `target`/`targetRect` ONLY on a found target, so a valid player
        // target from an earlier tick survives a failed re-search. Seed it
        // here so `GetTargetData` reads the far player's centre (a fresh
        // NPC's default 255 reads the `default` (0,0) and takes the short
        // hover arm instead of the far blend).
        n.target = 0;
        n.target_rect = (100000, 100000, 20, 42);
        let mut env = Style111Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            neighbors: &neighbors,
            net_mode: 0,
        };
        let _ = style111_step(&mut n, 578, &mut env);
        // The sibling is right+below: vx -= 0.1, vy -= 0.1 — then the
        // far-approach blend runs (the search finds nothing within
        // 200 px, so target stays 0 and GetTargetData reads the far
        // player's centre). Pin the push through the 20-blend:
        // v' = (v*19 + aim)/20 with aim = SafeNormalize(target-muzzle)*4.
        let muzzle = Vec2::new(100.0 + 15.0 - 20.0, 100.0 + 12.0 + 10.0);
        let target = Vec2::new(100000.0 + 10.0, 100000.0 + 21.0);
        let aim = safe_normalize(target - muzzle, Vec2::Y) * 4.0;
        let want_x = (-0.1 * 19.0 + aim.x) / 20.0;
        let want_y = (-0.1 * 19.0 + aim.y) / 20.0;
        assert!((n.vx - want_x).abs() < 1e-5, "vx {}", n.vx);
        assert!((n.vy - want_y).abs() < 1e-5, "vy {}", n.vy);
    }

    /// `:66344-66361` — the charge fires the jittered 682 bolt at 5
    /// ticks: the draw order is 2 NextDouble + 4 Next(-20,21)
    /// (X,Y,X,Y), the rest is -30, the facing follows the aim sign.
    #[test]
    fn w4_d33_s111_the_charge_bolt_draw_order() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        // The target below the hover band (aim_raw.y >= 50) and within
        // 200 px of the muzzle, sighted (air world).
        let players = [cand(100.0, 250.0)];
        let neighbors: [BugNeighbor; 0] = [];
        let mut rng = UnifiedRandom::new(111);
        let mut oracle = UnifiedRandom::new(111);
        let mut n = bug(100.0, 100.0);
        n.local_ai[1] = 60.0;
        n.local_ai[0] = 4.0; // one tick from the 5-tick fire
        let mut env = Style111Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            neighbors: &neighbors,
            net_mode: 0,
        };
        let out = style111_step(&mut n, 578, &mut env);
        // The two head gates drew first (menu ctx — no internals, but
        // the spark gate's NextFloat angle draws BEFORE the dusts).
        oracle.next_max(15);
        if oracle.next_max(80) == 0 {
            oracle.next_float();
        }
        // The bolt chain.
        oracle.next_double();
        oracle.next_double();
        oracle.next_range(-20, 21);
        oracle.next_range(-20, 21);
        oracle.next_range(-20, 21);
        oracle.next_range(-20, 21);
        assert_eq!(env.rng.peek(), oracle.peek(), "the bolt draw order");
        let shot = out.shot.expect("the 5-tick fire");
        assert_eq!(shot.projectile_type, 682);
        assert_eq!(shot.damage, 50);
        assert_eq!(n.local_ai[0], -30.0, "the rest");
        // The client never fires.
        let mut n = bug(100.0, 100.0);
        n.local_ai[1] = 60.0;
        n.local_ai[0] = 4.0;
        env.net_mode = 1;
        let out = style111_step(&mut n, 578, &mut env);
        assert!(out.shot.is_none());
        assert_eq!(n.local_ai[0], 4.0, "no charge tick on the client");
    }

    /// `:66368-66392` — the lift tail: the target above lifts -0.2
    /// capped -10; the solid centre tile lifts too.
    #[test]
    fn w4_d33_s111_the_lift_tail() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        // The target above (y 0 centre 21 < the bug's 112) — but the
        // hover arm may also touch vy; pin the cap with a deep fall.
        let players = [cand(100.0, 0.0)];
        let neighbors: [BugNeighbor; 0] = [];
        let mut rng = UnifiedRandom::new(111);
        let mut n = bug(100.0, 100.0);
        n.local_ai[1] = 60.0;
        n.vy = -9.95;
        let mut env = Style111Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            players: &players,
            neighbors: &neighbors,
            net_mode: 0,
        };
        let _ = style111_step(&mut n, 578, &mut env);
        assert!(n.vy >= -10.0, "the -10 cap, got {}", n.vy);
    }

    /// `NPC.cs:67875-67886` — the 4-tick row advance wraps at 4.
    #[test]
    fn w4_d33_s111_the_find_frame_wraps_at_four() {
        assert_eq!(bug_find_frame(0, 2.0), (0, 3.0));
        assert_eq!(bug_find_frame(0, 3.0), (1, 0.0));
        assert_eq!(bug_find_frame(3, 3.0), (0, 0.0), "the wrap");
    }

    /// The full-pool stream parity: 40 ticks on the REAL dust ctx must
    /// match a cloned twin bit-for-bit (the style101 twin precedent).
    #[test]
    fn w4_d33_s111_the_full_pool_run_is_twin_stable() {
        let w = air();
        let c = spawn_ctx();
        let players = [cand(300.0, 300.0)];
        let neighbors = [crystal(true, 0.0, 0.0), crystal(false, 400.0, 300.0)];
        let mut a = bug(280.0, 280.0);
        let mut b = a.clone();
        let mut pool_a = DustPool::new();
        let mut pool_b = DustPool::new();
        let mut rng_a = UnifiedRandom::new(578);
        let mut rng_b = UnifiedRandom::new(578);
        for tick in 0..40 {
            let mut env_a = Style111Env {
                world: &w,
                dust: &mut pool_a,
                dust_ctx: &c,
                rng: &mut rng_a,
                players: &players,
                neighbors: &neighbors,
                net_mode: 0,
            };
            let _ = style111_step(&mut a, 578, &mut env_a);
            let mut env_b = Style111Env {
                world: &w,
                dust: &mut pool_b,
                dust_ctx: &c,
                rng: &mut rng_b,
                players: &players,
                neighbors: &neighbors,
                net_mode: 0,
            };
            let _ = style111_step(&mut b, 578, &mut env_b);
            assert_eq!(rng_a.peek(), rng_b.peek(), "stream @{tick}");
            assert_eq!(a.vx.to_bits(), b.vx.to_bits(), "vx @{tick}");
            assert_eq!(
                a.local_ai[0].to_bits(),
                b.local_ai[0].to_bits(),
                "charge @{tick}"
            );
        }
    }
}
