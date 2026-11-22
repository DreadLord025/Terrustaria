//! The animated-tile animator — `Main.AnimateTiles` (`Main.cs:18516-19304`).
//!
//! C# advances the global per-type animation rows `Main.tileFrame[type]` with
//! the per-type tick counters `Main.tileFrameCounter[type]` once per client
//! update (`Main.cs:17150`, inside `DoUpdate`, after `DoUpdate_AnimateWalls`).
//! `GetTileDrawData` then keys its source rows on `Main.tileFrame[typeCache]`
//! (`TileDrawing.cs:4533`), which the port's [`crate::drawdata::draw_data`]
//! receives as its `tile_frame` argument. This module owns that state;
//! WIRED since S1-G-1f (2026-11-19): `App::animate_tiles_phase` ticks it on
//! the 60 Hz accumulator beside the waterfalls, `render/tiles.rs::draw_tiles`
//! feeds [`TileAnimator::animation_row`] / the counters / the bobframe into
//! the drawdata dispatch, and the G6 `WorldFrameKey::tile_anim` covers the
//! state via [`TileAnimator::anim_key`] gated on [`TileAnimator::is_animated_kind`].
//!
//! Determinism: the two `Main.rand` draws (tiles 453/456, `Main.cs:19109`,
//! `:19118`) and the weather-vane bob roll (`Main.cs:19325`) take the caller's
//! [`UnifiedRandom`] — the port's exact `Main.rand` Knuth subtractive stream —
//! in source order. No other RNG is touched; tile framing (`framing.rs`) is
//! untouched.
//!
//! Deliberate deviation: the trailing `Main.AnimateTiles_CritterCages()`
//! (`Main.cs:19303`, body `:13511-13700`) is NOT ported here — it animates the
//! separate `Main.bunnyCageFrame[]` cage-entity arrays (tile 620's deferred
//! special pass, research/09 deviation "critter cage MISSING"), not
//! `tileFrame[]`, and needs the `Main.critterCage` proximity gate. Recorded in
//! docs/ANALYSIS.md.

use crate::tiledata::TILE_COUNT;
use crate::unified_random::UnifiedRandom;

/// `Main.tileFrame` / `Main.tileFrameCounter` (`Main.cs:1596-1597`, allocated
/// `new int[TileID.Count]` at `Main.cs:66976-66977`) plus
/// `Main.weatherVaneBobframe` (`Main.cs:2237`). All zero-initialised — the
/// world-load reset is `Main.tileFrame[l] = 0; Main.tileFrameCounter[l] = 0`
/// (`Main.cs:8393-8394`).
#[derive(Clone, Debug)]
pub struct TileAnimator {
    /// `Main.tileFrame[type]` — the animation row `draw_data` consumes.
    pub tile_frame: [i32; TILE_COUNT],
    /// `Main.tileFrameCounter[type]` — the per-type tick counter.
    pub tile_frame_counter: [i32; TILE_COUNT],
    /// `Main.weatherVaneBobframe` (`Main.cs:2237`, reset `:67242`).
    pub weather_vane_bobframe: i32,
}

impl Default for TileAnimator {
    fn default() -> Self {
        Self::new()
    }
}

impl TileAnimator {
    pub fn new() -> Self {
        Self {
            tile_frame: [0; TILE_COUNT],
            tile_frame_counter: [0; TILE_COUNT],
            weather_vane_bobframe: 0,
        }
    }

    /// The world-load reset (`Main.cs:8393-8394`): every `tileFrame[l]` and
    /// `tileFrameCounter[l]` back to 0; `weatherVaneBobframe` resets with
    /// them (`Main.cs:67242`).
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// G6 key input (S1-G-1f wiring): a hash of every animator value that
    /// rides the world instance range's pixels — the animation rows
    /// (`draw_data`/`case_draw_data`'s `tile_frame`), the tick counters
    /// (slice B's `counter`, e.g. cases 453/456/489/493) and the weather-vane
    /// bob (`CaseInputs::weather_vane_bobframe`). Compared bit-exactly by
    /// `WorldFrameKey::tile_anim`; two frames with different animator state
    /// must not share a cached world range. FNV-1a over the raw `i32`
    /// little-endian bytes — deterministic, order-sensitive, no RNG.
    pub fn anim_key(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        let mut feed = |v: i32| {
            for b in v.to_le_bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
        };
        for &v in &self.tile_frame {
            feed(v);
        }
        for &v in &self.tile_frame_counter {
            feed(v);
        }
        feed(self.weather_vane_bobframe);
        h
    }

