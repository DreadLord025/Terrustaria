//! W4-e14 — FindFrame extra wave 2: the SECOND 15 remaining missing
//! case ids of `NPC.FindFrame` (`NPC.cs:66684-77370`), counted off the
//! ascending census (`ff_missing` — every C# case id whose port has no
//! arm) after the d46-d49 claims. The first 15 (37/43/46/47/48/56/61/
//! 62/63/64/67/69/73/74/83) ride `task/W4-e13-findframe-extra-1`; THIS
//! module owns **84/93/94/101/102/103/116/120/121/137/141/142/148/149/
//! 181** — the Enchanted Sword, the Giant Bat, the Corruptor, the
//! Clinger, the Angler Fish, the Green Jellyfish, the Hungry II, the
//! Chaos Elemental, the Slimer, the Illuminant Bat, the Toxic Sludge,
//! Santa Claus, the two Penguins and the Face Monster
//! (`ID/NPCID.cs:10845-11039`). Landed 6 kinds per commit.
//!
//! Shape: the kinds are NOT in `NpcKind` yet (their AI/spawn machines
//! are later rounds), so — like `npc_findframe_a.rs` /
//! `npc_findframe_b.rs` (S1-D-3 waves A/B) — the frame machines land
//! first as pure functions over a small [`FrameState`]/[`FrameInput`]
//! pair. The wiring seam is [`find_frame_extra2`]: when a kind joins
//! `NpcKind`, its `find_frame` arm copies `row`/`counter`/`sprite_dir`/
//! `rotation`/`local_ai0` out, runs one tick and copies them back (the
//! `Npc::find_frame_wave_a` bridge precedent). No RNG is touched (none
//! of these 15 arms draws `Main.rand` — the bat group's dust spawns are
//! types 60/634's, the eye group's is type 23's, and the humanoid town
//! pose zones' chat-bubble draws are the documented skip below), so
//! the wiring cannot move a stream.
//!
//! Row convention: the port tracks frame ROW indices, not pixels — the
//! C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer),
//! counted entry-by-entry in [`FindFrameExtra2Kind::frame_count`] and
//! cross-checked against the port's own `render/entities.rs::
//! NPC_FRAME_COUNT` table.
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "W4-e14 FindFrame
//! extra wave 2"):
//! * the bat group's type-60 dust (`NPC.cs:74839-74843`) and the
//!   type-634 `Main.rand.Next(10)` dust (`:74844-74849`) — neither is
//!   93/137; the 210/211 double-increment + `vx * 0.2` tilt
//!   (`:74859-74863`) and the 49/51/60/634 `num292--` (`:74867-74870`)
//!   are other kinds' too (93/137 flap the FULL 4 rows every 6 ticks);
//! * the eye group's type-23 dust (`:74007-74013`) and the 660-only
//!   6-tick cadence (`:74046-74049`) — the Slimer 121 rides the SECOND
//!   flip branch (`:74032-74043`, the vx-sign flip + `vx * 0.1` tilt)
//!   because the first list (`:74019`) does not carry 121, and its
//!   `localAI[0]` init (`:74014-74018`) IS carried (the row-2 pin);
//! * the fish group's out-of-slice siblings 55/57/58/241/465/592/607
//!   (`:74181-74188`) — the Angler Fish 102 shares the wet/dry bands;
//! * the jellyfish group's 256 exclusion from the pulse gate
//!   (`:73921` — the list is 63/64/103/242, so 103 pulses on
//!   `ai[1] == 1`);
//! * the humanoid group's town `ai[0]` sit/talk/attack pose zones
//!   (`:68662-69488`, including their chat-bubble `Main.rand` draws
//!   `:69412`/`:69436`) — the port's town machines do not model them
//!   (the `npc.rs::humanoid_town_frame` / W4-d47 Steampunker
//!   precedent); the walk/idle/air tail (`:69489-69655`) IS carried,
//!   with Santa's `ExtraFramesCount[142]` = 9 wrap (`NPCID.cs:10464`,
//!   `townNPC = true` `NPC.cs:10793`) vs the fighters' 0;
//! * the cursed-hammer group's 83/179 siblings (`:74622-74624`) — the
//!   Enchanted Sword 84 shares the spin/bonk arm (179 rides d47).

