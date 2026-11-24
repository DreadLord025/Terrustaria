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
//!   family (`boulder_movement`'s wet arm, `2d01693ca`).
//!   W4-d53 RETIRE refine (ANALYSIS "The wetVelocity R-seed refine"): the
//!   factor block is gated INSIDE `if (tileCollide)` (`:17476`) → the `:17590`
//!   gate → `else if (base.wet)` (`:17612`), so the FREE move (`tileCollide =
//!   false`) NEVER drags — `UpdatePosition` (`:20439-20446`) integrates the
//!   un-scaled `:16765` capture while wet. The port's `kill_step` free-move
//!   branch (`pos += vel`) is therefore ALREADY faithful for the ~42 free-move
//!   sites (the capture-lag deviation aside); the REAL gap is the tile-collision
//!   path's blocked/bounce integrate, which the port's `kill_step` blocked
//!   branch approximates as kill-on-contact. [`wet_factor`] below is the ONE
//!   shared factor pick that path adopts (keep-one with the d3 boulder inline).
//!   W4-d53 S1-P-i57 (R-244-1) ACTIVATED: [`wet_velocity`] is the integrate-swap
//!   helper — the `:17612-17655` factor + blocked-axis restore on the
//!   full-velocity `TileCollision` result, gated by [`update_position_skips`]
//!   (the `:20435` skip list) + the `:17476` free-move gate + the `:17590` latch
//!   / aiStyle-10 exemptions. It is PURE (returns the displacement, no slot
//!   writes, no draws — RNG-neutral physics, not stream). The 5 swap points
//!   (kill_step 54 callers / ai_style1_flight / special_arrows bounce / the
//!   thrown twin 3 callers / the 12 dispatch mirrors + the hook + charged-blaster
//!   mirrors) + the satellite blocked-test arms swap their integrate for it;
//!   the free moves + AI-internal steps (the wisp snap, the hoik loops, the
//!   vile-spit `:23732`, the spear offset) keep the full velocity (not swap
//!   sites). The flail keeps its landed `flail_wet_factor` probe (KEEP-ONE,
//!   S1-P-i07). 8 witnesses WRITTEN NOT RUN (the A/B same-seed shape).
//! * The two pre-AI wet substitutes stay as documented read-lag feeds: the
//!   bobber's head `probe_wet` (the i35 one-tick-lag deviation) and the
//!   aiStyle-16 dispatch's centre-cell `wet` write (the i16 stand-in). This
//!   machine is the authoritative writer at the C# position (post-AI); the
//!   substitutes feed only their arms' pre-AI reads, exactly the lagged
//!   state the C# AI reads (the previous tick's wet block).

use crate::projectile::Projectile;
use tr_core::world::World;

/// The liquid drag factor the generic `HandleMovement` wet arm applies to the
/// velocity before `UpdatePosition` integrates it (`Projectile.cs:17612-17655`):
/// shimmer ×0.375 (`:17618`), honey ×0.25 (`:17632`), else (water/lava) ×0.5
/// (`:17646`). `None` when dry — the C# `else if (base.wet)` gate (`:17612`)
/// never runs the factor on a dry slot. The factor pick reads the slot's wet
/// flags (the batch-4 machine's authoritative writes), NOT a fresh world probe
/// — the C# reads `base.shimmerWet`/`base.honeyWet`/`base.wet` the same way.
///
/// W4-d53 S1-P-i57: this is the ONE shared factor pick for the generic
/// integrate (the d3 `boulder_movement` wet arm `2d01693ca` inlines the same
/// three constants; keep-one at the movement-lane merge — the boulder arm
/// adopts this pick when d3 lands). The flail's `flail_wet_factor`
/// (projectile.rs) is the mid-tile-probe variant — KEEP-ONE: the flail's
/// ported arm (S1-P-i07, `:6114`) stays as landed; its probe idiom is the
/// recorded pre-AI read-lag stand-in, not a second factor table.
pub(crate) fn wet_factor(p: &Projectile) -> Option<f32> {
    if !p.wet {
        return None;
    }
    Some(if p.shimmer_wet {
        0.375
    } else if p.honey_wet {
        0.25
    } else {
        0.5
    })
}

