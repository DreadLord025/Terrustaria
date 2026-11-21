//! W4-f27 — FindFrame extra wave 12 (slice 12): the 551/564-577 DD2
//! tail (the slice-11 DEFERRED window). The slice-11 claim deferred
//! 551/564-577 as "the complex/RNG/emote/dust seams"; slice 12
//! re-checked EVERY cited range mechanically (a `Main.rand`/`.Next(`/
//! `NextFloat`/`NextDouble`/`NewBubble`/`Dust.NewDust` grep per range):
//! * **551 Betsy** (`:67258-67451`) — ZERO RNG. Ported + wired (the
//!   prior run, `70762233d`/`01888909a`): the ai[0] phase ladders.
//! * **566/567 skeleton** (`:67559-67583`), **568/569 wither beast**
//!   (`:67025-67257`), **570/571 drakin** (`:67711-67755`) — ZERO RNG,
//!   the ROW-index convention (`num` cancels). Ported + wired this
//!   slice (the 6-kind strip batch; the npc.rs v1 free strips WRONG).
//! * **564/565 dark mage** (`:67998-68421`), **576/577 ogre**
//!   (`:67584-67709`) — ZERO RNG, but the RAW-`frame.Y` GRID-cell
//!   convention (the draw is `Frame(5, 9, y/9, y%9)` `Main.cs:23890` /
//!   `Frame(5, 10, y/10, y%10)` `:23792`, NOT the strip row). Ported
//!   PURE this slice; the wiring is gated on the grid-draw seam (the
//!   contract below — the 435/436 pure-landing precedent).
//! * **572/573 kobold walker** (`:67493-67558`) — the lit-state row-8
//!   pin carries an `EmoteBubble.NewBubble(3, …, 60)` side effect
//!   (`:67506`); **574/575 kobold flyer** (`:67887-67978`) — the
//!   `AngleTowards` rotation + the row-9 ignition dust burst
//!   (`:67923-67931`) + the `ai[0] >= 3` RNG trail dust
//!   (`Main.rand.Next(maxValue)` `:67959`). DEFERRED — the emote/RNG
//!   seam contracts are published in the ANALYSIS claim (the 634
//!   SporeBat resolved-draw precedent), no machine this slice.
//!
//! The 551 Betsy machine (`NPC.cs:67258-67451`, `NPCID.DD2Betsy` =
//! 551, `ID/NPCID.cs:11779`): the ai[0] phase dispatch over the 11-row
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
//! The 6-kind strip batch (the ROW convention, `num` cancels):
//! * **566/567 the skeletons** (`:67559-67583`): the grounded walk —
//!   the vy == 0 sprite copy, the air-OR-backpedal row-8 pin, the
//!   idle row-0 snap, the |vx| accumulator with the >= 64 reset and
//!   the `(int)(counter/8)` row 0-7 (the 9-row sheet). The slice-11
//!   "materialize-cycle seam" deferral reason is AI-side (the
//!   localAI[3] spawn-in), NOT in this range — the FindFrame body is
//!   the plain walk, zero RNG, ported verbatim.
//! * **568/569 the wither beasts** (`:67025-67257`): the ai[0] > 0
//!   aura ladder — the UNCONDITIONAL sprite copy, the row outside
//!   5..16 zeroes the counter (LIVE — the ++counter follows, no
//!   overwrite; the row read is otherwise dead, y is always
//!   recomputed), the ++counter then the 27-rung 5-tick ladder rows
//!   8/9/10 (k 1-3) then 7/8/9/10 x5 (k 4-23) then 11/12/13/14
//!   (k 24-27) (the ILSpy num60 +1 noise = the static 5*k
//!   thresholds) with the >= 270 tail pinning row 14 and SUBTRACTING
//!   10 (`:67227-67231` — the counter cycles 260..270 forever);
//!   else the walk: air row 4,
//!   idle row 6, the |vx| >= 56 reset + `(int)(counter/8)` row 0-6.
//! * **570/571 the drakins** (`:67711-67755`): the ai[1] > 0 &&
//!   ai[0] > 0 attack ladder rows 10-15 (the ai[1] == 59 reset, the
//!   num188 = 5 step with the ROW-14 num188 = 35 HOLD, the >= step
//!   gated row < 15 — at row 15 the counter keeps growing); else the
//!   walk: air-OR-backpedal row 1, idle row 9, the |vx| >= 64 reset
//!   + `(int)(counter/8 + 2)` row 2-9 (the bomber's +2 strip).
//!
//! The 4-kind GRID batch (the RAW-`frame.Y` convention — for these
//! kinds `FrameState::row` holds the C#'s `frame.Y` VERBATIM, the
//! grid-cell index, no `num` multiply; `is_grid()` marks them):
//! * **564/565 the dark mages** (`:67998-68421`): the frame.Width/
//!   Height = 80 override (`:68002-68003`); the ai[0] > 0 && ai[1]
//!   == 0 summon ladder (7-tick, rows 6/7 then 5/6/7 x2 then
//!   8..12 then the >= 7*14 = 98 reset-to-5 + counter 0), the
//!   ai[1] == 1 attack ladder (8-tick, rows 14..20 then 18/19/20
//!   then 21..25 then the >= 8*16 = 128 reset-to-14 + counter 0),
//!   the ai[1] == 2 gate ladder (8-tick, 27/28/29 then 26/27/28/29
//!   x2 then 30..40 then the >= 8*23 = 184 reset-to-26 + counter 0)
//!   — each with the
//!   entry out-of-band counter-zero (NOT dead here: the counter++
//!   follows, no ai[1] overwrite) and the UNCONDITIONAL sprite
//!   copy; else the hover walk: counter += |velocity|*0.1 + 1, the
//!   >= 40 reset, `(int)(counter/8)` rows 0-4. The draw samples the
//!   5x9 grid (`Main.cs:23890` — y3/9 column, y3%9 row; the sheet
//!   runs PAST npcFrameCount[564] = 9, the 41-row-of-80px sheet).
//! * **576/577 the ogres** (`:67584-67709`): the frame.Width = 80
//!   override (`:67588`); the ai[0] > 0 && ai[1] == 0 stomp ladder
//!   (cells 11-20, the variable strides 4 / 8 at 13/19 / 2 at 14/18,
//!   the >= step gated cell < 20), the ai[1] == 2 pound ladder
//!   (cells 37-47, the entry snap to 39 NOT 37, the strides 5 / 6
//!   at 42 / 8 at 45 / 4 at 46 / 26 at 47 / 7 at 37/38, the
//!   counter-HOLD at 38 and at 46-while-airborne (`flag10`), the
//!   >= 47 wrap-to-37), the ai[1] == 1 swipe ladder (cells 21-38,
//!   the 5-tick step gated cell < 38); else the walk: air cell 43
//!   (the vy != 0 pin — NO backpedal branch), idle cell 0, the |vx|
//!   >= 60 reset + `1 + (int)(counter/6)` cells 1-10. The draw
//!   samples the 5x10 grid (`Main.cs:23792`). The entry gate reads
//!   the RAW cell (`num144 = this.frame.Y` `:67587` — the comparisons
//!   are cell-index, NOT pixel).
//!
//! The GRID-DRAW SEAM CONTRACT (the wiring gate for 564/565/576/577):
//! the port's draw arms sample STRIPS (`t.h / 20` / `t.h / 10` rows,
//! `render/entities.rs`'s dark_mage/dd2_ogres anchors) — wiring the
//! raw-cell machines under the strip draw would sample the wrong
//! pixels. The wire round must FIRST convert the four draw arms to
//! grid sampling (the deerclops 5x5 `deerclops_frame_uv` precedent):
//! dark mage `Frame(5, 9, cell/9, cell%9)` (`Main.cs:23890`, the
//! origin-X 52/-52 sprite flip + the origin-Y -10 + the localAI[3]
//! materialize fade), ogre `Frame(5, 10, cell/10, cell%10)`
//! (`:23792`, the origin-X 94 + the origin-Y -4). Until then the
//! four machines ride PURE (the seam fn claims the ids; the npc.rs
//! v1 strip arms stand — the 435/436 pure-landing precedent).
//!
//! Row convention: the port tracks frame ROW indices, not pixels —
//! the C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out for the STRIP kinds.
//! `Main.npcFrameCount[551]` = 11 (`Main.cs:67139` slot 1),
//! 566-567 = 9 (`:67140` slots 6/7), 568/569 = 17 (slots 8/9),
//! 570/571 = 16 (`:67141` slots 0/1), 564/565 = 9 (`:67140` slots
//! 4/5 — the GRID kinds' count is the DRAW row count, the machine
//! cells run to 40/47), 576/577 = 10 (`:67141` slots 6/7) —
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
    /// `NPCID.DD2DarkMageT1` = 564 (`ID/NPCID.cs:11805`) — the GRID
    /// caster ladders (`NPC.cs:67998`), the raw-cell convention.
    DD2DarkMageT1,
    /// `NPCID.DD2DarkMageT3` = 565 (`ID/NPCID.cs:11807`) — the same
    /// GRID machine (`NPC.cs:67999`).
    DD2DarkMageT3,
    /// `NPCID.DD2SkeletonT1` = 566 (`ID/NPCID.cs:11809`) — the
    /// grounded walk (`NPC.cs:67559`).
    DD2SkeletonT1,
    /// `NPCID.DD2SkeletonT3` = 567 (`ID/NPCID.cs:11811`) — the same
    /// walk (`NPC.cs:67560`).
    DD2SkeletonT3,
    /// `NPCID.DD2WitherBeastT2` = 568 (`ID/NPCID.cs:11813`) — the
    /// aura ladder + walk (`NPC.cs:67025`).
    DD2WitherBeastT2,
    /// `NPCID.DD2WitherBeastT3` = 569 (`ID/NPCID.cs:11815`) — the
    /// same machine (`NPC.cs:67026`).
    DD2WitherBeastT3,
    /// `NPCID.DD2DrakinT2` = 570 (`ID/NPCID.cs:11817`) — the attack
    /// ladder + walk (`NPC.cs:67711`).
    DD2DrakinT2,
    /// `NPCID.DD2DrakinT3` = 571 (`ID/NPCID.cs:11819`) — the same
    /// machine (`NPC.cs:67712`).
    DD2DrakinT3,
    /// `NPCID.DD2OgreT2` = 576 (`ID/NPCID.cs:11829`) — the GRID
    /// stomp/pound/swipe ladders (`NPC.cs:67584`), the raw-cell
    /// convention.
    DD2OgreT2,
    /// `NPCID.DD2OgreT3` = 577 (`ID/NPCID.cs:11831`) — the same GRID
    /// machine (`NPC.cs:67585`).
    DD2OgreT3,
}

