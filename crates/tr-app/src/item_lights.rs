//! The `Main.rand`-drawing and `Main.essScale` light arms of
//! `WorldItem.UpdateItem_VisualEffects` (`WorldItem.cs:1206-1293`, S1-E-5).
//!
//! Disjoint from [`crate::entity_lights::item_light`] (the RNG-free table):
//! these arms draw the item-update stream, so they run **inside the fixed
//! step** (`world_sim/items.rs`, at the C#'s `:436` call position — after the
//! movement half, per drop, in `Main.item` slot order) on the `UpdateItems`
//! swap's stream (`Main.cs:18024`), not in the frame feed. Both feeds append
//! to the same [`crate::entity_lights::EntityLights`] list pair.
//!
//! **Ported:** the twelve flicker arms (5043, 3191, 520/3454, 521/3455,
//! 547/3453, 548, 575, 549, 58/1734/1867, 184/1735/1868/4143, 522, 1332 —
//! each draws exactly one `Main.rand.Next(90, 111)` and nothing else) and the
//! four `Main.essScale` arms (3456-3459, RNG-free).
//!
//! **Deferred, draw-free skips (the stream stays exact):**
//! * 116 (`:1211-1222`): the flicker draw is followed by a `Next(250) == 0`
//!   gate whose `Dust.NewDust` chain draws a pool-state-dependent number of
//!   further rolls — porting the light half alone would shorten the stream.
//!   The whole arm is skipped until the item dust pass lands.
//! * 501 (`:1295-1303`): dust-only, no `AddLight` — nothing to port here.
//! * 75 / `BossBag` (`:1624-1648`): the light halves are already in
//!   `entity_lights::item_light`; the `timeSinceItemSpawned % 12` dust halves
//!   (`NextVector2CircularEdge` + two `NextFloat`s + the `NewDust` chain) stay
//!   skipped with the dust pass.

use glam::Vec2;
use tr_core::UnifiedRandom;

use crate::entity_lights::{XAnchor, tile_x, tile_y};

/// `WorldItem.UpdateItem_VisualEffects`'s rand/essScale head
/// (`WorldItem.cs:1206-1293`): the light this drop contributes this tick, or
/// `None`. Draws `rng` (the `UpdateItems` stream) exactly where the C# draws
/// `Main.rand`; every other kind returns `None` without touching the stream.
///
/// `pos` is the drop's top-left after the movement half and `size` its box
/// side (`ItemDrop::SIZE`, 16 — C# `width`/`height`, equal for a drop).
/// `ess_scale` is `Main.essScale` as of the last draw pass
/// (`Main.cs:62264-62277`).
pub(crate) fn rand_light(
    kind: u16,
    pos: Vec2,
    size: f32,
    ess_scale: f32,
    rng: &mut UnifiedRandom,
) -> Option<(i32, i32, [f32; 3])> {
    // Every arm here positions by `(int)((position + size/2) / 16f)` — the
    // rand arms literally (`:1209` etc.), the essScale arms through the
    // `AddLight(Vector2, Vector3)` overload's per-axis quotient truncation
    // (`Lighting.cs:135-138`). One anchor for the whole head.
    let rgb: [f32; 3] = if kind == 5043 {
        // `:1206-1210` — the Torch God's Favor's white flicker.
        let num = rng.next_range(90, 111) as f32 * 0.01 * ((ess_scale + 0.5) / 2.0);
        [0.25 * num, 0.25 * num, 0.25 * num]
    } else if kind == 116 {
        // `:1211-1222` — deferred wholesale (the dust gate, module docs).
        return None;
    } else if kind == 3191 {
        // `:1224-1228` — the Enchanted Nightcrawler's pink flicker.
        let num = rng.next_range(90, 111) as f32 * 0.01 * ((ess_scale + 0.5) / 2.0);
        [0.3 * num, 0.1 * num, 0.25 * num]
    } else if kind == 520 || kind == 3454 {
        // `:1229-1233` — Soul of Light / Nebula Pickup 2.
        let num = rng.next_range(90, 111) as f32 * 0.01 * ess_scale;
        [0.5 * num, 0.1 * num, 0.25 * num]
    } else if kind == 521 || kind == 3455 {
        // `:1234-1238` — Soul of Night / Nebula Pickup 3.
        let num = rng.next_range(90, 111) as f32 * 0.01 * ess_scale;
        [0.25 * num, 0.1 * num, 0.5 * num]
    } else if kind == 547 || kind == 3453 {
        // `:1239-1243` — Soul of Fright / Nebula Pickup 1.
        let num = rng.next_range(90, 111) as f32 * 0.01 * ess_scale;
        [0.5 * num, 0.3 * num, 0.05 * num]
    } else if kind == 548 {
        // `:1244-1248` — Soul of Might.
        let num = rng.next_range(90, 111) as f32 * 0.01 * ess_scale;
        [0.1 * num, 0.1 * num, 0.6 * num]
    } else if kind == 575 {
        // `:1249-1253` — Soul of Flight.
        let num = rng.next_range(90, 111) as f32 * 0.01 * ess_scale;
        [0.1 * num, 0.3 * num, 0.5 * num]
    } else if kind == 549 {
        // `:1254-1258` — Soul of Sight.
        let num = rng.next_range(90, 111) as f32 * 0.01 * ess_scale;
        [0.1 * num, 0.5 * num, 0.2 * num]
    } else if kind == 58 || kind == 1734 || kind == 1867 {
        // `:1259-1263` — Heart / Candy Apple / Candy Cane.
        let num = rng.next_range(90, 111) as f32 * 0.01 * (ess_scale * 0.5);
        [0.5 * num, 0.1 * num, 0.1 * num]
    } else if kind == 184 || kind == 1735 || kind == 1868 || kind == 4143 {
        // `:1264-1268` — Star / Soul Cake / Sugar Plum / Mana Cloak Star.
        let num = rng.next_range(90, 111) as f32 * 0.01 * (ess_scale * 0.5);
        [0.1 * num, 0.1 * num, 0.5 * num]
    } else if kind == 522 {
        // `:1269-1273` — Cursed Flame.
        let num = rng.next_range(90, 111) as f32 * 0.01 * (ess_scale * 0.2);
        [0.5 * num, 1.0 * num, 0.1 * num]
    } else if kind == 1332 {
        // `:1274-1278` — Ichor's yellow flicker.
        let num = rng.next_range(90, 111) as f32 * 0.01 * (ess_scale * 0.2);
        [1.0 * num, 1.0 * num, 0.1 * num]
    } else if kind == 3456 {
        // `:1279-1282` — the lunar fragments (Vortex/Nebula/Solar/Stardust):
        // `Vector3 * Main.essScale`, no rand draw.
        [0.2 * ess_scale, 0.4 * ess_scale, 0.5 * ess_scale]
    } else if kind == 3457 {
        [0.4 * ess_scale, 0.2 * ess_scale, 0.5 * ess_scale] // `:1285`
    } else if kind == 3458 {
        [0.5 * ess_scale, 0.4 * ess_scale, 0.2 * ess_scale] // `:1289`
    } else if kind == 3459 {
        [0.2 * ess_scale, 0.2 * ess_scale, 0.5 * ess_scale] // `:1293`
    } else {
        return None;
    };
    Some((
        tile_x(pos, size, XAnchor::Center),
        tile_y(pos, size, XAnchor::Center),
        rgb,
    ))
}