    /// Every tile kind [`Self::animate_tiles`] writes a frame or counter
    /// for (the `Main.AnimateTiles` body, `Main.cs:18516-19304`, including
    /// the mirrors 665/696/271/581/355 and the weather vane 490 whose bob
    /// rides `weather_vane_bobframe`). The G6 gate
    /// (`WorldFrameKey::tile_anim`) keys on the animator state only when
    /// one of these kinds is active in the view rect — animator-free
    /// worlds keep the pre-wiring cache behaviour bit-for-bit. The
    /// `every_touched_kind_is_in_the_gate_set` test proves the list covers
    /// every index the tick actually mutates.
    pub fn is_animated_kind(kind: u16) -> bool {
        matches!(
            kind,
            12 | 17
                | 31
                | 77
                | 96
                | 106
                | 129
                | 133
                | 171
                | 207
                | 215
                | 217
                | 218
                | 219
                | 220
                | 228
                | 231
                | 235
                | 238
                | 243
                | 244
                | 247
                | 270
                | 271
                | 272
                | 300
                | 301
                | 302
                | 303
                | 305
                | 306
                | 307
                | 308
                | 314
                | 326
                | 327
                | 328
                | 329
                | 336
                | 340
                | 341
                | 342
                | 343
                | 344
                | 345
                | 351
                | 354
                | 355
                | 377
                | 379
                | 390
                | 405
                | 406
                | 410
                | 412
                | 421
                | 422
                | 452
                | 453
                | 455
                | 456
                | 458
                | 459
                | 463
                | 464
                | 480
                | 485
                | 489
                | 490
                | 491
                | 493
                | 499
                | 507
                | 508
                | 509
                | 564
                | 565
                | 572
                | 581
                | 592
                | 593
                | 594
                | 597
                | 614
                | 639
                | 642
                | 657
                | 658
                | 660
                | 665
                | 696
                | 708
                | 720
                | 721
                | 725
                | 739
                | 748
        )
    }

    /// The `tile_frame` argument for [`crate::drawdata::draw_data`]:
    /// `Main.tileFrame[kind]` (`TileDrawing.cs:4533`).
    pub fn animation_row(&self, kind: u16) -> i32 {
        self.tile_frame[kind as usize]
    }

    // ---- the two C# counter idioms, kept textually distinct ----

    /// `tileFrameCounter[k]++; if (tileFrameCounter[k] > limit) { = 0;
    /// tileFrame[k]++; if (tileFrame[k] >= wrap) = 0; }`
    /// (e.g. tile 12, `Main.cs:18518-18527`).
    fn step_gt_ge(&mut self, k: usize, limit: i32, wrap: i32) {
        self.tile_frame_counter[k] += 1;
        if self.tile_frame_counter[k] > limit {
            self.tile_frame_counter[k] = 0;
            self.tile_frame[k] += 1;
            if self.tile_frame[k] >= wrap {
                self.tile_frame[k] = 0;
            }
        }
    }

    /// As [`Self::step_gt_ge`] but the frame wraps on `> wrap`
    /// (e.g. tile 247, `Main.cs:18739-18748`).
    fn step_gt_gt(&mut self, k: usize, limit: i32, wrap: i32) {
        self.tile_frame_counter[k] += 1;
        if self.tile_frame_counter[k] > limit {
            self.tile_frame_counter[k] = 0;
            self.tile_frame[k] += 1;
            if self.tile_frame[k] > wrap {
                self.tile_frame[k] = 0;
            }
        }
    }

    /// `tileFrameCounter[k]++; if (tileFrameCounter[k] >= limit) { = 0;
    /// tileFrame[k]++; if (tileFrame[k] >= wrap) = 0; }` — also covers the
    /// pre-increment spelling `if (++tileFrameCounter[k] >= limit) { ...
    /// if (++tileFrame[k] >= wrap) ... }` (identical semantics, e.g. tile 133,
    /// `Main.cs:18570-18577`).
    fn step_ge_ge(&mut self, k: usize, limit: i32, wrap: i32) {
        self.tile_frame_counter[k] += 1;
        if self.tile_frame_counter[k] >= limit {
            self.tile_frame_counter[k] = 0;
            self.tile_frame[k] += 1;
            if self.tile_frame[k] >= wrap {
                self.tile_frame[k] = 0;
            }
        }
    }

    /// As [`Self::step_ge_ge`] but the frame wraps on `> wrap`
    /// (e.g. tile 300, `Main.cs:18802-18811`).
    fn step_ge_gt(&mut self, k: usize, limit: i32, wrap: i32) {
        self.tile_frame_counter[k] += 1;
        if self.tile_frame_counter[k] >= limit {
            self.tile_frame_counter[k] = 0;
            self.tile_frame[k] += 1;
            if self.tile_frame[k] > wrap {
                self.tile_frame[k] = 0;
            }
        }
    }

