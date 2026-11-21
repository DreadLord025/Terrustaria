//! The hell parallax stack (`Main.DrawUnderworldBackground` /
//! `DrawUnderworldBackgroundLayer`, `Main.cs:52665-52796`), ported verbatim but
//! UNWIRED (W4-g72 lane 3, code-only).
//!
//! Why unwired: the stack draws `TextureAssets.Underworld[0..14]` — five sheets
//! per rolled style (`WorldGen.setBG(9)`, `WorldGen.cs:7609-7635`), the largest
//! 2048x1900. The single 8192 px atlas already packs the full static+gore set
//! to 7530 px (measured with the `tools/atlas_sim.js` skyline twin); even ONE
//! style set overflows the strict-GPU bound (8424 px for style 0). The C# draws
//! these OUTSIDE the tile atlas through `Main.spriteBatch` under the sky
//! matrix; the port needs the S1-G-13 background-target family or a second
//! texture array first. The `underworldBG` style roll is also unowned (the
//! `.wld` byte is skip-read, `wld/sections/flags.rs:566`).
//!
//! The geometry here is pure — no RNG anywhere in the two C# functions; the
//! only time input is `Main.GlobalTimeWrappedHourly` (the 4-frame animation
//! cycle). Witnessed line-by-line against the source.

/// `WorldGen.setBG` case 9 (`WorldGen.cs:7609-7635`): the rolled underworld
/// style expands into the five `Main.underworldBG` layer texture indices.
/// The `default:` arm covers style 0 (and any out-of-range byte).
pub(crate) fn underworld_bg_set(style: i32) -> [i32; 5] {
    match style {
        1 => [5, 6, 7, 8, 9],   // WorldGen.cs:7620-7626
        2 => [10, 11, 12, 13, 9], // WorldGen.cs:7627-7633 — shares layer 9
        _ => [0, 1, 2, 3, 4],   // WorldGen.cs:7613-7619 (default)
    }
}

/// `Main.DrawUnderworldBackground`'s gate + push-up (`Main.cs:52665-52670`).
/// The pass runs only when the screen bottom reaches `(maxTilesY - 220) * 16`;
/// `pushUp = (zoomForPushUp - 1) * 0.5 * 200` lifts every layer at zoom > 1.
pub(crate) fn underworld_pass_visible(
    screen_pos_y: f32,
    screen_height: f32,
    max_tiles_y: i32,
) -> bool {
    // Main.cs:52667: `if (!(screenY + screenHeight < (maxTilesY - 220) * 16f))`
    !(screen_pos_y + screen_height < (max_tiles_y - 220) as f32 * 16.0)
}

/// `Main.cs:52670`: `(zoomForPushUp - 1f) * 0.5f * 200f`.
pub(crate) fn underworld_push_up(zoom_for_push_up: f32) -> f32 {
    (zoom_for_push_up - 1.0) * 0.5 * 200.0
}

/// One layer's draw plan — the source rect, scale, anchor and the tiled
/// positions `DrawUnderworldBackgroundLayer` computes (`Main.cs:52684-52796`).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UnderworldLayerPlan {
    /// `value2` — the source rectangle (the 4-frame sheets pick a quadrant).
    pub src: [i32; 4],
    /// `num3` — the draw scale (1.3, or 0.5 for texture 4).
    pub scale: f32,
    /// `num2` — the parallax divisor `layerTextureIndex * 2 + 3`; the depth
    /// tracker boundary is `1 / (1/num2)` = `num2` (`Main.cs:52774`).
    pub depth: f32,
    /// `num10` — the scaled tile width (`num3 * value2.Width`).
    pub tile_w: f32,
    /// `num11` AFTER the off-screen walk — the first drawn tile index is
    /// `first_index - 2` (`Main.cs:52786`).
    pub first_index: i32,
    /// The loop's last index, inclusive (`num11 + 4 + num12`, `:52786`).
    pub last_index: i32,
    /// The `.Floor()`ed seed position (`Main.cs:52780`) BEFORE the off-screen
    /// walk — the walk only ever adds whole `tile_w` steps to X.
    pub seed_position: [f32; 2],
    /// `position` after the walk — the first tile's screen-space top-left;
    /// each next tile adds `tile_w` to X (`Main.cs:52794`).
    pub position: [f32; 2],
    /// The layer-0 black fill row (`Main.cs:52789-52793`): `(x, y, w, h)` with
    /// the (11, 3, 7) tint; `None` for the other layers. `h` clamps at 0.
    pub black_fill: Option<[i32; 4]>,
}

