//! W4-d47 — FindFrame mid-band slice 2: the SECOND 12 missing case ids
//! of `NPC.FindFrame` (`NPC.cs:66684-77370`) in the 150-350 band, counted
//! off the wave census (`ff_missing` — every C# case id whose port has no
//! arm): slice 1 (`task/W4-d46-findframe-mid-1`) owns 150/151/152/156/157/
//! 158/159/160/162/163/164/165; THIS module owns **166/168/169/170/171/
//! 172/173/175/176/178/179/180** — the Swamp Thing, the Corrupt Penguin,
//! the Ice Elemental, the three Pigrons, the Rune Wizard, the Crimera,
//! the Angry Trapper, the Moss Hornet, the Steampunker and the Crimson
//! Axe (`ID/NPCID.cs:11009-11037`).
//!
//! Shape: the kinds are NOT in `NpcKind` yet (their AI/spawn machines are
//! later rounds), so — like `npc_findframe_a.rs` / `npc_findframe_b.rs`
//! (S1-D-3 waves A/B) — the frame machines land first as pure functions
//! over a small [`FrameState`]/[`FrameInput`] pair. The wiring seam is
//! [`find_frame_mid2`]: when a kind joins `NpcKind`, its `find_frame` arm
//! copies `row`/`counter`/`sprite_dir`/`rotation` out, runs one tick and
//! copies them back (the `Npc::find_frame_wave_a` bridge precedent).
//! No RNG is touched (none of these 12 arms draws `Main.rand` — the
//! case-2 eye group's dust spawn is type 23's, not 169's), so the wiring
//! cannot move a stream.
//!
//! Row convention: the port tracks frame ROW indices, not pixels — the
//! C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer), counted
//! entry-by-entry in [`FindFrameMid2Kind::frame_count`] and cross-checked
//! against the port's own `render/entities.rs::NPC_FRAME_COUNT` table.
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "W4-d47 FindFrame
//! mid-band slice 2"):
//! * the eye group's type-23 dust (`NPC.cs:74007-74013`), the 121/660
//!   `localAI[0]` init (`:74014-74018`) and the 660-only 6-tick cadence
//!   (`:74046-74049`) — none is 169;
//! * the eye group's second flip branch (`:74032-74043`) is DEAD for
//!   every member (its `type == 2` arm is shadowed by the first `if`'s
//!   list which already carries 2, and 121/660 are the only remaining
//!   members) — 169 writes NO spriteDirection/rotation at all;
//! * the caster group's `type == 32 && localAI[3] == 1` Skeletron-pose
//!   bank (`:77050-77053`) — the Dark Caster's, not the Rune Wizard's;
//! * the Steampunker's town `ai[0]` sit/talk/attack pose zones
//!   (`:68662-69488`) — the port's town machines do not model them yet
//!   (the `npc.rs::humanoid_town_frame` precedent); the walk band's
//!   `ExtraFramesCount[178]` = 9 wrap (`NPCID.cs:10464`) IS carried.

// The wiring is a later round (the kinds are not in `NpcKind` yet) — the
// `npc_findframe_a.rs`/`npc_findframe_b.rs` precedent.
#![allow(dead_code)]

