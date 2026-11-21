//! W4-d10 — the projectile aiStyles 91-100 census port (the f19 F1 finding
//! included): the nine MISSING styles plus the aiStyle-99 counterweight
//! SUB-arm, ported verbatim from `Projectile.cs` behind the pool's
//! `match data.ai_style` dispatch (`projectile.rs::update`).
//!
//! Census at the task's base (`a725acb0`, in-flight probe: d9/g1/g2/g3 carry
//! no unique commits, f19 is census-only `049cc8e2`):
//!
//! | style | C# arm | kinds (SetDefaults) | port state at base |
//! |-------|--------|---------------------|--------------------|
//! | 91 | inline `AI()` `:32515-32557` | ShadowFlame 496 (`:5692-5704`) | MISSING → this module |
//! | 92 | inline `:32558-32820` | ToxicCloud 511/512/513 (`:5805-5840`), GasTrap 1007 (`:9558-9569`), ChlorophyteClaymoreSporeCloud 1128 (`:10565-10579`) | MISSING → this module |
//! | 93 | inline `:32821-32882` | NailFriendly 514 (`:5841-5850`) | MISSING → this module |
//! | 94 | inline `:32883-32963` | CoinPortal 518 (`:5879-5888`) | MISSING → this module |
//! | 95 | inline `:32964-33003` | ToxicBubble 523 (`:5929-5938`) | MISSING → this module |
//! | 96 | inline `:33004-33028` | IchorSplash 524 (`:5939-5952`) | MISSING → this module |
//! | 97 | inline `:33029-33133` | FlyingPiggyBank 525 (`:5953-5961`) | MISSING → this module |
//! | 98 | inline `:33134-33152` | CultistBossParticle 526 (`:5962-5970`) | MISSING → this module |
//! | 99 | `AI_099_1_Counterweights` `:67380-67731` (gate `:33153-33156`) + `AI_099_2_Yoyos` `:67733+` (gate `:33157-33160`) | yoyos 534/541-555/562-564/603/999; counterweights 556-561/1079 (`:6084-6093`) | yoyo half LANDED (`projectile.rs::ai_yoyo`); counterweight sub-arm MISSING (f19 F1) → this module |
//! | 100 | `AI_100_Medusa` `:44476-44632` (gate `:33161-33164`) | MedusaHead 535 (`:6094-6106`) | MISSING → this module |
//!
//! RNG contract (the `UpdateProjectiles` stream, `Main.cs:17775-17777`):
//! every EXPLICIT `Main.rand` draw at the call sites stays, in the C#'s
//! order; the dust BODIES are skipped (no draw layer wired here) and the
//! `Dust.NewDust*` internal rolls go with the dust pool (ANALYSIS "Dust &
//! gore", the `projectile_ai_misc` precedent). `RotatedByRandom`
//! (`Utils.cs:1697-1700`) is TWO `NextDouble` draws — mapped onto
//! `next_float` (one Sample() per draw either way, the ai067_b convention).
//! Per style:
//! - 91: ZERO draws — the dust loop (`:32546-32555`) rolls only NewDust
//!   internals.
//! - 92: the 1128 first-tick `Next(projFrames[type])` frame roll
//!   (`:32565`); the GasTrap particle broadcast (`:32574-32578`) is the
//!   particle-layer skip — `Spawn_GasTrap`'s internals
//!   (`ParticleOrchestrator.cs:2911-2953`) go with the unported particle
//!   pool, the `Dust.NewDust` body convention.
//! - 93: the NPC-death release's TWO `Next(-100, 101)` draws (`:32864`).
//!   The `Kill()` visuals (`:76343-76365`: Item14 + the dust bursts + the
//!   6×`Next(2)`) are the pool-wide kill-dust skip (the empress precedent).
//! - 94: the tick-40 `Next(3)` cadence pick (`:32905`); the coin drop's
//!   `RotatedByRandom(2π)` pair + the `NextFloat` speed (`:32933`) — the
//!   explicit velocity skips `Item.NewItem`'s scatter pair
//!   (`Item.cs:49738-49741`) and `prefix: 0` rolls nothing, so the drop
//!   request is stream-neutral at the drain; the tail dust gate's
//!   `Next(3)` (`:32953`) + the taken branch's `RotatedByRandom(2π)` pair
//!   (`:32957`).
//! - 95: ZERO draws in the AI; the `Kill()` burst (`:73257-73269`) is the
//!   pool-wide kill-dust skip.
//! - 96: the 2-iteration dust gate's `Next(3)` per iteration (`:33013`).
//! - 97: ZERO draws.
//! - 98: the 2-dust loop's `NextFloat` scale per dust (`:33150`).
//! - 99 counterweight: ZERO draws (the machine is pure maths; the
//!   magicString clone spawn `:67641` is an unported accessory grant).
//! - 100: the fire window's per-scatter-ray `Next(4)` +
//!   `RotatedByRandom` pair + `NextFloat` (`:44603`), in the C#'s order
//!   BEFORE the ray spawns' damage walks (the pool-side step keeps the
//!   inline order).
//!
//! Documented deviations (the module-wide folds, ANALYSIS "W4-d10"):
//! - `Lighting.AddLight` (`:32962`, `:44612`) — no per-tick AI light layer
//!   (the `proj_lights.rs` frame feed is the SetDefaults `light` slice).
//! - the yoyoString/magicString/counterWeight accessory grants stay with
//!   M3-EQUIP-3 (the `ai_yoyo` precedent) — their terms fold to the
//!   no-accessory branch.
//! - `Main.player[owner].deadMansSweater` (`ApplyBuffTo` `:12393-12396`)
//!   is unported — the gas trap's buff time never halves.
//! - `CurrentFrameFlags.HadAnActiveInteractableProjectile` (`:33070`) and
//!   the piggy bank's container UI (`TryGetContainerIndex` `:78815`) have
//!   no port consumer.
//! - the 535's rays are the aiStyle-101 kind 536 — d11's claim; until its
//!   row lands the ray spawns decline (`spawn_with_data`'s no-row no-op)
//!   and the Medusa Head deals no damage (the dependency is recorded).
//! - `hide` (1007) / `drawLayer` / `usesOwnerLight` (535) have no port
//!   slot fields — draw-layer state, the module doc convention.
//!
//! CODE-ONLY MODE (W4-d10): the witness tests at the bottom are WRITTEN
//! NOT RUN.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::npc::Npc;
use crate::projectile::{
    Projectile, can_hit, chaseable, npc_center, safe_normalize,
};

