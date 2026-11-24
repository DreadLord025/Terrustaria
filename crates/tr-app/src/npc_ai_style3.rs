//! W4-d22 (NPC aiStyle 3) — the shared `AI_003_Fighters` core
//! (`NPC.cs:55856-60298`), the phases the per-family walkers deferred:
//!
//! - the head `directionY` level-edge flip (`:55858-55861`);
//! - the chase/despawn arms (`:57336-57401`): the shared TargetClosest at
//!   `:57369`, the `:57370-57373` directionY refresh, and the discouraged
//!   arm's EncourageDespawn(10) + wander-flip + zero-direction snap;
//! - the `num107`/per-type velocity ladder (`npc_ai3_speeds`, the C#'s
//!   `:57403-58401` branch order) with the hop flag carried through;
//! - the obstacle scan tail (`npc_ai_fighter_scan`, `:59831-60284`): the
//!   ground probe, the step-up, the door bash and the jump ladder;
//! - the movement integration (the port's `GRAVITY`/`MAX_FALL` seam —
//!   `UpdateCollision` runs after `AI()` in the C#; the blocked-path -6f
//!   fallback is the S1-D-3z convention, gated so the scan's own jump
//!   survives its tick).
//!
//! The idle counter (`:57034-57149`) is NOT re-ticked here: the world sim
//! runs `ambient_sound::fighter_idle_counter` pre-AI for every
//! `fighter_walker_kind` (S1-E-21e — `old_x` holds the prior-tick stamp
//! there; `update_preamble` overwrites it before this core runs). The
//! ambient moans (`:57338-57367`) ride the sim's PRE-SLOT head emitter on
//! the `UpdateNPCs` stream (the W4-d22 retire fold — the C# rolls them at
//! the AI head BEFORE the machine's draws; the old sim-tail position is
//! retired). RNG census (the W4-d22/g4 reconcile): the
//! core draws ZERO rand for every claimed kind OUTSIDE a graveyard (the
//! moan type lists name none of them; the ladder/scan are draw-free); the
//! ONE possible draw is the bash's graveyard gate `Next(60)`
//! (`NPC.cs:59997`, W4-g4 lane 15) — the sim pre-stamps
//! `Npc::target_zone_graveyard` and the core threads the caller's
//! `UpdateNPCs` stream into the scan (the signature union). In a
//! graveyard the moan head draws first (none for the claimed set), then
//! the bash's conditional draw — the C# order, the S1-E-21e
//! moan-position remainder CLOSED.
//!
//! Deferred per-type HEAD machines (documented, they sit BEFORE the core
//! in the C# and are out of this lane's scope): the GigaZapper 389's
//! `:59046-59067` zap cycle (`Next(3)` at the fire tick), the
//! MartianEngineer 386's `:58984-59044` turret-spawn search (the two
//! `Next` draws per probe), the officer 383's bubble block (the sim's
//! `martian_officer_bubble` seam owns it), the shooter families
//! (381/382/520/214/215/216/409/411/424/426/350/111 — their machines own
//! `ai[1]`/`ai[2]`), the Selenian 419's `:56719-56784` roll and the
//! Drakomire 415's `:56786-56853` rider-spawn cycle (`Next(10)` +
//! the 20-dust `NextFloat` rolls). Their kinds are NOT claimed here.
//!
//! Co-lane note: the g4 lane owns the full per-type fighter census and
//! the g8 lane the kind wiring; this core is the shared seam both read —
//! the claimed set below is the plain-walker family only (no `ai[1]`/
//! `ai[2]`-owning machine, no unported draw-bearing head arm).

use crate::npc::{Npc, NpcKind, GRAVITY, MAX_FALL};
use tr_core::{World, TILE_PX};

/// The plain-walker set the shared core claims on the aiStyle-3 axis:
/// every kind whose C# machine is the generic fighter (no shooter block,
/// no draw-bearing head arm). The zombie family keeps its own
/// `update_zombie` fold (byte-identical phases, S1-D-3z); the frost
/// siblings 338-352 keep `npc_ai_frost_mob`'s machines (the ladder
/// declines them); the shooters/martian-special kinds keep their fns.
pub(crate) fn claims(kind: NpcKind) -> bool {
    matches!(
        kind,
        NpcKind::Skeleton
            | NpcKind::UndeadMiner
            | NpcKind::GoblinThief
            | NpcKind::GoblinWarrior
            | NpcKind::GoblinPeon
            | NpcKind::PirateDeckhand
            | NpcKind::PirateCaptainsMate
            | NpcKind::GrayGrunt
            | NpcKind::Scutlix
            | NpcKind::VortexSoldier
            | NpcKind::SolarSpearman
            | NpcKind::Scarecrow1
            | NpcKind::Scarecrow2
            | NpcKind::Scarecrow3
            | NpcKind::Scarecrow4
            | NpcKind::Scarecrow5
            | NpcKind::Scarecrow6
            | NpcKind::Scarecrow7
            | NpcKind::Scarecrow8
            | NpcKind::Scarecrow9
            | NpcKind::Scarecrow10
            | NpcKind::Splinterling
    )
}

