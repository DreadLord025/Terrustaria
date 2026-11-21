//! W4-f20 F7 — the Harpy-charm chain: the ai[2]-bits PRODUCER and the
//! AI_001 tail it feeds (census research/26f F7). The 26f first pass
//! recorded the tail as inert-unported ("the ai[2] bits stay 0"); this
//! round censused the WRITER and found it unclaimed-small: the producer
//! is NOT item-use — it is the per-tick `Player.Update_HarpyCharm`
//! accessory scan (`Player.cs:24565` → `:28436-28490`), so the whole
//! chain ports together (the F3 "row+arm together" pattern applied to
//! producer+tail).
//!
//! The chain, in the C#'s own order:
//! 1. the equip grants (`ApplyEquipFunctional` `Player.cs:14490-14493`
//!    6166 Harpy Charm / `:14588-14592` 6180 Seraph Necklace /
//!    `:14593-14598` 6181 Phoenix Quiver) arm `accHarpyCharm` (+ the
//!    seraph/phoenix halves); `ResetEffects` clears them per tick
//!    (`:18471-18472`, `:18609`) — the port's `player.rs` arms.
//! 2. [`update_harpy_charm`] (`Player.cs:28436-28446`): the cooldown
//!    decrements while positive, else [`harpy_charm_effect`] runs.
//! 3. [`harpy_charm_effect`] (`:28448-28490`): the 1000-slot scan picks
//!    the owner's NEAREST uncharmed aiStyle-1 arrow 400..1200 px out
//!    with a 1600-px line-of-sight target, then [`apply_harpy_charm`].
//! 4. [`apply_harpy_charm`] (`Projectile.cs:57796-57824`): the π turn
//!    toward the target, the ai[2] bit writes (HarpyCharmed +
//!    PhoenixQuivered/SeraphNecklaced per the itemID), the 300-tick
//!    timeLeft cap. The `NetMessage.SendData(27)` (`:57818`) folds to
//!    `net_update = true` (the pool's sync convention); the
//!    `ParticleOrchestrator` request (`:57819-57823`) is the particle
//!    layer's skip (no port layer, draws nothing).
//! 5. [`charm_tail`] (`Projectile.cs:57769-57793`): the AI_001 tail —
//!    while charmed-and-unconsumed, the per-sub-step π turn toward the
//!    1600-px LOS target, the <1 normalize and the <12 ×1.05 accel; a
//!    lost target sets the Consumed bit (the charm is spent).
//!
//! The ai[2] bits are the `AI_001_Features` FloatIntUnion
//! (`Projectile.cs:239-290`): bit 0 HarpyCharmed, bit 1
//! HarpyCharmConsumed, bit 2 PhoenixQuivered, bit 3 SeraphNecklaced —
//! the low four mantissa bits of the f32, so the small-integer bit
//! patterns survive the float round-trip exactly (1.4e-45 .. 1.1e-44,
//! far from any arithmetic the ai[2] slot sees elsewhere).
//!
//! RNG: the producer and the tail draw NOTHING (`FindTargetWithLineOf-
//! Sight` is a deterministic scan, `rotateTowards` pure trig) — the
//! `UpdatePlayers`/`UpdateProjectiles` streams are untouched BY
//! CONSTRUCTION. The PhoenixQuivered dust gate (`Projectile.cs:54005-
//! 54011`, the `Next(3)` + the `NextVector2Circular(3,3)` pair + the
//! `NextFloat` scale) is the ONE draw site the bits feed; it stays
//! SKIPPED with the dust layer (the ai001_a/b/c/d/e convention — the
//! 26f riders census row), so the bit-2 write is state-only here.
//!
//! Wiring: [`charm_tail`] rides the END of every aiStyle-1 flight body
//! (after the `flag8` clamp `:57753-57768`, before the movement — the
//! C#'s `:57769` slot): the generic `ai_style1_flight`, the slice-A/D/E
//! `finish` tails, `special_arrows::step`, the moonlord `finish_flight`
//! and the darts step. The early-return paths (the 709 age kill, the
//! 640 settle kill, the 442 arrival) skip it exactly as the C# `return`
//! exits AI_001 before `:57769`. The producer rides `update_player`
//! after `update_snake_band` (the C# `:24565` < `:24567` order — the
//! port's split-frame fold, the `grant_ammo_mastery` precedent).

