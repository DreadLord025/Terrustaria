//! Player-side `Gore.NewGore*` sites, wave E (FINAL): the melee-hitbox
//! sparkle ladder + the use-style-5 lantern mote
//! (`Player.cs:45048-51125`). W4-g17 (code-only mode): the module +
//! witnesses land first; the dispatch claims (the `ItemCheck_GetMelee
//! HitboxParticles` / `ItemCheck_ApplyUseStyle` lanes) are the wiring
//! wave — the `npc_hit_effect_a.rs` (W4-d55) precedent. With this
//! commit the Player.cs census (32 sites) is COMPLETE.
//!
//! ## Scope — this commit's sites (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 1-5 | `:45058`-`:45074` | item 1450 — the 415/414/413/412/411 ladder |
//! | 6 | `:51124` | useStyle 5 item 4262 — Next(570,573) mote |
//!
//! ## RNG contract (the player tick's `Main.rand` stream)
//!
//! * Sparkle ladder: the `Next(3)` outer gate ALWAYS draws; when it
//!   hits, the position rolls `Next(rect.Width)` + `Next(rect.Height)`
//!   (2) ride BEFORE the ladder gates. The ladder is an if/else-if
//!   CHAIN — each gate draws only when every earlier gate missed:
//!   `Next(500)` → 415, else `Next(250)` → 414, else `Next(80)` → 413,
//!   else `Next(10)` → 412, else `Next(3)` → 411. A hit spawns ONE
//!   gore with scale `Next(51,101)*0.01` (argument, before the kicks),
//!   then the kicks (2). The `num >= 0` post-write adds `direction*2`
//!   to X and scales Y ×0.3.
//!   NOTE: the 411-430 `NewGore` arm (`Gore.cs:1407-1421`) is
//!   asset-gated (`TextureAssets.Gore[Type].IsLoaded`) — the pool
//!   draws NOTHING for it (the `gore.rs:506-508` documented drop);
//!   the witness mirrors the pool's stream, not the asset-loaded C#.
//! * Lantern mote: the `DontHoldItems` mount gate is the caller's
//!   (`MountID.cs:46`: 55/56/61); the `Next(20)` gate ALWAYS draws;
//!   when it hits, `Next(570,573)` type FIRST, then the velocity's
//!   two `Next(-50,51)` multipliers, then the kicks (2). The 570-572
//!   arm RESTORES the passed velocity (`Gore.cs:1367-1370`) — the
//!   kicks are discarded, pool-side. The position nudges (-8 for 572,
//!   -4 for 571) are data.
//!
//! The post-spawn writes land on the returned slot, the sentinel
//! included (the `npc_hit_effect_a.rs` convention).
//!
//! ## Documented drops / stand-ins (this wave)
//!
//! * The ladder's `ItemCheck_GetMeleeHitbox` rect resolution
//!   (`:45052`) is the item lane's — the rect rides the state.
//! * The lantern's `itemLocation`/`MountedCenter` writes
//!   (`:51102-51103`) are the use-style lane's; the mote's Y reads
//!   the resolved `itemLocation.Y` (passed in). `Main.WindForVisuals`
//!   rides the state (`Main.cs:2823`, the `ambience.rs` port).
//! * netMode: single-player port, the pool's own convention.
//!
//! See docs/ANALYSIS.md "The player gore sites (W4-g17, wave E)".
#![allow(dead_code)] // wiring deferred (code-only mode) — the ItemCheck claims are the follow-up round

use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::{UnifiedRandom, World};

/// The melee-hitbox snapshot (`ItemCheck_GetMeleeHitboxParticles`,
/// `:45048-45081`).
#[derive(Clone, Copy, Debug)]
pub struct MeleeHitboxGoreState {
    pub rect_x: f32,
    pub rect_y: f32,
    pub rect_w: i32,
    pub rect_h: i32,
    pub direction: i32,
}

