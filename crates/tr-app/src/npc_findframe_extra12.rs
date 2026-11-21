//! W4-f27 — FindFrame extra wave 12 (slice 12): the ONE kind 551
//! DD2Betsy — the slice-11 DEFERRED tail's first pick-up. The extra-11
//! ANALYSIS claim deferred 551 with the "complex" seam cited; the
//! mechanical re-read shows the arm (`NPC.cs:67258-67451`) is a PURE
//! ai-driven pose machine: zero `Main.rand` (grep over the whole arm
//! range = 0 matches), no velocity reads, no sprite/rotation writes —
//! only `frame.Y` and `frameCounter`, and the counter is OVERWRITTEN
//! from `(int)ai[1]` every tick in the animated states (it never
//! accumulates). The extra-11 precedent shape: a pure machine + the
//! `find_frame_kind_extra12` seam + the npc.rs arm delegation (the
//! 549-fix bridge).
//!
//! The machine (`NPC.cs:67258-67451`, the decompiler's num258/num60
//! step-counter dance re-derived by hand — each `num60 = num258 + 1`
//! yields the ladder multiples 1, 2, 3... below):
//! * **ai[0] == 4** (`:67261-67313`, the fire-breath wind-up/hold):
//!   row < 5 zeroes the counter (`:67265-67268` — dead, the very next
//!   lines overwrite it), row = 5, counter = `(int)ai[1]`
//!   (`:67269-67270`); the ladder >= 5 → row 6, >= 60−6 = 54 → row 7,
//!   >= 60−3 = 57 → row 8, >= 60 → row `9 + (int)counter/3 % 2` (the
//!   2-row breath flicker, `:67290`), >= 60+60+3 = 123 → row 8,
//!   >= 123+5 = 128 → row 7, >= 123+10 = 133 → row 0. Later checks
//!   OVERWRITE earlier ones (no breaks).
//! * **ai[0] == 3** (`:67314-67367`, the charge wind-up): the same
//!   shape with num264 = 40, num265 = 40+80 = 120, num266 = 25:
//!   >= 5 → 6, >= 34 → 7, >= 37 → 8, >= 40 → the 9/10 flicker,
//!   >= 120−25+3 = 98 → 8, >= 103 → 7, >= 108 → 0.
//! * **ai[0] == 5** (`:67368-67371`): row = 3, counter UNTOUCHED.
//! * **ai[0] == 6** (`:67372-67444`, the death/spawn thrash): row > 4
//!   zeroes the counter (also overwritten), row = 1, counter =
//!   `(int)ai[1]`; the 8-tick ladder >= 8 → 2, >= 16 → 3, >= 24 → 4,
//!   >= 32 → 3, >= 40 → 4, >= 48 → 3, >= 56 → 2, >= 64 → 1,
//!   >= 72 → 0 (`:67384-67443` — the num273 counter is NOT reset
//!   between checks, so the multiples grow 1..9).
//! * **else** (`:67445-67448`): row = 0 (the fly cycle's idle pose),
//!   counter UNTOUCHED.
//! The tail (`:67449`) writes `frame.Y = num * row` EVERY tick.
//!
//! Row convention: the port tracks frame ROW indices, not pixels —
//! the C#'s `num` (sheet height / `Main.npcFrameCount[551]`,
//! `NPC.cs:66694`) cancels out of every comparison. The row count is
//! `Main.npcFrameCount[551]` = 11 (`Main.cs:67082`'s 697-entry
//! initializer, entry 551 = `Main.cs:67139` slot 1 — the row
//! `25, 11, 14, ...`), cross-checked against the port's own
//! `render/entities.rs::NPC_FRAME_COUNT` (`:1042`, same slot = 11).

/// The extra-wave-12 kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra12Kind {
    /// `NPCID.DD2Betsy` = 551 (`NPCID.cs:11779`, the port's
    /// `DD2Betsy`) — the ai-driven dragon pose machine
    /// (`NPC.cs:67258`).
    DD2Betsy,
}

