//! The projectile AI-arm `Lighting.AddLight` sources, second wave (`S1-E-5`,
//! W4-e6).
//!
//! C# `Lighting.AddLight` funnels every overload into
//! `Lighting.AddLight(int i, int j, float r, float g, float b)`
//! (`Lighting.cs:157-163`), which appends to the active engine's per-frame
//! list behind `!Main.gamePaused && Main.netMode != 2`
//! (`LightingEngine.cs:51-54`). The engine then max-merges that list onto the
//! working light map (`LightingEngine.cs:208-235`), one frame late. There is
//! ONE list pair per engine — [`crate::entity_lights::EntityLights`] — and
//! this family feeds the same pair as the `ProjLight` chain
//! (`proj_lights.rs::proj_light`), the item family
//! (`entity_lights.rs::item_light`), the NPC family (`npc_lights.rs`) and the
//! Player family (`player_lights.rs`); the tile-anchor vocabulary is shared
//! from `entity_lights.rs` too.
//!
//! Ported here: the **second 40** of `Projectile.cs`'s 107 `Lighting.AddLight`
//! call sites (sites 68-107 in line order, `:54173`…`:78556`) — the AI-arm
//! lights OUTSIDE `ProjLight` (`:16153-16661`, the first wave's chain tail).
//! The first 67 sites are W4-c19's slice. Like the `ProjLight` chain, an arm
//! whose kind has no `proj_data` row is absent wholesale (no slot can carry
//! it), and no ported arm draws `Main.rand` — the feed is pure, so its
//! placement cannot move any stream.
//!
//! **The site register (all 40, in line order):**
//!
//! * `:54173` — 436 BrainScramblerBolt (`AI_001`): `(int)Center/16`,
//!   (0.3, 1.1, 0.5). PORTED.
//! * `:54262` — 467 CultistBossFireBall: no row. Absent.
//! * `:54406` — 468 CultistBossFireBallClone: no row. Absent.
//! * `:54571` — 634/635 NebulaBlaze1/2 (`AI_001`): `AddLight(Center, rgb2)`,
//!   rgb2 = (0.7, 0.1, 0.5) for 634 (`:54470`), (0.4, 0.6, 0.9) for 635
//!   (`:54481`). PORTED.
//! * `:54674` — 459 ChargedBlasterOrb (`AI_001`): `(int)Center/16`,
//!   (0.4, 0.85, 0.9). The `:54670-54673` box rewrite (`position = Center`,
//!   `width = height = (int)(22*scale)`, `Center = position`) keeps the
//!   centre invariant, so the slot's centre is the exact anchor. PORTED.
//! * `:54714` — 709 MonkStaffT3_AltShot: no row. Absent.
//! * `:54755` — 442 ElectrosphereMissile (`AI_001`): `(int)Center/16`,
//!   (0.3, 0.45, 0.8), inside `if (alpha == 0)` AFTER the `localAI[1] >= 120`
//!   kill (`:54750-54754`) — the port's `ai_electrosphere_missile` runs that
//!   kill in the sim, so the feed gates on `alpha == 0` only. PORTED.
//! * `:54784` — 440 LaserMachinegunLaser: no row. Absent.
//! * `:54788` — 449 SaucerLaser (`AI_001`'s 440/449/606 shared arm):
//!   `(int)Center/16`, (0.7, 0.65, 0.3). PORTED.
//! * `:54792` — 606 ScutlixLaser: no row. Absent.
//! * `:54831` — 438 RayGunnerLaser (`AI_001`): `(int)Center/16`,
//!   (0.4, 0.1, 0.2). PORTED.
//! * `:54835` — 593 AncientDoomProjectile: no row. Absent.
//! * `:54858` — 592 MartianWalkerLaser: no row. Absent.
//! * `:54932` — 435 MartianTurretBolt: no row. Absent.
//! * `:54968` — 682 DD2LightningBugZap: no row. Absent.
//! * `:54992` — 684 DD2SquireSonicBoom: no row. Absent.
//! * `:55137` — 350 Missile (`AI_001`): `(int)Center/16`, (0.9, 0.6, 0.2).
//!   PORTED.
//! * `:55171` — 325 FlamingWood (`AI_001`): `(int)Center/16`,
//!   (0.9, 0.6, 0.2). PORTED.
//! * `:56126` — 585 ClothiersCurse: no row. Absent (the colour rides
//!   `NPCID.Sets.MagicAuraColor[54]` = (100, 4, 227)/255, `NPCID.cs:10475`).
//! * `:56294` — 577 VortexLaser: no row. Absent.
//! * `:56298` — 576 NebulaLaser: no row. Absent.
//! * `:56421` — 819 BloodArrow (`AI_001`'s else-if chain): `Center`,
//!   (0.3, 0.05, 0.05). PORTED.
//! * `:57185` — 312 JackOLantern: no row. Absent.
//! * `:58213` — 1094 PalworldMinionFoxsparks (`AI_026`): `Center`,
//!   (0.5, 0.3, 0.1). PORTED.
//! * `:58226` — 1113 PalworldMinionTrustyFoxsparks (`AI_026`): `Center`,
//!   (0.5, 0.3, 0.1). PORTED.
//! * `:61228` — 324 CursedSapling (`AI_026`'s frame chain): no row. Absent.
//! * `:65120` — 613 StardustCellMinion (`AI_062`): `Center`,
//!   (0.2, 0.6, 0.7). PORTED.
//! * `:66272` — 445 LaserDrill (`AI_075`): `(int)Center/16`, the
//!   `localAI[0]/60` triangle-wave `Vector3.Lerp((0,1,0.7),(0,0.7,1),
//!   1-num12*2) * 0.5` colour, gated on `Distance(hand, Center) >= 5f`
//!   (`:66256`). PORTED — but dark in the current port, the WaterBolt
//!   precedent (`proj_lights.rs`): the port's `ai_drill` stand-in pins the
//!   slot's centre to the hand every tick (`projectile.rs::ai_drill`), so
//!   the distance gate reads 0, and the `localAI[0]++`/wrap-60 cycle
//!   (`:66251-66255`) is an AI-side writer that lands with the full
//!   `AI_075` 445 arm — until then the colour reads the stock 0. The C#
//!   pins the drill to the hand at `AI_075`'s TAIL (`:67329`), AFTER this
//!   light, so the vanilla gate reads the pre-pin (moved) centre and the
//!   light is live there. ANALYSIS records the deviation.
//! * `:66683` — 595 Arkhalis (`AI_075`): no row. Absent (the anchor is
//!   `Center + velocity*3`, not the centre).
//! * `:66736` — 735 Terragrim (`AI_075`): no row. Absent (same lead
//!   anchor).
//! * `:68124`/`:68125` — 663 DD2FlameBurstTowerT1 (`AI_130`): the
//!   `Center` + `Bottom + (0,-10)` pair at (0.4, 0.2, 0.1). PORTED.
//! * `:68172`/`:68173` — 665 T2: the same pair `* 1.2f`. PORTED.
//! * `:68224`/`:68225` — 667 T3: the same pair `* 1.5f`. PORTED.
//! * `:69332` — 691/692/693 DD2ExplosiveTrapT1-3 (`AI_138`'s shared tail,
//!   after the sentry-backpack reposition): `Center`, (0.6, 0.5, 0.3).
//!   PORTED.
//! * `:69507` — 707 MonkStaffT3 (`AI_140`): no row. Absent (the anchor is
//!   the OWNER's centre, not the projectile's).
//! * `:69885` — 708 MonkStaffT3_Alt (`AI_142`): no row. Absent (same
//!   owner-centre anchor, behind `!player.frozen`).
//! * `:78556` — the lightning-liquid electrification scan
//!   (`SpawnLightningLiquidElectrificationParticles(int, int)`): the
//!   `Main.rand.Next(4) == 0` gate (`:78549`) draws on the projectile
//!   stream and the `ParticleOrchestrator` request has no port — absent
//!   wholesale (the S1-E-5 RNG rule).
//!
//! The anchor forms are kept as distinct vocabulary because they are
//! distinct C# expressions: `(int)Center.X / 16` truncates the centre
//! coordinate BEFORE the divide ([`XAnchor::CenterTruncated`]); the
//! `AddLight(Vector2, …)` overload (`Lighting.cs:140-143`) truncates the
//! QUOTIENT (`(int)(v / 16f)`, [`XAnchor::Center`]). Truncation commutes
//! with division by a positive integer, so the two agree for every real
//! coordinate — `the_two_anchor_forms_agree_for_every_position` pins that
//! across a sweep (the same invariant the `ProjLight` chain's even-side
//! test relies on).

