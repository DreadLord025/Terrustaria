//! W4-f27 — FindFrame extra wave 13 (slice 13): the 4-kind MICRO —
//! 566/567 the DD2 skeletons + 570/571 the DD2 drakins, the slice-11
//! DEFERRED tail's next pick-ups after slice 12's Betsy (551). The
//! slice-11 claim table deferred 566/567 with the "materialize-cycle
//! seam" cited; the mechanical re-read shows the FindFrame arm itself
//! (`NPC.cs:67559-67583`) is a PURE walk — zero `Main.rand` (grep over
//! both cited ranges, 70 lines: 0 matches), no ai reads at all. The
//! drakin arm (`NPC.cs:67711-67755`) is the bomber-shape ladder with
//! the row-14 long hold. Both npc.rs v1 arms (the M4-EVENT-19
//! `(vx != 0) + 0.6 >= 8` free strips — the skeletons' `% 14` WRONG
//! twice: the strip AND the count, the sheet is 9 rows) delegate here
//! in the wiring commit (the 549-fix bridge shape, the slice-9/10/11/
//! 12 precedent).
//!
//! The machines:
//! * **566/567 the skeletons** (`NPC.cs:67559-67583`, the shared
//!   `case 566`/`case 567` body) — the goblin walk shape MINUS the
//!   attack ladder: the vy == 0 sprite copy (`:67561-67564`, the
//!   UNCONDITIONAL `spriteDirection = direction`, not the ±1 gates),
//!   the air-OR-backpedal row-8 pin (`:67565-67570`), the idle row-0
//!   snap (`:67571-67574` — NOT the goblins' row 9), the |vx|
//!   accumulator with the >= 64 (or negative) reset (`:67577-67581`)
//!   and the `(int)(counter/8)` row (`:67582`) over rows 0-7. The
//!   9-row sheet: rows 0-7 the walk + row 8 the air pin.
//! * **570/571 the drakins** (`NPC.cs:67711-67755`, the shared
//!   `case 570`/`case 571` body) — the ai[1] > 0 && ai[0] > 0 attack
//!   ladder (`:67713-67732`): the UNCONDITIONAL sprite copy
//!   (`:67715`), the row outside 10..15 OR ai[1] == 59 resets to
//!   row 10 + counter 0 (`:67716-67720`), num188 = 5 EXCEPT row 14
//!   where num188 = 35 (`:67721-67725` — the long hold), the
//!   ++counter >= num188 step gated on row < 15 (`:67726-67730` — at
//!   row 15 the counter keeps growing, the bomber row-16 quirk); else
//!   the walk: the vy == 0 sprite copy (`:67733-67736`), the
//!   air-OR-backpedal row-1 pin (`:67737-67742`), the idle row-9 snap
//!   (`:67743-67748`), the |vx| accumulator with the >= 64 (or
//!   negative) reset (`:67749-67753`) and the `(int)(counter/8 + 2)`
//!   row (`:67754`) over rows 2-9 — the +2 INSIDE the int cast (for
//!   the non-negative post-reset counter it equals the bombers'
//!   outside-+2, `NPC.cs:67836`). The 16-row sheet.
//!
//! No arm reads velocity in the ladder branches, no rotation write
//! anywhere — only `frame.Y`, `frameCounter` and `spriteDirection`.
//! The accumulators are integers exact in f32 (the |vx| adds ride the
//! existing npc.rs f32 convention); the divisions are the C#'s
//! `(int)` truncations toward zero.
//!
//! Row convention: the port tracks frame ROW indices, not pixels —
//! the C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out. `Main.npcFrameCount[566/567]` = 9
//! (`Main.cs:67140` slots 6/7), `[570/571]` = 16 (`Main.cs:67141`
//! slots 0/1) — cross-checked entry-by-entry against the port's
//! `render/entities.rs::NPC_FRAME_COUNT` (the 697-slot table the
//! slice-11 wave re-diffed with ZERO mismatches).

