//! The projectile pool — the `Main.projectile[1000]` engine (`Projectile.cs`),
//! with the shuriken (`Projectile.cs` type 3, aiStyle 2) as its first AI family.
//!
//! Source map (line numbers into `decompiled/Terraria/`):
//! - pool / spawn: `NewProjectileSetup` `:10823-10880`, `FindOldestProjectile`
//!   `:10811-10821`, the 1000-slot array `Main.cs:66992`
//! - per-tick bookkeeping: `Projectile.Update` `:17199-17208` (`timeLeft--` →
//!   `Kill`, `penetrate == 0` → `Kill`) with the AI sub-step count
//!   `1 + extraUpdates` (`:16668`)
//! - the PVE damage walk: `Damage` `:12487-12511`, `Damage_PVE` `:12562-12600`
//!   (the target-selection immunity gates), `Damage_PVE_Inner` `:12600+`
//!   (the per-owner gate `:12620-12624`) and the on-hit immunity application
//!   `:14087-14119`
//! - the ID-static system: `IsNPCIndexImmuneToProjectileType` `:604-607`
//!   (a hit is allowed when the stored stamp is `<= GameUpdateCount`), the
//!   stamp write `:13972`
//! - defaults: `timeLeft = 3600` (`:787`), `penetrate = 1`,
//!   `localNPCHitCooldown = -2` (`:464`), `idStaticNPCHitCooldown = -1`
//!   (`:466`), `immunityIdentity = type` (`:782`)
//!
//! NPC-side per-owner i-frames: `NPC.immune[256]` (decremented once per NPC
//! update, `NPC.cs:90454-90459`), stored on [`crate::npc::Npc`].
//!
//! Multiplayer bookkeeping (`key`, `slotGenerations`, net spam) is out of scope
//! for the single-player port — recorded in ANALYSIS "Projectile pool
//! (M4-PROJ-1)".

use std::collections::HashMap;

use glam::Vec2;
use tr_core::lighting::LightMap;
use tr_core::{TILE_PX, World};
use tr_render::sprite::SpriteRenderer;

use crate::assets;
use crate::npc::Npc;

/// The port's internal projectile kind table (the C# `ProjectileID` mapping
/// grows with M4-PROJ-2..11; the shuriken keeps its registry id).
pub const SHURIKEN: u16 = 1;
/// `Main.projectile[1001]` — 1000 usable slots (`Projectile.cs:66992`,
/// `GetNextSlot` returns 1000 when full, `:10793`).
pub const MAX_PROJECTILES: usize = 1000;
/// `timeLeft`'s SetDefaults value (`Projectile.cs:787`).
pub const TIME_LEFT_DEFAULT: i32 = 3600;
/// The per-owner immunity a default penetrating hit applies
/// (`Projectile.cs:14117`).
const OWNER_IMMUNE_TICKS: i32 = 10;
const SIZE: f32 = 6.0;

/// One pool slot (`Projectile.cs:296-470`, the engine-relevant subset).
#[derive(Clone, Debug)]
pub struct Projectile {
    pub active: bool,
    pub kind: u16,
    /// `Projectile.owner`; 255 = unset.
    pub owner: u8,
    pub pos: Vec2,
    pub vel: Vec2,
    pub rot: f32,
    pub damage: i32,
    pub knock_back: f32,
    /// The crit decision rolled at throw time (the item stream lives in the
    /// app, not in the projectile step).
    pub crit: bool,
    /// `timeLeft` — ticks down once per update, `Kill()` at `<= 0` (`:17201`).
    pub time_left: i32,
    /// `maxPenetrate` / `penetrate`; 1 = single hit.
    pub max_penetrate: i32,
    pub penetrate: i32,
    /// `usesLocalNPCImmunity` + `localNPCHitCooldown` (default −2, `:464`).
    pub uses_local_immunity: bool,
    pub local_hit_cooldown: i32,
    /// `usesIDStaticNPCImmunity` + `idStaticNPCHitCooldown` (default −1, `:466`).
    pub uses_id_static_immunity: bool,
    pub id_static_hit_cooldown: i32,
    /// `extraUpdates`: the AI runs `1 + extraUpdates` times per tick (`:16668`).
    pub extra_updates: u32,
    bounces: u8,
    /// The tile-fall mode (`AI_010`'s `ai[0] == 1f`): the projectile carries a
    /// falling tile kind (`spawn_falling`), falls with `velocity.Y += 0.41f`
    /// per tick and places the tile on landing.
    pub falling_tile: Option<u16>,
}