use glam::Vec2;

use crate::entity_lights::{tile_x, tile_y, XAnchor};
use crate::projectile::{
    BRAIN_SCRAMBLER, CHARGED_BLASTER_ORB, ELECTROSPHERE_MISSILE, FLAMING_WOOD, LASER_DRILL, MISSILE,
    NEBULA_BLAZE_1, NEBULA_BLAZE_2, RAY_GUNNER, SAUCER_LASER, STARDUST_CELL,
};

/// The AI-arm `AddLight` sources this projectile contributes for one frame,
/// in C# call order (the tower arms append two), or an empty list.
///
/// `pos`/`w`/`h` are the slot's post-step box; `alpha` is `Projectile.alpha`
/// (the 442 gate); `local_ai0` is `localAI[0]` (the 445 colour cycle);
/// `hand` is the owner's `RotatedRelativePoint(MountedCenter)` — the 445
/// distance gate's anchor (`AI_075`'s `vector`, `Projectile.cs:66135`),
/// `None` when no owner player resolves (the C# reads `Main.player[owner]`
/// unconditionally; the port skips the arm, the `ai_drill` no-owner kill
/// precedent). Pure — no arm draws RNG and none mutates the slot, so the
/// feed's placement cannot move any stream.
pub(crate) fn proj_ai_lights(
    kind: u16,
    pos: Vec2,
    w: f32,
    h: f32,
    alpha: i32,
    local_ai0: f32,
    hand: Option<Vec2>,
) -> Vec<(i32, i32, [f32; 3])> {
    let mut out = Vec::new();
    let center = Vec2::new(pos.x + w * 0.5, pos.y + h * 0.5);
    // `AI_001`'s per-type arms, in source order.
    if kind == BRAIN_SCRAMBLER {
        // `:54173` — `AddLight((int)Center.X / 16, (int)Center.Y / 16,
        // 0.3f, 1.1f, 0.5f)`.
        out.push((
            tile_x(pos, w, XAnchor::CenterTruncated),
            tile_y(pos, h, XAnchor::CenterTruncated),
            [0.3, 1.1, 0.5],
        ));
    } else if kind == NEBULA_BLAZE_1 || kind == NEBULA_BLAZE_2 {
        // `:54571` — `AddLight(base.Center, rgb2)`; rgb2 is the per-kind
        // head value (`:54470` / `:54481`), untouched by the arm's later
        // writes. The Vector2 overload truncates the quotient.
        let rgb = if kind == NEBULA_BLAZE_1 {
            [0.7, 0.1, 0.5]
        } else {
            [0.4, 0.6, 0.9]
        };
        out.push((
            tile_x(pos, w, XAnchor::Center),
            tile_y(pos, h, XAnchor::Center),
            rgb,
        ));
    } else if kind == CHARGED_BLASTER_ORB {
        // `:54674` — `(int)Center/16`, (0.4, 0.85, 0.9). The `:54670-54673`
        // box rewrite keeps the centre invariant (module docs).
        out.push((
            tile_x(pos, w, XAnchor::CenterTruncated),
            tile_y(pos, h, XAnchor::CenterTruncated),
            [0.4, 0.85, 0.9],
        ));
    } else if kind == ELECTROSPHERE_MISSILE {
        // `:54747-54755` — inside `if (alpha == 0)`, after the
        // `localAI[1] >= 120` kill (the sim's `ai_electrosphere_missile`
        // runs it): `(int)Center/16`, (0.3, 0.45, 0.8).
        if alpha == 0 {
            out.push((
                tile_x(pos, w, XAnchor::CenterTruncated),
                tile_y(pos, h, XAnchor::CenterTruncated),
                [0.3, 0.45, 0.8],
            ));
        }
    } else if kind == SAUCER_LASER {
        // `:54786-54789` — the 440/449/606 shared arm's 449 sub-if:
        // `(int)Center/16`, (0.7, 0.65, 0.3).
        out.push((
            tile_x(pos, w, XAnchor::CenterTruncated),
            tile_y(pos, h, XAnchor::CenterTruncated),
            [0.7, 0.65, 0.3],
        ));
    } else if kind == RAY_GUNNER {
        // `:54829-54832` — `(int)Center/16`, (0.4, 0.1, 0.2).
        out.push((
            tile_x(pos, w, XAnchor::CenterTruncated),
            tile_y(pos, h, XAnchor::CenterTruncated),
            [0.4, 0.1, 0.2],
        ));
    } else if kind == MISSILE || kind == FLAMING_WOOD {
        // `:55130-55137` (350) / `:55164-55171` (325) — the twin arms:
        // `(int)Center/16`, (0.9, 0.6, 0.2). The `alpha == 0` dust halves
        // (`:55138-55157` / `:55172-…`) draw `Main.rand` and are skipped;
        // the light itself is unconditional.
        out.push((
            tile_x(pos, w, XAnchor::CenterTruncated),
            tile_y(pos, h, XAnchor::CenterTruncated),
            [0.9, 0.6, 0.2],
        ));
    } else if kind == crate::projectile::BLOOD_ARROW {
        // `:56388-56421` — the 819 arm's tail: `AddLight(base.Center,
        // 0.3f, 0.05f, 0.05f)`. The dust arms above it are skipped.
        out.push((
            tile_x(pos, w, XAnchor::Center),
            tile_y(pos, h, XAnchor::Center),
            [0.3, 0.05, 0.05],
        ));
    }
    // `AI_026`'s foxsparks pair (`:58203-58227`): the light is
    // unconditional inside each type block (the dead-player flag clear
    // and the `timeLeft = 2` refresh are sim writes; `DoPalAppearEffect`
    // is the dust/visual arm, skipped).
    if kind == 1094 || kind == 1113 {
        // `:58213` (1094) / `:58226` (1113) — `AddLight(base.Center,
        // 0.5f, 0.3f, 0.1f)`.
        out.push((
            tile_x(pos, w, XAnchor::Center),
            tile_y(pos, h, XAnchor::Center),
            [0.5, 0.3, 0.1],
        ));
    }
    if kind == STARDUST_CELL {
        // `AI_062`'s 613 arm (`:65110-65125`): `AddLight(base.Center,
        // 0.2f, 0.6f, 0.7f)` (`:65120`), unconditional inside the block.
        out.push((
            tile_x(pos, w, XAnchor::Center),
            tile_y(pos, h, XAnchor::Center),
            [0.2, 0.6, 0.7],
        ));
    }
    if kind == LASER_DRILL {
        // `AI_075`'s 445 arm (`:66244-66273`): the light sits inside
        // `if (Vector2.Distance(vector, base.Center) >= 5f)` (`:66256`),
        // `vector` = the owner's `RotatedRelativePoint(MountedCenter)`
        // (`:66135`). The colour is the `localAI[0]/60` triangle wave:
        // `num12 = localAI[0] / 60f; if (num12 > 0.5f) num12 = 1f - num12;`
        // (`:66258-66262`), then `Vector3.Lerp((0,1,0.7), (0,0.7,1),
        // 1f - num12 * 2f) * 0.5f` (`:66263`). `Vector3.Lerp` is UNCLAMPED
        // (`value1 + (value2 - value1) * amount`); the `localAI[0]++` /
        // wrap-60 cycle (`:66251-66255`) is an AI-side writer — the port's
        // `ai_drill` stand-in does not run it, so the slot reads the stock
        // 0 and the colour stays at the wave's `localAI[0] == 0` point
        // (the WaterBolt dead-arm precedent, module docs). The
        // `>= 30f` `PlotTileLine` flood half (`:66264-66271`) is the
        // light-map write with no port — skipped, the 870 precedent.
        if let Some(hand) = hand {
            if hand.distance(center) >= 5.0 {
                let mut num12 = local_ai0 / 60.0;
                if num12 > 0.5 {
                    num12 = 1.0 - num12;
                }
                let t = 1.0 - num12 * 2.0;
                let lerp = |a: f32, b: f32| a + (b - a) * t;
                out.push((
                    tile_x(pos, w, XAnchor::CenterTruncated),
                    tile_y(pos, h, XAnchor::CenterTruncated),
                    [
                        lerp(0.0, 0.0) * 0.5,
                        lerp(1.0, 0.7) * 0.5,
                        lerp(0.7, 1.0) * 0.5,
                    ],
                ));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(kind: u16) -> Vec<(i32, i32, [f32; 3])> {
        proj_ai_lights(kind, Vec2::new(160.0, 176.0), 16.0, 16.0, 0, 0.0, None)
    }

    #[test]
    fn the_scrambler_bolt_truncates_the_centre_before_dividing() {
        // `:54173` — (0.3, 1.1, 0.5) at `(int)Center/16`. The 16x16 box at
        // (160, 176) centres on (168, 184) → (10, 11).
        let got = at(BRAIN_SCRAMBLER);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0], (10, 11, [0.3, 1.1, 0.5]));
    }

    #[test]
    fn the_nebula_blazes_take_their_head_colours() {
        // `:54470`/`:54481`/`:54571` — 634 (0.7, 0.1, 0.5), 635
        // (0.4, 0.6, 0.9), both at the quotient-truncated centre.
        assert_eq!(at(NEBULA_BLAZE_1)[0].2, [0.7, 0.1, 0.5]);
        assert_eq!(at(NEBULA_BLAZE_2)[0].2, [0.4, 0.6, 0.9]);
        assert_eq!(at(NEBULA_BLAZE_1)[0].0, 10);
    }

    #[test]
    fn the_charged_blaster_orb_lights_its_invariant_centre() {
        // `:54674` — (0.4, 0.85, 0.9); the box rewrite keeps the centre.
        assert_eq!(at(CHARGED_BLASTER_ORB)[0], (10, 11, [0.4, 0.85, 0.9]));
    }

    #[test]
    fn the_electrosphere_missile_gates_on_alpha_zero() {
        // `:54747-54755` — the light sits inside `if (alpha == 0)`.
        assert!(proj_ai_lights(
            ELECTROSPHERE_MISSILE,
            Vec2::new(160.0, 176.0),
            14.0,
            14.0,
            255,
            0.0,
            None,
        )
        .is_empty());
        let got = at(ELECTROSPHERE_MISSILE);
        assert_eq!(got[0].2, [0.3, 0.45, 0.8]);
    }

    #[test]
    fn the_saucer_laser_and_ray_gunner_keep_their_colours() {
        // `:54788` — 449 (0.7, 0.65, 0.3); `:54831` — 438 (0.4, 0.1, 0.2).
        assert_eq!(at(SAUCER_LASER)[0].2, [0.7, 0.65, 0.3]);
        assert_eq!(at(RAY_GUNNER)[0].2, [0.4, 0.1, 0.2]);
    }

    #[test]
    fn the_two_anchor_forms_agree_for_every_position() {
        // `(int)Center.X / 16` (the 436/459/442/449/438 arms) truncates the
        // centre coordinate BEFORE the divide; the `AddLight(Vector2, …)`
        // overload (the 634/635 arms) truncates the QUOTIENT
        // (`Lighting.cs:140-143`). The two are kept as distinct vocabulary
        // (they are distinct C# expressions), but truncation commutes with
        // division by a positive integer, so they agree for every real
        // coordinate — including negatives. Pin the agreement across a
        // sweep so a future anchor change cannot silently diverge them.
        for i in -40..40 {
            let px = i as f32 * 3.7 - 60.0;
            let py = i as f32 * 2.3 - 45.0;
            let pos = Vec2::new(px, py);
            let trunc = proj_ai_lights(BRAIN_SCRAMBLER, pos, 16.0, 16.0, 0, 0.0, None);
            let quot = proj_ai_lights(NEBULA_BLAZE_1, pos, 16.0, 16.0, 0, 0.0, None);
            let cx = px + 8.0;
            let cy = py + 8.0;
            assert_eq!((trunc[0].0, trunc[0].1), (cx as i32 / 16, cy as i32 / 16));
            assert_eq!(
                (quot[0].0, quot[0].1),
                ((cx / 16.0) as i32, (cy / 16.0) as i32)
            );
            assert_eq!((trunc[0].0, trunc[0].1), (quot[0].0, quot[0].1));
        }
    }

    #[test]
    fn the_missile_and_flaming_wood_share_the_orange_arm() {
        // `:55137` (350) / `:55171` (325) — (0.9, 0.6, 0.2) at
        // `(int)Center/16`, unconditional (the dust halves are skipped).
        assert_eq!(at(MISSILE)[0], (10, 11, [0.9, 0.6, 0.2]));
        assert_eq!(at(FLAMING_WOOD)[0], (10, 11, [0.9, 0.6, 0.2]));
    }

    #[test]
    fn the_blood_arrow_lights_its_centre_red() {
        // `:56421` — 819 `AddLight(Center, 0.3, 0.05, 0.05)`.
        assert_eq!(
            at(crate::projectile::BLOOD_ARROW)[0],
            (10, 11, [0.3, 0.05, 0.05])
        );
    }

    #[test]
    fn the_foxsparks_pair_lights_the_same_warm_colour() {
        // `:58213` (1094) / `:58226` (1113) — `AddLight(Center,
        // 0.5, 0.3, 0.1)`, unconditional inside each type block.
        assert_eq!(at(1094)[0], (10, 11, [0.5, 0.3, 0.1]));
        assert_eq!(at(1113)[0], (10, 11, [0.5, 0.3, 0.1]));
    }

    #[test]
    fn the_stardust_cell_lights_cyan() {
        // `:65120` — 613 `AddLight(Center, 0.2, 0.6, 0.7)`.
        assert_eq!(at(STARDUST_CELL)[0], (10, 11, [0.2, 0.6, 0.7]));
    }

    #[test]
    fn the_laser_drill_gates_on_the_hand_distance() {
        // `:66256` — the light needs `Distance(hand, Center) >= 5f`. The
        // 16x16 box centres on (168, 184).
        let pos = Vec2::new(160.0, 176.0);
        // No owner resolves → no light (the ai_drill no-owner precedent).
        assert!(proj_ai_lights(LASER_DRILL, pos, 16.0, 16.0, 0, 0.0, None).is_empty());
        // The hand on the centre (the port's ai_drill pin) → distance 0,
        // the gate fails — the recorded dark-in-the-port deviation.
        let on_centre = Vec2::new(168.0, 184.0);
        assert!(
            proj_ai_lights(LASER_DRILL, pos, 16.0, 16.0, 0, 0.0, Some(on_centre)).is_empty()
        );
        // A hand 5+ px away passes the gate.
        let far = Vec2::new(168.0, 178.0); // distance 6
        let got = proj_ai_lights(LASER_DRILL, pos, 16.0, 16.0, 0, 0.0, Some(far));
        assert_eq!(got.len(), 1);
        assert_eq!((got[0].0, got[0].1), (10, 11));
    }

    #[test]
    fn the_laser_drill_colour_is_the_local_ai0_triangle_wave() {
        // `:66258-66263` — num12 = localAI[0]/60, folded above 0.5, then
        // Lerp((0,1,0.7),(0,0.7,1), 1-num12*2) * 0.5. At localAI[0] == 0:
        // num12 = 0, t = 1 → the (0, 0.7, 1) end * 0.5 = (0, 0.35, 0.5).
        let pos = Vec2::new(160.0, 176.0);
        let far = Vec2::new(168.0, 178.0);
        let zero = proj_ai_lights(LASER_DRILL, pos, 16.0, 16.0, 0, 0.0, Some(far));
        assert_eq!(zero[0].2, [0.0, 0.35, 0.5]);
        // At localAI[0] == 30: num12 = 0.5, t = 0 → the (0, 1, 0.7) start
        // * 0.5 = (0, 0.5, 0.35).
        let mid = proj_ai_lights(LASER_DRILL, pos, 16.0, 16.0, 0, 30.0, Some(far));
        assert_eq!(mid[0].2, [0.0, 0.5, 0.35]);
        // At localAI[0] == 60 the C# wraps to 0 before the next tick; the
        // feed reads the slot verbatim (num12 = 1 → folded to 0 → t = 1),
        // the same colour as 0 — the triangle wave's symmetry.
        let end = proj_ai_lights(LASER_DRILL, pos, 16.0, 16.0, 0, 60.0, Some(far));
        assert_eq!(end[0].2, [0.0, 0.35, 0.5]);
    }

    #[test]
    fn every_ported_kind_has_a_proj_data_row() {
        // The absent-wholesale rule's invariant: every kind this feed
        // serves resolves a `proj_data` row (the spawn gate).
        for kind in [
            BRAIN_SCRAMBLER,
            NEBULA_BLAZE_1,
            NEBULA_BLAZE_2,
            CHARGED_BLASTER_ORB,
            ELECTROSPHERE_MISSILE,
            SAUCER_LASER,
            RAY_GUNNER,
            MISSILE,
            FLAMING_WOOD,
            crate::projectile::BLOOD_ARROW,
            1094,
            1113,
            STARDUST_CELL,
            LASER_DRILL,
        ] {
            assert!(
                crate::projectile::proj_data(kind).is_some(),
                "kind {kind} must have a row"
            );
        }
    }
}
