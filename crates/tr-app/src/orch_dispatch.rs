//! W4-d54 phase 1 — the orchestrator dispatch router port:
//! `ParticleOrchestrator.SpawnParticlesDirect`
//! (`ParticleOrchestrator.cs:162-416`) for the d54-owned first-40 lane
//! (research/67), shaped after e4's `orchestrator_dust.rs::spawn_orchestra`
//! (its `:775` match). MERGER NOTE: this module IS the `_ => {}` arm of
//! e4's router — e4 keeps its 15 lane-owned kinds (1/3/5/7/8/9/12/13/14/
//! 15/25/37/55/72/77), this router owns the d54 arms (the first 8 are
//! here; phase 2 owes 16 more — the owed list at the `_` arm), the
//! remaining enum bytes are unported `Spawn_*` bodies (no draws — the
//! documented lane-boundary drop). At the merge the two compose into ONE
//! `SpawnParticlesDirect` port; never run both for one kind (double-draw
//! risk, research/131 §8.3 step 7).
//!
//! RNG contract (S1-A-10): the spawns run SYNCHRONOUSLY inside
//! `RequestParticleSpawn`/`BroadcastParticleSpawn` (`:52-90` →
//! `SpawnParticlesDirect` :162) — each arm drains on the stream of the
//! pass that invoked it; the caller passes `&mut UnifiedRandom` (`rng` =
//! `Main.rand`, `gen` = `WorldGen.genRand`'s runtime stand-in
//! `World::frame_rng`). Argument draws run BEFORE the pool internals.
//! The queue wiring follows the e4 S1-E-21c convention: settings
//! ARGUMENT draws run INLINE at the queue site, the bodies defer to the
//! pass-tail drain that calls this router on the SAME stream.
//!
//! WRITTEN NOT RUN (code-only mode).

// The router is a LATENT body until the wiring wave folds the queue
// drain (the netmodules.rs:447 Particles drop / the e4 pending_orchestra
// convention); the emitters stay reachable through it (the e4
// orchestrator_dust.rs precedent).
#![allow(dead_code)]

use crate::orch_dust::{self, InvokedPlayer};
use crate::orch_dust_lightning;
use crate::player::Player;
use glam::Vec2;
use tr_core::particles::{DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// `ParticleOrchestraSettings` (`ParticleOrchestraSettings.cs:6-14`) —
/// the d54-local copy of e4's `OrchestraSettings` (the net carrier is
/// `app/netmodules.rs::Particles`). KEEP-ONE at the merger: e4's type
/// wins, this alias retires (research/131 §8.3).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct D54Settings {
    pub position: Vec2,
    pub movement: Vec2,
    pub info: i32,
    pub invoker: u8,
}

/// `ParticleOrchestraType` (`ParticleOrchestraType.cs:3-88`) — the byte
/// values of the kinds this router owns (declaration order counted).
/// Disjoint from e4's `orchestra_kind` set; the merger keeps ONE const
/// module. Phase 2 adds the remaining 16 constants.
pub(crate) mod d54_kind {
    pub const PALADINS_SHIELD_HIT: u8 = 63;
    pub const PALADINS_HAMMER_SHOCKWAVE: u8 = 68;
    pub const SNAPPING_STONE: u8 = 69;
    pub const SNAPPING_STONE_MELEE: u8 = 74;
    pub const SNAPPING_STONE_SUMMON: u8 = 75;
    pub const SNAPPING_STONE_RECHARGE: u8 = 79;
    pub const PYROCLASTIC_STONE_RECHARGE: u8 = 80;
    pub const ARMLET_OF_RUIN_RECHARGE: u8 = 81;
}

/// The `Main.player` slice the router resolves indices against. The C#
/// array is 255 slots; the port is single-player — the caller passes
/// its live players (index 0 = the local player). `None` entries are
/// inactive slots.
pub(crate) type D54Players<'a> = [Option<&'a Player>];

