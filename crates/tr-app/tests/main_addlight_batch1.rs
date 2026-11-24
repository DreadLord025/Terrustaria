//! The Main.cs AddLight batch-1 behavioural witnesses (W4-g18, S1-E-5
//! wave 4, code-only - WRITTEN NOT RUN, no compiler invoked).
//!
//! Batch 1 = the FIRST FOUR of Main.cs's five `Lighting.AddLight` sites
//! (`:29016/:29020/:29024/:29028`), the type-332 Christmas-hook chain's
//! four rotating colour rows inside `DrawProj` (`Main.cs:28991-29045`).
//! Site 5 (`:29186`, the 488 Illuminant hook) is batch 2 (W4-g19).
//!
//! The PORT of all five sites is W4-e8's in-flight
//! `crates/tr-app/src/tile_lights.rs::christmas_hook_chain_lights`
//! (single copy - this lane adds no duplicate module, the W4-f9
//! `c2740784` census-close precedent). W4-g19's in-flight
//! `main_addlight_census.rs` freezes the whole-file census + arm
//! STRUCTURE (line numbers / statement text). THIS file is the
//! complementary BEHAVIOURAL oracle: a statement-for-statement f32
//! mirror of the C# walk's light half, pinned against hand-derived
//! fixtures (every arithmetic step rounded to f32 exactly where the C#
//! rounds - `Math.Sqrt` computes in double then casts to float, which
//! equals the correctly-rounded f32 sqrt, so `f32::sqrt` agrees).
//!
//! **Merger obligation:** replay e8's `christmas_hook_chain_lights`
//! against [`ORACLE_FIXTURES`] below (same inputs, same
//! `(tile_x, tile_y, [r, g, b])` sequence). The mirror here is the
//! reference, not the port.
//!
//! RNG: the 332 light walk draws NOTHING (the two `EntitySpriteDraw`
//! calls at `:29037-29038` are draw-only and dropped) - stream-neutral
//! by construction.

use glam::Vec2;

/// The verbatim f32 mirror of `Main.cs:28993-29044`'s light half.
/// `center` = the hook's `proj.Center` (`:28994`'s
/// `position + width*0.5f` - `Entity.cs:50-54`'s `/2f` is the same
/// IEEE product), `mounted_center` = `player.MountedCenter`
/// (`Main.cs:27936`). The rotation (`:28997`) and the two chain sprite
/// draws (`:29037-29038`) are draw-only - dropped, the light half kept.
fn christmas_hook_light_mirror(center: Vec2, mounted_center: Vec2) -> Vec<(i32, i32, [f32; 3])> {
    let mut vector19 = center; // `:28994`
    let mut num113 = mounted_center.x - vector19.x; // `:28995`
    let mut num114 = mounted_center.y - vector19.y; // `:28996`
    let mut num112 = 0i32; // `:28993`
    let mut out = Vec::new();
    loop {
        // `:29001` - `(float)Math.Sqrt(num113*num113 + num114*num114)`.
        let mut num115 = (num113 * num113 + num114 * num114).sqrt();
        if num115 < 30.0 {
            break; // `:29002-29006` - `flag21 = false; continue`
        }
        if num115.is_nan() {
            break; // `:29007-29011`
        }
        // `:29012-29013` - `(int)vector19.X / 16`: truncate the
        // coordinate FIRST, then integer-divide (NOT the Vector2
        // overload's quotient form, NOT `>> 4`).
        let i2 = vector19.x as i32 / 16;
        let j2 = vector19.y as i32 / 16;
        // `:29014-29029` - four SEPARATE `if`s (not else-if), source
        // order; the colour literals are the C#'s verbatim.
        if num112 == 0 {
            out.push((i2, j2, [0.0, 0.2, 0.2])); // `:29016`
        }
        if num112 == 1 {
            out.push((i2, j2, [0.1, 0.2, 0.0])); // `:29020`
        }
        if num112 == 2 {
            out.push((i2, j2, [0.2, 0.1, 0.0])); // `:29024`
        }
        if num112 == 3 {
            out.push((i2, j2, [0.2, 0.0, 0.2])); // `:29028`
        }
        // `:29030-29036` - the 16 px step toward the mount, re-aim.
        num115 = 16.0 / num115;
        num113 *= num115;
        num114 *= num115;
        vector19.x += num113;
        vector19.y += num114;
        num113 = mounted_center.x - vector19.x;
        num114 = mounted_center.y - vector19.y;
        // `:29039-29043` - the colour index wraps after 3.
        num112 += 1;
        if num112 > 3 {
            num112 = 0;
        }
    }
    out
}

