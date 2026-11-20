//! The cavern background STYLE machine (`Main.DrawBackground`'s pick + sheet
//! tables + cross-fade), ported verbatim but UNWIRED (W4-g72 lane 3, code-only).
//!
//! Why unwired: the live `render::bg::draw_background` pins
//! `undergroundBackground = 0` (the classic dirt/rock set) because the per-biome
//! sheet variety needs ~90 extra `Background_<id>` sheets and the single 8192 px
//! atlas already packs the full static+gore set to 7530 px — even ONE extra
//! biome set overflows the strict-GPU bound (measured with the
//! `tools/atlas_sim.js` skyline twin). Wiring needs the C#'s separate
//! background-target family (`Main.backgroundTarget`, S1-G-13) or an atlas
//! re-budget; both are out of this lane's scope. The STYLE inputs
//! (`caveBackX`/`caveBackStyle`/`iceBackStyle`/`jungleBackStyle`) are also
//! unowned: `WorldGen.RandomizeCaveBackgrounds` (`WorldGen.cs:8192-8249`) draws
//! them mid-tail and inserting those draws would shift the seed-finish stream
//! and move the frozen `world_hash` pins — a worldgen lane's call, not this one.
//!
//! Everything here is pure (no RNG draws anywhere in the three C# functions)
//! and witnessed against the source line-by-line.

/// The style-pick inputs — the `Main` statics + `SceneMetrics` fields
/// `DrawBackground_PickUndergroundBackgroundStyle` reads (`Main.cs:54033-54095`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct BgStylePickInput {
    /// `Main.screenPosition` (unscaled screen-space top-left).
    pub screen_pos: [f32; 2],
    /// `Main.screenWidth` / `Main.screenHeight`.
    pub screen: [f32; 2],
    /// `Main.caveBackX[0..3]` (`WorldGen.cs:8197-8228` rolls).
    pub cave_back_x: [i32; 4],
    /// `Main.caveBackStyle[0..4]` (each 0..8, `WorldGen.cs:8194`).
    pub cave_back_style: [i32; 4],
    /// `Main.rockLayer` (the f64 static).
    pub rock_layer: f64,
    /// `Main.maxTilesY`.
    pub max_tiles_y: i32,
    /// The `magmaLayer` local `DrawBackground` computes (`Main.cs:52814-52815`).
    pub magma_layer: f64,
    /// `Main.SceneMetrics.SnowTileCount` / `JungleTileCount`.
    pub snow_tile_count: i32,
    pub jungle_tile_count: i32,
    /// `SceneMetrics.SnowTileThreshold` (`SceneMetrics.cs:154-163`: 1500 normal,
    /// 300 Skyblock — the caller picks; the port has no Skyblock scan yet).
    pub snow_tile_threshold: i32,
    /// `SceneMetrics.JungleTileThreshold` (`SceneMetrics.cs:928`: 140).
    pub jungle_tile_threshold: i32,
    /// `Main.SceneMetrics.ZoneBeach` (`SceneMetrics.cs:691`).
    pub zone_beach: bool,
    pub zone_corrupt: bool,
    pub zone_crimson: bool,
    pub zone_hallow: bool,
    pub zone_snow: bool,
    /// `Main.SceneMetrics.ZoneGlowshroom` (`SceneMetrics.cs:677`).
    pub zone_glowshroom: bool,
}

