//! The aiStyle-9 family — `NPC.AI()`'s `if (this.aiStyle == 9)` arm
//! (`NPC.cs:21377-21571`), S1-D-3 (aiStyle-9 round). The caster-sphere
//! family: the Burning Sphere 25 (`SetDefaults` `NPC.cs:9093-9107`), the
//! Water Sphere 33 (`:9135-9149`), the Chaos Ball 30 (`:15699-15713`) and
//! its Tim clone 665 (`:17403-17417`, folded into `NpcKind::ChaosBall` —
//! identical stats), the Vile Spits 112/666 (`:10326-10341`,
//! `:17418-17433`) and the Solar Flare 516 (`:15394-15408`).
//!
//! The ported kinds are the three spheres (25/30+665/33). The Vile Spits
//! and the Solar Flare have no `NpcKind` yet; their per-type branches are
//! ported here as pure helpers (`vile_spit_counter_tick`,
//! `solar_flare_launch_velocity`, `solar_flare_steer`) so the maths and the
//! RNG order are pinned before the kinds land.
//!
//! ## RNG order (by construction)
//!
//! The arm's `Main.rand` draws, in C# order:
//! 1. type 516's first launch (`:21395-21400`): `RotatedByRandom(π/2)`
//!    draws `NextDouble()` then `NextDouble()` (`Utils.cs:1697-1700`,
//!    `d1 * max - d2 * max`), then `NextFloat()` for the 6..10 speed.
//! 2. the 112/666 `ai[0] == 2` burst (`:21481-21487`): 20 × `Dust.NewDust`
//!    (each draws its own `Main.rand` rolls inside `Dust.cs`).
//! 3. the two-iteration dust loop (`:21517-21567`): per iteration, the
//!    per-type `NewDust` rolls; type 33 adds `Main.rand.Next(5)` AFTER its
//!    3 `NewDust` calls (`:21545`) and a conditional 4th `NewDust`.
//!
//! The dust bursts are WIRED (W4-d52) through the live `DustPool` on the
//! caller's `UpdateNPCs` stream: the 112/666 `ai[0] == 2` burst stays an
//! unported-kind skip, the two-iteration dust loop emits for the ported
//! sphere kinds (30/665 type 27, 33 type 172 + the Next(5) gate, 25 the
//! else group's type 6). The 516 helper keeps the exact 3-draw order for
//! its future wiring (witnessed by
//! `solar_flare_launch_draws_two_doubles_then_a_float`).

use crate::npc::{Npc, NpcKind};
use crate::npc_target::{target_closest, TargetCandidate};
use crate::player::Player;
use tr_core::{UnifiedRandom, World, TILE_PX};

/// The world flags the arm's `getGoodWorld` branches read
/// (`NPC.cs:21419-21437`, `:21450-21464`). The port has no `getGoodWorld`
/// runtime flag yet (the seed only rides worldgen — `SeedFlags::
/// FOR_THE_WORTHY`), so the wiring passes all-false and the ftw rows stay
/// unreachable until the flag lands; the table itself is complete.
#[derive(Clone, Copy, Debug, Default)]
pub struct Style9Flags {
    /// `Main.getGoodWorld`.
    pub get_good_world: bool,
    /// `NPC.AnyNPCs(35)` — a live Skeletron head (`:21421`, `:21452`).
    pub skeletron_head_alive: bool,
    /// `NPC.AnyNPCs(113)` — a live Wall of Flesh (`:21429`, `:21456`).
    pub wall_of_flesh_alive: bool,
    /// `this.RedHatSkeletronAdjustmentsEnabled()` (`:21379`'s `flag9`,
    /// read at `:21424`) — the caller passes the NPC's own predicate.
    pub red_hat: bool,
}

/// The launch speed `num120` (`NPC.cs:21410-21437`): 6 default, the
/// Burning Sphere 25 → 5, the Vile Spits 112/666 → 7; the `getGoodWorld`
/// overrides — 33 with a live Skeletron head → 10 (8 with the red hat),
/// 25 with a live Wall of Flesh → 14, 666 → 10.
pub fn launch_speed(npc_type: i32, flags: Style9Flags) -> i32 {
    // `:21410-21418`.
    let mut speed = 6;
    if npc_type == 25 {
        speed = 5;
    }
    if npc_type == 112 || npc_type == 666 {
        speed = 7;
    }
    // `:21419-21437` — the ftw overrides, in the C#'s order (the 666 row
    // runs last and overwrites the 112/666 base 7).
    if flags.get_good_world {
        if npc_type == 33 && flags.skeletron_head_alive {
            speed = 10;
            if flags.red_hat {
                speed = 8;
            }
        }
        if npc_type == 25 && flags.wall_of_flesh_alive {
            speed = 14;
        }
        if npc_type == 666 {
            speed = 10;
        }
    }
    speed
}

