//! W4-d5 (aiStyle-42) — the CrystalLeaf (226), `Projectile.AI`'s
//! `else if (this.aiStyle == 42)` block (`Projectile.cs:27793-27859`),
//! ported 1:1 as a PURE machine (code-only mode: no compiler/tests were
//! run on this branch; the witness tests below are written, not executed).
//!
//! ## Census (research/156 row 42 — was 26d, registry §3)
//!
//! ONE vanilla member: CrystalLeaf 226 (`ProjectileID.cs:1006`,
//! `SetDefaults` `Projectile.cs:3111-3123`: `netImportant`, 22×42,
//! friendly, `tileCollide = false`, `penetrate = -1`, `timeLeft *= 5`,
//! `light = 0.4f`, `ignoreWater = true`). The port has no `proj_data` row
//! and no `player.crystalLeaf`/`crystalLeafCooldown` fields (buff 60's
//! `PetFlag::CrystalLeaf` lands in `buff_effects_misc.rs`), so this module
//! lands UNWIRED (the `npc_ai_style25` precedent): the owner state rides a
//! snapshot struct (the `SpearOwner` pattern) the wiring dispatch fills.
//!
//! ## The machine (verbatim)
//!
//! * `:27795-27799` — `!player.crystalLeaf` → `Kill()` + return.
//! * `:27800-27813` — the orbit placement: X = owner Centre.X − width/2
//!   (INTEGER half-box), Y = Centre.Y − height/2 + gfxOffY − 60; then
//!   `+= netOffset`; `gravDir == -1` adds 120 to Y and pins rotation 3.14,
//!   else rotation 0; BOTH axes truncate to int.
//! * `:27814` — `scale = ((int)mouseTextColor / 200f - 0.35f) * 0.2f + 0.95f`.
//! * `:27815-27818` — non-local owner or `crystalLeafCooldown != 0` → return.
//! * `:27819-27836` — the scan: chaseable NPCs within Manhattan 700 of the
//!   leaf's INTEGER-half-box centre with a clear `Collision.CanHit` line
//!   (full boxes); the closest wins (strict `<`, first on ties).
//! * `:27837-27858` — the shot: the aim starts at 12f toward
//!   `(position − Centre)` (i.e. the up-left half-box vector, `:27840-27847`
//!   — verbatim, including that it does NOT aim at the NPC); the
//!   `GetChaseResults(Centre, 12 * 180, npc.Center, npc.velocity)` solve
//!   overrides it when the interception happens within 180 ticks
//!   (`ChaserVelocity / 180`). Spawn 227 at `(Centre.X − 4, Centre.Y)` with
//!   `Player.crystalLeafDamage` = 100 / `crystalLeafKB` = 10
//!   (`Player.cs:57098-57099`); cooldown = 40.
//!
//! Movement: aiStyle 42 is not in `UpdatePosition`'s skip list, but the
//! placement re-pins the position every tick — the wiring caller's generic
//! move adds the (zero) velocity on top, faithful.
//!
//! ## RNG by construction
//!
//! The arm draws NO `Main.rand` (`:27793-27859` contains no Next/NextFloat;
//! neither does `CanBeChasedBy`/`CanHit`/`GetChaseResults`). The fn takes no
//! RNG parameter; `w4_d5_s42_the_arm_draws_no_rng` pins a `UnifiedRandom`
//! state across the idle, scan and fire ticks (written, not run).
#![allow(dead_code)] // UNWIRED (W4-d5): the 226 row + the player fields + the dispatch arm are the wiring step

use crate::npc::Npc;
use crate::projectile::{Projectile, can_hit, chaseable, get_chase_results};
use glam::Vec2;
use tr_core::World;

/// `ProjectileID.CrystalLeaf` (`ProjectileID.cs:1006`).
pub const CRYSTAL_LEAF: u16 = 226;
/// `ProjectileID.CrystalLeafShot` (`ProjectileID.cs:1008`) — the spawn kind.
pub const CRYSTAL_LEAF_SHOT: u16 = 227;
/// `Player.crystalLeafDamage` (`Player.cs:838`, set 100 at `:57098`).
pub const CRYSTAL_LEAF_DAMAGE: i32 = 100;
/// `Player.crystalLeafKB` (`Player.cs:840`, set 10 at `:57099`) — the C#
/// field is `int`; `NewProjectile` widens it to the float knockback.
pub const CRYSTAL_LEAF_KB: f32 = 10.0;

