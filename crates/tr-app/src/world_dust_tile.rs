//! W4-d54 batch 3 — `WorldGen.KillTile_MakeTileDust`
//! (`WorldGen.cs:66687-69169`, sites 69089-69166), the dust-amount gate
//! (`KillTile_GetTileDustAmount`, `:66597-66685`) and the tile→dust type
//! chain that feeds all 14 spawn arms.
//!
//! The pick-break / tile-replacement dust: the C# resolves a dust TYPE
//! from the tile (a long overwrite-if chain — later arms win, several
//! with `genRand` rolls), then the first matching spawn arm fires. The
//! `genRand` rolls ride the port's `frame_rng` stand-in (the caller
//! passes a snapshot); the spawn blocks + the type-19-style-30
//! `Main.rand.Next(3)` (`:67402`) + the type-139 `Main.rand.Next(4)`
//! (`:69158`) ride `UpdatePlayers`.
//!
//! Callers (the C# order):
//! - `KillTile` (`:63699-63703`): `KillTile_GetTileDustAmount(fail,
//!   tile)` calls of this body, AFTER `KillTile_PlaySounds` and BEFORE
//!   the drops. Wired in `editing.rs::break_tile` with the pre-kill
//!   tile snapshot (the C# `tile` reference is still intact at `:63699`
//!   — the clear happens in the drop half).
//! - `ReplaceTile` (`:62570-62574`): the same loop with `fail: false`
//!   — no live port caller yet (the smart-cursor replacement lane).
//!
//! The tree-ground scans (types 5/323, `:68163-68259`) read the LIVE
//! world below the trunk; the killed cell itself is never the scan's
//! answer (a trunk is not `tileSolid`, so the C# walk passes it while
//! still active and the port's cleared cell passes it the same way).
//! The type-5 scan's DUPLICATE `frameX == 44 && frameY 132..176` arm
//! (`:68179-68186`) increments the column TWICE — the C# quirk, kept.

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx, DUST_SENTINEL};
use tr_core::tile::Tile;
use tr_core::tiledata as td;
use tr_core::{UnifiedRandom, World, TILE_PX};

/// `TorchID.Dust` (`ID/TorchID.cs:179-184`) — the 24-row torch style →
/// dust type table (`TorchID.Count = 24`, `:185`).
const TORCH_DUST: [i32; 24] = [
    6, 59, 60, 61, 62, 63, 64, 65, 75, 135, 158, 169, 156, 234, 66, 242, 293, 294, 295, 296,
    297, 298, 307, 310,
];

/// `WorldGen.beachDistance` (`WorldGen.cs:86914`, the drawdata.rs:1053
/// precedent) — the type-529 ground default's beach band.
const BEACH_DISTANCE: i32 = 380;

/// `WorldGen.SolidTile(i, j)` (`WorldGen.cs:70111-70131`) — the
/// tile-only solidity test the type-518 fountain arm probes (the
/// growth.rs:340 copy's shape, plus the `!inActive()` half).
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    match world.tile(x, y) {
        Some(t) => {
            t.active()
                && td::TILE_SOLID.has(t.kind)
                && !td::TILE_SOLID_TOP.has(t.kind)
                && !t.half_brick()
                && t.slope() == 0
                && !t.inactive()
        }
        // The C# null tile returns TRUE (`:70115-70118`); the probe
        // only reads j-1 inside the world, so the arm never fires on
        // the null edge in practice — kept for the 1:1 shape.
        None => true,
    }
}

/// `WorldGen.KillTile_GetTileDustAmount` (`WorldGen.cs:66597-66685`):
/// the per-kind dust count (the fail tick drops 10 → 3 on the default
/// arm; the zero-list kinds never puff).
pub(crate) fn tile_dust_amount(fail: bool, kind: u16, frame_x: i16) -> i32 {
    match kind {
        654 | 630 | 631 => 0,
        231 => {
            if !fail {
                6
            } else {
                3
            }
        }
        481 | 482 | 483 => 3,
        541 => 1,
        549 => 2,
        125 | 138 | 172 | 287 | 300..=308 | 354 | 355 | 373..=376 | 461 | 484 | 499 | 564
        | 565 | 593 | 594 | 617 | 664 | 665 | 699 | 709 | 711..=716 => 0,
        534 | 536 | 539 | 625 | 627 => {
            if !fail {
                5
            } else {
                3
            }
        }
        184 => {
            if i32::from(frame_x) / 22 >= 5 {
                5
            } else if !fail {
                10
            } else {
                3
            }
        }
        _ => {
            if !fail {
                10
            } else {
                3
            }
        }
    }
}