/// `HandleMovement`'s wet drag (`Projectile.cs:17612-17655`) given the
/// FULL-velocity `TileCollision` result: scale the post-collision velocity by
/// the liquid factor, then restore each CHANGED axis to its full
/// post-collision value (the blocked-axis restore — `:17619-17626` shimmer /
/// `:17633-17640` honey / `:17647-17654` water, the three byte-identical
/// arms). `before` is the C# `vector2/3/4` — the PRE-collision velocity;
/// `post` the resolved `base.velocity`. RNG-neutral: no draws.
fn wet_drag(post: glam::Vec2, before: glam::Vec2, f: f32) -> glam::Vec2 {
    let mut wet = post * f;
    if post.x != before.x {
        wet.x = post.x;
    }
    if post.y != before.y {
        wet.y = post.y;
    }
    wet
}

/// THE `wet_velocity` helper (S1-P-i57, R-244-1): the integrate-swap vector
/// for the generic movement funnels — the C# `wetVelocity` local that
/// `UpdatePosition` (`:20439-20446`) integrates while wet. PURE (no slot
/// writes, no draws): the per-arm collision stand-ins own the velocity
/// rewrite, this returns only the displacement.
/// * dry (`wet_factor` `None`) OR skip-listed (`:20435`, see
///   [`update_position_skips`]) → `p.vel` untouched — byte-identical to the
///   pre-port integrate, no probe. The golf/aiStyle-4/latch kinds keep their
///   engines and full-velocity movement while wet (the gate exempts them).
/// * `tileCollide = false` → `p.vel` — the `:17476` gate closes the whole
///   tile pass, so the `:17612` drag never runs; the free move integrates the
///   un-scaled `:16765` capture (the port's post-AI `p.vel` stands in for the
///   capture — the recorded one-tick lead, the Lead's RETIRE refine).
/// * `tileCollide = true` wet → the `:17612-17655` wet arm's VALUE (the d3
///   `boulder_movement` reference): FULL-velocity `TileCollision` then
///   [`wet_drag`] scale + blocked-axis restore.
///
/// Folds (recorded, ANALYSIS): the `flag6` fall-through pair folds `true`
/// (the boulder precedent — the per-kind `:17479-17582` switch is the dry
/// probe stand-ins' own deviation); the `:17590` 440/449/606-ai[1]==1 /
/// 466/580/640-localAI[1] latch gate and the `:17656-17698` wet slope pass
/// stay with the dry arms' existing folds; `GetCollisionParams`' per-kind
/// box overrides fold to the slot box (the boulder precedent). The C# wet
/// arm's `base.velocity` rewrite is the stand-ins' concern, not this
/// integrate vector's (the d3 full port owns its own). The aiStyle-10
/// falling-block arm (`:17592`) runs its OWN `TileCollision` chain and never
/// writes `wetVelocity` — exempt (the un-scaled capture integrates). The
/// `:17590` latch gate (the 440/449/606 `ai[1] == 1` / 466 `localAI[1] == 1`
/// / 580/640 `localAI[1] > 0` states) wraps the whole factor block — those
/// latched states skip the drag too.
pub(crate) fn wet_velocity(p: &Projectile, world: &World) -> glam::Vec2 {
    let Some(f) = wet_factor(p) else {
        return p.vel;
    };
    if !p.tile_collide || update_position_skips(p) {
        return p.vel;
    }
    // `:17590-17592` — the latch gate + the aiStyle-10 arm precede the
    // `else if (base.wet)` factor block; neither writes `wetVelocity`.
    let style = crate::projectile::proj_data(p.kind).map_or(255, |d| d.ai_style);
    if style == 10
        || (matches!(p.kind, 440 | 449 | 606) && p.ai[1] == 1.0)
        || (p.kind == 466 && p.local_ai[1] == 1.0)
        || (matches!(p.kind, 580 | 640) && p.local_ai[1] > 0.0)
    {
        return p.vel;
    }
    let before = p.vel;
    let tc = crate::collision::tile_collision(
        world,
        [p.pos.x, p.pos.y],
        [p.vel.x, p.vel.y],
        p.w,
        p.h,
        true,
        true,
    );
    let post = glam::Vec2::new(tc.vel[0], tc.vel[1]);
    wet_drag(post, before, f)
}