/// `Main.essScale`'s draw-pass drift (`Main.cs:62264-62277`): once per draw,
/// behind `!Main.gamePaused` (the caller's gate), `essScale += essDir * 0.01f`
/// bouncing between 0.7 and 1.0. Seeded 1.0 / -1 (`Main.cs:66673-66674`).
/// The step-time light arms read the value the last draw left, exactly like
/// the C# (the update runs before the draw within a frame).
pub(crate) fn update_ess_scale(ess_scale: &mut f32, ess_dir: &mut i32) {
    *ess_scale += *ess_dir as f32 * 0.01;
    if *ess_scale > 1.0 {
        *ess_dir = -1;
        *ess_scale = 1.0;
    }
    // The C# compares `(double)essScale < 0.7` — the f32 promotes exactly.
    if (*ess_scale as f64) < 0.7 {
        *ess_dir = 1;
        *ess_scale = 0.7;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One flicker arm spends exactly one `Next(90, 111)` and the light is
    /// the C#'s product of that draw — the twin replays the draw directly.
    #[test]
    fn a_flicker_arm_spends_exactly_one_next_90_111() {
        for kind in [
            5043u16, 3191, 520, 3454, 521, 3455, 547, 3453, 548, 575, 549, 58, 1734, 1867, 184,
            1735, 1868, 4143, 522, 1332,
        ] {
            let mut rng = UnifiedRandom::new(4242);
            let got = rand_light(kind, Vec2::new(160.0, 176.0), 16.0, 1.0, &mut rng)
                .expect("a flicker kind lights");
            let mut twin = UnifiedRandom::new(4242);
            let draw = twin.next_range(90, 111);
            assert_eq!(rng, twin, "kind {kind}: exactly one Next(90,111)");
            // The anchor: center (168, 184) -> tile (10, 11).
            assert_eq!((got.0, got.1), (10, 11), "kind {kind}");
            // The colour is a positive multiple of the drawn flicker.
            let scale = match kind {
                5043 | 3191 => (1.0f32 + 0.5) / 2.0,
                520 | 3454 | 521 | 3455 | 547 | 3453 | 548 | 575 | 549 => 1.0,
                522 | 1332 => 1.0 * 0.2,
                _ => 1.0 * 0.5,
            };
            let num = draw as f32 * 0.01 * scale;
            let base: [f32; 3] = match kind {
                5043 => [0.25, 0.25, 0.25],
                3191 => [0.3, 0.1, 0.25],
                520 | 3454 => [0.5, 0.1, 0.25],
                521 | 3455 => [0.25, 0.1, 0.5],
                547 | 3453 => [0.5, 0.3, 0.05],
                548 => [0.1, 0.1, 0.6],
                575 => [0.1, 0.3, 0.5],
                549 => [0.1, 0.5, 0.2],
                58 | 1734 | 1867 => [0.5, 0.1, 0.1],
                184 | 1735 | 1868 | 4143 => [0.1, 0.1, 0.5],
                522 => [0.5, 1.0, 0.1],
                1332 => [1.0, 1.0, 0.1],
                _ => unreachable!(),
            };
            assert_eq!(
                got.2,
                [base[0] * num, base[1] * num, base[2] * num],
                "kind {kind}: the C# product order"
            );
        }
    }

    /// The skipped and unknown kinds are draw-free: the stream is untouched.
    #[test]
    fn skipped_and_unknown_kinds_never_draw() {
        // 116 is deferred wholesale (its dust gate), 501 is dust-only, 75 and
        // the boss bags live in the RNG-free table, 2 is a plain block.
        for kind in [116u16, 501, 75, 3318, 2, 1970, 0] {
            let mut rng = UnifiedRandom::new(7);
            assert!(
                rand_light(kind, Vec2::ZERO, 16.0, 1.0, &mut rng).is_none(),
                "kind {kind} contributes nothing here"
            );
            assert_eq!(rng, UnifiedRandom::new(7), "kind {kind} drew nothing");
        }
    }

    /// The four lunar-fragment arms scale their fixed Vector3 by `essScale`
    /// and draw nothing.
    #[test]
    fn the_fragment_arms_scale_by_ess_scale_without_drawing() {
        let rows: [(u16, [f32; 3]); 4] = [
            (3456, [0.2, 0.4, 0.5]),
            (3457, [0.4, 0.2, 0.5]),
            (3458, [0.5, 0.4, 0.2]),
            (3459, [0.2, 0.2, 0.5]),
        ];
        for (kind, base) in rows {
            let mut rng = UnifiedRandom::new(99);
            let got = rand_light(kind, Vec2::new(-20.0, -20.0), 16.0, 0.7, &mut rng)
                .expect("a lunar fragment lights");
            assert_eq!(rng, UnifiedRandom::new(99), "kind {kind} is RNG-free");
            assert_eq!(
                got.2,
                [base[0] * 0.7, base[1] * 0.7, base[2] * 0.7],
                "kind {kind}"
            );
            // The Vector2 overload truncates the quotient: center -12 ->
            // (int)(-12/16) = 0, not -1.
            assert_eq!((got.0, got.1), (0, 0));
        }
    }

    /// The essScale multiplier rides the flicker: at 0.7 the soul arms dim
    /// to 70%, the heart/star arms to 35%, the 522/1332 arms to 14%.
    #[test]
    fn ess_scale_dims_the_flicker_arms() {
        let mut a = UnifiedRandom::new(5);
        let mut b = UnifiedRandom::new(5);
        let full = rand_light(520, Vec2::ZERO, 16.0, 1.0, &mut a).unwrap().2;
        let dim = rand_light(520, Vec2::ZERO, 16.0, 0.7, &mut b).unwrap().2;
        assert_eq!(a, b, "the same single draw either way");
        assert_eq!(dim, [full[0] * 0.7, full[1] * 0.7, full[2] * 0.7]);
    }

    /// `Main.cs:62264-62277`: the drift bounces at 1.0 and 0.7 and flips the
    /// direction; from the 1.0/-1 seed the floor lands within ~31 draws (the
    /// f32 accumulation of 0.01 is the C#'s arithmetic too).
    #[test]
    fn ess_scale_drifts_between_07_and_10() {
        let mut s = 1.0f32;
        let mut d = -1i32;
        let mut draws = 0;
        while d == -1 && draws < 40 {
            update_ess_scale(&mut s, &mut d);
            draws += 1;
        }
        assert_eq!(d, 1, "the floor flips the direction");
        assert_eq!(s, 0.7, "the floor clamp is exact");
        assert!((30..=31).contains(&draws), "the ~30-draw descent");
        for _ in 0..60 {
            update_ess_scale(&mut s, &mut d);
        }
        assert!(s <= 1.0 && s >= 0.7, "stays in the band, got {s}");
        // The ceiling clamp is exact.
        s = 0.999;
        d = 1;
        update_ess_scale(&mut s, &mut d);
        assert_eq!(s, 1.0);
        assert_eq!(d, -1);
    }
}