/// `Main.DrawBackground_PickUndergroundBackgroundStyle` (`Main.cs:54033-54095`),
/// verbatim: the cave-band pick, the snow/jungle count overrides, the beach and
/// deep-cavern evil arms, and the glowshroom trump. No RNG.
pub(crate) fn pick_underground_background_style(i: &BgStylePickInput) -> i32 {
    // Main.cs:54035: the probe column is the screen CENTRE — `screenWidth / 2`
    // is INTEGER division (both ints) before the float cast, then cast-after-
    // divide on the sum.
    let num = ((i.screen_pos[0] + (i.screen[0] as i32 / 2) as f32) / 16.0) as i32;
    // Main.cs:54036-54037: `num2 = 3` is a dead store; the ternary chain picks
    // the band style and adds 3 (the classic cave sets are styles 3..10).
    let mut num2 = if num <= i.cave_back_x[0] {
        i.cave_back_style[0]
    } else if num <= i.cave_back_x[1] {
        i.cave_back_style[1]
    } else if num > i.cave_back_x[2] {
        i.cave_back_style[3]
    } else {
        i.cave_back_style[2]
    } + 3;
    // Main.cs:54038-54041: the snow count override — STRICT `>` on the count and
    // STRICT `<` on the depth gate, +1200 px lookahead, `maxTilesY - 250`.
    if i.snow_tile_count > i.snow_tile_threshold
        && (i.screen_pos[1] + i.screen[1] + 1200.0) / 16.0 < (i.max_tiles_y - 250) as f32
    {
        num2 = 1;
    }
    // Main.cs:54042-54055: jungle beats snow only when it OUT-COUNTS it.
    if i.jungle_tile_count > i.jungle_tile_threshold {
        if num2 == 1 {
            if i.jungle_tile_count > i.snow_tile_count {
                num2 = 11;
            }
        } else {
            num2 = 11;
        }
    }
    // Main.cs:54056-54089: the beach arm trumps the depth arm (else-if).
    if i.zone_beach {
        num2 = if i.zone_corrupt {
            19
        } else if i.zone_crimson {
            21
        } else if !i.zone_hallow {
            18
        } else {
            20
        };
    } else if (i.screen_pos[1] / 16.0) as f64 > i.rock_layer + 60.0
        && (i.screen_pos[1] / 16.0) as f64 < i.magma_layer - 60.0
    {
        // Main.cs:54060: `(double)(Main.screenPosition.Y / 16f)` — the divide
        // runs in f32 FIRST, then widens to f64 for both compares.
        if i.zone_snow {
            if i.zone_corrupt {
                num2 = 15;
            } else if i.zone_crimson {
                num2 = 16;
            } else if i.zone_hallow {
                num2 = 17;
            }
        } else if i.zone_corrupt {
            num2 = 12;
        } else if i.zone_crimson {
            num2 = 13;
        } else if i.zone_hallow {
            num2 = 14;
        }
    }
    // Main.cs:54090-54093: glowshroom trumps everything.
    if i.zone_glowshroom {
        num2 = 2;
    }
    num2
}

