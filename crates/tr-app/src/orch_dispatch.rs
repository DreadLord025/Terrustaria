//! W4-d54j — the orchestrator dispatch router port:
//! `ParticleOrchestrator.SpawnParticlesDirect`
//! (`ParticleOrchestrator.cs:162-416`) for the d54-owned first-40 lane
//! (research/67 + 370), shaped after e4's `orchestrator_dust.rs::spawn_orchestra`
//! (its `:775` match). MERGER NOTE: this module IS the `_ => {}` arm of
//! e4's router — e4 keeps its 15 lane-owned kinds (1/3/5/7/8/9/12/13/14/
//! 15/25/37/55/72/77), this router owns the 24-arm d54j catalog (28 kind
//! bytes: the recharge trio shares one arm, 50/60 share boulder4's body,
//! 58/59 share shadowbeam's), the remaining enum bytes are unported
//! `Spawn_*` bodies (no draws — the documented lane-boundary drop). At
//! the merge the two compose into ONE `SpawnParticlesDirect` port;
//! never run both for one kind (double-draw risk, research/131 §8.3
//! step 7).
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
//! The four line-emitter wrappers (`Spawn_UFOLaser`/`MagnetSphereBolt`/
//! `HeatRay`/`Shadowbeam`, `:1950-1994`) are dust-free methods routing
//! through `SpawnHelper_SpawnInLine` (`:1996-2014`) →
//! `SpawnHelper_SpawnSingleLineDust` (`:2016-2026`, site 28) — ported
//! here on the batch-7 helpers. StormLightning's bolt generation is the
//! documented renderer seam: the `:1038` `settings.PositionInWorld =
//! EndPosition` write-back means the CALLER pre-resolves the bolt end
//! into `settings.position` before the drain; the camera-punch
//! NextFloat (`:1041`) + `Main.NewLightning` thunder draws
//! (`Main.cs:58712-58726`) are consumed by the caller in order
//! (research/67 landing record).
//!
//! WRITTEN NOT RUN (code-only mode).

// The router is a LATENT body until the wiring wave folds the queue
// drain (the netmodules.rs:447 Particles drop / the e4 pending_orchestra
// convention); the emitters stay reachable through it (the e4
// orchestrator_dust.rs precedent).
#![allow(dead_code)]

use crate::orch_dust::{self, InvokedPlayer};
use crate::orch_dust_boulder::{self, spawn_in_line};
use crate::orch_dust_lightning;
use crate::orch_dust_town::{self, LoadOutPlayer};
use crate::player::{Player, HEIGHT, WIDTH};
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// `ParticleOrchestraSettings` (`ParticleOrchestraSettings.cs:6-14`) —
/// KEEP-ONE landed (research/131 §8.3 / 370): e4's `OrchestraSettings`
/// wins; this alias retires the d54-local struct copy.
pub(crate) type D54Settings = crate::orchestrator_dust::OrchestraSettings;