/// The tile→dust type chain (`WorldGen.cs:66689-69058`), source order —
/// a later arm replaces an earlier one. `rng` is `Main.rand`
/// (`UpdatePlayers`; only the type-19-style-30 arm draws), `gen` is the
/// `genRand` stand-in. Returns the dust type; a negative value spawns
/// nothing (`:69085`'s `num >= 0` gate).
#[allow(clippy::too_many_lines)]
pub(crate) fn tile_dust_type(
    world: &World,
    i: i32,
    j: i32,
    t: &Tile,
    rng: &mut UnifiedRandom,
    gen: &mut UnifiedRandom,
) -> i32 {
    let kind = t.kind;
    let fx = i32::from(t.frame_x);
    let fy = i32::from(t.frame_y);
    let mut num = 0i32;
    // `:66690-66757` — the head ifs + the 208/408/669 else-if chain.
    if kind == 216 {
        num = -1;
    }
    if kind == 324 {
        num = if fy != 0 { 281 + fx / 18 } else { 280 };
    }
    if kind == 216 {
        num = -1; // the C# repeats the arm (`:66698`), kept
    }
    if kind == 335 {
        num = -1;
    }
    if kind == 338 {
        num = -1;
    }
    if kind == 0 {
        num = 0;
    }
    if kind == 192 {
        num = 3;
    }
    if kind == 208 {
        num = 126;
    } else if matches!(kind, 408 | 409) {
        num = 265;
    } else if kind == 669 {
        num = 314;
    } else if kind == 670 {
        num = 315;
    } else if kind == 671 {
        num = 316;
    } else if kind == 672 {
        num = 317;
    } else if kind == 673 {
        num = 318;
    } else if kind == 674 {
        num = 319;
    } else if kind == 675 {
        num = 320;
    } else if kind == 676 {
        num = 321;
    }
    // `:66758-66785` — the 16/415/416/417/418/474 else-if chain.
    if kind == 16 {
        num = 1;
        if fx >= 36 {
            num = 82;
        }
    } else if matches!(kind, 415 | 500) {
        num = 6;
    } else if matches!(kind, 416 | 501) {
        num = 61;
    } else if matches!(kind, 417 | 502) {
        num = 242;
    } else if matches!(kind, 418 | 503) {
        num = 135;
    } else if kind == 474 {
        num = 18;
    }
    // `:66786-66789` — the stone family (Main.tileStone = TILE_STONE).
    if matches!(
        kind,
        1 | 17 | 38 | 39 | 41 | 43 | 44 | 481 | 482 | 483 | 48 | 85 | 90 | 92 | 96 | 97 | 99
            | 117 | 130 | 131 | 132 | 135 | 142 | 143 | 144 | 210 | 207 | 235 | 247 | 272 | 273
            | 283 | 410 | 480 | 509 | 618 | 657 | 658 | 677 | 678 | 679 | 720 | 721 | 725 | 733
    ) || td::TILE_STONE.has(kind)
    {
        num = 1;
    }
    if kind == 753 {
        num = -1;
    }
    if kind == 379 {
        num = 257;
    }
    if matches!(kind, 751 | 752) {
        num = -1;
    }
    if kind == 311 {
        num = 207;
    }
    if kind == 312 {
        num = 208;
    }
    if kind == 313 {
        num = 209;
    }
    if kind == 104 {
        num = -1;
    }
    if matches!(kind, 95 | 98 | 100 | 174 | 173) {
        num = 6;
    }
    if matches!(kind, 30 | 86 | 94 | 106 | 114 | 124 | 128 | 269) {
        num = 7;
    }
    if kind == 372 {
        num = 242;
    }
    if kind == 646 {
        num = 29;
    }
    if kind == 49 {
        num = 29;
    }
    if kind == 371 {
        num = 243;
    }
    if kind == 334 {
        num = 7;
    }
    // `:66846-67078` — the big switch.
    match kind {
        10 | 11 | 87 | 89 | 93 | 139 | 209 | 319 | 320 | 386 | 387 | 390 | 405 | 406 | 411
        | 412 | 419..=425 | 428 | 429 | 441 | 442 | 445..=453 | 455..=457 | 462..=466 | 468
        | 476 | 486 | 487 | 489..=491 | 493 | 494 | 497 | 510 | 511 | 520..=527 | 531 | 545
        | 547 | 548 | 560 | 564 | 565 | 567 | 572 | 579 | 591..=594 | 613 | 614 | 621..=624
        | 630 | 631 | 656 | 700 | 701 | 726 => num = -1,
        668 => num = 0,
        407 => num = 10,
        454 => num = 139,
        41 | 481 | 677 => num = 275,
        43 | 482 | 678 => num = 276,
        44 | 483 | 679 => num = 277,
        473 => num = 82,
        472 | 546 | 557 => num = 8,
        498 => num = 30,
        517 | 687 => num = 258,
        535 | 689 => num = 299,
        537 | 690 => num = 300,
        540 | 688 => num = 301,
        626 | 691 => num = 305,
        184 => {
            let num2 = fx / 22;
            num = match num2 {
                5 => 258,
                6 => 299,
                7 => 300,
                8 => 301,
                9 => 305,
                10 => 267,
                _ => 93 + num2,
            };
        }
        515 => num = 96,
        516 => num = 97,
        514 => num = 95,
        513 => num = 94,
        512 => num = 93,
        541 => num = 226,
        590 => num = 1,
        583 => num = if gen.next_max(10) != 0 { 1 } else { 87 },
        584 => num = if gen.next_max(10) != 0 { 1 } else { 86 },
        585 => num = if gen.next_max(10) != 0 { 1 } else { 88 },
        586 => num = if gen.next_max(10) != 0 { 1 } else { 89 },
        587 => num = if gen.next_max(10) != 0 { 1 } else { 90 },
        588 => num = if gen.next_max(10) != 0 { 1 } else { 91 },
        589 => num = if gen.next_max(10) != 0 { 1 } else { 138 },
        595 | 596 | 615 | 616 => num = 78,
        633 => num = if gen.next_max(6) != 0 { 237 } else { 36 },
        637 | 638 => num = 237,
        634 => {
            num = if gen.next_max(10) != 0 { 36 } else { 31 };
            if gen.next_max(12) == 0 {
                num = 6;
            }
        }
        _ => {}
    }
    // `:67079-67082` — the moss family (Main.tileMoss = TILE_MOSS).
    if td::TILE_MOSS.has(kind) {
        num = match kind {
            381 => 258,
            534 => 299,
            536 => 300,
            539 => 301,
            625 => 305,
            627 => 267,
            _ => i32::from(kind) - 179 + 93,
        };
    }
    // `:67083-67103` — the 240 statue frame decode.
    if kind == 240 {
        let mut num3 = fx / 54;
        if fy >= 54 {
            num3 += 36 * (fy / 54);
        }
        num = 7;
        if matches!(num3, 16 | 17) {
            num = 26;
        }
        if (46..=49).contains(&num3) {
            num = -1;
        }
        if num3 == 100 {
            num = -1;
        }
    }
    if kind == 241 {
        num = 1;
    }
    if kind == 242 {
        num = -1;
    }
    // `:67112-67121` — the 529 palm trunk's ground read.
    if kind == 529 {
        let ground = world.tile(i, j + 1).map_or(0, |g| g.kind);
        num = match ground {
            116 => 47,
            234 => 125,
            112 => 17,
            _ => {
                if i >= BEACH_DISTANCE && i <= world.width - BEACH_DISTANCE {
                    289
                } else {
                    290
                }
            }
        };
    }
    if kind == 356 {
        num = -1;
    }
    if kind == 663 {
        num = -1;
    }
    if kind == 351 {
        num = -1;
    }
    if kind == 246 {
        num = -1;
    }
    if kind == 36 {
        num = -1;
    }
    if kind == 365 {
        num = 239;
    }
    if kind == 366 {
        num = 30;
    }
    if kind == 504 {
        num = -1;
    }
    if matches!(kind, 357 | 367 | 561) {
        num = 236;
    }
    if matches!(kind, 368 | 369 | 576) {
        num = 240;
    }
    if kind == 170 {
        num = 196;
    }
    if kind == 315 {
        num = 225;
    }
    if kind == 641 {
        num = if gen.next_max(2) != 0 { 161 } else { 243 };
    }
    if kind == 659 {
        num = 308;
    }
    if kind == 667 {
        num = 308;
    }
    if kind == 346 {
        num = 128;
    }
    if kind == 347 {
        num = 117;
    }
    if kind == 348 {
        num = 42;
    }
    if kind == 350 {
        num = 226;
    }
    if kind == 370 {
        num = if gen.next_max(2) != 0 { 23 } else { 6 };
    }
    if kind == 171 {
        num = if gen.next_max(2) != 0 { -1 } else { 196 };
    }
    if matches!(kind, 326 | 327 | 345 | 708 | 458 | 459) {
        num = 13;
    }
    if kind == 336 {
        num = 6;
    }
    if kind == 340 {
        num = 75;
    }
    if kind == 341 {
        num = 65;
    }
    if kind == 342 {
        num = 135;
    }
    if kind == 343 {
        num = 169;
    }
    if kind == 344 {
        num = 156;
    }
    if matches!(kind, 328 | 329 | 507 | 508) {
        num = 13;
    }
    if kind == 562 {
        num = 273;
    }
    if kind == 571 {
        num = 40;
    }
    if kind == 563 {
        num = 273;
    }
    if kind == 330 {
        num = 9;
    }
    if kind == 331 {
        num = 11;
    }
    if kind == 332 {
        num = 19;
    }
    if kind == 333 {
        num = 11;
    }
    if kind == 722 {
        num = 57;
    }
    if kind == 734 {
        num = 0;
    }
    if kind == 735 {
        num = 10;
    }
    if kind == 736 {
        num = 277;
    }
    if kind == 737 {
        num = 26;
    }
    if kind == 738 {
        num = 277;
    }
    if kind == 739 {
        num = 68;
    }
    if kind == 740 {
        num = 1;
    }
    if kind == 741 {
        num = 78;
    }
    // `:67334-67363` — the 742-750 switch.
    match kind {
        750 => num = 26,
        749 => num = 1,
        748 => num = 13,
        747 => num = 10,
        746 => num = 148,
        745 => num = 26,
        744 => num = 78,
        742 => num = 96,
        743 => num = 236,
        _ => {}
    }
    if kind == 101 {
        num = -1;
    }
    // `:67368-67423` — the 19 bar frameY/18 table (style 30 draws
    // Main.rand.Next(3), `:67402`).
    if kind == 19 {
        num = match fy / 18 {
            0 => 7,
            1 => 77,
            2 => 78,
            3 => 79,
            4 => 26,
            5 => 126,
            6 => 275,
            7 => 277,
            8 => 276,
            9 => 1,
            10 | 11 | 12 => 214,
            13 => 109,
            14 => 13,
            15 => 189,
            16 => 191,
            17 => 215,
            18 => 26,
            19 => 214,
            20 => 4,
            21 => 10,
            22 => 32,
            23 => 78,
            24 => 147,
            25 => 40,
            26 => 226,
            27 => 23,
            28 => 240,
            29 => 236,
            30 => 68 + rng.next_max(3),
            31 => 10,
            32 => 78,
            33 => 148,
            34 => 5,
            35 => 80,
            37 => 18,
            38 => 6,
            39 => 61,
            40 => 242,
            41 => 135,
            42 => 287,
            44 => 273,
            45 | 46 => 243,
            47 => 36,
            48 => 226,
            49 => 16,
            50 => 309,
            _ => 1,
        };
    }
    // `:67424-67428` — the 79 bed frameY/36 ladder.
    if kind == 79 {
        let num4 = fy / 36;
        num = match num4 {
            0 => 7,
            1 => 77,
            2 => 78,
            3 => 79,
            4 => 126,
            8 => 109,
            _ if num4 < 9 => 1,
            _ => -1,
        };
    }
    // `:67429-67481` — the 18 platform frameX/36 switch.
    if kind == 18 {
        num = match fx / 36 {
            0 => 7,
            1 => 77,
            2 => 78,
            3 => 79,
            4 => 26,
            5 => 40,
            6 => 5,
            7 => 26,
            8 => 4,
            9 => 126,
            10 => 148,
            11 | 12 | 13 => 1,
            14 => 109,
            15 => 126,
            _ => -1,
        };
    }
    if matches!(kind, 14 | 87 | 88 | 469) {
        num = -1;
    }
    // `:67486-67512` — the gemspark/gemcorn families.
    if (255..=261).contains(&kind) {
        let num5 = i32::from(kind) - 255;
        num = 86 + num5;
        if num5 == 6 {
            num = 138;
        }
    }
    if (262..=268).contains(&kind) {
        let num6 = i32::from(kind) - 262;
        num = 86 + num6;
        if num6 == 6 {
            num = 138;
        }
    }
    if kind == 178 {
        let num7 = fx / 18;
        num = 86 + num7;
        if num7 == 6 {
            num = 138;
        }
    }
    // `:67513-67530` — the 440 gemcorn frame decode.
    if kind == 440 {
        num = match fx / 54 {
            0 => 90,
            1 => 88,
            2 => 89,
            3 => 87,
            4 => 86,
            5 => 91,
            6 => 138,
            _ => -1,
        };
        if fy < 54 {
            num = -1;
        }
    }
    // `:67531-67572` — the stained-glass / 496 / 549 / 552 switch.
    match kind {
        426 | 427 | 727 => num = 90,
        430 | 435 | 728 => num = 89,
        431 | 436 | 729 => num = 88,
        432 | 437 | 730 => num = 87,
        433 | 438 | 731 => num = 86,
        434 | 439 | 732 => num = 91,
        496 => num = 109,
        549 => num = 3,
        552 => num = 32,
        _ => {}
    }
    // `:67573-67576` — the 186 graveyard-stone frameX ladder.
    if kind == 186 {
        num = if fx <= 360 {
            26
        } else if fx <= 846 {
            1
        } else if fx <= 954 {
            9
        } else if fx <= 1062 {
            11
        } else if fx <= 1170 {
            10
        } else if fx > 1332 {
            if fx > 1386 {
                80
            } else {
                10
            }
        } else {
            0
        };
    }
    // `:67577-67627` — the 187 ladder (the second `<= 1132` arm is
    // DEAD in the C# — the first one always wins — kept for the shape).
    if kind == 187 {
        if fx <= 144 {
            num = 1;
        } else if fx <= 306 {
            num = 38;
        } else if fx <= 468 {
            num = 36;
        } else if fx <= 738 {
            num = 30;
        } else if fx <= 970 {
            num = 1;
        } else if fx <= 1132 {
            num = 148;
        } else if fx <= 1132 {
            num = 155; // dead arm (`:67603`), kept
        } else if fx <= 1348 {
            num = 1;
        } else if fx <= 1564 {
            num = 0;
        } else if fx <= 1890 {
            num = 250;
        } else if fx <= 2196 {
            num = 240;
        } else if fx <= 2520 {
            num = 236;
        }
    }
    // `:67628-67663` — the 647 banner frameX/54 ladder.
    if kind == 647 {
        let num8 = fx / 54;
        if num8 < 7 {
            num = 26;
        } else if num8 < 16 {
            num = 1;
        } else if num8 < 18 {
            num = 9;
        } else if num8 < 20 {
            num = 11;
        } else if num8 < 22 {
            num = 10;
        } else if num8 < 26 {
            num = 7;
        } else if num8 < 32 {
            num = 80;
        } else if num8 < 35 {
            num = 80;
        }
    }
    // `:67664-67723` — the 648 banner 2-D ladder.
    if kind == 648 {
        let num9 = fx / 54 + fy / 36 * 35;
        if num9 < 3 {
            num = 1;
        } else if num9 < 6 {
            num = 38;
        } else if num9 < 9 {
            num = 36;
        } else if num9 < 14 {
            num = 30;
        } else if num9 < 17 {
            num = 1;
        } else if num9 < 18 {
            num = 1;
        } else if num9 < 21 {
            num = 148;
        } else if num9 < 29 {
            num = 155;
        } else if num9 < 35 {
            num = 287;
        } else if num9 < 41 {
            num = 240;
        } else if num9 < 47 {
            num = 236;
        } else if num9 < 50 {
            num = 0;
        } else if num9 < 52 {
            num = 2;
        } else if num9 < 55 {
            num = 26;
        }
    }
    if kind == 706 {
        num = 1;
    }
    if kind == 105 {
        num = 1;
        if (1548..=1654).contains(&fx) && fy < 54 {
            num = 148;
        }
    }
    if kind == 349 {
        num = 1;
    }
    if matches!(kind, 337 | 506) {
        num = 1;
    }
    // `:67744-67839` — the 239 statue frameX/18 if-ladder.
    if kind == 239 {
        let num10 = fx / 18;
        match num10 {
            0 => num = 9,
            1 => num = 81,
            2 => num = 8,
            3 => num = 82,
            4 => num = 11,
            5 => num = 83,
            6 => num = 10,
            7 => num = 84,
            8 => num = 14,
            9 => num = 23,
            10 => num = 25,
            11 => num = 48,
            12 => num = 144,
            13 => num = 49,
            14 => num = 145,
            15 => num = 50,
            16 => num = 146,
            17 => num = 128,
            18 => num = 84,
            19 => num = 117,
            20 => num = 42,
            21 => num = -1,
            22 => num = 265,
            _ => {}
        }
    }
    // `:67840-67974` — the 185 chest/banner two-branch ladder.
    if kind == 185 {
        if fy == 18 {
            let num11 = fx / 36;
            if num11 < 6 {
                num = 1;
            } else if num11 < 16 {
                num = 26;
            } else if num11 == 16 {
                num = 9;
            } else if num11 == 17 {
                num = 11;
            } else if num11 == 18 {
                num = 10;
            } else if num11 == 19 {
                num = 86;
            } else if num11 == 20 {
                num = 87;
            } else if num11 == 21 {
                num = 88;
            } else if num11 == 22 {
                num = 89;
            } else if num11 == 23 {
                num = 90;
            } else if num11 == 24 {
                num = 91;
            } else if num11 < 31 {
                num = 80;
            } else if num11 < 33 {
                num = 7;
            } else if num11 < 34 {
                num = 8;
            } else if num11 < 38 {
                num = 30;
            } else if num11 < 41 {
                num = 1;
            } else if num11 < 47 {
                num = 287;
            } else if num11 < 53 {
                num = 240;
            } else if num11 < 59 {
                num = 236;
            }
        } else {
            let num12 = fx / 18;
            if num12 < 6 {
                num = 1;
            } else if num12 < 12 {
                num = 0;
            } else if num12 < 28 {
                num = 26;
            } else if num12 < 33 {
                num = 1;
            } else if num12 < 36 {
                num = 0;
            } else if num12 < 48 {
                num = 80;
            } else if num12 < 54 {
                num = 30;
            } else if num12 < 60 {
                num = 287;
            } else if num12 < 66 {
                num = 240;
            } else if num12 < 72 {
                num = 236;
            } else if num12 < 73 {
                num = 0;
            } else if num12 < 77 {
                num = 32;
            }
        }
    }
    // `:67975-68066` — the 649 banner 2-D ladder.
    if kind == 649 {
        let num13 = fx / 36 + fy / 18 * 53;
        if num13 < 6 {
            num = 1;
        } else if num13 < 16 {
            num = 26;
        } else if num13 == 16 {
            num = 9;
        } else if num13 == 17 {
            num = 11;
        } else if num13 == 18 {
            num = 10;
        } else if num13 == 19 {
            num = 86;
        } else if num13 == 20 {
            num = 87;
        } else if num13 == 21 {
            num = 88;
        } else if num13 == 22 {
            num = 89;
        } else if num13 == 23 {
            num = 90;
        } else if num13 == 24 {
            num = 91;
        } else if num13 < 31 {
            num = 80;
        } else if num13 < 33 {
            num = 7;
        } else if num13 < 34 {
            num = 8;
        } else if num13 < 38 {
            num = 30;
        } else if num13 < 41 {
            num = 1;
        } else if num13 < 47 {
            num = 287;
        } else if num13 < 53 {
            num = 240;
        } else if num13 < 59 {
            num = 236;
        } else if num13 < 62 {
            num = 0;
        } else if num13 < 64 || (65..=67).contains(&num13) {
            num = 32;
        } else if num13 == 64 {
            num = 40;
        }
    }
    // `:68067-68135` — the 650 banner ladder + switch tail.
    if kind == 650 {
        let num14 = fx / 18;
        if num14 < 6 {
            num = 1;
        } else if num14 < 12 {
            num = 0;
        } else if num14 < 28 {
            num = 26;
        } else if num14 < 33 {
            num = 1;
        } else if num14 < 36 {
            num = 0;
        } else if num14 < 48 {
            num = 80;
        } else if num14 < 54 {
            num = 30;
        } else if num14 < 60 {
            num = 287;
        } else if num14 < 66 {
            num = 240;
        } else if num14 < 72 {
            num = 236;
        } else if num14 < 73 {
            num = 0;
        } else {
            match num14 {
                73 | 74 | 76 | 78 | 79 | 80 | 81 => num = 32,
                75 | 77 => num = 40,
                _ => {}
            }
        }
    }
    if kind == 237 {
        num = 148;
    }
    if kind == 157 {
        num = 77;
    }
    if matches!(kind, 158 | 232 | 383 | 575) {
        num = 78;
    }
    if kind == 159 {
        num = 78;
    }
    if kind == 15 {
        num = -1;
    }
    if kind == 191 {
        num = 7;
    }
    // `:68160-68234` — the type-5 tree's ground scan (the DUPLICATE
    // 44/132..176 arm increments twice — the C# quirk, kept).
    if kind == 5 {
        num = 7;
        if i > 5 && i < world.width - 5 {
            let mut num15 = i;
            let mut k = j;
            if fx == 66 && fy <= 45 {
                num15 += 1;
            }
            if fx == 88 && (66..=110).contains(&fy) {
                num15 -= 1;
            }
            if fx == 22 && (132..=176).contains(&fy) {
                num15 -= 1;
            }
            if fx == 44 && (132..=176).contains(&fy) {
                num15 += 1;
            }
            if fx == 44 && (132..=176).contains(&fy) {
                num15 += 1; // the duplicate arm (`:68183-68186`)
            }
            if fx == 44 && fy >= 198 {
                num15 += 1;
            }
            if fx == 66 && fy >= 198 {
                num15 -= 1;
            }
            while let Some(cell) = world.tile(num15, k) {
                if cell.active() && td::TILE_SOLID.has(cell.kind) {
                    break;
                }
                k += 1;
            }
            if let Some(cell) = world.tile(num15, k) {
                if cell.active() {
                    match cell.kind {
                        23 | 661 => num = 77,
                        60 => num = 78,
                        70 => num = 26,
                        109 => num = 79,
                        199 | 662 => num = 121,
                        147 => num = 122,
                        _ => {}
                    }
                }
            }
        }
    }
    // `:68235-68260` — the 323 gem-tree ground scan.
    if kind == 323 {
        num = 215;
        if i > 5 && i < world.width - 5 {
            let mut l = j;
            while let Some(cell) = world.tile(i, l) {
                if cell.active() && td::TILE_SOLID.has(cell.kind) {
                    break;
                }
                l += 1;
            }
            if let Some(cell) = world.tile(i, l) {
                if cell.active() {
                    match cell.kind {
                        234 => num = 121,
                        116 => num = 79,
                        112 => num = 77,
                        _ => {}
                    }
                }
            }
        }
    }
    // `:68261-68278` — the 137 dart-trap frameY/18 switch (the default
    // and case-5 arms both write 1; only 1-4 write 148).
    if kind == 137 {
        num = match fy / 18 {
            1..=4 => 148,
            _ => 1,
        };
    }
    if kind == 443 {
        num = 1;
    }
    if kind == 444 {
        num = -1;
    }
    if kind == 212 {
        num = -1;
    }
    if kind == 213 {
        num = 129;
    }
    if kind == 214 {
        num = 1;
    }
    if kind == 215 {
        num = -6;
    }
    if kind == 325 {
        num = 81;
    }
    if kind == 251 {
        num = 189;
    }
    if kind == 252 {
        num = 190;
    }
    if kind == 253 {
        num = 191;
    }
    // `:68319-68349` — the 254 planter-box frameX ladder.
    if kind == 254 {
        if fx < 72 {
            num = 3;
        } else if fx < 108 {
            num = 3;
            if gen.next_max(3) == 0 {
                num = 189;
            }
        } else if fx < 144 {
            num = 3;
            if gen.next_max(2) == 0 {
                num = 189;
            }
        } else {
            num = 3;
            if gen.next_max(4) != 0 {
                num = 189;
            }
        }
    }
    if kind == 467 {
        num = -1;
    }
    // `:68354-68357` — the 21 chest frameX ladder.
    if kind == 21 {
        num = if fx >= 1008 {
            -1
        } else if fx >= 612 {
            11
        } else if fx >= 576 {
            148
        } else if fx >= 540 {
            26
        } else if fx >= 504 {
            126
        } else if fx >= 468 {
            116
        } else if fx >= 432 {
            7
        } else if fx >= 396 {
            11
        } else if fx >= 360 {
            10
        } else if fx >= 324 {
            79
        } else if fx >= 288 {
            78
        } else if fx >= 252 {
            77
        } else if fx >= 216 {
            1
        } else if fx >= 180 {
            7
        } else if fx >= 108 {
            37
        } else if fx < 36 {
            7
        } else {
            10
        };
    }
    if kind == 382 {
        num = 3;
    }
    if matches!(kind, 2 | 477) {
        num = if gen.next_max(2) != 0 { 2 } else { 0 };
    }
    if kind == 127 {
        num = 67;
    }
    if kind == 91 {
        num = -1;
    }
    if kind == 198 {
        num = 109;
    }
    if kind == 34 {
        num = -1;
    }
    if kind == 6 {
        num = 8;
    }
    if matches!(kind, 7 | 47 | 284 | 682) {
        num = 9;
    }
    if matches!(kind, 8 | 45 | 102 | 680) {
        num = 10;
    }
    if matches!(kind, 9 | 42 | 46 | 126 | 136 | 681) {
        num = 11;
    }
    if matches!(kind, 166 | 175) {
        num = 81;
    }
    if kind == 167 {
        num = 82;
    }
    if matches!(kind, 168 | 176) {
        num = 83;
    }
    if matches!(kind, 169 | 177) {
        num = 84;
    }
    if matches!(kind, 199 | 662) {
        num = 117;
    }
    if kind == 205 {
        num = 125;
    }
    if kind == 201 {
        num = 125;
    }
    if kind == 211 {
        num = 128;
    }
    // `:68430-68463` — the 227 dye-plant frameX/34 switch.
    if kind == 227 {
        match fx / 34 {
            0 | 1 => num = 26,
            3 => num = 3,
            2 | 4 | 5 | 6 => num = 40,
            7 => num = 117,
            8 => num = 17,
            9 => num = 6,
            10 => num = 3,
            11 => num = 26,
            _ => {}
        }
    }
    if matches!(kind, 204 | 478) {
        num = 117;
        if gen.next_max(2) == 0 {
            num = 1;
        }
    }
    if kind == 203 {
        num = 117;
    }
    if kind == 243 {
        num = if gen.next_max(2) != 0 { 13 } else { 7 };
    }
    if kind == 219 {
        num = -1;
    }
    if kind == 642 {
        num = -128;
    }
    if kind == 244 {
        num = if gen.next_max(2) == 0 { 1 } else { 13 };
    }
    // `:68492-68503` — the 597/furniture else-if chain: the big list
    // rolls gen.Next(3) and keeps 13 only on a 0.
    if kind == 597 {
        num = -1;
    } else if matches!(
        kind,
        358..=364 | 275..=282 | 285 | 286 | 288..=297 | 316..=318 | 298 | 299 | 309 | 310
            | 339 | 538 | 413 | 414 | 505 | 521..=527 | 532 | 543 | 544 | 550 | 551 | 533
            | 553..=556 | 558 | 559 | 542 | 391 | 394 | 392 | 393 | 568..=570 | 582 | 580
            | 598..=612 | 619 | 620 | 629 | 632 | 640 | 643..=645 | 710
    ) {
        num = 13;
        if gen.next_max(3) != 0 {
            num = -1;
        }
    }
    if kind == 13 {
        num = if fx < 90 { 13 } else { -1 };
    }
    if kind == 189 {
        num = 16;
    }
    if kind == 717 {
        num = 326;
    }
    if kind == 718 {
        num = 327;
    }
    if kind == 719 {
        num = 328;
    }
    if kind == 460 {
        num = 16;
    }
    // `:68528-68537` — the 530 palm trunk's ground read (j + 2 -
    // frameY/18 — the trunk's base cell).
    if kind == 530 {
        let ground = world.tile(i, j + 2 - fy / 18).map_or(0, |g| g.kind);
        num = match ground {
            116 => 47,
            234 => 125,
            112 => 17,
            _ => {
                if fx >= 270 {
                    291
                } else {
                    40
                }
            }
        };
    }
    // `:68538-68543` — the 705 palm 2-D decode.
    if kind == 705 {
        let num16 = fx / 54 + fy / 36 * 10;
        let flag = fx % 486 < 270;
        num = if (0..=8).contains(&num16) {
            if flag {
                291
            } else {
                40
            }
        } else if (9..=17).contains(&num16) {
            47
        } else if !(18..=26).contains(&num16) {
            17
        } else {
            125
        };
    }
    // `:68544-68593` — the 518/519/636/528 else-if chain.
    if kind == 518 {
        if fy == 0 {
            num = 3;
        } else if fy == 18 {
            num = 47;
        } else if fy == 36 {
            num = 40;
        }
    } else if kind == 519 {
        match fy {
            0 => num = 3,
            18 => num = 40,
            36 => num = 47,
            54 => num = 125,
            72 => num = 17,
            90 => num = 26,
            _ => {}
        }
    } else if kind == 636 {
        num = 17;
    } else if kind == 528 {
        num = 26;
    }
    if kind == 12 {
        num = 12;
    }
    if kind == 639 {
        num = 48;
    }
    if matches!(kind, 3 | 73) {
        num = 3;
    }
    if kind == 54 {
        num = 13;
    }
    if matches!(kind, 22 | 140) {
        num = 14;
    }
    if kind == 78 {
        num = 22;
    }
    // `:68618-68665` — the 28/653 sapling frameY ladder.
    if matches!(kind, 28 | 653) {
        num = 22;
        if (72..=90).contains(&fy) {
            num = 1;
        }
        if (144..=234).contains(&fy) {
            num = 48;
        }
        if (252..=358).contains(&fy) {
            num = 85;
        }
        if (360..=466).contains(&fy) {
            num = 26;
        }
        if (468..=574).contains(&fy) {
            num = 36;
        }
        if (576..=790).contains(&fy) {
            num = 18;
        }
        if (792..=898).contains(&fy) {
            num = 5;
        }
        if (900..=1006).contains(&fy) {
            num = 0;
        }
        if (1008..=1114).contains(&fy) {
            num = 148;
        }
        if (1116..=1222).contains(&fy) {
            num = 241;
        }
        if (1224..=1330).contains(&fy) {
            num = 287;
        }
    }
    if kind == 163 {
        num = 118;
    }
    if kind == 164 {
        num = 119;
    }
    if kind == 200 {
        num = 120;
    }
    if matches!(kind, 221 | 248) {
        num = 144;
    }
    if matches!(kind, 222 | 249) {
        num = 145;
    }
    if matches!(kind, 223 | 250) {
        num = 146;
    }
    if kind == 224 {
        num = 149;
    }
    if kind == 225 {
        num = 147;
    }
    if kind == 229 {
        num = 153;
    }
    if kind == 231 {
        num = 153;
        if gen.next_max(3) == 0 {
            num = 26;
        }
    }
    if kind == 226 {
        num = 148;
    }
    if kind == 103 {
        num = -1;
    }
    if kind == 29 {
        num = 23;
    }
    if kind == 40 {
        num = 28;
    }
    if matches!(kind, 50 | 707) {
        num = 22;
    }
    if matches!(kind, 51 | 697) {
        num = 30;
    }
    if matches!(kind, 52 | 353) {
        num = 3;
    }
    if matches!(kind, 53 | 81 | 151 | 202 | 274 | 495) {
        num = 32;
    }
    if matches!(kind, 56 | 152) {
        num = 37;
    }
    if matches!(kind, 75 | 683) {
        num = 109;
    }
    if matches!(kind, 57 | 119 | 141 | 234 | 635 | 654) {
        num = 36;
    }
    if matches!(kind, 59 | 120) {
        num = 38;
    }
    if matches!(
        kind,
        61 | 703 | 62 | 74 | 80 | 188 | 233 | 236 | 702 | 384 | 652 | 651
    ) {
        num = 40;
    }
    if kind == 485 {
        num = 32;
    }
    if kind == 238 {
        num = if gen.next_max(3) != 0 { 166 } else { 167 };
    }
    if kind == 69 {
        num = 7;
    }
    if kind == 655 {
        num = 166;
    }
    if matches!(kind, 71 | 72 | 190 | 578) {
        num = 26;
    }
    if kind == 70 {
        num = 17;
    }
    if kind == 112 {
        num = 14;
    }
    if kind == 123 {
        num = 53;
    }
    if kind == 161 {
        num = 80;
    }
    if kind == 206 {
        num = 80;
    }
    if kind == 162 {
        num = 80;
    }
    // `:68806-68825` — the 165/693/694 frameX/54 table.
    if matches!(kind, 165 | 693 | 694) {
        num = match fx / 54 {
            0 => 80,
            1 => 1,
            2 => 30,
            3 => 147,
            4 => 51,
            5 => 14,
            6 => 117,
            7 => 250,
            8 => 240,
            9 => 236,
            10 => 119,
            11 => 118,
            12 => 120,
            _ => 1,
        };
    }
    if kind == 666 {
        num = 322;
    }
    if kind == 193 {
        num = 4;
    }
    if kind == 194 {
        num = 26;
    }
    if kind == 195 {
        num = 5;
    }
    if kind == 196 {
        num = 108;
    }
    if kind == 460 {
        num = 108; // the C# repeats 460 (`:68846`, overwrites 16), kept
    }
    if kind == 197 {
        num = 4;
    }
    if kind == 153 {
        num = 26;
    }
    if kind == 154 {
        num = 32;
    }
    if kind == 155 {
        num = 2;
    }
    if kind == 156 {
        num = 1;
    }
    if matches!(kind, 116 | 117 | 118 | 147 | 148) {
        num = 51;
    }
    if matches!(kind, 109 | 492) {
        num = if gen.next_max(2) != 0 { 47 } else { 0 };
    }
    if matches!(kind, 110 | 113 | 115) {
        num = 47;
    }
    if matches!(kind, 107 | 121 | 685) {
        num = 48;
    }
    if matches!(kind, 108 | 122 | 146 | 686) {
        num = 49;
    }
    if matches!(kind, 111 | 145 | 150) {
        num = 50;
    }
    if kind == 133 {
        num = 50;
        if fx >= 54 {
            num = 146;
        }
    }
    if kind == 134 {
        num = 49;
        if fx >= 36 {
            num = 145;
        }
    }
    if kind == 149 {
        num = 49;
    }
    // `:68914-68945` — the herb family (Main.tileAlch = 82/83/84,
    // `Main.cs:7806-7808`).
    if matches!(kind, 82 | 83 | 84) {
        let num17 = fx / 18;
        match num17 {
            0 | 1 => num = 3,
            2 => num = 7,
            3 => num = 17,
            4 => num = 289,
            5 => num = 6,
            6 => num = 224,
            _ => {}
        }
    }
    if matches!(kind, 58 | 76 | 77 | 684) {
        num = if gen.next_max(2) != 0 { 25 } else { 6 };
    }
    if kind == 37 {
        num = if gen.next_max(2) != 0 { 23 } else { 6 };
    }
    if kind == 32 {
        num = if gen.next_max(2) != 0 { 24 } else { 14 };
    }
    if kind == 352 {
        num = if gen.next_max(3) != 0 { 125 } else { 5 };
    }
    if matches!(kind, 23 | 24 | 661) {
        num = if gen.next_max(2) != 0 { 17 } else { 14 };
    }
    // `:68966-68969` — the 25/31/696 arm: the (31|696) && fx>=36
    // branch takes 5 with NO roll (the ternary short-circuit).
    if matches!(kind, 25 | 31 | 696) {
        num = if matches!(kind, 31 | 696) && fx >= 36 {
            5
        } else if gen.next_max(2) != 0 {
            1
        } else {
            14
        };
    }
    // `:68970-68973` — the 26/695 altar arm: fx>=54 takes 5, NO roll.
    if matches!(kind, 26 | 695) {
        num = if fx >= 54 {
            5
        } else if gen.next_max(2) != 0 {
            1
        } else {
            14
        };
    }
    // `:68974-68985` — the 20 sapling frameX/54 table.
    if kind == 20 {
        num = match fx / 54 {
            1 => 122,
            2 => 78,
            3 => 77,
            4 => 121,
            5 => 79,
            _ => 7,
        };
    }
    if kind == 27 {
        num = if gen.next_max(2) != 0 { 19 } else { 3 };
    }
    // `:68990-68997` — the 129 ice-machine arm: the fx>=324 write is
    // DEAD (the next line overwrites unconditionally) — kept.
    if kind == 129 {
        if fx >= 324 {
            num = 69; // dead write (`:68992-68995`), kept
        }
        num = if fx != 0 && fx != 54 && fx != 108 {
            if fx != 18 && fx != 72 && fx != 126 {
                70
            } else {
                69
            }
        } else {
            68
        };
    }
    if kind == 385 {
        num = gen.next_range(68, 71);
    }
    // `:69002-69006` — the torch style table (MathHelper.Clamp of
    // frameY/22 into 0..TorchID.Count-1).
    if kind == 4 {
        let num18 = (fy / 22).clamp(0, 23);
        num = TORCH_DUST[num18 as usize];
    }
    // `:69007-69014` — the 35 campfire arm: the Next(2) draws ONLY
    // when fx < 36 (the && short-circuit).
    if kind == 35 {
        num = 189;
        if fx < 36 && gen.next_max(2) == 0 {
            num = 6;
        }
    }
    if matches!(kind, 34 | 42) && gen.next_max(2) == 0 {
        num = 6;
    }
    if kind == 270 {
        num = -1;
    }
    if kind == 271 {
        num = -1;
    }
    if kind == 581 {
        num = -1;
    }
    if kind == 660 {
        num = -1;
    }
    if kind == 698 {
        num = -1;
    }
    if matches!(kind, 79 | 90 | 101) {
        num = -1;
    }
    if matches!(kind, 33 | 34 | 42 | 93 | 100) {
        num = -1;
    }
    if matches!(kind, 321 | 574) {
        num = 214;
    }
    if kind == 322 {
        num = 215;
    }
    if kind == 635 {
        num = 36;
    }
    num
}