// The seam fn + vanilla_type ride tests only (the npc.rs arms key on
// NpcKind directly, the e11/e12 wiring shape) — the e11 precedent.
#![allow(dead_code)]

/// The extra-wave-13 kinds, keyed by their VANILLA `NPC.type`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra13Kind {
    /// `NPCID.DD2SkeletonT1` = 566 (`ID/NPCID.cs:11809`) — the
    /// etherian skeleton's walk (`NPC.cs:67559`).
    DD2SkeletonT1,
    /// `NPCID.DD2SkeletonT3` = 567 (`ID/NPCID.cs:11811`) — the same
    /// walk (`NPC.cs:67560`).
    DD2SkeletonT3,
    /// `NPCID.DD2DrakinT2` = 570 (`ID/NPCID.cs:11817`) — the drakin
    /// ladder + walk (`NPC.cs:67711`).
    DD2DrakinT2,
    /// `NPCID.DD2DrakinT3` = 571 (`ID/NPCID.cs:11819`) — the same
    /// machine (`NPC.cs:67712`).
    DD2DrakinT3,
}

impl FindFrameExtra13Kind {
    /// The vanilla `NPC.type` — the `FindFrame` switch key.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::DD2SkeletonT1 => 566,
            Self::DD2SkeletonT3 => 567,
            Self::DD2DrakinT2 => 570,
            Self::DD2DrakinT3 => 571,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// entry `type` on line `67084 + type/10`, slot `type % 10`) —
    /// cross-checked against `render/entities.rs::NPC_FRAME_COUNT`.
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67140 (560-569): slots 6/7 — the 9-row skeleton
            // sheet (rows 0-7 the walk, row 8 the air pin). The v1
            // stand-in's 14 was WRONG (the goblin count).
            Self::DD2SkeletonT1 | Self::DD2SkeletonT3 => 9,
            // Main.cs:67141 (570-579): slots 0/1 — the 16-row drakin
            // sheet (rows 2-9 the walk, 10-15 the attack ladder).
            Self::DD2DrakinT2 | Self::DD2DrakinT3 => 16,
        }
    }
}

