//! The generic `Projectile.Update` wet machine — `Projectile.cs:16786-17007`
//! (W4-d53 batch 3, the batch-1 census's one real host-gap: the 8 entry/exit
//! splash sites `:16851-16987`). Ported VERBATIM: the `wetCount` cooldown,
//! the lava/honey/shimmer flag latching, the `IsDestroyedByWater` kill, the
//! FireArrow 2→1 transform, the type-34 lava dry-out, the type-80 sandgun
//! arm, the entry/exit edge machine with the 155 carve-out, the dry-clear
//! tail and the `wetCount--`. The g15 WorldItem precedent
//! (`ItemDrop::wet_edges` / `SplashKind`) is the shape mirror; the projectile
//! constants differ (10 dust per arm, the ENTRY arm sets NO cooldown —
//! `:16843-16915` — only the EXIT arm sets `wetCount = 10`, `:16926`).
//!
//! The splash DUST rides the queue→drain contract: the edge parks
//! `wants_wet_splash = Some((entry, kind))` on the slot (the `wants_sound`
//! pattern), the dispatch forwards it into `pending_dust_bursts` and
//! `projectile_dust::drain` replays the burst on the `UpdateProjectiles`
//! stream. The case-19 SOUND rides the pool's existing `pending_splashes`
//! drain: shimmer entry style 4 (`:16871`) / exit style 5 (`:16953`), every
//! other arm the int overload's DEFAULT style 1 (`SoundEngine.cs:102`).
//! The shimmer loops play TEN times in C# but the case-19 playing guard
//! (`LegacySoundPlayer.cs:872-874`) refuses plays 2-10 before any draw (the
//! first instance is live within the tick), so the port queues ONE play —
//! stream-identical, the 9 guard-refused siblings cited.
//!
//! Probes: `Collision.LavaCollision` (`Collision.cs:1653-1684`, the FULL
//! box) → `crate::player::lava_collision`; `Collision.WetCollision`
//! (`Collision.cs:1572-1651`, the inset probe + the `Collision.honey`/
//! `shimmer` out-params + the sloped-tile arm, FIRST overlapping cell wins)
//! → `crate::npc_ai_slime::wet_collision`. Both are the existing verbatim
//! ports (the house keep-one rule). The C# `Utils.Clamp(v, 0, maxTilesY-40)`
//! with a NEGATIVE max folds the y-scan empty on worlds ≤40 tiles tall
//! (min>max: every value clamps past the loop bound) — the port guards the
//! slime probe (whose `.clamp(0, h-40)` would panic) with the same dry fold.
//!
//! Folds / cited skips (each at its site):
//! * The `try/catch` (`:16794-16815`) — the port's probes are infallible
//!   (the tile grid is bounds-checked); the catch's `active = false` arm is
//!   unreachable, dropped.
//! * `Shimmer()` (`:17009-17012`) — SUPERSEDED (W4-d53 FINAL): g2's STAGED
//!   helper (`63a6db269`, `Projectile::shimmer`) is now ACTIVATED — the
//!   dispatch runs the `:17011` caller-gate right after this machine (the
//!   authoritative `shimmer_wet` writer), the g2d-active probe found no
//!   pushed activation branch so the wiring lands here. The item-side
//!   shimmer transforms stay e19/f47's lane.
//! * `HandleMovement(wetVelocity)`'s wet velocity scaling (`:17612-17655`:
//!   ×0.5 water / ×0.25 honey / ×0.375 shimmer) — the movement lane, outside
//!   the window; the port's arms integrate movement inside the AI dispatch
//!   (the `kill_step`-in-arm convention), so the wet block runs AFTER the
//!   tick's move — a one-tick position lead for the splash box, the recorded
//!   queue→drain deviation family (ANALYSIS). W4-d53 FINAL census: the
//!   item-side wetVelocity blend landed (e19 `a849b310d`/`939e45dc8`, f47
//!   `8d11597b4` — WorldItem's ApplyMovement); the PROJECTILE side is
//!   PORTED for the flail (`flail_wet_factor`, S1-P-i07) and the d3 boulder
//!   family (`boulder_movement`'s wet arm, `2d01693ca`), but the GENERIC
//!   integrate (`kill_step` + `ai_style1_flight` + the per-arm free moves)
//!   still steps the FULL velocity while wet — a separate movement-lane
//!   arm, censused in ANALYSIS "The wetVelocity census" (not this lane's
//!   window; the wet machine's flags are its input, now authoritative).
//! * The two pre-AI wet substitutes stay as documented read-lag feeds: the
//!   bobber's head `probe_wet` (the i35 one-tick-lag deviation) and the
//!   aiStyle-16 dispatch's centre-cell `wet` write (the i16 stand-in). This
//!   machine is the authoritative writer at the C# position (post-AI); the
//!   substitutes feed only their arms' pre-AI reads, exactly the lagged
//!   state the C# AI reads (the previous tick's wet block).