/// Hand-derived f32-exact fixtures: (centre, mount, expected lights).
/// The tile sequence for the diagonal case pins the f32 rounding at
/// the near-boundary links (e.g. link 10 sits at x=95.99999 -> tile 5,
/// NOT 96 -> tile 6; link 15 at y=192.00003 -> tile 12).
const ORACLE_FIXTURES: &[(Vec2, Vec2, &[(i32, i32, [f32; 3])])] = &[
    // A: the straight 160 px drop - 9 links, colours wrap 0..3,0..3,0;
    // exit at remaining 16 < 30 (the W4-e8/g19 hand-replay agrees).
    (
        Vec2::new(160.0, 160.0),
        Vec2::new(160.0, 320.0),
        &[
            (10, 10, [0.0, 0.2, 0.2]),
            (10, 11, [0.1, 0.2, 0.0]),
            (10, 12, [0.2, 0.1, 0.0]),
            (10, 13, [0.2, 0.0, 0.2]),
            (10, 14, [0.0, 0.2, 0.2]),
            (10, 15, [0.1, 0.2, 0.0]),
            (10, 16, [0.2, 0.1, 0.0]),
            (10, 17, [0.2, 0.0, 0.2]),
            (10, 18, [0.0, 0.2, 0.2]),
        ],
    ),
    // B: a hook closer than 30 px never lights (`:29002` exit first).
    (Vec2::new(160.0, 300.0), Vec2::new(160.0, 320.0), &[]),
    // C: 150 px - 8 links (the 9th gate reads 22 < 30 after the step).
    (
        Vec2::new(160.0, 160.0),
        Vec2::new(160.0, 310.0),
        &[
            (10, 10, [0.0, 0.2, 0.2]),
            (10, 11, [0.1, 0.2, 0.0]),
            (10, 12, [0.2, 0.1, 0.0]),
            (10, 13, [0.2, 0.0, 0.2]),
            (10, 14, [0.0, 0.2, 0.2]),
            (10, 15, [0.1, 0.2, 0.0]),
            (10, 16, [0.2, 0.1, 0.0]),
            (10, 17, [0.2, 0.0, 0.2]),
        ],
    ),
    // D: negative x - the truncate-first anchor reads -165 -> -10
    // (NOT -11: `>> 4` would floor; the quotient overload agrees here
    // but the SOURCE form is `(int)v / 16`, pinned by g19's :29012).
    (
        Vec2::new(-165.0, 0.0),
        Vec2::new(-165.0, 160.0),
        &[
            (-10, 0, [0.0, 0.2, 0.2]),
            (-10, 1, [0.1, 0.2, 0.0]),
            (-10, 2, [0.2, 0.1, 0.0]),
            (-10, 3, [0.2, 0.0, 0.2]),
            (-10, 4, [0.0, 0.2, 0.2]),
            (-10, 5, [0.1, 0.2, 0.0]),
            (-10, 6, [0.2, 0.1, 0.0]),
            (-10, 7, [0.2, 0.0, 0.2]),
            (-10, 8, [0.0, 0.2, 0.2]),
        ],
    ),
];

#[test]
fn w4_g18_the_332_walk_replays_the_hand_derived_fixtures() {
    for (center, mount, expected) in ORACLE_FIXTURES {
        let got = christmas_hook_light_mirror(*center, *mount);
        assert_eq!(got.as_slice(), *expected, "centre {center:?} mount {mount:?}");
    }
}

#[test]
fn w4_g18_the_diagonal_walk_pins_the_f32_boundary_links() {
    // (0,0) -> (300,400): the 500 px diagonal, 16 px strides - 30
    // links (exit when the remaining distance < 30). The tile
    // sequence carries the f32 rounding at the near-boundary links.
    let got = christmas_hook_light_mirror(Vec2::ZERO, Vec2::new(300.0, 400.0));
    let tiles: Vec<(i32, i32)> = got.iter().map(|&(x, y, _)| (x, y)).collect();
    assert_eq!(
        tiles,
        vec![
            (0, 0), (0, 0), (1, 1), (1, 2), (2, 3), (3, 4), (3, 4),
            (4, 5), (4, 6), (5, 7), (5, 8), (6, 8), (7, 9), (7, 10),
            (8, 11), (9, 12), (9, 12), (10, 13), (10, 14), (11, 15),
            (12, 16), (12, 16), (13, 17), (13, 18), (14, 19), (15, 19),
            (15, 20), (16, 21), (16, 22), (17, 23),
        ]
    );
    // The colours cycle 0,1,2,3 for all 30 links (30 % 4 == 2: the
    // last two links are rows 2 then 3).
    let rows = [
        [0.0f32, 0.2, 0.2],
        [0.1, 0.2, 0.0],
        [0.2, 0.1, 0.0],
        [0.2, 0.0, 0.2],
    ];
    for (i, &(_, _, c)) in got.iter().enumerate() {
        assert_eq!(c, rows[i % 4], "link {i}");
    }
}

#[test]
fn w4_g18_the_nan_hook_stays_dark() {
    // `:29007-29011` - a NaN distance exits before any light.
    let got = christmas_hook_light_mirror(Vec2::new(f32::NAN, f32::NAN), Vec2::new(160.0, 320.0));
    assert!(got.is_empty());
}

#[test]
fn w4_g18_the_four_colour_literals_are_the_csharp_verbatim() {
    // The rows as written at Main.cs:29016/29020/29024/29028 - the
    // exact float literals, no scaling, no torch-id indirection.
    let got = christmas_hook_light_mirror(Vec2::new(160.0, 160.0), Vec2::new(160.0, 320.0));
    assert_eq!(got[0].2, [0.0, 0.2, 0.2]); // `0f, 0.2f, 0.2f`
    assert_eq!(got[1].2, [0.1, 0.2, 0.0]); // `0.1f, 0.2f, 0f`
    assert_eq!(got[2].2, [0.2, 0.1, 0.0]); // `0.2f, 0.1f, 0f`
    assert_eq!(got[3].2, [0.2, 0.0, 0.2]); // `0.2f, 0f, 0.2f`
}

#[test]
fn w4_g18_the_light_rides_the_pre_step_position() {
    // The FIRST light is at the hook centre itself (tile 10,10 for
    // (160,160)) - the anchor (:29012) precedes the step (:29030) in
    // source order; a post-step port would start at tile (10,11).
    let got = christmas_hook_light_mirror(Vec2::new(160.0, 160.0), Vec2::new(160.0, 320.0));
    assert_eq!(got[0], (10, 10, [0.0, 0.2, 0.2]));
}
