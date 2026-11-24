//! The herbs family of the world-growth ticks (S1-K-3d split out of
//! `growth.rs`; behaviour frozen - verbatim move). The alch-seed gate
//! (`WorldGen.cs:71606-71614`), `GrowAlch` (`:45777-45890`) and the
//! harvestability predicates `IsAlchemyPlantHarvestable` (`:66148-66175`)
//! + `IsHarvestableHerbWithSeed` (`:66118-66138`) over the caller-passed
//! `WeatherCtx` seam. The public seams stay re-exported from `growth`.

use crate::UnifiedRandom;
use crate::world::World;

/// `TileID.Sets.tileAlch` (`TileID.cs`) — the herb tiles GrowAlch runs on.
pub const TILE_ALCH: [u16; 3] = [82, 83, 84];

/// The alch-seed gate (`:71606-71614`): `Next(num7·100) == 0` skips the
/// pick; `num7 = Lerp(151, 151·2.8, clamp(maxTilesX/4200 − 1, 0, 1))`.
pub fn alch_seed_skips(world: &World, rng: &mut UnifiedRandom) -> bool {
    let t = ((world.width as f64 / 4200.0 - 1.0).clamp(0.0, 1.0));
    let num7 = 151.0 + (151.0 * 2.8 - 151.0) * t;
    rng.next_max((num7 * 100.0) as i32) == 0
}

/// `GrowAlch` (`WorldGen.cs:45777-45890`): the herb lifecycle. The stages
/// ride `frameX` (0/18/36/72/90/108) and the bloom target 83; the mature 84
/// blooms back; the liquid kill keeps only the matching stage.
pub fn grow_alch(world: &mut World, x: i32, y: i32, day_time: bool, raining: bool, rng: &mut UnifiedRandom) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    if !t.active() {
        return;
    }
    // `:45784-45796`: the liquid kill — the wrong liquid for the stage
    // destroys the herb (the lava keeps only the stage-5 cactus-stem shape).
    if t.liquid > 0 {
        let stage = t.frame_x / 18;
        let lava = t.flags & crate::tile::FLAG_LIQUID_LAVA != 0;
        let water = t.liquid_kind() == crate::tile::LiquidKind::Water;
        let keep = (lava && stage == 5) || (water && (stage == 1 || stage == 4));
        if !keep {
            // W4-g83: the kill cascade's framing dispatch rides the caller's
            // ACTIVE genRand stream (the growth tick's UpdateWorldGen).
            crate::edit::kill_tile(world, x, y, false, rng);
            return;
        }
    }
    if t.kind == 82 {
        // The sprout matures: the special cues bloom straight into 83.
        let frame_x = world.tile(x, y).map(|t| t.frame_x).unwrap_or(0);
        let below_surface = y as f64 > world.world_surface;
        let near_underworld = y > world.height - 200;
        // `:45798-45811`: the generic 1/50 arm. Only the fireblossom stem
        // (frameX 108) needs the extra `Next(2) == 0`; every other frame
        // blooms straight through (the C# `else { flag = true; }`). The
        // `Next(2)` draw rides the `||` short-circuit, so it fires ONLY on
        // frame 108 — the draw order matches the source. (W4-d105 doc-sweep:
        // the prior `frame_x == 108` inverted the gate — it bloomed 108
        // unconditionally and made every other frame draw the extra `Next(2)`.)
        let bloom = if rng.next_max(50) == 0 && (frame_x != 108 || rng.next_max(2) == 0) {
            true
        } else if day_time && frame_x == 0 && rng.next_max(50) == 0 {
            true
        } else if !day_time && frame_x == 18 && rng.next_max(50) == 0 {
            true
        } else if raining && frame_x == 72 && rng.next_max(50) == 0 {
            true
        } else if below_surface && frame_x == 36 && rng.next_max(50) == 0 {
            true
        } else if near_underworld && frame_x == 90 && rng.next_max(50) == 0 {
            true
        } else {
            false
        };
        if bloom {
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = 83;
            }
        }
        return;
    }
    // `:45868-45889`: the stage-3 flash (`frameX == 36`, 2 of 3 rolls skip).
    // The C# else-if chain: the `Next(3)` draws on EVERY frame-36 cell (a 0
    // falls out of the whole chain — the 108 tail cannot match frame 36);
    // the 108 tail below is the chain's final `else if`, so it runs only
    // when the frame-36 gate fails. The port keeps the else-if shape so the
    // stream matches (no early return on a failed Next(3)).
    if world.tile(x, y).is_some_and(|t| t.frame_x == 36) && rng.next_max(3) != 0 {
        let kind = world.tile(x, y).map(|t| t.kind).unwrap_or(0);
        if kind == 83 {
            if rng.next_max(2) == 0 {
                if let Some(t) = world.tile_mut(x, y) {
                    t.kind = 84;
                }
            }
        } else if kind == 84 || rng.next_max(5) == 0 {
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = 83;
            }
        }
    } else if world.tile(x, y).is_some_and(|t| t.frame_x == 108 && t.kind == 83)
        && rng.next_max(30) == 0
    {
        // `:45890-45898`: the fireblossom tail — a blooming fireblossom
        // (83, frameX 108) matures to 84 at 1/30. (W4-d105 doc-sweep: the
        // arm was missing; the `Next(30)` draws only on a kind-83
        // frame-108 cell.)
        if let Some(t) = world.tile_mut(x, y) {
            t.kind = 84;
        }
    }
}


