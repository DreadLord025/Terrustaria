//! W4-g14 batch 3 — `WorldGen.KillTile_MakeTileDust`
//! (`WorldGen.cs:66687-69169`), sites 69089/69101/69105/69114/69120/
//! 69124/69132/69138/69142/69146/69152/69158/69162/69166 of the
//! research/65 census (the whole function lands in one commit — the
//! documented exception to the 10-site cap: splitting the ladder from
//! its spawn arms would leave a temporarily unfaithful body).
//!
//! The tile dust-TYPE ladder (`:66689-69058`) is RNG-bearing: ~35
//! `WorldGen.genRand` draws plus ONE `Main.rand.Next(3)` (the type-19
//! row-30 arm, `:67402`). It ports verbatim in C# statement order —
//! sequential if/else-if chains stay chains, the switches stay
//! switches, the column walks (type 5 / type 323) read the world.
//!
//! Two streams (research/65): `gen` = `WorldGen.genRand`, `rng` =
//! `Main.rand` (the row-30 type draw, the 139 `Next(4)` argument, the
//! `NewDust` internals). `disco` is the `(DiscoR, DiscoG, DiscoB)`
//! snapshot; the torch table is `TorchID.Dust` (`TorchID.cs:179-184`).
//!
//! The spawn tail (`:69085-69168`) runs only when `num >= 0` and is a
//! sequence of early-return arms in C# order; the port returns the
//! spawned slot (or the sentinel when nothing spawns — the C#'s
//! `return 6000`).
//!
//! WRITTEN NOT RUN (code-only mode).

use crate::particles::{Color, DustPool, SpawnCtx, DUST_SENTINEL};
use crate::tile::Tile;
use crate::tiledata as td;
use crate::unified_random::UnifiedRandom;
use crate::world::World;
use glam::Vec2;

/// `TorchID.Dust` (`TorchID.cs:179-184`), Count = 24 (`:185`).
const TORCH_DUST: [i32; 24] = [
    6, 59, 60, 61, 62, 63, 64, 65, 75, 135, 158, 169, 156, 234, 66, 242, 293, 294, 295,
    296, 297, 298, 307, 310,
];

/// `WorldGen.beachDistance` (`WorldGen.cs:86914`) — the drawdata port's
/// constant, cited there; repeated here as the ladder's own read.
const BEACH_DISTANCE: i32 = 380;