use crate::projectile::Projectile;
use tr_core::world::World;

/// The liquid kind a splash edge resolved to (`Projectile.cs:16845-16913`
/// entry / `:16927-16995` exit): the entry reads the FRESH lava probe, the
/// exit the OLD `lavaWet` flag; then shimmer, then honey, else water. The
/// g15 `item::SplashKind` mirror (keep-one at merge: the two enums are
/// shape-identical, the hosts differ).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SplashKind {
    Water,
    Lava,
    Honey,
    Shimmer,
}

/// `ProjectileID.Sets.IsDestroyedByWater` (`ProjectileID.cs:429`:
/// `CreateBoolSet(85, 15, 188, 34, 1097, 1106)`).
fn is_destroyed_by_water(kind: u16) -> bool {
    matches!(kind, 85 | 15 | 188 | 34 | 1097 | 1106)
}

/// One tick of the generic wet machine (`Projectile.cs:16786-17007`), run
/// by the dispatch AFTER the AI sub-steps (the C# position: post-`AI()`,
/// pre-`HandleMovement`). Returns `true` when a `Kill()` fired inside the
/// block (the `IsDestroyedByWater` arm `:16820` or the type-80 lava kill
/// `:16838`) — the caller applies the pool's kill convention (the
/// `kill_time_left` latch + the end-of-tick death hook). The C# `Kill()`
/// does NOT return from `Update`: the rest of the block still runs on the
/// dead slot, so the machine keeps updating the flags after a kill.
pub(crate) fn wet_block(p: &mut Projectile, world: &World) -> bool {
    // `:16786-16789` — the wet-clear preamble (the Empress bullet 242, the
    // FairyQueenRangedItemShot 302, the MoonlordArrow 638 never hold wet).
    if matches!(p.kind, 242 | 302 | 638) {
        p.wet = false;
    }
    // `:16790` — the ignoreWater gate skips the WHOLE block.
    if p.ignore_water {
        return false;
    }
    // `:16796-16797` — LavaCollision (full box) THEN WetCollision (inset
    // probe, sets the honey/shimmer out-params). The ≤40-tile dry fold
    // (module doc) reproduces Utils.Clamp's min>max empty scan.
    let dry_world = world.height <= 40;
    let lava = !dry_world
        && crate::player::lava_collision(world, p.pos.x, p.pos.y, p.w, p.h);
    let (mut wet_now, honey, shimmer) = if dry_world {
        (false, false, false)
    } else {
        crate::npc_ai_slime::wet_collision(world, p.pos.x, p.pos.y, p.w as i32, p.h as i32)
    };
    // `:16798-16809` — the flags LATCH (sticky until the dry-clear tail).
    if lava {
        p.lava_wet = true;
    }
    if honey {
        p.honey_wet = true;
    }
    if shimmer {
        p.shimmer_wet = true;
    }
    let mut killed = false;
    // `:16816-16827` — the OLD wet flag + !lavaWet: the water-destroyed
    // kinds die; the FireArrow 2 transforms into the WoodenArrow 1
    // (`TransformType` `:692-695` writes ONLY the type — no SetDefaults
    // re-read) and zeroes its light.
    if p.wet && !p.lava_wet {
        if is_destroyed_by_water(p.kind) {
            killed = true;
        }
        if p.kind == 2 {
            p.kind = 1;
            p.light = 0.0;
        }
    }
    // `:16828-16831` — the Vilethorn base 34 in lava dries out completely
    // (`flag2 = (wet = (lavaWet = false))`).
    if p.kind == 34 && p.lava_wet {
        wet_now = false;
        p.wet = false;
        p.lava_wet = false;
    }
    // `:16832-16840` — the Sandgun ball 80 never holds wet; a fresh lava
    // probe with `ai[0] >= 0` kills it.
    if p.kind == 80 {
        wet_now = false;
        p.wet = false;
        if lava && p.ai[0] >= 0.0 {
            killed = true;
        }
    }
    // `:16841-16916` — the ENTRY edge: wetCount == 0 + !wet + not the
    // Beetle 155. NO cooldown is set here (the C# entry arm has no
    // wetCount write — the exit-only asymmetry, verbatim).
    if wet_now {
        if p.kind != 155 && p.wet_count == 0 && !p.wet {
            let kind = if !lava {
                if p.shimmer_wet {
                    SplashKind::Shimmer
                } else if p.honey_wet {
                    SplashKind::Honey
                } else {
                    SplashKind::Water
                }
            } else {
                SplashKind::Lava
            };
            p.wants_wet_splash = Some((true, kind));
        }
        p.wet = true;
    } else if p.wet {
        // `:16917-16997` — the EXIT edge: the Beetle 155 halves its fall
        // instead of splashing (`:16920-16923`); every other kind arms the
        // 10-tick cooldown (`:16926`) and resolves the kind from the OLD
        // flags (the dry-clear below has not run yet).
        p.wet = false;
        if p.kind == 155 {
            p.vel.y *= 0.5;
        } else if p.wet_count == 0 {
            p.wet_count = 10;
            let kind = if !p.lava_wet {
                if p.shimmer_wet {
                    SplashKind::Shimmer
                } else if p.honey_wet {
                    SplashKind::Honey
                } else {
                    SplashKind::Water
                }
            } else {
                SplashKind::Lava
            };
            p.wants_wet_splash = Some((false, kind));
        }
    }
    // `:16998-17003` — dry clears the three kind flags (NOT wet itself).
    if !p.wet {
        p.lava_wet = false;
        p.honey_wet = false;
        p.shimmer_wet = false;
    }
    // `:17004-17007` — the cooldown tail.
    if p.wet_count > 0 {
        p.wet_count -= 1;
    }
    killed
}