// The wiring is a later round (the kinds are not in `NpcKind` yet) —
// the `npc_findframe_a.rs`/`npc_findframe_b.rs` precedent.
#![allow(dead_code)]

/// The 15 extra-wave-2 kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra2Kind {
    /// `NPCID.EnchantedSword` = 84 (`NPCID.cs:10845`) — `case 84`
    /// (the 83/179 cursed-hammer group).
    EnchantedSword,
    /// `NPCID.GiantBat` = 93 (`NPCID.cs:10863`) — `case 93` (the
    /// 48/49/51/60/82/… bat/floater group).
    GiantBat,
    /// `NPCID.Corruptor` = 94 (`NPCID.cs:10865`) — `case 94`.
    Corruptor,
    /// `NPCID.Clinger` = 101 (`NPCID.cs:10879`) — `case 101`.
    Clinger,
    /// `NPCID.AnglerFish` = 102 (`NPCID.cs:10881`) — `case 102` (the
    /// 55/57/58/… fish group).
    AnglerFish,
    /// `NPCID.GreenJellyfish` = 103 (`NPCID.cs:10883`) — `case 103`
    /// (the 63/64/242/256 jellyfish group).
    GreenJellyfish,
    /// `NPCID.TheHungryII` = 116 (`NPCID.cs:10909`) — `case 116`.
    TheHungryII,
    /// `NPCID.ChaosElemental` = 120 (`NPCID.cs:10917`) — `case 120`
    /// (the humanoid fighter group).
    ChaosElemental,
    /// `NPCID.Slimer` = 121 (`NPCID.cs:10919`) — `case 121` (the
    /// case-2 eye group).
    Slimer,
    /// `NPCID.IlluminantBat` = 137 (`NPCID.cs:10951`) — `case 137`
    /// (the bat/floater group).
    IlluminantBat,
    /// `NPCID.ToxicSludge` = 141 (`NPCID.cs:10959`) — `case 141`.
    ToxicSludge,
    /// `NPCID.SantaClaus` = 142 (`NPCID.cs:10961`) — `case 142` (the
    /// humanoid TOWN group, `townNPC = true`).
    SantaClaus,
    /// `NPCID.Penguin` = 148 (`NPCID.cs:10973`) — `case 148` (the
    /// 149/168/470 penguin group).
    Penguin,
    /// `NPCID.PenguinBlack` = 149 (`NPCID.cs:10975`) — `case 149`.
    PenguinBlack,
    /// `NPCID.FaceMonster` = 181 (`NPCID.cs:11039`) — `case 181`
    /// (the humanoid fighter group).
    FaceMonster,
}

/// Every kind of this wave, in ascending vanilla-id order (the census
/// order; the strip-sweep witness walks it).
pub const ALL: [FindFrameExtra2Kind; 15] = [
    FindFrameExtra2Kind::EnchantedSword,
    FindFrameExtra2Kind::GiantBat,
    FindFrameExtra2Kind::Corruptor,
    FindFrameExtra2Kind::Clinger,
    FindFrameExtra2Kind::AnglerFish,
    FindFrameExtra2Kind::GreenJellyfish,
    FindFrameExtra2Kind::TheHungryII,
    FindFrameExtra2Kind::ChaosElemental,
    FindFrameExtra2Kind::Slimer,
    FindFrameExtra2Kind::IlluminantBat,
    FindFrameExtra2Kind::ToxicSludge,
    FindFrameExtra2Kind::SantaClaus,
    FindFrameExtra2Kind::Penguin,
    FindFrameExtra2Kind::PenguinBlack,
    FindFrameExtra2Kind::FaceMonster,
];