use glam::Vec2;
use tr_core::World;

use crate::npc::Npc;
use crate::player::Player;
use crate::projectile::{
    find_target_with_line_of_sight_pub, npc_center, proj_data, rotate_towards, safe_normalize_or,
    Projectile, ProjectilePool,
};

/// `AI_001_Features.HarpyCharmed` — bit 0 (`Projectile.cs:243-253`).
pub(crate) const HARPY_CHARMED: u32 = 1 << 0;
/// `AI_001_Features.HarpyCharmConsumed` — bit 1 (`:255-265`).
pub(crate) const HARPY_CHARM_CONSUMED: u32 = 1 << 1;
/// `AI_001_Features.PhoenixQuivered` — bit 2 (`:267-277`).
pub(crate) const PHOENIX_QUIVERED: u32 = 1 << 2;
/// `AI_001_Features.SeraphNecklaced` — bit 3 (`:279-289`).
pub(crate) const SERAPH_NECKLACED: u32 = 1 << 3;

/// The `FloatIntUnion` read: the ai[2] float's bit pattern.
pub(crate) fn bits(p: &Projectile) -> u32 {
    p.ai[2].to_bits()
}

/// `Player.Update_HarpyCharm` (`Player.cs:28436-28446`): the cooldown
/// decrements while positive; at zero the effect scan runs. Called from
/// `Player.Update`'s `Main.myPlayer == i` block (`:24558-24570`), between
/// `Update_SnappingStone` (`:24564`, unported — the snapping-stone lane)
/// and `Update_AmmoCycler` (`:24566`).
pub(crate) fn update_harpy_charm(
    p: &mut Player,
    pool: &mut ProjectilePool,
    world: &World,
    npcs: &[Npc],
    local_owner: u8,
) {
    if p.acc_harpy_charm_cooldown > 0 {
        p.acc_harpy_charm_cooldown -= 1;
    } else {
        harpy_charm_effect(p, pool, world, npcs, local_owner);
    }
}

/// `Player.HarpyCharmEffect` (`Player.cs:28448-28490`): the nearest-arrow
/// charm scan. The constants are the C#'s: 400 px minimum, 1200 px
/// maximum, 1600 px target range, 90-tick cooldown (Seraph ×0.8 → 72).
/// The scan walks the pool in slot order (`for i < 1000`); the
/// `!(num7 < num) && !(num7 >= num6) && !(num7 > num2)` gate is the
/// `[400, min(1200, best))` window — the strict `< num6` keeps the FIRST
/// (lowest-slot) arrow on a distance tie, exactly the C#'s.
fn harpy_charm_effect(
    p: &mut Player,
    pool: &mut ProjectilePool,
    world: &World,
    npcs: &[Npc],
    local_owner: u8,
) {
    // `:28450-28453` — the flag/cooldown gate.
    if !p.acc_harpy_charm || p.acc_harpy_charm_cooldown > 0 {
        return;
    }
    let (min_dist, max_dist, target_range) = (400.0f32, 1200.0, 1600.0);
    // `:28457-28461` — the Seraph cooldown cut: `(int)(90f * 0.8f)` = 72.
    let cooldown = if p.acc_seraph_necklace {
        (90.0f32 * 0.8) as i32
    } else {
        90
    };
    let (px, py) = p.center();
    let player_center = Vec2::new(px, py);
    let mut best_slot: i32 = -1;
    let mut best_target: i32 = -1;
    let mut best_dist = f32::MAX;
    for (i, q) in pool.slots().iter().enumerate() {
        // `:28468` — active && arrow && owner && aiStyle 1 && !HarpyCharmed.
        if !q.active || !q.arrow || q.owner != local_owner {
            continue;
        }
        if proj_data(q.kind).map_or(0, |d| d.ai_style) != 1 {
            continue;
        }
        if bits(q) & HARPY_CHARMED != 0 {
            continue;
        }
        let qc = Vec2::new(q.pos.x + q.w * 0.5, q.pos.y + q.h * 0.5);
        // `:28472-28473` — `base.Distance(projectile.Center)`: the C#
        // `Entity.Distance` is the centre-to-centre length.
        let dist = player_center.distance(qc);
        if dist < min_dist || dist >= best_dist || dist > max_dist {
            continue;
        }
        // `:28475` — the arrow's own 1600-px LOS scan (the local-immunity
        // slice rides the pool's flattened array, `:45645`).
        let target = find_target_with_line_of_sight_pub(
            world,
            npcs,
            q.pos,
            q.w,
            q.h,
            target_range,
            pool.local_immunity_slice(i),
        );
        if target >= 0 {
            best_dist = dist;
            best_slot = i as i32;
            best_target = target;
        }
    }
    if best_slot >= 0 {
        // `:28486` — the itemID pick: Phoenix 6181 > Seraph 6180 > 6166.
        let item_id = if p.has_phoenix_quiver {
            6181
        } else if p.acc_seraph_necklace {
            6180
        } else {
            6166
        };
        apply_harpy_charm(
            &mut pool.slots_mut()[best_slot as usize],
            npcs,
            best_target as usize,
            item_id,
        );
        // `:28488` — the cooldown arms only on a landed charm.
        p.acc_harpy_charm_cooldown = cooldown;
    }
}