/// The `flag2` gem family gate (`WorldGen.cs:69059-69084`): 178/440
/// plus the stained-glass 426-439/727-732 set.
fn flag2_kind(kind: u16) -> bool {
    matches!(
        kind,
        178 | 440 | 426 | 427 | 430 | 431 | 432 | 433 | 434 | 435 | 436 | 437 | 438 | 439
            | 727 | 728 | 729 | 730 | 731 | 732
    )
}

/// One `KillTile_MakeTileDust` call (`WorldGen.cs:66687-69169`): the
/// type chain, then the FIRST matching spawn arm (`:69085-69167`); a
/// negative type returns the sentinel without a spawn. `disco` is
/// `Main.DiscoR/G/B`. Returns the slot index (the sentinel included).
#[allow(clippy::too_many_arguments)]
pub(crate) fn make_tile_dust(
    rng: &mut UnifiedRandom,
    gen: &mut UnifiedRandom,
    disco: [u8; 3],
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    i: i32,
    j: i32,
    t: &Tile,
) -> usize {
    let num = tile_dust_type(world, i, j, t, rng, gen);
    if num < 0 {
        return DUST_SENTINEL;
    }
    let kind = t.kind;
    let fx = i32::from(t.frame_x);
    let fy = i32::from(t.frame_y);
    let pos = Vec2::new(i as f32 * TILE_PX, j as f32 * TILE_PX);
    // `:69087-69093` — the disco-tile arm (627/628/184-style-10).
    if matches!(kind, 627 | 628) || (kind == 184 && fx / 22 == 10) {
        let idx = dust.new_dust(
            rng, ctx, world, pos, 16, 16, num, 0.0, 0.0, 0,
            Color::new(disco[0], disco[1], disco[2], 255), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.no_light_emittance = true;
        return idx;
    }
    // `:69094-69102` — the 518 fountain's liquid-offset arm.
    if kind == 518 {
        let mut num20 = i32::from(t.liquid) / 16 - 3;
        if solid_tile(world, i, j - 1) && num20 > 8 {
            num20 = 8;
        }
        return dust.new_dust(
            rng, ctx, world, Vec2::new(pos.x, pos.y - num20 as f32), 16, 16, num,
            0.0, 0.0, 0, Color::default(), 1.0,
        );
    }
    // `:69103-69111` — the 352 campfire's type-5 ember.
    if kind == 352 && num == 5 {
        let idx = dust.new_dust(rng, ctx, world, pos, 16, 16, 5, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.scale = 1.5;
        d.no_gravity = true;
        d.velocity *= 1.65;
        d.fade_in = 1.6;
        return idx;
    }
    // `:69112-69117` — the 160/692 disco arm.
    if matches!(kind, 160 | 692) {
        let idx = dust.new_dust(
            rng, ctx, world, pos, 16, 16, 66, 0.0, 0.0, 100,
            Color::new(disco[0], disco[1], disco[2], 255), 0.75,
        );
        dust.slot_mut(idx).no_gravity = true;
        return idx;
    }
    // `:69118-69121` — the 323 gem-tree's frameY x-offset.
    if kind == 323 {
        return dust.new_dust(
            rng, ctx, world, Vec2::new(pos.x + fy as f32, pos.y), 16, 16, num,
            0.0, 0.0, 0, Color::default(), 1.0,
        );
    }
    // `:69122-69129` — the 314 minecart-track arm: two genRand velocity
    // ARGUMENTS, then the fadeIn roll AFTER the spawn (scale + 1 +
    // 0.01*Next(0,51)); noGravity written twice (the C# repeats it).
    if kind == 314 {
        let sx = gen.next_range(-2, 3) as f32;
        let sy = gen.next_range(-2, 3) as f32;
        let idx = dust.new_dust(rng, ctx, world, pos, 16, 16, 213, sx, sy, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.fade_in = d.scale + 1.0 + 0.01 * gen.next_range(0, 51) as f32;
        d.no_gravity = true;
        return idx;
    }
    // `:69130-69135` — the gem family's noLight arm.
    if flag2_kind(kind) {
        let idx = dust.new_dust(rng, ctx, world, pos, 16, 16, num, 0.0, 0.0, 75, Color::default(), 0.75);
        dust.slot_mut(idx).no_light = true;
        return idx;
    }
    // `:69136-69139` — the 193/18/19 blue arm.
    if kind == 193 || (kind == 18 && num == 4) || (kind == 19 && num == 4) {
        return dust.new_dust(
            rng, ctx, world, pos, 16, 16, num, 0.0, 0.0, 75,
            Color::new(0, 80, 255, 100), 0.75,
        );
    }
    // `:69140-69143` — the 197 cyan arm.
    if kind == 197 {
        return dust.new_dust(
            rng, ctx, world, pos, 16, 16, num, 0.0, 0.0, 75,
            Color::new(97, 200, 255, 100), 0.75,
        );
    }
    // `:69144-69149` — the 185 gem-banner noLight arm.
    if kind == 185 && (86..=91).contains(&num) {
        let idx = dust.new_dust(rng, ctx, world, pos, 16, 16, num, 0.0, 0.0, 75, Color::default(), 0.75);
        dust.slot_mut(idx).no_light = true;
        return idx;
    }
    // `:69150-69155` — the torch-66 disco arm: the FLOAT Color ctor
    // (`Color(discoR/255f, ...)`) — XNA multiplies back by 255 and
    // truncates, so the f32 round-trip is kept verbatim.
    if kind == 4 && num == 66 {
        let ch = |b: u8| (b as f32 / 255.0 * 255.0).clamp(0.0, 255.0) as u8;
        let idx = dust.new_dust(
            rng, ctx, world, pos, 16, 16, num, 0.0, 0.0, 0,
            Color::new(ch(disco[0]), ch(disco[1]), ch(disco[2]), 255), 1.0,
        );
        dust.slot_mut(idx).no_gravity = true;
        return idx;
    }
    // `:69156-69159` — the 139 random-type arm: the named-argument
    // order evaluates `Type: num + Main.rand.Next(4)` FIRST.
    if num == 139 {
        let ty = num + rng.next_max(4);
        return dust.new_dust(rng, ctx, world, pos, 16, 16, ty, 0.0, 0.0, 0, Color::default(), 1.0);
    }
    // `:69160-69165` — the 722 slowed arm.
    if kind == 722 {
        let idx = dust.new_dust(rng, ctx, world, pos, 16, 16, num, 0.0, 0.0, 0, Color::default(), 1.0);
        dust.slot_mut(idx).velocity *= 0.4;
        return idx;
    }
    // `:69166` — the default arm.
    dust.new_dust(rng, ctx, world, pos, 16, 16, num, 0.0, 0.0, 0, Color::default(), 1.0)
}

/// The `KillTile` / `ReplaceTile` dust loop (`WorldGen.cs:63699-63703`
/// / `:62570-62574`): `tile_dust_amount` calls of the body. `gen` is
/// the caller's `frame_rng` snapshot; `t` the pre-kill tile.
#[allow(clippy::too_many_arguments)]
pub(crate) fn tile_break_burst(
    rng: &mut UnifiedRandom,
    gen: &mut UnifiedRandom,
    disco: [u8; 3],
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    i: i32,
    j: i32,
    t: &Tile,
    fail: bool,
) {
    let count = tile_dust_amount(fail, t.kind, t.frame_x);
    for _ in 0..count {
        make_tile_dust(rng, gen, disco, dust, ctx, world, i, j, t);
    }
}

#[cfg(test)]
mod tests {
    //! W4-d54 batch-3 dust witnesses — WRITTEN, NOT RUN (code-only).

    use super::*;

    fn dust_ctx(w: &World) -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(w.width as f32 * 16.0, w.height as f32 * 16.0),
            max_dust_to_draw: 6000,
        }
    }

    fn tile(kind: u16, fx: i16, fy: i16) -> Tile {
        Tile { kind, frame_x: fx, frame_y: fy, ..Default::default() }
    }

    /// The default block + the per-type tweak family (`Dust.cs:214-
    /// 252`): only the 228/279/269/135/6/242/75/169/29/59..65/158/
    /// 293..298/302/307/310 family adds the `Next(-10,6)`; the other
    /// tweak arms (127/187, 308, 33/52/266/98..105, 41, 80, 34/35/152)
    /// draw nothing.
    fn block(r: &mut UnifiedRandom, dust_type: i32, box_w: i32, box_h: i32) {
        r.next_max(box_w.max(5) - 4);
        r.next_max(box_h.max(5) - 4);
        r.next_range(-20, 21);
        r.next_range(-20, 21);
        r.next_max(3);
        r.next_range(-20, 21);
        if matches!(
            dust_type,
            228 | 279 | 269 | 135 | 6 | 242 | 75 | 169 | 29 | 59..=65 | 158
                | 293..=298 | 302 | 307 | 310
        ) {
            r.next_range(-10, 6);
        }
    }

    fn drain(dust: &mut DustPool) {
        for s in dust.slots_mut() {
            s.active = false;
        }
    }

    /// `WorldGen.cs:66597-66685` — the dust-amount table pinned to the
    /// C# literals, both fail arms + the 184 frame gate.
    #[test]
    fn w4_d54_tile_dust_amount_matches_the_csharp_table() {
        for fail in [false, true] {
            let d = |k: u16, fx: i16| tile_dust_amount(fail, k, fx);
            assert_eq!(d(1, 0), if fail { 3 } else { 10 });
            assert_eq!(d(654, 0), 0);
            assert_eq!(d(630, 0), 0);
            assert_eq!(d(631, 0), 0);
            assert_eq!(d(231, 0), if fail { 3 } else { 6 });
            assert_eq!(d(481, 0), 3);
            assert_eq!(d(482, 0), 3);
            assert_eq!(d(483, 0), 3);
            assert_eq!(d(541, 0), 1);
            assert_eq!(d(549, 0), 2);
            for k in [125u16, 138, 172, 287, 300, 308, 354, 355, 373, 376, 461,
                484, 499, 564, 565, 593, 594, 617, 664, 665, 699, 709, 711, 716] {
                assert_eq!(d(k, 0), 0, "kind {k}");
            }
            for k in [534u16, 536, 539, 625, 627] {
                assert_eq!(d(k, 0), if fail { 3 } else { 5 }, "kind {k}");
            }
            assert_eq!(d(184, 110), 5); // frameX/22 == 5
            assert_eq!(d(184, 109), if fail { 3 } else { 10 });
        }
    }

    /// The type chain's constant arms pinned to the C# literals — a
    /// spread across the head, the switches, the frame ladders and the
    /// later-wins overwrites. No genRand/Main.rand draws for these.
    #[test]
    fn w4_d54_tile_dust_type_matches_the_csharp_table() {
        let w = World::new("t".into(), 200, 200);
        let mut rng = UnifiedRandom::new(1);
        let mut gen = UnifiedRandom::new(1);
        let cases: &[(Tile, i32)] = &[
            (tile(0, 0, 0), 0),
            (tile(216, 0, 0), -1),
            (tile(324, 0, 0), 280),
            (tile(324, 18, 18), 282),
            (tile(335, 0, 0), -1),
            (tile(192, 0, 0), 3),
            (tile(208, 0, 0), 126),
            (tile(408, 0, 0), 265),
            (tile(669, 0, 0), 314),
            (tile(676, 0, 0), 321),
            (tile(16, 0, 0), 1),
            (tile(16, 36, 0), 82),
            (tile(415, 0, 0), 6),
            (tile(474, 0, 0), 18),
            (tile(1, 0, 0), 1),
            (tile(63, 0, 0), 1), // TILE_STONE
            (tile(566, 0, 0), 1),
            (tile(753, 0, 0), -1),
            (tile(379, 0, 0), 257),
            (tile(311, 0, 0), 207),
            (tile(104, 0, 0), -1),
            (tile(95, 0, 0), 6),
            (tile(30, 0, 0), 7),
            (tile(646, 0, 0), 29),
            (tile(10, 0, 0), -1),
            (tile(668, 0, 0), 0),
            (tile(407, 0, 0), 10),
            (tile(454, 0, 0), 139),
            (tile(41, 0, 0), 275), // the switch overwrites the stone 1
            (tile(43, 0, 0), 276),
            (tile(44, 0, 0), 277),
            (tile(473, 0, 0), 82),
            (tile(498, 0, 0), 30),
            (tile(517, 0, 0), 258),
            (tile(184, 110, 0), 258), // style 5
            (tile(184, 220, 0), 267), // style 10
            (tile(184, 0, 0), 93),    // style 0 default
            (tile(515, 0, 0), 96),
            (tile(541, 0, 0), 226),
            (tile(595, 0, 0), 78),
            (tile(637, 0, 0), 237),
            (tile(179, 0, 0), 93),  // moss default: 179-179+93
            (tile(381, 0, 0), 258), // moss arm
            (tile(627, 0, 0), 267),
            (tile(240, 0, 0), 7),
            (tile(240, 54 * 16, 0), 26),
            (tile(240, 54 * 46, 0), -1),
            (tile(241, 0, 0), 1),
            (tile(365, 0, 0), 239),
            (tile(357, 0, 0), 236),
            (tile(170, 0, 0), 196),
            (tile(326, 0, 0), 13),
            (tile(336, 0, 0), 6),
            (tile(340, 0, 0), 75),
            (tile(562, 0, 0), 273),
            (tile(330, 0, 0), 9),
            (tile(722, 0, 0), 57),
            (tile(734, 0, 0), 0),
            (tile(750, 0, 0), 26),
            (tile(742, 0, 0), 96),
            (tile(101, 0, 0), -1),
            (tile(19, 0, 0), 7),
            (tile(19, 0, 18 * 50), 309),
            (tile(79, 0, 0), 7),
            (tile(79, 0, 36 * 8), 109),
            (tile(18, 0, 0), 7),
            (tile(18, 36 * 15, 0), 126),
            (tile(18, 36 * 16, 0), -1),
            (tile(14, 0, 0), -1),
            (tile(255, 0, 0), 86),
            (tile(261, 0, 0), 138),
            (tile(268, 0, 0), 138),
            (tile(178, 18 * 6, 0), 138),
            (tile(440, 0, 54), 90),
            (tile(440, 0, 0), -1), // frameY < 54
            (tile(426, 0, 0), 90),
            (tile(732, 0, 0), 91),
            (tile(549, 0, 0), 3),
            (tile(186, 0, 0), 26),
            (tile(186, 1400, 0), 80),
            (tile(187, 0, 0), 1),
            (tile(187, 1200, 0), 1), // the dead 155 arm never fires
            (tile(647, 0, 0), 26),
            (tile(648, 0, 0), 1),
            (tile(105, 1600, 0), 148),
            (tile(239, 18 * 22, 0), 265),
            (tile(185, 0, 18), 1),
            (tile(185, 36 * 20, 18), 87),
            (tile(185, 18 * 75, 0), 32),
            (tile(649, 36 * 64, 0), 40),
            (tile(650, 18 * 75, 0), 40),
            (tile(237, 0, 0), 148),
            (tile(157, 0, 0), 77),
            (tile(15, 0, 0), -1),
            (tile(137, 0, 18 * 2), 148),
            (tile(137, 0, 18 * 5), 1),
            (tile(213, 0, 0), 129),
            (tile(215, 0, 0), -6),
            (tile(254, 0, 0), 3),
            (tile(21, 0, 0), 7),
            (tile(21, 1008, 0), -1),
            (tile(21, 612, 0), 11),
            (tile(382, 0, 0), 3),
            (tile(127, 0, 0), 67),
            (tile(6, 0, 0), 8),
            (tile(7, 0, 0), 9),
            (tile(166, 0, 0), 81),
            (tile(227, 34 * 8, 0), 17),
            (tile(203, 0, 0), 117),
            (tile(642, 0, 0), -128),
            (tile(13, 0, 0), 13),
            (tile(13, 90, 0), -1),
            (tile(717, 0, 0), 326),
            (tile(460, 0, 0), 108), // the later 460 arm overwrites 16
            (tile(518, 0, 18), 47),
            (tile(519, 0, 90), 26),
            (tile(636, 0, 0), 17),
            (tile(12, 0, 0), 12),
            (tile(3, 0, 0), 3),
            (tile(22, 0, 0), 14),
            (tile(28, 0, 0), 22),
            (tile(28, 0, 700), 18),
            (tile(163, 0, 0), 118),
            (tile(224, 0, 0), 149),
            (tile(226, 0, 0), 148),
            (tile(29, 0, 0), 23),
            (tile(50, 0, 0), 22),
            (tile(53, 0, 0), 32),
            (tile(57, 0, 0), 36),
            (tile(61, 0, 0), 40),
            (tile(69, 0, 0), 7),
            (tile(70, 0, 0), 17),
            (tile(112, 0, 0), 14),
            (tile(123, 0, 0), 53),
            (tile(165, 54 * 6, 0), 117),
            (tile(666, 0, 0), 322),
            (tile(193, 0, 0), 4),
            (tile(196, 0, 0), 108),
            (tile(116, 0, 0), 51),
            (tile(110, 0, 0), 47),
            (tile(107, 0, 0), 48),
            (tile(133, 54, 0), 146),
            (tile(134, 0, 0), 49),
            (tile(82, 18 * 4, 0), 289), // herb style 4
            (tile(83, 18 * 6, 0), 224),
            (tile(20, 54 * 3, 0), 77),
            (tile(129, 0, 0), 68),
            (tile(129, 18, 0), 69),
            (tile(129, 36, 0), 70),
            (tile(129, 324, 0), 70), // the dead 69 write never survives
            (tile(4, 0, 0), 6),        // torch style 0
            (tile(4, 0, 22 * 14), 66), // torch style 14
            (tile(4, 0, 22 * 99), 310), // clamped to 23
            (tile(321, 0, 0), 214),
            (tile(635, 0, 0), 36),
            (tile(79, 0, 36 * 9), -1), // the final 79/90/101 overwrite
            (tile(33, 0, 0), -1),
        ];
        for (t, want) in cases {
            assert_eq!(
                tile_dust_type(&w, 100, 100, t, &mut rng, &mut gen),
                *want,
                "kind {} fx {} fy {}", t.kind, t.frame_x, t.frame_y
            );
        }
        assert_eq!(rng.peek(), UnifiedRandom::new(1).peek());
        assert_eq!(gen.peek(), UnifiedRandom::new(1).peek());
    }

    /// The genRand/Main.rand arms draw exactly the C#'s rolls, in
    /// order, with the right polarity — one oracle replay per arm.
    #[test]
    fn w4_d54_tile_dust_type_random_arms_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        for seed in [1, 54, 99] {
            let mut rng = UnifiedRandom::new(seed);
            let mut rng_p = UnifiedRandom::new(seed);
            let mut gen = UnifiedRandom::new(seed);
            let mut gen_p = UnifiedRandom::new(seed);
            // (tile, oracle arm id)
            let cases: &[(Tile, u8)] = &[
                (tile(583, 0, 0), 1), (tile(589, 0, 0), 22),
                (tile(633, 0, 0), 2), (tile(634, 0, 0), 3),
                (tile(641, 0, 0), 4), (tile(370, 0, 0), 4),
                (tile(171, 0, 0), 5), (tile(254, 90, 0), 6),
                (tile(254, 120, 0), 7), (tile(254, 200, 0), 8),
                (tile(2, 0, 0), 4), (tile(204, 0, 0), 9),
                (tile(243, 0, 0), 4), (tile(244, 0, 0), 9),
                (tile(597, 0, 0), 10), (tile(275, 0, 0), 11),
                (tile(231, 0, 0), 12), (tile(238, 0, 0), 13),
                (tile(109, 0, 0), 4), (tile(58, 0, 0), 4),
                (tile(37, 0, 0), 4), (tile(32, 0, 0), 4),
                (tile(352, 0, 0), 14), (tile(23, 0, 0), 4),
                (tile(25, 0, 0), 4), (tile(31, 36, 0), 15),
                (tile(26, 54, 0), 16), (tile(27, 0, 0), 4),
                (tile(385, 0, 0), 17), (tile(35, 0, 0), 18),
                (tile(35, 36, 0), 19), (tile(34, 0, 0), 20),
                (tile(19, 0, 18 * 30), 21),
            ];
            for (t, arm) in cases {
                let got = tile_dust_type(&w, 100, 100, t, &mut rng, &mut gen);
                let want = match arm {
                    1 => if gen_p.next_max(10) != 0 { 1 } else { 87 },
                    22 => if gen_p.next_max(10) != 0 { 1 } else { 138 },
                    2 => if gen_p.next_max(6) != 0 { 237 } else { 36 },
                    3 => {
                        let mut n = if gen_p.next_max(10) != 0 { 36 } else { 31 };
                        if gen_p.next_max(12) == 0 { n = 6; }
                        n
                    }
                    4 => {
                        let r = gen_p.next_max(2);
                        match t.kind {
                            641 => if r != 0 { 161 } else { 243 },
                            370 => if r != 0 { 23 } else { 6 },
                            2 | 477 => if r != 0 { 2 } else { 0 },
                            243 => if r != 0 { 13 } else { 7 },
                            109 | 492 => if r != 0 { 47 } else { 0 },
                            58 | 76 | 77 | 684 => if r != 0 { 25 } else { 6 },
                            37 => if r != 0 { 23 } else { 6 },
                            32 => if r != 0 { 24 } else { 14 },
                            23 | 24 | 661 => if r != 0 { 17 } else { 14 },
                            25 => if r != 0 { 1 } else { 14 },
                            27 => if r != 0 { 19 } else { 3 },
                            _ => unreachable!(),
                        }
                    }
                    5 => if gen_p.next_max(2) != 0 { -1 } else { 196 },
                    6 => { let mut n = 3; if gen_p.next_max(3) == 0 { n = 189; } n }
                    7 => { let mut n = 3; if gen_p.next_max(2) == 0 { n = 189; } n }
                    8 => { let mut n = 3; if gen_p.next_max(4) != 0 { n = 189; } n }
                    9 => {
                        // 204: 117 then Next(2)==0 → 1; 244: Next(2)==0 → 1 else 13
                        if t.kind == 244 {
                            if gen_p.next_max(2) == 0 { 1 } else { 13 }
                        } else {
                            let mut n = 117;
                            if gen_p.next_max(2) == 0 { n = 1; }
                            n
                        }
                    }
                    10 => -1, // 597: no roll
                    11 => { let mut n = 13; if gen_p.next_max(3) != 0 { n = -1; } n }
                    12 => { let mut n = 153; if gen_p.next_max(3) == 0 { n = 26; } n }
                    13 => if gen_p.next_max(3) != 0 { 166 } else { 167 },
                    14 => if gen_p.next_max(3) != 0 { 125 } else { 5 },
                    15 => 5,  // 31 fx>=36: no roll
                    16 => 5,  // 26 fx>=54: no roll
                    17 => gen_p.next_range(68, 71),
                    18 => { let mut n = 189; if gen_p.next_max(2) == 0 { n = 6; } n }
                    19 => 189, // 35 fx>=36: the && short-circuit, no roll
                    20 => {
                        // 34: the roll fires, but the final 33|34|42|
                        // 93|100 overwrite (`:69043`) resets to -1.
                        gen_p.next_max(2);
                        -1
                    }
                    21 => 68 + rng_p.next_max(3), // Main.rand!
                    _ => unreachable!(),
                };
                assert_eq!(got, want, "kind {} arm {arm} seed {seed}", t.kind);
                assert_eq!(rng.peek(), rng_p.peek(), "main kind {}", t.kind);
                assert_eq!(gen.peek(), gen_p.peek(), "gen kind {}", t.kind);
            }
        }
    }

    /// `WorldGen.cs:69085-69167` — the spawn arms' draw census: the
    /// disco tile (627), the fountain (518), the gem noLight arm (426),
    /// the blue arm (193), the torch-66 float arm (4 style 14), the
    /// 139 random-type arm (454), the 722 slowed arm, the default (1),
    /// the negative-type refusal (216) and the minecart (314).
    #[test]
    fn w4_d54_make_tile_dust_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let disco = [10u8, 20, 30];
        // (tile, expected spawned type, arm id)
        // arm: 0 default, 1 disco-tile, 2 fountain, 7 flag2, 8 blue,
        //      11 torch-float, 12 rand139, 99 no-spawn, 6 minecart
        let cases: &[(Tile, i32, u8)] = &[
            (tile(1, 0, 0), 1, 0),
            (tile(627, 0, 0), 267, 1),
            (tile(518, 0, 0), 3, 2),
            (tile(426, 0, 0), 90, 7),
            (tile(193, 0, 0), 4, 8),
            (tile(4, 0, 22 * 14), 66, 11),
            (tile(454, 0, 0), 139, 12),
            (tile(722, 0, 0), 57, 0),
            (tile(216, 0, 0), -1, 99),
            (tile(314, 0, 0), 213, 6),
        ];
        for (t, ty, arm) in cases {
            let mut rng = UnifiedRandom::new(54);
            let mut rng_p = UnifiedRandom::new(54);
            let mut gen = UnifiedRandom::new(7);
            let mut gen_p = UnifiedRandom::new(7);
            let mut dust = DustPool::new();
            for i in 0..40 {
                make_tile_dust(&mut rng, &mut gen, disco, &mut dust, &ctx, &w, 10, 20, t);
                match arm {
                    99 => {}
                    12 => {
                        let ty2 = ty + rng_p.next_max(4);
                        block(&mut rng_p, ty2, 16, 16);
                    }
                    6 => {
                        gen_p.next_range(-2, 3);
                        gen_p.next_range(-2, 3);
                        block(&mut rng_p, 213, 16, 16);
                        gen_p.next_range(0, 51);
                    }
                    _ => block(&mut rng_p, *ty, 16, 16),
                }
                assert_eq!(rng.peek(), rng_p.peek(), "kind {} @{i}", t.kind);
                assert_eq!(gen.peek(), gen_p.peek(), "kind {} gen @{i}", t.kind);
                drain(&mut dust);
            }
        }
    }

    /// The post writes: the disco tile's noGravity + noLightEmittance,
    /// the minecart's fadeIn = scale + 1 + 0.01*roll + the gen velocity
    /// arguments, the 722 velocity ×0.4, the torch-float arm's byte
    /// round-trip.
    #[test]
    fn w4_d54_make_tile_dust_writes_the_post_fields() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let disco = [10u8, 20, 30];
        let mut rng = UnifiedRandom::new(54);
        let mut gen = UnifiedRandom::new(7);
        let mut dust = DustPool::new();
        // 627 disco tile.
        make_tile_dust(&mut rng, &mut gen, disco, &mut dust, &ctx, &w, 10, 20, &tile(627, 0, 0));
        let d = dust.slot(0);
        assert_eq!(d.dust_type, 267);
        assert_eq!(d.color, Color::new(10, 20, 30, 255));
        assert!(d.no_gravity && d.no_light_emittance);
        drain(&mut dust);
        // 314 minecart: replay the block to recover scale, then fadeIn.
        let mut rng = UnifiedRandom::new(54);
        let mut gen = UnifiedRandom::new(7);
        make_tile_dust(&mut rng, &mut gen, disco, &mut dust, &ctx, &w, 10, 20, &tile(314, 0, 0));
        let mut gen_p = UnifiedRandom::new(7);
        let sx = gen_p.next_range(-2, 3) as f32;
        let sy = gen_p.next_range(-2, 3) as f32;
        let mut rng_p = UnifiedRandom::new(54);
        rng_p.next_max(12);
        rng_p.next_max(12);
        let vx = rng_p.next_range(-20, 21) as f32 * 0.1 + sx;
        let vy = rng_p.next_range(-20, 21) as f32 * 0.1 + sy;
        rng_p.next_max(3);
        let scale = 1.0 + rng_p.next_range(-20, 21) as f32 * 0.01;
        let fade_roll = gen_p.next_range(0, 51);
        let d = dust.slot(0);
        assert_eq!(d.dust_type, 213);
        assert!(d.no_gravity);
        assert!((d.velocity.x - vx).abs() < 1e-6);
        assert!((d.velocity.y - vy).abs() < 1e-6);
        assert!((d.fade_in - (scale + 1.0 + 0.01 * fade_roll as f32)).abs() < 1e-5);
        drain(&mut dust);
        // 722 velocity ×0.4.
        let mut rng = UnifiedRandom::new(54);
        make_tile_dust(&mut rng, &mut gen, disco, &mut dust, &ctx, &w, 10, 20, &tile(722, 0, 0));
        let mut rng_p = UnifiedRandom::new(54);
        rng_p.next_max(12);
        rng_p.next_max(12);
        let vx = rng_p.next_range(-20, 21) as f32 * 0.1;
        let vy = rng_p.next_range(-20, 21) as f32 * 0.1;
        let d = dust.slot(0);
        assert!((d.velocity.x - vx * 0.4).abs() < 1e-6);
        assert!((d.velocity.y - vy * 0.4).abs() < 1e-6);
        drain(&mut dust);
        // The torch-float arm: the f32 round-trip keeps the bytes.
        let mut rng = UnifiedRandom::new(54);
        make_tile_dust(&mut rng, &mut gen, disco, &mut dust, &ctx, &w, 10, 20, &tile(4, 0, 22 * 14));
        let d = dust.slot(0);
        assert_eq!(d.dust_type, 66);
        assert!(d.no_gravity);
        let ch = |b: u8| (b as f32 / 255.0 * 255.0).clamp(0.0, 255.0) as u8;
        assert_eq!(d.color, Color::new(ch(10), ch(20), ch(30), 255));
    }

    /// `WorldGen.cs:63699-63703` — the break loop runs exactly
    /// `tile_dust_amount` bodies; a zero-amount kind draws nothing.
    #[test]
    fn w4_d54_tile_break_burst_counts_the_amount() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let disco = [0u8; 3];
        let mut rng = UnifiedRandom::new(54);
        let mut rng_p = UnifiedRandom::new(54);
        let mut gen = UnifiedRandom::new(7);
        let mut dust = DustPool::new();
        let t = tile(1, 0, 0);
        for i in 0..20 {
            tile_break_burst(&mut rng, &mut gen, disco, &mut dust, &ctx, &w, 10, 20, &t, false);
            for _ in 0..10 {
                block(&mut rng_p, 1, 16, 16);
            }
            assert_eq!(rng.peek(), rng_p.peek(), "stone burst @{i}");
            drain(&mut dust);
        }
        // The zero-amount kind (654) draws nothing.
        let mut rng = UnifiedRandom::new(54);
        tile_break_burst(&mut rng, &mut gen, disco, &mut dust, &ctx, &w, 10, 20, &tile(654, 0, 0), false);
        assert_eq!(rng.peek(), UnifiedRandom::new(54).peek());
    }
}