/// `Main`'s weather/seed state `IsAlchemyPlantHarvestable` reads
/// (`WorldGen.cs:66148-66175`) beyond the `World`'s saved fields: the
/// app's weather clock passes the live values.
#[derive(Clone, Copy, Default)]
pub struct WeatherCtx {
    /// `Main.raining`.
    pub raining: bool,
    /// `Main.cloudAlpha`.
    pub cloud_alpha: f32,
    /// `Main.remixWorld`.
    pub remix_world: bool,
}

/// `WorldGen.IsAlchemyPlantHarvestable` (`WorldGen.cs:66148-66175`): the
/// per-style condition an herb may be harvested with a seed in hand —
/// style 0 daybloom (day), 1 moonglow (night), 3 deathweed (night +
/// blood moon / full moon), 4 waterleaf (rain or any cloud), 5 fireblossom
/// (dry or below the surface, late afternoon).
pub fn is_alchemy_plant_harvestable(world: &World, style: i32, y: i32, wx: &WeatherCtx) -> bool {
    if style == 0 && world.day_time {
        return true;
    }
    if style == 1 && !world.day_time {
        return true;
    }
    if style == 3 && !world.day_time && (world.blood_moon || world.moon_phase == 0) {
        return true;
    }
    if style == 4 && (wx.raining || wx.cloud_alpha > 0.0) {
        return true;
    }
    if style == 5 {
        let mut flag = y as f64 > world.world_surface;
        if wx.remix_world {
            flag = y < world.height - 350;
        }
        // The C# `(!Main.raining | flag)` is the bitwise bool `|` — both
        // operands are pure, so `||` is equivalent.
        if ((!wx.raining || flag) && world.time > 40500.0) {
            return true;
        }
    }
    false
}