/// `Player.ItemCheck_GetMeleeHitboxParticles`' item-1450 arm
/// (`Player.cs:45050-45080`, sites 1-5): the `Next(3)` gate, the
/// position rolls, then the 500/250/80/10/3 if-else ladder spawning
/// ONE of 415/414/413/412/411 at scale `Next(51,101)*0.01`.
pub fn melee_hitbox_sparkle(
    state: &MeleeHitboxGoreState,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:45050` — the outer gate ALWAYS draws.
    if rng.next_max(3) != 0 {
        return;
    }
    // `:45054-45055` — the position rolls ride before the ladder.
    let x = state.rect_x + rng.next_max(state.rect_w) as f32;
    let y = state.rect_y + rng.next_max(state.rect_h) as f32;
    // `:45056-45075` — the if/else-if chain: each gate draws only
    // when every earlier gate missed.
    let ty = if rng.next_max(500) == 0 {
        415 // :45058
    } else if rng.next_max(250) == 0 {
        414 // :45062
    } else if rng.next_max(80) == 0 {
        413 // :45066
    } else if rng.next_max(10) == 0 {
        412 // :45070
    } else if rng.next_max(3) == 0 {
        411 // :45074
    } else {
        return;
    };
    // The scale argument draw rides before the kicks.
    let scale = rng.next_range(51, 101) as f32 * 0.01;
    let idx = gore.new_gore(rng, world, Vec2::new(x, y), Vec2::ZERO, ty, scale);
    // `:45076-45080` — the `num >= 0` post-write (the sentinel slot
    // included; a refused spawn returns 600 which ALSO passes >= 0).
    let g = gore.slot_mut(idx);
    g.velocity.x += state.direction as f32 * 2.0;
    g.velocity.y *= 0.3;
}

/// The lantern-mote snapshot (`ItemCheck_ApplyUseStyle` useStyle 5,
/// item 4262, `:51099-51125`).
#[derive(Clone, Copy, Debug)]
pub struct LanternMoteState {
    /// `base.Center` (`Entity.cs:50-60`).
    pub center_x: f32,
    /// The resolved `itemLocation.Y` (`:51103`: MountedCenter.Y - 6).
    pub item_location_y: f32,
    pub direction: i32,
    /// `Main.WindForVisuals` (`Main.cs:2823`).
    pub wind_for_visuals: f32,
}

/// `Player.ItemCheck_ApplyUseStyle`'s item-4262 mote (`Player.cs:51109-51125`,
/// site 6): the `Next(20)` gate, type `Next(570,573)`, the wind-driven
/// velocity with two `Next(-50,51)` multipliers, scale 0.8. The caller
/// owns the `flag` gate (mount active + `DontHoldItems`, `:51104-51108`).
pub fn lantern_mote_gore(
    state: &LanternMoteState,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:51109` — the gate ALWAYS draws.
    if rng.next_max(20) != 0 {
        return;
    }
    // `:51111-51123` — the type FIRST, then the position/velocity.
    let num17 = rng.next_range(570, 573);
    let mut x = state.center_x + (state.direction * 30) as f32 - 6.0;
    let y = state.item_location_y - 30.0;
    let mut vx = state.wind_for_visuals * 2.0 + state.direction as f32 * 0.3;
    let mut vy = -0.5f32;
    vx *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
    vy *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
    if num17 == 572 {
        x -= 8.0;
    }
    if num17 == 571 {
        x -= 4.0;
    }
    // `:51124` — the 570-572 arm restores this velocity pool-side
    // (`Gore.cs:1367-1370`); the kicks are discarded.
    gore.new_gore(rng, world, Vec2::new(x, y), Vec2::new(vx, vy), num17, 0.8);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn air() -> World {
        World::new("t".into(), 200, 200)
    }

    /// `Player.cs:45050-45080` — the ladder: the outer gate + position
    /// rolls, then the chain gates in order until one hits. The twin
    /// replays the exact short-circuit order.
    #[test]
    fn w4_g17_the_sparkle_ladder_short_circuits_in_order() {
        let w = air();
        let s = MeleeHitboxGoreState {
            rect_x: 100.0, rect_y: 200.0, rect_w: 32, rect_h: 32, direction: 1,
        };
        // Scan seeds for a ladder hit and verify the full draw order.
        let mut hit = None;
        for seed in 0..2000 {
            let mut p = UnifiedRandom::new(seed);
            if p.next_max(3) != 0 {
                continue;
            }
            let _px = p.next_max(32);
            let _py = p.next_max(32);
            // Replay the chain until a gate hits.
            let gates = [500, 250, 80, 10, 3];
            let mut landed = false;
            for g in gates {
                if p.next_max(g) == 0 {
                    landed = true;
                    break;
                }
            }
            if landed {
                hit = Some(seed);
                break;
            }
        }
        let h = hit.expect("a ladder-hit seed exists in 0..2000");
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(h);
        melee_hitbox_sparkle(&s, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(h);
        let _outer = twin.next_max(3);
        let px = twin.next_max(32);
        let py = twin.next_max(32);
        let gates = [(500, 415), (250, 414), (80, 413), (10, 412), (3, 411)];
        let mut ty = 0;
        for (g, t) in gates {
            if twin.next_max(g) == 0 {
                ty = t;
                break;
            }
        }
        let _scale = twin.next_range(51, 101);
        // NewGore writes the position EXACTLY (`Gore.cs:1325`); only
        // the velocity kicks draw (Y then X, `:1327-1328`).
        let _ky = twin.next_range(10, 31);
        let _kx = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek(), "the ladder mirrors the C# draws");
        assert!(gore.slot(0).active);
        assert_eq!(gore.slot(0).gore_type, ty);
        assert_eq!(
            gore.slot(0).position,
            Vec2::new(100.0 + px as f32, 200.0 + py as f32)
        );
        // The post-write: X carries the +direction*2 nudge.
        assert!(gore.slot(0).velocity.x >= 2.0 - 2.0);
    }

    /// An outer-gate miss draws exactly ONE roll and spawns nothing.
    #[test]
    fn w4_g17_the_sparkle_outer_gate_miss_draws_once() {
        let w = air();
        let s = MeleeHitboxGoreState {
            rect_x: 100.0, rect_y: 200.0, rect_w: 32, rect_h: 32, direction: 1,
        };
        let mut miss = None;
        for seed in 0..500 {
            let mut p = UnifiedRandom::new(seed);
            if p.next_max(3) != 0 {
                miss = Some(seed);
                break;
            }
        }
        let m = miss.expect("an outer-miss seed");
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(m);
        melee_hitbox_sparkle(&s, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(m);
        let _g = twin.next_max(3);
        assert_eq!(rng.peek(), twin.peek(), "one gate draw only");
        assert!(!gore.slots().iter().any(|g| g.active));
    }

    /// `Player.cs:51109-51124` — the lantern mote: the Next(20) gate,
    /// type FIRST, then the two velocity multipliers, then the kicks
    /// (which the 570-572 arm discards, restoring the passed velocity).
    #[test]
    fn w4_g17_the_lantern_mote_draws_type_then_multipliers() {
        let w = air();
        let s = LanternMoteState {
            center_x: 110.0, item_location_y: 215.0, direction: 1,
            wind_for_visuals: 0.5,
        };
        let mut hit = None;
        for seed in 0..500 {
            let mut p = UnifiedRandom::new(seed);
            if p.next_max(20) == 0 {
                hit = Some(seed);
                break;
            }
        }
        let h = hit.expect("a gate-hit seed");
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(h);
        lantern_mote_gore(&s, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(h);
        let _g = twin.next_max(20);
        let ty = twin.next_range(570, 573);
        let mx = twin.next_range(-50, 51);
        let my = twin.next_range(-50, 51);
        let _ky = twin.next_range(10, 31);
        let _kx = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek(), "the mote mirrors the C# draws");
        assert_eq!(gore.slot(0).gore_type, ty);
        assert!((570..=572).contains(&gore.slot(0).gore_type));
        // The velocity is the PASSED one (the 570-572 restore), with
        // the multiplier rolls applied.
        let vx = (0.5f32 * 2.0 + 0.3) * (1.0 + mx as f32 * 0.01);
        let vy = -0.5f32 * (1.0 + my as f32 * 0.01);
        assert_eq!(gore.slot(0).velocity, Vec2::new(vx, vy));
        assert_eq!(gore.slot(0).scale, 0.8);
        // The position nudge rides the type.
        let nudge = if ty == 572 { 8.0 } else if ty == 571 { 4.0 } else { 0.0 };
        assert_eq!(gore.slot(0).position, Vec2::new(110.0 + 30.0 - 6.0 - nudge, 185.0));
    }
}