impl FindFrameExtra2Kind {
    /// The vanilla `NPC.type` (`ID/NPCID.cs`'s const rows).
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::EnchantedSword => 84,
            Self::GiantBat => 93,
            Self::Corruptor => 94,
            Self::Clinger => 101,
            Self::AnglerFish => 102,
            Self::GreenJellyfish => 103,
            Self::TheHungryII => 116,
            Self::ChaosElemental => 120,
            Self::Slimer => 121,
            Self::IlluminantBat => 137,
            Self::ToxicSludge => 141,
            Self::SantaClaus => 142,
            Self::Penguin => 148,
            Self::PenguinBlack => 149,
            Self::FaceMonster => 181,
        }
    }

    /// `Main.npcFrameCount[type]` — the row counts counted entry-by-entry
    /// out of the 697-entry initializer (`Main.cs:67082-67153`; entry
    /// `type` sits on line `67084 + type / 10`, slot `type % 10`),
    /// cross-checked against `render/entities.rs::NPC_FRAME_COUNT`.
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67092 (entries 80-89): 84 = 6.
            Self::EnchantedSword => 6,
            // Main.cs:67093 (entries 90-99): 93 = 4, 94 = 3.
            Self::GiantBat => 4,
            Self::Corruptor => 3,
            // Main.cs:67094 (entries 100-109): 101 = 5, 102 = 6,
            // 103 = 7.
            Self::Clinger => 5,
            Self::AnglerFish => 6,
            Self::GreenJellyfish => 7,
            // Main.cs:67095 (entries 110-119): 116 = 6.
            Self::TheHungryII => 6,
            // Main.cs:67096 (entries 120-129): 120 = 15, 121 = 4.
            Self::ChaosElemental => 15,
            Self::Slimer => 4,
            // Main.cs:67097 (entries 130-139): 137 = 4.
            Self::IlluminantBat => 4,
            // Main.cs:67098 (entries 140-149): 141 = 3, 142 = 25,
            // 148/149 = 12.
            Self::ToxicSludge => 3,
            Self::SantaClaus => 25,
            Self::Penguin => 12,
            Self::PenguinBlack => 12,
            // Main.cs:67102 (entries 180-189): 181 = 16.
            Self::FaceMonster => 16,
        }
    }
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection`,
/// `rotation`, and `localAI[0]` (the Slimer's one-shot init writes it,
/// `NPC.cs:74014-74018`; the penguin group reads it as the colour bank).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter`.
    pub counter: f32,
    /// `NPC.spriteDirection` (+/-1).
    pub sprite_dir: i32,
    /// `NPC.rotation`.
    pub rotation: f32,
    /// `NPC.localAI[0]` — the Slimer init latch / the penguin colour
    /// bank (the d48 `FrameState::local_ai0` precedent).
    pub local_ai0: f32,
}