    /// One client update — `Main.AnimateTiles()` (`Main.cs:18516-19304`), in
    /// source order. `wind_for_visuals` is `Main.WindForVisuals`, `rand` is
    /// `Main.rand`, `generating_or_loading` is
    /// `WorldGen.isGeneratingOrLoadingWorld` (the 453/456 draw gate,
    /// `Main.cs:19109`/`:19118`).
    pub fn animate_tiles(
        &mut self,
        wind_for_visuals: f32,
        rand: &mut UnifiedRandom,
        generating_or_loading: bool,
    ) {
        self.step_gt_ge(12, 5, 10); // Main.cs:18518-18527 — Hellforge
                                    // Main.cs:18528-18529 — AdamantiteForge mirrors both arrays.
        self.tile_frame_counter[665] = self.tile_frame_counter[12];
        self.tile_frame[665] = self.tile_frame[12];
        self.step_gt_ge(639, 5, 10); // Main.cs:18530-18539 — TitaniumForge
        self.step_gt_ge(739, 10, 4); // Main.cs:18540-18549
        self.step_gt_ge(748, 7, 8); // Main.cs:18550-18559
        self.step_gt_ge(17, 5, 12); // Main.cs:18560-18569 — Furnace
        self.step_ge_ge(133, 4, 6); // Main.cs:18570-18577 — pre-increment form
        self.step_ge_ge(31, 8, 4); // Main.cs:18578-18587 — Sunflower
        self.tile_frame[696] = self.tile_frame[31]; // Main.cs:18588
        self.step_gt_ge(77, 5, 12); // Main.cs:18589-18598 — HellstoneBrick forge glow
        self.step_gt_ge(106, 4, 2); // Main.cs:18599-18608 — Sawmill
        self.step_gt_ge(207, 4, 6); // Main.cs:18609-18618 — WaterFountain
        self.step_ge_ge(215, 4, 8); // Main.cs:18619-18628 — HogWild
        self.step_ge_ge(592, 5, 8); // Main.cs:18629-18638
        self.step_gt_ge(217, 4, 5); // Main.cs:18639-18648
        self.step_gt_ge(218, 4, 2); // Main.cs:18649-18658
        self.step_gt_ge(219, 4, 10); // Main.cs:18659-18668
        self.step_gt_ge(642, 4, 6); // Main.cs:18669-18678
        self.step_gt_ge(220, 4, 4); // Main.cs:18679-18688
        self.step_gt_ge(231, 16, 7); // Main.cs:18689-18698 — Larva
        self.step_gt_ge(235, 20, 4); // Main.cs:18699-18708 — Teleporter
        self.step_gt_ge(238, 20, 4); // Main.cs:18709-18718 — PlanteraBulb
        self.step_gt_ge(243, 4, 6); // Main.cs:18719-18728
        self.step_gt_ge(244, 4, 6); // Main.cs:18729-18738
        self.step_gt_gt(247, 4, 7); // Main.cs:18739-18748
        self.step_gt_gt(96, 4, 3); // Main.cs:18749-18758
        self.step_gt_gt(171, 16, 3); // Main.cs:18759-18768
        self.step_gt_gt(270, 8, 5); // Main.cs:18769-18778
                                    // Main.cs:18779-18781 — both mirror tile 270's frame.
        let num = self.tile_frame[270];
        self.tile_frame[271] = num;
        self.tile_frame[581] = num;
        self.step_gt_gt(660, 8, 4); // Main.cs:18782-18791
        self.step_ge_gt(272, 10, 1); // Main.cs:18792-18801
        self.step_ge_gt(300, 5, 6); // Main.cs:18802-18811
        self.step_ge_gt(301, 5, 7); // Main.cs:18812-18821
        self.step_ge_gt(302, 5, 3); // Main.cs:18822-18831
        self.step_ge_gt(303, 5, 4); // Main.cs:18832-18841
        self.step_ge_gt(305, 5, 11); // Main.cs:18842-18851
        self.step_ge_gt(306, 5, 11); // Main.cs:18852-18861
        self.step_ge_gt(307, 5, 1); // Main.cs:18862-18871
        self.step_ge_gt(308, 5, 7); // Main.cs:18872-18881
        self.step_ge_gt(314, 10, 4); // Main.cs:18882-18891
        self.step_ge_gt(326, 5, 7); // Main.cs:18892-18901
        self.step_ge_gt(327, 10, 7); // Main.cs:18902-18911
        self.step_ge_gt(345, 10, 7); // Main.cs:18912-18921
        self.step_ge_gt(458, 10, 7); // Main.cs:18922-18931
        self.step_ge_gt(459, 10, 7); // Main.cs:18932-18941
        self.step_ge_gt(708, 10, 7); // Main.cs:18942-18951
        self.step_ge_gt(336, 5, 3); // Main.cs:18952-18961
        self.step_ge_gt(328, 5, 7); // Main.cs:18962-18971
        self.step_ge_gt(329, 5, 7); // Main.cs:18972-18981
                                    // Main.cs:18982-18990 — counter-only, `num2 = 20`, wrap `num2 * 8`.
        let num2 = 20;
        self.tile_frame_counter[507] += 1;
        if self.tile_frame_counter[507] >= num2 * 8 {
            self.tile_frame_counter[507] = 0;
        }
        self.tile_frame_counter[508] += 1;
        if self.tile_frame_counter[508] >= num2 * 8 {
            self.tile_frame_counter[508] = 0;
        }
        // Main.cs:18991-19003 — the 340..=344 loop.
        for i in 340..=344 {
            self.step_ge_gt(i, 5, 3);
        }
        self.step_ge_gt(351, 5, 2); // Main.cs:19004-19013
        self.step_ge_ge(354, 5, 8); // Main.cs:19014-19023
        self.tile_frame[355] = self.tile_frame[354]; // Main.cs:19024
        self.step_ge_ge(377, 5, 4); // Main.cs:19025-19034
        self.step_ge_ge(379, 10, 4); // Main.cs:19035-19044
        self.step_ge_ge(390, 8, 7); // Main.cs:19045-19052
        self.step_ge_ge(228, 5, 3); // Main.cs:19053-19060
        self.step_ge_ge(405, 5, 8); // Main.cs:19061-19068
        self.step_ge_ge(406, 8, 6); // Main.cs:19069-19076
        self.step_ge_ge(452, 5, 15); // Main.cs:19077-19084
        self.step_ge_ge(455, 5, 6); // Main.cs:19085-19092
        self.step_ge_ge(499, 5, 8); // Main.cs:19093-19100
        self.step_ge_ge(129, 8, 6); // Main.cs:19101-19108
                                    // Main.cs:19109-19113 — tile 453: random counter drift, no frame.
        self.tile_frame_counter[453] += if !generating_or_loading {
            rand.next_max(3)
        } else {
            0
        };
        self.tile_frame_counter[453] += 1;
        if self.tile_frame_counter[453] >= 60 {
            self.tile_frame_counter[453] = 0;
        }
        // Main.cs:19114-19117 — tile 412: frame-only, every tick.
        self.tile_frame[412] += 1;
        if self.tile_frame[412] >= 240 {
            self.tile_frame[412] = 0;
        }
        // Main.cs:19118-19126 — tile 456: random counter drift AND a
        // frame step every tick.
        self.tile_frame_counter[456] += if !generating_or_loading {
            rand.next_max(3)
        } else {
            0
        };
        self.tile_frame_counter[456] += 1;
        if self.tile_frame_counter[456] >= 80 {
            self.tile_frame_counter[456] = 0;
        }
        self.tile_frame[456] += 1;
        if self.tile_frame[456] >= 240 {
            self.tile_frame[456] = 0;
        }
        self.step_ge_ge(410, 8, 8); // Main.cs:19127-19134
        self.step_ge_ge(480, 8, 8); // Main.cs:19135-19142
        self.step_ge_ge(509, 8, 8); // Main.cs:19143-19150
        self.step_ge_ge(657, 8, 8); // Main.cs:19151-19158
        self.step_ge_ge(658, 4, 10); // Main.cs:19159-19166
        self.step_ge_ge(720, 8, 16); // Main.cs:19167-19174
        self.step_ge_ge(721, 8, 32); // Main.cs:19175-19182
        self.step_ge_ge(725, 8, 4); // Main.cs:19183-19190
        self.step_ge_ge(421, 4, 4); // Main.cs:19191-19198
                                    // Main.cs:19199-19206 — tile 422 counts DOWN, wrapping to 3.
        self.tile_frame_counter[422] += 1;
        if self.tile_frame_counter[422] >= 4 {
            self.tile_frame_counter[422] = 0;
            self.tile_frame[422] -= 1;
            if self.tile_frame[422] < 0 {
                self.tile_frame[422] = 3;
            }
        }
        self.step_ge_ge(463, 10, 6); // Main.cs:19207-19214
        self.step_ge_ge(464, 5, 23); // Main.cs:19215-19222
                                     // Main.cs:19223-19230 — counter-only wraps (485: 20, 491: 40).
        self.tile_frame_counter[485] += 1;
        if self.tile_frame_counter[485] >= 20 {
            self.tile_frame_counter[485] = 0;
        }
        self.tile_frame_counter[491] += 1;
        if self.tile_frame_counter[491] >= 40 {
            self.tile_frame_counter[491] = 0;
        }
        self.step_ge_ge(564, 5, 36); // Main.cs:19231-19239
                                     // Main.cs:19240-19257 — tiles 593/594 wrap to row 5, not 0.
        for k in [593usize, 594] {
            self.tile_frame_counter[k] += 1;
            if self.tile_frame_counter[k] >= 5 {
                self.tile_frame_counter[k] = 0;
                self.tile_frame[k] += 1;
                if self.tile_frame[k] >= 9 {
                    self.tile_frame[k] = 5;
                }
            }
        }
        self.step_ge_ge(614, 5, 6); // Main.cs:19258-19266
        self.step_ge_ge(565, 4, 5); // Main.cs:19267-19275
        self.step_ge_ge(572, 6, 4); // Main.cs:19276-19283
                                    // Main.cs:19284-19287 — pylon counter-only wrap at 64.
        self.tile_frame_counter[597] += 1;
        if self.tile_frame_counter[597] >= 64 {
            self.tile_frame_counter[597] = 0;
        }
        // Main.cs:19288-19294 — tile 489: wind-driven counter, mod 320.
        let num3 = wind_push(wind_for_visuals, true);
        self.tile_frame_counter[489] += num3;
        self.tile_frame_counter[489] %= 320;
        if self.tile_frame_counter[489] < 0 {
            self.tile_frame_counter[489] += 320;
        }
        self.animate_weather_vane(wind_for_visuals, rand); // Main.cs:19295
                                                           // Main.cs:19296-19302 — tile 493: wind-driven counter, mod 120.
        let num4 = wind_push(wind_for_visuals, true);
        self.tile_frame_counter[493] += num4;
        self.tile_frame_counter[493] %= 120;
        if self.tile_frame_counter[493] < 0 {
            self.tile_frame_counter[493] += 120;
        }
        // Main.cs:19303 — AnimateTiles_CritterCages: deferred (module docs).
    }