/// `Main.DrawUnderworldBackgroundLayer` (`Main.cs:52684-52796`), verbatim.
/// `tex_w`/`tex_h` are the sheet dims (`TextureAssets.Underworld[num]`),
/// `global_time` is `Main.GlobalTimeWrappedHourly`, `screen_offset` is
/// `screenPosition + (screenWidth >> 1, screenHeight >> 1)` (`:52669`),
/// `underworld_layer` is `Main.UnderworldLayer` (`maxTilesY - 200`,
/// `Main.cs:2790`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn underworld_layer_plan(
    layer_texture_index: i32,
    tex_id: i32,
    tex_w: i32,
    tex_h: i32,
    screen_offset: [f32; 2],
    screen_pos: [f32; 2],
    screen: [f32; 2],
    push_up: f32,
    global_time: f32,
    underworld_layer: i32,
) -> UnderworldLayerPlan {
    // Main.cs:52693-52699: the half-size anchor, the parallax divisor, the
    // full-sheet source rect, the 1.3 scale, the zero offset, num4 = 0.
    let mut vec = [tex_w as f32 * 0.5, tex_h as f32 * 0.5];
    let num2 = (layer_texture_index * 2) as f32 + 3.0;
    let vector = 1.0 / num2;
    let mut src = [0, 0, tex_w, tex_h];
    let mut num3 = 1.3f32;
    let mut zero = [0.0f32; 2];
    let num4 = 0.0f32; // Main.cs:52699 — never reassigned; the += num4 are no-ops
    // Main.cs:52700-52772: the per-texture switch. The 4-frame cycle is
    // `(int)(GlobalTimeWrappedHourly * 8f) % 4`; textures 1/6/7/8/13 pick a
    // quadrant and halve the anchor. NOTE texture 1's quadrant is TRANSPOSED
    // vs the others: `(num9 >> 1)` picks the COLUMN, `num9 % 2` the ROW.
    let frame = (global_time * 8.0) as i32 % 4;
    let half = [tex_w >> 1, tex_h >> 1];
    match tex_id {
        1 => {
            // Main.cs:52702-52709.
            src = [(frame >> 1) * half[0], frame % 2 * half[1], half[0], half[1]];
            vec = [vec[0] * 0.5, vec[1] * 0.5];
            zero[1] += 175.0;
        }
        2 => zero[1] += 100.0, // Main.cs:52710-52712
        3 => zero[1] += 75.0,  // Main.cs:52713-52715
        4 => {
            // Main.cs:52716-52719: the 48x1300 strip draws at 0.5.
            num3 = 0.5;
            zero[1] -= 0.0;
        }
        5 => zero[1] += num4, // Main.cs:52720-52722
        6 => {
            // Main.cs:52723-52731.
            src = [frame % 2 * half[0], (frame >> 1) * half[1], half[0], half[1]];
            vec = [vec[0] * 0.5, vec[1] * 0.5];
            zero[1] += num4;
            zero[1] += -60.0;
        }
        7 => {
            // Main.cs:52732-52741.
            src = [frame % 2 * half[0], (frame >> 1) * half[1], half[0], half[1]];
            vec = [vec[0] * 0.5, vec[1] * 0.5];
            zero[1] += num4;
            zero[0] -= 400.0;
            zero[1] += 90.0;
        }
        8 => {
            // Main.cs:52742-52750.
            src = [frame % 2 * half[0], (frame >> 1) * half[1], half[0], half[1]];
            vec = [vec[0] * 0.5, vec[1] * 0.5];
            zero[1] += num4;
            zero[1] += 90.0;
        }
        9 => {
            // Main.cs:52751-52754.
            zero[1] += num4;
            zero[1] -= 30.0;
        }
        10 => zero[1] += 250.0 * num2, // Main.cs:52755-52757
        11 => zero[1] += 100.0 * num2, // Main.cs:52758-52760
        12 => zero[1] += 20.0 * num2,  // Main.cs:52761-52763
        13 => {
            // Main.cs:52764-52771: the 20*num2 lift AND the quadrant cycle.
            zero[1] += 20.0 * num2;
            src = [frame % 2 * half[0], (frame >> 1) * half[1], half[0], half[1]];
            vec = [vec[0] * 0.5, vec[1] * 0.5];
        }
        _ => {} // textures 0/14+: the full sheet at 1.3, no offset
    }
    // Main.cs:52773-52775: scale the anchor, then lift by the zoom push-up.
    vec = [vec[0] * num3, vec[1] * num3];
    zero[1] -= push_up;
    // Main.cs:52776-52777: the tile width and the first index — the WHOLE
    // numerator `(float)(int)(screenOffset.X * vector.X - vec.X + zero.X -
    // (float)(screenWidth >> 1))` truncates to int BEFORE the divide by num10,
    // then the quotient casts to int again.
    let num10 = num3 * src[2] as f32;
    let mut num11 = (((screen_offset[0] * vector - vec[0] + zero[0]
        - (screen[0] as i32 >> 1) as f32) as i32 as f32)
        / num10) as i32;
    // Main.cs:52778: the anchor floors AFTER the index computation.
    vec = [vec[0].floor(), vec[1].floor()];
    // Main.cs:52779: `Math.Ceiling((float)screenWidth / num10)` — int cast.
    let num12 = (screen[0] / num10).ceil() as i32;
    // Main.cs:52780: the position chain. The X pitch is
    // `(int)(num3 * ((float)(value2.Width - 1) / vector.X))` — width MINUS ONE,
    // divided by the parallax factor, scaled, truncated. The Y anchor is
    // `UnderworldLayer * 16f`. Then `+ vec - screenOffset`, parallax-scaled,
    // `+ screenOffset - screenPosition - vec + zero`, floored per component.
    let pitch = (num3 * ((src[2] - 1) as f32 / vector)) as i32;
    let px = ((num11 - 2) * pitch) as f32;
    let py = underworld_layer as f32 * 16.0;
    let mut position = [
        ((px + vec[0] - screen_offset[0]) * vector + screen_offset[0]
            - screen_pos[0]
            - vec[0]
            + zero[0])
            .floor(),
        ((py + vec[1] - screen_offset[1]) * vector + screen_offset[1]
            - screen_pos[1]
            - vec[1]
            + zero[1])
            .floor(),
    ];
    // Main.cs:52781-52785: walk the first tile on-screen (strict `< 0`).
    let seed_position = position;
    while position[0] + num10 < 0.0 {
        num11 += 1;
        position[0] += num10;
    }
    // Main.cs:52789-52793: layer 0 paints the (11,3,7) black fill under every
    // tile, from the tile's scaled bottom to the screen bottom (clamped >= 0).
    let black_fill = if layer_texture_index == 0 {
        let num13 = (position[1] + src[3] as f32 * num3) as i32;
        Some([
            position[0] as i32,
            num13,
            (src[2] as f32 * num3) as i32,
            (screen[1] as i32 - num13).max(0),
        ])
    } else {
        None
    };
    UnderworldLayerPlan {
        src,
        scale: num3,
        depth: num2,
        tile_w: num10,
        first_index: num11,
        last_index: num11 + 4 + num12,
        seed_position,
        position,
        black_fill,
    }
}