/// One style's `_drawBackground_tempBack[0..7]` sheet table —
/// `Main.DrawBackground_UpdateBackgroundStyles`'s switch body
/// (`Main.cs:53812-54008`), verbatim. The C# runs the whole switch twice
/// (`undergroundBackground` then `oldUndergroundBackground`, `:53804-53810`);
/// the port exposes one pure call per style. Slots the switch never writes
/// stay 0 (`Array.Clear`, `:53811`); `LoadBackground(0)` (`Main.cs:59039-59045`)
/// still runs for them — `Background_0` is the sky gradient sheet, so the
/// width/height statics get its dims, but no draw arm consumes slot 0's id.
pub(crate) fn background_style_sheets(
    style: i32,
    ice_back_style: i32,
    jungle_back_style: i32,
    hell_back_style: i32,
    world_id: i32,
) -> [i32; 7] {
    let mut back = [0i32; 7]; // Main.cs:53811: Array.Clear(tempBack)
    match style {
        0 => {
            // Main.cs:53814-53821: the classic set.
            back[0] = 1;
            back[1] = 2;
            back[2] = 4;
            back[3] = 3;
            back[4] = 6;
            back[5] = 5; // dead store: the hellBackStyle tail overrides it
        }
        1 => {
            // Main.cs:53822-53852: the ice cavern, four iceBackStyle variants.
            // NOTE the write ORDER: [1], [3], [0], [2] — kept verbatim.
            match ice_back_style {
                0 => {
                    back[1] = 33;
                    back[3] = 32;
                    back[0] = 40;
                    back[2] = 34;
                }
                1 => {
                    back[1] = 118;
                    back[3] = 117;
                    back[0] = 160;
                    back[2] = 161;
                }
                2 => {
                    back[1] = 165;
                    back[3] = 167;
                    back[0] = 164;
                    back[2] = 166;
                }
                _ => {
                    back[1] = 120;
                    back[3] = 119;
                    back[0] = 162;
                    back[2] = 163;
                }
            }
            back[4] = 128 + hell_back_style;
        }
        2 => {
            // Main.cs:53853-53859: glowshroom.
            back[0] = 62;
            back[1] = 63;
            back[2] = 64;
            back[3] = 65;
            back[4] = 143 + hell_back_style;
        }
        3 => {
            // Main.cs:53860-53866.
            back[0] = 66;
            back[1] = 67;
            back[2] = 68;
            back[3] = 69;
            back[4] = 128 + hell_back_style;
        }
        4 => {
            // Main.cs:53867-53873: shares the 68 band with style 3.
            back[0] = 70;
            back[1] = 71;
            back[2] = 68;
            back[3] = 72;
            back[4] = 128 + hell_back_style;
        }
        5 => {
            // Main.cs:53874-53880.
            back[0] = 73;
            back[1] = 74;
            back[2] = 75;
            back[3] = 76;
            back[4] = 131 + hell_back_style;
        }
        6 => {
            // Main.cs:53881-53887.
            back[0] = 77;
            back[1] = 78;
            back[2] = 79;
            back[3] = 80;
            back[4] = 134 + hell_back_style;
        }
        7 => {
            // Main.cs:53888-53894: style 6 with the 81/82 rock pair.
            back[0] = 77;
            back[1] = 81;
            back[2] = 79;
            back[3] = 82;
            back[4] = 134 + hell_back_style;
        }
        8 => {
            // Main.cs:53895-53901.
            back[0] = 83;
            back[1] = 84;
            back[2] = 85;
            back[3] = 86;
            back[4] = 137 + hell_back_style;
        }
        9 => {
            // Main.cs:53902-53908.
            back[0] = 83;
            back[1] = 87;
            back[2] = 88;
            back[3] = 89;
            back[4] = 137 + hell_back_style;
        }
        10 => {
            // Main.cs:53909-53915.
            back[0] = 121;
            back[1] = 122;
            back[2] = 123;
            back[3] = 124;
            back[4] = 140 + hell_back_style;
        }
        11 => {
            // Main.cs:53916-53933: jungle, two jungleBackStyle variants.
            if jungle_back_style == 0 {
                back[0] = 153;
                back[1] = 147;
                back[2] = 148;
                back[3] = 149;
                back[4] = 150 + hell_back_style;
            } else {
                back[0] = 146;
                back[1] = 154;
                back[2] = 155;
                back[3] = 156;
                back[4] = 157 + hell_back_style;
            }
        }
        12 | 13 | 14 => {
            // Main.cs:53934-53953: the deep evil caverns — style 3's sheets with
            // a worldID-rolled evil rock row. [3] is written AFTER [4] here.
            back[0] = 66;
            back[1] = 67;
            back[2] = 68;
            back[4] = 128 + hell_back_style;
            back[3] = match style {
                12 => 193 + world_id % 4, // Main.cs:53944
                13 => 188 + world_id % 5, // Main.cs:53947
                _ => 197 + world_id % 3,  // Main.cs:53950 (style 14)
            };
        }
        15..=17 => {
            // Main.cs:53955-53974: the deep snow evil caverns — ice style 0's
            // sheets with an evil ice rock row.
            back[0] = 40;
            back[1] = 33;
            back[2] = 34;
            back[4] = 128 + hell_back_style;
            back[3] = match style {
                15 => 200,                    // Main.cs:53964
                16 => 201 + world_id % 2,     // Main.cs:53967
                _ => 203 + world_id % 4,      // Main.cs:53970 (style 17)
            };
        }
        18 => {
            // Main.cs:53977-53980: the ocean caverns — ONLY [0]/[1]; the rock
            // slots stay 0 (no rock/magma passes draw for these styles).
            back[0] = 290;
            back[1] = 291;
        }
        19 => {
            // Main.cs:53981-53984.
            back[0] = 292;
            back[1] = 293;
        }
        20 => {
            // Main.cs:53985-53988.
            back[0] = 294;
            back[1] = 295;
        }
        21 => {
            // Main.cs:53989-53992.
            back[0] = 296;
            back[1] = 297;
        }
        _ => {} // Main.cs:53954/53975: unmatched styles keep the cleared table
    }
    // Main.cs:53996-54007: the three sequential ifs (NOT else-if) — a
    // hellBackStyle outside 0..2 leaves the switch's own [5] store standing.
    if hell_back_style == 0 {
        back[5] = 125;
    }
    if hell_back_style == 1 {
        back[5] = 126;
    }
    if hell_back_style == 2 {
        back[5] = 127;
    }
    back[6] = 185 + hell_back_style; // Main.cs:54008
    back
}