/// Ladder head (`WorldGen.cs:66689-66845`): the opening if-chain and
/// the 208/408-409/669-676 else-if family, the 16/415-418/474 chain,
/// the stone-family set, and the small fixed overrides through 334.
fn tile_ladder_head(t: &Tile) -> i32 {
    let kind = t.kind;
    let fx = t.frame_x as i32;
    let fy = t.frame_y as i32;
    let mut num = 0i32;
    if kind == 216 {
        num = -1;
    }
    if kind == 324 {
        num = if fy != 0 { 281 + fx / 18 } else { 280 };
    }
    if kind == 216 {
        num = -1; // `:66698` — the C#'s own duplicate arm, kept.
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
    // `:66718-66757` — the else-if chain.
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
    // `:66758-66785` — the second else-if chain.
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
    // `:66786-66789` — the stone family (Main.tileStone included).
    if matches!(
        kind,
        1 | 17 | 38 | 39 | 41 | 43 | 44 | 481 | 482 | 483 | 48 | 85 | 90 | 92 | 96 | 97
            | 99 | 117 | 130 | 131 | 132 | 135 | 142 | 143 | 144 | 210 | 207 | 235 | 247
            | 272 | 273 | 283 | 410 | 480 | 509 | 618 | 657 | 658 | 677 | 678 | 679 | 720
            | 721 | 725 | 733
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
    num
}

/// Ladder frame tables 1 (`WorldGen.cs:67364-67723`): the 101 veto,
/// the type-19 row table (the row-30 arm rolls **Main.rand**.Next(3) —
/// the ladder's one non-genRand draw), the 79/18 row switches, the
/// 14/87/88/469 veto, the 255-261/262-268/178 gem bands, the 440
/// column table, the 426-439/727-732 switch, and the 186/187/647/648
/// frame ladders.
fn tile_ladder_frames1(t: &Tile, num: i32, rng: &mut UnifiedRandom) -> i32 {
    let kind = t.kind;
    let fx = t.frame_x as i32;
    let fy = t.frame_y as i32;
    let mut num = num;
    if kind == 101 {
        num = -1;
    }
    if kind == 19 {
        // `:67368-67423` — the row table.
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
            // `:67402` — the ONE Main.rand draw in the ladder.
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
    if kind == 79 {
        // `:67424-67428`.
        let row = fy / 36;
        num = match row {
            0 => 7,
            1 => 77,
            2 => 78,
            3 => 79,
            4 => 126,
            8 => 109,
            r if r < 9 => 1,
            _ => -1,
        };
    }
    if kind == 18 {
        // `:67429-67481`.
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
    if (255..=261).contains(&kind) {
        // `:67486-67494`.
        let off = kind as i32 - 255;
        num = 86 + off;
        if off == 6 {
            num = 138;
        }
    }
    if (262..=268).contains(&kind) {
        // `:67495-67503`.
        let off = kind as i32 - 262;
        num = 86 + off;
        if off == 6 {
            num = 138;
        }
    }
    if kind == 178 {
        // `:67504-67512`.
        let col = fx / 18;
        num = 86 + col;
        if col == 6 {
            num = 138;
        }
    }
    if kind == 440 {
        // `:67513-67530`.
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
    // `:67531-67572` — the gem-colour switch.
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
    if kind == 186 {
        // `:67573-67576` — the ternary nest, verbatim order.
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
            if fx > 1386 { 80 } else { 10 }
        } else {
            0
        };
    }
    if kind == 187 {
        // `:67577-67627` — the <= ladder; the second `<= 1132` arm
        // (`:67603`, num = 155) is DEAD in the C# (the first shadows
        // it) — the port's else-if chain shadows it identically.
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
    if kind == 647 {
        // `:67628-67663`.
        let col = fx / 54;
        if col < 7 {
            num = 26;
        } else if col < 16 {
            num = 1;
        } else if col < 18 {
            num = 9;
        } else if col < 20 {
            num = 11;
        } else if col < 22 {
            num = 10;
        } else if col < 26 {
            num = 7;
        } else if col < 32 {
            num = 80;
        } else if col < 35 {
            num = 80;
        }
    }
    if kind == 648 {
        // `:67664-67723`.
        let idx = fx / 54 + fy / 36 * 35;
        if idx < 3 {
            num = 1;
        } else if idx < 6 {
            num = 38;
        } else if idx < 9 {
            num = 36;
        } else if idx < 14 {
            num = 30;
        } else if idx < 17 {
            num = 1;
        } else if idx < 18 {
            num = 1;
        } else if idx < 21 {
            num = 148;
        } else if idx < 29 {
            num = 155;
        } else if idx < 35 {
            num = 287;
        } else if idx < 41 {
            num = 240;
        } else if idx < 47 {
            num = 236;
        } else if idx < 50 {
            num = 0;
        } else if idx < 52 {
            num = 2;
        } else if idx < 55 {
            num = 26;
        }
    }
    num
}

/// Ladder frame tables 2 (`WorldGen.cs:67724-68159`): 706/105/349/
/// 337-506, the 239 bar table, the 185 two-branch frame ladders, 649,
/// 650 (the 73-81 tail switch), 237/157/158-family/159, the 15 veto
/// and 191.
fn tile_ladder_frames2(t: &Tile, num: i32) -> i32 {
    let kind = t.kind;
    let fx = t.frame_x as i32;
    let fy = t.frame_y as i32;
    let mut num = num;
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
    if kind == 239 {
        // `:67744-67839` — the bar table (sequential ifs, not else).
        let col = fx / 18;
        match col {
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
    if kind == 185 {
        // `:67840-67974` — the two frame ladders.
        if fy == 18 {
            let col = fx / 36;
            if col < 6 {
                num = 1;
            } else if col < 16 {
                num = 26;
            } else if col == 16 {
                num = 9;
            } else if col == 17 {
                num = 11;
            } else if col == 18 {
                num = 10;
            } else if col == 19 {
                num = 86;
            } else if col == 20 {
                num = 87;
            } else if col == 21 {
                num = 88;
            } else if col == 22 {
                num = 89;
            } else if col == 23 {
                num = 90;
            } else if col == 24 {
                num = 91;
            } else if col < 31 {
                num = 80;
            } else if col < 33 {
                num = 7;
            } else if col < 34 {
                num = 8;
            } else if col < 38 {
                num = 30;
            } else if col < 41 {
                num = 1;
            } else if col < 47 {
                num = 287;
            } else if col < 53 {
                num = 240;
            } else if col < 59 {
                num = 236;
            }
        } else {
            let col = fx / 18;
            if col < 6 {
                num = 1;
            } else if col < 12 {
                num = 0;
            } else if col < 28 {
                num = 26;
            } else if col < 33 {
                num = 1;
            } else if col < 36 {
                num = 0;
            } else if col < 48 {
                num = 80;
            } else if col < 54 {
                num = 30;
            } else if col < 60 {
                num = 287;
            } else if col < 66 {
                num = 240;
            } else if col < 72 {
                num = 236;
            } else if col < 73 {
                num = 0;
            } else if col < 77 {
                num = 32;
            }
        }
    }
    if kind == 649 {
        // `:67975-68066`.
        let idx = fx / 36 + fy / 18 * 53;
        if idx < 6 {
            num = 1;
        } else if idx < 16 {
            num = 26;
        } else if idx == 16 {
            num = 9;
        } else if idx == 17 {
            num = 11;
        } else if idx == 18 {
            num = 10;
        } else if idx == 19 {
            num = 86;
        } else if idx == 20 {
            num = 87;
        } else if idx == 21 {
            num = 88;
        } else if idx == 22 {
            num = 89;
        } else if idx == 23 {
            num = 90;
        } else if idx == 24 {
            num = 91;
        } else if idx < 31 {
            num = 80;
        } else if idx < 33 {
            num = 7;
        } else if idx < 34 {
            num = 8;
        } else if idx < 38 {
            num = 30;
        } else if idx < 41 {
            num = 1;
        } else if idx < 47 {
            num = 287;
        } else if idx < 53 {
            num = 240;
        } else if idx < 59 {
            num = 236;
        } else if idx < 62 {
            num = 0;
        } else if idx < 64 || (65..=67).contains(&idx) {
            num = 32;
        } else if idx == 64 {
            num = 40;
        }
    }
    if kind == 650 {
        // `:68067-68135`.
        let col = fx / 18;
        if col < 6 {
            num = 1;
        } else if col < 12 {
            num = 0;
        } else if col < 28 {
            num = 26;
        } else if col < 33 {
            num = 1;
        } else if col < 36 {
            num = 0;
        } else if col < 48 {
            num = 80;
        } else if col < 54 {
            num = 30;
        } else if col < 60 {
            num = 287;
        } else if col < 66 {
            num = 240;
        } else if col < 72 {
            num = 236;
        } else if col < 73 {
            num = 0;
        } else {
            // `:68116-68133` — the tail switch with the inner if.
            match col {
                73 | 74 | 76 | 78 | 79 | 80 | 81 => num = 32,
                _ => {
                    if col == 75 || col == 77 {
                        num = 40;
                    }
                }
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
    num
}

/// The `Main.tileSolid` walk predicate (`:68195`/`:68241`): the C#
/// loops while the cell exists and is `!active() || !tileSolid[type]`.
/// Out-of-world cells are `null` in the C# (the loop stops); the port's
/// `world.tile` returns `None` there — same stop.
fn walk_stops(world: &World, x: i32, y: i32) -> bool {
    match world.tile(x, y) {
        None => true,
        Some(t) => t.active() && td::TILE_SOLID.has(t.kind),
    }
}

/// Ladder walks + ores (`WorldGen.cs:68160-68503`): the type-5 tree
/// column walk (the seven frame-offset arms, the duplicated
/// `44/132-176` arm kept), the type-323 palm walk, 137, 443/444,
/// 212-215, 325, 251-253, the 254 frame-band genRand arms, 467, the 21
/// chest ternary nest, 382, the 2/477 genRand arm, 127, 91, 198, 34,
/// the ore families, 166-169/175-177, 199/662, 205/201/211, the 227
/// column switch, the 204/478 + 243 + 244 genRand arms, 219, 642, and
/// the 597/big-furniture-set else-if with its Next(3).
fn tile_ladder_walks_ores(
    t: &Tile,
    i: i32,
    j: i32,
    num: i32,
    world: &World,
    gen: &mut UnifiedRandom,
) -> i32 {
    let kind = t.kind;
    let fx = t.frame_x as i32;
    let fy = t.frame_y as i32;
    let mut num = num;
    if kind == 5 {
        // `:68160-68234` — the tree trunk's ground read.
        num = 7;
        if i > 5 && i < world.width - 5 {
            let mut col = i;
            if fx == 66 && fy <= 45 {
                col += 1;
            }
            if fx == 88 && (66..=110).contains(&fy) {
                col -= 1;
            }
            if fx == 22 && (132..=176).contains(&fy) {
                col -= 1;
            }
            if fx == 44 && (132..=176).contains(&fy) {
                col += 1;
            }
            // `:68183-68186` — the C#'s duplicated arm, kept (a second
            // +1 on the same condition).
            if fx == 44 && (132..=176).contains(&fy) {
                col += 1;
            }
            if fx == 44 && fy >= 198 {
                col += 1;
            }
            if fx == 66 && fy >= 198 {
                col -= 1;
            }
            let mut k = j;
            while !walk_stops(world, col, k) {
                k += 1;
            }
            if let Some(g) = world.tile(col, k) {
                if g.active() && g.kind == 23 {
                    num = 77;
                }
                if g.active() && g.kind == 661 {
                    num = 77;
                }
                if g.active() && g.kind == 60 {
                    num = 78;
                }
                if g.active() && g.kind == 70 {
                    num = 26;
                }
                if g.active() && g.kind == 109 {
                    num = 79;
                }
                if g.active() && g.kind == 199 {
                    num = 121;
                }
                if g.active() && g.kind == 662 {
                    num = 121;
                }
                if g.active() && g.kind == 147 {
                    num = 122;
                }
            }
        }
    }
    if kind == 323 {
        // `:68235-68260` — the palm trunk's ground read.
        num = 215;
        if i > 5 && i < world.width - 5 {
            let mut k = j;
            while !walk_stops(world, i, k) {
                k += 1;
            }
            if let Some(g) = world.tile(i, k) {
                if g.active() && g.kind == 234 {
                    num = 121;
                }
                if g.active() && g.kind == 116 {
                    num = 79;
                }
                if g.active() && g.kind == 112 {
                    num = 77;
                }
            }
        }
    }
    if kind == 137 {
        // `:68261-68278`.
        num = match fy / 18 {
            1 | 2 | 3 | 4 => 148,
            5 => 1,
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
    if kind == 254 {
        // `:68319-68349` — the frame-band genRand arms (the <72 band
        // draws NOTHING).
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
    if kind == 21 {
        // `:68354-68357` — the chest ternary nest, verbatim order.
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
        // `:68362-68365` — genRand arm.
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
    if kind == 227 {
        // `:68430-68463` — the column switch (no default).
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
        // `:68464-68471` — genRand arm.
        num = 117;
        if gen.next_max(2) == 0 {
            num = 1;
        }
    }
    if kind == 203 {
        num = 117;
    }
    if kind == 243 {
        // `:68476-68479` — genRand arm.
        num = if gen.next_max(2) != 0 { 13 } else { 7 };
    }
    if kind == 219 {
        num = -1;
    }
    if kind == 642 {
        num = -128;
    }
    if kind == 244 {
        // `:68488-68491` — genRand arm (== 0 polarity).
        num = if gen.next_max(2) == 0 { 1 } else { 13 };
    }
    // `:68492-68503` — the 597 else-if with the big furniture set.
    if kind == 597 {
        num = -1;
    } else if matches!(
        kind,
        358..=364 | 275..=282 | 285 | 286 | 288..=297 | 316..=318 | 298 | 299 | 309
            | 310 | 339 | 538 | 413 | 414 | 505 | 521..=527 | 532 | 543 | 544 | 550
            | 551 | 533 | 553..=556 | 558 | 559 | 542 | 391 | 394 | 392 | 393 | 568
            | 569 | 570 | 582 | 580 | 598..=612 | 619 | 620 | 629 | 632 | 640 | 643
            | 644 | 645 | 710
    ) {
        num = 13;
        if gen.next_max(3) != 0 {
            num = -1;
        }
    }
    num
}

/// Ladder furniture + herbs (`WorldGen.cs:68504-68825`): 13, 189,
/// 717-719, the first 460 arm, the 530 below-tile read, 705, the
/// 518/519/636/528 else-if chain, 12/639/3-73/54/22-140/78, the
/// 28/653 frame bands, 163/164/200, the 221-225/248-250 family, 229,
/// the 231 genRand arm, 226, 103, 29/40, the herb families, 485, the
/// 238 genRand arm, 69/655, 71-family/70/112/123/161/206/162, and the
/// 165/693/694 column table.
fn tile_ladder_furniture_herbs(
    t: &Tile,
    i: i32,
    j: i32,
    num: i32,
    world: &World,
    gen: &mut UnifiedRandom,
) -> i32 {
    let kind = t.kind;
    let fx = t.frame_x as i32;
    let fy = t.frame_y as i32;
    let mut num = num;
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
        num = 16; // `:68524` — the SECOND 460 arm (`:68846`) overrides.
    }
    if kind == 530 {
        // `:68528-68537` — the tile two below (frame-adjusted).
        let below = world.tile(i, j + 2 - fy / 18);
        num = match below.map(|b| b.kind) {
            Some(116) => 47,
            Some(234) => 125,
            Some(112) => 17,
            _ => {
                if fx >= 270 {
                    291
                } else {
                    40
                }
            }
        };
    }
    if kind == 705 {
        // `:68538-68543`.
        let idx = fx / 54 + fy / 36 * 10;
        let flag = fx % 486 < 270;
        num = if (0..=8).contains(&idx) {
            if flag { 291 } else { 40 }
        } else if (9..=17).contains(&idx) {
            47
        } else if idx < 18 || idx > 26 {
            17
        } else {
            125
        };
    }
    // `:68544-68593` — the else-if chain.
    if kind == 518 {
        match fy {
            0 => num = 3,
            18 => num = 47,
            36 => num = 40,
            _ => {}
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
    if matches!(kind, 28 | 653) {
        // `:68618-68665` — the sequential frame bands.
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
        // `:68702-68709` — genRand arm.
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
        // `:68766-68769` — genRand arm.
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
    if matches!(kind, 165 | 693 | 694) {
        // `:68806-68825` — the column table.
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
    num
}

/// Ladder tail (`WorldGen.cs:68826-69058`): 666, the 193-197 family
/// (the second 460 arm), 153-156, the 116-family, the 109/492 genRand
/// arm, the herb-colour families, 133/134 frame flips, 149, the
/// Main.tileAlch table, the 58/37/32/352 genRand arms, the 23/25/26
/// sapling arms (the 31/696 + frameX>=36 short-circuit skips the
/// draw), the 20 column table, the 27 genRand arm, 129, the 385
/// Next(68,71), the type-4 torch table, the 35 + 34/42 genRand gates,
/// the late vetoes, 321/574/322/635, and `flag2` (178/440 or the
/// 426-440/727-732 gem set). Returns `(num, flag2)`.
fn tile_ladder_tail(
    t: &Tile,
    num: i32,
    gen: &mut UnifiedRandom,
) -> (i32, bool) {
    let kind = t.kind;
    let fx = t.frame_x as i32;
    let fy = t.frame_y as i32;
    let mut num = num;
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
        num = 108; // `:68846` — overrides the earlier 16.
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
        // `:68874-68877` — genRand arm.
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
    if crate::tileobject::tile_alch(kind) {
        // `:68914-68945` — Main.tileAlch, the sequential column ifs.
        let col = fx / 18;
        match col {
            0 => num = 3,
            1 => num = 3,
            2 => num = 7,
            3 => num = 17,
            4 => num = 289,
            5 => num = 6,
            6 => num = 224,
            _ => {}
        }
    }
    if matches!(kind, 58 | 76 | 77 | 684) {
        // `:68946-68949` — genRand arm.
        num = if gen.next_max(2) != 0 { 25 } else { 6 };
    }
    if kind == 37 {
        // `:68950-68953` — genRand arm.
        num = if gen.next_max(2) != 0 { 23 } else { 6 };
    }
    if kind == 32 {
        // `:68954-68957` — genRand arm.
        num = if gen.next_max(2) != 0 { 24 } else { 14 };
    }
    if kind == 352 {
        // `:68958-68961` — genRand arm.
        num = if gen.next_max(3) != 0 { 125 } else { 5 };
    }
    if matches!(kind, 23 | 24 | 661) {
        // `:68962-68965` — genRand arm.
        num = if gen.next_max(2) != 0 { 17 } else { 14 };
    }
    if matches!(kind, 25 | 31 | 696) {
        // `:68966-68969` — the 31/696 mature arm SHORT-CIRCUITS the
        // genRand draw (the C# ternary's condition first).
        num = if (kind == 31 || kind == 696) && fx >= 36 {
            5
        } else if gen.next_max(2) != 0 {
            1
        } else {
            14
        };
    }
    if matches!(kind, 26 | 695) {
        // `:68970-68973` — the frameX>=54 arm short-circuits likewise.
        num = if fx >= 54 {
            5
        } else if gen.next_max(2) != 0 {
            1
        } else {
            14
        };
    }
    if kind == 20 {
        // `:68974-68985` — the sapling column table.
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
        // `:68986-68989` — genRand arm.
        num = if gen.next_max(2) != 0 { 19 } else { 3 };
    }
    if kind == 129 {
        // `:68990-68997` — the >=324 write is OVERWRITTEN by the
        // unconditional ternary on the next line (the C#'s own dead
        // write, kept in order).
        if fx >= 324 {
            num = 69;
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
        // `:68998-69001` — genRand arm.
        num = gen.next_range(68, 71);
    }
    if kind == 4 {
        // `:69002-69006` — the torch table (MathHelper.Clamp on the
        // INTEGER division frameY/22, 0..Count-1).
        let idx = (fy / 22).clamp(0, TORCH_DUST.len() as i32 - 1);
        num = TORCH_DUST[idx as usize];
    }
    if kind == 35 {
        // `:69007-69014` — the genRand gate is SHORT-CIRCUITED by
        // frameX >= 36 (the && evaluates left first).
        num = 189;
        if fx < 36 && gen.next_max(2) == 0 {
            num = 6;
        }
    }
    if matches!(kind, 34 | 42) && gen.next_max(2) == 0 {
        // `:69015-69018` — the draw runs for EVERY 34/42 tile.
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
    // `:69059-69084` — flag2: the 178/440 base OR the gem-sparkle set.
    let mut flag2 = matches!(kind, 178 | 440);
    if matches!(
        kind,
        178 | 426 | 427 | 430 | 431 | 432 | 433 | 434 | 435 | 436 | 437 | 438 | 439
            | 440 | 727 | 728 | 729 | 730 | 731 | 732
    ) {
        flag2 = true;
    }
    (num, flag2)
}

/// `WorldGen.KillTile_MakeTileDust(i, j, tileCache)`
/// (`WorldGen.cs:66687-69169`): the tile-break dust. The type ladder
/// runs FIRST (the seven chunks above, in C# order), then the spawn
/// tail when `num >= 0`. `disco` is the `(DiscoR, DiscoG, DiscoB)`
/// snapshot. Returns the spawned slot, or [`DUST_SENTINEL`] when the
/// C# returns 6000 (num < 0, or the pool refused).
///
/// The tail's arm order is load-bearing: 627/628/184-col-10 disco →
/// 518 liquid lift → 352-num5 → 160/692 disco → 323 palm offset →
/// 314 track (genRand speeds) → flag2 → 193/18-4/19-4 blue → 197
/// cyan → 185-86..91 → 4-num66 torch disco → num139 (+Main.rand.Next(4))
/// → 722 damp → the plain tail.
#[allow(clippy::too_many_arguments)]
pub fn kill_tile_make_tile_dust(
    i: i32,
    j: i32,
    t: &Tile,
    disco: (u8, u8, u8),
    rng: &mut UnifiedRandom,
    gen: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> usize {
    let mut num = tile_ladder_head(t);
    num = tile_ladder_switch1(t, num, gen);
    num = tile_ladder_middle(t, i, j, num, world, gen);
    num = tile_ladder_frames1(t, num, rng);
    num = tile_ladder_frames2(t, num);
    num = tile_ladder_walks_ores(t, i, j, num, world, gen);
    num = tile_ladder_furniture_herbs(t, i, j, num, world, gen);
    let (num, flag2) = tile_ladder_tail(t, num, gen);
    if num < 0 {
        return DUST_SENTINEL; // `:69168`.
    }
    let kind = t.kind;
    let fx = t.frame_x as i32;
    let fy = t.frame_y as i32;
    let pos = Vec2::new((i * 16) as f32, (j * 16) as f32);
    // `:69087-69093` — the moss-disco arm.
    if kind == 627 || kind == 628 || (kind == 184 && fx / 22 == 10) {
        let idx = dust.new_dust(
            rng,
            ctx,
            world,
            pos,
            16,
            16,
            num,
            0.0,
            0.0,
            0,
            Color::new(disco.0, disco.1, disco.2, 255),
            1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.no_light_emittance = true;
        return idx;
    }
    // `:69094-69102` — the 518 liquid lift.
    if kind == 518 {
        let mut lift = t.liquid as i32 / 16 - 3;
        if crate::edit::solid_tile(world, i, j - 1) && lift > 8 {
            lift = 8;
        }
        return dust.new_dust(
            rng,
            ctx,
            world,
            Vec2::new(pos.x, pos.y - lift as f32),
            16,
            16,
            num,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
    }
    // `:69103-69111` — the 352 num==5 blood burst.
    if kind == 352 && num == 5 {
        let idx = dust.new_dust(
            rng, ctx, world, pos, 16, 16, 5, 0.0, 0.0, 100, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.scale = 1.5;
        d.no_gravity = true;
        d.velocity *= 1.65;
        d.fade_in = 1.6;
        return idx;
    }
    // `:69112-69117` — the 160/692 disco puff.
    if matches!(kind, 160 | 692) {
        let idx = dust.new_dust(
            rng,
            ctx,
            world,
            pos,
            16,
            16,
            66,
            0.0,
            0.0,
            100,
            Color::new(disco.0, disco.1, disco.2, 255),
            0.75,
        );
        dust.slot_mut(idx).no_gravity = true;
        return idx;
    }
    // `:69118-69121` — the 323 palm offset (frameY rides X).
    if kind == 323 {
        return dust.new_dust(
            rng,
            ctx,
            world,
            Vec2::new(pos.x + fy as f32, pos.y),
            16,
            16,
            num,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
    }
    // `:69122-69129` — the 314 track spark: the two genRand speed
    // arguments BEFORE the internals, the fadeIn draw AFTER (it reads
    // the post-spawn scale), the duplicated noGravity write kept.
    if kind == 314 {
        let sx = gen.next_range(-2, 3) as f32;
        let sy = gen.next_range(-2, 3) as f32;
        let idx = dust.new_dust(
            rng, ctx, world, pos, 16, 16, 213, sx, sy, 0, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.fade_in = d.scale + 1.0 + 0.01 * gen.next_range(0, 51) as f32;
        d.no_gravity = true;
        return idx;
    }
    // `:69130-69135` — the gem-sparkle flag2 arm.
    if flag2 {
        let idx = dust.new_dust(
            rng, ctx, world, pos, 16, 16, num, 0.0, 0.0, 75, Color::default(), 0.75,
        );
        dust.slot_mut(idx).no_light = true;
        return idx;
    }
    // `:69136-69139` — the blue family.
    if kind == 193 || (kind == 18 && num == 4) || (kind == 19 && num == 4) {
        return dust.new_dust(
            rng,
            ctx,
            world,
            pos,
            16,
            16,
            num,
            0.0,
            0.0,
            75,
            Color::new(0, 80, 255, 100),
            0.75,
        );
    }
    // `:69140-69143` — the 197 cyan.
    if kind == 197 {
        return dust.new_dust(
            rng,
            ctx,
            world,
            pos,
            16,
            16,
            num,
            0.0,
            0.0,
            75,
            Color::new(97, 200, 255, 100),
            0.75,
        );
    }
    // `:69144-69149` — the 185 ore-band noLight arm.
    if kind == 185 && (86..=91).contains(&num) {
        let idx = dust.new_dust(
            rng, ctx, world, pos, 16, 16, num, 0.0, 0.0, 75, Color::default(), 0.75,
        );
        dust.slot_mut(idx).no_light = true;
        return idx;
    }
    // `:69150-69155` — the torch-disco arm. The C# float ctor
    // `new Color((float)R/255f, …)` goes through XNA/MonoGame's
    // `(byte)(v*255f)` per channel — the disco bytes round-trip
    // (modulo the C#'s own f32 divide/multiply rounding, kept
    // verbatim). The repo convention agrees: `dust_table.rs:66`,
    // `render/gore.rs:80-83`, e4's `color_f`, g61's pylon colour
    // (research/66 §1 arm 11 — this replaces g14's direct [0,1]
    // truncation, which landed every channel 0 except full-255).
    if kind == 4 && num == 66 {
        let ch = |b: u8| (b as f32 / 255.0 * 255.0).clamp(0.0, 255.0) as u8;
        let idx = dust.new_dust(
            rng,
            ctx,
            world,
            pos,
            16,
            16,
            num,
            0.0,
            0.0,
            0,
            Color::new(ch(disco.0), ch(disco.1), ch(disco.2), 255),
            1.0,
        );
        dust.slot_mut(idx).no_gravity = true;
        return idx;
    }
    // `:69156-69159` — the 139 family: the Main.rand.Next(4) argument
    // draw runs BEFORE the internals.
    if num == 139 {
        let ty = num + rng.next_max(4);
        return dust.new_dust(
            rng, ctx, world, pos, 16, 16, ty, 0.0, 0.0, 0, Color::default(), 1.0,
        );
    }
    // `:69160-69165` — the 722 damp.
    if kind == 722 {
        let idx = dust.new_dust(
            rng, ctx, world, pos, 16, 16, num, 0.0, 0.0, 0, Color::default(), 1.0,
        );
        dust.slot_mut(idx).velocity *= 0.4;
        return idx;
    }
    // `:69166` — the plain tail.
    dust.new_dust(
        rng, ctx, world, pos, 16, 16, num, 0.0, 0.0, 0, Color::default(), 1.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> World {
        World::new("test".into(), 900, 400)
    }

    fn ctx() -> SpawnCtx {
        SpawnCtx {
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(14400.0, 6400.0),
            max_dust_to_draw: 6000,
            ..SpawnCtx::default()
        }
    }

    fn tile(kind: u16, fx: i16, fy: i16) -> Tile {
        Tile {
            kind,
            frame_x: fx,
            frame_y: fy,
            flags: crate::tile::FLAG_ACTIVE,
            ..Tile::default()
        }
    }

    fn streams_agree(a: &mut UnifiedRandom, b: &mut UnifiedRandom) {
        for _ in 0..16 {
            assert_eq!(a.next(), b.next());
        }
    }

    /// W4-g14 — site 69166: a plain stone (kind 1) resolves num 1 with
    /// ZERO ladder draws on either stream, and spawns one type-1 dust.
    #[test]
    fn w4_g14_tile_plain_stone_no_ladder_draws() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(3);
        let mut gen = UnifiedRandom::new(5);
        let mut rtail = UnifiedRandom::new(3);
        let mut gtail = UnifiedRandom::new(5);
        let t = tile(1, 0, 0);
        let idx = kill_tile_make_tile_dust(
            400, 100, &t, (0, 0, 0), &mut rng, &mut gen, &mut pool, &c, &w,
        );
        assert_ne!(idx, DUST_SENTINEL);
        assert_eq!(pool.slot(idx).dust_type, 1);
        // The spawn internals drew on rng; the ladder drew nothing on
        // gen. Replay the one spawn on the mirror and compare tails.
        let mut mpool = DustPool::new();
        mpool.new_dust(
            &mut rtail, &c, &w, Vec2::new(6400.0, 1600.0), 16, 16, 1,
            0.0, 0.0, 0, Color::default(), 1.0,
        );
        streams_agree(&mut rng, &mut rtail);
        streams_agree(&mut gen, &mut gtail);
    }

    /// W4-g14 — the negative-num veto: kind 216 resolves -1 and spawns
    /// nothing (the C#'s return 6000), no draws at all.
    #[test]
    fn w4_g14_tile_negative_num_returns_sentinel() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        let mut gen = UnifiedRandom::new(11);
        let mut rtail = UnifiedRandom::new(7);
        let mut gtail = UnifiedRandom::new(11);
        let t = tile(216, 0, 0);
        let idx = kill_tile_make_tile_dust(
            400, 100, &t, (0, 0, 0), &mut rng, &mut gen, &mut pool, &c, &w,
        );
        assert_eq!(idx, DUST_SENTINEL);
        streams_agree(&mut rng, &mut rtail);
        streams_agree(&mut gen, &mut gtail);
    }

    /// W4-g14 — site 69089: the 627 moss-disco arm — the ladder's moss
    /// set resolves 267, the spawn carries the disco bytes, noGravity
    /// and noLightEmittance.
    #[test]
    fn w4_g14_tile_627_moss_disco() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(13);
        let mut gen = UnifiedRandom::new(17);
        let t = tile(627, 0, 0);
        let idx = kill_tile_make_tile_dust(
            400, 100, &t, (10, 20, 30), &mut rng, &mut gen, &mut pool, &c, &w,
        );
        let d = pool.slot(idx);
        assert_eq!(d.dust_type, 267);
        assert_eq!(d.color, Color::new(10, 20, 30, 255));
        assert!(d.no_gravity);
        assert!(d.no_light_emittance);
    }

    /// W4-g14 — site 69124: the 314 track spark draws its two genRand
    /// speed arguments BEFORE the internals and the fadeIn draw AFTER
    /// (reading the post-spawn scale). The mirror replays the exact
    /// three-draw gen order + the rng internals.
    #[test]
    fn w4_g14_tile_314_track_gen_order() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(19);
        let mut gen = UnifiedRandom::new(23);
        let mut mpool = DustPool::new();
        let mut mrng = UnifiedRandom::new(19);
        let mut mgen = UnifiedRandom::new(23);
        let t = tile(314, 0, 0);
        let idx = kill_tile_make_tile_dust(
            400, 100, &t, (0, 0, 0), &mut rng, &mut gen, &mut pool, &c, &w,
        );
        let sx = mgen.next_range(-2, 3) as f32;
        let sy = mgen.next_range(-2, 3) as f32;
        let midx = mpool.new_dust(
            &mut mrng, &c, &w, Vec2::new(6400.0, 1600.0), 16, 16, 213,
            sx, sy, 0, Color::default(), 1.0,
        );
        let md = mpool.slot_mut(midx);
        md.no_gravity = true;
        md.fade_in = md.scale + 1.0 + 0.01 * mgen.next_range(0, 51) as f32;
        md.no_gravity = true;
        assert_eq!(pool.slot(idx).dust_type, 213);
        assert_eq!(pool.slot(idx).velocity, mpool.slot(midx).velocity);
        assert_eq!(pool.slot(idx).fade_in, mpool.slot(midx).fade_in);
        assert!(pool.slot(idx).no_gravity);
        streams_agree(&mut rng, &mut mrng);
        streams_agree(&mut gen, &mut mgen);
    }

    /// W4-g14 — site 69158: the num==139 arm (kind 454) draws
    /// Main.rand.Next(4) BEFORE the internals; the type lands in
    /// 139..=142 and the mirror's stream agrees.
    #[test]
    fn w4_g14_tile_139_plus_rand4() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(29);
        let mut gen = UnifiedRandom::new(31);
        let mut mpool = DustPool::new();
        let mut mrng = UnifiedRandom::new(29);
        let t = tile(454, 0, 0);
        let idx = kill_tile_make_tile_dust(
            400, 100, &t, (0, 0, 0), &mut rng, &mut gen, &mut pool, &c, &w,
        );
        let ty = 139 + mrng.next_max(4);
        mpool.new_dust(
            &mut mrng, &c, &w, Vec2::new(6400.0, 1600.0), 16, 16, ty,
            0.0, 0.0, 0, Color::default(), 1.0,
        );
        assert_eq!(pool.slot(idx).dust_type, ty);
        assert!((139..=142).contains(&pool.slot(idx).dust_type));
        streams_agree(&mut rng, &mut mrng);
    }

    /// W4-g14 — site 69105: the 352 num==5 arm forces scale 1.5,
    /// noGravity, velocity ×1.65, fadeIn 1.6 — but only when the
    /// ladder's Next(3) landed 5; the gen stream must carry that one
    /// draw either way.
    #[test]
    fn w4_g14_tile_352_gen_draw_always() {
        let w = world();
        let c = ctx();
        // Find a gen seed whose Next(3) == 0 (num = 5) and one whose
        // != 0 (num = 125): both must consume exactly one gen draw.
        for seed in [37i32, 41, 43, 47] {
            let mut probe = UnifiedRandom::new(seed);
            let roll = probe.next_max(3);
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(53);
            let mut gen = UnifiedRandom::new(seed);
            let mut gtail = UnifiedRandom::new(seed);
            let _ = gtail.next_max(3);
            let t = tile(352, 0, 0);
            let idx = kill_tile_make_tile_dust(
                400, 100, &t, (0, 0, 0), &mut rng, &mut gen, &mut pool, &c, &w,
            );
            if roll == 0 {
                let d = pool.slot(idx);
                assert_eq!(d.dust_type, 5);
                assert_eq!(d.scale, 1.5);
                assert_eq!(d.fade_in, 1.6);
                assert!(d.no_gravity);
            } else {
                assert_eq!(pool.slot(idx).dust_type, 125);
            }
            streams_agree(&mut gen, &mut gtail);
        }
    }

    /// W4-g14 — the type-19 row-30 arm is the ladder's ONE Main.rand
    /// draw: it rides rng (not gen), before the spawn internals.
    #[test]
    fn w4_g14_tile_19_row30_main_rand_draw() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(59);
        let mut gen = UnifiedRandom::new(61);
        let mut mpool = DustPool::new();
        let mut mrng = UnifiedRandom::new(59);
        let mut gtail = UnifiedRandom::new(61);
        let t = tile(19, 0, 30 * 18);
        let idx = kill_tile_make_tile_dust(
            400, 100, &t, (0, 0, 0), &mut rng, &mut gen, &mut pool, &c, &w,
        );
        let ty = 68 + mrng.next_max(3);
        mpool.new_dust(
            &mut mrng, &c, &w, Vec2::new(6400.0, 1600.0), 16, 16, ty,
            0.0, 0.0, 0, Color::default(), 1.0,
        );
        assert_eq!(pool.slot(idx).dust_type, ty);
        streams_agree(&mut rng, &mut mrng);
        streams_agree(&mut gen, &mut gtail); // the ladder drew nothing on gen
    }

    /// W4-g14 (rewritten per research/66 §1 arm 11) — site 69152: the
    /// torch-disco arm's float ctor round-trips the bytes: XNA's
    /// `(byte)(v*255f)` undoes the `/255f`, so (200,100,255) lands
    /// (200,100,255) modulo the f32 divide/multiply rounding — NOT the
    /// direct [0,1] truncation this witness originally pinned. The
    /// torch table resolves row 14 → 66.
    #[test]
    fn w4_g14_tile_torch_disco_float_ctor_round_trip() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(67);
        let mut gen = UnifiedRandom::new(71);
        let t = tile(4, 0, 14 * 22);
        let idx = kill_tile_make_tile_dust(
            400, 100, &t, (200, 100, 255), &mut rng, &mut gen, &mut pool, &c, &w,
        );
        let d = pool.slot(idx);
        assert_eq!(d.dust_type, 66);
        let ch = |b: u8| (b as f32 / 255.0 * 255.0).clamp(0.0, 255.0) as u8;
        assert_eq!(d.color, Color::new(ch(200), ch(100), ch(255), 255));
        assert!(d.no_gravity);
    }

    /// W4-g14 — site 69101: the 518 liquid lift — liquid/16 - 3, capped
    /// at 8 only when the tile above is solid. The ladder's 518 arm
    /// picks the type by frameY (0 → 3).
    #[test]
    fn w4_g14_tile_518_liquid_lift_cap() {
        let mut w = world();
        let c = ctx();
        // No solid above: lift = 255/16 - 3 = 12 (uncapped).
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(73);
        let mut gen = UnifiedRandom::new(79);
        let mut t = tile(518, 0, 0);
        t.liquid = 255;
        let idx = kill_tile_make_tile_dust(
            400, 100, &t, (0, 0, 0), &mut rng, &mut gen, &mut pool, &c, &w,
        );
        assert_eq!(pool.slot(idx).position.y, 1600.0 - 12.0);
        // Solid above: capped at 8.
        if let Some(above) = w.tile_mut(400, 99) {
            above.kind = 1;
            above.flags = crate::tile::FLAG_ACTIVE;
        }
        let mut pool2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(73);
        let mut gen2 = UnifiedRandom::new(79);
        let idx2 = kill_tile_make_tile_dust(
            400, 100, &t, (0, 0, 0), &mut rng2, &mut gen2, &mut pool2, &c, &w,
        );
        assert_eq!(pool2.slot(idx2).position.y, 1600.0 - 8.0);
    }

    /// W4-g14 — the 25/31/696 sapling arm: the mature 31/696 frame
    /// short-circuits the genRand draw; the immature arm draws one
    /// Next(2). Both paths pinned on the gen stream.
    #[test]
    fn w4_g14_tile_sapling_short_circuit() {
        let w = world();
        let c = ctx();
        // Mature 31 (fx >= 36): num 5, NO gen draw.
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(83);
        let mut gen = UnifiedRandom::new(89);
        let mut gtail = UnifiedRandom::new(89);
        let t = tile(31, 36, 0);
        let idx = kill_tile_make_tile_dust(
            400, 100, &t, (0, 0, 0), &mut rng, &mut gen, &mut pool, &c, &w,
        );
        assert_eq!(pool.slot(idx).dust_type, 5);
        streams_agree(&mut gen, &mut gtail);
        // Immature 25: one Next(2) draw, num 1 or 14.
        let mut probe = UnifiedRandom::new(89);
        let roll = probe.next_max(2);
        let mut pool2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(83);
        let mut gen2 = UnifiedRandom::new(89);
        let mut gtail2 = UnifiedRandom::new(89);
        let _ = gtail2.next_max(2);
        let t2 = tile(25, 0, 0);
        let idx2 = kill_tile_make_tile_dust(
            400, 100, &t2, (0, 0, 0), &mut rng2, &mut gen2, &mut pool2, &c, &w,
        );
        assert_eq!(pool2.slot(idx2).dust_type, if roll != 0 { 1 } else { 14 });
        streams_agree(&mut gen2, &mut gtail2);
    }
}

/// Ladder switch 1 (`WorldGen.cs:66846-67078`): the big furniture/
/// no-dust switch, the 184 column table, the 512-516/541/590 arms, the
/// seven 583-589 genRand.Next(10) arms, 595/596/615/616, the 633
/// Next(6), 637/638 and the 634 Next(10)+Next(12) pair.
fn tile_ladder_switch1(t: &Tile, num: i32, gen: &mut UnifiedRandom) -> i32 {
    let kind = t.kind;
    let fx = t.frame_x as i32;
    let mut num = num;
    match kind {
        10 | 11 | 87 | 89 | 93 | 139 | 209 | 319 | 320 | 386 | 387 | 390 | 405 | 406
        | 411 | 412 | 419..=425 | 428 | 429 | 441 | 442 | 445..=453 | 455..=457
        | 462..=466 | 468 | 476 | 486 | 487 | 489..=491 | 493 | 494 | 497 | 510 | 511
        | 520..=527 | 531 | 545 | 547 | 548 | 560 | 564 | 565 | 567 | 572 | 579 | 591
        | 592 | 593 | 594 | 613 | 614 | 621..=624 | 630 | 631 | 656 | 700 | 701 | 726 => {
            num = -1;
        }
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
            // `:66995-67009` — the column table.
            let col = fx / 22;
            num = match col {
                5 => 258,
                6 => 299,
                7 => 300,
                8 => 301,
                9 => 305,
                10 => 267,
                _ => 93 + col,
            };
        }
        515 => num = 96,
        516 => num = 97,
        514 => num = 95,
        513 => num = 94,
        512 => num = 93,
        541 => num = 226,
        590 => num = 1,
        // `:67031-67051` — the seven Next(10) arms.
        583 => num = if gen.next_max(10) != 0 { 1 } else { 87 },
        584 => num = if gen.next_max(10) != 0 { 1 } else { 86 },
        585 => num = if gen.next_max(10) != 0 { 1 } else { 88 },
        586 => num = if gen.next_max(10) != 0 { 1 } else { 89 },
        587 => num = if gen.next_max(10) != 0 { 1 } else { 90 },
        588 => num = if gen.next_max(10) != 0 { 1 } else { 91 },
        589 => num = if gen.next_max(10) != 0 { 1 } else { 138 },
        595 | 596 | 615 | 616 => num = 78,
        // `:67064-67066` — Next(6).
        633 => num = if gen.next_max(6) != 0 { 237 } else { 36 },
        637 | 638 => num = 237,
        634 => {
            // `:67071-67077` — Next(10) then Next(12), both always.
            num = if gen.next_max(10) != 0 { 36 } else { 31 };
            if gen.next_max(12) == 0 {
                num = 6;
            }
        }
        _ => {}
    }
    num
}

/// Ladder middle (`WorldGen.cs:67079-67363`): the moss set, the 240
/// banner column, 241/242, the 529 below-tile read (beach band), the
/// fixed overrides through 741, and the 742-750 switch.
fn tile_ladder_middle(
    t: &Tile,
    i: i32,
    j: i32,
    num: i32,
    world: &World,
    gen: &mut UnifiedRandom,
) -> i32 {
    let kind = t.kind;
    let fx = t.frame_x as i32;
    let fy = t.frame_y as i32;
    let mut num = num;
    // `:67079-67082` — Main.tileMoss.
    if td::TILE_MOSS.has(kind) {
        num = match kind {
            381 => 258,
            534 => 299,
            536 => 300,
            539 => 301,
            625 => 305,
            627 => 267,
            _ => kind as i32 - 179 + 93,
        };
    }
    if kind == 240 {
        // `:67083-67103` — the banner column.
        let mut col = fx / 54;
        if fy >= 54 {
            col += 36 * (fy / 54);
        }
        num = 7;
        if matches!(col, 16 | 17) {
            num = 26;
        }
        if (46..=49).contains(&col) {
            num = -1;
        }
        if col == 100 {
            num = -1;
        }
    }
    if kind == 241 {
        num = 1;
    }
    if kind == 242 {
        num = -1;
    }
    if kind == 529 {
        // `:67112-67121` — the tile below picks the sand family.
        let below = world.tile(i, j + 1);
        num = match below.map(|b| b.kind) {
            Some(116) => 47,
            Some(234) => 125,
            Some(112) => 17,
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
        // `:67170-67173` — genRand arm.
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
        // `:67198-67201` — genRand arm.
        num = if gen.next_max(2) != 0 { 23 } else { 6 };
    }
    if kind == 171 {
        // `:67202-67205` — genRand arm.
        num = if gen.next_max(2) != 0 { -1 } else { 196 };
    }
    if kind == 326 {
        num = 13;
    }
    if kind == 327 {
        num = 13;
    }
    if kind == 345 {
        num = 13;
    }
    if kind == 708 {
        num = 13;
    }
    if kind == 458 {
        num = 13;
    }
    if kind == 459 {
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
    if kind == 328 {
        num = 13;
    }
    if kind == 329 {
        num = 13;
    }
    if kind == 507 {
        num = 13;
    }
    if kind == 508 {
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
    num
}