/// One `AI_003_Fighters` tick for a claimed plain walker: the phases in
/// the C#'s order, then the port's gravity + collision integration.
/// `rng` is the caller's `UpdateNPCs` stream (the W4-g4 lane-15 union):
/// the only possible draw is the graveyard bash gate's `Next(60)`
/// (`NPC.cs:59997`); outside a graveyard the core is draw-free (the
/// module header's census).
pub(crate) fn fighter_core_step(
    npc: &mut Npc,
    world: &World,
    player: &crate::player::Player,
    rng: &mut tr_core::UnifiedRandom,
) {
    let t = npc.kind.cs_id();
    let (w, h) = npc.size();
    // `:55858-55861` — the head: the player's bottom edge exactly level
    // with the NPC's flips directionY up.
    crate::npc_ai_fighter_scan::fighter_head_direction_y(npc, player.y + player.height);
    // `:57017-57026` — flag5/flag6 init; flag6 is the scan's own read.
    // `:57031-57034` — the Nutcracker transform: 348 has no claim here
    // (the frost sibling owns it); the 198 row names no port kind.
    // `:57035-57038` — num57: 60 (the type-120 row's 180 cannot fire —
    // 120 has no claim).
    let num57 = 60.0f32;
    // `:57067-57075` — flag8 from the type gate (the scarecrows 305-309
    // and the Scutlix 391 fold false).
    let flag8 = crate::npc_ai_fighter_scan::fighter_flag8(t);
    // `:57120-57149` — the idle counter: the sim's pre-pass ticked it
    // (S1-E-21e); no re-tick here (the module header).
    // `:57336` — the chase gate; `:57338-57367` the moan arms ride the
    // sim's PRE-SLOT head emitter (zero draws for the claimed set).
    if npc.ai[3] < num57 && crate::npc_ai::fighters_not_discouraged(npc, world) {
        // `:57369` — the shared selection (the S1-D-3p candidate seam).
        let cand = crate::npc_target::session_candidate_for(player, npc.kind);
        crate::npc_target::target_closest(npc, world, &[cand], true, 3, 0);
        // BUG-07's lockstep write (the walker_step_fighter convention).
        npc.sprite_dir = npc.facing;
        // `:57370-57373` — the directionY refresh.
        if npc.direction_y > 0 && player.center().1 <= npc.y + h {
            npc.direction_y = -1;
        }
    } else if !(npc.ai[2] > 0.0) || !can_be_busy_with_action(t) {
        // `:57375-57401` — the discouraged arm.
        // `:57377-57380` — the daytime flee-despawn (624/631 exempt;
        // neither is claimed).
        if world.day_time
            && (npc.y as f64 / 16.0) < world.world_surface
            && t != 624
            && t != 631
        {
            npc.encourage_despawn(10);
        }
        // `:57381-57397` — the wander-flip: grounded and stopped, ai[0]
        // advances and flips the direction every 2 ticks; motion resets.
        if npc.vx == 0.0 {
            if npc.vy == 0.0 {
                npc.ai[0] += 1.0;
                if npc.ai[0] >= 2.0 {
                    npc.facing = -npc.facing;
                    npc.sprite_dir = npc.facing;
                    npc.ai[0] = 0.0;
                }
            }
        } else {
            npc.ai[0] = 0.0;
        }
        // `:57398-57401` — a zero direction snaps to +1.
        if npc.facing == 0 {
            npc.facing = 1;
        }
    }
    // `:57403-58401` — the velocity ladder runs EVERY tick in the C#
    // (outside the chase/despawn if-chain): a discouraged fighter still
    // walks at its row's cap in the wander direction. The claimed set is
    // all ladder-named types; a declined id (none today) keeps its
    // current velocity — the C# has no fallback write.
    let (pcx, pcy) = player.center();
    let hop_flag = run_ladder(npc, world, pcx, pcy, player.dead).unwrap_or(false);
    // `:59831-60284` — the obstacle scan tail (the ground probe, the
    // step-up, the door bash, the jump ladder), at the C#'s position:
    // after the chase/ladder arms, BEFORE the integration.
    crate::npc_ai_fighter_scan::fighter_obstacle_scan_g(
        npc,
        world,
        rng,
        flag8,
        hop_flag,
        num57,
        player.center(),
    );
    integrate(npc, world, w, h);
}

