//! W4-f27 — FindFrame extra wave 12 (slice 12): the 551 Betsy MICRO.
//! The slice-11 claim deferred 551/564-577 as "the complex/RNG/emote/
//! dust seams"; slice 12 re-checked **551 alone** mechanically: the
//! `case 551` range (`NPC.cs:67258-67451`) carries ZERO `Main.rand`
//! (grep over the range: 0 matches) — the deferral reason was the
//! ladder's complexity, not RNG. This wave ports it verbatim as a pure
//! machine; the npc.rs v1 arm (the M4-EVENT-19 `(vx != 0) + 0.6 >= 8`
//! free strip — WRONG: Betsy's row NEVER reads velocity) delegates
//! here in the wiring commit (the 549-fix bridge shape, the slice-9/
//! 10/11 precedent).
//!
//! The machine (`NPC.cs:67258-67451`, `NPCID.DD2Betsy` = 551,
//! `ID/NPCID.cs:11779`): the ai[0] phase dispatch over the 11-row
//! dragon sheet —
//! * **ai[0] == 4** (`:67261-67313`): the long breath ladder — base
//!   num256 = 60, num257 = 6*10 = 60, tail base 60+60+3 = 123: rows
//!   5 (charge) / 6 (>= 5) / 7 (>= 54) / 8 (>= 57) / 9 + (int)c/3 % 2
//!   (>= 60, the breath flap) / 8 (>= 123) / 7 (>= 128) / 0 (>= 133).
//! * **ai[0] == 3** (`:67314-67367`): the short breath ladder — base
//!   num264 = 40, num265 = 40+80 = 120, num266 = 25, tail base
//!   120-25+3 = 98: the same shape at >= 5/34/37/40/98/103/108.
//! * **ai[0] == 5** (`:67368-67371`): the row-3 pin, counter UNTOUCHED.
//! * **ai[0] == 6** (`:67372-67444`): the 8-tick landing ladder —
//!   rows 2/3/4/3/4/3/2/1/0 at >= 8/16/24/32/40/48/56/64/72.
//! * **else** (`:67445-67448`): row 0, counter UNTOUCHED.
//!
//! Quirk (all three ladder branches): the entry `if row < 5` (4/3) /
//! `if row > 4` (6) `frameCounter = 0` write (`:67265-67268` /
//! `:67319-67322` / `:67374-67377`) is DEAD — the very next lines
//! overwrite the counter with `(int)ai[1]` (`:67270` / `:67324` /
//! `:67379`). Ported verbatim; the row read survives only in that
//! dead gate. The ILSpy num258/num267/num273 `+1` noise collapses to
//! the static thresholds cited above (each ladder counter starts 0
//! and the `num258 = 0` resets at `:67279`/`:67292` restart it).
//!
//! No arm reads velocity/direction, no sprite/rotation write — only
//! `frame.Y` and `frameCounter`. The counters are `(int)ai[1]`
//! truncations (toward zero), exact integers in f32; the one division
//! is INTEGER (`(int)frameCounter / 3 % 2`, `:67290`/`:67344`).
//!
//! Row convention: the port tracks frame ROW indices, not pixels —
//! the C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out. `Main.npcFrameCount[551]` = 11
//! (`Main.cs:67082`'s 697-entry initializer, `:67139` slot 1),
//! cross-checked entry-by-entry against the port's
//! `render/entities.rs::NPC_FRAME_COUNT` (all 697 slots, zero
//! mismatches).

// The seam fn + vanilla_type ride tests only (the npc.rs arm keys on
// NpcKind directly, the e11 wiring shape) — the e11 precedent.
#![allow(dead_code)]

/// The extra-wave-12 kinds, keyed by their VANILLA `NPC.type`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra12Kind {
    /// `NPCID.DD2Betsy` = 551 (`ID/NPCID.cs:11779`) — the dragon
    /// boss's phase ladders (`NPC.cs:67258`).
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
            // sheet (rows 0-4 flight, 5-10 the breath phases).
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