/// The 12 slice-2 kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameMid2Kind {
    /// `NPCID.SwampThing` = 166 (`NPCID.cs:11009`) — `case 166`.
    SwampThing,
    /// `NPCID.CorruptPenguin` = 168 (`NPCID.cs:11013`) — `case 168`
    /// (the 148/149/470 penguin group).
    CorruptPenguin,
    /// `NPCID.IceElemental` = 169 (`NPCID.cs:11015`) — `case 169`
    /// (the case-2 eye group).
    IceElemental,
    /// `NPCID.PigronCorruption` = 170 (`NPCID.cs:11017`) — `case 170`
    /// (the 171/180 pigron group).
    PigronCorruption,
    /// `NPCID.PigronHallow` = 171 (`NPCID.cs:11019`) — `case 171`.
    PigronHallow,
    /// `NPCID.RuneWizard` = 172 (`NPCID.cs:11021`) — `case 172`
    /// (the 29/32/45 caster pose group).
    RuneWizard,
    /// `NPCID.Crimera` = 173 (`NPCID.cs:11023`) — `case 173`
    /// (shared with the Eater of Souls' `case 6`).
    Crimera,
    /// `NPCID.AngryTrapper` = 175 (`NPCID.cs:11027`) — `case 175`
    /// (the 43/56 man-eater group).
    AngryTrapper,
    /// `NPCID.MossHornet` = 176 (`NPCID.cs:11029`) — `case 176`.
    MossHornet,
    /// `NPCID.Steampunker` = 178 (`NPCID.cs:11033`) — `case 178`
    /// (the humanoid town group).
    Steampunker,
    /// `NPCID.CrimsonAxe` = 179 (`NPCID.cs:11035`) — `case 179`
    /// (the 83/84 cursed-hammer group).
    CrimsonAxe,
    /// `NPCID.PigronCrimson` = 180 (`NPCID.cs:11037`) — `case 180`.
    PigronCrimson,
}

impl FindFrameMid2Kind {
    /// The vanilla `NPC.type` (`ID/NPCID.cs`'s const rows).
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::SwampThing => 166,
            Self::CorruptPenguin => 168,
            Self::IceElemental => 169,
            Self::PigronCorruption => 170,
            Self::PigronHallow => 171,
            Self::RuneWizard => 172,
            Self::Crimera => 173,
            Self::AngryTrapper => 175,
            Self::MossHornet => 176,
            Self::Steampunker => 178,
            Self::CrimsonAxe => 179,
            Self::PigronCrimson => 180,
        }
    }

    /// `Main.npcFrameCount[type]` — the row counts counted entry-by-entry
    /// out of the 697-entry initializer (`Main.cs:67082-67153`; entry
    /// `type` sits on line `67084 + type / 10`, slot `type % 10`),
    /// cross-checked against `render/entities.rs::NPC_FRAME_COUNT`.
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67100 (entries 160-169): …, 166 = 13, 168 = 12,
            // 169 = 4.
            Self::SwampThing => 13,
            Self::CorruptPenguin => 12,
            Self::IceElemental => 4,
            // Main.cs:67101 (entries 170-179): 170/171 = 14, 172 = 3,
            // 173 = 2, 175 = 3, 176 = 2, 178 = 23, 179 = 5.
            Self::PigronCorruption => 14,
            Self::PigronHallow => 14,
            Self::RuneWizard => 3,
            Self::Crimera => 2,
            Self::AngryTrapper => 3,
            Self::MossHornet => 2,
            Self::Steampunker => 23,
            Self::CrimsonAxe => 5,
            // Main.cs:67102 (entry 180): 14.
            Self::PigronCrimson => 14,
        }
    }
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection`,
/// `rotation` (the wave-B pair's shape — none of the 12 arms writes
/// rotation, but the bridge copies all four fields uniformly).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter`.
    pub counter: f32,
    /// `NPC.spriteDirection` (+/-1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — untouched by every slice-2 arm.
    pub rotation: f32,
}

/// The per-tick inputs the arms read: `velocity`, `direction`, the `ai`
/// slots and `localAI[0]` (the penguin group's colour bank).
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (+/-1; 0 leaves `spriteDirection` alone in the
    /// arms that gate on `direction == 1` / `== -1`).
    pub direction: i32,
    pub ai: [f32; 4],
    /// `NPC.localAI[0]` — the penguin group's +3/+6/+9 colour bank
    /// (`NPC.cs:76398-76409`); every other arm ignores it.
    pub local_ai0: f32,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1, ai: [0.0; 4], local_ai0: 0.0 }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the slice-2 kinds — one tick.