#[cfg(test)]
mod underworld_bg_tests {
    use super::*;

    /// The reference camera: a 1600x900 screen at the underworld floor of a
    /// 1200-row world (`UnderworldLayer = 1000`, the floor line at 16000 px).
    fn scene() -> ([f32; 2], [f32; 2], [f32; 2]) {
        let screen = [1600.0, 900.0];
        let screen_pos = [32000.0, 15600.0];
        let screen_offset = [
            screen_pos[0] + (screen[0] as i32 >> 1) as f32,
            screen_pos[1] + (screen[1] as i32 >> 1) as f32,
        ];
        (screen_offset, screen_pos, screen)
    }

    #[test]
    fn the_style_sets_match_set_bg_case_nine() {
        // WorldGen.cs:7613-7633.
        assert_eq!(underworld_bg_set(0), [0, 1, 2, 3, 4]);
        assert_eq!(underworld_bg_set(1), [5, 6, 7, 8, 9]);
        assert_eq!(underworld_bg_set(2), [10, 11, 12, 13, 9]);
        // The default arm covers any out-of-range byte.
        assert_eq!(underworld_bg_set(7), [0, 1, 2, 3, 4]);
    }

    #[test]
    fn the_gate_and_push_up_match_the_head() {
        // Main.cs:52667: visible when screenY + screenH >= (maxTilesY-220)*16.
        // (1200-220)*16 = 15680.
        assert!(underworld_pass_visible(15000.0, 900.0, 1200)); // 15900 >= 15680
        assert!(!underworld_pass_visible(14000.0, 900.0, 1200)); // 14900 < 15680
        assert!(underworld_pass_visible(14780.0, 900.0, 1200)); // == 15680 inclusive
        // Main.cs:52670: (zoom-1)*0.5*200.
        assert_eq!(underworld_push_up(1.0), 0.0);
        assert_eq!(underworld_push_up(1.5), 50.0);
        assert_eq!(underworld_push_up(2.0), 100.0);
    }