/// The `DrawBackground` head's style latch + cross-fade decay
/// (`Main.cs:52823-52841`), verbatim. `shimmer_alpha == 1` short-circuits the
/// whole pass with `ugBackTransition = 0` (`:52800-52805`) — the caller owns
/// that early return; this step runs only when the pass proceeds.
pub(crate) struct UgBackState {
    /// `Main.undergroundBackground`.
    pub style: i32,
    /// `Main.oldUndergroundBackground`.
    pub old_style: i32,
    /// `Main.ugBackTransition` (1 → 0 in 0.25 steps).
    pub transition: f32,
}

/// One `DrawBackground` head step. Returns the (possibly new) state; no RNG.
pub(crate) fn ug_back_step(
    state: &mut UgBackState,
    picked: i32,
    shimmer_alpha: f32,
) {
    // Main.cs:52823-52829: a style change latches the old style and re-arms 1.
    if picked != state.style {
        state.old_style = state.style;
        state.style = picked;
        state.transition = 1.0;
    }
    // Main.cs:52830-52833: ANY shimmer kills the cross-fade outright.
    if shimmer_alpha > 0.0 {
        state.transition = 0.0;
    }
    // Main.cs:52834-52841: decay 0.25 per draw, clamped at 0.
    if state.transition > 0.0 {
        state.transition -= 0.25;
    }
    if state.transition < 0.0 {
        state.transition = 0.0;
    }
}

#[cfg(test)]
mod bg_style_tests {
    use super::*;

    fn input() -> BgStylePickInput {
        BgStylePickInput {
            screen_pos: [32000.0, 12000.0],
            screen: [1600.0, 900.0],
            // The small-world roll shape (`WorldGen.cs:8226-8228`): three band
            // edges around the quartiles; styles 0/1/2/3.
            cave_back_x: [1050, 2100, 3150, 0],
            cave_back_style: [0, 1, 2, 3],
            rock_layer: 516.0,
            max_tiles_y: 1200,
            magma_layer: 1000.0,
            snow_tile_count: 0,
            jungle_tile_count: 0,
            snow_tile_threshold: 1500,
            jungle_tile_threshold: 140,
            zone_beach: false,
            zone_corrupt: false,
            zone_crimson: false,
            zone_hallow: false,
            zone_snow: false,
            zone_glowshroom: false,
        }
    }

    #[test]
    fn the_band_pick_walks_the_three_edges_and_adds_three() {
        // Main.cs:54035-54037: centre column = (screenX + w/2)/16 cast to int.
        let mut i = input();
        // centre 2050 -> band 1 (<= 2100) -> style 1 + 3 = 4.
        i.screen_pos[0] = 2050.0 * 16.0 - 800.0;
        assert_eq!(pick_underground_background_style(&i), 4);
        // centre 1050 -> band 0 (inclusive <=) -> 3.
        i.screen_pos[0] = 1050.0 * 16.0 - 800.0;
        assert_eq!(pick_underground_background_style(&i), 3);
        // centre 3151 -> band 3 (> caveBackX[2], the LAST edge is exclusive).
        i.screen_pos[0] = 3151.0 * 16.0 - 800.0;
        assert_eq!(pick_underground_background_style(&i), 6);
        // centre 3150 -> band 2 (== edge falls to the else) -> 5.
        i.screen_pos[0] = 3150.0 * 16.0 - 800.0;
        assert_eq!(pick_underground_background_style(&i), 5);
    }