/// The `NPC` fields `FindFrame` reads/writes for this arm: `frame.Y /
/// num` (the row) and `frameCounter` ONLY — case 551 never touches
/// `spriteDirection`/`rotation`. The four-field shape is the e11
/// bridge convention (the wiring copies all four to/from `Npc`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter` (the C# field is a `double`, `NPC.cs:6485`;
    /// the port's f32 is the existing `npc.rs` convention — the
    /// `(int)ai[1]` integer counters here are exact in both).
    pub counter: f32,
    /// `NPC.spriteDirection` — case 551 never writes it.
    pub sprite_dir: i32,
    /// `NPC.rotation` — case 551 never writes it.
    pub rotation: f32,
}

/// The per-tick inputs the arm reads: ONLY `ai[0]` (the phase) and
/// `ai[1]` (the phase timer). The six-field shape is the e11 bridge
/// convention; case 551 reads no velocity, no direction, no ai[2].
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    pub direction: i32,
    /// `NPC.ai[0]` — the phase dispatch (4 long breath / 3 short
    /// breath / 5 the row-3 pin / 6 the landing ladder / else row 0).
    pub ai0: f32,
    /// `NPC.ai[1]` — the phase timer, truncated `(int)` into the
    /// counter every ladder tick.
    pub ai1: f32,
    pub ai2: f32,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1, ai0: 0.0, ai1: 0.0, ai2: 0.0 }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-12 kinds — one