    #[test]
    fn the_depth_ladder_is_two_per_layer_plus_three() {
        // Main.cs:52694-52695 + :52774: DrawToDepth(1/vector.X) = num2.
        let (so, sp, sc) = scene();
        for layer in 0..5 {
            let p = underworld_layer_plan(layer, 0, 1024, 858, so, sp, sc, 0.0, 0.0, 1000);
            assert_eq!(p.depth, (layer * 2 + 3) as f32);
        }
    }

    #[test]
    fn texture_one_transposes_the_quadrant_and_lifts_175() {
        // Main.cs:52702-52709: column = frame>>1, row = frame%2 (the OTHER
        // animated sheets swap these), anchor halved, +175 Y.
        let (so, sp, sc) = scene();
        // 2048x1900 sheet (Underworld 1's real dims), frame at t=0.30s:
        // (int)(0.30*8) = 2 -> col 1, row 0.
        let p = underworld_layer_plan(1, 1, 2048, 1900, so, sp, sc, 0.0, 0.30, 1000);
        assert_eq!(p.src, [1024, 0, 1024, 950]);
        assert_eq!(p.scale, 1.3);
        // The non-transposed twin (texture 6) at the same frame: col 0, row 1.
        let q = underworld_layer_plan(1, 6, 2048, 1716, so, sp, sc, 0.0, 0.30, 1000);
        assert_eq!(q.src, [0, 858, 1024, 858]);
        // Frame 3 (t=0.40s -> (int)3.2=3): texture 1 col 1 row 1; 6 col 1 row 1.
        let r = underworld_layer_plan(1, 1, 2048, 1900, so, sp, sc, 0.0, 0.40, 1000);
        assert_eq!(r.src, [1024, 950, 1024, 950]);
    }

    #[test]
    fn texture_four_draws_the_strip_at_half_scale() {
        // Main.cs:52716-52719: num3 = 0.5, the 48x1300 strip (Underworld 4).
        let (so, sp, sc) = scene();
        let p = underworld_layer_plan(4, 4, 48, 1300, so, sp, sc, 0.0, 0.0, 1000);
        assert_eq!(p.scale, 0.5);
        assert_eq!(p.src, [0, 0, 48, 1300]);
        assert_eq!(p.tile_w, 24.0);
    }

    #[test]
    fn the_static_lifts_and_the_num2_scaled_lifts() {
        // Main.cs:52710-52715/52751-52763: textures 2/3/9 lift 100/75/-30;
        // 10/11/12/13 lift 250/100/20/20 times num2. The C# position chain is
        // `(... + vec - screenOffset) * vector + screenOffset - screenPosition
        // - vec + zero` — the anchor is parallax-scaled INSIDE and subtracted
        // UNSCALED outside, so a `zero` lift lands 1:1 on the position (only
        // the shared floor base can shift the delta by one). Expectations
        // derived with exact f32 arithmetic (tools witness derivation).
        let (so, sp, sc) = scene();
        let base = underworld_layer_plan(0, 0, 1024, 858, so, sp, sc, 0.0, 0.0, 1000);
        let lifted = underworld_layer_plan(0, 2, 1024, 858, so, sp, sc, 0.0, 0.0, 1000);
        // +100 Y: seed 62 -> 162 (the floor base is identical here).
        assert_eq!(base.seed_position[1], 62.0);
        assert_eq!(lifted.seed_position[1], 162.0);
        // Texture 10 at layer 2 (num2 = 7): +250*7 = 1750 lands 1:1:
        // seed -35 -> 1715.
        let t10 = underworld_layer_plan(2, 10, 1024, 858, so, sp, sc, 0.0, 0.0, 1000);
        let t0 = underworld_layer_plan(2, 0, 1024, 858, so, sp, sc, 0.0, 0.0, 1000);
        assert_eq!(t0.seed_position[1], -35.0);
        assert_eq!(t10.seed_position[1], 1715.0);
        // The push-up subtracts AFTER the lifts (Main.cs:52775): zoom 2's
        // 100 px push moves the seed 62 -> -38.
        let pushed = underworld_layer_plan(0, 0, 1024, 858, so, sp, sc, 100.0, 0.0, 1000);
        assert_eq!(pushed.seed_position[1], -38.0);
        // Texture 9's -30 lift at layer 4 (num2 = 11): the floor base shifts
        // the delta to -30 exactly here (-61 -> -91).
        let t9 = underworld_layer_plan(4, 9, 1024, 858, so, sp, sc, 0.0, 0.0, 1000);
        let t0l4 = underworld_layer_plan(4, 0, 1024, 858, so, sp, sc, 0.0, 0.0, 1000);
        assert_eq!(t0l4.seed_position[1], -61.0);
        assert_eq!(t9.seed_position[1], -91.0);
    }