/// The `getGoodWorld` invulnerability block (`NPC.cs:21450-21464`): the
/// Water Sphere behind a live Skeletron head, the Burning Sphere behind a
/// live Wall of Flesh, and the Vile Spit EoW above the surface
/// (`Center.Y / 16 < Main.worldSurface`, a DOUBLE comparison — `:21460`).
/// Returns whether `dontTakeDamage` should latch true (the C# never
/// clears it here).
pub fn good_world_latches_invulnerability(
    npc_type: i32,
    center_y: f32,
    world_surface: f64,
    flags: Style9Flags,
) -> bool {
    if !flags.get_good_world {
        return false;
    }
    if npc_type == 33 && flags.skeletron_head_alive {
        return true;
    }
    if npc_type == 25 && flags.wall_of_flesh_alive {
        return true;
    }
    if npc_type == 666 && (center_y as f64 / 16.0) < world_surface {
        return true;
    }
    false
}

/// The Vile Spits' `ai[0]` counter (`NPC.cs:21472-21488`): increment,
/// clamp at 3, and the `ai[0] == 2` tick doubles the move
/// (`position += velocity` ahead of the generic integration) beside the
/// 20-dust burst (a documented skip). Returns the new counter and whether
/// the extra move fires. The damage rows (`:21467-21471` — 112 restores
/// `defDamage`, 666 rides `GetAttackDamage_CappedAtMaster(32f)`,
/// `NPC.cs:7041-7045`) wait on the port's difficulty scale.
#[allow(dead_code)] // the 112/666 kinds' wiring consumes the counter
pub fn vile_spit_counter_tick(ai0: f32) -> (f32, bool) {
    let mut ai0 = ai0 + 1.0;
    if ai0 > 3.0 {
        ai0 = 3.0;
    }
    (ai0, ai0 == 2.0)
}

/// `Utils.RotatedBy` (`Utils.cs:1675-1684`) about the origin: the f32
/// cos/sin of the f64 angle, the XNA rotation direction.
#[allow(dead_code)] // the Solar Flare 516 kind's wiring consumes the rotation
fn rotated_by(v: (f32, f32), radians: f64) -> (f32, f32) {
    let num = radians.cos() as f32;
    let num2 = radians.sin() as f32;
    (v.0 * num - v.1 * num2, v.0 * num2 + v.1 * num)
}

/// The Solar Flare's first-launch velocity (`NPC.cs:21386-21400`): the
/// normalized player delta (NaN → `-UnitY`), `RotatedByRandom(π/2)`
/// (`Utils.cs:1697-1700` — `NextDouble() * max - NextDouble() * max`, TWO
/// draws in that order), `RotatedBy(-π/4)`, the `Y > 0.2` clamp, then
/// `6f + NextFloat() * 4f` (the THIRD draw). `dir` is the already
/// normalized `player.Center - npc.Center`.
#[allow(dead_code)] // the Solar Flare 516 kind's wiring consumes the launch
pub fn solar_flare_launch_velocity(dir: (f32, f32), rng: &mut UnifiedRandom) -> (f32, f32) {
    // `:21391-21394` — `Vector2.HasNaNs()` → `-Vector2.UnitY`.
    let mut v = if dir.0.is_nan() || dir.1.is_nan() {
        (0.0, -1.0)
    } else {
        dir
    };
    // `:21395` — RotatedByRandom(1.5707963705062866) then
    // RotatedBy(-0.7853981852531433). The two NextDouble draws happen
    // left-to-right inside the subtraction (C# operand order).
    let d1 = rng.next_double();
    let d2 = rng.next_double();
    let angle = d1 * 1.5707963705062866 - d2 * 1.5707963705062866;
    v = rotated_by(v, angle);
    v = rotated_by(v, -0.7853981852531433);
    // `:21396-21399`.
    if v.1 > 0.2 {
        v.1 = 0.2;
    }
    // `:21400` — the speed draw comes AFTER both rotation draws.
    let speed = 6.0 + rng.next_float() * 4.0;
    (v.0 * speed, v.1 * speed)
}

