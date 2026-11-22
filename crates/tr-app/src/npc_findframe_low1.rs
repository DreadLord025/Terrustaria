//! W4-d44 — FindFrame low-band slice 1: the FIRST 12 missing case ids
//! of `NPC.FindFrame` (`NPC.cs:66684-77370`) in the low band, counted
//! off the wave census (`ff_missing` — every C# top-level case id whose
//! port has no arm: the `npc.rs::find_frame` match kinds mapped through
//! the bestiary net ids + the wave-A/wave-B module rosters, diffed
//! against the 602 top-level `case` labels of the C# switch). Slice 2
//! (`task/W4-d45-findframe-low-2`) owns the SECOND 12 (69/73/74/83/84/
//! 93/94/101/102/103/116/120); THIS module owns **37/43/46/47/48/56/57/
//! 61/62/63/64/67** — the Old Man, the Man Eater, the Bunny pair, the
//! Harpy, the Snatcher, the Corrupt Goldfish, the Vulture, the Demon,
//! the jellyfish pair and the Crab (`ID/NPCID.cs:10751-10811`).
//! Landed in two 6-kind batches (37/43/46/47/48/56, then
//! 57/61/62/63/64/67).
//!
//! Shape: the kinds are NOT in `NpcKind` yet (their AI/spawn machines are
//! later rounds), so — like `npc_findframe_a.rs` / `npc_findframe_b.rs`
//! (S1-D-3 waves A/B) and `npc_findframe_mid2.rs` (W4-d47) — the frame
//! machines land first as pure functions over a small
//! [`FrameState`]/[`FrameInput`] pair. The wiring seam is
//! [`find_frame_low1`]: when a kind joins `NpcKind`, its `find_frame` arm
//! copies `row`/`counter`/`sprite_dir`/`rotation` out, runs one tick and
//! copies them back (the `Npc::find_frame_wave_a` bridge precedent).
//! No RNG is touched (none of these arms draws `Main.rand` — the
//! floater group's dust spawns are types 60/634's, not the Harpy's), so
//! the wiring cannot move a stream.
//!
//! Row convention: the port tracks frame ROW indices, not pixels — the
//! C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer), counted
//! entry-by-entry in [`FindFrameLow1Kind::frame_count`] and cross-checked
//! against the port's own `render/entities.rs::NPC_FRAME_COUNT` table.
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "W4-d44 FindFrame
//! low-band slice 1"):
//! * the humanoid town group's `ai[0]` sit/talk/attack pose zones
//!   (`NPC.cs:68662-69488`) — the port's town machines do not model them
//!   yet (the `npc.rs::humanoid_town_frame` precedent); the Old Man's
//!   walk band's `ExtraFramesCount[37]` = 5 wrap (`NPCID.cs:10464`) IS
//!   carried (37 is town-like: `townNPC = true`, `NPC.cs:9199`);
//! * the floater group's type-60 flame dust (`:74839-74843`) and the
//!   type-634 `Main.rand.Next(10)` dust (`:74844-74849`) — neither is
//!   the Harpy's; the 210/211 double-increment + 0.2 tilt
//!   (`:74859-74863`) and the 49/51/60/634 `num292--` list
//!   (`:74867-74870`) are other members' too;
//! * the jellyfish group's 103/242/256 members (`:73918-73920`) — the
//!   `ai[1] == 1` attack gate (`:73921`) DOES carry 63/64 and is ported;
//! * the disabled profile-texture block (`NPC.cs:68637`, `if (false &&
//!   …)`) — dead in the C# itself.

// The wiring is a later round (the kinds are not in `NpcKind` yet) — the
// `npc_findframe_a.rs`/`npc_findframe_mid2.rs` precedent.
#![allow(dead_code)]