    #[test]
    fn the_index_walk_and_loop_bounds_match() {
        // Main.cs:52777-52786: the truncated numerator, the width-minus-one
        // pitch, the strict off-screen walk, the -2/+4+ceil loop span.
        // Expectations derived with exact f32 arithmetic (every intermediate
        // rounded, mirroring the port's op order): vector = 1/3f;
        // vec = (512,429)*1.3f = (665.59998, 557.69995);
        // numerator = 32800*vector - 665.59998 - 800 = 9467.734 -> int 9467;
        // num10 = 1.3f*1024 = 1331.19995; 9467/1331.19995 = 7.11 -> num11 = 7.
        // pitch = (int)(1.3f * (1023/(1/3f))) = 3989;
        // seedX = ((7-2)*3989 + 665 - 32800)*vector + 32800 - 32000 - 665
        //       = -3929 (floored). Walk (step 1331.19995): -3929 -> -2597.8
        //       -> -1266.6001 (stop: + num10 >= 0). num11 = 7 + 2 = 9.
        let (so, sp, sc) = scene();
        let p = underworld_layer_plan(0, 0, 1024, 858, so, sp, sc, 0.0, 0.0, 1000);
        assert_eq!(p.tile_w, 1331.199951171875);
        assert_eq!(p.seed_position[0], -3929.0);
        assert_eq!(p.first_index, 9);
        assert_eq!(p.position[0], -1266.60009765625);
        // num12 = ceil(1600/1331.19995) = 2; the loop spans first-2 .. first+4+2.
        assert_eq!(p.last_index, 9 + 4 + 2);
        assert!(p.position[0] + p.tile_w >= 0.0);
        // The walk's X step is num10, NOT the pitch (Main.cs:52794 vs :52780).
        assert_eq!(p.position[0] - p.seed_position[0], 2.0 * p.tile_w);
    }

    #[test]
    fn layer_zero_paints_the_black_fill_row() {
        // Main.cs:52789-52793: only layerTextureIndex == 0; y = position.Y +
        // srcH*scale truncated; h = max(0, screenH - y); the (11,3,7) tint is
        // the caller's (the plan carries the rect).
        let (so, sp, sc) = scene();
        let p = underworld_layer_plan(0, 0, 1024, 858, so, sp, sc, 0.0, 0.0, 1000);
        let f = p.black_fill.expect("layer 0 fills");
        // seedY (exact f32 derivation): py = 16000; vec.Y = 429*1.3f =
        // 557.69995, floored 557; (16000 + 557 - 16050)*vector + 16050 - 15600
        //   - 557 = 507*0.333333343 - 107 = 169.0000 - 107 = 62.0 -> floor 62.
        // The walk never touches Y.
        assert_eq!(p.position[1], 62.0);
        // num13 = (int)(62 + 858*1.3f) = (int)(62 + 1115.3999) = 1177.
        assert_eq!(f[1], 1177);
        assert_eq!(f[2], (1024.0 * 1.3f32) as i32); // 1331
        assert_eq!(f[3], 0); // screenH 900 - 1177 < 0 -> clamped
        assert_eq!(f[0], p.position[0] as i32); // -1266 (trunc toward zero)
        // Other layers never fill.
        let q = underworld_layer_plan(1, 1, 2048, 1900, so, sp, sc, 0.0, 0.0, 1000);
        assert!(q.black_fill.is_none());
    }

    #[test]
    fn the_frame_cycle_wraps_every_half_second() {
        // Main.cs:52704: (int)(GlobalTimeWrappedHourly * 8f) % 4 — one frame
        // per 0.125 s, wrapping at 4 (0.5 s).
        let (so, sp, sc) = scene();
        let f = |t: f32| {
            underworld_layer_plan(0, 6, 2048, 1716, so, sp, sc, 0.0, t, 1000).src
        };
        assert_eq!(f(0.0), [0, 0, 1024, 858]);
        assert_eq!(f(0.125), [1024, 0, 1024, 858]);
        assert_eq!(f(0.25), [0, 858, 1024, 858]);
        assert_eq!(f(0.375), [1024, 858, 1024, 858]);
        assert_eq!(f(0.5), [0, 0, 1024, 858]);
    }
}