/// The Solar Flare's per-tick homing steer (`NPC.cs:21499-21513`): the
/// rotation `0.1 * direction`, then the velocity blend
/// `(velocity * 14 + aim * (|velocity| + 1/12)) / 15` with the NaN aim
/// falling back to `(direction, 0)`, and the `|velocity| < 6` → `×1.05`
/// boost. Returns the new (velocity, rotation delta). `aim` is the
/// normalized player delta.
#[allow(dead_code)] // the Solar Flare 516 kind's wiring consumes the steer
pub fn solar_flare_steer(
    vel: (f32, f32),
    aim: (f32, f32),
    direction: i32,
    rotation: f32,
) -> ((f32, f32), f32) {
    // `:21499`.
    let rotation = rotation + 0.1 * direction as f32;
    // `:21505-21508` — the NaN fallback.
    let aim = if aim.0.is_nan() || aim.1.is_nan() {
        (direction as f32, 0.0)
    } else {
        aim
    };
    // `:21500-21509` — num126 = 15, num127 = 1/12.
    let len = (vel.0 * vel.0 + vel.1 * vel.1).sqrt();
    let mut out = (
        (vel.0 * (15.0 - 1.0) + aim.0 * (len + 1.0 / 12.0)) / 15.0,
        (vel.1 * (15.0 - 1.0) + aim.1 * (len + 1.0 / 12.0)) / 15.0,
    );
    // `:21510-21513` — the length test reads the BLEND's length.
    if (out.0 * out.0 + out.1 * out.1).sqrt() < 6.0 {
        out.0 *= 1.05;
        out.1 *= 1.05;
    }
    (out, rotation)
}

/// One fixed step of the aiStyle-9 arm for the ported sphere kinds
/// (25/30+665/33) — `NPC.cs:21377-21571` minus the unported per-type
/// branches (516's homing `:21380-21406`/`:21497-21515`, the 112/666
/// counter/damage `:21465-21495`); W4-d52 wires the dust loop through the
/// live pool (the `:21561` light stays the visual lane's cited skip — see
/// the module header for the exact rand order).
///
/// The seam: `TargetClosest` rides `npc_target::target_closest` (the
/// `EverLastingTicker` is not plumbed into `Npc::update` yet — the 0
/// stamp, the S1-D-3p zombie precedent); the position integration is the
/// generic `noGravity`/`noTileCollide` step (`NPC.cs:90495-90498`) the
/// port runs inside each family arm; the off-world guard is the port's
/// existing world-edge stop (the pre-axis `sphere_step`'s guard).
pub fn sphere_family_step(
    npc: &mut Npc,
    world: &World,
    player: &Player,
    flags: Style9Flags,
    rng: &mut UnifiedRandom,
    dust: &mut tr_core::particles::DustPool,
    dust_ctx: &tr_core::particles::SpawnCtx,
) {
    // The C# arm keys on the VANILLA type (`this.type`, NPCID: BurningSphere
    // 25 / ChaosBall 30 / WaterSphere 33) - `net_id()` carries it; `to_u16`
    // is the port's INTERNAL protocol space (data.rs:1006-1009) and would
    // mis-route the speed ladder (MERGER-L7 first-run fix).
    let npc_type = npc.kind.net_id();
    // `:21407-21449` — the ONE-SHOT launch: only while `target == 255`
    // (the SetDefaults sentinel). `TargetClosest()` (`:21409`,
    // `NPC.cs:77652-77666`) picks and faces the player first.
    if npc.target == 255 {
        let cand = TargetCandidate {
            active: true,
            dead: player.dead,
            // The port has no hardcore ghosts (`Player.ghost`).
            ghost: false,
            x: player.x,
            y: player.y,
            width: crate::player::WIDTH as i32,
            height: crate::player::HEIGHT as i32,
            // `Player.aggro`/`npcTypeNoAggro`/`tankPet` are unported —
            // the stock 0/false/None (the S1-D-3p zombie precedent).
            aggro: 0,
            item_animation: player.frame_inputs.swing_anim as i32,
            npc_type_no_aggro: false,
            tank_pet: None,
        };
        target_closest(npc, world, &[cand], true, 9, 0);
        let speed = launch_speed(npc_type, flags) as f32;
        // `:21438-21448` — the normalized delta × speed, with the C#'s
        // multiply order (`num123 = speed / dist` FIRST, then the
        // per-axis multiply) and the `dist <= 0 → 1` guard.
        let (cx, cy) = npc.center();
        let (pcx, pcy) = player.center();
        let dx = pcx - cx;
        let dy = pcy - cy;
        let mut dist = (dx * dx + dy * dy).sqrt();
        if dist <= 0.0 {
            dist = 1.0;
        }
        let scale = speed / dist;
        npc.vx = dx * scale;
        npc.vy = dy * scale;
    }
    // `:21450-21464` — the ftw invulnerability latch (never cleared).
    if !npc.dont_take_damage {
        let (_, cy) = npc.center();
        if good_world_latches_invulnerability(npc_type, cy, world.world_surface, flags) {
            npc.dont_take_damage = true;
        }
    }
    // `:21496` — `EncourageDespawn(100)` (`NPC.cs:7249-7256`): the port's
    // `encourage_despawn` keeps the -1 sentinel semantics.
    npc.encourage_despawn(100);
    // `:21516-21567` — the dust loop. W4-d52 wires the ported kinds'
    // emissions through the live pool on the caller's stream: 30/665 the
    // alpha write + 2×2 type-27 dusts (`:21519-21529`, site 21524), 33 the
    // 2×(3 type-172 dusts + the Next(5) gate + the conditional 4th)
    // (`:21531-21551`, site 21538), 25 the else group's 2 type-6 dusts
    // (`:21559-21566`; the `:21561` centre AddLight(1, 0.3, 0.1) is the
    // visual lane's cited skip — `npc_lights` covers only the aiStyle-8
    // tail, and the light draws no RNG). The netOffset add/subtract pair
    // (`:21516`/`:21569`) nets to zero single-player.
    sphere_dust_loop(npc, rng, dust, dust_ctx, world);
    // `:21568` — the spin rides `base.direction` (the facing the
    // TargetClosest launch wrote; the C# never re-faces afterwards).
    npc.rotation += 0.4 * npc.facing as f32;
    // The generic integration (`NPC.cs:90495-90498`): noGravity +
    // noTileCollide — straight through everything.
    npc.x += npc.vx;
    npc.y += npc.vy;
    // The port's world-edge stop (the pre-axis `sphere_step` guard).
    if npc.y > world.height as f32 * TILE_PX + 200.0 {
        npc.hp = 0;
    }
}