/// `Projectile.ApplyHarpyCharm` (`Projectile.cs:57796-57824`): the π
/// turn toward the target (`Utils.rotateTowards`, `:57801`), the
/// `velocity *= 1f` no-op (`:57802`), the bit writes (`:57803-57813`)
/// and the 300-tick life cap (`:57814-57817`). The `SendData(27)`
/// (`:57818`) folds to `net_update`; the particle request (`:57819-
/// 57823`) is the particle-layer skip (no draws).
pub(crate) fn apply_harpy_charm(p: &mut Projectile, npcs: &[Npc], target: usize, item_id: u16) {
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    // `:57798-57801` — maxChange π, the turn toward the NPC centre.
    p.vel = rotate_towards(center, p.vel, npc_center(&npcs[target]), std::f32::consts::PI);
    // `:57802` — `velocity *= 1f` (the C# no-op, kept for the cite).
    p.vel *= 1.0;
    // `:57803-57813` — the bit writes: HarpyCharmed always; the
    // PhoenixQuivered bit iff itemID 6181, SeraphNecklaced iff 6180.
    let mut b = bits(p) | HARPY_CHARMED;
    if item_id == 6181 {
        b |= PHOENIX_QUIVERED;
    }
    if item_id == 6180 {
        b |= SERAPH_NECKLACED;
    }
    p.ai[2] = f32::from_bits(b);
    // `:57814-57817` — the life cap.
    if p.time_left > 300 {
        p.time_left = 300;
    }
    // `:57818` — the sync send folds to the flag.
    p.net_update = true;
}