    /// `Main.AnimateTiles_WeatherVane` (`Main.cs:19306-19329`).
    fn animate_weather_vane(&mut self, wind_for_visuals: f32, rand: &mut UnifiedRandom) {
        let num = sign(wind_for_visuals); // :19308 — Math.Sign
                                          // :19309 — clamp(floor(|wind| * 10), -5, 5), NO sign multiply here.
        let num2 = wind_push(wind_for_visuals, false);
        let num3 = 6; // :19310
        self.tile_frame_counter[490] += num2; // :19311
        if self.tile_frame_counter[490] < num3 {
            return; // :19312-19315
        }
        // :19316 — verbatim: `counter -= num3 * counter` (the decompiled form;
        // NOT a modulo — it flips the counter to -5x its value).
        self.tile_frame_counter[490] -= num3 * self.tile_frame_counter[490];
        // :19317-19324
        if (self.tile_frame[490] != 0 || num != -1) && (self.tile_frame[490] != 6 || num != 1) {
            self.tile_frame[490] += 1;
            if self.tile_frame[490] >= 12 {
                self.tile_frame[490] = 0;
                self.weather_vane_bobframe = 0;
            }
        } else if rand.next_float() < wind_for_visuals.abs() * 0.5 {
            // :19325-19328 — the bob roll; `++bobframe == 8` only on a pass.
            self.weather_vane_bobframe += 1;
            if self.weather_vane_bobframe == 8 {
                self.weather_vane_bobframe = 0;
            }
        }
    }
}

