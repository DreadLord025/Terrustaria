//! S1-F-9 wave C — the FURNITURE `DrawData` cases that need animation /
//! container state: the 92 critter-cage frame-table rewrites and the chest
//! sub-cell rewrite. The static furniture metrics (chairs 15, tables 14,
//! beds 79, workbenches 18, chests 21, the anvil/furnace/beam 16-group, the
//! pot/statue 28-group) already live in [`crate::drawdata::draw_data`]; this
//! module ports only the parts that table documents as deviations, so the
//! two stay disjoint (waves A/B own `drawdata.rs`/`tree.rs`/`drawdata_anim.rs`).
//!
//! Sources (`decompiled/Terraria/GameContent/Drawing/TileDrawing.cs`):
//! - big-animal cages   `case 275..` group :5629-5755
//! - small-animal cages `case 285..` group :5756-5832
//! - water-animal cages (fish bowls / fairy jars / butterfly jars /
//!   dragonfly jars)    :5833-5897
//! - jellyfish cages    :5898-5907
//! - the cage index probes `GetWaterAnimalCageFrame` :6528-6533,
//!   `GetSmallAnimalCageFrame` :6535-6540, `GetBigAnimalCageFrame` :6542-6547
//! - the chest rewrite `CacheSpecialDraws_Part2` :4287-4330 (runs AFTER
//!   `GetTileDrawData`, `DrawSingleTile` :993), `TileID.Sets.BasicChest =
//!   CreateBoolSet(21, 467)` (TileID.cs:570).
//!
//! The `Main.*CageFrame` tables are animator state (`Main.cageFrames = 25`,
//! Main.cs:66901; the tables are advanced by `Main.AnimateTiles_CritterCages`,
//! Main.cs:13511+, on `Main.rand` — NOT ported here, it is the animator's
//! scope). Like `tile_frame` in `draw_data`, the tables ride in as a lookup
//! parameter: all-zero tables are the exact vanilla t=0 state and reproduce
//! the static arms' output byte-for-byte. No RNG is drawn in this module —
//! the tile-framing RNG streams are untouched.

use crate::drawdata::TileDrawData;

/// `Main.cageFrames` (Main.cs:66901) — the cage animation slot count.
pub const CAGE_FRAMES: i32 = 25;

/// One `Main.*CageFrame` table (the names follow the C# fields, Main.cs:
/// 66902-66972). 1-D tables use `row == 0`; the 2-D tables (`Scorpion`,
/// `Slug`, `Butterfly`, `Dragonfly`, `Jellyfish`) carry the C# first index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CageAnim {
    Bunny,
    Turtle,
    Owl,
    Squirrel,
    Mallard,
    Duck,
    Grebe,
    Seagull,
    Bird,
    BlueBird,
    RedBird,
    Macaw,
    /// `Main.scorpionCageFrame[0, i]` — both scorpion cages read row 0
    /// (TileDrawing.cs:5735-5738).
    Scorpion,
    Penguin,
    Seahorse,
    Pufferfish,
    Snail,
    Snail2,
    Frog,
    Grasshopper,
    Mouse,
    Worm,
    /// `Main.slugCageFrame[typeCache - 392, i]` (TileDrawing.cs:5810-5814).
    Slug,
    Maggot,
    Rat,
    Ladybug,
    WaterStrider,
    FishBowl,
    LavaFishBowl,
    FairyJar,
    /// `Main.butterflyCageFrame[num8, i]` (TileDrawing.cs:5876-5881).
    Butterfly,
    /// `Main.dragonflyJarFrame[typeCache - 521, i]` (TileDrawing.cs:5895).
    Dragonfly,
    /// `Main.jellyfishCageFrame[typeCache - 316, i]` (TileDrawing.cs:5905).
    Jellyfish,
}

/// `TileDrawing.GetWaterAnimalCageFrame` (TileDrawing.cs:6528-6533):
/// `(x - fx/18) / 2 * ((y - fy/18) / 3) % Main.cageFrames`. C# integer
/// division truncates toward zero and `%` keeps the sign — Rust matches.
pub fn water_animal_cage_frame(x: i32, y: i32, fx: i32, fy: i32, cage_frames: i32) -> i32 {
    let num = x - fx / 18;
    let num2 = y - fy / 18;
    num / 2 * (num2 / 3) % cage_frames
}