/// The slice-1 kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameLow1Kind {
    /// `NPCID.OldMan` = 37 (`NPCID.cs:10751`) — `case 37` (the humanoid
    /// town group, `townNPC = true` at `NPC.cs:9199`).
    OldMan,
    /// `NPCID.ManEater` = 43 (`NPCID.cs:10763`) — `case 43` (the 56/175
    /// man-eater chomp group).
    ManEater,
    /// `NPCID.Bunny` = 46 (`NPCID.cs:10769`) — `case 46` (the critter
    /// hop group).
    Bunny,
    /// `NPCID.CorruptBunny` = 47 (`NPCID.cs:10771`) — `case 47` (the
    /// same critter group).
    CorruptBunny,
    /// `NPCID.Harpy` = 48 (`NPCID.cs:10773`) — `case 48` (the floater
    /// group, the 5-tick member).
    Harpy,
    /// `NPCID.Snatcher` = 56 (`NPCID.cs:10789`) — `case 56` (the 43/175
    /// man-eater chomp group).
    Snatcher,
    /// `NPCID.CorruptGoldfish` = 57 (`NPCID.cs:10791`) — `case 57` (the
    /// 55/58 fish group).
    CorruptGoldfish,
    /// `NPCID.Vulture` = 61 (`NPCID.cs:10799`) — `case 61`.
    Vulture,
    /// `NPCID.Demon` = 62 (`NPCID.cs:10801`) — `case 62` (shared with
    /// the Voodoo Demon's 66, already in `npc.rs`).
    Demon,
    /// `NPCID.BlueJellyfish` = 63 (`NPCID.cs:10803`) — `case 63` (the
    /// 64/103/242/256 jellyfish group).
    BlueJellyfish,
    /// `NPCID.PinkJellyfish` = 64 (`NPCID.cs:10805`) — `case 64`.
    PinkJellyfish,
    /// `NPCID.Crab` = 67 (`NPCID.cs:10811`) — `case 67` (the 217-219
    /// critter walk group).
    Crab,
}

impl FindFrameLow1Kind {
    /// The vanilla `NPC.type` (`ID/NPCID.cs`'s const rows).
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::OldMan => 37,
            Self::ManEater => 43,
            Self::Bunny => 46,
            Self::CorruptBunny => 47,
            Self::Harpy => 48,
            Self::Snatcher => 56,
            Self::CorruptGoldfish => 57,
            Self::Vulture => 61,
            Self::Demon => 62,
            Self::BlueJellyfish => 63,
            Self::PinkJellyfish => 64,
            Self::Crab => 67,
        }
    }

    /// `Main.npcFrameCount[type]` — the row counts counted entry-by-entry
    /// out of the 697-entry initializer (`Main.cs:67082-67153`; entry
    /// `type` sits on line `67084 + type / 10`, slot `type % 10`),
    /// cross-checked against `render/entities.rs::NPC_FRAME_COUNT`.
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67087 (entries 30-39): 37 = 21.
            Self::OldMan => 21,
            // Main.cs:67088 (entries 40-49): 43 = 3, 46/47 = 7, 48 = 6.
            Self::ManEater => 3,
            Self::Bunny => 7,
            Self::CorruptBunny => 7,
            Self::Harpy => 6,
            // Main.cs:67089 (entries 50-59): 56 = 3, 57 = 6.
            Self::Snatcher => 3,
            Self::CorruptGoldfish => 6,
            // Main.cs:67090 (entries 60-69): 61 = 6, 62 = 5, 63/64 = 7,
            // 67 = 8.
            Self::Vulture => 6,
            Self::Demon => 5,
            Self::BlueJellyfish => 7,
            Self::PinkJellyfish => 7,
            Self::Crab => 8,
        }
    }
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection`,
/// `rotation`. The wiring step copies them to/from `Npc`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter`.
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — only the tilt arms (48/61/62) write it.
    pub rotation: f32,
}