    #[test]
    fn snow_and_jungle_counts_override_with_strict_compares() {
        // Main.cs:54038-54055.
        let mut i = input();
        i.snow_tile_count = 1500; // == threshold: NOT >
        assert_ne!(pick_underground_background_style(&i), 1);
        i.snow_tile_count = 1501;
        // Depth gate: (12000 + 900 + 1200)/16 = 881.25 < 950 -> snow wins.
        assert_eq!(pick_underground_background_style(&i), 1);
        // Jungle at threshold: NOT > -> snow stays.
        i.jungle_tile_count = 140;
        assert_eq!(pick_underground_background_style(&i), 1);
        // Jungle over threshold but NOT out-counting snow -> snow stays.
        i.jungle_tile_count = 1000;
        assert_eq!(pick_underground_background_style(&i), 1);
        // Jungle out-counts snow -> 11.
        i.jungle_tile_count = 1502;
        assert_eq!(pick_underground_background_style(&i), 11);
        // The depth gate closes near the world floor: screenY such that
        // (Y + 900 + 1200)/16 >= 950 -> Y >= 13100.
        i.screen_pos[1] = 13100.0;
        i.jungle_tile_count = 0;
        assert_ne!(pick_underground_background_style(&i), 1);
    }

    #[test]
    fn beach_trumps_the_deep_cavern_evil_arm() {
        // Main.cs:54056-54089: the beach arm is the if, the depth arm the else.
        let mut i = input();
        // Deep enough for the cavern arm: Y/16 in (576, 940).
        i.screen_pos[1] = 700.0 * 16.0;
        i.zone_corrupt = true;
        assert_eq!(pick_underground_background_style(&i), 12);
        i.zone_beach = true;
        assert_eq!(pick_underground_background_style(&i), 19);
        i.zone_corrupt = false;
        i.zone_crimson = true;
        assert_eq!(pick_underground_background_style(&i), 21);
        i.zone_crimson = false;
        assert_eq!(pick_underground_background_style(&i), 18);
        i.zone_hallow = true;
        assert_eq!(pick_underground_background_style(&i), 20);
    }

    #[test]
    fn the_deep_snow_evil_ladder_and_the_glowshroom_trump() {
        // Main.cs:54062-54076 + :54090-54093.
        let mut i = input();
        i.screen_pos[1] = 700.0 * 16.0;
        i.zone_snow = true;
        i.zone_corrupt = true;
        assert_eq!(pick_underground_background_style(&i), 15);
        i.zone_corrupt = false;
        i.zone_crimson = true;
        assert_eq!(pick_underground_background_style(&i), 16);
        i.zone_crimson = false;
        i.zone_hallow = true;
        assert_eq!(pick_underground_background_style(&i), 17);
        i.zone_glowshroom = true;
        assert_eq!(pick_underground_background_style(&i), 2);
        // The depth window is EXCLUSIVE on both ends (rockLayer+60, magma-60).
        let mut j = input();
        j.zone_corrupt = true;
        j.screen_pos[1] = 576.0 * 16.0; // == rockLayer + 60: not >
        assert_ne!(pick_underground_background_style(&j), 12);
        j.screen_pos[1] = 940.0 * 16.0; // == magmaLayer - 60: not <
        assert_ne!(pick_underground_background_style(&j), 12);
    }

    #[test]
    fn style_zero_matches_the_lived_classic_table() {
        // The live port's `draw_background_update_background_styles` pins this
        // same row (render/bg.rs `background_styles_follow_hell_back_style`).
        for hell in 0..3 {
            assert_eq!(
                background_style_sheets(0, 0, 0, hell, 0),
                [1, 2, 4, 3, 6, 125 + hell, 185 + hell]
            );
        }
    }

