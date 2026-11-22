//! W4-d9 aiStyle 89 — the CultistRitual 490 marker (`AI()`'s inline arm
//! `Projectile.cs:32262-32386`). Census: aiStyles 82/83/84/85/88 were armed
//! on main; 89 was a gap (the row + id landed with the W4-d9 census commit).
//!
//! The marker machine:
//! * `:32264-32286` — the alpha ramp (`ai[1]==-1` the cancel fade +12, else
//!   −5 to 300 then +12), the scale `(1 - alpha/255) * 0.6`, the `PI/210`
//!   rotation step.
//! * `:32287-32291` — the first-tick sound latch (Item123).
//! * `:32292-32319` — the alpha-0 burst (two dusts, the `Next(2,4)` speed,
//!   the `Next(-10,11)`×2 direction, the `Next(8)` variant gate).
//! * `:32320-32345` — the `ai[0]++` and the 60-tick burst (the `Next(1,3)`
//!   speed, the `Next(-10,11)`×2 direction, the `Next(2)` variant gate).
//! * `:32346-32356` — the 300-tick spawn: `NPC.NewNPC` at the centre, the
//!   dragon head (454) if `!NPC.AnyNPCs(454)` else the squidhead (521).
//! * `:32357-32361` — the 320-tick kill.
//! * `:32362-32381` — the host-validity kill: `ai[1]==-1` dies at alpha 255;
//!   else the host NPC must be active and type 439/454/521.
//!
//! The dragon/squid spawn rides `p.wants_ritual_npc_spawn` (the pool's
//! NPC-vec drain, the `AI_103` precedent), carrying the VANILLA id 454/521
//! verbatim (`:32350`/`:32354`). The C# stores the returned NPC index into
//! `ai[1]`; the port's drain resolves the index (the wiring's job — the
//! request carries the spawn, the `ai[1]` write-back lands with the drain).
//!
//! W4-d9 WIRING: the dragon head (vanilla 454, the d40 7a88c61c/45dbafc3
//! rows) and the squidhead (521, ported from `NPC.cs:15768-15779`) now have
//! port `NpcKind`s. The pool's `drain_ritual_npc_spawns` resolves the ids
//! through `kind_from_net_id` (the bestiary NET-id space, the
//! `spawn_shimmer_release` precedent) and writes the spawned NPC's index
//! back into the marker's `ai[1]` — the host-validity arm below reads it.
//!
//! Movement: the marker is stationary (the C# `UpdatePosition` adds the
//! zero velocity); the dispatch runs the generic `kill_step` (tileCollide
//! false → the free-move branch, net zero).
//!
//! RNG contract (the `UpdateProjectiles` stream): the alpha-0 burst's
//! per-dust `Next(2,4)`/`Next(-10,11)`×2/`Next(8)` (`:32296-32317`) and the
//! 60-tick burst's `Next(1,3)`/`Next(-10,11)`×2/`Next(2)` (`:32327-32343`).
//! Dust-only, consumed in order. The spawn itself draws nothing.
//!
//! Code-only round (W4-d9): the witness tests below are WRITTEN NOT RUN.

#![allow(dead_code)] // wiring deferred (W4-d9, code-only mode)

use std::f32::consts::PI;

use glam::Vec2;
use tr_core::loot::DropRng;

use crate::npc::Npc;
use crate::projectile::Projectile;

/// `Kill()` (`Projectile.cs:70644-70648`) — the sibling-module deactivation
/// (the `projectile_ai_spear` precedent; `Projectile::kill` is private).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// `Entity.Center` for the pool's slots (`pos + size/2`).
fn center(p: &Projectile) -> Vec2 {
    Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5)
}