/// The `SetDefaults` row (`Projectile.cs:3111-3123`):
/// (width, height, timeLeft = 3600 * 5, penetrate, light).
pub const LEAF_DEFAULTS: (f32, f32, i32, i32, f32) = (22.0, 42.0, 18000, -1, 0.4);

/// The owner state the arm reads off `Main.player[owner]` each tick
/// (`:27795-27815`), snapshotted by the wiring dispatch (the `SpearOwner`
/// pattern — the slot loop cannot borrow the player).
#[derive(Clone, Copy, Debug)]
pub struct CrystalLeafOwner {
    /// `player.crystalLeaf` — the buff-60 flag.
    pub crystal_leaf: bool,
    /// `player.Center`.
    pub center: Vec2,
    /// `player.gfxOffY`.
    pub gfx_off_y: f32,
    /// `player.netOffset`.
    pub net_offset: Vec2,
    /// `player.gravDir` (1 or −1).
    pub grav_dir: f32,
    /// `player.crystalLeafCooldown` — read AND written (the 40-tick re-arm).
    pub cooldown: i32,
    /// `Main.mouseTextColor` (the 190..255 pulse, `app/flow.rs:2403`).
    pub mouse_text_color: u8,
}

/// One queued 227 shot (`Projectile.NewProjectile` `:27856`): the C# spawn
/// arguments — the wiring spawn subtracts the row's half-box
/// (`Projectile.cs:10876-10877`, the `SpearSubShot` convention).
#[derive(Clone, Copy, Debug)]
pub struct LeafShot {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

/// `Kill()` (`Projectile.cs:70644-70648`) — the sibling-module deactivation
/// (the `projectile_ai_spear::kill` precedent; `Projectile::kill` is private).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// `AI()`'s aiStyle-42 arm (`Projectile.cs:27793-27859`), verbatim.
/// `local_owner` is `Main.myPlayer` (the port's slot-owner comparison).
pub fn crystal_leaf_step(
    p: &mut Projectile,
    world: &World,
    npcs: &[Npc],
    owner: &mut CrystalLeafOwner,
    local_owner: u8,
) -> Option<LeafShot> {
    // `:27795-27799` — the buff-off kill.
    if !owner.crystal_leaf {
        kill(p);
        return None;
    }
    // `:27800-27802` — the orbit placement (INTEGER half-box, `width / 2`).
    p.pos.x = owner.center.x - (p.w as i32 / 2) as f32;
    p.pos.y = owner.center.y - (p.h as i32 / 2) as f32 + owner.gfx_off_y - 60.0;
    p.pos += owner.net_offset;
    // `:27803-27811` — the gravity flip.
    if owner.grav_dir == -1.0 {
        p.pos.y += 120.0;
        p.rot = 3.14;
    } else {
        p.rot = 0.0;
    }
    // `:27812-27813` — the int truncation.
    p.pos.x = p.pos.x as i32 as f32;
    p.pos.y = p.pos.y as i32 as f32;
    // `:27814` — the mouse-pulse scale.
    p.scale = (owner.mouse_text_color as i32 as f32 / 200.0 - 0.35) * 0.2 + 0.95;
    // `:27815-27818` — the owner + cooldown gate.
    if p.owner != local_owner || owner.cooldown != 0 {
        return None;
    }
    // `:27819-27836` — the closest-chaseable scan. The C# centres use
    // INTEGER half-boxes (`(float)(width / 2)`, `:27827-27829`).
    let x2 = p.pos.x;
    let y2 = p.pos.y;
    let mut num307 = 700.0f32;
    let mut target: Option<usize> = None;
    let lc = Vec2::new(p.pos.x + (p.w as i32 / 2) as f32, p.pos.y + (p.h as i32 / 2) as f32);
    for (i, n) in npcs.iter().enumerate() {
        if !chaseable(n) {
            continue;
        }
        let (nw, nh) = n.size();
        let nc = Vec2::new(n.x + (nw as i32 / 2) as f32, n.y + (nh as i32 / 2) as f32);
        let num311 = (lc.x - nc.x).abs() + (lc.y - nc.y).abs();
        if num311 < num307 && can_hit(world, p.pos.x, p.pos.y, p.w, p.h, n.x, n.y, nw, nh) {
            num307 = num311;
            target = Some(i);
        }
    }
    let Some(t) = target else { return None };
    // `:27839-27847` — the 12f aim at `(position − Centre)` (the verbatim
    // up-left half-box vector, NOT the NPC direction).
    let num312 = 12.0f32;
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let mut num313 = x2 - center.x;
    let mut num314 = y2 - center.y;
    let num315 = (num313 * num313 + num314 * num314).sqrt();
    let scale = num312 / num315;
    num313 *= scale;
    num314 *= scale;
    // `:27848-27855` — the interception override (12 × 180 cruise, ≤180 ticks).
    let num317 = 180;
    let n = &npcs[t];
    let (nw, nh) = n.size();
    let npc_center = Vec2::new(n.x + nw * 0.5, n.y + nh * 0.5);
    if let Some(chase) = get_chase_results(
        center,
        num312 * num317 as f32,
        npc_center,
        Vec2::new(n.vx, n.vy),
    ) {
        if chase.interception_time <= 180.0 {
            let v = chase.chaser_velocity / num317 as f32;
            num313 = v.x;
            num314 = v.y;
        }
    }
    // `:27856-27857` — the spawn + the 40-tick cooldown.
    owner.cooldown = 40;
    Some(LeafShot { x: center.x - 4.0, y: center.y, vx: num313, vy: num314 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::UnifiedRandom;

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    fn leaf() -> Projectile {
        Projectile {
            active: true,
            kind: CRYSTAL_LEAF,
            owner: 0,
            w: LEAF_DEFAULTS.0,
            h: LEAF_DEFAULTS.1,
            time_left: LEAF_DEFAULTS.2,
            penetrate: LEAF_DEFAULTS.3,
            max_penetrate: LEAF_DEFAULTS.3,
            tile_collide: false,
            ignore_water: true,
            light: LEAF_DEFAULTS.4,
            ..Default::default()
        }
    }

    fn owner_at(cx: f32, cy: f32) -> CrystalLeafOwner {
        CrystalLeafOwner {
            crystal_leaf: true,
            center: Vec2::new(cx, cy),
            gfx_off_y: 0.0,
            net_offset: Vec2::ZERO,
            grav_dir: 1.0,
            cooldown: 0,
            mouse_text_color: 255,
        }
    }

    fn zombie_at(cx: f32, cy: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, 0.0, 0.0, 1, 0.0);
        n.max_hp = 40;
        n.hp = 40;
        let (w, h) = n.size();
        n.x = cx - w * 0.5;
        n.y = cy - h * 0.5;
        n
    }

    /// `:27795-27799` — the buff-off kill: dead slot, no placement.
    #[test]
    fn w4_d5_s42_the_buff_off_kills_the_leaf() {
        let w = air();
        let mut p = leaf();
        p.pos = Vec2::new(500.0, 500.0);
        let mut o = owner_at(1000.0, 1000.0);
        o.crystal_leaf = false;
        assert!(crystal_leaf_step(&mut p, &w, &[], &mut o, 0).is_none());
        assert!(!p.active);
        assert_eq!(p.time_left, 0);
        assert_eq!(p.pos, Vec2::new(500.0, 500.0), "Kill returns before the placement");
    }

    /// `:27800-27814` — the placement: Centre − (11, 21) − 60, truncated;
    /// scale at mouseTextColor 255 = (255/200 − 0.35) × 0.2 + 0.95 = 1.135.
    #[test]
    fn w4_d5_s42_the_orbit_placement_truncates() {
        let w = air();
        let mut p = leaf();
        let mut o = owner_at(1000.7, 1000.9);
        o.gfx_off_y = 2.5;
        o.net_offset = Vec2::new(0.4, 0.4);
        let _ = crystal_leaf_step(&mut p, &w, &[], &mut o, 0);
        // X: 1000.7 − 11 + 0.4 = 990.1 → 990; Y: 1000.9 − 21 + 2.5 − 60 + 0.4 = 922.8 → 922.
        assert_eq!(p.pos, Vec2::new(990.0, 922.0));
        assert_eq!(p.rot, 0.0);
        assert!((p.scale - 1.135).abs() < 1e-5);
    }

    /// `:27803-27807` — gravDir −1 adds 120 and pins rotation 3.14.
    #[test]
    fn w4_d5_s42_the_gravity_flip_mirrors_the_orbit() {
        let w = air();
        let mut p = leaf();
        let mut o = owner_at(1000.0, 1000.0);
        o.grav_dir = -1.0;
        let _ = crystal_leaf_step(&mut p, &w, &[], &mut o, 0);
        // Y: 1000 − 21 − 60 + 120 = 1039.
        assert_eq!(p.pos, Vec2::new(989.0, 1039.0));
        assert_eq!(p.rot, 3.14);
    }

    /// `:27815-27818` — a running cooldown blocks the scan; a non-local
    /// owner blocks it too (the placement still runs).
    #[test]
    fn w4_d5_s42_the_cooldown_and_owner_gates_block_the_shot() {
        let w = air();
        let mut p = leaf();
        let mut o = owner_at(1000.0, 1000.0);
        o.cooldown = 12;
        let npcs = [zombie_at(1000.0, 940.0)];
        assert!(crystal_leaf_step(&mut p, &w, &npcs, &mut o, 0).is_none());
        assert_eq!(o.cooldown, 12, "the gate does not touch the cooldown");
        o.cooldown = 0;
        p.owner = 1;
        assert!(crystal_leaf_step(&mut p, &w, &npcs, &mut o, 0).is_none());
    }

    /// `:27837-27857` — a stationary NPC in range fires: the interception
    /// solve at 2160 cruise lands within 180 ticks, so the velocity is
    /// `ChaserVelocity / 180` — 12f toward the NPC centre — and the spawn
    /// is at (Centre.X − 4, Centre.Y); the cooldown re-arms to 40.
    #[test]
    fn w4_d5_s42_the_stationary_target_fires_at_12f() {
        let w = air();
        let mut p = leaf();
        let mut o = owner_at(1000.0, 1000.0);
        // Leaf centre after placement: (1000, 940). NPC centre 100 px right.
        let npcs = [zombie_at(1100.0, 940.0)];
        let shot = crystal_leaf_step(&mut p, &w, &npcs, &mut o, 0).expect("fires");
        assert_eq!((shot.x, shot.y), (996.0, 940.0));
        assert!((shot.vx - 12.0).abs() < 1e-3, "vx {}", shot.vx);
        assert!(shot.vy.abs() < 1e-3, "vy {}", shot.vy);
        assert_eq!(o.cooldown, 40);
    }

    /// The Manhattan-700 bound is strict and the scan keeps the CLOSEST.
    #[test]
    fn w4_d5_s42_the_scan_keeps_the_closest_in_700() {
        let w = air();
        let mut p = leaf();
        let mut o = owner_at(1000.0, 1000.0);
        // Leaf centre (1000, 940): far NPC at dx 600, near at dx 100.
        let npcs = [zombie_at(1600.0, 940.0), zombie_at(1100.0, 940.0)];
        let shot = crystal_leaf_step(&mut p, &w, &npcs, &mut o, 0).expect("fires");
        assert!((shot.vx - 12.0).abs() < 1e-3, "aims at the NEAR npc");
        // Out of range: dx 701 > 700 → no shot.
        let mut p = leaf();
        o.cooldown = 0;
        let npcs = [zombie_at(1701.0, 940.0)];
        assert!(crystal_leaf_step(&mut p, &w, &npcs, &mut o, 0).is_none());
    }

    /// RNG-free by construction: the idle, scan and fire ticks leave the
    /// stream untouched (the module-doc contract).
    #[test]
    fn w4_d5_s42_the_arm_draws_no_rng() {
        let w = air();
        let mut rng = UnifiedRandom::new(42);
        let mut control = UnifiedRandom::new(42);
        let npcs = [zombie_at(1100.0, 940.0)];
        for cooldown in [0, 40, 0] {
            let mut p = leaf();
            let mut o = owner_at(1000.0, 1000.0);
            o.cooldown = cooldown;
            let _ = crystal_leaf_step(&mut p, &w, &npcs, &mut o, 0);
        }
        assert_eq!(rng.next_range(0, 1000), control.next_range(0, 1000));
    }
}