impl FindFrameExtra12Kind {
    /// The vanilla `NPC.type` — the `FindFrame` switch key.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::DD2Betsy => 551,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// entry `type` on line `67084 + type/10`, slot `type % 10`) —
    /// cross-checked against `render/entities.rs::NPC_FRAME_COUNT`.
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67139 (550-559): slot 1 — the 11-row dragon
            // sheet (rows 0-10; the breath flicker rides 9/10).
            Self::DD2Betsy => 11,
        }
    }
}

/// The wiring seam: the vanilla type → this module's kind (the
/// `find_frame_kind_extra11` precedent). `None` outside the slice.
pub fn find_frame_kind_extra12(vanilla_type: i16) -> Option<FindFrameExtra12Kind> {
    Some(match vanilla_type {
        551 => FindFrameExtra12Kind::DD2Betsy,
        _ => return None,
    })
}

/// The `NPC` fields the bridge copies (the e11 four-field shape):
/// the 551 arm itself reads/writes ONLY `frame.Y / num` (the row) and
/// `frameCounter` (`NPC.cs:67258-67451` — no sprite, no rotation);
/// `sprite_dir`/`rotation` ride the bridge UNTOUCHED. The wiring step
/// copies them to/from `Npc` (the extra-11 bridge shape).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter` (the C# field is a `double`, `NPC.cs:6485`;
    /// the port's f32 is the existing `npc.rs` convention — the arm
    /// only ever stores `(int)ai[1]`, exact in both).
    pub counter: f32,
    /// `NPC.spriteDirection` — the 551 arm NEVER writes it (the
    /// bridge copies it through unchanged).
    pub sprite_dir: i32,
    /// `NPC.rotation` — the 551 arm NEVER writes it.
    pub rotation: f32,
}

/// The per-tick inputs the bridge copies (the e11 shape). The 551 arm
/// reads ONLY the `ai[0]` state selector and the `ai[1]` animation
/// tick — no velocity, no direction, no ai[2], no world reads
/// (`NPC.cs:67258-67451`).
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    /// `velocity.X` — UNREAD by the 551 arm.
    pub vx: f32,
    /// `velocity.Y` — UNREAD by the 551 arm.
    pub vy: f32,
    /// `Entity.direction` — UNREAD by the 551 arm.
    pub direction: i32,
    /// `NPC.ai[0]` — 4 breath / 3 charge / 5 the row-3 pin / 6 thrash,
    /// anything else the row-0 idle.
    pub ai0: f32,
    /// `NPC.ai[1]` — the state's animation tick; `(int)` truncated
    /// into `frameCounter` every tick in the animated states.
    pub ai1: f32,
    /// `NPC.ai[2]` — UNREAD by the 551 arm.
    pub ai2: f32,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1, ai0: 0.0, ai1: 0.0, ai2: 0.0 }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-12 kinds — one