/// `UpdatePosition`'s skip list (`Projectile.cs:20435`) VERBATIM: aiStyles
/// 4/38/84/148, the aiStyle-7 `ai[0] == 2` attached-hook latch, the
/// 440/449/606 `ai[1] == 1` latch, aiStyle-93 `ai[0] < 0`, the
/// 540/756/818/856/961/1041/1125/933/1100 kinds and `IsAGolfBall`
/// (`ProjectileID.cs:428`: 721 + 739-752). In the C# a skipped slot returns
/// from `UpdatePosition` WITHOUT integrating — the golf engine
/// (`projectile_ai_golf.rs:228`'s `p.pos = body.position`) and the aiStyle-4
/// vine fold (`projectile.rs:3038`) keep their own position writers. The
/// port's stand-ins integrate these kinds dry today (a pre-existing fold);
/// the gate here EXEMPTS them from the wet drag (they keep full-velocity
/// movement while wet, no factor) without touching that dry behavior. The
/// dead `AI_015_HandleMovementCollision_Old` (`:44171`/`:44201`, no caller —
/// grep finds only the definition) is cited dead, not owed.
pub(crate) fn update_position_skips(p: &Projectile) -> bool {
    let style = crate::projectile::proj_data(p.kind).map_or(255, |d| d.ai_style);
    matches!(style, 4 | 38 | 84 | 148)
        || (style == 7 && p.ai[0] == 2.0)
        || (matches!(p.kind, 440 | 449 | 606) && p.ai[1] == 1.0)
        || (style == 93 && p.ai[0] < 0.0)
        || matches!(
            p.kind,
            540 | 756 | 818 | 856 | 961 | 1041 | 1125 | 933 | 1100
        )
        || matches!(p.kind, 721 | 739..=752)
}

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

    // --- W4-d53 RETIRE: the shared wet_factor pick (WRITTEN NOT RUN) --------

    /// `:17612-17655` — the factor pick reads the slot's wet flags: shimmer
    /// ×0.375, honey ×0.25, water/lava ×0.5, dry `None`. The pick is the ONE
    /// shared shape the generic tile-collision integrate adopts (the d3
    /// boulder arm inlines the same constants; keep-one at the movement merge).
    #[test]
    fn w4_d53_wet_factor_picks_the_liquid_constant() {
        let mut p = shot(3);
        assert_eq!(wet_factor(&p), None, "dry: the :17612 gate never runs");
        p.wet = true;
        assert_eq!(wet_factor(&p), Some(0.5), "water ×0.5 (:17646)");
        p.lava_wet = true;
        assert_eq!(wet_factor(&p), Some(0.5), "lava folds into the water factor");
        p.honey_wet = true;
        assert_eq!(wet_factor(&p), Some(0.25), "honey ×0.25 (:17632)");
        p.shimmer_wet = true;
        assert_eq!(wet_factor(&p), Some(0.375), "shimmer ×0.375 (:17618) wins");
    }

    /// The factor pick agrees with the d3 boulder arm's inline constants
    /// (`boulder_movement` `2d01693ca`: shimmer 0.375 / honey 0.25 / else 0.5)
    /// — the keep-one contract: one shared pick, no per-arm divergence.
    #[test]
    fn w4_d53_wet_factor_matches_the_boulder_inline() {
        for (shimmer, honey, want) in [
            (true, false, 0.375f32),
            (false, true, 0.25),
            (false, false, 0.5),
            (true, true, 0.375), // shimmer precedes honey (:17614 before :17628)
        ] {
            let mut p = shot(3);
            p.wet = true;
            p.shimmer_wet = shimmer;
            p.honey_wet = honey;
            assert_eq!(wet_factor(&p), Some(want), "shimmer={shimmer} honey={honey}");
        }
    }

    // --- S1-P-i57: the wet_velocity helper + the skip-list gate -------------
    // WRITTEN NOT RUN (code-only). The A/B witness shape: same-seed, same
    // slot state, dry vs wet — the drag has NO draws, so the RNG stream is
    // untouched (RNG-neutral physics, not stream).

    /// `:20439-20446` dry arm — a DRY slot integrates the FULL velocity,
    /// byte-identical to the pre-port `pos += vel` (no probe, no writes).
    /// The A/B baseline: the wet arm below must differ ONLY by the factor.
    #[test]
    fn s1_p_i57_dry_integrates_the_full_velocity() {
        let w = pond(LiquidKind::Water);
        let mut p = shot(3);
        p.tile_collide = true;
        p.vel = Vec2::new(8.0, 4.0);
        // Dry: wet_factor None → p.vel untouched.
        assert_eq!(wet_velocity(&p, &w), Vec2::new(8.0, 4.0));
        assert_eq!(p.vel, Vec2::new(8.0, 4.0), "the helper is pure (no writes)");
    }

    /// `:17612-17655` wet arm — a WET tile-colliding slot in open water (no
    /// tile blocks the box) integrates `velocity × 0.5`. Same-seed A/B: the
    /// only difference from the dry witness is the ×0.5 factor, no draws.
    #[test]
    fn s1_p_i57_wet_water_halves_the_integrate() {
        let w = pond(LiquidKind::Water);
        let mut p = shot(3);
        p.tile_collide = true;
        p.wet = true;
        p.vel = Vec2::new(8.0, 4.0);
        // The pond box is open air around the shot (the liquid tiles are
        // non-solid), so TileCollision leaves the velocity unchanged and
        // the drag scales it ×0.5.
        assert_eq!(wet_velocity(&p, &w), Vec2::new(4.0, 2.0));
    }

    /// The three liquid factors ride the same integrate: honey ×0.25,
    /// shimmer ×0.375 (`:17632`/`:17618`). RNG-neutral (no draws).
    #[test]
    fn s1_p_i57_wet_honey_and_shimmer_scale() {
        let mut p = shot(3);
        p.tile_collide = true;
        p.vel = Vec2::new(8.0, 4.0);
        let honey = pond(LiquidKind::Honey);
        p.wet = true;
        p.honey_wet = true;
        assert_eq!(wet_velocity(&p, &honey), Vec2::new(2.0, 1.0), "honey ×0.25");
        let shimmer = pond(LiquidKind::Shimmer);
        p.honey_wet = false;
        p.shimmer_wet = true;
        assert_eq!(wet_velocity(&p, &shimmer), Vec2::new(3.0, 1.5), "shimmer ×0.375");
    }

    /// `:17476` gate — a FREE move (`tileCollide = false`) never drags: the
    /// `:17612` factor block is inside `if (tileCollide)`, so a non-colliding
    /// wet slot integrates the un-scaled capture (the Lead's RETIRE refine).
    #[test]
    fn s1_p_i57_the_free_move_never_drags() {
        let w = pond(LiquidKind::Water);
        let mut p = shot(3);
        p.tile_collide = false;
        p.wet = true;
        p.vel = Vec2::new(8.0, 4.0);
        assert_eq!(wet_velocity(&p, &w), Vec2::new(8.0, 4.0), "free move: full velocity");
    }

    /// `:20435` skip-list gate — the golf balls (`IsAGolfBall`, `ProjectileID.
    /// cs:428`) and the aiStyle-4/38/84/148 kinds keep full-velocity movement
    /// while wet (the gate exempts them from the drag; their engines own the
    /// position). The dead `AI_015_HandleMovementCollision_Old` (`:44171`, no
    /// caller) is cited dead, not owed — no witness for it.
    #[test]
    fn s1_p_i57_the_skip_list_exempts_the_drag() {
        let w = pond(LiquidKind::Water);
        // 721 is a golf ball (IsAGolfBall); 540/933/1100 are named kinds.
        for kind in [721u16, 739, 752, 540, 933, 1100] {
            let mut p = shot(kind);
            p.tile_collide = true;
            p.wet = true;
            p.vel = Vec2::new(8.0, 4.0);
            assert!(update_position_skips(&p), "kind {kind} is skip-listed");
            assert_eq!(wet_velocity(&p, &w), Vec2::new(8.0, 4.0), "kind {kind}: no drag");
        }
        // A plain arrow (kind 3, aiStyle 1) is NOT skip-listed — it drags.
        let mut a = shot(3);
        a.tile_collide = true;
        a.wet = true;
        a.vel = Vec2::new(8.0, 4.0);
        assert!(!update_position_skips(&a));
        assert_eq!(wet_velocity(&a, &w), Vec2::new(4.0, 2.0));
    }

    /// `:17590-17592` exemptions — aiStyle 10 (the falling-block arm runs its
    /// own `TileCollision` chain, never writes `wetVelocity`) and the latched
    /// 440/449/606 `ai[1] == 1` / 466 `localAI[1] == 1` / 580/640
    /// `localAI[1] > 0` states keep the un-scaled capture while wet.
    #[test]
    fn s1_p_i57_the_aistyle10_and_latch_exemptions() {
        let w = pond(LiquidKind::Water);
        // 466 (CULTIST_LIGHTNING_ORB_ARC) with localAI[1] == 1 is latched.
        let mut p = shot(466);
        p.tile_collide = true;
        p.wet = true;
        p.local_ai[1] = 1.0;
        p.vel = Vec2::new(8.0, 4.0);
        assert_eq!(wet_velocity(&p, &w), Vec2::new(8.0, 4.0), "the 466 latch exempts");
        // Unlatched (localAI[1] == 0) it drags normally.
        p.local_ai[1] = 0.0;
        assert_eq!(wet_velocity(&p, &w), Vec2::new(4.0, 2.0), "unlatched: the drag runs");
    }

    /// `:17619-17626`/`:17633-17640`/`:17647-17654` BLOCKED-axis restore — the
    /// core wet-arm mechanic the d3 boulder arm ports verbatim: a CHANGED axis
    /// keeps its FULL post-collision value, an UNCHANGED axis scales by the
    /// factor. A shot parked flush against a solid wall flying +x: the
    /// `TileCollision` resolves `velocity.X` to 0 (the flush distance), the
    /// y axis untouched. Hand-derived: pos=(470,1616) w=10 → right edge 480 =
    /// the wall face (tiles x=30..32 → rx=480); vel=(6,2) → post=(0,2);
    /// wet = post×0.5 = (0,1), then the changed x restores to 0 → (0,1).
    #[test]
    fn s1_p_i57_the_blocked_axis_restores_full() {
        let mut w = World::new("wall".into(), 200, 200);
        for x in 30..32 {
            for y in 99..104 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        let mut p = shot(3);
        p.tile_collide = true;
        p.wet = true;
        p.pos = Vec2::new(470.0, 101.0 * 16.0); // right edge 480 = the wall face
        p.vel = Vec2::new(6.0, 2.0);
        let got = wet_velocity(&p, &w);
        assert_eq!(got.x, 0.0, "the blocked x keeps its full post-collision value (0)");
        assert_eq!(got.y, 1.0, "the unblocked y scales ×0.5 (2 → 1)");
        // The dry A/B: the same slot dry integrates the full post-collision
        // velocity (0,2) — the wet arm differs ONLY by the y factor.
        p.wet = false;
        assert_eq!(wet_velocity(&p, &w), Vec2::new(6.0, 2.0), "dry: full velocity, no probe");
    }

    /// RNG-neutral A/B (same-seed) — the helper is PURE: it reads the slot +
    /// world and returns a vector, drawing nothing and writing nothing. Two
    /// calls on the same slot/world return the same vector and leave the slot
    /// unchanged (the drag is a physics deviation, not a stream one — the
    /// per-funnel witnesses' determinism argument).
    #[test]
    fn s1_p_i57_rng_neutral_pure_and_idempotent() {
        let w = pond(LiquidKind::Water);
        let mut p = shot(3);
        p.tile_collide = true;
        p.wet = true;
        p.vel = Vec2::new(8.0, 4.0);
        let before_pos = p.pos;
        let before_vel = p.vel;
        let a = wet_velocity(&p, &w);
        let b = wet_velocity(&p, &w);
        assert_eq!(a, b, "the same slot/world yields the same integrate vector");
        assert_eq!(p.pos, before_pos, "the helper writes no position");
        assert_eq!(p.vel, before_vel, "the helper writes no velocity (pure)");
    }
}