/// `AI_001`'s Harpy-charm tail (`Projectile.cs:57769-57793`): the
/// per-sub-step steer for a charmed, unconsumed arrow. Runs at the END
/// of AI_001 — after the `flag8` clamp (`:57753-57768`), before
/// `HandleMovement` — so every aiStyle-1 flight body calls it as its
/// last AI-side step. The early-return paths (the C# `Kill(); return`
/// arms) skip it exactly as the C# `return` exits before `:57769`.
///
/// `:57769-57772` — `!HarpyCharmed || HarpyCharmConsumed` → return (the
/// no-accessory case: ai[2] stays 0, the tail is a two-bit test).
/// `:57773-57787` — the 1600-px LOS retarget; on a target the π turn,
/// then the <1 `SafeNormalize(UnitY)` and the <12 ×1.05 accel.
/// `:57788-57793` — no target: the Consumed bit sets (the charm is
/// spent — the arrow flies on unsteered) + `netUpdate`.
pub(crate) fn charm_tail(p: &mut Projectile, world: &World, npcs: &[Npc], immune: &[i32]) {
    let b = bits(p);
    if b & HARPY_CHARMED == 0 || b & HARPY_CHARM_CONSUMED != 0 {
        return;
    }
    let target = find_target_with_line_of_sight_pub(world, npcs, p.pos, p.w, p.h, 1600.0, immune);
    if target >= 0 {
        let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
        // `:57778` — the π turn (maxChange = π, `:57774`).
        p.vel = rotate_towards(
            center,
            p.vel,
            npc_center(&npcs[target as usize]),
            std::f32::consts::PI,
        );
        // `:57779-57782` — the <1 normalize (the UnitY default).
        if p.vel.length() < 1.0 {
            p.vel = safe_normalize_or(p.vel, Vec2::Y);
        }
        // `:57783-57786` — the <12 accel.
        if p.vel.length() < 12.0 {
            p.vel *= 1.05;
        }
    } else {
        // `:57790-57792` — the Consumed bit + the sync flag.
        p.ai[2] = f32::from_bits(b | HARPY_CHARM_CONSUMED);
        p.net_update = true;
    }
}