/// tick. The ladders are the C#'s sequential overwrites (later checks
/// win), NOT else-ifs.
pub fn find_frame_extra12(kind: FindFrameExtra12Kind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // `case 551` (`NPC.cs:67258-67451`).
        FindFrameExtra12Kind::DD2Betsy => {
            let mut row = s.row;
            if i.ai0 == 4.0 {
                // :67261-67313 — the breath: base 60, the tail hold
                // 6*10 = 60 ticks (num257).
                if row < 5 {
                    s.counter = 0.0; // :67267 (dead — overwritten below)
                }
                row = 5;
                s.counter = i.ai1 as i32 as f32; // :67270 `(int)ai[1]`
                let c = s.counter;
                if c >= 5.0 {
                    row = 6; // :67275-67278 (5 * 1)
                }
                if c >= 54.0 {
                    row = 7; // :67280-67283 (60 - 6)
                }
                if c >= 57.0 {
                    row = 8; // :67284-67287 (60 - 3)
                }
                if c >= 60.0 {
                    row = 9 + (c as i32 / 3 % 2) as u32; // :67288-67291
                }
                if c >= 123.0 {
                    row = 8; // :67293-67296 (60 + 60 + 3)
                }
                if c >= 128.0 {
                    row = 7; // :67301-67304 (123 + 5*1)
                }
                if c >= 133.0 {
                    row = 0; // :67309-67312 (123 + 5*2)
                }
            } else if i.ai0 == 3.0 {
                // :67314-67367 — the charge: num264 = 40,
                // num265 = 40 + 80 = 120, num266 = 25.
                if row < 5 {
                    s.counter = 0.0; // :67321 (dead — overwritten below)
                }
                row = 5;
                s.counter = i.ai1 as i32 as f32; // :67324
                let c = s.counter;
                if c >= 5.0 {
                    row = 6; // :67329-67332
                }
                if c >= 34.0 {
                    row = 7; // :67334-67337 (40 - 6)
                }
                if c >= 37.0 {
                    row = 8; // :67338-67341 (40 - 3)
                }
                if c >= 40.0 {
                    row = 9 + (c as i32 / 3 % 2) as u32; // :67342-67345
                }
                if c >= 98.0 {
                    row = 8; // :67347-67350 (120 - 25 + 3)
                }
                if c >= 103.0 {
                    row = 7; // :67355-67358 (98 + 5*1)
                }
                if c >= 108.0 {
                    row = 0; // :67363-67366 (98 + 5*2)
                }
            } else if i.ai0 == 5.0 {
                row = 3; // :67368-67371 — counter UNTOUCHED
            } else if i.ai0 == 6.0 {
                // :67372-67444 — the thrash: the 8-tick ladder, the
                // num273 step counter growing 1..9 (NOT reset).
                if row > 4 {
                    s.counter = 0.0; // :67376 (dead — overwritten below)
                }
                row = 1;
                s.counter = i.ai1 as i32 as f32; // :67379
                let c = s.counter;
                if c >= 8.0 {
                    row = 2; // :67384-67387 (8 * 1)
                }
                if c >= 16.0 {
                    row = 3; // :67391-67394 (8 * 2)
                }
                if c >= 24.0 {
                    row = 4; // :67398-67401 (8 * 3)
                }
                if c >= 32.0 {
                    row = 3; // :67405-67408 (8 * 4)
                }
                if c >= 40.0 {
                    row = 4; // :67412-67415 (8 * 5)
                }
                if c >= 48.0 {
                    row = 3; // :67419-67422 (8 * 6)
                }
                if c >= 56.0 {
                    row = 2; // :67426-67429 (8 * 7)
                }
                if c >= 64.0 {
                    row = 1; // :67433-67436 (8 * 8)
                }
                if c >= 72.0 {
                    row = 0; // :67440-67443 (8 * 9)
                }
            } else {
                row = 0; // :67445-67448 — counter UNTOUCHED
            }
            s.row = row; // :67449 `frame.Y = num * num255` EVERY tick
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The slice-12 witness: the 551 machine's five branches replayed
    /// against line-for-line C# transcriptions (`NPC.cs:67258-67451`)
    /// — expectations re-derived by mechanical replay, written not
    /// run (code-only). Also pins the seam + the frame count.
    #[test]
    fn f27s12_betsy_pose_machine_replays_the_c_sharp_arm() {
        // The seam + the count (Main.cs:67139 slot 1 = 11).
        let k = find_frame_kind_extra12(551).unwrap();
        assert_eq!(k, FindFrameExtra12Kind::DD2Betsy);
        assert_eq!(k.vanilla_type(), 551);
        assert_eq!(k.frame_count(), 11);
        assert!(find_frame_kind_extra12(550).is_none());
        assert!(find_frame_kind_extra12(552).is_none());

        // ai[0] == 4 (the breath): sweep ai[1] 0..=135, the row at
        // each tick — <5 row 5, 5..53 row 6, 54..56 row 7, 57..59
        // row 8, 60..122 the 9/10 flicker ((int)c/3 % 2), 123..127
        // row 8, 128..132 row 7, >=133 row 0. The counter tracks
        // (int)ai[1] every tick.
        for c in 0..=135i32 {
            let mut s = FrameState { row: 0, counter: 99.0, ..FrameState::default() };
            let i = FrameInput { ai0: 4.0, ai1: c as f32 + 0.7, ..FrameInput::default() };
            find_frame_extra12(k, &mut s, &i);
            let want = if c < 5 {
                5
            } else if c < 54 {
                6
            } else if c < 57 {
                7
            } else if c < 60 {
                8
            } else if c < 123 {
                9 + (c / 3 % 2) as u32
            } else if c < 128 {
                8
            } else if c < 133 {
                7
            } else {
                0
            };
            assert_eq!(s.row, want, "breath tick {c}");
            assert_eq!(s.counter, c as f32, "the (int) truncation");
        }
        // The flicker's exact alternation at 60..71 (the C#'s
        // 9 + (int)counter/3 % 2, :67290): 60-62 row 9, 63-65 row 10,
        // 66-68 row 9, 69-71 row 10.
        let mut rows = Vec::new();
        for c in 60..72 {
            let mut s = FrameState::default();
            find_frame_extra12(k, &mut s, &FrameInput { ai0: 4.0, ai1: c as f32, ..FrameInput::default() });
            rows.push(s.row);
        }
        assert_eq!(rows, vec![9, 9, 9, 10, 10, 10, 9, 9, 9, 10, 10, 10]);

        // ai[0] == 3 (the charge): the same shape at 5/34/37/40/98/
        // 103/108 — spot the band edges.
        for (c, want) in [
            (0, 5), (4, 5), (5, 6), (33, 6), (34, 7), (36, 7), (37, 8),
            (39, 8), (40, 9), (43, 10), (97, 10), (98, 8), (102, 8),
            (103, 7), (107, 7), (108, 0), (200, 0),
        ] {
            let mut s = FrameState { row: 7, counter: 0.0, ..FrameState::default() };
            find_frame_extra12(k, &mut s, &FrameInput { ai0: 3.0, ai1: c as f32, ..FrameInput::default() });
            assert_eq!(s.row, want, "charge tick {c}");
            assert_eq!(s.counter, c as f32);
        }

        // ai[0] == 5: the row-3 pin, counter UNTOUCHED (:67368-67371).
        let mut s = FrameState { row: 9, counter: 42.0, ..FrameState::default() };
        find_frame_extra12(k, &mut s, &FrameInput { ai0: 5.0, ai1: 7.0, ..FrameInput::default() });
        assert_eq!((s.row, s.counter), (3, 42.0));

        // ai[0] == 6 (the thrash): the 8-tick ladder band edges —
        // 1/2/3/4/3/4/3/2/1/0.
        for (c, want) in [
            (0, 1), (7, 1), (8, 2), (15, 2), (16, 3), (23, 3), (24, 4),
            (31, 4), (32, 3), (39, 3), (40, 4), (47, 4), (48, 3),
            (55, 3), (56, 2), (63, 2), (64, 1), (71, 1), (72, 0),
            (300, 0),
        ] {
            let mut s = FrameState { row: 6, counter: 0.0, ..FrameState::default() };
            find_frame_extra12(k, &mut s, &FrameInput { ai0: 6.0, ai1: c as f32, ..FrameInput::default() });
            assert_eq!(s.row, want, "thrash tick {c}");
            assert_eq!(s.counter, c as f32);
        }

        // else: the row-0 idle, counter UNTOUCHED (:67445-67448) —
        // ai[0] 0/1/2/7 all fall here.
        for ai0 in [0.0, 1.0, 2.0, 7.0] {
            let mut s = FrameState { row: 10, counter: 13.0, ..FrameState::default() };
            find_frame_extra12(k, &mut s, &FrameInput { ai0, ai1: 55.0, ..FrameInput::default() });
            assert_eq!((s.row, s.counter), (0, 13.0));
        }
    }
}