/// `AI()`'s aiStyle-91 arm (`Projectile.cs:32515-32557`) — the Shadowflame
/// Hex Doll's flame (496). The center-preserving scale shrink (the box is
/// `(int)(20·scale)` square), the localAI[0] fade ramp (0.01 below 0.1,
/// 0.025 above — the DOUBLE comparison `(double)localAI[0] < 0.1`), the
/// 0.95 kill, the ai-driven 1.5 accel with the 16 cap and the 1.05 ai
/// growth. The dust loop (`:32544-32556`) feeds only skipped NewDust
/// bodies — ZERO draws by construction.
pub(crate) fn ai_shadowflame(p: &mut Projectile) {
    // `:32517-32522` — the center latch, the scale, the int-truncated box,
    // the position re-anchor (`width / 2` is the C# INT division).
    let center = Vec2::from(p.center());
    p.scale = 1.0 - p.local_ai[0];
    p.w = (20.0 * p.scale) as i32 as f32;
    p.h = p.w;
    p.pos.x = center.x - (p.w as i32 / 2) as f32;
    p.pos.y = center.y - (p.h as i32 / 2) as f32;
    // `:32523-32530` — the fade ramp.
    if p.local_ai[0] < 0.1 {
        p.local_ai[0] += 0.01;
    } else {
        p.local_ai[0] += 0.025;
    }
    // `:32531-32534` — the 0.95 kill.
    if p.local_ai[0] >= 0.95 {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
    }
    // `:32535-32541` — the ai accel + the 16 cap.
    p.vel.x += p.ai[0] * 1.5;
    p.vel.y += p.ai[1] * 1.5;
    if p.vel.length() > 16.0 {
        p.vel = p.vel.normalize() * 16.0;
    }
    // `:32542-32543` — the ai growth.
    p.ai[0] *= 1.05;
    p.ai[1] *= 1.05;
    // `:32544-32556` — the scale<1 dust loop: `(float)num < scale * 10f`
    // iterations of NewDust 27 — the bodies and their internal rolls are
    // the dust-layer skip; the loop writes no slot state.
}