/// `ParticleOrchestrator.SpawnParticlesDirect` (`:162-416`), the
/// d54-owned slice — phase 1: the FIRST 8 ARMS (the recharge trio,
/// SnappingStone + melee/summon, PaladinsHammerShockwave,
/// PaladinsShieldHit). e4's `spawn_orchestra` `_ => {}` arm becomes
/// this call at the merger. `gen` is `WorldGen.genRand`'s stand-in,
/// `disco` the `Main.DiscoR/G/B` snapshot (the tile-sweep delegation),
/// `players` the `Main.player` resolution slice. Every other byte is
/// an e4-owned or unported `Spawn_*` body — no draws here (the lane
/// boundary).
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_orchestra_d54(
    kind: u8,
    settings: &D54Settings,
    rng: &mut UnifiedRandom,
    gen: &mut UnifiedRandom,
    disco: [u8; 3],
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    players: &D54Players<'_>,
) {
    use d54_kind::*;
    match kind {
        ARMLET_OF_RUIN_RECHARGE | PYROCLASTIC_STONE_RECHARGE | SNAPPING_STONE_RECHARGE => {
            // `:420`/`:439`/`:456` — Main.player[IndexOfPlayerWhoInvokedThis].
            let Some(Some(p)) = players.get(settings.invoker as usize) else { return };
            let player = InvokedPlayer::of(p);
            match kind {
                ARMLET_OF_RUIN_RECHARGE => orch_dust::armlet_of_ruin_recharge(rng, dust, ctx, world, &player),
                PYROCLASTIC_STONE_RECHARGE => orch_dust::pyroclastic_stone_recharge(rng, dust, ctx, world, &player),
                _ => orch_dust::snapping_stone_recharge(rng, dust, ctx, world, &player),
            }
        }
        SNAPPING_STONE => orch_dust::snapping_stone(rng, dust, ctx, world, settings.position, settings.movement.x),
        SNAPPING_STONE_MELEE => orch_dust::snapping_stone_melee(rng, dust, ctx, world, settings.position),
        SNAPPING_STONE_SUMMON => orch_dust::snapping_stone_summon(rng, dust, ctx, world, settings.position),
        PALADINS_HAMMER_SHOCKWAVE => orch_dust::paladins_hammer_shockwave(rng, gen, disco, dust, ctx, world, settings.position, settings.movement),
        PALADINS_SHIELD_HIT => {
            // `:911-914`/`:940` — the two indices ride the position's
            // truncated components; `Utils.IndexInRange` (Utils.cs:
            // 2441-2448) is `0 <= i < len` (SP: index 0 only).
            let num = settings.position.x as i32;
            let num2 = settings.position.y as i32;
            let a = players.get(num.max(0) as usize).filter(|_| num >= 0).and_then(|p| *p).map(InvokedPlayer::of);
            let b = players.get(num2.max(0) as usize).filter(|_| num2 >= 0).and_then(|p| *p).map(InvokedPlayer::of);
            orch_dust_lightning::paladins_shield_hit(rng, dust, ctx, world, a.as_ref(), b.as_ref());
        }
        // phase 2: 16 arms owed (61/47/49/50/60/44/42/41/38/39/56/57/58/59/35/32/29/27/36/26)
        // — plus e4-owned (its router) / unported Spawn_* bodies: the
        // lane boundary drops them, no draws (the merger composes the two).
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dust_ctx(w: &World) -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(w.width as f32 * 16.0, w.height as f32 * 16.0),
            max_dust_to_draw: 6000,
        }
    }

    /// The default block (the orch_dust oracle): position pair, velocity
    /// pair, frame Next(3), scale roll; the 6/228 family adds Next(-10,6).
    fn block(r: &mut UnifiedRandom, dust_type: i32, box_w: i32, box_h: i32) {
        r.next_max(box_w.max(5) - 4);
        r.next_max(box_h.max(5) - 4);
        r.next_range(-20, 21);
        r.next_range(-20, 21);
        r.next_max(3);
        r.next_range(-20, 21);
        if matches!(dust_type, 6 | 228) {
            r.next_range(-10, 6);
        }
    }

    fn drain(dust: &mut DustPool) {
        for s in dust.slots_mut() {
            s.active = false;
        }
    }

    fn player(w: &World) -> Player {
        let mut p = Player::spawn(w);
        p.x = 1600.0;
        p.y = 1600.0;
        p
    }

    fn settings() -> D54Settings {
        D54Settings { position: Vec2::new(1600.0, 1600.0), movement: Vec2::new(3.0, -4.0), info: 0, invoker: 0 }
    }

    /// WITNESS 1 — the arm-count pin: exactly the 8 phase-1 kinds draw
    /// (a non-zero stream advance on a live player); every other byte —
    /// e4's 15 lane-owned kinds AND the 16 phase-2-owed kinds — drops
    /// with ZERO draws (`_ => {}`). The owed list is the phase-2 debt.
    #[test]
    fn w4_d54j_phase1_arm_count_pin() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let p = player(&w);
        let players = [Some(&p)];
        let mut s = settings();
        s.position = Vec2::new(0.0, 0.0); // ShieldHit fires at SP index 0
        let mut gen = UnifiedRandom::new(7);
        let mut dust = DustPool::new();
        let owned = [63u8, 68, 69, 74, 75, 79, 80, 81];
        for kind in owned {
            let mut rng = UnifiedRandom::new(54);
            spawn_orchestra_d54(kind, &s, &mut rng, &mut gen, [255, 255, 255], &mut dust, &ctx, &w, &players);
            assert_ne!(rng.peek(), UnifiedRandom::new(54).peek(), "kind {kind} must draw");
            drain(&mut dust);
        }
        // e4's lane-owned kinds + the phase-2-owed kinds + strays.
        let unowned = [
            0u8, 1, 3, 5, 7, 8, 9, 12, 13, 14, 15, 25, 37, 48, 55, 62, 72, 77, 82,
            61, 47, 49, 50, 60, 44, 42, 41, 38, 39, 56, 57, 58, 59, 35, 32, 29, 27, 36, 26,
        ];
        for kind in unowned {
            let mut rng = UnifiedRandom::new(54);
            spawn_orchestra_d54(kind, &s, &mut rng, &mut gen, [255, 255, 255], &mut dust, &ctx, &w, &players);
            assert_eq!(rng.peek(), UnifiedRandom::new(54).peek(), "kind {kind} must drop");
        }
    }

    /// WITNESS 2 — a draw-order replay through the router: kind 81 adds
    /// NO draws of its own — the stream drains exactly the armlet body's
    /// ten type-15 blocks over the 20×42 player box (`:421-434`), in
    /// order, on the caller's stream.
    #[test]
    fn w4_d54j_router_armlet_draw_order_replay() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let p = player(&w);
        let players = [Some(&p)];
        let s = settings();
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut gen = UnifiedRandom::new(7);
        let mut dust = DustPool::new();
        for i in 0..50 {
            spawn_orchestra_d54(d54_kind::ARMLET_OF_RUIN_RECHARGE, &s, &mut rng, &mut gen, [255, 255, 255], &mut dust, &ctx, &w, &players);
            for _ in 0..10 {
                block(&mut probe, 15, 20, 42);
            }
            assert_eq!(rng.peek(), probe.peek(), "router armlet stream @{i}");
            drain(&mut dust);
        }
    }
}