/// `TileDrawing.GetSmallAnimalCageFrame` (TileDrawing.cs:6535-6540):
/// `(x - fx/18) / 3 * ((y - fy/18) / 3) % Main.cageFrames`.
pub fn small_animal_cage_frame(x: i32, y: i32, fx: i32, fy: i32, cage_frames: i32) -> i32 {
    let num = x - fx / 18;
    let num2 = y - fy / 18;
    num / 3 * (num2 / 3) % cage_frames
}

/// `TileDrawing.GetBigAnimalCageFrame` (TileDrawing.cs:6542-6547):
/// `(x - fx/18) / 6 * ((y - fy/18) / 4) % Main.cageFrames`.
pub fn big_animal_cage_frame(x: i32, y: i32, fx: i32, fy: i32, cage_frames: i32) -> i32 {
    let num = x - fx / 18;
    let num2 = y - fy / 18;
    num / 6 * (num2 / 4) % cage_frames
}

/// The per-kind table pick of the four cage `case` groups: returns
/// `(table, row, stride, big_index)` — `stride` is the C# `* 54` (big) or
/// `* 36` (small/water/jellyfish) multiplier, `big_index` selects the
/// `Get*AnimalCageFrame` probe. `None` for non-cage kinds.
/// (TileDrawing.cs:5674-5753 big, :5783-5830 small, :5833-5897 water,
/// :5898-5907 jellyfish.)
fn cage_table(kind: u16) -> Option<(CageAnim, i32, i32, u8)> {
    const BIG: u8 = 0;
    const SMALL: u8 = 1;
    const WATER: u8 = 2;
    Some(match kind {
        // Big-animal group (TileDrawing.cs:5674-5753), stride 54.
        275 | 359 | 599..=605 => (CageAnim::Bunny, 0, 54, BIG),
        550 | 551 => (CageAnim::Turtle, 0, 54, BIG),
        542 => (CageAnim::Owl, 0, 54, BIG),
        276 | 413 | 414 | 606..=612 => (CageAnim::Squirrel, 0, 54, BIG),
        277 => (CageAnim::Mallard, 0, 54, BIG),
        278 => (CageAnim::Duck, 0, 54, BIG),
        553 => (CageAnim::Grebe, 0, 54, BIG),
        554 => (CageAnim::Seagull, 0, 54, BIG),
        279 | 358 => (CageAnim::Bird, 0, 54, BIG),
        280 => (CageAnim::BlueBird, 0, 54, BIG),
        281 => (CageAnim::RedBird, 0, 54, BIG),
        632 | 640 | 643..=645 => (CageAnim::Macaw, 0, 54, BIG),
        296 | 297 => (CageAnim::Scorpion, 0, 54, BIG),
        309 => (CageAnim::Penguin, 0, 54, BIG),
        558 | 559 => (CageAnim::Seahorse, 0, 54, BIG),
        710 => (CageAnim::Pufferfish, 0, 54, BIG),
        // Small-animal group (TileDrawing.cs:5783-5830), stride 36.
        285 => (CageAnim::Snail, 0, 36, SMALL),
        286 | 582 => (CageAnim::Snail2, 0, 36, SMALL),
        298 | 361 => (CageAnim::Frog, 0, 36, SMALL),
        339 | 362 => (CageAnim::Grasshopper, 0, 36, SMALL),
        299 | 363 => (CageAnim::Mouse, 0, 36, SMALL),
        310 | 364 | 391 | 619 => (CageAnim::Worm, 0, 36, SMALL),
        392..=394 => (CageAnim::Slug, kind as i32 - 392, 36, SMALL),
        532 => (CageAnim::Maggot, 0, 36, SMALL),
        533 => (CageAnim::Rat, 0, 36, SMALL),
        538 | 544 | 629 => (CageAnim::Ladybug, 0, 36, SMALL),
        555 | 556 => (CageAnim::WaterStrider, 0, 36, SMALL),
        // Water-animal group (TileDrawing.cs:5833-5897), stride 36.
        282 | 505 | 543 => (CageAnim::FishBowl, 0, 36, WATER),
        598 => (CageAnim::LavaFishBowl, 0, 36, WATER),
        568..=570 => (CageAnim::FairyJar, 0, 36, WATER),
        // num8 = typeCache - 288, except 360/580/620 -> 8 (:5876-5880).
        288..=295 => (CageAnim::Butterfly, kind as i32 - 288, 36, WATER),
        360 | 580 | 620 => (CageAnim::Butterfly, 8, 36, WATER),
        521..=527 => (CageAnim::Dragonfly, kind as i32 - 521, 36, WATER),
        // Jellyfish cages read the SMALL index (TileDrawing.cs:5898-5907).
        316..=318 => (CageAnim::Jellyfish, kind as i32 - 316, 36, SMALL),
        _ => return None,
    })
}