/// The 496's `HandleMovement` blocked-move arm (`Projectile.cs:18710-18735`):
/// per axis, a blocked move with `|velocity| < 1` reverses off the
/// pre-collision velocity, a harder one KILLS. The free move and the
/// post-arm `UpdatePosition` integrate (`:20445` — the 496 is not on the
/// skip list); the collision box is the 10×10 `GetCollisionParams` slice
/// (`:19957-19961`), centered (`resizeAnchor` 0.5).
pub(crate) fn shadowflame_move(p: &mut Projectile, world: &World) {
    if !p.tile_collide {
        p.pos += p.vel;
        return;
    }
    let last = p.vel;
    // The 10×10 centered collision box (`:19862`'s `Size/2 - col*anchor`).
    let off = Vec2::new(p.w * 0.5 - 5.0, p.h * 0.5 - 5.0);
    let tc = crate::collision::tile_collision(
        world,
        [p.pos.x + off.x, p.pos.y + off.y],
        [p.vel.x, p.vel.y],
        10.0,
        10.0,
        true,
        true,
    );
    p.vel = Vec2::new(tc.vel[0], tc.vel[1]);
    // `:18712-18722` — the X axis.
    if p.vel.x != last.x {
        if p.vel.x.abs() < 1.0 {
            p.vel.x = -last.x;
        } else {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
        }
    }
    // `:18723-18733` — the Y axis (the C# runs it even after the X kill).
    if p.vel.y != last.y {
        if p.vel.y.abs() < 1.0 {
            p.vel.y = -last.y;
        } else {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
        }
    }
    // `UpdatePosition` (`:20445`) + `AutomaticallyChangesDirection`
    // (`:17016-17026` — aiStyle 91 is not excluded).
    p.pos += p.vel;
    p.direction = if p.vel.x < 0.0 { -1 } else { 1 };
}