/// `WorldGen.IsHarvestableHerbWithSeed` (`WorldGen.cs:66118-66138`): only
/// the grown herbs 83/84 carry the seed-harvest arm — 84 always is, and
/// 83 when its style's condition holds (the plain herb 82 never is).
pub fn is_harvestable_herb_with_seed(
    world: &World,
    kind: u16,
    style: i32,
    y: i32,
    wx: &WeatherCtx,
) -> bool {
    match kind {
        82 => false,
        83 | 84 => kind == 84 || is_alchemy_plant_harvestable(world, style, y, wx),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::growth::test_fixtures::{put, world};
    use crate::tile::FLAG_ACTIVE;

    #[test]
    fn the_alch_gate_uses_the_world_size_lerp() {
        // `:71606-71614`: `num7 = Lerp(151, 422.8, maxTilesX/4200 − 1)` —
        // the 4200-wide world sits at t = 0 → 151; a 8400-wide world at
        // t = 1 → 422.8. The gate skips on `Next(num7·100) == 0`.
        let w = world();
        let mut rng = UnifiedRandom::new(5);
        let skips = (0..1000).filter(|_| alch_seed_skips(&w, &mut rng)).count();
        // The skip chance = 1/(151·100) ≈ 1/15100 → ~0.07/1000 rolls; the
        // 4200-world roll is nearly never skipped.
        assert!(skips <= 2, "skips {skips}");
    }

    #[test]
    fn the_herb_lifecycle_blooms_by_the_cues() {
        // The sprout 82 (the stage-0 frame) blooms in the daylight.
        let mut w = world();
        put(&mut w, 30, 90, 82, 0);
        let mut rng = UnifiedRandom::new(1);
        let mut bloomed = false;
        for _ in 0..4000 {
            grow_alch(&mut w, 30, 90, true, false, &mut rng);
            if w.tile(30, 90).unwrap().kind == 83 {
                bloomed = true;
                break;
            }
        }
        assert!(bloomed, "the daylight cue blooms the stage-0 sprout");
        // The rain cue drives the stage-72 frame.
        let mut w = world();
        put(&mut w, 30, 90, 82, 72);
        let mut rng = UnifiedRandom::new(2);
        let mut bloomed = false;
        for _ in 0..4000 {
            grow_alch(&mut w, 30, 90, false, true, &mut rng);
            if w.tile(30, 90).unwrap().kind == 83 {
                bloomed = true;
                break;
            }
        }
        assert!(bloomed, "the rain cue blooms the stage-72 frame");
        // The generic arm (`:45785-45800`): every frame except 108 blooms on
        // `Next(50) == 0` regardless of the cue — the cue arms are the
        // chain's fallbacks.
        let mut w = world();
        put(&mut w, 30, 90, 82, 0);
        let mut rng = UnifiedRandom::new(3);
        let mut bloomed = false;
        for _ in 0..400 {
            grow_alch(&mut w, 30, 90, false, false, &mut rng);
            if w.tile(30, 90).unwrap().kind == 83 {
                bloomed = true;
                break;
            }
        }
        assert!(bloomed, "the generic 1/50 arm blooms any frame");
    }

    #[test]
    fn the_wrong_liquid_kills_the_herb() {
        let mut w = world();
        put(&mut w, 30, 90, 82, 0);
        let i = w.index(30, 90);
        w.tiles[i].liquid = 200;
        let mut rng = UnifiedRandom::new(1);
        grow_alch(&mut w, 30, 90, true, false, &mut rng);
        assert!(
            w.tile(30, 90).unwrap().flags & FLAG_ACTIVE == 0,
            "the water on the stage-0 herb kills it"
        );
    }

    /// W4-d105 doc-sweep: the generic 1/50 arm's frame-108 gate
    /// (`WorldGen.cs:45798-45811`). The fireblossom stem (frameX 108) blooms
    /// only when the extra `Next(2) == 0` passes; every other frame blooms
    /// straight through. Seeds 35/102 are hand-derived .NET UnifiedRandom
    /// fixtures whose FIRST `Next(50)` is 0 and whose following `Next(2)` is
    /// 0 (seed 35) / 1 (seed 102) — one tick, no cue arms involved.
    #[test]
    fn w4_d105_the_generic_bloom_gate_is_frame_108_only() {
        // Frame 0 (daybloom stem): the generic arm blooms with NO extra
        // draw — seed 35's first Next(50) == 0 suffices.
        let mut w = world();
        put(&mut w, 30, 90, 82, 0);
        let mut rng = UnifiedRandom::new(35);
        grow_alch(&mut w, 30, 90, false, false, &mut rng);
        assert_eq!(w.tile(30, 90).unwrap().kind, 83, "frame 0 blooms outright");
        // Frame 108 + Next(2) == 0 (seed 35): the fireblossom blooms too.
        let mut w = world();
        put(&mut w, 30, 90, 82, 108);
        let mut rng = UnifiedRandom::new(35);
        grow_alch(&mut w, 30, 90, false, false, &mut rng);
        assert_eq!(w.tile(30, 90).unwrap().kind, 83, "108 blooms on Next(2)==0");
        // Frame 108 + Next(2) == 1 (seed 102): the gate holds the sprout —
        // and the cue arms cannot rescue it (108 matches no cue), so the
        // tick ends with the kind unchanged.
        let mut w = world();
        put(&mut w, 30, 90, 82, 108);
        let mut rng = UnifiedRandom::new(102);
        grow_alch(&mut w, 30, 90, false, false, &mut rng);
        assert_eq!(w.tile(30, 90).unwrap().kind, 82, "108 holds on Next(2)==1");
        // The stream order: on frame 108 the extra Next(2) IS drawn (two
        // samples per armed tick); on frame 0 it is not (one sample). The
        // third raw draw differs between the two replays of seed 35.
        let mut r108 = UnifiedRandom::new(35);
        let _ = r108.next_max(50);
        let _ = r108.next_max(2);
        let after108 = r108.next();
        let mut r0 = UnifiedRandom::new(35);
        let _ = r0.next_max(50);
        let after0 = r0.next();
        assert_ne!(after108, after0, "the Next(2) rides only frame 108");
    }

    /// W4-d105 doc-sweep: the fireblossom maturity tail
    /// (`WorldGen.cs:45890-45898`) — a blooming fireblossom (83, frameX 108)
    /// matures to 84 at 1/30. Seed 35's first `Next(30)` is 0 (fires), seed
    /// 1's is non-zero (skips); both hand-derived .NET fixtures.
    #[test]
    fn w4_d105_the_fireblossom_tail_matures_at_one_in_thirty() {
        let mut w = world();
        put(&mut w, 30, 90, 83, 108);
        let mut rng = UnifiedRandom::new(35);
        grow_alch(&mut w, 30, 90, false, false, &mut rng);
        assert_eq!(w.tile(30, 90).unwrap().kind, 84, "the 1/30 tail fires");
        // The miss: seed 1's Next(30) != 0 — the bloom stays.
        let mut w = world();
        put(&mut w, 30, 90, 83, 108);
        let mut rng = UnifiedRandom::new(1);
        grow_alch(&mut w, 30, 90, false, false, &mut rng);
        assert_eq!(w.tile(30, 90).unwrap().kind, 83, "the 1/30 tail skips");
        // The kind gate: a frame-108 SPROUT (82) never reaches the tail —
        // the 82 arm returns first (its generic gate misses on seed 1).
        let mut w = world();
        put(&mut w, 30, 90, 82, 108);
        let mut rng = UnifiedRandom::new(1);
        grow_alch(&mut w, 30, 90, false, false, &mut rng);
        assert_eq!(w.tile(30, 90).unwrap().kind, 82, "the tail is 83-only");
        // The frame gate: a frame-36 bloom (blinkroot) takes the stage-3
        // flash arm instead — never the 108 tail. Seed 35's first Next(3)
        // is 0, so the flash arm's `Next(3) != 0` gate FAILS and the tick
        // ends with the kind unchanged (the 108 tail must not have run).
        let mut w = world();
        put(&mut w, 30, 90, 83, 36);
        let mut rng = UnifiedRandom::new(35);
        grow_alch(&mut w, 30, 90, false, false, &mut rng);
        assert_eq!(w.tile(30, 90).unwrap().kind, 83, "the flash arm owns frame 36");
    }

    #[test]
    fn the_alch_harvest_matrix_runs_the_style_conditions() {
        // `IsAlchemyPlantHarvestable` (`WorldGen.cs:66148-66175`): the
        // per-style gates, against a daytime world under the surface
        // (world_surface = 100 in the 4200×400 fixture).
        let mut w = world();
        let dry = WeatherCtx::default();
        let rainy = WeatherCtx {
            raining: true,
            ..WeatherCtx::default()
        };
        // style 0 (daybloom): day only.
        w.day_time = true;
        assert!(is_alchemy_plant_harvestable(&w, 0, 120, &dry));
        w.day_time = false;
        assert!(!is_alchemy_plant_harvestable(&w, 0, 120, &dry));
        // style 1 (moonglow): night only.
        assert!(is_alchemy_plant_harvestable(&w, 1, 120, &dry));
        // style 3 (deathweed): night + (blood moon or full moon) — the
        // fixture's moon_phase 0 is the full moon.
        assert!(is_alchemy_plant_harvestable(&w, 3, 120, &dry));
        w.moon_phase = 4;
        assert!(!is_alchemy_plant_harvestable(&w, 3, 120, &dry));
        w.blood_moon = true;
        assert!(is_alchemy_plant_harvestable(&w, 3, 120, &dry));
        w.blood_moon = false;
        // style 4 (waterleaf): rain or any cloud.
        assert!(!is_alchemy_plant_harvestable(&w, 4, 120, &dry));
        assert!(is_alchemy_plant_harvestable(&w, 4, 120, &rainy));
        let cloudy = WeatherCtx {
            cloud_alpha: 0.1,
            ..WeatherCtx::default()
        };
        assert!(is_alchemy_plant_harvestable(&w, 4, 120, &cloudy));
        // style 5 (fireblossom): late afternoon (the C#'s
        // `Main.time > 40500.0` is strict) + the rain/height arm — the
        // herb harvests when it is not raining, or when it sits BELOW the
        // surface (`y > worldSurface`, the rows the rain does not reach).
        w.time = 40501.0;
        assert!(is_alchemy_plant_harvestable(&w, 5, 90, &dry));
        assert!(is_alchemy_plant_harvestable(&w, 5, 120, &rainy));
        assert!(!is_alchemy_plant_harvestable(&w, 5, 90, &rainy));
        // An unknown style never harvests.
        assert!(!is_alchemy_plant_harvestable(&w, 2, 90, &dry));
    }

    #[test]
    fn the_seed_harvest_arm_covers_the_grown_herbs() {
        // `IsHarvestableHerbWithSeed` (`WorldGen.cs:66118-66138`): only
        // 83/84 — 84 always, 83 through its style, 82 never.
        let mut w = world();
        w.day_time = true;
        let dry = WeatherCtx::default();
        assert!(!is_harvestable_herb_with_seed(&w, 82, 0, 90, &dry));
        assert!(is_harvestable_herb_with_seed(&w, 84, 0, 90, &dry));
        assert!(is_harvestable_herb_with_seed(&w, 83, 0, 90, &dry));
        w.day_time = false;
        assert!(!is_harvestable_herb_with_seed(&w, 83, 0, 90, &dry));
        assert!(is_harvestable_herb_with_seed(&w, 83, 1, 90, &dry));
        // A non-herb kind never harvests.
        assert!(!is_harvestable_herb_with_seed(&w, 1, 0, 90, &dry));
    }
}