/// The four cage `case` groups' full draw metrics (TileDrawing.cs:5629-5907):
/// `tileTop = 2` plus the per-animal `addFrY = Main.<table>[row, idx] * stride`
/// rewrite (the pufferfish also rewrites `addFrX`, :5746-5752). Returns `None`
/// for non-cage kinds — the caller keeps the `draw_data` table's result.
///
/// `frame_of(table, row, idx)` reads the animator's `Main.*CageFrame` tables;
/// an all-zero lookup is the vanilla t=0 state (the tables start zeroed,
/// Main.cs:66902-66972, and only `AnimateTiles_CritterCages` moves them).
/// The `Main.critterCage = true` side effect (:5672 etc.) gates that animator
/// and is not draw state — it stays with the animator port.
pub fn cage_draw_data(
    kind: u16,
    x: i32,
    y: i32,
    frame_x: i16,
    frame_y: i16,
    cage_frames: i32,
    frame_of: impl Fn(CageAnim, i32, i32) -> i32,
) -> Option<TileDrawData> {
    let (table, row, stride, probe) = cage_table(kind)?;
    let fx = frame_x as i32;
    let fy = frame_y as i32;
    let idx = match probe {
        0 => big_animal_cage_frame(x, y, fx, fy, cage_frames),
        1 => small_animal_cage_frame(x, y, fx, fy, cage_frames),
        _ => water_animal_cage_frame(x, y, fx, fy, cage_frames),
    };
    let mut data = TileDrawData {
        top: 2,
        ..TileDrawData::default()
    };
    let frame = frame_of(table, row, idx);
    if table == CageAnim::Pufferfish {
        // `case 710` (TileDrawing.cs:5746-5752): the frame value splits into
        // a 108-wide column block and a 54-tall row within it.
        let num9 = frame / 33;
        data.add_fr_x = (108 * num9) as i16;
        data.add_fr_y = ((frame - num9 * 33) * 54) as i16;
    } else {
        data.add_fr_y = (frame * stride) as i16;
    }
    Some(data)
}

/// `TileID.Sets.BasicChest = CreateBoolSet(21, 467)` (TileID.cs:570).
pub fn is_basic_chest(kind: u16) -> bool {
    kind == 21 || kind == 467
}

/// Whether a kind rides one of the four cage `case` groups — the G6 key's
/// in-view gate for the cage animator (the `TileAnimator::is_animated_kind`
/// convention) and the draw's `Main.critterCage` set probe
/// (TileDrawing.cs:5672 etc.).
pub fn is_cage_kind(kind: u16) -> bool {
    cage_table(kind).is_some()
}

/// The chest half of `CacheSpecialDraws_Part2` (TileDrawing.cs:4289-4330),
/// which `DrawSingleTile` runs AFTER `GetTileDrawData` (:993): the source
/// rect folds back to the chest's sheet origin per sub-cell, the opening
/// animation adds +38/+76 to the row, and every row below the first is 18 px
/// tall. `chest_frame` is `Main.chest[Chest.FindChest(anchor)].frame` —
/// `None` when `FindChest` returns -1 (:4309). The anchor probe itself
/// (`key.X--` when `fx % 36 != 0`, `key.Y--` when `fy % 36 != 0`, :4292-4299)
/// belongs to the caller's chest lookup; the port's `World::chests`
/// (`save::ChestRecord`) carries the anchor `(x, y)` for it.
///
/// Deviation (TILE-DRAW-2 scope): the 21-columns-48/49 `glowSourceRect`
/// (:4326-4329) is not carried — `TileDrawData` has no glow fields yet.
/// The `typeCache == 378` training-dummy half (:4331-4355) needs
/// `TETrainingDummy` + `Main.npc[npc].frame` and stays unported.
pub fn chest_draw_rewrite(
    kind: u16,
    frame_x: i16,
    frame_y: i16,
    chest_frame: Option<i32>,
    data: &mut TileDrawData,
) {
    if !is_basic_chest(kind) {
        return;
    }
    let fx = frame_x as i32;
    let fy = frame_y as i32;
    let num = fx / 18;
    let num2 = fy / 18;
    data.add_fr_x = (num * 18 - fx) as i16;
    let mut num4 = num2 * 18;
    if let Some(frame) = chest_frame {
        if frame == 1 {
            num4 += 38;
        }
        if frame == 2 {
            num4 += 76;
        }
    }
    data.add_fr_y = (num4 - fy) as i16;
    if num2 != 0 {
        data.height = 18;
    }
}