/// `ParticleOrchestraType` (`ParticleOrchestraType.cs:3-88`) — the byte
/// values of the kinds this router owns (declaration order counted).
/// Disjoint from e4's `orchestra_kind` set; the merger keeps ONE const
/// module.
pub(crate) mod d54_kind {
    pub const TOWN_SLIME_TRANSFORM: u8 = 26;
    pub const LOADOUT_CHANGE: u8 = 27;
    pub const DIGESTION: u8 = 29;
    pub const SHIMMER_TOWN_NPC: u8 = 32;
    pub const DEAD_CELLS_DOWN_DASH_EXPLOSION: u8 = 35;
    pub const DEAD_CELLS_BARNACLE_SHOT_FIRING: u8 = 36;
    pub const SHADOW_ORB_EXPLOSION: u8 = 38;
    pub const UFO_LASER: u8 = 39;
    pub const DEAD_CELLS_FLINT: u8 = 41;
    pub const DEAD_CELLS_BARREL_EXPLOSION: u8 = 42;
    pub const MOON_LORD_WHIP_HIT: u8 = 44;
    pub const RAINBOW_BOULDER1: u8 = 47;
    pub const RAINBOW_BOULDER3: u8 = 49;
    pub const RAINBOW_BOULDER4: u8 = 50;
    pub const MAGNET_SPHERE_BOLT: u8 = 56;
    pub const HEAT_RAY: u8 = 57;
    pub const SHADOWBEAM_HOSTILE: u8 = 58;
    pub const SHADOWBEAM_FRIENDLY: u8 = 59;
    pub const RAINBOW_BOULDER_PET_BOUNCE: u8 = 60;
    pub const STORM_LIGHTNING: u8 = 61;
    pub const PALADINS_SHIELD_HIT: u8 = 63;
    pub const PALADINS_HAMMER_SHOCKWAVE: u8 = 68;
    pub const SNAPPING_STONE: u8 = 69;
    pub const SNAPPING_STONE_MELEE: u8 = 74;
    pub const SNAPPING_STONE_SUMMON: u8 = 75;
    pub const SNAPPING_STONE_RECHARGE: u8 = 79;
    pub const PYROCLASTIC_STONE_RECHARGE: u8 = 80;
    pub const ARMLET_OF_RUIN_RECHARGE: u8 = 81;
}

/// XNA `Color.LimeGreen` (50, 205, 50) / `Color.CornflowerBlue`
/// (100, 149, 237) — the UFOLaser tint pair (`:1954-1955`). LimeGreen
/// matches the port's `ui::uistate::LEGENDARY_MODE_COLOR` bytes.
const LIME_GREEN: Color = Color::new(50, 205, 50, 255);
const CORNFLOWER_BLUE: Color = Color::new(100, 149, 237, 255);

/// `Spawn_UFOLaser` (`:1950-1957`): per line step TWO type-160 singles
/// (the second at `dustPos − velocity·0.25`), each colour argument a
/// `Next(2) == 0 ? LimeGreen : CornflowerBlue` draw BEFORE the pool.
fn ufo_laser(s: &D54Settings, rng: &mut UnifiedRandom, dust: &mut DustPool, ctx: &SpawnCtx, world: &World) {
    spawn_in_line(s.position, s.movement, s.info, |dust_pos, velocity| {
        let c1 = if rng.next_max(2) == 0 { LIME_GREEN } else { CORNFLOWER_BLUE };
        orch_dust_boulder::single_line_dust(rng, dust, ctx, world, 160, dust_pos, c1);
        let c2 = if rng.next_max(2) == 0 { LIME_GREEN } else { CORNFLOWER_BLUE };
        orch_dust_boulder::single_line_dust(rng, dust, ctx, world, 160, dust_pos - velocity * 0.25, c2);
    });
}

/// `Spawn_MagnetSphereBolt` (`:1959-1968`): FOUR type-160 singles per
/// step (offsets 0/0.25/0.5/0.75), default colour — no Next(2) draws.
fn magnet_sphere_bolt(s: &D54Settings, rng: &mut UnifiedRandom, dust: &mut DustPool, ctx: &SpawnCtx, world: &World) {
    spawn_in_line(s.position, s.movement, s.info, |dust_pos, velocity| {
        orch_dust_boulder::single_line_dust(rng, dust, ctx, world, 160, dust_pos, Color::default());
        for k in 1..4 {
            orch_dust_boulder::single_line_dust(rng, dust, ctx, world, 160, dust_pos - velocity * (k as f32 * 0.25), Color::default());
        }
    });
}

/// `Spawn_HeatRay` (`:1970-1979`): the Magnet shape with type 162.
fn heat_ray(s: &D54Settings, rng: &mut UnifiedRandom, dust: &mut DustPool, ctx: &SpawnCtx, world: &World) {
    spawn_in_line(s.position, s.movement, s.info, |dust_pos, velocity| {
        orch_dust_boulder::single_line_dust(rng, dust, ctx, world, 162, dust_pos, Color::default());
        for k in 1..4 {
            orch_dust_boulder::single_line_dust(rng, dust, ctx, world, 162, dust_pos - velocity * (k as f32 * 0.25), Color::default());
        }
    });
}