/// `Math.Sign(Main.WindForVisuals)` (`Main.cs:19308`).
fn sign(w: f32) -> i32 {
    if w > 0.0 {
        1
    } else if w < 0.0 {
        -1
    } else {
        0
    }
}

/// `(int)MathHelper.Clamp((float)Math.Floor(Math.Abs(w) * 10f)
/// [* (float)Math.Sign(w)], -5f, 5f)` — `Main.cs:19288`/`:19296` with the sign
/// multiply, `:19309` without. `Math.Floor` runs on the double-promoted
/// product, then the result is cast back to float before the clamp.
fn wind_push(w: f32, with_sign: bool) -> i32 {
    let mut v = ((w.abs() * 10.0f32) as f64).floor() as f32;
    if with_sign {
        v *= sign(w) as f32;
    }
    v.clamp(-5.0, 5.0) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick_n(a: &mut TileAnimator, n: usize, wind: f32, rand: &mut UnifiedRandom) {
        for _ in 0..n {
            a.animate_tiles(wind, rand, false);
        }
    }

    /// S1-G-1f key-coverage witness: every array index `animate_tiles`
    /// mutates must be in [`TileAnimator::is_animated_kind`]'s gate set —
    /// otherwise a view holding that kind would keep a stale cached world
    /// range while its pixels animate. Runs the full tick under several
    /// winds (the weather-vane bob arms differ) and diffs both arrays plus
    /// the bobframe against a fresh state each tick.
    #[test]
    fn every_touched_kind_is_in_the_gate_set() {
        for wind in [0.0f32, 0.3, -0.7, 1.2] {
            let mut a = TileAnimator::new();
            let mut r = UnifiedRandom::new(9);
            for _ in 0..700 {
                let before = a.clone();
                a.animate_tiles(wind, &mut r, false);
                for k in 0..TILE_COUNT {
                    if a.tile_frame[k] != before.tile_frame[k]
                        || a.tile_frame_counter[k] != before.tile_frame_counter[k]
                    {
                        assert!(
                            TileAnimator::is_animated_kind(k as u16),
                            "kind {k} animates (wind {wind}) but is outside the G6 gate set"
                        );
                    }
                }
                if a.weather_vane_bobframe != before.weather_vane_bobframe {
                    assert!(
                        TileAnimator::is_animated_kind(490),
                        "the weather vane (490) bobs but is outside the G6 gate set"
                    );
                }
            }
        }
    }

    /// S1-G-1f: `anim_key` is deterministic, changes with state, and the
    /// world-load reset returns the fresh key.
    #[test]
    fn anim_key_tracks_state_and_reset() {
        let mut a = TileAnimator::new();
        let fresh = a.anim_key();
        assert_eq!(fresh, TileAnimator::new().anim_key());
        let mut r = UnifiedRandom::new(3);
        tick_n(&mut a, 1, 0.0, &mut r);
        assert_ne!(fresh, a.anim_key());
        let after_one = a.anim_key();
        tick_n(&mut a, 1, 0.0, &mut r);
        assert_ne!(after_one, a.anim_key());
        a.reset();
        assert_eq!(fresh, a.anim_key(), "reset restores the fresh key");
        assert_eq!(a.animation_row(12), 0);
    }

    /// Fresh state is the world-load reset: all rows/counters zero
    /// (`Main.cs:8393-8394`, `:67242`).
    #[test]
    fn fresh_animator_is_the_vanilla_zero_state() {
        let a = TileAnimator::new();
        assert!(a.tile_frame.iter().all(|&f| f == 0));
        assert!(a.tile_frame_counter.iter().all(|&c| c == 0));
        assert_eq!(a.weather_vane_bobframe, 0);
        assert_eq!(a.animation_row(12), 0);
    }

    /// Tile 12 (`Main.cs:18518-18527`): counter > 5 → period 6 ticks,
    /// frame >= 10 → 10 rows; tile 665 mirrors BOTH arrays (`:18528-18529`).
    #[test]
    fn hellforge_period_and_adamantite_mirror() {
        let mut a = TileAnimator::new();
        let mut r = UnifiedRandom::new(1);
        tick_n(&mut a, 5, 0.0, &mut r);
        assert_eq!(a.tile_frame[12], 0);
        tick_n(&mut a, 1, 0.0, &mut r); // tick 6
        assert_eq!(a.tile_frame[12], 1);
        assert_eq!(a.tile_frame_counter[12], 0);
        tick_n(&mut a, 53, 0.0, &mut r); // tick 59
        assert_eq!(a.tile_frame[12], 9);
        tick_n(&mut a, 1, 0.0, &mut r); // tick 60 — full cycle
        assert_eq!(a.tile_frame[12], 0);
        for _ in 0..137 {
            a.animate_tiles(0.0, &mut r, false);
            assert_eq!(a.tile_frame[665], a.tile_frame[12]);
            assert_eq!(a.tile_frame_counter[665], a.tile_frame_counter[12]);
        }
    }

    /// Tile 133 pre-increment form (`Main.cs:18570-18577`): period 4, 6 rows.
    #[test]
    fn pre_increment_idiom_tile_133() {
        let mut a = TileAnimator::new();
        let mut r = UnifiedRandom::new(1);
        tick_n(&mut a, 3, 0.0, &mut r);
        assert_eq!(a.tile_frame[133], 0);
        tick_n(&mut a, 1, 0.0, &mut r);
        assert_eq!(a.tile_frame[133], 1);
        tick_n(&mut a, 20, 0.0, &mut r); // tick 24 = 4 * 6
        assert_eq!(a.tile_frame[133], 0);
    }

    /// Tile 31 (`Main.cs:18578-18588`): post-increment `>= 8`, 4 rows;
    /// tile 696 mirrors the frame only — its counter stays 0.
    #[test]
    fn sunflower_period_and_696_frame_only_mirror() {
        let mut a = TileAnimator::new();
        let mut r = UnifiedRandom::new(1);
        tick_n(&mut a, 8, 0.0, &mut r);
        assert_eq!(a.tile_frame[31], 1);
        assert_eq!(a.tile_frame[696], 1);
        assert_eq!(a.tile_frame_counter[696], 0);
        tick_n(&mut a, 24, 0.0, &mut r); // tick 32 = 8 * 4
        assert_eq!(a.tile_frame[31], 0);
        assert_eq!(a.tile_frame[696], 0);
    }

    /// W4-g84: tile 238 PlanteraBulb (`Main.cs:18709-18718`): the
    /// post-increment `> 20` counter → period 21 ticks, `>= 4` wrap → 4
    /// rows (the bulb's pulse animation, one row per 21 ticks, 84-tick
    /// cycle). The row feeds case 238's `addFrY = tileFrame * 36`
    /// (`TileDrawing.cs:5441-5444`).
    #[test]
    fn w4_g84_plantera_bulb_period_and_wrap() {
        let mut a = TileAnimator::new();
        let mut r = UnifiedRandom::new(1);
        tick_n(&mut a, 20, 0.0, &mut r);
        assert_eq!(a.tile_frame[238], 0);
        assert_eq!(a.tile_frame_counter[238], 20);
        tick_n(&mut a, 1, 0.0, &mut r); // tick 21 — first step
        assert_eq!(a.tile_frame[238], 1);
        assert_eq!(a.tile_frame_counter[238], 0);
        tick_n(&mut a, 62, 0.0, &mut r); // tick 83
        assert_eq!(a.tile_frame[238], 3);
        tick_n(&mut a, 1, 0.0, &mut r); // tick 84 — full cycle
        assert_eq!(a.tile_frame[238], 0);
        assert!(TileAnimator::is_animated_kind(238));
    }

    /// Tile 247 (`Main.cs:18739-18748`): the `frame > 7` wrap variant.
    #[test]
    fn frame_greater_than_wrap_tile_247() {
        let mut a = TileAnimator::new();
        let mut r = UnifiedRandom::new(1);
        tick_n(&mut a, 35, 0.0, &mut r); // 7 steps of 5 ticks
        assert_eq!(a.tile_frame[247], 7);
        tick_n(&mut a, 5, 0.0, &mut r);
        assert_eq!(a.tile_frame[247], 0);
    }

    /// Tiles 270/271/581 (`Main.cs:18769-18781`) and 354/355 (`:19014-19024`).
    #[test]
    fn frame_mirrors_271_581_and_355() {
        let mut a = TileAnimator::new();
        let mut r = UnifiedRandom::new(1);
        for _ in 0..200 {
            a.animate_tiles(0.0, &mut r, false);
            assert_eq!(a.tile_frame[271], a.tile_frame[270]);
            assert_eq!(a.tile_frame[581], a.tile_frame[270]);
            assert_eq!(a.tile_frame[355], a.tile_frame[354]);
        }
        assert_ne!(a.tile_frame[270], 0); // the source actually moved
    }

    /// The 340..=344 loop (`Main.cs:18991-19003`): five identical animators.
    #[test]
    fn the_340_to_344_loop_advances_in_lockstep() {
        let mut a = TileAnimator::new();
        let mut r = UnifiedRandom::new(1);
        for _ in 0..60 {
            a.animate_tiles(0.0, &mut r, false);
            for i in 341..=344 {
                assert_eq!(a.tile_frame[i], a.tile_frame[340]);
                assert_eq!(a.tile_frame_counter[i], a.tile_frame_counter[340]);
            }
        }
        // period 5 ticks, wrap > 3 → 4 rows.
        assert_eq!(a.tile_frame[340], 0); // 60 = 5 * 4 * 3
    }

    /// Tile 422 counts DOWN (`Main.cs:19199-19206`): 0 → 3 → 2 → 1 → 0.
    #[test]
    fn tile_422_counts_down() {
        let mut a = TileAnimator::new();
        let mut r = UnifiedRandom::new(1);
        tick_n(&mut a, 4, 0.0, &mut r);
        assert_eq!(a.tile_frame[422], 3);
        tick_n(&mut a, 4, 0.0, &mut r);
        assert_eq!(a.tile_frame[422], 2);
        tick_n(&mut a, 8, 0.0, &mut r);
        assert_eq!(a.tile_frame[422], 0);
        tick_n(&mut a, 4, 0.0, &mut r);
        assert_eq!(a.tile_frame[422], 3);
    }

    /// Tiles 593/594 wrap to row 5 (`Main.cs:19240-19257`): 0..8 then 5..8.
    #[test]
    fn tiles_593_594_wrap_to_five() {
        let mut a = TileAnimator::new();
        let mut r = UnifiedRandom::new(1);
        tick_n(&mut a, 40, 0.0, &mut r); // 8 steps of 5
        assert_eq!(a.tile_frame[593], 8);
        tick_n(&mut a, 5, 0.0, &mut r);
        assert_eq!(a.tile_frame[593], 5);
        assert_eq!(a.tile_frame[594], 5);
        tick_n(&mut a, 20, 0.0, &mut r); // 5,6,7,8 → 5
        assert_eq!(a.tile_frame[593], 5);
    }

    /// Tile 412 is frame-only (`Main.cs:19114-19117`); 507/508/485/491/597
    /// are counter-only (`:18982-18990`, `:19223-19230`, `:19284-19287`).
    #[test]
    fn frame_only_412_and_counter_only_wraps() {
        let mut a = TileAnimator::new();
        let mut r = UnifiedRandom::new(1);
        tick_n(&mut a, 239, 0.0, &mut r);
        assert_eq!(a.tile_frame[412], 239);
        assert_eq!(a.tile_frame_counter[412], 0);
        tick_n(&mut a, 1, 0.0, &mut r);
        assert_eq!(a.tile_frame[412], 0);
        // counter-only types never move their frame.
        for k in [507, 508, 485, 491, 597] {
            assert_eq!(a.tile_frame[k], 0);
        }
        assert_eq!(a.tile_frame_counter[485], 240 % 20);
        assert_eq!(a.tile_frame_counter[491], 240 % 40);
        assert_eq!(a.tile_frame_counter[597], 240 % 64);
        assert_eq!(a.tile_frame_counter[507], 240 % 160);
    }

    /// Tiles 453/456 (`Main.cs:19109-19126`): the counter replay matches a
    /// cloned `Main.rand` stream draw-for-draw; 456's frame steps every tick.
    #[test]
    fn random_counter_drift_replays_the_rand_stream() {
        let mut a = TileAnimator::new();
        let mut r = UnifiedRandom::new(0x5EED);
        let mut probe = r.clone();
        let (mut c453, mut c456) = (0i32, 0i32);
        for t in 1..=500 {
            a.animate_tiles(0.0, &mut r, false);
            c453 += probe.next_max(3);
            c453 += 1;
            if c453 >= 60 {
                c453 = 0;
            }
            c456 += probe.next_max(3);
            c456 += 1;
            if c456 >= 80 {
                c456 = 0;
            }
            assert_eq!(a.tile_frame_counter[453], c453, "tick {t}");
            assert_eq!(a.tile_frame_counter[456], c456, "tick {t}");
            assert_eq!(a.tile_frame[453], 0);
            assert_eq!(a.tile_frame[456], t % 240);
        }
        // the animator consumed exactly the probe's draws
        assert_eq!(r.next(), probe.next());
    }

    /// The `WorldGen.isGeneratingOrLoadingWorld` gate (`Main.cs:19109`,
    /// `:19118`): no draws while generating — deterministic +1 counters.
    #[test]
    fn generating_gate_consumes_no_rand() {
        let mut a = TileAnimator::new();
        let mut r = UnifiedRandom::new(7);
        let probe = r.clone();
        for _ in 0..100 {
            a.animate_tiles(0.0, &mut r, true);
        }
        assert_eq!(a.tile_frame_counter[453], 100 % 60);
        assert_eq!(a.tile_frame_counter[456], 100 % 80);
        assert_eq!(r.next(), probe.clone().next()); // stream untouched
    }

    /// Tiles 489/493 (`Main.cs:19288-19302`): wind push ±5 max, signed mod.
    #[test]
    fn wind_counters_489_493_signed_mod() {
        let mut a = TileAnimator::new();
        let mut r = UnifiedRandom::new(1);
        a.animate_tiles(0.5, &mut r, false); // floor(5) * +1 = 5
        assert_eq!(a.tile_frame_counter[489], 5);
        assert_eq!(a.tile_frame_counter[493], 5);
        a.animate_tiles(-0.5, &mut r, false); // -5 → 0
        assert_eq!(a.tile_frame_counter[489], 0);
        a.animate_tiles(-0.5, &mut r, false); // -5 → 315 / 115
        assert_eq!(a.tile_frame_counter[489], 315);
        assert_eq!(a.tile_frame_counter[493], 115);
        // clamp at ±5 even in a gale
        a.tile_frame_counter[489] = 0;
        a.animate_tiles(9.0, &mut r, false);
        assert_eq!(a.tile_frame_counter[489], 5);
        // zero wind: counters hold (weather vane returns early)
        let before = a.tile_frame_counter[489];
        a.animate_tiles(0.0, &mut r, false);
        assert_eq!(a.tile_frame_counter[489], before);
    }

    /// Weather vane (`Main.cs:19306-19329`): the verbatim `-= 6 * counter`
    /// arithmetic, the 12-row wrap and the bob reset.
    #[test]
    fn weather_vane_counter_arithmetic_and_wrap() {
        let mut a = TileAnimator::new();
        let mut r = UnifiedRandom::new(1);
        a.animate_tiles(1.0, &mut r, false); // counter 5 < 6 → early return
        assert_eq!(a.tile_frame_counter[490], 5);
        assert_eq!(a.tile_frame[490], 0);
        a.animate_tiles(1.0, &mut r, false); // 10 → 10 - 60 = -50, frame 1
        assert_eq!(a.tile_frame_counter[490], -50);
        assert_eq!(a.tile_frame[490], 1);
        // frame 11 wraps to 0 and clears the bob frame (:19319-19323)
        a.tile_frame[490] = 11;
        a.tile_frame_counter[490] = 6;
        a.weather_vane_bobframe = 3;
        a.animate_tiles(1.0, &mut r, false);
        assert_eq!(a.tile_frame[490], 0);
        assert_eq!(a.weather_vane_bobframe, 0);
    }

    /// Weather vane bob branch (`Main.cs:19317`, `:19325-19328`): at frame 0
    /// with wind -1 the roll runs; the rand stream is consumed exactly when
    /// the branch is taken, and `++bob == 8` resets.
    #[test]
    fn weather_vane_bob_roll_matches_the_rand_stream() {
        let mut a = TileAnimator::new();
        let mut r = UnifiedRandom::new(0xC0FFEE);
        let mut probe = r.clone();
        a.tile_frame_counter[490] = 6; // pass the < 6 gate on the next tick
        let mut bob = 0i32;
        for _ in 0..400 {
            a.tile_frame_counter[490] = 6; // keep the gate open
            a.tile_frame[490] = 0; // keep the bob branch selected (num = -1)
                                   // generating=true gates the 453/456 draws so the ONLY rand
                                   // consumption is the bob roll (Main.cs:19109/:19118/:19325).
            a.animate_tiles(-1.0, &mut r, true);
            // C#: rand.NextFloat() < |−1| * 0.5 → ++bob; == 8 → 0
            if probe.next_float() < 0.5 {
                bob += 1;
                if bob == 8 {
                    bob = 0;
                }
            }
            assert_eq!(a.weather_vane_bobframe, bob);
        }
        assert_eq!(r.next(), probe.next());
    }

    /// Determinism: identical seeds + wind history replay byte-identically;
    /// the whole table stays inside its documented row bands over a long run.
    #[test]
    fn long_run_is_deterministic_and_bounded() {
        let mut a = TileAnimator::new();
        let mut b = TileAnimator::new();
        let mut ra = UnifiedRandom::new(42);
        let mut rb = UnifiedRandom::new(42);
        for t in 0..5000 {
            let wind = ((t as f32) * 0.01).sin() * 2.0;
            a.animate_tiles(wind, &mut ra, false);
            b.animate_tiles(wind, &mut rb, false);
        }
        assert_eq!(a.tile_frame, b.tile_frame);
        assert_eq!(a.tile_frame_counter, b.tile_frame_counter);
        assert_eq!(a.weather_vane_bobframe, b.weather_vane_bobframe);
        // spot bands straight from the source wraps
        assert!((0..10).contains(&a.tile_frame[12]));
        assert!((0..4).contains(&a.tile_frame[422]));
        assert!((0..12).contains(&a.tile_frame[490]));
        assert!((0..240).contains(&a.tile_frame[412]));
        assert!((0..320).contains(&a.tile_frame_counter[489]));
        assert!((0..120).contains(&a.tile_frame_counter[493]));
        assert!((0..8).contains(&a.weather_vane_bobframe));
    }
}