/// The wiring seam: the vanilla type → this module's kind (the
/// `find_frame_kind_extra12` precedent). `None` outside the slice.
pub fn find_frame_kind_extra13(vanilla_type: i16) -> Option<FindFrameExtra13Kind> {
    Some(match vanilla_type {
        566 => FindFrameExtra13Kind::DD2SkeletonT1,
        567 => FindFrameExtra13Kind::DD2SkeletonT3,
        570 => FindFrameExtra13Kind::DD2DrakinT2,
        571 => FindFrameExtra13Kind::DD2DrakinT3,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for these arms: `frame.Y /
/// num` (the row), `frameCounter` and `spriteDirection` — no arm
/// writes `rotation`. The four-field shape is the e11/e12 bridge
/// convention (the wiring copies all four to/from `Npc`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter` (the C# field is a `double`, `NPC.cs:6485`;
    /// the port's f32 is the existing `npc.rs` convention — the
    /// integer accumulators here are exact in both).
    pub counter: f32,
    /// `NPC.spriteDirection` (±1; the copies here are UNCONDITIONAL).
    pub sprite_dir: i32,
    /// `NPC.rotation` — these arms never write it.
    pub rotation: f32,
}

/// The per-tick inputs the arms read: `velocity`, `direction` and (the
/// drakins only) `ai[0]`/`ai[1]` (the attack gate + the 59 reset). The
/// six-field shape is the e11/e12 bridge convention; the skeletons
/// read no ai slot at all.
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; the copies are UNCONDITIONAL, 0
    /// included — `NPC.cs:67563`/`:67715`/`:67735`).
    pub direction: i32,
    /// `NPC.ai[0]` — the drakin attack gate's second slot (`> 0`).
    pub ai0: f32,
    /// `NPC.ai[1]` — the drakin attack gate (`> 0`) + the == 59 reset.
    pub ai1: f32,
    pub ai2: f32,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1, ai0: 0.0, ai1: 0.0, ai2: 0.0 }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-13 kinds — one
/// tick.
pub fn find_frame_extra13(kind: FindFrameExtra13Kind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // `case 566`/`case 567` (`NPC.cs:67559-67560` → `:67561-67583`):
        // the goblin-shape WALK ONLY — no attack ladder, no ai reads.
        FindFrameExtra13Kind::DD2SkeletonT1 | FindFrameExtra13Kind::DD2SkeletonT3 => {
            // `:67561-67564` — the grounded UNCONDITIONAL sprite copy.
            if i.vy == 0.0 {
                s.sprite_dir = i.direction;
            }
            // `:67565-67570` — the air-OR-backpedal row-8 pin.
            if i.vy != 0.0
                || (i.direction == -1 && i.vx > 0.0)
                || (i.direction == 1 && i.vx < 0.0)
            {
                s.counter = 0.0;
                s.row = 8;
                return;
            }
            // `:67571-67576` — the idle row-0 snap.
            if i.vx == 0.0 {
                s.counter = 0.0;
                s.row = 0;
                return;
            }
            // `:67577-67582` — the |vx| accumulator, the >= 64 (or
            // negative) reset, the (int)(counter/8) row 0-7.
            s.counter += i.vx.abs();
            if s.counter >= 64.0 || s.counter < 0.0 {
                s.counter = 0.0;
            }
            s.row = (s.counter / 8.0) as i32 as u32;
        }
        // `case 570`/`case 571` (`NPC.cs:67711-67712` → `:67713-67755`):
        // the bomber-shape ladder with the row-14 num188 = 35 hold.
        FindFrameExtra13Kind::DD2DrakinT2 | FindFrameExtra13Kind::DD2DrakinT3 => {
            // `:67713-67732` — the attack ladder.
            if i.ai1 > 0.0 && i.ai0 > 0.0 {
                // `:67715` — the UNCONDITIONAL sprite copy.
                s.sprite_dir = i.direction;
                // `:67716-67720` — the row outside 10..15 OR the
                // ai[1] == 59 reset.
                if s.row < 10 || s.row > 15 || i.ai1 == 59.0 {
                    s.row = 10;
                    s.counter = 0.0;
                }
                // `:67721-67725` — num188 = 5 EXCEPT row 14 = 35.
                let mut num188 = 5.0;
                if s.row == 14 {
                    num188 = 35.0;
                }
                // `:67726-67730` — the ++counter >= num188 step gated
                // on row < 15 (at row 15 the counter keeps growing).
                s.counter += 1.0;
                if s.counter >= num188 && s.row < 15 {
                    s.counter = 0.0;
                    s.row += 1;
                }
                return;
            }
            // `:67733-67736` — the grounded sprite copy.
            if i.vy == 0.0 {
                s.sprite_dir = i.direction;
            }
            // `:67737-67742` — the air-OR-backpedal row-1 pin.
            if i.vy != 0.0
                || (i.direction == -1 && i.vx > 0.0)
                || (i.direction == 1 && i.vx < 0.0)
            {
                s.counter = 0.0;
                s.row = 1;
                return;
            }
            // `:67743-67748` — the idle row-9 snap.
            if i.vx == 0.0 {
                s.counter = 0.0;
                s.row = 9;
                return;
            }
            // `:67749-67754` — the |vx| accumulator, the >= 64 (or
            // negative) reset, the (int)(counter/8 + 2) row 2-9 (the
            // +2 INSIDE the cast, the C# literal).
            s.counter += i.vx.abs();
            if s.counter >= 64.0 || s.counter < 0.0 {
                s.counter = 0.0;
            }
            s.row = (s.counter / 8.0 + 2.0) as i32 as u32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 566/567 the skeleton walk + 570/571 the drakin ladder/walk —
    /// every band edge, the >= 64 reset tick, the drakin 5/35 strides,
    /// the ai[1] == 59 reset, the row-15 counter-keep-growing quirk.
    /// Expectations re-derived by mechanical replay of the
    /// line-for-line C# transcription (integer accumulators exact in
    /// f32).
    #[test]
    fn f27s13_skeleton_walk_and_drakin_ladder_replays() {
        for k in [FindFrameExtra13Kind::DD2SkeletonT1, FindFrameExtra13Kind::DD2SkeletonT3] {
            // vx 4: +4 a tick — the counter/8 rows 0-7, the >= 64
            // reset tick shows row 0 (the goblin walk verbatim).
            let mut s = FrameState::default();
            let mut rows = Vec::new();
            for _ in 0..16 {
                find_frame_extra13(k, &mut s, &FrameInput { vx: 4.0, ..Default::default() });
                rows.push(s.row);
            }
            assert_eq!(rows, vec![0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 0]);
            // The air pin: row 8, counter zeroed, the sprite/rotation
            // UNTOUCHED (the vy == 0 copy skipped).
            let mut s = FrameState { row: 3, counter: 9.0, sprite_dir: 1, rotation: 0.5 };
            let i = FrameInput { vx: 4.0, vy: -3.0, direction: -1, ..Default::default() };
            find_frame_extra13(k, &mut s, &i);
            assert_eq!((s.row, s.counter, s.sprite_dir, s.rotation), (8, 0.0, 1, 0.5));
            // The backpedal pin: direction -1 + vx +2 → row 8, but the
            // grounded sprite copy fires FIRST (-1).
            let mut s = FrameState::default();
            let i = FrameInput { vx: 2.0, direction: -1, ..Default::default() };
            find_frame_extra13(k, &mut s, &i);
            assert_eq!((s.row, s.counter, s.sprite_dir), (8, 0.0, -1));
            // The idle snap: row 0 (NOT the goblins' row 9).
            let mut s = FrameState { row: 5, counter: 4.0, ..Default::default() };
            find_frame_extra13(k, &mut s, &FrameInput::default());
            assert_eq!((s.row, s.counter), (0, 0.0));
            // The negative-counter guard: -5 + 2 = -3 < 0 → 0, row 0.
            let mut s = FrameState { counter: -5.0, ..Default::default() };
            find_frame_extra13(k, &mut s, &FrameInput { vx: 2.0, ..Default::default() });
            assert_eq!((s.row, s.counter), (0, 0.0));
        }
        for k in [FindFrameExtra13Kind::DD2DrakinT2, FindFrameExtra13Kind::DD2DrakinT3] {
            // The ladder: ai0 1 + ai1 1 — the 5-tick stride, rows
            // 10(x4)/11(x5)/12(x5)/13(x5)/14 (the reset gate lifts the
            // fresh row 0 to 10 on tick 1).
            let mut s = FrameState::default();
            let i = FrameInput { ai0: 1.0, ai1: 1.0, ..Default::default() };
            let mut rows = Vec::new();
            for _ in 0..20 {
                find_frame_extra13(k, &mut s, &i);
                rows.push(s.row);
            }
            assert_eq!(
                rows,
                vec![10, 10, 10, 10, 11, 11, 11, 11, 11, 12, 12, 12, 12, 12, 13, 13, 13, 13, 13, 14]
            );
            // The row-14 hold: num188 = 35 — 34 hold ticks, the 35th
            // steps to 15 (the ladder ignores velocity entirely; the
            // sprite copy fires even airborne).
            let mut s = FrameState { row: 14, counter: 0.0, ..Default::default() };
            let i = FrameInput { vy: -3.0, direction: -1, ai0: 1.0, ai1: 1.0, ..Default::default() };
            for tick in 1..=35 {
                find_frame_extra13(k, &mut s, &i);
                if tick < 35 {
                    assert_eq!((s.row, s.counter), (14, tick as f32), "tick {tick}");
                } else {
                    assert_eq!((s.row, s.counter), (15, 0.0));
                }
                assert_eq!(s.sprite_dir, -1);
            }
            // The row-15 quirk: the counter keeps growing, NO reset
            // (the C# gates the step on row < 15).
            let mut s = FrameState { row: 15, counter: 10.0, ..Default::default() };
            find_frame_extra13(k, &mut s, &i);
            assert_eq!((s.row, s.counter), (15, 11.0));
            // The ai[1] == 59 reset: row 12 + counter 3 → row 10,
            // counter 0 THEN ++ → 1.
            let mut s = FrameState { row: 12, counter: 3.0, ..Default::default() };
            let i = FrameInput { ai0: 1.0, ai1: 59.0, ..Default::default() };
            find_frame_extra13(k, &mut s, &i);
            assert_eq!((s.row, s.counter), (10, 1.0));
            // The gate needs BOTH slots: ai0 alone falls to the walk.
            let mut s = FrameState::default();
            let i = FrameInput { vx: 4.0, ai0: 1.0, ..Default::default() };
            find_frame_extra13(k, &mut s, &i);
            assert_eq!((s.row, s.counter), (2, 4.0));
            // The walk: vx 4 — the counter/8+2 strip rows 2-9, the
            // >= 64 reset tick shows row 2 (the bomber walk verbatim).
            let mut s = FrameState::default();
            let mut rows = Vec::new();
            for _ in 0..16 {
                find_frame_extra13(k, &mut s, &FrameInput { vx: 4.0, ..Default::default() });
                rows.push(s.row);
            }
            assert_eq!(rows, vec![2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 2]);
            // The air pin: row 1; the idle snap: row 9.
            let mut s = FrameState { row: 5, counter: 4.0, ..Default::default() };
            find_frame_extra13(k, &mut s, &FrameInput { vx: 4.0, vy: -3.0, ..Default::default() });
            assert_eq!((s.row, s.counter), (1, 0.0));
            find_frame_extra13(k, &mut s, &FrameInput::default());
            assert_eq!((s.row, s.counter), (9, 0.0));
        }
    }

    /// The frame counts (Main.cs:67140 slots 6/7 = 9, :67141 slots
    /// 0/1 = 16) match the port's verbatim table; the seam claims
    /// exactly 566/567/570/571; a velocity/direction/ai sweep stays
    /// inside each kind's own sheet.
    #[test]
    fn f27s13_frame_counts_seam_and_strip_sweep() {
        for k in [
            FindFrameExtra13Kind::DD2SkeletonT1,
            FindFrameExtra13Kind::DD2SkeletonT3,
            FindFrameExtra13Kind::DD2DrakinT2,
            FindFrameExtra13Kind::DD2DrakinT3,
        ] {
            assert_eq!(
                k.frame_count(),
                crate::render::entities::NPC_FRAME_COUNT[k.vanilla_type() as usize],
                "type {}",
                k.vanilla_type()
            );
        }
        assert_eq!(FindFrameExtra13Kind::DD2SkeletonT1.frame_count(), 9);
        assert_eq!(FindFrameExtra13Kind::DD2DrakinT2.frame_count(), 16);
        for t in 0..697i16 {
            let seam = find_frame_kind_extra13(t);
            assert_eq!(seam.is_some(), matches!(t, 566 | 567 | 570 | 571), "type {t}");
            if let Some(sk) = seam {
                assert_eq!(sk.vanilla_type(), t);
            }
        }
        for k in [
            FindFrameExtra13Kind::DD2SkeletonT1,
            FindFrameExtra13Kind::DD2SkeletonT3,
            FindFrameExtra13Kind::DD2DrakinT2,
            FindFrameExtra13Kind::DD2DrakinT3,
        ] {
            for vx in [-4.0f32, 0.0, 4.0] {
                for vy in [-3.0f32, 0.0] {
                    for direction in [-1i32, 1] {
                        for (ai0, ai1) in [(0.0f32, 0.0f32), (1.0, 0.0), (1.0, 1.0), (1.0, 59.0)] {
                            let mut s = FrameState::default();
                            for tick in 0..60 {
                                let i = FrameInput { vx, vy, direction, ai0, ai1, ..Default::default() };
                                find_frame_extra13(k, &mut s, &i);
                                assert!(
                                    s.row < k.frame_count(),
                                    "tick {tick} row {} (type {})",
                                    s.row,
                                    k.vanilla_type()
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