/// `Spawn_Shadowbeam` (`:1981-1994`): THREE type-173 singles per step;
/// `velocityScalar` 0.03334 hostile (`:1986`, offsets ×1/×2) vs 0.25
/// friendly (`:1983`; the Friendly kind passes `hostile: false`, `:355`).
fn shadowbeam(s: &D54Settings, hostile: bool, rng: &mut UnifiedRandom, dust: &mut DustPool, ctx: &SpawnCtx, world: &World) {
    let velocity_scalar = if hostile { 0.03334f32 } else { 0.25 };
    spawn_in_line(s.position, s.movement, s.info, |dust_pos, velocity| {
        orch_dust_boulder::single_line_dust(rng, dust, ctx, world, 173, dust_pos, Color::default());
        orch_dust_boulder::single_line_dust(rng, dust, ctx, world, 173, dust_pos - velocity * velocity_scalar, Color::default());
        orch_dust_boulder::single_line_dust(rng, dust, ctx, world, 173, dust_pos - velocity * velocity_scalar * 2.0, Color::default());
    });
}

/// `Spawn_LoadOutChange`'s player reads (`:2372-2381`): the Hitbox
/// (the port's `Entity.getRect` truncating box — `(int)x/(int)y`,
/// `WIDTH`/`HEIGHT` as ints), `gravDir` (the port keeps 1 — no gravity
/// flip, player.rs doc) and `velocity`.
fn load_out_player(p: &Player) -> LoadOutPlayer {
    LoadOutPlayer {
        hitbox_x: p.x as i32 as f32,
        hitbox_y: p.y as i32 as f32,
        hitbox_w: WIDTH as i32 as f32,
        hitbox_h: HEIGHT as i32 as f32,
        grav_dir: 1.0,
        velocity: Vec2::new(p.vx, p.vy),
    }
}

/// The `Main.player` slice the router resolves indices against. The C#
/// array is 255 slots; the port is single-player — the caller passes
/// its live players (index 0 = the local player). `None` entries are
/// inactive slots.
pub(crate) type D54Players<'a> = [Option<&'a Player>];