/// `NPC.DespawnEncouragement_AIStyle3_Fighters_CanBeBusyWithAction`
/// (`NPC.cs:60332-60339`), verbatim: the listed types (and 520) return
/// true; every other type false. Among the claimed set the GrayGrunt 385
/// and the Scutlix 391 ARE listed — but their `ai[2]` action machines are
/// deferred (the module header), so `ai[2]` stays 0 and the discouraged
/// arm's `!(ai[2] > 0)` term still opens. The gate stays whole for the
/// census / the future per-type ports.
pub(crate) fn can_be_busy_with_action(t: u16) -> bool {
    matches!(
        t,
        110 | 111
            | 206
            | 216
            | 214
            | 215
            | 291
            | 292
            | 293
            | 350
            | 381
            | 382
            | 383
            | 385
            | 386
            | 389
            | 391
            | 469
            | 166
            | 466
            | 471
            | 411
            | 409
            | 424
            | 425
            | 426
            | 415
            | 419
            | 520
    )
}

/// The ladder run with the hop flag carried out (the `run_ai3_ladder`
/// shape, `npc_ai.rs:197`, plus `fx.hop_flag` — the scarecrow 305-309
/// arm's `flag` write, `NPC.cs:57847`, feeding the scan's ground-probe
/// gate `:59831`). Returns `None` when the ladder declines the type.
fn run_ladder(
    npc: &mut Npc,
    world: &World,
    target_x: f32,
    target_y: f32,
    target_dead: bool,
) -> Option<bool> {
    let (w, h) = npc.size();
    let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
    let mut ctx = crate::npc_ai3_speeds::LadderCtx {
        npc_type: npc.kind.cs_id(),
        vx: npc.vx,
        vy: npc.vy,
        direction: npc.facing,
        sprite_direction: npc.sprite_dir,
        scale: npc.scale,
        life: npc.hp,
        life_max: npc.max_hp,
        ai2: npc.ai[2],
        ai3: npc.ai[3],
        center_x: cx,
        center_y: cy,
        gravity: GRAVITY,
        target_center_x: target_x,
        target_center_y: target_y,
        target_dead,
        day_time: world.day_time,
        below_surface: (npc.y as f64) > world.world_surface * 16.0,
        can_hit_target: crate::projectile::can_hit_line(
            world, cx, cy, 1.0, 1.0, target_x, target_y, 1.0, 1.0,
        ),
        def_damage: npc.kind.contact_damage(),
    };
    let ((vx, vy), fx) = crate::npc_ai3_speeds::apply_ladder(&mut ctx)?;
    npc.vx = vx;
    npc.vy = vy;
    if let Some(v) = fx.ai2 {
        npc.ai[2] = v;
    }
    if let Some(v) = fx.ai3 {
        npc.ai[3] = v;
    }
    if let Some(v) = fx.damage {
        npc.damage_override = Some(v);
    }
    Some(fx.hop_flag)
}

/// The gravity + tile-collision integration — the `update_zombie` tail
/// shape plus the C# `UpdateNPC` ±0.005 X deadband (`NPC.cs:90473-90476`,
/// the DOUBLE compare) that runs after `AI()` and before the collision
/// pass. The blocked-path fallback jumps the C#'s -6f (`NPC.cs:60125`)
/// only while falling or level, so the scan's own jump survives its tick.
fn integrate(npc: &mut Npc, world: &World, w: f32, h: f32) {
    npc.vy = (npc.vy + GRAVITY).min(MAX_FALL);
    // `:90473-90476` — the dead-band X snap (double compare).
    if (npc.vx as f64) < 0.005 && (npc.vx as f64) > -0.005 {
        npc.vx = 0.0;
    }
    let nx = npc.x + npc.vx;
    if !crate::player::collides_rect(world, nx, npc.y, w, h) {
        npc.x = nx;
    } else if npc.on_ground && npc.vy >= 0.0 {
        npc.vy = -6.0;
        npc.on_ground = false;
    } else {
        npc.vx = 0.0;
    }
    let ny = npc.y + npc.vy;
    if !crate::player::collides_rect(world, npc.x, ny, w, h) {
        npc.y = ny;
        npc.on_ground = false;
    } else if npc.vy > 0.0 {
        let ty = ((ny + h) / TILE_PX).floor() as i32;
        npc.y = ty as f32 * TILE_PX - h;
        npc.vy = 0.0;
        npc.on_ground = true;
    } else if npc.vy < 0.0 {
        let ty = (ny / TILE_PX).floor() as i32;
        npc.y = (ty + 1) as f32 * TILE_PX;
        npc.vy = 0.0;
    }
    // The world-bottom cull (the update_zombie convention).
    if npc.y > world.height as f32 * TILE_PX + 200.0 {
        npc.hp = 0;
    }
}