impl FindFrameExtra12Kind {
    /// The vanilla `NPC.type` — the `FindFrame` switch key.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::DD2Betsy => 551,
            Self::DD2DarkMageT1 => 564,
            Self::DD2DarkMageT3 => 565,
            Self::DD2SkeletonT1 => 566,
            Self::DD2SkeletonT3 => 567,
            Self::DD2WitherBeastT2 => 568,
            Self::DD2WitherBeastT3 => 569,
            Self::DD2DrakinT2 => 570,
            Self::DD2DrakinT3 => 571,
            Self::DD2OgreT2 => 576,
            Self::DD2OgreT3 => 577,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// entry `type` on line `67084 + type/10`, slot `type % 10`) —
    /// cross-checked against `render/entities.rs::NPC_FRAME_COUNT`.
    /// For the GRID kinds (564/565/576/577) this is the DRAW row
    /// count — the machine's raw cells run past it (the dark mage
    /// to 40 on the 5x9 grid, the ogre to 47 on the 5x10 grid).
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67139 (550-559): slot 1 — the 11-row dragon
            // sheet (rows 0-4 flight, 5-10 the breath phases).
            Self::DD2Betsy => 11,
            // Main.cs:67140 (560-569): slots 4/5 — the 9-row draw
            // count (the 5x9 GRID sheet's rows).
            Self::DD2DarkMageT1 | Self::DD2DarkMageT3 => 9,
            // Main.cs:67140: slots 6/7 — the 9-row skeleton sheet.
            Self::DD2SkeletonT1 | Self::DD2SkeletonT3 => 9,
            // Main.cs:67140: slots 8/9 — the 17-row wither sheet.
            Self::DD2WitherBeastT2 | Self::DD2WitherBeastT3 => 17,
            // Main.cs:67141 (570-579): slots 0/1 — the 16-row drakin
            // sheet.
            Self::DD2DrakinT2 | Self::DD2DrakinT3 => 16,
            // Main.cs:67141: slots 6/7 — the 10-row draw count (the
            // 5x10 GRID sheet's rows).
            Self::DD2OgreT2 | Self::DD2OgreT3 => 10,
        }
    }

    /// Whether the kind's machine rides the RAW-`frame.Y` GRID-cell
    /// convention (`FrameState::row` holds the raw cell index) rather
    /// than the strip ROW convention. The grid kinds' wiring is gated
    /// on the grid-draw seam (the module header).
    pub fn is_grid(self) -> bool {
        matches!(
            self,
            Self::DD2DarkMageT1 | Self::DD2DarkMageT3 | Self::DD2OgreT2 | Self::DD2OgreT3
        )
    }
}

