//! W4-d9 aiStyle 87 — the ClingerStaff 482's cursed-flame wall (`AI()`'s
//! inline arm `Projectile.cs:31901-31936`). Census: aiStyles 82/83/84/85/88
//! were armed on main; 87 was a gap (the row + id landed with the W4-d9
//! census commit).
//!
//! The wall machine:
//! * `:31903-31904` — the position/height pins: `position.Y = ai[0]`,
//!   `height = (int)ai[1]` (the spawn site seeds the wall's top + height).
//! * `:31905-31912` — face the owner: `direction = ±1` off the centre-X
//!   comparison against `Main.player[owner].Center.X`.
//! * `:31913` — the micro-velocity `velocity.X = direction * 1E-06f`.
//! * `:31914-31924` — the keep-one sibling kill (owner-gated): an active
//!   same-type same-owner projectile with a HIGHER timeLeft kills this one.
//! * `:31925-31935` — the dust curtain (the `(width*height)*0.0045` count,
//!   the 75 dust's noGravity/velocity/scale/position writes). No explicit
//!   `Main.rand` draws — the dust bodies are the skipped visual layer.
//!
//! The supporting seams (the C# sites outside `AI()`):
//! * the CursedInferno debuff — `StatusNPC`'s 482 arm (`:11665-11668`):
//!   buff 39 for 420 ticks.
//! * the flyer half-damage — `Damage_PVE_Inner`'s 482 clause (`:12951-
//!   12954`): `dmg /= 2` against aiStyle-6/37 NPCs.
//!
//! Movement: the wall pins `position.Y` every tick and the micro-velocity
//! X is negligible; the C# `UpdatePosition` (`:20445`) adds the velocity
//! (the wall is not in the skip list). The dispatch runs the generic
//! `kill_step` (tileCollide false → the free-move branch).
//!
//! RNG contract: ZERO explicit draws — the dust curtain rolls nothing.
//!
//! Code-only round (W4-d9): the witness tests below are WRITTEN NOT RUN.

#![allow(dead_code)] // wiring deferred (W4-d9, code-only mode)

use crate::npc::Npc;
use crate::projectile::{Projectile, CLINGER_STAFF};

/// `Kill()` (`Projectile.cs:70644-70648`) — the sibling-module deactivation
/// (the `projectile_ai_spear` precedent; `Projectile::kill` is private).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// `Entity.Center` for the pool's slots (`pos + size/2`).
fn center_x(p: &Projectile) -> f32 {
    p.pos.x + p.w * 0.5
}

/// `AI()`'s aiStyle-87 arm (`Projectile.cs:31901-31936`). `owner_center_x`
/// is `Main.player[owner].Center.X`; `sibling_kill` is the keep-one scan's
/// verdict (the pool resolves it — the AI sees only its slot). The dust
/// curtain rolls nothing explicit.
pub(crate) fn ai_clinger_staff(p: &mut Projectile, owner_center_x: f32, sibling_kill: bool) {
    // `:31903-31904` — the position/height pins.
    p.pos.y = p.ai[0];
    p.h = p.ai[1];
    // `:31905-31912` — face the owner.
    p.direction = if center_x(p) > owner_center_x { 1 } else { -1 };
    // `:31913` — the micro-velocity.
    p.vel.x = p.direction as f32 * 1e-6;
    // `:31914-31924` — the keep-one sibling kill (owner-gated in C#; the
    // pool's local-owner gate folds true for the single-player port).
    if sibling_kill {
        kill(p);
        return;
    }
    // `:31925-31935` — the dust curtain (skipped: no explicit draws).
}

/// The keep-one sibling scan (`Projectile.cs:31916-31923`): an active
/// same-type same-owner projectile with a HIGHER timeLeft kills this one.
/// `who_am_i` is this slot's index. The pool calls it with the slot array.
pub(crate) fn clinger_staff_sibling_kill(slots: &[Projectile], who_am_i: usize) -> bool {
    let me = &slots[who_am_i];
    slots.iter().enumerate().any(|(i, other)| {
        other.active
            && i != who_am_i
            && other.kind == me.kind
            && other.owner == me.owner
            && other.time_left > me.time_left
    })
}

/// `StatusNPC`'s type-482 arm (`Projectile.cs:11665-11668`): CursedInferno
/// (buff 39) for 420 ticks. Called from the damage walk's StatusNPC seam.
pub(crate) fn clinger_staff_status_npc(npc: &mut Npc) {
    npc.buffs.add_buff(39, 420);
}