#[cfg(test)]
mod tests {
    //! W4-d22 — tests-as-code, WRITTEN NOT RUN (code-only mode). Each
    //! assertion pins a phase to its C# literal (`NPC.cs:55856-60298`).
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// A 64x64 world, dirt ground at row 30 (the fighter_scan tests' shape).
    fn ground() -> World {
        let mut w = World::new("t".into(), 64, 64);
        for x in 0..64 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn skeleton_at(w: &World, tx: i32) -> Npc {
        let mut n = Npc::new(NpcKind::Skeleton, tx as f32 * TILE_PX, 440.0, 1, 0.0);
        n.facing = 1;
        n.sprite_dir = 1;
        n.on_ground = true;
        n
    }

    /// The claimed set: the plain walkers only — no shooter, no frost
    /// sibling, no zombie (its own fold), no martian head-arm kind.
    #[test]
    fn the_claim_set_is_the_plain_walker_family() {
        for k in [
            NpcKind::Skeleton,
            NpcKind::UndeadMiner,
            NpcKind::GoblinPeon,
            NpcKind::PirateDeckhand,
            NpcKind::GrayGrunt,
            NpcKind::Scutlix,
            NpcKind::VortexSoldier,
            NpcKind::SolarSpearman,
            NpcKind::Scarecrow1,
            NpcKind::Scarecrow10,
            NpcKind::Splinterling,
        ] {
            assert!(claims(k), "{k:?} is a plain walker");
        }
        for k in [
            NpcKind::Zombie,
            NpcKind::BloodZombie,
            NpcKind::GigaZapper,
            NpcKind::MartianEngineer,
            NpcKind::MartianOfficer,
            NpcKind::MartianWalker,
            NpcKind::BrainScrambler,
            NpcKind::RayGunner,
            NpcKind::PirateArcher,
            NpcKind::PirateCaptain,
            NpcKind::GoblinArcher,
            NpcKind::GoblinSummoner,
            NpcKind::VortexRifleman,
            NpcKind::StardustSoldier,
            NpcKind::StardustSpiderBig,
            NpcKind::SolarDrakomire,
            NpcKind::SolarSelenian,
            NpcKind::Predictor,
            NpcKind::Nutcracker,
            NpcKind::Krampus,
            NpcKind::Yeti,
            NpcKind::GingerbreadMan,
            NpcKind::ElfArcher,
        ] {
            assert!(!claims(k), "{k:?} keeps its own machine");
        }
    }

    /// The chase arm (`NPC.cs:57336-57373` + the num83 ladder
    /// `:57606-57740`): a night-time skeleton accelerates toward the
    /// player at 0.07/tick toward the 1.5 cap (the scale row `:57693`
    /// multiplies by 1+(1-scale); scale 1 keeps 1.5).
    #[test]
    fn the_chase_arm_accelerates_on_the_num83_row() {
        let mut w = ground();
        w.day_time = false;
        let mut n = skeleton_at(&w, 20);
        let mut p = crate::player::Player::spawn(&w);
        p.x = 30.0 * TILE_PX;
        p.y = 440.0 - crate::player::HEIGHT;
        p.dead = false;
        fighter_core_step(&mut n, &w, &p, &mut tr_core::UnifiedRandom::new(0));
        assert_eq!(n.facing, 1, "the player is to the right");
        assert!((n.vx - 0.07).abs() < 1e-6, "one 0.07 accel step: {}", n.vx);
        for _ in 0..100 {
            fighter_core_step(&mut n, &w, &p, &mut tr_core::UnifiedRandom::new(0));
        }
        assert!(n.vx <= 1.5 + 1e-6, "the num83 cap: {}", n.vx);
    }

    /// The discouraged arm (`NPC.cs:57375-57401`): a day-above-surface
    /// skeleton takes EncourageDespawn(10) and the wander-flip counter
    /// advances while the body is parked (vx 0, grounded). The ladder
    /// accelerates it away on the next tick (the C# runs the ladder
    /// outside the chase chain) — the flip itself fires on the 2-tick
    /// boundary, pinned here on the counter arithmetic the arm writes.
    #[test]
    fn the_discouraged_arm_wanders_and_encourages_despawn() {
        let mut w = ground();
        w.day_time = true;
        w.world_surface = 40.0; // the skeleton at y 440 (row 27) is above
        let mut n = skeleton_at(&w, 20);
        n.vx = 0.0;
        n.vy = 0.0;
        let mut p = crate::player::Player::spawn(&w);
        p.x = 30.0 * TILE_PX;
        p.y = 440.0 - crate::player::HEIGHT;
        fighter_core_step(&mut n, &w, &p, &mut tr_core::UnifiedRandom::new(0));
        assert!(n.despawn_timer >= 0, "EncourageDespawn(10) armed");
        assert_eq!(n.ai[0], 1.0, "the idle counter advances (parked)");
        assert_eq!(n.facing, 1, "no flip yet");
        // The 2-tick boundary: re-park the body (the ladder walked it)
        // and step again — ai[0] reaches 2, the direction flips + resets.
        n.vx = 0.0;
        n.vy = 0.0;
        fighter_core_step(&mut n, &w, &p, &mut tr_core::UnifiedRandom::new(0));
        assert_eq!(n.facing, -1, "the 2-tick flip");
        assert_eq!(n.ai[0], 0.0, "the counter resets");
    }

    /// The core draws ZERO rand for the claimed set OUTSIDE a graveyard:
    /// the moan lists name none of the ids, the bash's Next(60)
    /// short-circuits on the false pre-stamp (the W4-g4 lane-15 wiring),
    /// the ladder/scan are draw-free. The stream-clone pin replays the
    /// untouched-stream contract through the union signature.
    #[test]
    fn the_core_is_draw_free_for_the_claimed_set() {
        let mut w = ground();
        w.day_time = false;
        let mut n = skeleton_at(&w, 20);
        let mut p = crate::player::Player::spawn(&w);
        p.x = 30.0 * TILE_PX;
        p.y = 440.0 - crate::player::HEIGHT;
        let before = (n.x, n.y, n.vx, n.vy, n.ai);
        let mut rng = tr_core::UnifiedRandom::new(3);
        let pristine = rng.clone();
        fighter_core_step(&mut n, &w, &p, &mut rng);
        assert_ne!((n.x, n.vx), (before.0, before.2), "the machine moved");
        assert_eq!(rng, pristine, "outside a graveyard: zero draws (the census)");
    }

    /// W4-d22 reconcile — the graveyard gate rides the axis core too: a
    /// Skeleton (21, in flag24) bashing a door INSIDE the zone spends
    /// exactly one Next(60) on the caller stream (the W4-g4 lane-15
    /// contract through `fighter_core_step`). The fixture parks the
    /// skeleton at the door probe cell with the bash counter at 59.
    #[test]
    fn w4_d22_the_core_threads_the_graveyard_bash_draw() {
        let mut w = ground();
        w.day_time = false;
        // The door at the skeleton's ahead-probe cell: the skeleton is
        // 18x40 (kind.size), x = 319 → centre 328 + 15 = 343 → col 21;
        // the probe row (440 + 40 - 15)/16 = 29, the door at 28.
        let i = w.index(21, 28);
        w.tiles[i] = tr_core::tile::Tile { kind: 10, frame_x: 0, frame_y: 0, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.blood_moon = true;
        let mut n = skeleton_at(&w, 20);
        n.x = 319.0;
        n.vx = 1.0;
        n.ai[2] = 59.0;
        n.target_zone_graveyard = true;
        let mut p = crate::player::Player::spawn(&w);
        p.x = 30.0 * TILE_PX;
        p.y = 440.0 - crate::player::HEIGHT;
        // The live target: TargetClosest faces +1 (the player is right),
        // the num83 ladder accelerates vx 1.0 → 1.07 (under the 1.5 cap),
        // so the ahead probe stays on the door column and the bash fires.
        let mut rng = tr_core::UnifiedRandom::new(7);
        let mut mirror = rng.clone();
        let _ = mirror.next_max(60); // the ONE cited draw (:59997)
        fighter_core_step(&mut n, &w, &p, &mut rng);
        assert!(n.wants_door_bash.is_some(), "the bash fired");
        assert_eq!(rng, mirror, "exactly one Next(60) through the core");
    }

    /// `CanBeBusyWithAction` (`NPC.cs:60332-60339`): the listed types
    /// true, the plain walkers false.
    #[test]
    fn the_busy_gate_pins_the_c_list() {
        for t in [110u16, 111, 216, 350, 381, 385, 391, 411, 425, 520] {
            assert!(can_be_busy_with_action(t), "type {t} is listed");
        }
        for t in [3u16, 21, 26, 27, 28, 44, 212, 305, 326, 429, 518] {
            assert!(!can_be_busy_with_action(t), "type {t} is not listed");
        }
    }
}