/// The per-tick inputs the arms read: `velocity`, `direction`, the `ai`
/// slots, `localAI[0]` (the penguin bank) and `wet` (the fish group's
/// swim/flop split, `NPC.cs:74191`).
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (+/-1; 0 leaves `spriteDirection` alone in
    /// the arms that gate on `direction == 1` / `== -1`, but the
    /// unconditional-copy arms write it verbatim).
    pub direction: i32,
    pub ai: [f32; 4],
    /// `NPC.localAI[0]` — the penguin group's +3/+6/+9 colour bank
    /// (`NPC.cs:76398-76409`); every other arm ignores it.
    pub local_ai0: f32,
    /// `Entity.wet` — the fish group's swim bands (`NPC.cs:74191`).
    pub wet: bool,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self {
            vx: 0.0,
            vy: 0.0,
            direction: 1,
            ai: [0.0; 4],
            local_ai0: 0.0,
            wet: false,
        }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-2 kinds — one
/// tick.
pub fn find_frame_extra2(kind: FindFrameExtra2Kind, s: &mut FrameState, i: &FrameInput) {
    let count = kind.frame_count();
    match kind {
        // `case 83 : case 84 : case 179` (`NPC.cs:74622-74641`): the
        // cursed-hammer spin — `ai[0] == 2` (the bonked state) pins
        // row 0 and zeroes the counter; else the 4-tick counter walks
        // the strip and wraps at the frame count. The wrap check sits
        // INSIDE the step branch (the C#'s nesting), not after it.
        FindFrameExtra2Kind::EnchantedSword => {
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
        // `case 93` / `case 137` in the bat/floater group
        // (`NPC.cs:74823-74884`, shared with 48/49/51/60/82/182/210/
        // 211/253/316/634/662/664): the spriteDirection rides the
        // velocity's sign (0 leaves it alone), the tilt is
        // `rotation = vx * 0.1`, and the flap steps every 6 ticks
        // (`num291` = 6 — the 5 is type 48's, `:74871-74874`) over the
        // FULL 4 rows (`num292` — the `--` list 49/51/60/634
        // `:74867-74870` carries neither 93 nor 137). The 60/634 dust
        // spawns and the 210/211 double-increment are other kinds'.
        FindFrameExtra2Kind::GiantBat | FindFrameExtra2Kind::IlluminantBat => {
            // `:74850-74857`.
            if i.vx > 0.0 {
                s.sprite_dir = 1;
            }
            if i.vx < 0.0 {
                s.sprite_dir = -1;
            }
            // `:74858`.
            s.rotation = i.vx * 0.1;
            // `:74864` (the single increment — 210/211's extra one at
            // `:74861` is not this group's).
            s.counter += 1.0;
            // `:74875-74879`.
            if s.counter >= 6.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            // `:74880-74883`.
            if s.row >= count {
                s.row = 0;
            }
        }
        // `case 94` (`NPC.cs:76943-76965`): the Corruptor's four-band
        // float — <6 row 0, <12 row 1, <18 row 2, else row 1 (the tail
        // writes `num`, NOT `num * 3` — the 3-row sheet's middle row
        // rides the long tail) with the counter resetting at >= 23.
        FindFrameExtra2Kind::Corruptor => {
            s.counter += 1.0;
            if s.counter < 6.0 {
                // `:76945-76949`.
                s.row = 0;
            } else if s.counter < 12.0 {
                // `:76950-76954`.
                s.row = 1;
            } else if s.counter < 18.0 {
                // `:76955-76959`.
                s.row = 2;
            } else {
                // `:76960-76964` — the tail holds row 1.
                s.row = 1;
                if s.counter >= 23.0 {
                    s.counter = 0.0;
                }
            }
        }
        // `case 101` (`NPC.cs:75018-75029`): the Clinger's +2 stride —
        // the >6 counter steps TWO rows, and the `> num * 2` wrap
        // fires on row 4 (4 > 2), so the 5-row sheet only ever shows
        // rows 0 and 2 (row 4 is written then immediately wrapped —
        // the vanilla quirk kept verbatim).
        FindFrameExtra2Kind::Clinger => {
            s.counter += 1.0;
            // `:75020-75024`.
            if s.counter > 6.0 {
                s.row += 2;
                s.counter = 0.0;
            }
            // `:75025-75028`.
            if s.row > 2 {
                s.row = 0;
            }
        }
        // `case 102` in the fish group (`NPC.cs:74181-74226`, shared
        // with 55/57/58/241/465/592/607): `spriteDirection = direction`
        // UNCONDITIONALLY, then the wet/dry split — wet rides the
        // <6/<12/<18/<24 bands over rows 0/1/2/3 (the >= 24 tail
        // zeroes the counter and HOLDS the row), dry rides <6/<12 over
        // rows 4/5 (the >= 12 tail likewise holds).
        FindFrameExtra2Kind::AnglerFish => {
            // `:74189-74190`.
            s.sprite_dir = i.direction;
            s.counter += 1.0;
            if i.wet {
                // `:74191-74213`.
                if s.counter < 6.0 {
                    s.row = 0;
                } else if s.counter < 12.0 {
                    s.row = 1;
                } else if s.counter < 18.0 {
                    s.row = 2;
                } else if s.counter < 24.0 {
                    s.row = 3;
                } else {
                    // `:74209-74212` — the row holds.
                    s.counter = 0.0;
                }
            } else if s.counter < 6.0 {
                // `:74214-74217`.
                s.row = 4;
            } else if s.counter < 12.0 {
                // `:74218-74221`.
                s.row = 5;
            } else {
                // `:74222-74225` — the row holds.
                s.counter = 0.0;
            }
        }
        // `case 103` in the jellyfish group (`NPC.cs:73916-73960`,
        // shared with 63/64/242/256): the pulse gate
        // (`(type 63/64/103/242) && ai[1] == 1`, `:73921` — 103 IS in
        // the list, 256 is not) cycles rows 4..6 on the >4 counter
        // with BOTH clamps (the first pulse tick lifts any row into
        // the band) and RETURNS early; else the idle bands <6/<12/<18
        // over rows 0/1/2 with the row-3 tail resetting at >= 23.
        FindFrameExtra2Kind::GreenJellyfish => {
            if i.ai[1] == 1.0 {
                // `:73921-73938`.
                s.counter += 1.0;
                if s.counter > 4.0 {
                    s.counter = 0.0;
                    s.row += 1;
                }
                if s.row > 6 {
                    s.row = 4;
                }
                if s.row < 4 {
                    s.row = 4;
                }
                return;
            }
            // `:73939-73959`.
            s.counter += 1.0;
            if s.counter < 6.0 {
                s.row = 0;
            } else if s.counter < 12.0 {
                s.row = 1;
            } else if s.counter < 18.0 {
                s.row = 2;
            } else {
                s.row = 3;
                if s.counter >= 23.0 {
                    s.counter = 0.0;
                }
            }
        }
        // The remaining nine kinds land in the next commits.
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn rows_over(kind: FindFrameExtra2Kind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra2(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The 15 `Main.npcFrameCount` rows, counted entry-by-entry out of
    /// the `Main.cs:67082` initializer and cross-checked against the
    /// port's `NPC_FRAME_COUNT` table.
    #[test]
    fn w4_e14_the_frame_counts_match_the_vanilla_table() {
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

    /// `NPC.cs:74631-74640`: the Enchanted Sword spins every 4 ticks
    /// over its 6 rows; `ai[0] == 2` (the bonk) pins row 0 + counter 0.
    #[test]
    fn w4_e14_the_enchanted_sword_spins_and_bonks() {
        let rows = rows_over(FindFrameExtra2Kind::EnchantedSword, input(0.0, 0.0), 24);
        assert_eq!(&rows[..8], &[0, 0, 0, 1, 1, 1, 1, 2]);
        assert_eq!(rows[23], 0, "the wrap past row 5 lands back on 0");
        let mut s = FrameState { row: 3, counter: 2.0, ..Default::default() };
        let i = FrameInput { ai: [2.0, 0.0, 0.0, 0.0], ..Default::default() };
        find_frame_extra2(FindFrameExtra2Kind::EnchantedSword, &mut s, &i);
        assert_eq!((s.row, s.counter), (0, 0.0), "the bonk pins row 0");
    }

    /// `NPC.cs:74850-74883`: the Giant/Illuminant bats flap every 6
    /// ticks over the FULL 4 rows (no `num292--`), tilt `vx * 0.1`,
    /// and freeze the flip at `vx == 0`.
    #[test]
    fn w4_e14_the_bats_flap_six_ticks_over_four_rows() {
        for kind in [FindFrameExtra2Kind::GiantBat, FindFrameExtra2Kind::IlluminantBat] {
            let rows = rows_over(kind, input(1.0, 0.0), 24);
            assert_eq!(&rows[..7], &[0, 0, 0, 0, 0, 1, 1]);
            assert_eq!(rows[23], 0, "the 4-row strip wraps");
            let mut s = FrameState { sprite_dir: -1, ..Default::default() };
            find_frame_extra2(kind, &mut s, &input(0.0, 0.0));
            assert_eq!(s.sprite_dir, -1, "vx == 0 leaves the flip alone");
            assert_eq!(s.rotation, 0.0);
            find_frame_extra2(kind, &mut s, &input(-2.0, 0.0));
            assert_eq!(s.sprite_dir, -1);
            assert_eq!(s.rotation, -2.0 * 0.1);
        }
    }

    /// `NPC.cs:76943-76965`: the Corruptor's bands 0/1/2 then the
    /// row-1 tail (NOT row 3), the counter resetting at 23.
    #[test]
    fn w4_e14_the_corruptor_floats_the_row_one_tail() {
        let rows = rows_over(FindFrameExtra2Kind::Corruptor, input(0.0, 0.0), 23);
        assert_eq!(rows[0], 0);
        assert_eq!(rows[5], 1, "tick 6 enters the <12 band");
        assert_eq!(rows[11], 2, "tick 12 enters the <18 band");
        assert_eq!(rows[17], 1, "tick 18 enters the row-1 tail");
        assert_eq!(rows[22], 1, "the tail holds row 1 through tick 23");
        let mut s = FrameState::default();
        let i = input(0.0, 0.0);
        for _ in 0..23 {
            find_frame_extra2(FindFrameExtra2Kind::Corruptor, &mut s, &i);
        }
        assert_eq!(s.counter, 0.0, "the >= 23 reset fires on tick 23");
    }

    /// `NPC.cs:75018-75029`: the Clinger's +2 stride only ever shows
    /// rows 0 and 2 — row 4 is written then wrapped by `> 2`.
    #[test]
    fn w4_e14_the_clinger_strides_two_and_wraps_past_two() {
        let rows = rows_over(FindFrameExtra2Kind::Clinger, input(0.0, 0.0), 21);
        assert_eq!(&rows[..8], &[0, 0, 0, 0, 0, 0, 2, 2]);
        assert_eq!(rows[13], 0, "the second stride writes 4 then wraps to 0");
        assert!(rows.iter().all(|&r| r == 0 || r == 2), "rows 1/3/4 never ride: {rows:?}");
    }

    /// `NPC.cs:74189-74225`: the Angler Fish swims rows 0..3 wet and
    /// flops rows 4/5 dry; both tails zero the counter and HOLD the
    /// row; the flip copies `direction` unconditionally (even 0).
    #[test]
    fn w4_e14_the_angler_fish_swims_wet_and_flops_dry() {
        let wet = FrameInput { wet: true, ..input(0.0, 0.0) };
        let rows = rows_over(FindFrameExtra2Kind::AnglerFish, wet, 24);
        assert_eq!(rows[0], 0);
        assert_eq!(rows[5], 1);
        assert_eq!(rows[11], 2);
        assert_eq!(rows[17], 3);
        assert_eq!(rows[23], 3, "the >= 24 tail holds row 3");
        let dry = rows_over(FindFrameExtra2Kind::AnglerFish, input(0.0, 0.0), 12);
        assert_eq!(dry[0], 4);
        assert_eq!(dry[5], 5);
        assert_eq!(dry[11], 5, "the >= 12 tail holds row 5");
        let mut s = FrameState { sprite_dir: 1, ..Default::default() };
        let i = FrameInput { direction: 0, ..Default::default() };
        find_frame_extra2(FindFrameExtra2Kind::AnglerFish, &mut s, &i);
        assert_eq!(s.sprite_dir, 0, "the unconditional copy writes direction 0");
    }

    /// `NPC.cs:73921-73959`: the Green Jellyfish pulses rows 4..6 on
    /// `ai[1] == 1` (both clamps lift the first tick into the band,
    /// early return) and idles the 0/1/2/3 bands otherwise.
    #[test]
    fn w4_e14_the_green_jellyfish_pulses_and_idles() {
        let pulse = FrameInput { ai: [0.0, 1.0, 0.0, 0.0], ..input(0.0, 0.0) };
        let rows = rows_over(FindFrameExtra2Kind::GreenJellyfish, pulse, 15);
        assert_eq!(rows[0], 4, "the low clamp lifts row 0 into the band");
        assert_eq!(&rows[..6], &[4, 4, 4, 4, 5, 5]);
        assert_eq!(rows[14], 4, "the >6 wrap returns to row 4");
        let idle = rows_over(FindFrameExtra2Kind::GreenJellyfish, input(0.0, 0.0), 23);
        assert_eq!(idle[0], 0);
        assert_eq!(idle[5], 1);
        assert_eq!(idle[11], 2);
        assert_eq!(idle[17], 3);
        assert_eq!(idle[22], 3, "the >= 23 reset tick still shows row 3");
    }
}