impl Default for Projectile {
    fn default() -> Self {
        Self {
            active: false,
            kind: 0,
            owner: 255,
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            rot: 0.0,
            damage: 0,
            knock_back: 0.0,
            crit: false,
            time_left: TIME_LEFT_DEFAULT,
            max_penetrate: 1,
            penetrate: 1,
            uses_local_immunity: false,
            local_hit_cooldown: -2,
            uses_id_static_immunity: false,
            id_static_hit_cooldown: -1,
            extra_updates: 0,
            bounces: 0,
            falling_tile: None,
        }
    }
}

impl Projectile {
    fn shuriken(x: f32, y: f32, vx: f32, vy: f32, crit: bool) -> Self {
        Self {
            active: true,
            kind: SHURIKEN,
            owner: 0,
            pos: Vec2::new(x, y),
            vel: Vec2::new(vx, vy),
            damage: 10,
            knock_back: 12.0,
            crit,
            // The placeholder's `age > 2 s || bounces > 6` lifetime is replaced
            // by the faithful `timeLeft`; `penetrate` stays the vanilla 1.
            ..Default::default()
        }
    }

    /// The aiStyle-2 physics (the first pass's body, unchanged): two half-steps
    /// so 11 px/tick cannot tunnel, 0.6-damped bounces.
    fn ai_shuriken(&mut self, world: &World) {
        self.rot += if self.vel.x >= 0.0 { 0.42 } else { -0.42 };
        for _ in 0..2 {
            let step = self.vel * 0.5;
            let nx = self.pos.x + step.x;
            if crate::player::collides_rect(world, nx, self.pos.y, SIZE, SIZE) {
                self.vel.x = -self.vel.x * 0.6;
                self.bounces += 1;
            } else {
                self.pos.x = nx;
            }
            let ny = self.pos.y + step.y;
            if crate::player::collides_rect(world, self.pos.x, ny, SIZE, SIZE) {
                self.vel.y = -self.vel.y * 0.6;
                self.bounces += 1;
            } else {
                self.pos.y = ny;
            }
        }
    }

    /// The per-slot draw (unchanged from the first pass).
    pub fn draw(
        &self,
        r: &mut SpriteRenderer,
        lookups: &assets::Lookups,
        light: &LightMap,
        camera: Vec2,
        zoom: f32,
        screen: Vec2,
    ) {
        let Some(uv) = lookups.shuriken_uv() else {
            return;
        };
        let tx = (self.pos.x / TILE_PX) as i32;
        let ty = (self.pos.y / TILE_PX) as i32;
        let c = if light.w > 0 && light.h > 0 {
            light.get(
                tx.clamp(light.x0, light.x0 + light.w - 1),
                ty.clamp(light.y0, light.y0 + light.h - 1),
            )
        } else {
            [1.0; 3]
        };
        let pos = [
            self.pos.x - (camera.x - screen.x * 0.5 / zoom),
            self.pos.y - (camera.y - screen.y * 0.5 / zoom),
        ];
        r.push_ui_sprite_rot(
            pos,
            [SIZE * zoom, SIZE * zoom],
            uv,
            [c[0], c[1], c[2], 1.0],
            self.rot,
        );
    }
}