/// `AI()`'s aiStyle-89 arm (`Projectile.cs:32262-32386`). `npcs` is the live
/// NPC list (the host-validity read), `any_dragon` is `NPC.AnyNPCs(454)`,
/// `rng` the `UpdateProjectiles` stream. The dragon/squid spawn rides
/// `p.wants_ritual_npc_spawn`.
pub(crate) fn ai_cultist_ritual(
    p: &mut Projectile,
    npcs: &[Npc],
    any_dragon: bool,
    rng: &mut dyn DropRng,
) {
    // `:32264-32286` — the alpha/scale/rotation ramp.
    if p.ai[1] == -1.0 {
        p.alpha += 12;
    } else if p.ai[0] < 300.0 {
        p.alpha -= 5;
    } else {
        p.alpha += 12;
    }
    p.alpha = p.alpha.clamp(0, 255);
    p.scale = 1.0 - p.alpha as f32 / 255.0;
    p.scale *= 0.6;
    p.rot += PI / 210.0;
    // `:32287-32291` — the first-tick sound latch (Item123, style 2).
    if p.local_ai[1] == 0.0 {
        p.local_ai[1] = 1.0;
        p.wants_sound = Some((2, 123, p.pos.x, p.pos.y, 0.0));
    }
    // `:32292-32319` — the alpha-0 burst (skipped bodies, consumed draws).
    if p.alpha == 0 {
        for _ in 0..2 {
            let _ = rng.next_range(2, 4); // the speed
            let _ = rng.next_range(-10, 11); // the direction X
            let _ = rng.next_range(-10, 11); // the direction Y
            let _ = rng.next_range(0, 8); // the variant gate
        }
    }
    p.ai[0] += 1.0;
    // `:32321-32345` — the 60-tick burst (skipped bodies, consumed draws).
    if p.ai[0] >= 60.0 {
        let _ = rng.next_range(1, 3); // the speed
        let _ = rng.next_range(-10, 11); // the direction X
        let _ = rng.next_range(-10, 11); // the direction Y
        let _ = rng.next_range(0, 2); // the variant gate
    }
    // `:32346-32356` — the 300-tick dragon/squid spawn.
    if p.ai[0] == 300.0 && p.ai[1] != -1.0 {
        let c = center(p);
        // NPC.NewNPC's Bottom = (X, Y) (`NPC.cs:80770`); the dragon head
        // (454) if none alive, else the squidhead (521). The spawn rides
        // wants_ritual_npc_spawn (the pool's NPC-vec drain).
        let npc_type = if !any_dragon { 454u16 } else { 521u16 };
        p.wants_ritual_npc_spawn = Some((npc_type, c.x, c.y));
        // The C# stores the returned NPC index into ai[1]; the port's
        // drain resolves the index (the wiring's job — the request carries
        // the spawn, the ai[1] write-back lands with the drain).
    } else if p.ai[0] == 320.0 {
        // `:32357-32361` — the 320-tick kill.
        kill(p);
        return;
    }
    // `:32362-32381` — the host-validity kill.
    let flag43 = if p.ai[1] == -1.0 {
        p.alpha == 255
    } else {
        let host = npcs.get(p.ai[1] as usize);
        let invalid = p.ai[1] < 0.0 || host.map_or(true, |h| h.hp <= 0);
        // The C# reads `Main.npc[ai[1]].type` (the vanilla id 439/454/521);
        // the port's `net_id()` carries the vanilla id (the `to_u16` CS id
        // maps AncientCultist→112, NOT 439).
        let kind_ok = host.map_or(false, |h| matches!(h.kind.net_id(), 439 | 454 | 521));
        invalid || !kind_ok
    };
    if flag43 {
        kill(p);
    }
    // `:32384` — the light (skipped: the ProjLight seam owns lights).
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::proj_data;
    use tr_core::TILE_PX;

    struct Recorder {
        draws: Vec<(i32, i32)>,
    }
    impl DropRng for Recorder {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push((min, max));
            1
        }
        fn next_float(&mut self) -> f32 {
            0.5
        }
    }

    fn proj() -> Projectile {
        let d = proj_data(crate::projectile::CULTIST_RITUAL).expect("the 490 row");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = crate::projectile::CULTIST_RITUAL;
        p.owner = 255;
        p.friendly = false;
        p.pos = Vec2::new(10.0 * TILE_PX, 10.0 * TILE_PX);
        p.vel = Vec2::ZERO;
        p.w = d.w;
        p.h = d.h;
        p.damage = 0;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.alpha = d.alpha;
        p.tile_collide = d.tile_collide;
        p.net_important = d.net_important;
        p.ignore_water = d.ignore_water;
        p
    }

    fn host() -> Npc {
        let mut n = Npc::new(crate::npc::NpcKind::AncientCultist, 0.0, 0.0, 1, 0.0);
        n.hp = n.max_hp;
        n
    }

    /// The row matches `SetDefaults` verbatim (`Projectile.cs:5655-5666`).
    #[test]
    fn w4_d9_the_ritual_row_matches_set_defaults() {
        let d = proj_data(crate::projectile::CULTIST_RITUAL).unwrap();
        assert_eq!((d.w, d.h), (14.0, 14.0));
        assert_eq!(d.ai_style, 89);
        assert_eq!(d.alpha, 255);
        assert!(d.ignore_water);
        assert!(!d.tile_collide);
        assert_eq!(d.time_left, 600);
        assert!(d.net_important);
    }

    /// The 300-tick spawn (no dragon → 454) + the 320-tick kill.
    #[test]
    fn w4_d9_the_ritual_spawns_and_dies() {
        let mut p = proj();
        p.ai[1] = 0.0; // a valid host index
        let npcs = vec![host()];
        let mut rng = Recorder { draws: Vec::new() };
        for _ in 0..300 {
            ai_cultist_ritual(&mut p, &npcs, false, &mut rng);
        }
        let req = p.wants_ritual_npc_spawn.expect("the 300-tick spawn");
        assert_eq!(req.0, 454, "the dragon head");
        for _ in 0..20 {
            ai_cultist_ritual(&mut p, &npcs, false, &mut rng);
        }
        assert!(!p.active, "the ritual dies at 320");
    }

    /// The 300-tick spawn with a dragon alive → the squidhead 521.
    #[test]
    fn w4_d9_the_ritual_spawns_the_squidhead_when_a_dragon_lives() {
        let mut p = proj();
        p.ai[1] = 0.0;
        let npcs = vec![host()];
        let mut rng = Recorder { draws: Vec::new() };
        for _ in 0..300 {
            ai_cultist_ritual(&mut p, &npcs, true, &mut rng);
        }
        assert_eq!(p.wants_ritual_npc_spawn.expect("the 300-tick spawn").0, 521, "the squidhead");
    }

    /// The host-validity kill: a dead host kills the marker.
    #[test]
    fn w4_d9_the_ritual_host_validity_dead() {
        let mut p = proj();
        p.ai[1] = 0.0;
        let mut h = host();
        h.hp = 0;
        let npcs = vec![h];
        let mut rng = Recorder { draws: Vec::new() };
        ai_cultist_ritual(&mut p, &npcs, false, &mut rng);
        assert!(!p.active, "a dead host kills the marker");
    }

    /// The host-validity kill: a wrong-kind host kills the marker.
    #[test]
    fn w4_d9_the_ritual_host_validity_wrong_kind() {
        let mut p = proj();
        p.ai[1] = 0.0;
        let mut h = Npc::new(crate::npc::NpcKind::Zombie, 0.0, 0.0, 1, 0.0);
        h.hp = h.max_hp;
        let npcs = vec![h];
        let mut rng = Recorder { draws: Vec::new() };
        ai_cultist_ritual(&mut p, &npcs, false, &mut rng);
        assert!(!p.active, "a non-cultist host kills the marker");
    }

    /// The cancel form (ai[1]==-1): the alpha fades up and dies at 255.
    #[test]
    fn w4_d9_the_ritual_cancel_fades_out() {
        let mut p = proj();
        p.ai[1] = -1.0;
        p.alpha = 0;
        let npcs: Vec<Npc> = Vec::new();
        let mut rng = Recorder { draws: Vec::new() };
        // +12/tick from 0: 22 ticks reach 264 → clamp 255 → kill.
        for _ in 0..22 {
            ai_cultist_ritual(&mut p, &npcs, false, &mut rng);
        }
        assert!(!p.active, "the cancel form dies at alpha 255");
    }

    /// The first-tick sound latch (Item123).
    #[test]
    fn w4_d9_the_ritual_sound_latch() {
        let mut p = proj();
        p.ai[1] = 0.0;
        let npcs = vec![host()];
        let mut rng = Recorder { draws: Vec::new() };
        ai_cultist_ritual(&mut p, &npcs, false, &mut rng);
        assert_eq!(p.wants_sound, Some((2, 123, p.pos.x, p.pos.y, 0.0)));
        assert_eq!(p.local_ai[1], 1.0);
    }
}