/// The per-tick inputs the arms read: `velocity`, `direction`, `wet` and
/// the `ai` slots (only the jellyfish group's `ai[1]` attack gate reads
/// one). `ai_action` is not carried — none of these kinds reads the
/// `num2` motion class (`NPC.cs:66696-66704`) in its arm.
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone in the
    /// arms that gate on `direction == 1` / `== -1`; the unconditional
    /// writers copy it verbatim, 0 included).
    pub direction: i32,
    pub wet: bool,
    pub ai: [f32; 4],
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1, wet: false, ai: [0.0; 4] }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the slice-1 kinds — one tick.
pub fn find_frame_low1(kind: FindFrameLow1Kind, s: &mut FrameState, i: &FrameInput) {
    let count = kind.frame_count();
    match kind {
        // `case 37` (`NPC.cs:68553` in the humanoid town case list,
        // `:68634-69655`): the Old Man is town-like (`townNPC = true`,
        // `:9199`), so `num57 = ExtraFramesCount[37]` = 5
        // (`NPCID.cs:10464`) — the walk band wraps at row 21 - 5 = 16
        // (rows 2..15). Not a town slime (`IsTownSlime` is 670/678-684),
        // so `num108` = 6 and `num109` = row 2; the idle snaps to row 0
        // (37 is not the 140/489 idle-row-1 pair nor the 638 idle
        // cycle); the air holds row 1 (37 is NOT in the `:69647` pin
        // list). The town `ai[0]` sit/talk/attack pose zones
        // (`:68662-69488`) are the documented skip — the port's town
        // machines do not model them (the `npc.rs::humanoid_town_frame`
        // precedent).
        FindFrameLow1Kind::OldMan => {
            let num57 = 5;
            if i.vy == 0.0 {
                // `:68649-68656` — only direction ±1 writes.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                if i.vx == 0.0 {
                    // `:69515-69519`.
                    s.row = 0;
                    s.counter = 0.0;
                } else {
                    // `:69550-69584` — num108 = 6, num109 = row 2, the
                    // |vx| * 2 add THEN the ++; the wrap subtracts num57.
                    if s.row < 2 {
                        s.row = 2;
                    }
                    s.counter += i.vx.abs() * 2.0;
                    s.counter += 1.0;
                    if s.counter > 6.0 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                    if s.row >= count - num57 {
                        s.row = 2;
                    }
                }
            } else {
                // `:69642-69650` — the air row 1; 37 is not in the pin
                // list, so the row stands.
                s.counter = 0.0;
                s.row = 1;
            }
        }
        // `case 43 : case 56 : case 175` (`NPC.cs:74970-74994`): the
        // man-eater chomp — the 0/1/2/1 bands over the 1..23 counter
        // (six ticks a phase). The reset is EXACT `== 23` (`:74990`) —
        // a fractional counter overshoots it, no band writes past 24
        // and the row HOLDS 1 forever; the quirk is kept verbatim (the
        // W4-d47 AngryTrapper sibling arm's identical shape).
        FindFrameLow1Kind::ManEater | FindFrameLow1Kind::Snatcher => {
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
        // `case 46 : case 47 : …` (`NPC.cs:76851-76904`, the critter hop
        // group): the grounded flip writes only on direction ±1; the
        // idle snaps to row 0 + zero counter; the walk accumulates
        // `|vx| * 1` THEN `++` (`:76882-76883`), steps at > 6 and wraps
        // at the FULL `npcFrameCount` (7); the rise pins row 4 and the
        // fall row 6, both zeroing the counter.
        FindFrameLow1Kind::Bunny | FindFrameLow1Kind::CorruptBunny => {
            if i.vy == 0.0 {
                // `:76868-76875`.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                if i.vx == 0.0 {
                    // `:76876-76881`.
                    s.row = 0;
                    s.counter = 0.0;
                } else {
                    // `:76882-76892`.
                    s.counter += i.vx.abs();
                    s.counter += 1.0;
                    if s.counter > 6.0 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                    if s.row >= count {
                        s.row = 0;
                    }
                }
            } else if i.vy < 0.0 {
                // `:76894-76898`.
                s.counter = 0.0;
                s.row = 4;
            } else {
                // `:76899-76903` (vy > 0).
                s.counter = 0.0;
                s.row = 6;
            }
        }
        // `case 48` in the floater group (`NPC.cs:74823-74885`, shared
        // with 49/51/60/82/93/137/182/210/211/253/316/634/662/664): the
        // Harpy faces by velocity sign (0 leaves the flip alone), tilts
        // `vx * 0.1`, and steps every 5 ticks (`num291` = 5 — the
        // type-48 override at `:74871-74874`) over the FULL 6 rows
        // (`num292` — the `--` list 49/51/60/634 does not carry 48).
        // The group's dust spawns (60/634) and the 210/211
        // double-increment are other kinds'.
        FindFrameLow1Kind::Harpy => {
            // `:74850-74857`.
            if i.vx > 0.0 {
                s.sprite_dir = 1;
            }
            if i.vx < 0.0 {
                s.sprite_dir = -1;
            }
            // `:74858`.
            s.rotation = i.vx * 0.1;
            // `:74864-74883`.
            s.counter += 1.0;
            if s.counter >= 5.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= count {
                s.row = 0;
            }
        }
        // `case 57` in the fish group (`NPC.cs:74181-74226`, shared with
        // 55/58/102/241/465/592/607): `spriteDirection = direction`
        // UNCONDITIONALLY (no ground gate), then the wet/dry split —
        // wet walks the <6/<12/<18/<24 bands over rows 0/1/2/3 and the
        // tail zeroes the counter (the row HOLDS 3); dry walks <6/<12
        // over rows 4/5 and the tail zeroes the counter (the row HOLDS
        // 5). The hold-then-restart shape is verbatim.
        FindFrameLow1Kind::CorruptGoldfish => {
            // `:74189`.
            s.sprite_dir = i.direction;
            s.counter += 1.0;
            if i.wet {
                // `:74193-74212`.
                if s.counter < 6.0 {
                    s.row = 0;
                } else if s.counter < 12.0 {
                    s.row = 1;
                } else if s.counter < 18.0 {
                    s.row = 2;
                } else if s.counter < 24.0 {
                    s.row = 3;
                } else {
                    s.counter = 0.0;
                }
            } else if s.counter < 6.0 {
                // `:74214-74217`.
                s.row = 4;
            } else if s.counter < 12.0 {
                // `:74218-74221`.
                s.row = 5;
            } else {
                // `:74222-74225`.
                s.counter = 0.0;
            }
        }
        // `case 61` (`NPC.cs:73537-73557`): the Vulture faces by
        // `direction` unconditionally, tilts `vx * 0.1`, and the FULL
        // still snap (`vx == 0 && vy == 0`) pins row 0 + zero counter;
        // otherwise the 25-tick cycle (`num147 * num148` = 5 * 5) walks
        // rows 1..5 (`(counter / 5) + 1` — the +1 lifts the whole strip
        // off row 0, which belongs to the still pose).
        FindFrameLow1Kind::Vulture => {
            // `:73539-73540`.
            s.sprite_dir = i.direction;
            s.rotation = i.vx * 0.1;
            if i.vx == 0.0 && i.vy == 0.0 {
                // `:73541-73546`.
                s.row = 0;
                s.counter = 0.0;
                return;
            }
            // `:73547-73555` — num147 = 5, num148 = 5.
            s.counter += 1.0;
            if s.counter >= 25.0 {
                s.counter = 0.0;
            }
            s.row = (s.counter / 5.0) as u32 + 1;
        }
        // `case 62 : case 66` (`NPC.cs:73887-73902`): the Demon faces by
        // `direction` unconditionally and tilts `vx * 0.1`; the same
        // 25-tick cycle as the Vulture but WITHOUT the still snap and
        // WITHOUT the +1 — rows 0..4 (`counter / 5`). 66 (the Voodoo
        // Demon) is already in `npc.rs`; this arm owns 62.
        FindFrameLow1Kind::Demon => {
            // `:73890-73891`.
            s.sprite_dir = i.direction;
            s.rotation = i.vx * 0.1;
            // `:73892-73900` — num6 = 5, num7 = 5.
            s.counter += 1.0;
            if s.counter >= 25.0 {
                s.counter = 0.0;
            }
            s.row = (s.counter / 5.0) as u32;
        }
        // `case 63 : case 64 : …` (`NPC.cs:73916-73960`, the jellyfish
        // group): the `ai[1] == 1` attack gate (63/64/103/242 — NOT
        // 256) runs the >4 counter over rows 4..6 (BOTH clamps run
        // every tick: >6 wraps to 4, <4 lifts to 4) and RETURNS; the
        // normal strip walks the <6/<12/<18 bands over rows 0/1/2, the
        // tail pins row 3 and the >= 23 reset zeroes the counter.
        FindFrameLow1Kind::BlueJellyfish | FindFrameLow1Kind::PinkJellyfish => {
            if i.ai[1] == 1.0 {
                // `:73921-73937`.
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
                return;
            }
            if s.counter < 12.0 {
                s.row = 1;
                return;
            }
            if s.counter < 18.0 {
                s.row = 2;
                return;
            }
            s.row = 3;
            if s.counter >= 23.0 {
                s.counter = 0.0;
            }
        }
        // `case 67 : case 217 : case 218 : case 219`
        // (`NPC.cs:74571-74589`): the Crab's plain 6-tick strip over
        // its 8 rows; the flip rides the ground ONLY (`vy == 0` →
        // `spriteDirection = direction` unconditionally, 0 included) —
        // the air keeps animating but never flips.
        FindFrameLow1Kind::Crab => {
            // `:74575-74578`.
            if i.vy == 0.0 {
                s.sprite_dir = i.direction;
            }
            // `:74579-74588`.
            s.counter += 1.0;
            if s.counter >= 6.0 {
                s.counter = 0.0;
                s.row += 1;
                if s.row >= count {
                    s.row = 0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn rows_over(kind: FindFrameLow1Kind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_low1(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The `Main.npcFrameCount` rows, counted entry-by-entry out
    /// of the `Main.cs:67082` initializer (the line/slot arithmetic in
    /// `frame_count`'s doc).
    #[test]
    fn the_slice_1_frame_counts_match_the_main_initializer() {
        let table = [
            (FindFrameLow1Kind::OldMan, 37, 21),
            (FindFrameLow1Kind::ManEater, 43, 3),
            (FindFrameLow1Kind::Bunny, 46, 7),
            (FindFrameLow1Kind::CorruptBunny, 47, 7),
            (FindFrameLow1Kind::Harpy, 48, 6),
            (FindFrameLow1Kind::Snatcher, 56, 3),
            (FindFrameLow1Kind::CorruptGoldfish, 57, 6),
            (FindFrameLow1Kind::Vulture, 61, 6),
            (FindFrameLow1Kind::Demon, 62, 5),
            (FindFrameLow1Kind::BlueJellyfish, 63, 7),
            (FindFrameLow1Kind::PinkJellyfish, 64, 7),
            (FindFrameLow1Kind::Crab, 67, 8),
        ];
        for (kind, vanilla, count) in table {
            assert_eq!(kind.vanilla_type(), vanilla, "{kind:?} id");
            assert_eq!(kind.frame_count(), count, "{kind:?} npcFrameCount");
        }
    }

    /// The Old Man's walk (`NPC.cs:69550-69584`): `|vx| * 2 + 1` = 3 a
    /// tick at vx 1, the row steps at > 6 (every 3rd tick), the band
    /// floor is row 2 and the wrap at 21 - 5 = 16 returns to 2.
    #[test]
    fn the_old_man_walk_wraps_at_sixteen_not_the_full_sheet() {
        let mut s = FrameState::default();
        let i = input(1.0, 0.0);
        let mut rows = Vec::new();
        for _ in 0..6 {
            find_frame_low1(FindFrameLow1Kind::OldMan, &mut s, &i);
            rows.push(s.row);
        }
        assert_eq!(rows, [2, 2, 3, 3, 3, 4]);
        assert_eq!(s.counter, 0.0);
        assert_eq!(s.sprite_dir, 1);
        // Row 15 + one step = 16 → the num57 wrap to 2 (`:69581-69584`).
        s.row = 15;
        s.counter = 6.0;
        find_frame_low1(FindFrameLow1Kind::OldMan, &mut s, &i);
        assert_eq!(s.row, 2);
    }

    /// The Old Man's idle row 0 + air row 1 (`NPC.cs:69515-69519`,
    /// `:69642-69650` — 37 is not in the pin list), and the direction-0
    /// no-flip gate (`:68649-68656`).
    #[test]
    fn the_old_man_idle_air_and_direction_zero() {
        let mut s = FrameState { row: 9, counter: 4.0, sprite_dir: -1, rotation: 0.0 };
        // direction 0 writes nothing (`:68649-68656`).
        let i = FrameInput { direction: 0, ..input(0.0, 0.0) };
        find_frame_low1(FindFrameLow1Kind::OldMan, &mut s, &i);
        assert_eq!((s.row, s.counter), (0, 0.0));
        assert_eq!(s.sprite_dir, -1);
        find_frame_low1(FindFrameLow1Kind::OldMan, &mut s, &i);
        assert_eq!(s.sprite_dir, -1);
        // Both air signs hold row 1.
        find_frame_low1(FindFrameLow1Kind::OldMan, &mut s, &input(0.0, -2.0));
        assert_eq!((s.row, s.counter), (1, 0.0));
        s.row = 9;
        find_frame_low1(FindFrameLow1Kind::OldMan, &mut s, &input(0.0, 2.0));
        assert_eq!(s.row, 1);
    }

    /// The man-eater chomp (`NPC.cs:74973-74993`): the 0/1/2/1 bands at
    /// the 6/12/18/24 counter marks, the EXACT == 23 reset — the 24th
    /// tick restarts at row 0. The Snatcher rides the identical arm.
    #[test]
    fn the_man_eater_chomp_cycles_0_1_2_1_over_23_ticks() {
        for kind in [FindFrameLow1Kind::ManEater, FindFrameLow1Kind::Snatcher] {
            let rows = rows_over(kind, input(0.0, 0.0), 24);
            assert_eq!(
                rows,
                [0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 0]
            );
        }
    }

    /// The == 23 reset quirk (`NPC.cs:74990`): a fractional counter
    /// overshoots the exact reset, no band writes past 24 and the row
    /// HOLDS 1 forever — kept verbatim (the W4-d47 trapper precedent).
    #[test]
    fn the_man_eater_fractional_counter_holds_row_one_forever() {
        let mut s = FrameState { row: 1, counter: 22.5, sprite_dir: 1, rotation: 0.0 };
        let i = input(0.0, 0.0);
        find_frame_low1(FindFrameLow1Kind::ManEater, &mut s, &i);
        assert_eq!((s.row, s.counter), (1, 23.5));
        find_frame_low1(FindFrameLow1Kind::ManEater, &mut s, &i);
        assert_eq!((s.row, s.counter), (1, 24.5));
        find_frame_low1(FindFrameLow1Kind::ManEater, &mut s, &i);
        assert_eq!(s.row, 1);
    }

    /// The bunny hop (`NPC.cs:76876-76892`): idle row 0, the |vx| + 1
    /// walk (2 a tick at vx 1, steps every 4th tick), the wrap at the
    /// FULL 7 rows, and the rise/fall pins 4/6 (`:76894-76903`).
    #[test]
    fn the_bunny_hop_walks_wraps_at_seven_and_pins_the_air_rows() {
        for kind in [FindFrameLow1Kind::Bunny, FindFrameLow1Kind::CorruptBunny] {
            let mut s = FrameState::default();
            let i = input(1.0, 0.0);
            let mut rows = Vec::new();
            for _ in 0..8 {
                find_frame_low1(kind, &mut s, &i);
                rows.push(s.row);
            }
            assert_eq!(rows, [0, 0, 0, 1, 1, 1, 1, 2]);
            // Row 6 + one step = 7 → the full-count wrap to 0.
            s.row = 6;
            s.counter = 6.0;
            find_frame_low1(kind, &mut s, &i);
            assert_eq!(s.row, 0);
            // The air pins.
            find_frame_low1(kind, &mut s, &input(1.0, -1.0));
            assert_eq!((s.row, s.counter), (4, 0.0));
            find_frame_low1(kind, &mut s, &input(1.0, 1.0));
            assert_eq!((s.row, s.counter), (6, 0.0));
            // The idle snap.
            find_frame_low1(kind, &mut s, &input(0.0, 0.0));
            assert_eq!((s.row, s.counter), (0, 0.0));
        }
    }

    /// The Harpy's floater strip (`NPC.cs:74850-74883`): the velocity
    /// flip (0 leaves it alone), the vx * 0.1 tilt, the 5-tick step over
    /// the FULL 6 rows (48 is not in the num292-- list).
    #[test]
    fn the_harpy_flaps_every_five_ticks_over_six_rows() {
        let mut s = FrameState::default();
        let i = input(2.0, 0.0);
        let mut rows = Vec::new();
        for _ in 0..10 {
            find_frame_low1(FindFrameLow1Kind::Harpy, &mut s, &i);
            rows.push(s.row);
        }
        assert_eq!(rows, [0, 0, 0, 0, 1, 1, 1, 1, 1, 2]);
        assert_eq!(s.sprite_dir, 1);
        assert_eq!(s.rotation, 0.2);
        // Row 5 + one step = 6 → the wrap to 0.
        s.row = 5;
        s.counter = 4.0;
        find_frame_low1(FindFrameLow1Kind::Harpy, &mut s, &i);
        assert_eq!(s.row, 0);
        // vx 0: no flip write, the tilt zeroes.
        s.sprite_dir = -1;
        find_frame_low1(FindFrameLow1Kind::Harpy, &mut s, &input(0.0, 0.0));
        assert_eq!(s.sprite_dir, -1);
        assert_eq!(s.rotation, 0.0);
    }

    /// The Corrupt Goldfish's fish strip (`NPC.cs:74189-74225`): the
    /// unconditional flip (direction 0 included), the wet 0/1/2/3 bands
    /// with the row-3 hold at the >= 24 tail, the dry 4/5 bands with the
    /// row-5 hold at the >= 12 tail.
    #[test]
    fn the_corrupt_goldfish_wet_and_dry_bands_hold_then_restart() {
        let mut s = FrameState::default();
        let wet = FrameInput { wet: true, direction: -1, ..input(0.0, 0.0) };
        let mut rows = Vec::new();
        for _ in 0..25 {
            find_frame_low1(FindFrameLow1Kind::CorruptGoldfish, &mut s, &wet);
            rows.push(s.row);
        }
        // ticks 1-5 row 0, 6-11 row 1, 12-17 row 2, 18-23 row 3,
        // tick 24 (counter 24) zeroes the counter and HOLDS row 3,
        // tick 25 restarts at row 0.
        assert_eq!(&rows[..6], &[0, 0, 0, 0, 0, 1]);
        assert_eq!(&rows[10..12], &[1, 2]);
        assert_eq!(&rows[16..18], &[2, 3]);
        assert_eq!(&rows[22..25], &[3, 3, 0]);
        assert_eq!(s.sprite_dir, -1);
        // Dry: rows 4/5, the tail holds 5.
        let mut s = FrameState::default();
        let dry = FrameInput { direction: 0, ..input(0.0, 0.0) };
        let mut rows = Vec::new();
        for _ in 0..13 {
            find_frame_low1(FindFrameLow1Kind::CorruptGoldfish, &mut s, &dry);
            rows.push(s.row);
        }
        assert_eq!(rows, [4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 4]);
        // direction 0 IS written (the unconditional flip).
        assert_eq!(s.sprite_dir, 0);
    }

    /// The Vulture (`NPC.cs:73539-73555`): the still snap pins row 0;
    /// the moving 25-tick cycle walks rows 1..5 (the +1 offset), the
    /// tilt rides vx * 0.1 and the flip copies direction verbatim.
    #[test]
    fn the_vulture_still_snap_and_the_plus_one_flap_cycle() {
        let mut s = FrameState { row: 3, counter: 9.0, sprite_dir: 1, rotation: 0.5 };
        find_frame_low1(FindFrameLow1Kind::Vulture, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        // Moving (vy only still counts — the snap needs BOTH zero).
        let i = input(0.0, 1.0);
        let mut rows = Vec::new();
        for _ in 0..25 {
            find_frame_low1(FindFrameLow1Kind::Vulture, &mut s, &i);
            rows.push(s.row);
        }
        assert_eq!(&rows[..5], &[1, 1, 1, 1, 2]);
        assert_eq!(&rows[5..10], &[2, 2, 2, 2, 3]);
        assert_eq!(&rows[20..25], &[5, 5, 5, 5, 1]);
        let tilt = input(-2.0, 0.0);
        find_frame_low1(FindFrameLow1Kind::Vulture, &mut s, &tilt);
        assert_eq!(s.rotation, -0.2);
    }

    /// The Demon (`NPC.cs:73890-73900`): the same 25-tick cycle WITHOUT
    /// the still snap and WITHOUT the +1 — rows 0..4; the still tick
    /// keeps cycling (row 1 on the first tick).
    #[test]
    fn the_demon_cycle_starts_at_row_zero_and_never_still_snaps() {
        let rows = rows_over(FindFrameLow1Kind::Demon, input(0.0, 0.0), 25);
        assert_eq!(&rows[..5], &[0, 0, 0, 0, 1]);
        assert_eq!(&rows[5..10], &[1, 1, 1, 1, 2]);
        assert_eq!(&rows[20..25], &[4, 4, 4, 4, 0]);
    }

    /// The jellyfish normal strip (`NPC.cs:73939-73959`): the
    /// 0/1/2/3 bands at 6/12/18, the >= 23 reset — row 3 holds ticks
    /// 18-23 then the cycle restarts.
    #[test]
    fn the_jellyfish_normal_strip_bands_and_the_23_reset() {
        for kind in [FindFrameLow1Kind::BlueJellyfish, FindFrameLow1Kind::PinkJellyfish] {
            let rows = rows_over(kind, input(0.0, 0.0), 24);
            assert_eq!(&rows[..6], &[0, 0, 0, 0, 0, 1]);
            assert_eq!(&rows[10..12], &[1, 2]);
            assert_eq!(&rows[17..24], &[3, 3, 3, 3, 3, 3, 0]);
        }
    }

    /// The jellyfish attack gate (`NPC.cs:73921-73937`): ai[1] == 1
    /// lifts any row into the 4..6 band (the <4 clamp), the >4 counter
    /// steps every 5 ticks, the >6 wrap returns to 4, and the arm
    /// RETURNS before the normal strip.
    #[test]
    fn the_jellyfish_attack_gate_clamps_into_the_four_to_six_band() {
        let mut s = FrameState { row: 0, counter: 0.0, sprite_dir: 1, rotation: 0.0 };
        let attack = FrameInput { ai: [0.0, 1.0, 0.0, 0.0], ..input(0.0, 0.0) };
        find_frame_low1(FindFrameLow1Kind::BlueJellyfish, &mut s, &attack);
        assert_eq!(s.row, 4);
        let mut rows = Vec::new();
        for _ in 0..15 {
            find_frame_low1(FindFrameLow1Kind::BlueJellyfish, &mut s, &attack);
            rows.push(s.row);
        }
        assert_eq!(rows, [4, 4, 4, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 4, 4]);
        // The gate's return keeps the normal strip untouched: the
        // counter never enters the <6/<12/<18 bands while attacking.
        assert!(s.counter <= 4.0);
    }

    /// The Crab (`NPC.cs:74575-74588`): the plain 6-tick strip over 8
    /// rows; the flip rides the ground only — the air keeps stepping
    /// but leaves spriteDirection alone (direction 0 on the ground IS
    /// written).
    #[test]
    fn the_crab_six_tick_strip_and_the_grounded_only_flip() {
        let mut s = FrameState::default();
        let ground = FrameInput { direction: -1, ..input(1.0, 0.0) };
        let mut rows = Vec::new();
        for _ in 0..12 {
            find_frame_low1(FindFrameLow1Kind::Crab, &mut s, &ground);
            rows.push(s.row);
        }
        assert_eq!(rows, [0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2]);
        assert_eq!(s.sprite_dir, -1);
        // Row 7 + one step = 8 → the wrap to 0.
        s.row = 7;
        s.counter = 5.0;
        find_frame_low1(FindFrameLow1Kind::Crab, &mut s, &ground);
        assert_eq!(s.row, 0);
        // The air: the strip steps, the flip does NOT fire.
        let air = FrameInput { direction: 1, ..input(1.0, 1.0) };
        s.counter = 5.0;
        find_frame_low1(FindFrameLow1Kind::Crab, &mut s, &air);
        assert_eq!(s.row, 1);
        assert_eq!(s.sprite_dir, -1);
    }
}