    #[test]
    fn the_ice_and_jungle_variants_pick_their_sheet_quartets() {
        // Main.cs:53822-53852 (ice) + :53916-53933 (jungle).
        assert_eq!(
            background_style_sheets(1, 0, 0, 0, 0),
            [40, 33, 34, 32, 128, 125, 185]
        );
        assert_eq!(
            background_style_sheets(1, 2, 0, 1, 0),
            [164, 165, 166, 167, 129, 126, 186]
        );
        // iceBackStyle 3 falls to the else arm (119/120/162/163).
        assert_eq!(
            background_style_sheets(1, 3, 0, 0, 0),
            [162, 120, 163, 119, 128, 125, 185]
        );
        assert_eq!(
            background_style_sheets(11, 0, 0, 2, 0),
            [153, 147, 148, 149, 152, 127, 187]
        );
        assert_eq!(
            background_style_sheets(11, 0, 1, 0, 0),
            [146, 154, 155, 156, 157, 125, 185]
        );
    }

    #[test]
    fn the_world_id_rolls_pick_the_evil_rock_rows() {
        // Main.cs:53944/53947/53950 + :53964/53967/53970. C# `%` on a negative
        // worldID stays negative — the port's i32 `%` matches sign-for-sign.
        assert_eq!(background_style_sheets(12, 0, 0, 0, 7)[3], 193 + 3);
        assert_eq!(background_style_sheets(13, 0, 0, 0, 7)[3], 188 + 2);
        assert_eq!(background_style_sheets(14, 0, 0, 0, 7)[3], 197 + 1);
        assert_eq!(background_style_sheets(15, 0, 0, 0, 7)[3], 200);
        assert_eq!(background_style_sheets(16, 0, 0, 0, 7)[3], 201 + 1);
        assert_eq!(background_style_sheets(17, 0, 0, 0, 7)[3], 203 + 3);
        assert_eq!(background_style_sheets(12, 0, 0, 0, -7)[3], 193 - 3);
        // The evil caverns keep style 3's [0..2] and the 128+hell magma band.
        let b = background_style_sheets(13, 0, 0, 1, 0);
        assert_eq!([b[0], b[1], b[2], b[4]], [66, 67, 68, 129]);
    }

    #[test]
    fn the_ocean_styles_leave_the_rock_slots_cleared() {
        // Main.cs:53977-53992: only [0]/[1]; [2..4] stay 0, the hell tail still
        // writes [5]/[6] (the magma pass never runs without a rock sheet, but
        // the TABLE carries the slots — verbatim).
        assert_eq!(
            background_style_sheets(18, 0, 0, 0, 0),
            [290, 291, 0, 0, 0, 125, 185]
        );
        assert_eq!(
            background_style_sheets(21, 0, 0, 2, 0),
            [296, 297, 0, 0, 0, 127, 187]
        );
        // An out-of-range style keeps everything cleared except the tail.
        assert_eq!(
            background_style_sheets(22, 0, 0, 1, 0),
            [0, 0, 0, 0, 0, 126, 186]
        );
    }

    #[test]
    fn the_crossfade_latches_decays_and_dies_under_shimmer() {
        // Main.cs:52823-52841.
        let mut s = UgBackState { style: 0, old_style: 0, transition: 0.0 };
        // Same style: nothing latches, nothing decays.
        ug_back_step(&mut s, 0, 0.0);
        assert_eq!((s.style, s.old_style, s.transition), (0, 0, 0.0));
        // A change latches old=0 and arms 1, then the SAME step decays to 0.75.
        ug_back_step(&mut s, 4, 0.0);
        assert_eq!((s.style, s.old_style, s.transition), (4, 0, 0.75));
        ug_back_step(&mut s, 4, 0.0);
        assert_eq!(s.transition, 0.5);
        ug_back_step(&mut s, 4, 0.0);
        assert_eq!(s.transition, 0.25);
        ug_back_step(&mut s, 4, 0.0);
        assert_eq!(s.transition, 0.0);
        // A mid-fade change re-latches: old becomes the CURRENT style (4), the
        // fade re-arms at 1 and the same step decays it to 0.75.
        ug_back_step(&mut s, 7, 0.0);
        assert_eq!((s.style, s.old_style, s.transition), (7, 4, 0.75));
        // ANY shimmer > 0 zeroes the fade BEFORE the decay (no negative dip).
        ug_back_step(&mut s, 7, 0.3);
        assert_eq!(s.transition, 0.0);
    }
}