/// The wave-C dispatch hook: given the `draw_data` result for a cell, apply
/// the furniture rewrites this module owns. `cage_frames` is
/// `Main.cageFrames` ([`CAGE_FRAMES`]); `frame_of` reads the cage tables
/// (all-zero until the animator lands); `chest_frame` is the opened chest's
/// animation frame (see [`chest_draw_rewrite`]).
///
/// Wiring note for the S1-F-9 merge: the cage half REPLACES the static
/// cage arms of `drawdata.rs` (the `:5629-5907` deviation arms) inside the
/// table; the chest half runs AFTER the whole table + campfire epilogue,
/// matching `DrawSingleTile`'s `CacheSpecialDraws_Part2` position (:993).
pub fn furniture_post_pass(
    kind: u16,
    x: i32,
    y: i32,
    frame_x: i16,
    frame_y: i16,
    cage_frames: i32,
    frame_of: impl Fn(CageAnim, i32, i32) -> i32,
    chest_frame: Option<i32>,
    data: &mut TileDrawData,
) {
    if let Some(cage) = cage_draw_data(kind, x, y, frame_x, frame_y, cage_frames, frame_of) {
        *data = cage;
    }
    chest_draw_rewrite(kind, frame_x, frame_y, chest_frame, data);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drawdata::draw_data;

    /// The vanilla t=0 cage tables: every `Main.*CageFrame` starts zeroed
    /// (Main.cs:66902-66972).
    fn zero_tables(_: CageAnim, _: i32, _: i32) -> i32 {
        0
    }

    // ---- the index probes (TileDrawing.cs:6528-6547) ----

    #[test]
    fn cage_index_probes_match_the_csharp_arithmetic() {
        // Anchor at (12, 12): fx = 0, fy = 0 -> num = 12, num2 = 12.
        assert_eq!(big_animal_cage_frame(12, 12, 0, 0, 25), 6); // 2 * 3
        assert_eq!(small_animal_cage_frame(12, 12, 0, 0, 25), 16); // 4 * 4
        assert_eq!(water_animal_cage_frame(12, 12, 0, 0, 25), 24); // 6 * 4
                                                                   // Sub-cell frames fold back to the anchor: (13, 13) with fx = 18,
                                                                   // fy = 18 reads the same slot as (12, 12) with zero frames.
        assert_eq!(
            big_animal_cage_frame(13, 13, 18, 18, 25),
            big_animal_cage_frame(12, 12, 0, 0, 25)
        );
        // The % keeps the C# sign on negative products (truncating div).
        assert_eq!(water_animal_cage_frame(2, 3, 0, 0, 25), 1 * 1 % 25);
        assert_eq!(big_animal_cage_frame(30, 20, 0, 0, 25), 5 * 5 % 25); // 0
        assert_eq!(big_animal_cage_frame(36, 24, 0, 0, 25), 6 * 6 % 25); // 11
    }

    // ---- per-case cage draws: the table pick, row and stride ----

    /// Distinct value per (table, row) so a wrong pick/row is visible; the
    /// slot index rides in low bits so a wrong probe is visible too.
    fn fake_tables(table: CageAnim, row: i32, idx: i32) -> i32 {
        (table as i32 + 1) * 100 + row * 10 + idx % 10
    }

    fn cage(kind: u16, x: i32, y: i32, fx: i16, fy: i16) -> TileDrawData {
        cage_draw_data(kind, x, y, fx, fy, CAGE_FRAMES, fake_tables).unwrap()
    }

    #[test]
    fn big_cages_read_the_big_probe_and_stride_54() {
        // (12, 12) anchor -> big idx 6.
        let d = cage(275, 12, 12, 0, 0);
        assert_eq!(d.top, 2);
        assert_eq!(d.add_fr_y, (fake_tables(CageAnim::Bunny, 0, 6) * 54) as i16);
        assert_eq!(d.add_fr_x, 0);
        // Every big-group id picks its C# table (TileDrawing.cs:5674-5745).
        let picks: &[(u16, CageAnim)] = &[
            (275, CageAnim::Bunny),
            (359, CageAnim::Bunny),
            (599, CageAnim::Bunny),
            (605, CageAnim::Bunny),
            (550, CageAnim::Turtle),
            (551, CageAnim::Turtle),
            (542, CageAnim::Owl),
            (276, CageAnim::Squirrel),
            (413, CageAnim::Squirrel),
            (414, CageAnim::Squirrel),
            (606, CageAnim::Squirrel),
            (612, CageAnim::Squirrel),
            (277, CageAnim::Mallard),
            (278, CageAnim::Duck),
            (553, CageAnim::Grebe),
            (554, CageAnim::Seagull),
            (279, CageAnim::Bird),
            (358, CageAnim::Bird),
            (280, CageAnim::BlueBird),
            (281, CageAnim::RedBird),
            (632, CageAnim::Macaw),
            (640, CageAnim::Macaw),
            (643, CageAnim::Macaw),
            (644, CageAnim::Macaw),
            (645, CageAnim::Macaw),
            (296, CageAnim::Scorpion),
            (297, CageAnim::Scorpion),
            (309, CageAnim::Penguin),
            (558, CageAnim::Seahorse),
            (559, CageAnim::Seahorse),
        ];
        for &(kind, table) in picks {
            let d = cage(kind, 12, 12, 0, 0);
            assert_eq!(
                (d.top, d.add_fr_y),
                (2, (fake_tables(table, 0, 6) * 54) as i16),
                "kind {kind}"
            );
        }
    }

    #[test]
    fn pufferfish_splits_the_frame_into_column_and_row() {
        // case 710 (TileDrawing.cs:5746-5752): num9 = frame / 33.
        let d = cage(710, 12, 12, 0, 0);
        let f = fake_tables(CageAnim::Pufferfish, 0, 6);
        let n9 = f / 33;
        assert_eq!(d.add_fr_x, (108 * n9) as i16);
        assert_eq!(d.add_fr_y, ((f - n9 * 33) * 54) as i16);
        // A frame past the first 33-row block moves a full 108-wide column.
        let d2 = cage_draw_data(710, 12, 12, 0, 0, 25, |_, _, _| 40).unwrap();
        assert_eq!((d2.add_fr_x, d2.add_fr_y), (108, 7 * 54));
        let d3 = cage_draw_data(710, 12, 12, 0, 0, 25, |_, _, _| 33).unwrap();
        assert_eq!((d3.add_fr_x, d3.add_fr_y), (108, 0));
    }

    #[test]
    fn small_cages_read_the_small_probe_and_stride_36() {
        // (12, 12) anchor -> small idx 16.
        let picks: &[(u16, CageAnim, i32)] = &[
            (285, CageAnim::Snail, 0),
            (286, CageAnim::Snail2, 0),
            (582, CageAnim::Snail2, 0),
            (298, CageAnim::Frog, 0),
            (361, CageAnim::Frog, 0),
            (339, CageAnim::Grasshopper, 0),
            (362, CageAnim::Grasshopper, 0),
            (299, CageAnim::Mouse, 0),
            (363, CageAnim::Mouse, 0),
            (310, CageAnim::Worm, 0),
            (364, CageAnim::Worm, 0),
            (391, CageAnim::Worm, 0),
            (619, CageAnim::Worm, 0),
            (392, CageAnim::Slug, 0),
            (393, CageAnim::Slug, 1),
            (394, CageAnim::Slug, 2),
            (532, CageAnim::Maggot, 0),
            (533, CageAnim::Rat, 0),
            (538, CageAnim::Ladybug, 0),
            (544, CageAnim::Ladybug, 0),
            (629, CageAnim::Ladybug, 0),
            (555, CageAnim::WaterStrider, 0),
            (556, CageAnim::WaterStrider, 0),
        ];
        for &(kind, table, row) in picks {
            let d = cage(kind, 12, 12, 0, 0);
            assert_eq!(
                (d.top, d.add_fr_y),
                (2, (fake_tables(table, row, 16) * 36) as i16),
                "kind {kind}"
            );
        }
    }

    #[test]
    fn water_cages_read_the_water_probe_and_stride_36() {
        // (12, 12) anchor -> water idx 24.
        let picks: &[(u16, CageAnim, i32)] = &[
            (282, CageAnim::FishBowl, 0),
            (505, CageAnim::FishBowl, 0),
            (543, CageAnim::FishBowl, 0),
            (598, CageAnim::LavaFishBowl, 0),
            (568, CageAnim::FairyJar, 0),
            (569, CageAnim::FairyJar, 0),
            (570, CageAnim::FairyJar, 0),
            (288, CageAnim::Butterfly, 0),
            (295, CageAnim::Butterfly, 7),
            (360, CageAnim::Butterfly, 8),
            (580, CageAnim::Butterfly, 8),
            (620, CageAnim::Butterfly, 8),
            (521, CageAnim::Dragonfly, 0),
            (527, CageAnim::Dragonfly, 6),
        ];
        for &(kind, table, row) in picks {
            let d = cage(kind, 12, 12, 0, 0);
            assert_eq!(
                (d.top, d.add_fr_y),
                (2, (fake_tables(table, row, 24) * 36) as i16),
                "kind {kind}"
            );
        }
    }

    #[test]
    fn jellyfish_cages_use_the_small_probe() {
        // case 316-318 (TileDrawing.cs:5898-5907): SMALL index, row kind-316.
        for kind in 316..=318u16 {
            let d = cage(kind, 12, 12, 0, 0);
            assert_eq!(
                (d.top, d.add_fr_y),
                (
                    2,
                    (fake_tables(CageAnim::Jellyfish, kind as i32 - 316, 16) * 36) as i16
                ),
                "kind {kind}"
            );
        }
    }

    #[test]
    fn zero_tables_reproduce_the_static_cage_arms() {
        // The 92 cage ids of drawdata.rs's test_DW_1_1_group_lists_complete:
        // with the vanilla t=0 tables the rewrite is byte-equal to the
        // static arms (top 2, addFrY 0) — no pixel change until the
        // animator lands.
        const CAGES: &[u16] = &[
            275, 276, 277, 278, 279, 280, 281, 296, 297, 309, 358, 359, 413, 414, 542, 550, 551,
            553, 554, 558, 559, 599, 600, 601, 602, 603, 604, 605, 606, 607, 608, 609, 610, 611,
            612, 632, 640, 643, 644, 645, 710, 285, 286, 298, 299, 310, 339, 361, 362, 363, 364,
            391, 392, 393, 394, 532, 533, 538, 544, 555, 556, 582, 619, 629, 282, 505, 543, 598,
            568, 569, 570, 288, 289, 290, 291, 292, 293, 294, 295, 360, 580, 620, 521, 522, 523,
            524, 525, 526, 527, 316, 317, 318,
        ];
        assert_eq!(CAGES.len(), 92);
        for &kind in CAGES {
            let static_arm = draw_data(kind, 13, 7, 18, 18, 0, false);
            let rewritten = cage_draw_data(kind, 13, 7, 18, 18, CAGE_FRAMES, zero_tables).unwrap();
            assert_eq!(rewritten, static_arm, "kind {kind}");
        }
        // Non-cage kinds are left to the draw_data table.
        assert!(cage_draw_data(21, 0, 0, 0, 0, CAGE_FRAMES, zero_tables).is_none());
        assert!(cage_draw_data(15, 0, 0, 0, 0, CAGE_FRAMES, zero_tables).is_none());
    }

    // ---- the chest rewrite (TileDrawing.cs:4289-4330) ----

    fn chest_dd(kind: u16, fx: i16, fy: i16, frame: Option<i32>) -> TileDrawData {
        let mut d = draw_data(kind, 0, 0, fx, fy, 0, false);
        chest_draw_rewrite(kind, fx, fy, frame, &mut d);
        d
    }

    #[test]
    fn closed_chest_folds_to_the_sheet_origin_per_sub_cell() {
        // Top-left cell: nothing moves.
        let d = chest_dd(21, 0, 0, None);
        assert_eq!((d.add_fr_x, d.add_fr_y, d.height), (0, 0, 16));
        // On the 18-px framing grid `num * 18 - fx` / `num2 * 18 - fy` are
        // both 0 — the fold's visible effects are the bottom-row height,
        // the open offsets below, and the addFrY ASSIGNMENT (which discards
        // the table's `tileFrame * 38` default, see the next test).
        let d = chest_dd(21, 18, 0, None);
        assert_eq!((d.add_fr_x, d.add_fr_y, d.height), (0, 0, 16));
        // Bottom row (fy 18): tileHeight 18 (:4322-4325) — matches the
        // static case-21 arm's fy == 18 rule.
        let d = chest_dd(21, 18, 18, None);
        assert_eq!((d.add_fr_x, d.add_fr_y, d.height), (0, 0, 18));
        // Containers2 (467) rides the same set (TileID.cs:570).
        let d = chest_dd(467, 0, 18, None);
        assert_eq!(d.height, 18);
        assert!(is_basic_chest(21) && is_basic_chest(467));
        assert!(!is_basic_chest(441)); // BasicChestFake is a different set
    }

    #[test]
    fn open_chest_frames_shift_the_row_by_38_and_76() {
        // frame 1 -> +38, frame 2 -> +76 (:4311-4320), on every sub-cell.
        for (frame, shift) in [(Some(1), 38), (Some(2), 76), (Some(0), 0), (None, 0)] {
            let top = chest_dd(21, 0, 0, frame);
            assert_eq!(top.add_fr_y, shift, "frame {frame:?} top row");
            assert_eq!(top.height, 16);
            let bottom = chest_dd(21, 18, 18, frame);
            assert_eq!(bottom.add_fr_y, shift, "frame {frame:?} bottom row");
            assert_eq!(bottom.height, 18);
        }
        // A frame value outside {1, 2} adds nothing (the C# has no else).
        assert_eq!(chest_dd(21, 0, 0, Some(3)).add_fr_y, 0);
    }

    #[test]
    fn chest_rewrite_replaces_the_animation_row_default() {
        // The C# ASSIGNS addFrY (it does not +=): the GetTileDrawData
        // default `tileFrame * 38` is discarded for chests.
        let mut d = draw_data(21, 0, 0, 0, 0, 3, false);
        assert_eq!(d.add_fr_y, 114); // the table default before the pass
        chest_draw_rewrite(21, 0, 0, Some(1), &mut d);
        assert_eq!(d.add_fr_y, 38);
        // Non-chest kinds keep the table's result untouched.
        let mut d = draw_data(15, 0, 0, 0, 0, 3, false);
        let before = d;
        chest_draw_rewrite(15, 0, 0, Some(2), &mut d);
        assert_eq!(d, before);
    }

    #[test]
    fn post_pass_dispatches_cage_and_chest_disjointly() {
        // A cage cell goes through the cage rewrite only.
        let mut d = draw_data(275, 12, 12, 0, 0, 0, false);
        furniture_post_pass(275, 12, 12, 0, 0, CAGE_FRAMES, fake_tables, None, &mut d);
        assert_eq!(d.add_fr_y, (fake_tables(CageAnim::Bunny, 0, 6) * 54) as i16);
        // A chest cell goes through the chest rewrite only.
        let mut d = draw_data(21, 0, 0, 18, 18, 0, false);
        furniture_post_pass(21, 0, 0, 18, 18, CAGE_FRAMES, fake_tables, Some(2), &mut d);
        assert_eq!((d.add_fr_y, d.height), (76, 18));
        // Anything else is untouched.
        let mut d = draw_data(15, 0, 0, 0, 18, 0, false);
        let before = d;
        furniture_post_pass(15, 0, 0, 0, 18, CAGE_FRAMES, fake_tables, Some(1), &mut d);
        assert_eq!(d, before);
    }

    // ---- per-case tests over the STATIC furniture arms (draw_data) ----

    #[test]
    fn tables_chests_containers_are_18_on_the_seam_row() {
        // case 14/21/411/467/469 (TileDrawing.cs:4943-4952).
        for kind in [14, 21, 411, 467, 469] {
            assert_eq!(
                draw_data(kind, 0, 0, 0, 18, 0, false).height,
                18,
                "k {kind}"
            );
            assert_eq!(draw_data(kind, 0, 0, 0, 0, 0, false).height, 16, "k {kind}");
            assert_eq!(
                draw_data(kind, 0, 0, 0, 54, 0, false).height,
                16,
                "k {kind}"
            );
        }
    }

    #[test]
    fn chairs_and_toilets_are_18_on_the_40_stride_seam() {
        // case 15/497 (TileDrawing.cs:4953-4959): fy % 40 == 18.
        for kind in [15, 497] {
            assert_eq!(draw_data(kind, 0, 0, 0, 18, 0, false).height, 18);
            assert_eq!(draw_data(kind, 0, 0, 0, 58, 0, false).height, 18);
            assert_eq!(draw_data(kind, 0, 0, 0, 38, 0, false).height, 16);
            assert_eq!(draw_data(kind, 0, 0, 0, 0, 0, false).height, 16);
        }
    }

    #[test]
    fn beds_unwind_the_2016_style_stride() {
        // case 79 (TileDrawing.cs:4909-4916): height 18 always; style block
        // n23 = fy / 2016 moves addFrX by 144 and unwinds addFrY.
        let d = draw_data(79, 0, 0, 0, 0, 0, false);
        assert_eq!((d.height, d.add_fr_x, d.add_fr_y), (18, 0, 0));
        let d = draw_data(79, 0, 0, 0, 2016, 0, false);
        assert_eq!((d.height, d.add_fr_x, d.add_fr_y), (18, 144, -2016));
        let d = draw_data(79, 0, 0, 0, 2016, 1, false);
        assert_eq!(d.add_fr_y, 38 - 2016);
        // Bathtubs (90) share the stride without the height (:4917-4923).
        let d = draw_data(90, 0, 0, 0, 2016, 0, false);
        assert_eq!((d.height, d.add_fr_x, d.add_fr_y), (16, 144, -2016));
    }

    #[test]
    fn workbenches_unwind_the_stride_from_frame_x() {
        // case 18 (TileDrawing.cs:4924-4930): n21 = fx / 2016.
        let d = draw_data(18, 0, 0, 2016, 0, 0, false);
        assert_eq!((d.add_fr_x, d.add_fr_y), (-2016, 20));
        let d = draw_data(18, 0, 0, 2016, 0, 2, false);
        assert_eq!(d.add_fr_y, 76 + 20);
        let d = draw_data(18, 0, 0, 0, 0, 0, false);
        assert_eq!((d.add_fr_x, d.add_fr_y), (0, 0));
    }

    #[test]
    fn sinks_and_crates_are_18_on_the_38_stride_seam() {
        // case 172/376 (TileDrawing.cs:4960-4966): fy % 38 == 18.
        for kind in [172, 376] {
            assert_eq!(draw_data(kind, 0, 0, 0, 18, 0, false).height, 18);
            assert_eq!(draw_data(kind, 0, 0, 0, 56, 0, false).height, 18);
            assert_eq!(draw_data(kind, 0, 0, 0, 36, 0, false).height, 16);
        }
    }

    #[test]
    fn anvil_furnace_beam_group_is_18_tall() {
        // case 16-group (TileDrawing.cs:4880-4908).
        for kind in [
            16, 17, 26, 32, 69, 72, 77, 124, 137, 138, 352, 462, 487, 488, 574, 575, 576, 577, 578,
            664, 695, 704, 712, 713, 714, 715, 716,
        ] {
            let d = draw_data(kind, 0, 0, 0, 0, 0, false);
            assert_eq!((d.width, d.height, d.top), (16, 18, 0), "kind {kind}");
        }
    }

    #[test]
    fn pot_statue_dresser_group_sits_2_low() {
        // case 28-group (TileDrawing.cs:5258-5275): tileTop = 2.
        for kind in [
            28, 105, 470, 475, 506, 547, 548, 552, 560, 597, 613, 621, 622, 623, 653, 699,
        ] {
            let d = draw_data(kind, 0, 0, 0, 0, 0, false);
            assert_eq!((d.width, d.height, d.top), (16, 16, 2), "kind {kind}");
        }
    }
}