/// tick.
pub fn find_frame_extra12(kind: FindFrameExtra12Kind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // `case 551` (`NPC.cs:67258-67451`): the ai[0] phase dispatch.
        FindFrameExtra12Kind::DD2Betsy => {
            if i.ai0 == 4.0 {
                // `:67261-67313` — the long breath ladder. num256 =
                // 60, num257 = 6*10 = 60, tail base 60+60+3 = 123.
                // `:67265-67268` — the DEAD counter reset (the
                // `:67270` ai[1] copy overwrites it).
                if s.row < 5 {
                    s.counter = 0.0;
                }
                // `:67269-67270`.
                s.row = 5;
                s.counter = i.ai1 as i32 as f32;
                // `:67271-67278` — >= 5*1.
                if s.counter >= 5.0 {
                    s.row = 6;
                }
                // `:67280-67283` — >= num256-6 = 54.
                if s.counter >= 54.0 {
                    s.row = 7;
                }
                // `:67284-67287` — >= num256-3 = 57.
                if s.counter >= 57.0 {
                    s.row = 8;
                }
                // `:67288-67291` — >= num256 = 60, the breath flap
                // (the INTEGER /3 %2).
                if s.counter >= 60.0 {
                    s.row = 9 + ((s.counter as i32) / 3 % 2) as u32;
                }
                // `:67293-67296` — >= 60+60+3 = 123.
                if s.counter >= 123.0 {
                    s.row = 8;
                }
                // `:67297-67304` — >= 123+5*1 = 128.
                if s.counter >= 128.0 {
                    s.row = 7;
                }
                // `:67305-67312` — >= 123+5*2 = 133.
                if s.counter >= 133.0 {
                    s.row = 0;
                }
            } else if i.ai0 == 3.0 {
                // `:67314-67367` — the short breath ladder. num264 =
                // 40, num265 = 40+80 = 120, num266 = 25, tail base
                // 120-25+3 = 98. `:67319-67322` — the DEAD reset.
                if s.row < 5 {
                    s.counter = 0.0;
                }
                // `:67323-67324`.
                s.row = 5;
                s.counter = i.ai1 as i32 as f32;
                // `:67325-67332` — >= 5*1.
                if s.counter >= 5.0 {
                    s.row = 6;
                }
                // `:67334-67337` — >= num264-6 = 34.
                if s.counter >= 34.0 {
                    s.row = 7;
                }
                // `:67338-67341` — >= num264-3 = 37.
                if s.counter >= 37.0 {
                    s.row = 8;
                }
                // `:67342-67345` — >= num264 = 40, the breath flap.
                if s.counter >= 40.0 {
                    s.row = 9 + ((s.counter as i32) / 3 % 2) as u32;
                }
                // `:67347-67350` — >= num265-num266+3 = 98.
                if s.counter >= 98.0 {
                    s.row = 8;
                }
                // `:67351-67358` — >= 98+5*1 = 103.
                if s.counter >= 103.0 {
                    s.row = 7;
                }
                // `:67359-67366` — >= 98+5*2 = 108.
                if s.counter >= 108.0 {
                    s.row = 0;
                }
            } else if i.ai0 == 5.0 {
                // `:67368-67371` — the row-3 pin, counter UNTOUCHED.
                s.row = 3;
            } else if i.ai0 == 6.0 {
                // `:67372-67444` — the 8-tick landing ladder.
                // `:67374-67377` — the DEAD counter reset.
                if s.row > 4 {
                    s.counter = 0.0;
                }
                // `:67378-67379`.
                s.row = 1;
                s.counter = i.ai1 as i32 as f32;
                // `:67380-67443` — the nine >= 8*k rungs.
                if s.counter >= 8.0 {
                    s.row = 2;
                }
                if s.counter >= 16.0 {
                    s.row = 3;
                }
                if s.counter >= 24.0 {
                    s.row = 4;
                }
                if s.counter >= 32.0 {
                    s.row = 3;
                }
                if s.counter >= 40.0 {
                    s.row = 4;
                }
                if s.counter >= 48.0 {
                    s.row = 3;
                }
                if s.counter >= 56.0 {
                    s.row = 2;
                }
                if s.counter >= 64.0 {
                    s.row = 1;
                }
                if s.counter >= 72.0 {
                    s.row = 0;
                }
            } else {
                // `:67445-67448` — row 0, counter UNTOUCHED.
                s.row = 0;
            }
            // `:67449` — frame.Y = num * row (the row IS the state).
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 551 Betsy: the ai[0] phase ladders — every rung boundary of
    /// the 4/3 breath machines + the 6 landing ladder, the 5 pin and
    /// the else snap, the DEAD entry-reset quirk, the (int)ai[1]
    /// truncation. Expectations re-derived by mechanical replay of
    /// the line-for-line C# transcription (integer counters exact in
    /// f32, the one division INTEGER).
    #[test]
    fn f27s12_betsy_rides_the_phase_ladders() {
        let k = FindFrameExtra12Kind::DD2Betsy;
        // The long breath (ai0 4): >= 5/54/57/60/123/128/133, the
        // flap 9 + (int)c/3 % 2 (60→9, 63→10, 66→9, 122→9).
        for (ai1, row) in [
            (0.0, 5u32), (4.0, 5), (5.0, 6), (53.0, 6), (54.0, 7), (56.0, 7),
            (57.0, 8), (59.0, 8), (60.0, 9), (63.0, 10), (66.0, 9), (122.0, 9),
            (123.0, 8), (127.0, 8), (128.0, 7), (132.0, 7), (133.0, 0), (200.0, 0),
        ] {
            let mut s = FrameState { row: 7, counter: 99.0, sprite_dir: -1, rotation: 0.5 };
            let i = FrameInput { ai0: 4.0, ai1, ..Default::default() };
            find_frame_extra12(k, &mut s, &i);
            // The DEAD entry reset: counter 99 → (int)ai1, never 0
            // first; sprite/rotation UNTOUCHED.
            assert_eq!((s.row, s.counter), (row, ai1 as i32 as f32), "ai1 {ai1}");
            assert_eq!((s.sprite_dir, s.rotation), (-1, 0.5));
        }
        // The short breath (ai0 3): >= 5/34/37/40/98/103/108, the
        // flap at 40: 40/3 = 13 % 2 = 1 → row 10; 42/3 = 14 → 9.
        for (ai1, row) in [
            (0.0, 5u32), (5.0, 6), (33.0, 6), (34.0, 7), (36.0, 7), (37.0, 8),
            (39.0, 8), (40.0, 10), (42.0, 9), (97.0, 9), (98.0, 8), (102.0, 8),
            (103.0, 7), (107.0, 7), (108.0, 0),
        ] {
            let mut s = FrameState::default();
            let i = FrameInput { ai0: 3.0, ai1, ..Default::default() };
            find_frame_extra12(k, &mut s, &i);
            assert_eq!((s.row, s.counter), (row, ai1 as i32 as f32), "ai1 {ai1}");
        }
        // The landing ladder (ai0 6): rows 2/3/4/3/4/3/2/1/0 at the
        // nine >= 8*k rungs; the entry row > 4 reset also DEAD.
        for (ai1, row) in [
            (0.0, 1u32), (7.0, 1), (8.0, 2), (15.0, 2), (16.0, 3), (23.0, 3),
            (24.0, 4), (31.0, 4), (32.0, 3), (39.0, 3), (40.0, 4), (47.0, 4),
            (48.0, 3), (55.0, 3), (56.0, 2), (63.0, 2), (64.0, 1), (71.0, 1),
            (72.0, 0), (100.0, 0),
        ] {
            let mut s = FrameState { row: 9, counter: 50.0, ..Default::default() };
            let i = FrameInput { ai0: 6.0, ai1, ..Default::default() };
            find_frame_extra12(k, &mut s, &i);
            assert_eq!((s.row, s.counter), (row, ai1 as i32 as f32), "ai1 {ai1}");
        }
        // ai0 5: the row-3 pin, counter UNTOUCHED.
        let mut s = FrameState { row: 9, counter: 42.0, ..Default::default() };
        find_frame_extra12(k, &mut s, &FrameInput { ai0: 5.0, ai1: 7.0, ..Default::default() });
        assert_eq!((s.row, s.counter), (3, 42.0));
        // else: row 0, counter UNTOUCHED; velocity NEVER feeds the
        // machine (the v1 free strip was WRONG).
        for ai0 in [0.0f32, 1.0, 2.0, 7.0] {
            let mut s = FrameState { row: 9, counter: 42.0, ..Default::default() };
            let i = FrameInput { vx: 9.0, vy: -9.0, ai0, ai1: 7.0, ..Default::default() };
            find_frame_extra12(k, &mut s, &i);
            assert_eq!((s.row, s.counter), (0, 42.0), "ai0 {ai0}");
        }
        // The (int)ai[1] truncation toward zero: 4.9 → 4 (row 5),
        // -5.5 → -5 (row 5, no rung fires).
        let mut s = FrameState::default();
        find_frame_extra12(k, &mut s, &FrameInput { ai0: 4.0, ai1: 4.9, ..Default::default() });
        assert_eq!((s.row, s.counter), (5, 4.0));
        let mut s = FrameState::default();
        find_frame_extra12(k, &mut s, &FrameInput { ai0: 4.0, ai1: -5.5, ..Default::default() });
        assert_eq!((s.row, s.counter), (5, -5.0));
    }

    /// The frame count (Main.cs:67139 slot 1 = 11) matches the port's
    /// verbatim table; the seam claims exactly 551; a phase/timer
    /// sweep stays inside the 11-row sheet.
    #[test]
    fn f27s12_frame_count_seam_and_strip_sweep() {
        let k = FindFrameExtra12Kind::DD2Betsy;
        assert_eq!(k.frame_count(), 11);
        assert_eq!(crate::render::entities::NPC_FRAME_COUNT[551], 11);
        assert_eq!(k.frame_count(), crate::render::entities::NPC_FRAME_COUNT[k.vanilla_type() as usize]);
        for t in 0..697i16 {
            let seam = find_frame_kind_extra12(t);
            assert_eq!(seam.is_some(), t == 551, "type {t}");
            if let Some(sk) = seam {
                assert_eq!(sk.vanilla_type(), t);
            }
        }
        for ai0 in [0.0f32, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0] {
            for ai1 in [
                -5.5f32, 0.0, 4.9, 5.0, 8.0, 34.0, 40.0, 54.0, 60.0, 63.0, 72.0,
                98.0, 123.0, 133.0, 200.0,
            ] {
                let mut s = FrameState::default();
                for tick in 0..40 {
                    let i = FrameInput { ai0, ai1, ..Default::default() };
                    find_frame_extra12(k, &mut s, &i);
                    assert!(s.row < 11, "tick {tick} row {} (ai0 {ai0}, ai1 {ai1})", s.row);
                }
            }
        }
    }
}