/// The 1000-slot pool with the shared immunity state.
#[derive(Debug)]
pub struct ProjectilePool {
    slots: Vec<Projectile>,
    /// `localNPCImmunity[slot][npc]` — flattened `MAX_PROJECTILES × 256`
    /// (the C# per-projectile array is `Main.maxNPCs`-sized). The index is the
    /// NPC's position in the app's `Vec<Npc>`: stable between death compactions
    /// (documented in ANALYSIS — the C# indexes stable NPC slots).
    local_immunity: Vec<i32>,
    /// `perIDStaticNPCImmunity[immunityIdentity = type, npc]` → the update
    /// count after which a hit is allowed again (`:13972`, check `:604-607`).
    id_static: HashMap<(u16, u16), u64>,
    /// `Main.GameUpdateCount`.
    update_count: u64,
}

impl Default for ProjectilePool {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectilePool {
    pub fn new() -> Self {
        Self {
            slots: vec![Projectile::default(); MAX_PROJECTILES],
            local_immunity: vec![0; MAX_PROJECTILES * 256],
            id_static: HashMap::new(),
            update_count: 0,
        }
    }

    pub fn slots(&self) -> &[Projectile] {
        &self.slots
    }

    #[cfg(test)]
    fn slots_mut(&mut self) -> &mut [Projectile] {
        &mut self.slots
    }

    pub fn active_count(&self) -> usize {
        self.slots.iter().filter(|p| p.active).count()
    }

    /// `Projectile.NewProjectile` → `NewProjectileSetup` (`:10823-10880`): the
    /// first inactive slot; when the pool is full, recycle
    /// `FindOldestProjectile` (`:10811-10821` — lowest `timeLeft`, which puts
    /// dead slots first because `Kill` leaves them at 0). Returns `None` for
    /// the C#'s "1000" failure sentinel.
    pub fn spawn(
        &mut self,
        kind: u16,
        owner: u8,
        pos: Vec2,
        vel: Vec2,
        damage: i32,
        knock_back: f32,
        crit: bool,
    ) -> Option<usize> {
        let mut slot = None;
        for (i, p) in self.slots.iter().enumerate() {
            if !p.active {
                slot = Some(i);
                break;
            }
        }
        let slot = match slot {
            Some(i) => i,
            None => {
                let mut oldest = 0usize;
                let mut lowest = i32::MAX;
                for (i, p) in self.slots.iter().enumerate() {
                    if p.time_left < lowest {
                        oldest = i;
                        lowest = p.time_left;
                    }
                }
                oldest
            }
        };
        self.slots[slot] = Projectile {
            active: true,
            kind,
            owner,
            pos,
            vel,
            damage,
            knock_back,
            crit,
            ..Default::default()
        };
        // A spawned slot starts with clean local immunity.
        let base = slot * 256;
        self.local_immunity[base..base + 256].fill(0);
        Some(slot)
    }

    /// Configure the immunity system of a just-spawned slot (the C# sets these
    /// in the per-type `SetDefaults`).
    pub fn set_immunity(
        &mut self,
        slot: usize,
        max_penetrate: i32,
        uses_local: bool,
        local_hit_cooldown: i32,
        uses_id_static: bool,
        id_static_hit_cooldown: i32,
    ) {
        let p = &mut self.slots[slot];
        p.max_penetrate = max_penetrate;
        p.penetrate = max_penetrate;
        p.uses_local_immunity = uses_local;
        p.local_hit_cooldown = local_hit_cooldown;
        p.uses_id_static_immunity = uses_id_static;
        p.id_static_hit_cooldown = id_static_hit_cooldown;
    }