/// W4-d52 — the sphere family's two-iteration dust loop (`NPC.cs:21516-
/// 21567`): per iteration, the Chaos Ball (30/665) writes alpha 255 and
/// spawns 2 type-27 dusts (`:21519-21529`, site 21524); the Water Sphere
/// (33) spawns 3 type-172 dusts with the velocity/3 × iteration position
/// offset, then the `Next(5)==0` gate opens a 4th (`:21531-21551`, site
/// 21538); the else group (25) spawns one type-6 dust (`:21559-21566`;
/// the `:21561` centre light is the visual lane's cited skip, zero RNG).
/// The 112/666 arm (`:21553-21558`) is an unported-kind skip.
fn sphere_dust_loop(
    npc: &mut Npc,
    rng: &mut UnifiedRandom,
    dust: &mut tr_core::particles::DustPool,
    ctx: &tr_core::particles::SpawnCtx,
    world: &World,
) {
    use tr_core::particles::Color;
    let (w, h) = npc.size();
    let (wi, hi) = (w as i32, h as i32);
    let pos = glam::Vec2::new(npc.x, npc.y + 2.0);
    for num128 in 0..2 {
        if npc.kind == NpcKind::ChaosBall {
            // `:21519-21521` — alpha = 255 (idempotent across iterations).
            npc.alpha = 255;
            // `:21522-21529` — the two type-27 dusts.
            for _ in 0..2 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, wi, hi, 27, npc.vx * 0.2, npc.vy * 0.2, 100,
                    Color::default(), 1.3,
                );
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 0.3;
                d.velocity.x -= npc.vx * 0.2;
                d.velocity.y -= npc.vy * 0.2;
            }
        } else if npc.kind == NpcKind::WaterSphere {
            // `:21533-21544` — the three type-172 dusts with the
            // velocity/3 × iteration position offset.
            for _num131 in 0..3 {
                let num132 = npc.vx / 3.0 * num128 as f32;
                let num133 = npc.vy / 3.0 * num128 as f32;
                let num134 = 2;
                let idx = dust.new_dust(
                    rng,
                    ctx,
                    world,
                    glam::Vec2::new(npc.x + num134 as f32, npc.y + num134 as f32),
                    wi - num134 * 2,
                    hi - num134 * 2,
                    172,
                    0.0,
                    0.0,
                    100,
                    Color::default(),
                    1.2,
                );
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 0.1;
                d.velocity += glam::Vec2::new(npc.vx * 0.5, npc.vy * 0.5);
                d.position.x -= num132;
                d.position.y -= num133;
            }
            // `:21545-21551` — the Next(5) gate + the conditional 4th.
            if rng.next_max(5) == 0 {
                let num136 = 2;
                let idx = dust.new_dust(
                    rng,
                    ctx,
                    world,
                    glam::Vec2::new(npc.x + num136 as f32, npc.y + num136 as f32),
                    wi - num136 * 2,
                    hi - num136 * 2,
                    172,
                    0.0,
                    0.0,
                    100,
                    Color::default(),
                    0.6,
                );
                let d = dust.slot_mut(idx);
                d.velocity *= 0.25;
                d.velocity += glam::Vec2::new(npc.vx * 0.5, npc.vy * 0.5);
            }
        } else {
            // `:21559-21566` — the else group (25): the `:21561` centre
            // AddLight is the visual lane's cited skip (npc_lights covers
            // only the aiStyle-8 tail; zero RNG); the type-6 dust rides
            // the pool.
            let idx = dust.new_dust(
                rng, ctx, world, pos, wi, hi, 6, npc.vx * 0.2, npc.vy * 0.2, 100,
                Color::default(), 2.0,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.velocity.x *= 0.3;
            d.velocity.y *= 0.3;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// The sphere arena: air above a dirt floor at y = 120 (the npc.rs
    /// `ground_air` shape).
    fn arena() -> World {
        let mut w = World::new("t".into(), 200, 200);
        for x in 0..200 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn player_at(w: &World, x: f32, y: f32) -> Player {
        let mut p = Player::spawn(w);
        p.x = x;
        p.y = y;
        p.dead = false;
        p
    }

    /// W4-d52 — the solo-test entry: a fresh scratch pool + the permissive
    /// whole-world ctx (the `update_rng` house pattern). The live sim
    /// threads the real pool through the 7-arg step.
    fn step_solo(npc: &mut Npc, w: &World, player: &Player, rng: &mut UnifiedRandom) {
        let mut dust = tr_core::particles::DustPool::new();
        let ctx = tr_core::particles::SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: glam::Vec2::ZERO,
            screen_size: glam::Vec2::new(w.width as f32 * TILE_PX, w.height as f32 * TILE_PX),
            max_dust_to_draw: 6000,
        };
        sphere_family_step(npc, w, player, Style9Flags::default(), rng, &mut dust, &ctx);
    }

    /// `NPC.cs:21410-21437` — the full speed table, classic and ftw.
    #[test]
    fn s1_d_3_s9_the_launch_speed_table_matches_the_csharp() {
        let classic = Style9Flags::default();
        assert_eq!(launch_speed(30, classic), 6); // Chaos Ball
        assert_eq!(launch_speed(665, classic), 6); // ChaosBallTim
        assert_eq!(launch_speed(33, classic), 6); // Water Sphere
        assert_eq!(launch_speed(25, classic), 5); // Burning Sphere
        assert_eq!(launch_speed(112, classic), 7); // Vile Spit
        assert_eq!(launch_speed(666, classic), 7); // Vile Spit EoW
        let ftw = Style9Flags {
            get_good_world: true,
            ..Default::default()
        };
        // No bosses alive: the base rows stand (666 → 10 regardless).
        assert_eq!(launch_speed(33, ftw), 6);
        assert_eq!(launch_speed(25, ftw), 5);
        assert_eq!(launch_speed(666, ftw), 10);
        let head = Style9Flags {
            skeletron_head_alive: true,
            ..ftw
        };
        assert_eq!(launch_speed(33, head), 10);
        let red_hat = Style9Flags { red_hat: true, ..head };
        assert_eq!(launch_speed(33, red_hat), 8);
        let wof = Style9Flags {
            wall_of_flesh_alive: true,
            ..ftw
        };
        assert_eq!(launch_speed(25, wof), 14);
        // The red hat alone (no head) never discounts.
        let hat_only = Style9Flags { red_hat: true, ..ftw };
        assert_eq!(launch_speed(33, hat_only), 6);
    }

    /// `NPC.cs:21450-21464` — the ftw invulnerability rows, incl. the
    /// 666 double-precision surface comparison.
    #[test]
    fn s1_d_3_s9_the_ftw_invulnerability_rows_match() {
        let classic = Style9Flags::default();
        assert!(!good_world_latches_invulnerability(33, 0.0, 100.0, classic));
        let ftw = Style9Flags {
            get_good_world: true,
            skeletron_head_alive: true,
            wall_of_flesh_alive: true,
            red_hat: false,
        };
        assert!(good_world_latches_invulnerability(33, 0.0, 100.0, ftw));
        assert!(good_world_latches_invulnerability(25, 0.0, 100.0, ftw));
        // 666 above the surface (Center.Y/16 < worldSurface) latches;
        // at/below it does not.
        assert!(good_world_latches_invulnerability(666, 99.0 * 16.0, 100.0, ftw));
        assert!(!good_world_latches_invulnerability(666, 100.0 * 16.0, 100.0, ftw));
        assert!(!good_world_latches_invulnerability(30, 0.0, 100.0, ftw));
    }

    /// `NPC.cs:21472-21488` — the Vile Spit counter: ++, clamp 3, the
    /// ==2 double-move tick.
    #[test]
    fn s1_d_3_s9_the_vile_spit_counter_clamps_and_double_moves() {
        assert_eq!(vile_spit_counter_tick(0.0), (1.0, false));
        assert_eq!(vile_spit_counter_tick(1.0), (2.0, true));
        assert_eq!(vile_spit_counter_tick(2.0), (3.0, false));
        assert_eq!(vile_spit_counter_tick(3.0), (3.0, false));
    }

    /// `NPC.cs:21395-21400` + `Utils.cs:1697-1700` — the launch consumes
    /// EXACTLY three draws in the order NextDouble, NextDouble,
    /// NextFloat: replaying the same seed manually must reproduce the
    /// velocity bit-for-bit, and the stream position must match.
    #[test]
    fn solar_flare_launch_draws_two_doubles_then_a_float() {
        let dir = (0.6f32, 0.8f32); // already normalized
        let mut rng = UnifiedRandom::new(42);
        let (vx, vy) = solar_flare_launch_velocity(dir, &mut rng);
        // The manual replay in the C#'s draw order.
        let mut replay = UnifiedRandom::new(42);
        let d1 = replay.next_double();
        let d2 = replay.next_double();
        let angle = d1 * 1.5707963705062866 - d2 * 1.5707963705062866;
        let v = rotated_by(dir, angle);
        let v = rotated_by(v, -0.7853981852531433);
        let y = if v.1 > 0.2 { 0.2 } else { v.1 };
        let speed = 6.0 + replay.next_float() * 4.0;
        assert_eq!(vx, v.0 * speed);
        assert_eq!(vy, y * speed);
        // The streams are at the same position: the next draw agrees.
        assert_eq!(rng.next_double(), replay.next_double());
    }

    /// `NPC.cs:21396-21399` — the Y clamp caps a downward launch.
    #[test]
    fn solar_flare_launch_clamps_y_to_02() {
        // Straight down: any rotation in ±π/2 - π/4 keeps Y >= cos(3π/4)
        // ... pick the unrotated case via a seed-independent check: the
        // clamp is on the FINAL vector, so run several seeds and assert.
        for seed in 0..16i32 {
            let mut rng = UnifiedRandom::new(seed);
            let (_, vy) = solar_flare_launch_velocity((0.0, 1.0), &mut rng);
            let speed = 10.0; // the max possible speed
            assert!(vy <= 0.2 * speed + 1e-4, "seed {seed}: vy {vy}");
        }
    }

    /// `NPC.cs:21499-21513` — the homing blend maths and the <6 boost.
    #[test]
    fn solar_flare_steer_blends_and_boosts() {
        // A fast velocity aimed straight at the player: the blend keeps
        // the direction, |out| = (14*|v| + |v| + 1/12)/15 > 6 → no boost.
        let ((vx, vy), rot) = solar_flare_steer((10.0, 0.0), (1.0, 0.0), 1, 0.0);
        let expect = (10.0 * 14.0 + (10.0 + 1.0 / 12.0)) / 15.0;
        assert_eq!(vx, expect);
        assert_eq!(vy, 0.0);
        assert_eq!(rot, 0.1);
        // A slow velocity gets the 1.05 boost.
        let ((sx, _), _) = solar_flare_steer((1.0, 0.0), (1.0, 0.0), 1, 0.0);
        let blended = (1.0 * 14.0 + (1.0 + 1.0 / 12.0)) / 15.0;
        assert_eq!(sx, blended * 1.05);
        // The NaN aim falls back to (direction, 0).
        let ((nx, ny), _) = solar_flare_steer((10.0, 0.0), (f32::NAN, 0.0), -1, 0.0);
        let expect = (10.0 * 14.0 + (-1.0) * (10.0 + 1.0 / 12.0)) / 15.0;
        assert_eq!(nx, expect);
        assert_eq!(ny, 0.0);
    }

    /// `NPC.cs:21407-21449` — the launch fires ONCE (the target == 255
    /// gate), aims at the player with the C#'s multiply order, and the
    /// velocity never re-aims.
    #[test]
    fn s1_d_3_s9_the_sphere_launches_once_via_target_closest() {
        let w = arena();
        let mut player = player_at(&w, 100.0 * 16.0, 110.0 * 16.0);
        let mut ball = Npc::new(NpcKind::ChaosBall, 80.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        assert_eq!(ball.target, 255);
        let mut rng = UnifiedRandom::new(0);
        step_solo(&mut ball, &w, &player, &mut rng);
        assert_eq!(ball.target, 0, "TargetClosest committed the slot");
        let speed = (ball.vx * ball.vx + ball.vy * ball.vy).sqrt();
        assert!((speed - 6.0).abs() < 0.01, "speed {speed}");
        assert!(ball.vx > 0.0 && ball.vy > 0.0, "aims right/down");
        let v = (ball.vx, ball.vy);
        // Move the player: the launched ball never re-aims.
        player.x = 60.0 * 16.0;
        for _ in 0..50 {
            step_solo(&mut ball, &w, &player, &mut rng);
        }
        assert_eq!((ball.vx, ball.vy), v);
        // The Burning Sphere's 5 (`:21411-21414`).
        let mut sphere = Npc::new(NpcKind::BurningSphere, 80.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        step_solo(&mut sphere, &w, &player, &mut rng);
        let speed = (sphere.vx * sphere.vx + sphere.vy * sphere.vy).sqrt();
        assert!((speed - 5.0).abs() < 0.01, "burning speed {speed}");
    }

    /// `NPC.cs:21442-21445` — the coincident-centre guard: dist <= 0 → 1,
    /// so the launch writes zero velocity instead of NaN.
    #[test]
    fn s1_d_3_s9_the_coincident_launch_guard() {
        let w = arena();
        // The player's centre exactly on the ball's centre.
        let ball_x = 80.0 * 16.0;
        let ball_y = 100.0 * 16.0;
        let mut player = player_at(
            &w,
            ball_x + 8.0 - crate::player::WIDTH as f32 / 2.0,
            ball_y + 8.0 - crate::player::HEIGHT as f32 / 2.0,
        );
        player.dead = false;
        let mut ball = Npc::new(NpcKind::ChaosBall, ball_x, ball_y, 1, 0.0);
        let mut rng = UnifiedRandom::new(0);
        step_solo(&mut ball, &w, &player, &mut rng);
        assert_eq!(ball.vx, 0.0);
        assert_eq!(ball.vy, 0.0);
    }

    /// `NPC.cs:21496` + `:21519-21521` + `:21568` — the per-tick state:
    /// EncourageDespawn(100), the Chaos Ball's alpha 255, the 0.4 spin.
    #[test]
    fn s1_d_3_s9_the_per_tick_state_writes() {
        let w = arena();
        let player = player_at(&w, 100.0 * 16.0, 110.0 * 16.0);
        let mut ball = Npc::new(NpcKind::ChaosBall, 80.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        let mut rng = UnifiedRandom::new(0);
        step_solo(&mut ball, &w, &player, &mut rng);
        assert_eq!(ball.despawn_timer, 100, "EncourageDespawn(100)");
        assert_eq!(ball.alpha, 255, "the 30/665 dust-loop alpha write");
        let rot = ball.rotation;
        assert!((rot - 0.4 * ball.facing as f32).abs() < 1e-6);
        step_solo(&mut ball, &w, &player, &mut rng);
        assert!((ball.rotation - 2.0 * rot).abs() < 1e-6, "the spin accumulates");
        // The Burning Sphere keeps its SetDefaults alpha 100.
        let mut sphere = Npc::new(NpcKind::BurningSphere, 80.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        let alpha0 = sphere.alpha;
        step_solo(&mut sphere, &w, &player, &mut rng);
        assert_eq!(sphere.alpha, alpha0);
    }

    /// The RNG-order witness (W4-d52 re-derivation): the dust-wired arm
    /// draws through the live pool. The oracle replays the EXACT draw
    /// sequence from the same seed: per tick, per dust, `NewDust`'s six
    /// default-block rolls (`Dust.cs:170-175,190` — the empty/low pool
    /// takes no pressure-band draws, ChildSafety is disabled), plus the
    /// type-6 tweak's one `Next(-10,6)` (`Dust.cs:214-220`) and the Water
    /// Sphere's `Next(5)` gate + conditional 4th dust. Any extra or
    /// reordered draw desynchronizes the probe.
    #[test]
    fn s1_d_3_s9_the_sphere_arm_draws_match_the_oracle() {
        let w = arena();
        let player = player_at(&w, 100.0 * 16.0, 110.0 * 16.0);
        let ctx = tr_core::particles::SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: glam::Vec2::ZERO,
            screen_size: glam::Vec2::new(w.width as f32 * TILE_PX, w.height as f32 * TILE_PX),
            max_dust_to_draw: 6000,
        };
        // One NewDust's six default-block draws (the pool stays under the
        // 50% pressure band for 100 ticks: <= 800 active slots). The
        // sphere boxes are 16x16 (`data.rs`); the Water Sphere's inset
        // dusts ride 12x12 (`:21538`'s width - 2*2).
        let new_dust_draws = |r: &mut UnifiedRandom, dust_type: i32, box_px: i32| {
            let n = box_px.max(5) - 4;
            r.next_max(n); // position X (`Dust.cs:170`)
            r.next_max(n); // position Y (`:171`)
            r.next_range(-20, 21);
            r.next_range(-20, 21);
            r.next_max(3);
            r.next_range(-20, 21);
            if dust_type == 6 {
                r.next_range(-10, 6); // the type-6 tweak (`Dust.cs:214-220`)
            }
        };
        for kind in [NpcKind::ChaosBall, NpcKind::BurningSphere, NpcKind::WaterSphere] {
            let mut rng = UnifiedRandom::new(123);
            let mut probe = UnifiedRandom::new(123);
            let mut dust = tr_core::particles::DustPool::new();
            let mut ball = Npc::new(kind, 80.0 * 16.0, 100.0 * 16.0, 1, 0.0);
            for tick in 0..100 {
                sphere_family_step(&mut ball, &w, &player, Style9Flags::default(), &mut rng, &mut dust, &ctx);
                // The oracle: the C#'s per-tick draw census.
                for _iter in 0..2 {
                    match kind {
                        NpcKind::ChaosBall => {
                            for _ in 0..2 {
                                new_dust_draws(&mut probe, 27, 16);
                            }
                        }
                        NpcKind::WaterSphere => {
                            for _ in 0..3 {
                                new_dust_draws(&mut probe, 172, 12);
                            }
                            if probe.next_max(5) == 0 {
                                new_dust_draws(&mut probe, 172, 12);
                            }
                        }
                        _ => new_dust_draws(&mut probe, 6, 16),
                    }
                }
                assert_eq!(rng.peek(), probe.peek(), "{kind:?} stream @{tick}");
            }
            assert_ne!(rng.peek(), UnifiedRandom::new(123).peek(), "{kind:?} the dust drew");
        }
    }

    /// W4-d52 — the post-spawn field mirror: replay the first tick's rolls
    /// from the same seed and assert slot 0's exact writes — the Chaos
    /// Ball's `:21525-21528` noGravity + *0.3 - v*0.2, the Water Sphere's
    /// `:21539-21541` inset box + *0.1 + v*0.5, the Burning Sphere's
    /// `:21563-21565` noGravity + per-axis *0.3 on top of the type-6
    /// tweak's Y overwrite (`Dust.cs:214-218`).
    #[test]
    fn w4_d52_sphere_dust_writes_the_post_fields() {
        let w = arena();
        let player = player_at(&w, 100.0 * 16.0, 110.0 * 16.0);
        let ctx = tr_core::particles::SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: glam::Vec2::ZERO,
            screen_size: glam::Vec2::new(w.width as f32 * TILE_PX, w.height as f32 * TILE_PX),
            max_dust_to_draw: 6000,
        };
        for kind in [NpcKind::ChaosBall, NpcKind::BurningSphere, NpcKind::WaterSphere] {
            let mut ball = Npc::new(kind, 80.0 * 16.0, 100.0 * 16.0, 1, 0.0);
            let mut rng = UnifiedRandom::new(7);
            let mut probe = UnifiedRandom::new(7);
            let mut dust = tr_core::particles::DustPool::new();
            sphere_family_step(&mut ball, &w, &player, Style9Flags::default(), &mut rng, &mut dust, &ctx);
            // The launch velocity is dust-loop-constant; the integration
            // moved the NPC AFTER the loop, so undo it for the spawn box.
            let (vx, vy) = (ball.vx, ball.vy);
            let (pre_x, pre_y) = (ball.x - vx, ball.y - vy);
            let (n, ty) = match kind {
                NpcKind::ChaosBall => (12, 27),
                NpcKind::WaterSphere => (8, 172),
                _ => (12, 6),
            };
            let px = probe.next_max(n);
            let py = probe.next_max(n);
            let rvx = probe.next_range(-20, 21) as f32;
            let rvy = probe.next_range(-20, 21) as f32;
            let _frame = probe.next_max(3);
            let rs = probe.next_range(-20, 21) as f32;
            let d = dust.slot(0);
            assert!(d.active, "{kind:?} slot 0 spawned");
            assert_eq!(d.dust_type, ty);
            match kind {
                NpcKind::ChaosBall => {
                    assert!(d.no_gravity);
                    assert_eq!(d.position.x, pre_x + px as f32 + 4.0);
                    assert_eq!(d.position.y, pre_y + 2.0 + py as f32 + 4.0);
                    assert_eq!(d.velocity.x, (rvx * 0.1 + vx * 0.2) * 0.3 - vx * 0.2);
                    assert_eq!(d.velocity.y, (rvy * 0.1 + vy * 0.2) * 0.3 - vy * 0.2);
                    assert_eq!(d.scale, (1.0 + rs * 0.01) * 1.3);
                }
                NpcKind::WaterSphere => {
                    assert!(d.no_gravity);
                    // Iteration 0: the num132/num133 offsets are zero.
                    assert_eq!(d.position.x, pre_x + 2.0 + px as f32 + 4.0);
                    assert_eq!(d.position.y, pre_y + 2.0 + py as f32 + 4.0);
                    assert_eq!(d.velocity.x, rvx * 0.1 * 0.1 + vx * 0.5);
                    assert_eq!(d.velocity.y, rvy * 0.1 * 0.1 + vy * 0.5);
                    assert_eq!(d.scale, (1.0 + rs * 0.01) * 1.2);
                }
                _ => {
                    let rt = probe.next_range(-10, 6) as f32;
                    assert!(d.no_gravity);
                    assert_eq!(d.velocity.x, (rvx * 0.1 + vx * 0.2) * 0.3 * 0.3);
                    assert_eq!(d.velocity.y, rt * 0.1 * 0.3, "the tweak's Y overwrite");
                    assert_eq!(d.scale, (1.0 + rs * 0.01) * 2.0 * 0.7);
                }
            }
        }
    }
}