/// `ParticleOrchestrator.SpawnParticlesDirect` (`:162-416`), the
/// d54-owned 24-arm slice (28 kind bytes) — e4's `spawn_orchestra`
/// `_ => {}` arm becomes this call at the merger. `gen` is
/// `WorldGen.genRand`'s stand-in, `disco` the `Main.DiscoR/G/B` snapshot
/// (the tile-sweep delegation), `players` the `Main.player` resolution
/// slice. StormLightning reads the caller-pre-resolved
/// `settings.position` (the `:1038` EndPosition seam, module doc).
/// Every other byte is an e4-owned or unported `Spawn_*` body — no
/// draws here (the lane boundary).
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
            // `:420`/`:439`/`:456` — Main.player[IndexOfPlayerWhoInvokedThis]
            // (the C# indexes unconditionally; the port's SP slice resolves
            // index 0 only — an unresolvable invoker drops, no draws).
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
        STORM_LIGHTNING => {
            // `:1029-1033` — white = Color.White, PackedValue = info when
            // info != 0 (XNA packed bytes: R low, A high). `:1034` —
            // weapon = invoker != byte.MaxValue. `:1038`'s EndPosition
            // write-back is caller-resolved into settings.position (the
            // renderer seam); the `:1041`/NewLightning draws are the
            // caller's ordered draws (module doc).
            let white = if settings.info != 0 {
                let packed = settings.info as u32;
                Color::new((packed & 0xFF) as u8, ((packed >> 8) & 0xFF) as u8, ((packed >> 16) & 0xFF) as u8, ((packed >> 24) & 0xFF) as u8)
            } else {
                Color::WHITE
            };
            let weapon = settings.invoker != u8::MAX;
            orch_dust_lightning::storm_lightning_dust(rng, gen, disco, dust, ctx, world, settings.position, white, weapon);
        }
        RAINBOW_BOULDER1 => orch_dust_boulder::rainbow_boulder1(rng, dust, ctx, world, settings.position),
        RAINBOW_BOULDER3 => orch_dust_boulder::rainbow_boulder3(rng, dust, ctx, world, settings.position, settings.movement),
        RAINBOW_BOULDER4 => orch_dust_boulder::rainbow_boulder4(rng, dust, ctx, world, settings.position, settings.movement, false),
        RAINBOW_BOULDER_PET_BOUNCE => orch_dust_boulder::rainbow_boulder4(rng, dust, ctx, world, settings.position, settings.movement, true),
        MOON_LORD_WHIP_HIT => orch_dust_boulder::moon_lord_whip(rng, dust, ctx, world, settings.position, settings.movement),
        DEAD_CELLS_BARREL_EXPLOSION => orch_dust_boulder::dead_cells_barrel_explosion(rng, dust, ctx, world, settings.position),
        DEAD_CELLS_FLINT => orch_dust_boulder::dead_cells_flint(rng, gen, disco, dust, ctx, world, settings.position, settings.movement),
        SHADOW_ORB_EXPLOSION => orch_dust_boulder::shadow_orb_explosion(rng, dust, ctx, world, settings.position),
        UFO_LASER => ufo_laser(settings, rng, dust, ctx, world),
        MAGNET_SPHERE_BOLT => magnet_sphere_bolt(settings, rng, dust, ctx, world),
        HEAT_RAY => heat_ray(settings, rng, dust, ctx, world),
        SHADOWBEAM_HOSTILE => shadowbeam(settings, true, rng, dust, ctx, world),
        SHADOWBEAM_FRIENDLY => shadowbeam(settings, false, rng, dust, ctx, world),
        DEAD_CELLS_DOWN_DASH_EXPLOSION => orch_dust_town::dead_cells_down_dash_explosion(rng, gen, disco, dust, ctx, world, settings.position, settings.movement, settings.info),
        SHIMMER_TOWN_NPC => orch_dust_town::shimmer_town_npc(rng, dust, ctx, world, settings.position),
        DIGESTION => orch_dust_town::digestion(rng, dust, ctx, world, settings.position, settings.movement.x),
        LOADOUT_CHANGE => {
            // `:2372-2373` — the player must exist AND be active.
            let Some(Some(p)) = players.get(settings.invoker as usize) else { return };
            orch_dust_town::load_out_change(rng, dust, ctx, world, &load_out_player(p));
        }
        DEAD_CELLS_BARNACLE_SHOT_FIRING => orch_dust_town::barnacle_shot_firing(rng, dust, ctx, world, settings.position, settings.movement),
        TOWN_SLIME_TRANSFORM => match settings.info {
            // `Spawn_TownSlimeTransform` (`:2401-2415`): 0 Nerdy / 1
            // Copper / 2 Elder; no default arm — other info draws nothing.
            0 => orch_dust_town::nerdy_slime_effect(rng, dust, ctx, world, settings.position, settings.movement),
            1 => orch_dust_town::copper_slime_effect(rng, dust, ctx, world, settings.position, settings.movement),
            2 => orch_dust_town::elder_slime_effect(rng, dust, ctx, world, settings.position, settings.movement),
            _ => {}
        },
        // e4-owned (its router) or unported Spawn_* bodies — the lane
        // boundary drops them, no draws (the merger composes the two).
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
        p.vx = 1.0;
        p.vy = 0.0;
        p
    }

    fn settings() -> D54Settings {
        D54Settings { position: Vec2::new(1600.0, 1600.0), movement: Vec2::new(3.0, -4.0), info: 0, invoker: 0 }
    }

    /// WITNESS 1 — the arm-count pin: exactly the 28 d54j kind bytes draw
    /// (a non-zero stream advance on a live player / a 16px line); every
    /// other byte — e4's 15 lane-owned kinds AND the unported bodies —
    /// drops with ZERO draws (`_ => {}`).
    #[test]
    fn w4_d54j_arm_count_pin() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let p = player(&w);
        let players = [Some(&p)];
        let mut s = settings();
        s.position = Vec2::new(1600.0, 1600.0);
        // The line kinds need a non-zero |position − movement| to walk.
        s.movement = s.position - Vec2::new(16.0, 0.0);
        let mut gen = UnifiedRandom::new(7);
        let mut dust = DustPool::new();
        let owned = [
            26u8, 27, 29, 32, 35, 36, 38, 39, 42, 44, 47, 49, 50,
            56, 57, 58, 59, 60, 61, 63, 68, 69, 74, 75, 79, 80, 81,
        ];
        for kind in owned {
            // ShieldHit fires only at SP index 0 — give it the (0,0) box.
            let mut sk = s;
            if kind == d54_kind::PALADINS_SHIELD_HIT {
                sk.position = Vec2::ZERO;
            }
            let mut rng = UnifiedRandom::new(54);
            spawn_orchestra_d54(kind, &sk, &mut rng, &mut gen, [255, 255, 255], &mut dust, &ctx, &w, &players);
            assert_ne!(rng.peek(), UnifiedRandom::new(54).peek(), "kind {kind} must draw");
            drain(&mut dust);
        }
        // Kind 41 (Flint) on an EMPTY world: the tile scan finds no hit,
        // `flag` stays false and the C# returns BEFORE the 30-dust tail
        // (`:1874-1877`) — zero draws is the faithful empty-world pin
        // (the head draws nothing; the scan's gates are tile reads).
        {
            let mut rng = UnifiedRandom::new(54);
            spawn_orchestra_d54(d54_kind::DEAD_CELLS_FLINT, &s, &mut rng, &mut gen, [255, 255, 255], &mut dust, &ctx, &w, &players);
            assert_eq!(rng.peek(), UnifiedRandom::new(54).peek(), "flint empty-world early return");
        }
        // e4's lane-owned kinds + unported bodies + strays.
        let unowned = [
            0u8, 1, 3, 5, 7, 8, 9, 12, 13, 14, 15, 25, 37, 48, 55, 62, 72, 77, 82,
            2, 4, 6, 10, 11, 16, 28, 30, 31, 33, 34, 40, 43, 45, 46, 51, 64, 65, 66, 67, 70, 71, 73, 76, 78,
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

    /// WITNESS 3 — the line trio's step ledger: `spawn_in_line` walks
    /// |Δ| = 16 px in 4 px steps (info 0 → arg zero → num2 clamps to 4)
    /// = 4 steps. UFO draws 2 Next(2) colour gates per step; Magnet/
    /// HeatRay/Shadowbeam draw NONE (default colour). Each single: a
    /// 1×1 block + Next(70,110) (types 160/162/173 are outside the
    /// Dust.cs:214-229 tweak chain).
    #[test]
    fn w4_d54j_line_wrappers_draw_the_step_ledger() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let mut s = settings();
        s.movement = s.position - Vec2::new(16.0, 0.0);
        for (kind, dtype, singles, gates) in [
            (d54_kind::UFO_LASER, 160, 2, true),
            (d54_kind::MAGNET_SPHERE_BOLT, 160, 4, false),
            (d54_kind::HEAT_RAY, 162, 4, false),
            (d54_kind::SHADOWBEAM_HOSTILE, 173, 3, false),
            (d54_kind::SHADOWBEAM_FRIENDLY, 173, 3, false),
        ] {
            let mut rng = UnifiedRandom::new(54);
            let mut probe = UnifiedRandom::new(54);
            let mut gen = UnifiedRandom::new(7);
            let mut dust = DustPool::new();
            spawn_orchestra_d54(kind, &s, &mut rng, &mut gen, [255, 255, 255], &mut dust, &ctx, &w, &[]);
            for _ in 0..4 {
                for _ in 0..singles {
                    if gates {
                        probe.next_max(2);
                    }
                    block(&mut probe, dtype, 1, 1);
                    probe.next_range(70, 110);
                }
            }
            assert_eq!(rng.peek(), probe.peek(), "line kind {kind}");
            let live = dust.slots().iter().filter(|d| d.active).count();
            assert_eq!(live, 4 * singles, "kind {kind} spawns 4 steps × {singles}");
        }
    }

    /// WITNESS 4 — TownSlimeTransform routes info 0→Nerdy (sixty type-4
    /// blocks, NO colour draw), 1→Copper (40 type-43), 2→Elder (30
    /// type-43, no clones); any other info draws NOTHING (`:2403-2414`'s
    /// switch has no default arm).
    #[test]
    fn w4_d54j_town_slime_transform_switches_on_info() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let mut s = settings();
        s.info = 0;
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut gen = UnifiedRandom::new(7);
        let mut dust = DustPool::new();
        spawn_orchestra_d54(d54_kind::TOWN_SLIME_TRANSFORM, &s, &mut rng, &mut gen, [255, 255, 255], &mut dust, &ctx, &w, &[]);
        for _ in 0..60 {
            // circular(2) + velocity NextFloat + scale NextFloat + block.
            probe.next_float();
            probe.next_float();
            probe.next_float();
            probe.next_float();
            block(&mut probe, 4, 0, 0);
        }
        assert_eq!(rng.peek(), probe.peek(), "nerdy via router");
        for info in [1, 2] {
            s.info = info;
            let mut r2 = UnifiedRandom::new(54);
            let mut d2 = DustPool::new();
            spawn_orchestra_d54(d54_kind::TOWN_SLIME_TRANSFORM, &s, &mut r2, &mut gen, [255, 255, 255], &mut d2, &ctx, &w, &[]);
            let live: Vec<_> = d2.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), if info == 1 { 40 } else { 30 }, "copper 40 / elder 30 perfects, no clones");
            assert!(live.iter().all(|d| d.dust_type == 43));
        }
        s.info = 7;
        let mut r3 = UnifiedRandom::new(54);
        let mut d3 = DustPool::new();
        spawn_orchestra_d54(d54_kind::TOWN_SLIME_TRANSFORM, &s, &mut r3, &mut gen, [255, 255, 255], &mut d3, &ctx, &w, &[]);
        assert_eq!(r3.peek(), UnifiedRandom::new(54).peek(), "unknown info draws nothing");
    }

    /// WITNESS 5 — LoadoutChange: a missing/inactive player draws NOTHING
    /// (`:2373`); the live player drains the 40+5 perfect ledger (the
    /// orch_dust_town witness shape — from-rect 2 + scale 1 + block +
    /// up/direction/velocity 3 per type-16; +1 hsl float per type-43).
    #[test]
    fn w4_d54j_loadout_gates_on_the_active_player() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let s = settings();
        let mut gen = UnifiedRandom::new(7);
        let mut rng = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        spawn_orchestra_d54(d54_kind::LOADOUT_CHANGE, &s, &mut rng, &mut gen, [255, 255, 255], &mut dust, &ctx, &w, &[]);
        spawn_orchestra_d54(d54_kind::LOADOUT_CHANGE, &s, &mut rng, &mut gen, [255, 255, 255], &mut dust, &ctx, &w, &[None]);
        assert_eq!(rng.peek(), UnifiedRandom::new(54).peek(), "no player, no draws");
        let p = player(&w);
        let players = [Some(&p)];
        let mut probe = UnifiedRandom::new(54);
        spawn_orchestra_d54(d54_kind::LOADOUT_CHANGE, &s, &mut rng, &mut gen, [255, 255, 255], &mut dust, &ctx, &w, &players);
        for _ in 0..40 {
            probe.next_float();
            probe.next_float();
            probe.next_float();
            block(&mut probe, 16, 0, 0);
            probe.next_float();
            probe.next_float_direction();
            probe.next_float();
        }
        for _ in 0..5 {
            probe.next_float();
            probe.next_float();
            probe.next_float();
            probe.next_float();
            block(&mut probe, 43, 0, 0);
            probe.next_float();
            probe.next_float_direction();
            probe.next_float();
        }
        assert_eq!(rng.peek(), probe.peek(), "live loadout ledger");
    }

    /// WITNESS 6 — ShieldHit: the indices ride the truncated position —
    /// SP index 0 fires BOTH arms on a (0,0) position (8 type-57 blocks);
    /// out-of-range draws nothing.
    #[test]
    fn w4_d54j_shield_hit_resolves_the_position_indices() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let p = player(&w);
        let players = [Some(&p)];
        let mut s = settings();
        s.position = Vec2::new(0.0, 0.0);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut gen = UnifiedRandom::new(7);
        let mut dust = DustPool::new();
        spawn_orchestra_d54(d54_kind::PALADINS_SHIELD_HIT, &s, &mut rng, &mut gen, [255, 255, 255], &mut dust, &ctx, &w, &players);
        for _ in 0..8 {
            block(&mut probe, 57, 20, 42);
        }
        assert_eq!(rng.peek(), probe.peek(), "both arms at index 0");
        s.position = Vec2::new(1600.0, 1600.0);
        let mut r2 = UnifiedRandom::new(54);
        let mut d2 = DustPool::new();
        spawn_orchestra_d54(d54_kind::PALADINS_SHIELD_HIT, &s, &mut r2, &mut gen, [255, 255, 255], &mut d2, &ctx, &w, &players);
        assert_eq!(r2.peek(), UnifiedRandom::new(54).peek(), "out-of-range draws nothing");
    }

    /// WITNESS 7 — StormLightning: the caller pre-resolves EndPosition
    /// into settings.position (the `:1038` seam); on an EMPTY world the
    /// tile sweep finds nothing, so the stream drains the explosion's
    /// six-arm ledger head (the 12×226 arm: block + HackFrame Next(3) +
    /// 2 NextFloat) + the 4 gore triples + the 12-step RandomizedFrame
    /// loop — verified through the port's own body on a twin stream
    /// (the delegation is the oracle, the orch_dust_lightning shape).
    #[test]
    fn w4_d54j_storm_lightning_routes_the_resolved_position() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let mut s = settings();
        s.position = Vec2::new(1600.0, 1600.0); // the caller's EndPosition
        s.info = 0; // white stays Color.White
        s.invoker = u8::MAX; // storm (weapon = false → 12-step loop)
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut gen = UnifiedRandom::new(7);
        let mut gprobe = UnifiedRandom::new(7);
        let mut dust = DustPool::new();
        let mut mirror = DustPool::new();
        spawn_orchestra_d54(d54_kind::STORM_LIGHTNING, &s, &mut rng, &mut gen, [255, 255, 255], &mut dust, &ctx, &w, &[]);
        orch_dust_lightning::storm_lightning_dust(&mut probe, &mut gprobe, [255, 255, 255], &mut mirror, &ctx, &w, s.position, Color::WHITE, false);
        assert_eq!(rng.peek(), probe.peek(), "storm body parity");
        assert_eq!(gen.peek(), gprobe.peek(), "gen stream parity");
        let live = dust.slots().iter().filter(|d| d.active).count();
        let mlive = mirror.slots().iter().filter(|d| d.active).count();
        assert_eq!(live, mlive, "slot census parity");
        assert!(live > 0, "the explosion arms spawn on an empty world");
    }
}