    /// The tile-fall spawn (`SpawnFallingBlockProjectile`'s
    /// `Projectile.NewProjectile(..., 0f, 0.41f, projType, dmg, ...)` +
    /// `ai[0] = 1f`, `WorldGen.cs:86071-86073`): the 10×10 sand ball falls
    /// with the tile-fall gravity and places its tile on landing.
    pub fn spawn_falling(&mut self, tile_kind: u16, proj_kind: u16, damage: i32, x: f32, y: f32) -> Option<usize> {
        let mut slot = None;
        for (i, p) in self.slots.iter().enumerate() {
            if !p.active {
                slot = Some(i);
                break;
            }
        }
        let slot = slot?;
        self.slots[slot] = Projectile {
            active: true,
            kind: proj_kind,
            owner: 0,
            pos: Vec2::new(x, y),
            rot: 0.0,
            vel: Vec2::new(0.0, 0.41),
            damage,
            knock_back: 0.0,
            crit: false,
            time_left: tr_core::falling::TIME_LEFT_DEFAULT,
            max_penetrate: 1,
            penetrate: 1,
            uses_local_immunity: false,
            local_hit_cooldown: -2,
            uses_id_static_immunity: false,
            id_static_hit_cooldown: -1,
            extra_updates: 0,
            bounces: 0,
            falling_tile: Some(tile_kind),
        };
        let base = slot * 256;
        self.local_immunity[base..base + 256].fill(0);
        Some(slot)
    }

    /// `Projectile.Update` (`:16668-17208`): the AI sub-steps, the PVE damage
    /// walk, then the `timeLeft`/`penetrate` bookkeeping. The NPC per-owner
    /// i-frame decay runs with the NPC update (`NPC.cs:90454-90459`).
    pub fn update(&mut self, world: &mut World, npcs: &mut [Npc]) {
        self.update_count += 1;
        for i in 0..MAX_PROJECTILES {
            if !self.slots[i].active {
                continue;
            }
            // `DecrementLocalImmuneTimeCounters` (`:16674`, `:17258-17277`):
            // the PVP frames plus the slot's local-NPC cooldowns decay first.
            if self.slots[i].uses_local_immunity {
                let base = i * 256;
                for v in &mut self.local_immunity[base..base + 256] {
                    *v = (*v - 1).max(0);
                }
            }
            let sub_steps = 1 + self.slots[i].extra_updates;
            for _ in 0..sub_steps {
                if self.slots[i].kind == SHURIKEN {
                    self.slots[i].ai_shuriken(world);
                }
                // `AI_010`'s tile-fall mode (`ai[0] == 1f`): `velocity.Y +=
                // 0.41f` per tick, the position integrates the velocity (the
                // generic `position += velocity` of `Update`), and the
                // landing places the tile back.
                if self.slots[i].falling_tile.is_some() {
                    self.slots[i].vel.y =
                        (self.slots[i].vel.y + tr_core::falling::TILE_FALL_GRAVITY).min(10.0);
                    let vel = self.slots[i].vel;
                    self.slots[i].pos += vel;
                }
            }
            if let Some(tile_kind) = self.slots[i].falling_tile {
                // The landing: the bottom edge inside a solid tile → place
                // the tile back and kill the projectile.
                let p = &self.slots[i];
                if tr_core::falling::landed(world, p.pos.x, p.pos.y)
                    && tr_core::falling::can_place_back(world, p.pos.x, p.pos.y, tile_kind)
                {
                    let tx = (p.pos.x / 16.0) as i32;
                    let ty = (p.pos.y / 16.0) as i32;
                    if let Some(t) = world.tile_mut(tx, ty) {
                        t.kind = tile_kind;
                        t.flags |= tr_core::tile::FLAG_ACTIVE;
                        t.liquid = 0;
                    }
                    self.slots[i].active = false;
                    self.slots[i].time_left = 0;
                }
            }
            self.damage_pve(i, npcs);
            // `:17199-17208`.
            let (time_left, penetrate) = {
                let p = &self.slots[i];
                (p.time_left - 1, p.penetrate)
            };
            let p = &mut self.slots[i];
            p.time_left = time_left;
            if time_left <= 0 || penetrate == 0 {
                p.active = false;
                p.time_left = 0;
            }
        }
    }