// --- W4-d53 batch-4 machine witnesses (WRITTEN NOT RUN — code-only) --------
#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;
    use tr_core::tile::{LiquidKind, Tile};

    /// A 200×200 world (above the ≤40 dry fold) with a 10×4 liquid pond at
    /// tiles (20..30, 100..104); `kind` picks the liquid.
    fn pond(kind: LiquidKind) -> World {
        let mut w = World::new("t".into(), 200, 200);
        for x in 20..30 {
            for y in 100..104 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 0, ..Default::default() };
                w.tiles[i].liquid = 255;
                w.tiles[i].set_liquid_kind(kind);
            }
        }
        w
    }

    /// A 10×10 shot parked inside the pond (centre probe overlaps).
    fn shot(kind: u16) -> Projectile {
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.w = 10.0;
        p.h = 10.0;
        p.pos = Vec2::new(25.0 * 16.0, 101.0 * 16.0);
        p
    }

    /// `:16843-16915` — the water ENTRY edge queues the splash and sets
    /// wet, but arms NO cooldown (the entry-only asymmetry). A second wet
    /// tick queues nothing (the `!wet` gate).
    #[test]
    fn w4_d53_entry_edge_sets_wet_without_a_cooldown() {
        let w = pond(LiquidKind::Water);
        let mut p = shot(3);
        assert!(!wet_block(&mut p, &w));
        assert!(p.wet && !p.lava_wet && !p.honey_wet && !p.shimmer_wet);
        assert_eq!(p.wet_count, 0, "the entry arm sets NO cooldown");
        assert_eq!(p.wants_wet_splash.take(), Some((true, SplashKind::Water)));
        assert!(!wet_block(&mut p, &w));
        assert!(p.wants_wet_splash.is_none(), "no re-entry churn");
    }

    /// `:16917-16997` — the EXIT edge arms wetCount = 10 and queues the
    /// exit splash; the cooldown then counts down once per tick and blocks
    /// a re-entry splash until it runs out (`:16843`'s wetCount gate).
    #[test]
    fn w4_d53_exit_edge_arms_the_ten_tick_cooldown() {
        let w = pond(LiquidKind::Water);
        let dry = World::new("d".into(), 200, 200);
        let mut p = shot(3);
        wet_block(&mut p, &w);
        p.wants_wet_splash.take();
        // Leave the liquid: the exit edge fires.
        assert!(!wet_block(&mut p, &dry));
        assert!(!p.wet);
        assert_eq!(p.wants_wet_splash.take(), Some((false, SplashKind::Water)));
        assert_eq!(p.wet_count, 9, "10 set at :16926, the tail decrements");
        // Re-enter while the cooldown runs: wet latches, NO splash.
        for expected in (0..9).rev() {
            wet_block(&mut p, &w);
            assert!(p.wants_wet_splash.is_none(), "the cooldown suppresses");
            assert_eq!(p.wet_count, expected);
        }
        // Cooldown spent: the next EXIT re-arms.
        wet_block(&mut p, &dry);
        assert_eq!(p.wants_wet_splash.take(), Some((false, SplashKind::Water)));
    }

    /// The kind resolution: lava first (the entry reads the FRESH probe,
    /// the exit the OLD flag), then shimmer, then honey, else water
    /// (`:16845-16913` / `:16927-16995`).
    #[test]
    fn w4_d53_the_liquid_kind_transitions_resolve() {
        for (liquid, expected) in [
            (LiquidKind::Lava, SplashKind::Lava),
            (LiquidKind::Shimmer, SplashKind::Shimmer),
            (LiquidKind::Honey, SplashKind::Honey),
            (LiquidKind::Water, SplashKind::Water),
        ] {
            let w = pond(liquid);
            let mut p = shot(3);
            wet_block(&mut p, &w);
            assert_eq!(p.wants_wet_splash.take(), Some((true, expected)), "{liquid:?} entry");
            let dry = World::new("d".into(), 200, 200);
            wet_block(&mut p, &dry);
            assert_eq!(p.wants_wet_splash.take(), Some((false, expected)), "{liquid:?} exit");
        }
    }

    /// `:16816-16827` — the IsDestroyedByWater kill (the 15 Starfury shot)
    /// and the FireArrow 2→1 transform + light-0 (type-only, no row re-read).
    #[test]
    fn w4_d53_water_kill_and_fire_arrow_transform() {
        let w = pond(LiquidKind::Water);
        let mut p = shot(15);
        p.wet = true; // the OLD flag gates the arm
        assert!(wet_block(&mut p, &w), "the 15 dies in water");
        let mut f = shot(2);
        f.wet = true;
        f.light = 0.8;
        assert!(!wet_block(&mut f, &w));
        assert_eq!(f.kind, 1, "TransformType(1)");
        assert_eq!(f.light, 0.0);
        // In LAVA the arm refuses (lavaWet latched): the 2 keeps its type.
        let l = pond(LiquidKind::Lava);
        let mut g = shot(2);
        g.wet = true;
        assert!(!wet_block(&mut g, &l));
        assert_eq!(g.kind, 2);
    }

    /// `:16828-16840` — the 34 lava dry-out and the 80 sandgun arm (never
    /// wet; the fresh lava probe + ai[0] >= 0 kills).
    #[test]
    fn w4_d53_the_34_dryout_and_80_lava_kill() {
        let l = pond(LiquidKind::Lava);
        let mut v = shot(34);
        v.wet = true;
        assert!(!wet_block(&mut v, &l));
        assert!(!v.wet && !v.lava_wet, "the 34 dries out in lava");
        assert!(v.wants_wet_splash.is_none());
        let mut s = shot(80);
        assert!(wet_block(&mut s, &l), "the 80 dies in lava (ai0 = 0)");
        assert!(!s.wet);
        let mut t = shot(80);
        t.ai[0] = -1.0;
        assert!(!wet_block(&mut t, &l), "ai0 < 0 refuses the kill");
    }

    /// `:16920-16923` — the Beetle 155 exit halves velocity.Y, queues no
    /// splash and arms no cooldown; its entry is suppressed too (`:16843`).
    #[test]
    fn w4_d53_the_155_carve_out() {
        let w = pond(LiquidKind::Water);
        let dry = World::new("d".into(), 200, 200);
        let mut p = shot(155);
        p.vel.y = 8.0;
        wet_block(&mut p, &w);
        assert!(p.wants_wet_splash.is_none(), "the 155 entry is suppressed");
        wet_block(&mut p, &dry);
        assert_eq!(p.vel.y, 4.0, "the exit halves the fall");
        assert_eq!(p.wet_count, 0);
        assert!(p.wants_wet_splash.is_none());
    }

    /// `:16786-16789` + `:16790` — the 242/302/638 preamble clears wet; the
    /// ignoreWater gate skips the whole machine (no probe, no flags).
    #[test]
    fn w4_d53_the_preamble_and_ignore_water_gate() {
        let w = pond(LiquidKind::Water);
        for kind in [242u16, 302, 638] {
            let mut p = shot(kind);
            p.wet = true;
            wet_block(&mut p, &w);
            // The preamble cleared wet, then the entry edge re-latched it.
            assert!(p.wet);
            assert_eq!(p.wants_wet_splash.take(), Some((true, SplashKind::Water)));
        }
        let mut p = shot(3);
        p.ignore_water = true;
        p.wet = true;
        wet_block(&mut p, &w);
        assert!(p.wet, "the gate keeps the old state");
        assert!(p.wants_wet_splash.is_none());
    }

    /// The ≤40-tile dry fold (module doc): an 8×8 front-flow world probes
    /// dry without panicking (Utils.Clamp's min>max empty scan).
    #[test]
    fn w4_d53_small_worlds_fold_dry() {
        let mut w = World::new("s".into(), 8, 8);
        let i = w.index(2, 2);
        w.tiles[i].liquid = 255;
        let mut p = shot(3);
        p.pos = Vec2::new(32.0, 32.0);
        assert!(!wet_block(&mut p, &w));
        assert!(!p.wet && !p.lava_wet);
        assert!(p.wants_wet_splash.is_none());
    }

    /// `:16998-17003` — the dry-clear tail drops the three kind flags once
    /// the slot is dry (the exit tick itself keeps them for the kind
    /// resolution, the NEXT dry tick clears).
    #[test]
    fn w4_d53_the_dry_clear_tail() {
        let w = pond(LiquidKind::Honey);
        let dry = World::new("d".into(), 200, 200);
        let mut p = shot(3);
        wet_block(&mut p, &w);
        assert!(p.honey_wet);
        wet_block(&mut p, &dry); // the exit tick: the kind read the flag
        assert_eq!(p.wants_wet_splash.take(), Some((false, SplashKind::Honey)));
        assert!(!p.honey_wet, "the same tick's tail clears (wet now false)");
        assert!(!p.lava_wet && !p.shimmer_wet);
    }
}