// =====================================================================
// Witness tests — WRITTEN, NOT RUN (code-only mode: no cargo/rustc).
// Every expected value below is hand-derived from the cited C# lines;
// the RNG-order argument is by construction (the module takes no rng).
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::SlimeVariant;
    use crate::projectile::{FlailInputs, YoyoInputs, WOODEN_ARROW};

    fn world() -> World {
        World::new("t".into(), 256, 128)
    }

    /// A player centred at (1000, 800) with the charm flag armed.
    fn charmer(w: &World) -> Player {
        let mut p = Player::spawn(w);
        // centre = x + width/2; pin the centre directly.
        p.x = 1000.0 - p.width * 0.5;
        p.y = 800.0 - p.height * 0.5;
        p.acc_harpy_charm = true;
        p
    }

    /// A blue slime (life 25 > 5, chaseable) centred at (cx, cy).
    fn slime_at(cx: f32, cy: f32) -> Npc {
        let mut n = Npc::spawn_slime_sky(0.0, 0.0, SlimeVariant::Blue);
        let (w, h) = n.size();
        n.x = cx - w * 0.5;
        n.y = cy - h * 0.5;
        n
    }

    /// A wooden arrow (arrow=true, aiStyle 1) at the given centre, flying
    /// right at `speed`, owner 0, timeLeft 1200.
    fn arrow_at(pool: &mut ProjectilePool, cx: f32, cy: f32, speed: f32) -> usize {
        pool.spawn_with_data_ai_owned(
            WOODEN_ARROW,
            0,
            cx - 5.0,
            cy - 5.0,
            speed,
            0.0,
            10,
            2.0,
            0.0,
            0.0,
        )
        .expect("the 1 row")
    }

    /// The producer charms the nearest eligible arrow: the 400..1200 px
    /// window, the 1600-px LOS target, the ai[2] bit-0 write, the π turn
    /// and the 300-tick life cap (`Player.cs:28448-28490`,
    /// `Projectile.cs:57796-57824`).
    #[test]
    fn w4_f20_f7_the_producer_charms_the_nearest_arrow_in_the_window() {
        let w = world();
        let mut p = charmer(&w);
        let mut pool = ProjectilePool::new();
        // Too close (< 400): never charmed.
        let near = arrow_at(&mut pool, 1200.0, 800.0, 6.0); // 200 px out
        // In the window (600 px), with a target 300 px beyond it.
        let mid = arrow_at(&mut pool, 1600.0, 800.0, 6.0);
        // Farther (900 px) — the nearer mid arrow wins.
        let far = arrow_at(&mut pool, 1900.0, 800.0, 6.0);
        let npcs = vec![slime_at(1900.0, 800.0)];
        update_harpy_charm(&mut p, &mut pool, &w, &npcs, 0);
        assert_eq!(bits(&pool.slots()[near]), 0, "the <400 px arrow stays uncharmed");
        assert_eq!(bits(&pool.slots()[far]), 0, "the farther arrow loses to the nearer");
        let q = &pool.slots()[mid];
        assert_eq!(bits(q) & HARPY_CHARMED, HARPY_CHARMED, ":57804 — bit 0");
        assert_eq!(bits(q) & PHOENIX_QUIVERED, 0, "no phoenix quiver");
        assert_eq!(bits(q) & SERAPH_NECKLACED, 0, "no seraph necklace");
        // The π turn: the arrow flies +X, the target is +X — no turn.
        assert_eq!(q.vel, Vec2::new(6.0, 0.0), ":57801 — the turn keeps the heading");
        assert_eq!(q.time_left, 300, ":57814-57817 — the 1200 life caps at 300");
        assert!(q.net_update, ":57818 — the sync flag");
        assert_eq!(p.acc_harpy_charm_cooldown, 90, ":28488 — the 90-tick cooldown");
    }

    /// The Seraph necklace cuts the cooldown 90 → 72 and sets bit 3; the
    /// Phoenix quiver sets bit 2 (`Player.cs:28458-28461`/`:28486`,
    /// `Projectile.cs:57805-57812`).
    #[test]
    fn w4_f20_f7_the_seraph_and_phoenix_halves_write_their_bits() {
        let w = world();
        let mut pool = ProjectilePool::new();
        let npcs = vec![slime_at(1900.0, 800.0)];
        // Seraph: cooldown 72, bit 3.
        let mut p = charmer(&w);
        p.acc_seraph_necklace = true;
        let s = arrow_at(&mut pool, 1600.0, 800.0, 6.0);
        update_harpy_charm(&mut p, &mut pool, &w, &npcs, 0);
        assert_eq!(p.acc_harpy_charm_cooldown, 72, "(int)(90 * 0.8) = 72");
        assert_eq!(bits(&pool.slots()[s]) & SERAPH_NECKLACED, SERAPH_NECKLACED);
        // Phoenix: cooldown 90, bit 2 (the phoenix pick wins over seraph).
        let mut p2 = charmer(&w);
        p2.has_phoenix_quiver = true;
        p2.acc_seraph_necklace = true;
        let mut pool2 = ProjectilePool::new();
        let ph = arrow_at(&mut pool2, 1600.0, 800.0, 6.0);
        update_harpy_charm(&mut p2, &mut pool2, &w, &npcs, 0);
        assert_eq!(p2.acc_harpy_charm_cooldown, 72, "the seraph cut still applies");
        let b = bits(&pool2.slots()[ph]);
        assert_eq!(b & PHOENIX_QUIVERED, PHOENIX_QUIVERED, "the 6181 pick");
        assert_eq!(b & SERAPH_NECKLACED, 0, "the phoenix pick is exclusive");
    }

    /// The cooldown gates the scan: while positive it decrements and no
    /// charm lands; at zero the scan re-arms (`Player.cs:28438-28445`).
    #[test]
    fn w4_f20_f7_the_cooldown_gates_the_recharm() {
        let w = world();
        let mut p = charmer(&w);
        p.acc_harpy_charm_cooldown = 2;
        let mut pool = ProjectilePool::new();
        let a = arrow_at(&mut pool, 1600.0, 800.0, 6.0);
        let npcs = vec![slime_at(1900.0, 800.0)];
        update_harpy_charm(&mut p, &mut pool, &w, &npcs, 0);
        assert_eq!(p.acc_harpy_charm_cooldown, 1, "the decrement");
        assert_eq!(bits(&pool.slots()[a]), 0, "no charm while cooling");
        update_harpy_charm(&mut p, &mut pool, &w, &npcs, 0);
        assert_eq!(p.acc_harpy_charm_cooldown, 0);
        assert_eq!(bits(&pool.slots()[a]), 0, "the zero tick still decrements to 0 first");
        // The NEXT tick (cooldown already 0) runs the scan.
        update_harpy_charm(&mut p, &mut pool, &w, &npcs, 0);
        assert_eq!(bits(&pool.slots()[a]) & HARPY_CHARMED, HARPY_CHARMED);
        assert_eq!(p.acc_harpy_charm_cooldown, 90, "the re-arm");
    }

    /// The tail steers a charmed arrow at the target: the π turn, the <12
    /// ×1.05 accel (`Projectile.cs:57773-57787`). A lost target sets the
    /// Consumed bit (`:57788-57793`).
    #[test]
    fn w4_f20_f7_the_tail_steers_and_consumes_on_a_lost_target() {
        let w = world();
        let mut pool = ProjectilePool::new();
        let a = arrow_at(&mut pool, 1600.0, 800.0, 6.0);
        // Charm it by hand (bit 0), the target ABOVE the arrow.
        pool.slots_mut()[a].ai[2] = f32::from_bits(HARPY_CHARMED);
        let npcs = vec![slime_at(1600.0, 500.0)];
        let immune = [0i32; 256];
        charm_tail(&mut pool.slots_mut()[a], &w, &npcs, &immune);
        let q = &pool.slots()[a];
        // The π turn points the velocity straight up (−Y): (0, −6).
        assert!((q.vel.x - 0.0).abs() < 1e-4, "the turn faces the target: {:?}", q.vel);
        assert!((q.vel.y - (-6.0 * 1.05)).abs() < 1e-4, "the <12 accel: {:?}", q.vel);
        assert_eq!(bits(q) & HARPY_CHARM_CONSUMED, 0, "the target holds");
        // Now the target is gone (empty NPC slice): the Consumed bit sets.
        charm_tail(&mut pool.slots_mut()[a], &w, &[], &immune);
        let q = &pool.slots()[a];
        assert_eq!(bits(q) & HARPY_CHARM_CONSUMED, HARPY_CHARM_CONSUMED, ":57790");
        assert!(q.net_update, ":57792");
        // A consumed arrow no longer steers (the early return).
        let vel_before = q.vel;
        charm_tail(&mut pool.slots_mut()[a], &w, &npcs, &immune);
        assert_eq!(pool.slots()[a].vel, vel_before, "the consumed tail is inert");
    }

    /// The no-charm case: ai[2] = 0 → the tail is a two-bit no-op (the
    /// behaviour-correct no-accessory path, census 26f F7's inert claim).
    #[test]
    fn w4_f20_f7_the_uncharmed_arrow_tail_is_inert() {
        let w = world();
        let mut pool = ProjectilePool::new();
        let a = arrow_at(&mut pool, 1600.0, 800.0, 6.0);
        let npcs = vec![slime_at(1600.0, 500.0)];
        let immune = [0i32; 256];
        let before = pool.slots()[a].vel;
        charm_tail(&mut pool.slots_mut()[a], &w, &npcs, &immune);
        assert_eq!(pool.slots()[a].vel, before, "no charm, no steer");
        assert_eq!(pool.slots()[a].ai[2], 0.0, "the bits stay 0");
    }

    /// The pool seam: a charmed arrow steers through the live dispatch
    /// (the generic `ai_arrow` → `ai_style1_flight` → `charm_tail` chain).
    #[test]
    fn w4_f20_f7_the_seam_steers_a_charmed_arrow_through_the_dispatch() {
        let mut w = world();
        let mut pool = ProjectilePool::new();
        let a = arrow_at(&mut pool, 1600.0, 800.0, 6.0);
        pool.slots_mut()[a].ai[2] = f32::from_bits(HARPY_CHARMED);
        let mut npcs = vec![slime_at(1600.0, 500.0)];
        let mut rng = tr_core::tile_light::FastRandom::new(7);
        pool.update(
            &mut w,
            &mut npcs,
            None,
            None,
            None,
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut rng,
            0,
            true,
        );
        let q = &pool.slots()[a];
        // The tail turned the velocity toward the target (−Y) and the
        // flight integrated it: the arrow moved UP, not right.
        assert!(q.vel.y < 0.0, "the charm steer: {:?}", q.vel);
        assert!(q.pos.y < 800.0 - 5.0, "the upward move: {:?}", q.pos);
    }
}