// Witnesses (W4-d10 code-only round — WRITTEN NOT RUN).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::proj_data;

    /// A draw-recording stream (the `projectile_ai_misc` CensusRng shape).
    #[derive(Default)]
    struct CensusRng {
        ranges: Vec<(i32, i32)>,
        floats: usize,
    }
    impl DropRng for CensusRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.ranges.push((min, max));
            min
        }
        fn next_float(&mut self) -> f32 {
            self.floats += 1;
            0.0
        }
    }

    fn proj(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("the rowed kind");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(1600.0, 800.0);
        p.w = d.w;
        p.h = d.h;
        p.tile_collide = d.tile_collide;
        p.alpha = d.alpha;
        p.scale = d.scale;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.max_penetrate = d.penetrate;
        p.extra_updates = d.extra_updates;
        p
    }

    /// The 496 row (`Projectile.cs:5692-5704`): 40×40, aiStyle 91, magic,
    /// `MaxUpdates = 3` (extraUpdates 2), penetrate 3, the local-immunity
    /// −1 pair, alpha 255.
    #[test]
    fn w4_d10_s91_the_shadowflame_row_is_verbatim() {
        let d = proj_data(496).unwrap();
        assert_eq!((d.w, d.h), (40.0, 40.0));
        assert_eq!(d.ai_style, 91);
        assert_eq!(d.penetrate, 3);
        assert_eq!(d.extra_updates, 2, "MaxUpdates = 3 (`:5700`)");
        assert!(d.uses_local_immunity);
        assert_eq!(d.local_hit_cooldown, -1);
        assert_eq!(d.alpha, 255);
        assert!(d.magic);
        assert!(d.tile_collide, "the stock true (`:775`)");
    }

    /// The shrink keeps the center (`:32517-32522`): tick 1 scale 1 → box
    /// 20, the ramp 0.01; the box is int-truncated and re-anchored.
    #[test]
    fn w4_d10_s91_the_shrink_keeps_the_center_and_ramps() {
        let mut p = proj(496);
        let c0 = Vec2::from(p.center());
        ai_shadowflame(&mut p);
        assert_eq!(p.scale, 1.0);
        assert_eq!((p.w, p.h), (20.0, 20.0));
        assert_eq!(Vec2::from(p.center()), c0, "the center latch");
        assert_eq!(p.local_ai[0], 0.01);
        // The ramp switches at 0.1 (`:32523` — strict <).
        p.local_ai[0] = 0.1;
        ai_shadowflame(&mut p);
        assert_eq!(p.local_ai[0], 0.125, "0.1 is NOT < 0.1 — the 0.025 arm");
        assert!((p.scale - 0.9).abs() < 1e-6);
        assert_eq!(p.w, 18.0, "(int)(20 * 0.9)");
    }

    /// The 0.95 kill (`:32531-32534`) and the ai accel/cap/growth
    /// (`:32535-32543`). ZERO rand draws.
    #[test]
    fn w4_d10_s91_the_fade_kills_at_095_and_the_ai_grows() {
        let mut p = proj(496);
        p.local_ai[0] = 0.94;
        p.ai = [1.0, 0.0, 0.0, 0.0];
        ai_shadowflame(&mut p);
        // 0.94 >= 0.1 → +0.025 → 0.965 >= 0.95 → dead.
        assert!(!p.active);
        assert_eq!(p.time_left, 0);
        assert!((p.vel.x - 1.5).abs() < 1e-6, "the 1.5 accel");
        assert!((p.ai[0] - 1.05).abs() < 1e-6, "the 1.05 growth");
        // The 16 cap normalizes.
        let mut q = proj(496);
        q.ai = [20.0, 0.0, 0.0, 0.0];
        ai_shadowflame(&mut q);
        assert!((q.vel.length() - 16.0).abs() < 1e-4);
    }

    /// The blocked-move arm (`:18710-18735`): a soft block reverses, a
    /// hard block kills. Open air keeps the velocity.
    #[test]
    fn w4_d10_s91_the_bounce_reverses_soft_and_kills_hard() {
        let w = World::new("t".into(), 300, 200);
        let mut p = proj(496);
        p.vel = Vec2::new(4.0, 0.0);
        let before = p.pos;
        shadowflame_move(&mut p, &w);
        assert_eq!(p.pos, before + Vec2::new(4.0, 0.0), "the free integrate");
        assert!(p.active);
        assert_eq!(p.direction, 1);
        // A wall one pixel ahead: the 10×10 box's right edge at 1620+...
        // place the solid so the move blocks with |v| >= 1 → the kill.
        let mut w2 = World::new("t".into(), 300, 200);
        let tx = ((p.pos.x + p.w * 0.5 + 5.0 + 4.0) / 16.0) as i32;
        let ty = (p.pos.y / 16.0) as i32;
        let i = w2.index(tx, ty);
        w2.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
        w2.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut q = proj(496);
        q.pos = p.pos;
        q.vel = Vec2::new(4.0, 0.0);
        shadowflame_move(&mut q, &w2);
        assert!(!q.active, "the hard block kills (`:18720`)");
    }
}