pub fn find_frame_mid2(kind: FindFrameMid2Kind, s: &mut FrameState, i: &FrameInput) {
    let count = kind.frame_count();
    match kind {
        // `case 166` (`NPC.cs:75618-75664`): the Swamp Thing's three
        // bands — the grounded idle cycles rows 0..5 on the +2 counter
        // (>7 steps), the grounded walk cycles rows 7..12 on the
        // |vx|+1 counter (>9 steps, BOTH clamps), the air holds row 6.
        FindFrameMid2Kind::SwampThing => {
            if i.vy == 0.0 {
                // `:75621-75628` — only direction +/-1 writes.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                if i.vx == 0.0 {
                    // `:75629-75642`.
                    s.counter += 2.0;
                    if s.counter > 7.0 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                    if s.row > 5 {
                        s.row = 0;
                    }
                } else {
                    // `:75643-75657` — the |vx| add THEN the ++; both
                    // clamps run.
                    s.counter += i.vx.abs();
                    s.counter += 1.0;
                    if s.counter > 9.0 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                    if s.row > 12 {
                        s.row = 7;
                    }
                    if s.row < 7 {
                        s.row = 7;
                    }
                }
            } else {
                // `:75659-75663`.
                s.counter = 0.0;
                s.row = 6;
            }
        }
        // `case 148 : case 149 : case 168 : case 470`
        // (`NPC.cs:76392-76456`): the penguin walk — the localAI[0]
        // 2/3/4 colour bank offsets the whole strip +3/+6/+9 rows, the
        // grounded idle snaps to the bank's row 0, the walk rides the
        // |vx| accumulator's <6/<12/<15 bands over rows 0/1/2 (the last
        // branch zeroes the counter and HOLDS row 2), the air pins
        // row 2 (both vy-sign arms write the same row).
        FindFrameMid2Kind::CorruptPenguin => {
            // `:76397-76410` — num40, the colour bank.
            let bank: u32 = if i.local_ai0 == 2.0 {
                3
            } else if i.local_ai0 == 3.0 {
                6
            } else if i.local_ai0 == 4.0 {
                9
            } else {
                0
            };
            if i.vy == 0.0 {
                // `:76413-76420`.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                if i.vx == 0.0 {
                    // `:76421-76426`.
                    s.row = bank;
                    s.counter = 0.0;
                } else {
                    // `:76427-76444` — the band breaks; the tail
                    // zeroes the counter and holds row 2.
                    s.counter += i.vx.abs();
                    if s.counter < 6.0 {
                        s.row = bank;
                    } else if s.counter < 12.0 {
                        s.row = 1 + bank;
                    } else if s.counter < 15.0 {
                        s.row = 2 + bank;
                    } else {
                        s.counter = 0.0;
                        s.row = 2 + bank;
                    }
                }
            } else {
                // `:76446-76455` — both sign arms write row 2 + bank.
                s.counter = 0.0;
                s.row = 2 + bank;
            }
        }
        // `case 2 : … : case 169 : …` (`NPC.cs:73994-74059`): the Ice
        // Elemental rides the eye group's plain 8-tick strip over its
        // 4 rows — NO spriteDirection/rotation write (169 is in neither
        // flip list; the second branch's `type == 2` arm is dead), no
        // dust (type 23's), no localAI init (121/660's), num142 = 8
        // (the 6 override is 660's).
        FindFrameMid2Kind::IceElemental => {
            // `:74044-74058`.
            s.counter += 1.0;
            if s.counter >= 8.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= count {
                s.row = 0;
            }
        }
        // `case 170 : case 171 : case 180` (`NPC.cs:73418-73432`): the
        // pigrons' 14-row fly cycle — spriteDirection = direction
        // UNCONDITIONALLY (no ground gate), the 4-tick counter walks
        // every row and wraps past 13 to 0.
        FindFrameMid2Kind::PigronCorruption
        | FindFrameMid2Kind::PigronHallow
        | FindFrameMid2Kind::PigronCrimson => {
            // `:73421-73431`.
            s.sprite_dir = i.direction;
            s.counter += 1.0;
            if s.counter >= 4.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= 14 {
                s.row = 0;
            }
        }
        // `case 29 : case 32 : case 45 : case 172 : …`
        // (`NPC.cs:77019-77054`): the caster pose machine — row 0
        // stands, row 1 casts (`ai[1] > 0`), row 2 airborne (the air
        // gate wins over the cast); the grounded flip writes only on
        // direction +/-1. The +3 Skeletron-pose bank is type 32's.
        FindFrameMid2Kind::RuneWizard => {
            // `:77030-77040`.
            if i.vy == 0.0 {
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
            }
            // `:77041-77049`.
            s.row = 0;
            if i.vy != 0.0 {
                s.row += 2;
            } else if i.ai[1] > 0.0 {
                s.row += 1;
            }
        }
        // `case 6 : case 173` (`NPC.cs:76966-76978`): the Crimera rides
        // the Eater of Souls' plain 8-tick flap over its 2 rows — no
        // direction/rotation write in the C# arm.
        FindFrameMid2Kind::Crimera => {
            // `:76968-76977`.
            s.counter += 1.0;
            if s.counter >= 8.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= count {
                s.row = 0;
            }
        }
        // `case 43 : case 56 : case 175` (`NPC.cs:74970-74994`): the
        // man-eater group's four-phase chomp — the counter counts
        // 1..23 (six ticks a phase) and the rows run 0/1/2/1 (the
        // downstroke repeats); the `== 23` reset is EXACT (a fractional
        // counter overshoots it and the row HOLDS 1 — the C# quirk kept
        // verbatim).
        FindFrameMid2Kind::AngryTrapper => {
            // `:74973-74993`.
            s.counter += 1.0;
            if s.counter < 6.0 {
                s.row = 0;
            } else if s.counter < 12.0 {
                s.row = 1;
            } else if s.counter < 18.0 {
                s.row = 2;
            } else if s.counter < 24.0 {
                s.row = 1;
            }
            if s.counter == 23.0 {
                s.counter = 0.0;
            }
        }
        // `case 176` (`NPC.cs:74958-74969`): the Moss Hornet's 2-row
        // buzz — the >4 counter steps every 5 ticks and the row wraps
        // past 1 to 0 (`frame.Y > num`, NOT the frame-count wrap).
        FindFrameMid2Kind::MossHornet => {
            // `:74959-74968`.
            s.counter += 1.0;
            if s.counter > 4.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row > 1 {
                s.row = 0;
            }
        }
        // `case 178` (`NPC.cs:68573`, the humanoid town group
        // `:68634-69655`): the Steampunker's town walk — the grounded
        // flip (only direction +/-1 writes), the idle row 0 snap, the
        // |vx|*2+1 accumulator stepping every >6 over rows
        // 2..count-ExtraFramesCount-1 (178's extra = 9,
        // `NPCID.cs:10464` — the wrap band stops at row 13 of the
        // 23-row sheet), and the airborne row 1 (178 is NOT in the
        // `:69647` pin list). The town ai[0] pose zones are the
        // documented skip (the module header).
        FindFrameMid2Kind::Steampunker => {
            if i.vy == 0.0 {
                // `:68649-68656`.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                if i.vx == 0.0 {
                    // `:69515-69519` — 178 is not the 140/489 idle-row-1
                    // pair, not 638.
                    s.row = 0;
                    s.counter = 0.0;
                } else {
                    // `:69550-69554` — num108 = 6 (not town slime /
                    // 632 / 534 / 638 / 656 / 462).
                    s.counter += i.vx.abs() * 2.0;
                    s.counter += 1.0;
                    // `:69559-69575` — num109 = row 2.
                    if s.row < 2 {
                        s.row = 2;
                    }
                    if s.counter > 6.0 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                    // `:69581-69584` — the wrap at npcFrameCount -
                    // num57 = 23 - 9 = 14.
                    if s.row >= count - 9 {
                        s.row = 2;
                    }
                }
            } else {
                // `:69642-69650` — the group's air row 1; 178 is not
                // in the pin list.
                s.counter = 0.0;
                s.row = 1;
            }
        }
        // `case 83 : case 84 : case 179` (`NPC.cs:74622-74641`): the
        // cursed-hammer spin — `ai[0] == 2` (the bonked state) pins
        // row 0 and zeroes the counter; else the 4-tick counter walks
        // the 5-row strip and wraps at the frame count.
        FindFrameMid2Kind::CrimsonAxe => {
            if i.ai[0] == 2.0 {
                // `:74625-74630`.
                s.counter = 0.0;
                s.row = 0;
            } else {
                // `:74631-74640`.
                s.counter += 1.0;
                if s.counter >= 4.0 {
                    s.counter = 0.0;
                    s.row += 1;
                    if s.row >= count {
                        s.row = 0;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [FindFrameMid2Kind; 12] = [
        FindFrameMid2Kind::SwampThing,
        FindFrameMid2Kind::CorruptPenguin,
        FindFrameMid2Kind::IceElemental,
        FindFrameMid2Kind::PigronCorruption,
        FindFrameMid2Kind::PigronHallow,
        FindFrameMid2Kind::RuneWizard,
        FindFrameMid2Kind::Crimera,
        FindFrameMid2Kind::AngryTrapper,
        FindFrameMid2Kind::MossHornet,
        FindFrameMid2Kind::Steampunker,
        FindFrameMid2Kind::CrimsonAxe,
        FindFrameMid2Kind::PigronCrimson,
    ];

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn rows_over(kind: FindFrameMid2Kind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_mid2(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The 12 row counts agree entry-by-entry with the port's own
    /// `Main.npcFrameCount` port (`Main.cs:67100-67102`).
    #[test]
    fn w4_d47_the_frame_counts_match_the_vanilla_table() {
        for kind in ALL {
            assert_eq!(
                kind.frame_count(),
                crate::render::entities::npc_rows(kind.vanilla_type() as u16),
                "{} ({})",
                kind.vanilla_type(),
                kind.frame_count()
            );
        }
    }

    /// `NPC.cs:75629-75642`: the Swamp Thing's idle cycles rows 0..5 —
    /// the +2 counter steps on ticks 4/8/12/… (>7) and wraps past 5.
    #[test]
    fn w4_d47_the_swamp_thing_idles_rows_zero_to_five() {
        let rows = rows_over(FindFrameMid2Kind::SwampThing, input(0.0, 0.0), 24);
        assert_eq!(&rows[..8], &[0, 0, 0, 1, 1, 1, 1, 2]);
        assert_eq!(rows[23], 0, "the wrap past row 5 lands back on 0");
        assert!(rows.iter().all(|&r| r <= 5));
    }

    /// `NPC.cs:75643-75663`: the walk lifts to the 7..12 band (both
    /// clamps), the air holds row 6 and zeroes the counter.
    #[test]
    fn w4_d47_the_swamp_thing_walks_seven_to_twelve_and_flies_six() {
        let mut s = FrameState::default();
        find_frame_mid2(FindFrameMid2Kind::SwampThing, &mut s, &input(3.0, 0.0));
        assert_eq!(s.row, 7, "the low clamp lifts the idle row into the band");
        let rows = rows_over(FindFrameMid2Kind::SwampThing, input(3.0, 0.0), 20);
        assert!(rows.iter().all(|&r| (7..=12).contains(&r)));
        // The >9 step at |vx|+1 = 4: rows advance every 3 ticks.
        assert_eq!(&rows[..6], &[7, 7, 8, 8, 8, 9]);
        let mut s = FrameState { row: 9, counter: 5.0, ..Default::default() };
        find_frame_mid2(FindFrameMid2Kind::SwampThing, &mut s, &input(3.0, -2.0));
        assert_eq!((s.row, s.counter), (6, 0.0));
    }

    /// `NPC.cs:76397-76444`: the penguin's colour bank offsets every
    /// row write +3/+6/+9; the walk bands ride <6/<12/<15 and the tail
    /// zeroes the counter holding row 2.
    #[test]
    fn w4_d47_the_penguin_banks_offset_the_whole_strip() {
        for (bank_flag, bank) in [(0.0f32, 0u32), (2.0, 3), (3.0, 6), (4.0, 9)] {
            let i = FrameInput { vx: 0.0, vy: 0.0, local_ai0: bank_flag, ..Default::default() };
            let mut s = FrameState { row: 5, counter: 9.0, ..Default::default() };
            find_frame_mid2(FindFrameMid2Kind::CorruptPenguin, &mut s, &i);
            assert_eq!((s.row, s.counter), (bank, 0.0), "the idle snap");
            // |vx| = 7: tick 1 row 1+bank (7 in [6,12)), tick 2 row
            // 2+bank (14 in [12,15)), tick 3 the tail (21 >= 15).
            let walk = FrameInput { vx: 7.0, vy: 0.0, local_ai0: bank_flag, ..Default::default() };
            let mut s = FrameState::default();
            find_frame_mid2(FindFrameMid2Kind::CorruptPenguin, &mut s, &walk);
            assert_eq!(s.row, 1 + bank);
            find_frame_mid2(FindFrameMid2Kind::CorruptPenguin, &mut s, &walk);
            assert_eq!(s.row, 2 + bank);
            find_frame_mid2(FindFrameMid2Kind::CorruptPenguin, &mut s, &walk);
            assert_eq!((s.row, s.counter), (2 + bank, 0.0), "the >=15 tail");
            // The air pins row 2 + bank on BOTH vy signs.
            for vy in [-3.0f32, 3.0] {
                let air = FrameInput { vx: 1.0, vy, local_ai0: bank_flag, ..Default::default() };
                let mut s = FrameState { row: 1, counter: 4.0, ..Default::default() };
                find_frame_mid2(FindFrameMid2Kind::CorruptPenguin, &mut s, &air);
                assert_eq!((s.row, s.counter), (2 + bank, 0.0));
            }
        }
    }

    /// `NPC.cs:74044-74058`: the Ice Elemental's plain 8-tick strip
    /// over 4 rows — and NO spriteDirection/rotation write (169 is in
    /// neither flip list; the second branch is dead code for it).
    #[test]
    fn w4_d47_the_ice_elemental_flaps_without_a_flip() {
        let rows = rows_over(FindFrameMid2Kind::IceElemental, input(2.0, -1.0), 32);
        assert_eq!(&rows[..9], &[0, 0, 0, 0, 0, 0, 0, 1, 1]);
        assert_eq!(rows[31], 0, "the 4-row wrap");
        let mut s = FrameState { sprite_dir: -1, rotation: 0.5, ..Default::default() };
        find_frame_mid2(FindFrameMid2Kind::IceElemental, &mut s, &input(-2.0, 1.0));
        assert_eq!(s.sprite_dir, -1, "no flip write for 169");
        assert_eq!(s.rotation, 0.5, "no rotation write for 169");
    }

    /// `NPC.cs:73418-73432`: the pigrons flip UNCONDITIONALLY (no
    /// ground gate) and walk all 14 rows on the 4-tick counter.
    #[test]
    fn w4_d47_the_pigrons_cycle_fourteen_rows() {
        for kind in [
            FindFrameMid2Kind::PigronCorruption,
            FindFrameMid2Kind::PigronHallow,
            FindFrameMid2Kind::PigronCrimson,
        ] {
            let rows = rows_over(kind, input(0.0, -2.0), 56);
            assert_eq!(rows[3], 1, "the 4th tick steps");
            assert_eq!(rows[55], 0, "the wrap past row 13");
            assert!(rows.iter().all(|&r| r < 14));
            let mut s = FrameState { sprite_dir: 1, ..Default::default() };
            let air = FrameInput { vx: 0.0, vy: -3.0, direction: -1, ..Default::default() };
            find_frame_mid2(kind, &mut s, &air);
            assert_eq!(s.sprite_dir, -1, "the flip writes in the air too");
        }
    }

    /// `NPC.cs:77030-77049`: the Rune Wizard's caster poses — row 0
    /// stands, row 1 casts, row 2 airborne (the air gate wins over the
    /// cast); the flip only writes grounded.
    #[test]
    fn w4_d47_the_rune_wizard_rides_the_caster_poses() {
        let mut s = FrameState::default();
        find_frame_mid2(FindFrameMid2Kind::RuneWizard, &mut s, &input(0.0, 0.0));
        assert_eq!(s.row, 0);
        let cast = FrameInput { ai: [0.0, 30.0, 0.0, 0.0], ..input(0.0, 0.0) };
        find_frame_mid2(FindFrameMid2Kind::RuneWizard, &mut s, &cast);
        assert_eq!(s.row, 1, "ai[1] > 0 casts");
        let air_cast = FrameInput { vy: -2.0, ai: [0.0, 30.0, 0.0, 0.0], ..Default::default() };
        find_frame_mid2(FindFrameMid2Kind::RuneWizard, &mut s, &air_cast);
        assert_eq!(s.row, 2, "the air gate wins over the cast");
        // The grounded flip: direction 0 leaves the sprite alone.
        let mut s = FrameState { sprite_dir: 1, ..Default::default() };
        let zero = FrameInput { direction: 0, ..input(0.0, 0.0) };
        find_frame_mid2(FindFrameMid2Kind::RuneWizard, &mut s, &zero);
        assert_eq!(s.sprite_dir, 1);
    }

    /// `NPC.cs:76966-76978`: the Crimera rides the Eater of Souls'
    /// 8-tick 2-row flap — the same cadence `npc.rs`'s case-6 arm runs.
    #[test]
    fn w4_d47_the_crimera_flaps_like_the_eater_of_souls() {
        let rows = rows_over(FindFrameMid2Kind::Crimera, input(1.0, 0.0), 16);
        assert_eq!(&rows[..16], &[0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0]);
    }

    /// `NPC.cs:74973-74993`: the Angry Trapper's 0/1/2/1 chomp over the
    /// 23-tick counter, and the EXACT `== 23` reset — a fractional
    /// counter overshoots it and the row HOLDS 1 (the C# quirk).
    #[test]
    fn w4_d47_the_trapper_chomps_and_the_exact_reset_is_a_quirk() {
        let rows = rows_over(FindFrameMid2Kind::AngryTrapper, input(0.0, 0.0), 24);
        assert_eq!(&rows[..6], &[0, 0, 0, 0, 0, 1]);
        assert_eq!(&rows[11..18], &[1, 2, 2, 2, 2, 2, 1]);
        assert_eq!(rows[22], 1, "tick 23 resets the counter");
        assert_eq!(rows[23], 0, "the cycle restarts");
        // The overshoot: counter 22.5 + 1 = 23.5 misses `== 23`, the
        // row stays in the <24 band and never resets on its own.
        let mut s = FrameState { row: 1, counter: 22.5, ..Default::default() };
        find_frame_mid2(FindFrameMid2Kind::AngryTrapper, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (1, 23.5));
        find_frame_mid2(FindFrameMid2Kind::AngryTrapper, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (1, 24.5), "past 24 no band writes — the row holds");
    }

    /// `NPC.cs:74959-74968`: the Moss Hornet's >4 counter steps every
    /// 5 ticks over 2 rows (the `> num` wrap, not the count wrap).
    #[test]
    fn w4_d47_the_moss_hornet_buzzes_two_rows() {
        let rows = rows_over(FindFrameMid2Kind::MossHornet, input(0.0, 0.0), 10);
        assert_eq!(&rows[..10], &[0, 0, 0, 0, 1, 1, 1, 1, 1, 0]);
    }

    /// `NPC.cs:68634-69655` (case 178): the Steampunker's town walk
    /// wraps at 23 - 9 = 14 (the ExtraFramesCount attack rows), the
    /// idle snaps to row 0 and the air holds row 1 (not the pin list).
    #[test]
    fn w4_d47_the_steampunker_walks_the_town_band() {
        // |vx| = 1: the accumulator +3 steps every 3rd tick (>6).
        let rows = rows_over(FindFrameMid2Kind::Steampunker, input(1.0, 0.0), 36);
        assert_eq!(&rows[..6], &[2, 2, 3, 3, 3, 4]);
        assert!(rows.iter().all(|&r| (2..14).contains(&r)), "the wrap band 2..13: {rows:?}");
        assert!(rows.contains(&13), "row 13 is reachable");
        let mut s = FrameState { row: 9, counter: 4.0, ..Default::default() };
        find_frame_mid2(FindFrameMid2Kind::Steampunker, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0), "the idle snap");
        find_frame_mid2(FindFrameMid2Kind::Steampunker, &mut s, &input(0.0, -4.0));
        assert_eq!((s.row, s.counter), (1, 0.0), "the air row 1 — 178 is not pinned to 0");
    }

    /// `NPC.cs:74625-74640`: the Crimson Axe's ai[0]==2 bonk pins row 0
    /// and zeroes the counter; else the 4-tick spin wraps over 5 rows.
    #[test]
    fn w4_d47_the_crimson_axe_spins_and_bonks() {
        let rows = rows_over(FindFrameMid2Kind::CrimsonAxe, input(0.0, 0.0), 20);
        assert_eq!(&rows[..8], &[0, 0, 0, 1, 1, 1, 1, 2]);
        assert_eq!(rows[19], 0, "the 5-row wrap");
        let bonk = FrameInput { ai: [2.0, 0.0, 0.0, 0.0], ..input(3.0, 0.0) };
        let mut s = FrameState { row: 3, counter: 2.0, ..Default::default() };
        find_frame_mid2(FindFrameMid2Kind::CrimsonAxe, &mut s, &bonk);
        assert_eq!((s.row, s.counter), (0, 0.0));
    }

    /// The strip sweep (the M4-NPC-3t precedent): every arm over a
    /// grid of velocities/directions/ai/localAI states stays inside
    /// its own sheet — no machine can leave its strip.
    #[test]
    fn w4_d47_every_arm_stays_inside_its_strip() {
        for kind in ALL {
            let count = kind.frame_count();
            for vx in [-4.0f32, -0.5, 0.0, 0.5, 4.0] {
                for vy in [-3.0f32, 0.0, 3.0] {
                    for dir in [-1i32, 0, 1] {
                        for ai0 in [0.0f32, 2.0] {
                            for ai1 in [0.0f32, 30.0] {
                                for bank in [0.0f32, 2.0, 3.0, 4.0] {
                                    let i = FrameInput {
                                        vx,
                                        vy,
                                        direction: dir,
                                        ai: [ai0, ai1, 0.0, 0.0],
                                        local_ai0: bank,
                                    };
                                    let mut s = FrameState::default();
                                    for _ in 0..40 {
                                        find_frame_mid2(kind, &mut s, &i);
                                        assert!(
                                            s.row < count,
                                            "{kind:?} left its {count}-row strip: row {} (vx {vx}, vy {vy}, dir {dir}, ai0 {ai0}, ai1 {ai1}, bank {bank})",
                                            s.row
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