    /// The target-selection gate of `Damage_PVE` (`:12577-12583`): the slot's
    /// immunity system must allow a hit on NPC `npc` right now.
    fn target_allowed(&self, slot: usize, p: &Projectile, npc: u16) -> bool {
        if !p.uses_local_immunity && !p.uses_id_static_immunity {
            return true;
        }
        if p.uses_local_immunity {
            return self.local_immunity[slot * 256 + npc as usize] == 0;
        }
        match self.id_static.get(&(p.kind, npc)) {
            Some(stamp) => *stamp <= self.update_count,
            None => true,
        }
    }

    /// `Damage_PVE` + `Damage_PVE_Inner` (`:12562-12700`) with the on-hit
    /// immunity application (`:14087-14119`). Single-hit slots bypass the
    /// per-owner gate (`flag3`, `:12620-12624`) and set no immunity
    /// (`:14112-14119`).
    fn damage_pve(&mut self, slot: usize, npcs: &mut [Npc]) {
        let (damage, owner, uses_local, uses_id_static, max_penetrate, kb, crit, center) = {
            let p = &self.slots[slot];
            if p.damage <= 0 {
                return;
            }
            (
                p.damage,
                p.owner as usize,
                p.uses_local_immunity,
                p.uses_id_static_immunity,
                p.max_penetrate,
                p.knock_back,
                p.crit,
                p.pos + Vec2::splat(SIZE * 0.5),
            )
        };
        // `keepIterating`: the walk stops once a hit consumed the projectile.
        for (idx, n) in npcs.iter_mut().enumerate() {
            if n.hp <= 0 {
                continue;
            }
            let p = &self.slots[slot];
            if !self.target_allowed(slot, p, idx as u16) {
                continue;
            }
            // `:12620-12624`: the per-owner gate, bypassed by single-hitters.
            let single_hit = max_penetrate == 1 && !uses_local && !uses_id_static;
            if !single_hit && n.immune[owner] != 0 {
                continue;
            }
            if !n.contains(center.x, center.y) {
                continue;
            }
            let landed = n.hurt_from_projectile(damage, center.x, kb, crit);
            // `:14087-14119`: the immunity application.
            if uses_local && self.slots[slot].local_hit_cooldown != -2 {
                n.immune[owner] = 0;
                self.local_immunity[slot * 256 + idx] = self.slots[slot].local_hit_cooldown;
            } else if uses_id_static && self.slots[slot].id_static_hit_cooldown > 0 {
                n.immune[owner] = 0;
                self.id_static.insert(
                    (self.slots[slot].kind, idx as u16),
                    self.update_count + self.slots[slot].id_static_hit_cooldown as u64,
                );
            } else if max_penetrate != 1 {
                n.immune[owner] = OWNER_IMMUNE_TICKS;
            }
            if landed {
                let p = &mut self.slots[slot];
                p.penetrate -= 1;
                if p.penetrate == 0 {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    fn ground() -> World {
        let mut w = World::new("t".into(), 64, 64);
        for x in 0..64 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.spawn_x = 20;
        w.spawn_y = 20;
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn one_npc(world: &World) -> Npc {
        Npc::spawn_slime(world, 20).unwrap()
    }

    /// One pool update plus the per-NPC decay the NPC update would run
    /// (`NPC.cs:90454-90459`), so multi-tick tests see the gates age.
    fn tick(world: &mut World, pool: &mut ProjectilePool, npc: &mut Npc) {
        for v in npc.immune.iter_mut() {
            *v = (*v - 1).max(0);
        }
        npc.hurt_cd = (npc.hurt_cd - 1.0 / 60.0).max(0.0);
        pool.update(world, std::slice::from_mut(npc));
    }

    #[test]
    fn spawn_takes_the_first_free_slot_and_recycles_the_dead() {
        let mut pool = ProjectilePool::new();
        let a = pool.spawn(SHURIKEN, 0, Vec2::ZERO, Vec2::ZERO, 10, 0.0, false);
        let b = pool.spawn(SHURIKEN, 0, Vec2::ZERO, Vec2::ZERO, 10, 0.0, false);
        assert_eq!((a, b), (Some(0), Some(1)));
        pool.slots_mut()[0].active = false;
        pool.slots_mut()[0].time_left = 0;
        assert_eq!(
            pool.spawn(SHURIKEN, 0, Vec2::ZERO, Vec2::ZERO, 10, 0.0, false),
            Some(0)
        );
    }

    #[test]
    fn full_pool_recycles_the_lowest_time_left() {
        let mut pool = ProjectilePool::new();
        for i in 0..MAX_PROJECTILES {
            pool.spawn(SHURIKEN, 0, Vec2::ZERO, Vec2::ZERO, 10, 0.0, false);
            pool.slots_mut()[i].time_left = 1000 - (i % 10) as i32;
        }
        // The lowest timeLeft (991) sits at slot 9 — `FindOldestProjectile`
        // scans front-to-back keeping the strict minimum (`:10811-10821`).
        assert_eq!(pool.slots()[9].time_left, 991);
        assert_eq!(
            pool.spawn(SHURIKEN, 0, Vec2::ZERO, Vec2::ZERO, 10, 0.0, false),
            Some(9)
        );
    }

    #[test]
    fn single_hit_shuriken_dies_without_touching_immunity() {
        let mut world = ground();
        let mut pool = ProjectilePool::new();
        let mut npc = one_npc(&world);
        npc.x = 20.0 * TILE_PX;
        npc.y = 24.0 * TILE_PX;
        let slot = pool
            .spawn(
                SHURIKEN,
                0,
                Vec2::new(20.0 * TILE_PX, 24.0 * TILE_PX),
                Vec2::ZERO,
                10,
                12.0,
                false,
            )
            .unwrap();
        pool.update(&mut world, std::slice::from_mut(&mut npc));
        assert!(!pool.slots()[slot].active, "penetrate 1 → dead after the hit");
        assert_eq!(pool.slots()[slot].penetrate, 0);
        assert!(npc.hp < npc.max_hp);
        // `:14112-14119`: a single-hit projectile applies no immunity.
        assert_eq!(npc.immune[0], 0);
    }

    #[test]
    fn local_immunity_gates_rehits_for_the_cooldown() {
        let mut world = ground();
        let mut pool = ProjectilePool::new();
        let mut npc = one_npc(&world);
        npc.x = 20.0 * TILE_PX;
        npc.y = 24.0 * TILE_PX;
        let slot = pool
            .spawn(
                SHURIKEN,
                0,
                Vec2::new(20.0 * TILE_PX, 24.0 * TILE_PX),
                Vec2::ZERO,
                10,
                12.0,
                false,
            )
            .unwrap();
        // A 3-hit piercing shot with the 12-tick local cooldown (the C# sets
        // `usesLocalNPCImmunity` + `localNPCHitCooldown = 12` in SetDefaults).
        pool.set_immunity(slot, 3, true, 12, false, -1);
        let hp0 = npc.hp;
        tick(&mut world, &mut pool, &mut npc);
        assert!(pool.slots()[slot].active);
        assert_eq!(pool.slots()[slot].penetrate, 2);
        assert_eq!(npc.hp, hp0 - 10);
        // The local entry decays per update (`:17270-17277`) and gates every
        // re-hit until it reaches zero; a local hit zeroes
        // `NPC.immune[owner]` (`:14108-14109`). Set to 12 on tick 1, it frees
        // the gate on tick 13.
        for _ in 0..11 {
            tick(&mut world, &mut pool, &mut npc);
        }
        assert_eq!(npc.hp, hp0 - 10, "gated for the 11 middle ticks");
        tick(&mut world, &mut pool, &mut npc);
        assert_eq!(npc.hp, hp0 - 20, "tick 13: the local entry expired");
        assert_eq!(pool.slots()[slot].penetrate, 1);
    }

    #[test]
    fn default_penetrating_hits_use_the_owner_immunity_of_10() {
        let mut world = ground();
        let mut pool = ProjectilePool::new();
        let mut npc = one_npc(&world);
        npc.x = 20.0 * TILE_PX;
        npc.y = 24.0 * TILE_PX;
        let slot = pool
            .spawn(
                SHURIKEN,
                0,
                Vec2::new(20.0 * TILE_PX, 24.0 * TILE_PX),
                Vec2::ZERO,
                10,
                12.0,
                false,
            )
            .unwrap();
        pool.set_immunity(slot, 5, false, -2, false, -1);
        let hp0 = npc.hp;
        tick(&mut world, &mut pool, &mut npc);
        assert_eq!(npc.hp, hp0 - 10);
        assert_eq!(npc.immune[0], 10, "`:14117`");
        for _ in 0..9 {
            tick(&mut world, &mut pool, &mut npc);
        }
        assert_eq!(npc.hp, hp0 - 10, "the owner gate holds for the 10 ticks");
        assert_eq!(pool.slots()[slot].penetrate, 4, "tick 1 already spent one");
        tick(&mut world, &mut pool, &mut npc);
        assert_eq!(npc.hp, hp0 - 20, "the 10th decay tick freed the gate");
        assert_eq!(npc.immune[0], 10, "re-applied");
        assert_eq!(pool.slots()[slot].penetrate, 3, "the freed gate landed a hit");
    }

    #[test]
    fn id_static_immunity_is_shared_across_slots_of_the_same_kind() {
        let mut world = ground();
        let mut pool = ProjectilePool::new();
        let mut npc = one_npc(&world);
        npc.x = 20.0 * TILE_PX;
        npc.y = 24.0 * TILE_PX;
        let a = pool
            .spawn(
                SHURIKEN,
                0,
                Vec2::new(20.0 * TILE_PX, 24.0 * TILE_PX),
                Vec2::ZERO,
                10,
                12.0,
                false,
            )
            .unwrap();
        let b = pool
            .spawn(
                SHURIKEN,
                0,
                Vec2::new(20.0 * TILE_PX, 24.0 * TILE_PX),
                Vec2::ZERO,
                10,
                12.0,
                false,
            )
            .unwrap();
        pool.set_immunity(a, 2, false, -2, true, 30);
        pool.set_immunity(b, 2, false, -2, true, 30);
        let hp0 = npc.hp;
        tick(&mut world, &mut pool, &mut npc);
        // Both slots overlap the NPC in the same tick: the ID-static stamp
        // written by the first hit (`:13972`) gates the second.
        assert_eq!(npc.hp, hp0 - 10);
        let stamp = *pool.id_static.get(&(SHURIKEN, 0)).unwrap();
        assert!(stamp > pool.update_count, "the stamp must still be live");
        for _ in 0..31 {
            tick(&mut world, &mut pool, &mut npc);
        }
        assert!(
            npc.hp < hp0 - 10,
            "the expired stamp (`<= GameUpdateCount`, `:604-607`) allowed a re-hit"
        );
    }

    #[test]
    fn time_left_expires_the_slot() {
        let mut world = ground();
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn(SHURIKEN, 0, Vec2::ZERO, Vec2::ZERO, 10, 0.0, false)
            .unwrap();
        pool.slots_mut()[slot].time_left = 3;
        for _ in 0..3 {
            pool.update(&mut world, &mut []);
        }
        assert!(!pool.slots()[slot].active);
        assert_eq!(pool.slots()[slot].time_left, 0);
    }

    #[test]
    fn extra_updates_runs_the_ai_twice_per_tick() {
        let mut world = ground();
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn(
                SHURIKEN,
                0,
                Vec2::new(10.0 * TILE_PX, 20.0 * TILE_PX),
                Vec2::new(22.0, 0.0),
                10,
                0.0,
                false,
            )
            .unwrap();
        pool.slots_mut()[slot].extra_updates = 1;
        pool.update(&mut world, &mut []);
        assert!(
            (pool.slots()[slot].pos.x - (10.0 * TILE_PX + 44.0)).abs() < 0.01,
            "two AI steps × 22 px = 44 px, got {}",
            pool.slots()[slot].pos.x - 10.0 * TILE_PX
        );
    }
}
#[cfg(test)]
mod falling_tests {
    use super::*;

    fn world() -> tr_core::World {
        tr_core::World::new("t".into(), 64, 64)
    }

    // The update needs `&mut World` (the landing placement) — the fixtures
    // build the world, then pass it mutably.

    fn put_solid(w: &mut tr_core::World, x: i32, y: i32) {
        use tr_core::tile::{FLAG_ACTIVE, Tile};
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    #[test]
    fn the_falling_block_falls_at_the_tile_fall_gravity() {
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_falling(53, 31, 10, 160.0, 200.0)
            .unwrap();
        let mut npcs: Vec<Npc> = Vec::new();
        let mut w = world();
        pool.update(&mut w, &mut npcs);
        // The spawn vel (0, 0.41) + one gravity step (0.41) = 0.82.
        assert!(
            (pool.slots()[slot].vel.y - 0.82).abs() < 0.001,
            "the tile-fall gravity (`AI_010`'s `velocity.Y += 0.41f`)"
        );
        assert!(pool.slots()[slot].active);
        assert_eq!(pool.slots()[slot].kind, 31, "the sand ball projectile");
    }

    #[test]
    fn the_falling_block_lands_and_places_the_tile() {
        let mut pool = ProjectilePool::new();
        // The spawn just above the solid row 24: the bottom edge (y + 10)
        // starts at 384 — the top of row 24 — and the first gravity step
        // pushes it inside.
        let slot = pool
            .spawn_falling(53, 31, 10, 160.0, 374.0)
            .unwrap();
        let mut w = world();
        put_solid(&mut w, 10, 24);
        let mut npcs: Vec<Npc> = Vec::new();
        pool.update(&mut w, &mut npcs);
        assert!(!pool.slots()[slot].active, "the projectile died on landing");
        // The placement resolves through the box center → the tile row 23
        // (the last free row above the solid).
        let placed = w.tile(10, 23).unwrap();
        assert_eq!(placed.kind, 53, "the sand placed back");
        assert!(placed.active());
        assert_eq!(placed.liquid, 0);
    }

    #[test]
    fn the_falling_block_chain_refills_the_column() {
        // A two-sand column: breaking the support spawns the first fall; the
        // landing frees the second (the next edit pass re-checks).
        let mut pool = ProjectilePool::new();
        let mut w = world();
        put_solid(&mut w, 10, 24);
        use tr_core::tile::{FLAG_ACTIVE, Tile};
        for y in [21, 22] {
            let i = w.index(10, y);
            w.tiles[i] = Tile {
                kind: 53,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut npcs: Vec<Npc> = Vec::new();
        // Break the support (the tile at (10, 23)).
        {
            let i = w.index(10, 23);
            w.tiles[i] = Tile::default();
        }
        let Some(fall) = tr_core::falling::check_falling(&w, 10, 22) else {
            panic!("the sand at (10, 22) lost its support");
        };
        // Clear the falling tile (the C# `tileCache.ClearTile()`).
        {
            let i = w.index(10, 22);
            w.tiles[i] = Tile::default();
        }
        pool.spawn_falling(fall.tile_kind, fall.proj_kind, fall.damage, fall.x, fall.y);
        // Update until the landing.
        for _ in 0..40 {
            pool.update(&mut w, &mut npcs);
        }
        assert_eq!(w.tile(10, 23).unwrap().kind, 53, "the sand placed back");
        assert!(!pool.slots().iter().any(|p| p.active), "the fall is spent");
    }
}