/// The wiring seam: the vanilla type → this module's kind (the
/// `find_frame_kind_extra11` precedent). `None` outside the slice.
pub fn find_frame_kind_extra12(vanilla_type: i16) -> Option<FindFrameExtra12Kind> {
    Some(match vanilla_type {
        551 => FindFrameExtra12Kind::DD2Betsy,
        564 => FindFrameExtra12Kind::DD2DarkMageT1,
        565 => FindFrameExtra12Kind::DD2DarkMageT3,
        566 => FindFrameExtra12Kind::DD2SkeletonT1,
        567 => FindFrameExtra12Kind::DD2SkeletonT3,
        568 => FindFrameExtra12Kind::DD2WitherBeastT2,
        569 => FindFrameExtra12Kind::DD2WitherBeastT3,
        570 => FindFrameExtra12Kind::DD2DrakinT2,
        571 => FindFrameExtra12Kind::DD2DrakinT3,
        576 => FindFrameExtra12Kind::DD2OgreT2,
        577 => FindFrameExtra12Kind::DD2OgreT3,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for these arms: the
/// frame row (`frame.Y / num` for the STRIP kinds, the RAW `frame.Y`
/// grid-cell index for the GRID kinds — `is_grid()`), `frameCounter`,
/// `spriteDirection` and `rotation`. Case 551 touches only row +
/// counter; the strip walks write the sprite on the grounded copy;
/// no arm writes rotation. The four-field shape is the e11 bridge
/// convention (the wiring copies all four to/from `Npc`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row (STRIP kinds); the RAW
    /// `frame.Y` grid-cell index (GRID kinds, `is_grid()`).
    pub row: u32,
    /// `NPC.frameCounter` (the C# field is a `double`, `NPC.cs:6485`;
    /// the port's f32 is the existing `npc.rs` convention — the
    /// integer accumulators here are exact in both; the dark mage's
    /// hover walk adds `|velocity|*0.1 + 1`, the one non-integer
    /// feed, the e10 sand-elemental precedent).
    pub counter: f32,
    /// `NPC.spriteDirection` (±1) — the strip walks' grounded copy +
    /// the grid ladders' UNCONDITIONAL copy write it; case 551 never.
    pub sprite_dir: i32,
    /// `NPC.rotation` — no extra-12 arm writes it (the kobold flyer's
    /// AngleTowards is the DEFERRED 574/575 seam, not this module).
    pub rotation: f32,
}

/// The per-tick inputs the arms read: `velocity`, `direction`, the
/// `ai[0]`/`ai[1]` slots. The six-field shape is the e11 bridge
/// convention; case 551 reads only ai0/ai1 (no velocity/direction),
/// the strip walks read velocity + direction, the grid ladders read
/// ai0/ai1 + direction (+ vy for the ogre's airborne pound-hold).
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; the strip walks' grounded copy is
    /// UNCONDITIONAL, the grid ladders' copy UNCONDITIONAL).
    pub direction: i32,
    /// `NPC.ai[0]` — Betsy's phase dispatch (4/3/5/6), the wither
    /// beasts' aura gate (`> 0`), the drakins' attack gate
    /// (`ai[1] > 0 && ai[0] > 0`), the grid ladders' attack gate
    /// (`> 0`).
    pub ai0: f32,
    /// `NPC.ai[1]` — Betsy's phase timer (truncated `(int)` into the
    /// counter every ladder tick); the drakins' 59 reset; the grid
    /// ladders' 0/1/2 sub-state selector.
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
        // `case 566`/`case 567` (`NPC.cs:67559-67560` → `:67561-67583`):
        // the grounded walk — the vy == 0 sprite copy (`:67561-67564`),
        // the air-OR-backpedal row-8 pin (`:67565-67570`), the idle
        // row-0 snap (`:67571-67576`), the |vx| accumulator with the
        // >= 64 (or negative) reset (`:67577-67581`) and the
        // `(int)(counter/8)` row 0-7 (`:67582`). The goblin walk shape
        // with idle row 0 (not 9). Zero RNG — the slice-11
        // "materialize-cycle" deferral is AI-side, not this range.
        FindFrameExtra12Kind::DD2SkeletonT1 | FindFrameExtra12Kind::DD2SkeletonT3 => {
            // `:67561-67564`.
            if i.vy == 0.0 {
                s.sprite_dir = i.direction;
            }
            // `:67565-67570`.
            if i.vy != 0.0
                || (i.direction == -1 && i.vx > 0.0)
                || (i.direction == 1 && i.vx < 0.0)
            {
                s.counter = 0.0;
                s.row = 8;
                return;
            }
            // `:67571-67576`.
            if i.vx == 0.0 {
                s.counter = 0.0;
                s.row = 0;
                return;
            }
            // `:67577-67582`.
            s.counter += i.vx.abs();
            if s.counter >= 64.0 || s.counter < 0.0 {
                s.counter = 0.0;
            }
            s.row = (s.counter / 8.0) as i32 as u32;
        }
        // `case 568`/`case 569` (`NPC.cs:67025-67026` → `:67027-67257`):
        // the ai[0] > 0 aura ladder (`:67027-67233`) + the walk
        // (`:67235-67256`). The ladder: the UNCONDITIONAL sprite copy
        // (`:67030`), the entry row outside 5..16 zeroes the counter
        // (`:67031-67034` — LIVE, the ++ follows), the row base 7
        // (`:67035`), the ++counter (`:67036`), the 27 sequential
        // `if (counter >= 5*k)` rungs (`:67037-67226`, the ILSpy num60
        // +1 noise = the static 5*k thresholds; last-match-wins) and
        // the >= 270 tail pinning row 14 + SUBTRACTING 10
        // (`:67227-67231`). Else the walk: air row 4, idle row 6, the
        // |vx| >= 56 reset + `(int)(counter/8)` row 0-6.
        FindFrameExtra12Kind::DD2WitherBeastT2 | FindFrameExtra12Kind::DD2WitherBeastT3 => {
            // `:67027-67233`.
            if i.ai0 > 0.0 {
                // `:67029-67034` — the entry gate reads the CURRENT
                // row (before the base-7 overwrite); LIVE counter reset.
                if s.row < 5 || s.row > 16 {
                    s.counter = 0.0;
                }
                // `:67030` — the UNCONDITIONAL sprite copy.
                s.sprite_dir = i.direction;
                // `:67035-67036`.
                s.row = 7;
                s.counter += 1.0;
                // `:67037-67226` — the 27 rungs, row per k = 1..27:
                // k 1-3 → 8/9/10, k 4-23 → 7/8/9/10 x5, k 24-27 →
                // 11/12/13/14. Last-match-wins (monotonic thresholds).
                const AURA: [u32; 27] = [
                    8, 9, 10, 7, 8, 9, 10, 7, 8, 9, 10, 7, 8, 9, 10, 7, 8, 9, 10, 7, 8, 9,
                    10, 11, 12, 13, 14,
                ];
                for k in 1..=27usize {
                    if s.counter >= (5 * k) as f32 {
                        s.row = AURA[k - 1];
                    }
                }
                // `:67227-67231` — the >= 270 tail: row 14, counter -10
                // (the counter cycles 260..270 forever, never resets 0).
                if s.counter >= 270.0 {
                    s.row = 14;
                    s.counter -= 10.0;
                }
                // `:67232` — frame.Y = num * row.
                return;
            }
            // `:67235-67238`.
            if i.vy == 0.0 {
                s.sprite_dir = i.direction;
            }
            // `:67239-67244`.
            if i.vy != 0.0
                || (i.direction == -1 && i.vx > 0.0)
                || (i.direction == 1 && i.vx < 0.0)
            {
                s.counter = 0.0;
                s.row = 4;
                return;
            }
            // `:67245-67250`.
            if i.vx == 0.0 {
                s.counter = 0.0;
                s.row = 6;
                return;
            }
            // `:67251-67256`.
            s.counter += i.vx.abs();
            if s.counter >= 56.0 || s.counter < 0.0 {
                s.counter = 0.0;
            }
            s.row = (s.counter / 8.0) as i32 as u32;
        }
        // `case 570`/`case 571` (`NPC.cs:67711-67712` → `:67713-67755`):
        // the ai[1] > 0 && ai[0] > 0 attack ladder (`:67713-67731`) +
        // the walk (`:67733-67754`). The ladder: the UNCONDITIONAL
        // sprite copy, the row outside 10..15 OR ai[1] == 59 resets to
        // row 10 + counter 0, the num188 = 5 step with the ROW-14
        // num188 = 35 HOLD, the ++counter >= num188 step gated row < 15
        // (at row 15 the counter keeps growing). Else the walk: the
        // vy == 0 sprite copy, the air-OR-backpedal row-1 pin, the idle
        // row-9 snap, the |vx| >= 64 reset + `(int)(counter/8 + 2)` row
        // 2-9 (the bomber's +2 strip).
        FindFrameExtra12Kind::DD2DrakinT2 | FindFrameExtra12Kind::DD2DrakinT3 => {
            // `:67713-67731`.
            if i.ai1 > 0.0 && i.ai0 > 0.0 {
                s.sprite_dir = i.direction;
                if s.row < 10 || s.row > 15 || i.ai1 == 59.0 {
                    s.row = 10;
                    s.counter = 0.0;
                }
                // `:67721-67725` — num188 = 5, the row-14 = 35 hold.
                let step = if s.row == 14 { 35.0 } else { 5.0 };
                // `:67726-67730` — the ++counter >= step gated row < 15.
                s.counter += 1.0;
                if s.counter >= step && s.row < 15 {
                    s.counter = 0.0;
                    s.row += 1;
                }
                return;
            }
            // `:67733-67736`.
            if i.vy == 0.0 {
                s.sprite_dir = i.direction;
            }
            // `:67737-67742`.
            if i.vy != 0.0
                || (i.direction == -1 && i.vx > 0.0)
                || (i.direction == 1 && i.vx < 0.0)
            {
                s.counter = 0.0;
                s.row = 1;
                return;
            }
            // `:67743-67748`.
            if i.vx == 0.0 {
                s.counter = 0.0;
                s.row = 9;
                return;
            }
            // `:67749-67754`.
            s.counter += i.vx.abs();
            if s.counter >= 64.0 || s.counter < 0.0 {
                s.counter = 0.0;
            }
            s.row = (s.counter / 8.0 + 2.0) as i32 as u32;
        }
        // The GRID batch (564/565 dark mage, 576/577 ogre) lands in the
        // next commit — the catch-all keeps every intermediate commit
        // compiling (the e10/e11 batch precedent). The raw-cell
        // machines ride the grid-draw seam (the module header).
        _ => {}
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

    /// The frame counts (Main.cs:67139-67141) match the port's
    /// verbatim table for all 11 kinds; the seam claims exactly the
    /// 11 ids; the Betsy phase/timer sweep stays inside the 11-row
    /// sheet.
    #[test]
    fn f27s12_frame_count_seam_and_strip_sweep() {
        let kinds = [
            FindFrameExtra12Kind::DD2Betsy,
            FindFrameExtra12Kind::DD2DarkMageT1,
            FindFrameExtra12Kind::DD2DarkMageT3,
            FindFrameExtra12Kind::DD2SkeletonT1,
            FindFrameExtra12Kind::DD2SkeletonT3,
            FindFrameExtra12Kind::DD2WitherBeastT2,
            FindFrameExtra12Kind::DD2WitherBeastT3,
            FindFrameExtra12Kind::DD2DrakinT2,
            FindFrameExtra12Kind::DD2DrakinT3,
            FindFrameExtra12Kind::DD2OgreT2,
            FindFrameExtra12Kind::DD2OgreT3,
        ];
        let claimed = [551i16, 564, 565, 566, 567, 568, 569, 570, 571, 576, 577];
        for k in kinds {
            assert_eq!(
                k.frame_count(),
                crate::render::entities::NPC_FRAME_COUNT[k.vanilla_type() as usize],
                "type {}",
                k.vanilla_type()
            );
            assert_eq!(k.is_grid(), matches!(k.vanilla_type(), 564 | 565 | 576 | 577));
        }
        for t in 0..697i16 {
            let seam = find_frame_kind_extra12(t);
            assert_eq!(seam.is_some(), claimed.contains(&t), "type {t}");
            if let Some(sk) = seam {
                assert_eq!(sk.vanilla_type(), t);
            }
        }
        let k = FindFrameExtra12Kind::DD2Betsy;
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

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn rows_over(kind: FindFrameExtra12Kind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra12(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// 566/567 the skeletons: the grounded walk — air/backpedal row 8,
    /// idle row 0 (NOT the goblins' 9), the |vx| >= 64 reset +
    /// counter/8 row 0-7. Expectations re-derived by mechanical replay
    /// of the C# transcription (tools/_f27_s12_replay.py).
    #[test]
    fn f27s12_skeletons_walk_the_64_reset_idle_zero() {
        for k in [FindFrameExtra12Kind::DD2SkeletonT1, FindFrameExtra12Kind::DD2SkeletonT3] {
            // vx 4: +4 a tick — the (int)(counter/8) row, the >= 64
            // reset tick shows row 0.
            let rows = rows_over(k, input(4.0, 0.0), 16);
            assert_eq!(rows, vec![0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 0]);
            // The air pin: row 8, counter zeroed.
            let mut s = FrameState { row: 3, counter: 20.0, ..Default::default() };
            find_frame_extra12(k, &mut s, &input(4.0, -2.0));
            assert_eq!((s.row, s.counter), (8, 0.0));
            // The backpedal pin: direction -1 + vx +4 → row 8, the
            // grounded flip fires FIRST (sprite -1).
            let mut s = FrameState::default();
            let mut i = input(4.0, 0.0);
            i.direction = -1;
            find_frame_extra12(k, &mut s, &i);
            assert_eq!((s.row, s.counter, s.sprite_dir), (8, 0.0, -1));
            // The idle snap: row 0 (NOT 9), counter zeroed.
            let mut s = FrameState { row: 5, counter: 30.0, ..Default::default() };
            find_frame_extra12(k, &mut s, &input(0.0, 0.0));
            assert_eq!((s.row, s.counter), (0, 0.0));
            // The grounded sprite copy; no rotation write.
            let mut s = FrameState { rotation: 0.7, ..Default::default() };
            find_frame_extra12(k, &mut s, &input(4.0, 0.0));
            assert_eq!((s.sprite_dir, s.rotation), (1, 0.7));
        }
    }

    /// 568/569 the wither beasts: the ai[0] > 0 aura ladder (the
    /// 27-rung 5-tick machine + the >= 270 counter-10 tail) + the walk
    /// (air 4, idle 6, the |vx| >= 56 reset). Expectations re-derived
    /// by mechanical replay.
    #[test]
    fn f27s12_wither_beasts_ride_the_aura_ladder_and_the_270_tail() {
        for k in [FindFrameExtra12Kind::DD2WitherBeastT2, FindFrameExtra12Kind::DD2WitherBeastT3] {
            // The aura ladder from row 0 (the entry reset fires):
            // rows 7 x4, 8 x5, 9 x5, 10 x5, then the 7/8/9/10 cycle.
            let mut i = input(0.0, 0.0);
            i.ai0 = 1.0;
            let rows = rows_over(k, i, 40);
            assert_eq!(
                rows,
                vec![
                    7, 7, 7, 7, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 10, 10, 10, 10, 10, 7,
                    7, 7, 7, 7, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 10, 10, 10, 10, 10, 7,
                ]
            );
            // The >= 270 tail: from row 7 (in-band, no entry reset)
            // the counter climbs to 270, pins row 14, subtracts 10 →
            // 260, then cycles 260..270 forever (row 14 throughout).
            let mut s = FrameState { row: 7, counter: 268.0, ..Default::default() };
            find_frame_extra12(k, &mut s, &i);
            assert_eq!((s.row, s.counter), (14, 269.0));
            find_frame_extra12(k, &mut s, &i);
            assert_eq!((s.row, s.counter), (14, 260.0));
            // The entry reset is LIVE: row 0 (out of 5..16) zeroes the
            // counter before the ++ (so the first aura tick is 1).
            let mut s = FrameState { row: 0, counter: 99.0, ..Default::default() };
            find_frame_extra12(k, &mut s, &i);
            assert_eq!((s.row, s.counter), (7, 1.0));
            // The walk (ai0 0): vx 4 — the >= 56 reset tick shows row 0.
            let rows = rows_over(k, input(4.0, 0.0), 15);
            assert_eq!(rows, vec![0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 0, 0]);
            // The air pin row 4, the idle snap row 6.
            let mut s = FrameState { row: 9, counter: 40.0, ..Default::default() };
            find_frame_extra12(k, &mut s, &input(4.0, 2.0));
            assert_eq!((s.row, s.counter), (4, 0.0));
            find_frame_extra12(k, &mut s, &input(0.0, 0.0));
            assert_eq!((s.row, s.counter), (6, 0.0));
        }
    }

    /// 570/571 the drakins: the ai[1] > 0 && ai[0] > 0 attack ladder
    /// (the ai[1] == 59 reset, the row-14 num188 = 35 hold, the row-15
    /// counter-keep-growing quirk) + the walk (air 1, idle 9, the
    /// counter/8+2 strip). Expectations re-derived by mechanical replay.
    #[test]
    fn f27s12_drakins_ride_the_row14_hold_and_the_plus2_strip() {
        for k in [FindFrameExtra12Kind::DD2DrakinT2, FindFrameExtra12Kind::DD2DrakinT3] {
            // The attack ladder: rows 10 x4, 11/12/13 x5, 14 x35 (the
            // num188 = 35 hold), then 15 (the counter keeps growing).
            let mut i = input(0.0, 0.0);
            i.ai0 = 1.0;
            i.ai1 = 1.0;
            let rows = rows_over(k, i, 60);
            let mut expect = vec![10u32; 4];
            for r in [11u32, 12, 13] {
                expect.extend(std::iter::repeat(r).take(5));
            }
            expect.extend(std::iter::repeat(14u32).take(35));
            expect.extend(std::iter::repeat(15u32).take(6));
            assert_eq!(rows, expect);
            // The row-15 quirk: the counter keeps growing (no reset).
            let mut s = FrameState { row: 15, counter: 5.0, ..Default::default() };
            find_frame_extra12(k, &mut s, &i);
            assert_eq!((s.row, s.counter), (15, 6.0));
            // The ai[1] == 59 reset: row 10, counter 0 THEN ++ → 1.
            let mut s = FrameState { row: 13, counter: 3.0, ..Default::default() };
            let mut i59 = i;
            i59.ai1 = 59.0;
            find_frame_extra12(k, &mut s, &i59);
            assert_eq!((s.row, s.counter), (10, 1.0));
            // The ai0-only gate falls to the walk (the drakin gate is
            // ai1 && ai0, NOT the goblins' ai0-only).
            let mut s = FrameState::default();
            let mut i0 = input(4.0, 0.0);
            i0.ai0 = 1.0;
            find_frame_extra12(k, &mut s, &i0);
            assert_eq!(s.row, 2);
            // The walk: vx 4 — the counter/8+2 strip rows 2-9, the
            // >= 64 reset tick shows row 2.
            let rows = rows_over(k, input(4.0, 0.0), 16);
            assert_eq!(rows, vec![2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 2]);
            // The air pin row 1, the idle snap row 9.
            let mut s = FrameState { row: 5, counter: 30.0, ..Default::default() };
            find_frame_extra12(k, &mut s, &input(4.0, -2.0));
            assert_eq!((s.row, s.counter), (1, 0.0));
            find_frame_extra12(k, &mut s, &input(0.0, 0.0));
            assert_eq!((s.row, s.counter), (9, 0.0));
        }
    }
}