/// `Damage_PVE_Inner`'s type-482 clause (`Projectile.cs:12951-12954`): the
/// wall halves its damage against aiStyle-6/37 NPCs (the flyers). Returns
/// the adjusted damage.
pub(crate) fn clinger_staff_damage(p: &Projectile, npc: &Npc, dmg: i32) -> i32 {
    if p.kind == CLINGER_STAFF && matches!(npc.kind.ai_style(), 6 | 37) {
        dmg / 2
    } else {
        dmg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::proj_data;
    use glam::Vec2;
    use tr_core::TILE_PX;

    fn proj() -> Projectile {
        let d = proj_data(CLINGER_STAFF).expect("the 482 row");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = CLINGER_STAFF;
        p.owner = 0;
        p.pos = Vec2::new(10.0 * TILE_PX, 10.0 * TILE_PX);
        p.vel = Vec2::ZERO;
        p.w = d.w;
        p.h = d.h;
        p.damage = 40;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.alpha = d.alpha;
        p.tile_collide = d.tile_collide;
        p.magic = d.magic;
        p.uses_id_static_immunity = d.uses_id_static_immunity;
        p.id_static_hit_cooldown = d.id_static_hit_cooldown;
        p.ignore_water = d.ignore_water;
        p
    }

    /// The row matches `SetDefaults` verbatim (`Projectile.cs:5572-5586`).
    #[test]
    fn w4_d9_the_clinger_row_matches_set_defaults() {
        let d = proj_data(CLINGER_STAFF).unwrap();
        assert_eq!((d.w, d.h), (16.0, 200.0));
        assert_eq!(d.ai_style, 87);
        assert!(!d.tile_collide);
        assert_eq!(d.penetrate, -1);
        assert_eq!(d.alpha, 255);
        assert!(d.ignore_water);
        assert_eq!(d.time_left, 18000);
        assert!(d.magic);
        assert!(d.uses_id_static_immunity);
        assert_eq!(d.id_static_hit_cooldown, 10);
    }

    /// The wall pins position.Y/height, faces the owner, sets the micro-X.
    #[test]
    fn w4_d9_the_clinger_wall_pins() {
        let mut p = proj();
        p.ai[0] = 500.0;
        p.ai[1] = 300.0;
        p.pos.x = 100.0;
        ai_clinger_staff(&mut p, 200.0, false);
        assert_eq!(p.pos.y, 500.0);
        assert_eq!(p.h, 300.0);
        assert_eq!(p.direction, -1, "the wall faces the owner to its right");
        assert!((p.vel.x - -1e-6).abs() < 1e-9);
        // The keep-one kill.
        ai_clinger_staff(&mut p, 200.0, true);
        assert!(!p.active);
    }

    /// The keep-one scan: a higher-timeLeft sibling kills this one.
    #[test]
    fn w4_d9_the_clinger_keep_one_scan() {
        let mut older = proj();
        older.time_left = 100;
        let mut newer = proj();
        newer.time_left = 200;
        let slots = vec![older.clone(), newer.clone()];
        assert!(clinger_staff_sibling_kill(&slots, 0), "the older wall dies");
        assert!(!clinger_staff_sibling_kill(&slots, 1), "the newer wall lives");
        let _ = (&mut older, &mut newer);
    }

    /// The CursedInferno debuff: buff 39 for 420 ticks.
    #[test]
    fn w4_d9_the_clinger_applies_cursed_inferno() {
        let mut n = Npc::new(crate::npc::NpcKind::Zombie, 0.0, 0.0, 1, 0.0);
        clinger_staff_status_npc(&mut n);
        assert!(n.buffs.find_buff_index(39).is_some(), "the 39 buff landed");
    }

    /// The flyer half-damage: aiStyle-6/37 NPCs take half.
    #[test]
    fn w4_d9_the_clinger_flyer_half_damage() {
        let p = proj();
        // The Devourer head is aiStyle 6 (a flyer/worm).
        let flyer = Npc::new(crate::npc::NpcKind::DevourerHead, 0.0, 0.0, 1, 0.0);
        assert_eq!(flyer.kind.ai_style(), 6);
        assert_eq!(clinger_staff_damage(&p, &flyer, 40), 20);
        // A walker (aiStyle 3) takes full.
        let walker = Npc::new(crate::npc::NpcKind::Zombie, 0.0, 0.0, 1, 0.0);
        assert_eq!(walker.kind.ai_style(), 3);
        assert_eq!(clinger_staff_damage(&p, &walker, 40), 40);
    }
}
