//! W4-f26 slice 7b — FindFrame extra wave 7b: the NEXT unclaimed
//! missing case ids of `NPC.FindFrame` (`NPC.cs:66684-77370`), counted
//! off the ascending census after the d44-d49/e13/e14/f26/f27 claims
//! AND the landed d38/d40 roster arms (the pushed branches read: d44
//! owns 37/43/46/47/48/56/57/61/62/63/64/67, d45
//! 69/73/74/83/84/93/94/101/102/103/116/120, d46
//! 150/151/152/156/157/158/159/160/162/163/164/165, d47
//! 166/168/169/170/171/172/173/175/176/178/179/180, d48 355-366, d49
//! 367/374/375/377/378/379/380/430-436/591, e13
//! 121/137/141/142/148/149/181/182/185/195/196/198/199/201/202, e14's
//! declared next-15 203/204/205/209/217/218/219/220/221/224/226/229/
//! 230/231/232, f26 slice-3 182/185/195/196/198/199/201/202/203/204/
//! 205/209/217/218/219, f26 slice-5 254..258/268..277, f26 slice-6
//! 278..287/290..293/295, f27 wave-4 233..244/251..253 — the overlaps
//! dedupe). The census re-derived mechanically: 602 C# top-level case
//! ids − 248 port-covered (the `npc.rs::find_frame` arms through the
//! `cs_id` hand rows + the bestiary net ids + the wave-A/B rosters) −
//! 75 d44-d49 claims = 279 remaining (the e13 base, re-confirmed);
//! minus the 75 deduped e13/e14/f26×3/f27-wave-4 claim ids = 204;
//! minus f27 slice-7's pushed CLAIM (316/317/318/322/323/324/333/334/
//! 335/336/337 + 437/438/440/442, `origin/task/W4-f27-findframe-extra-4`
//! tip `ef07e06b`) = 189; minus the d38 LANDED arms for the in-band
//! remainder 297/298/299/300/301/302/303/304 (`origin/task/
//! W4-d38-npc-kinds-281-350` b4/b5 — the birds/squirrel/mouse/raven +
//! the halloween slimes/bunny ALL have `npc.rs` find_frame arms now,
//! so the 297..304 remainder is EMPTY) = 181; minus the d40 LANDED
//! arms 443..454/460..470/472 (`origin/task/W4-d40-npc-kinds-421-490`
//! batches 3-8) = 157 unclaimed. THIS wave owns the FIRST 15 of the
//! 351+ band from 473 up — **473/474/475/476/477/479/480/481/482/483/
//! 484/485/486/487/488** — the four Big Mimics, Mothron + its spawn,
//! Medusa, the Greek Skeleton, the Granite Golem/Flyer, the Enchanted
//! Nightcrawler, the Grubby/Sluggy/Buggy trio and the Target Dummy
//! (`ID/NPCID.cs:11619-11649`). 478 MothronEgg has NO case label in
//! the switch (the egg holds its single frame) — skipped with cite.
//! Landed 5 kinds per commit.
//!
//! Shape: the kinds are NOT in `NpcKind` yet (their AI/spawn machines
//! are later rounds), so — like `npc_findframe_a.rs` /
//! `npc_findframe_b.rs` (S1-D-3 waves A/B) and the f26/f27 extra waves —
//! the frame machines land first as pure functions over a small
//! [`FrameState`]/[`FrameInput`] pair. The wiring seam is
//! [`find_frame_kind_extra7b`] (the f27 precedent): when a kind joins
//! `NpcKind`, its `find_frame` arm copies the state fields out, runs
//! one tick and copies them back (the `Npc::find_frame_wave_a` bridge
//! precedent). No arm draws `Main.rand`, so the wiring cannot move a
//! stream.
//!
//! Row convention: the port tracks frame ROW indices, not pixels —
//! the C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer),
//! counted entry-by-entry in [`FindFrameExtra7bKind::frame_count`] and
//! cross-checked against the port's own `render/entities.rs::
//! NPC_FRAME_COUNT` table.
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "W4-f26 FindFrame
//! extra wave 7b"):
//! * the crawler/grub group's tile-slope probe (`NPC.cs:72807-72826`/
//!   `:72862-72881` — `Framing.GetTileSafely` at the center/position
//!   tile with the +1 fall-through) is a WORLD read: the pure machine
//!   takes the RESOLVED slope as [`FrameInput::slope`] (0 = none, 1/2
//!   = the two diagonal overrides); the bridge owns the two-tile
//!   probe;
//! * the Target Dummy's `Framing.GetTileSafely(ai[0], ai[1]).frameX
//!   >= 36` sign flip (`NPC.cs:70850-70853`) is the same kind of world
//!   read — the bridge passes [`FrameInput::frame_x36`];
//! * the mimic group's `ai[0]` values outside {0,1,2,3,4,4.1,5,6,7,8}
//!   BREAK with no write (`NPC.cs:71287-71290`) — carried verbatim.

// The wiring is a later round (the kinds are not in `NpcKind` yet) —
// the `npc_findframe_a.rs`/`npc_findframe_b.rs` precedent.
#![allow(dead_code)]

/// The extra-wave-7b kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra7bKind {
    /// `NPCID.BigMimicCorruption` = 473 (`NPCID.cs:11619`) — the
    /// 473..476 Big Mimic group (`NPC.cs:71159`).
    BigMimicCorruption,
    /// `NPCID.BigMimicCrimson` = 474 (`NPCID.cs:11621`) — the same
    /// group (`NPC.cs:71160`).
    BigMimicCrimson,
    /// `NPCID.BigMimicHallow` = 475 (`NPCID.cs:11623`) — the same
    /// group (`NPC.cs:71161`).
    BigMimicHallow,
    /// `NPCID.BigMimicJungle` = 476 (`NPCID.cs:11625`) — the same
    /// group (`NPC.cs:71162`).
    BigMimicJungle,
    /// `NPCID.Mothron` = 477 (`NPCID.cs:11627`) — `case 477`
    /// (`NPC.cs:71330-71342`).
    Mothron,
}

impl FindFrameExtra7bKind {
    /// The vanilla `NPC.type` — the `FindFrame` switch key.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::BigMimicCorruption => 473,
            Self::BigMimicCrimson => 474,
            Self::BigMimicHallow => 475,
            Self::BigMimicJungle => 476,
            Self::Mothron => 477,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// entry `type` on line `67084 + type/10`, slot `type % 10`) —
    /// cross-checked against `render/entities.rs::NPC_FRAME_COUNT`.
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67131 (entries 473..476): 14, 14, 14, 14.
            Self::BigMimicCorruption => 14,
            Self::BigMimicCrimson => 14,
            Self::BigMimicHallow => 14,
            Self::BigMimicJungle => 14,
            // Main.cs:67131 (entry 477): 6.
            Self::Mothron => 6,
        }
    }
}

/// The wiring seam: the vanilla type → this module's kind (the
/// `find_frame_kind_extra6` precedent). `None` outside the slice.
pub fn find_frame_kind_extra7b(vanilla_type: i16) -> Option<FindFrameExtra7bKind> {
    Some(match vanilla_type {
        473 => FindFrameExtra7bKind::BigMimicCorruption,
        474 => FindFrameExtra7bKind::BigMimicCrimson,
        475 => FindFrameExtra7bKind::BigMimicHallow,
        476 => FindFrameExtra7bKind::BigMimicJungle,
        477 => FindFrameExtra7bKind::Mothron,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection`,
/// `rotation` (the mimic states zero it, Mothron never writes it) and
/// the `localAI[0]`/`localAI[1]` pair (the later crawler/grub/dummy
/// batches). The wiring step copies them to/from `Npc`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter`.
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — the mimic state arms write it (the zero
    /// snaps); the crawler/grub/dummy batches ride it too.
    pub rotation: f32,
    /// `NPC.localAI[0]`/`[1]` — the crawler/grub slope latch and the
    /// Target Dummy's recoil clock + hit direction.
    pub local_ai: [f32; 2],
}

/// The per-tick inputs the arms read: `velocity`, `direction`, the
/// `ai` slots (the mimic `ai[0]` state / `ai[1]` windup, the Medusa/
/// Granite `ai[2]` gaze gates) plus the two resolved WORLD reads (the
/// header's skips): `slope` (the crawler/grub tile probe) and
/// `frame_x36` (the dummy's weapon-rack tile).
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone in
    /// the arms that gate on `direction == 1` / `== -1`; the mimic
    /// grounded copy and the grub copy are UNCONDITIONAL).
    pub direction: i32,
    pub ai: [f32; 4],
    /// The RESOLVED tile slope at the crawler/grub probe (the C#'s
    /// two-tile fall-through, `NPC.cs:72807-72815`): 0 none, 1/2 the
    /// diagonal overrides. Only the crawler/grub arms read it.
    pub slope: i32,
    /// `Framing.GetTileSafely(ai[0], ai[1]).frameX >= 36` — the Target
    /// Dummy's sign-flip probe (`NPC.cs:70850`). Only the dummy reads
    /// it.
    pub frame_x36: bool,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self {
            vx: 0.0,
            vy: 0.0,
            direction: 1,
            ai: [0.0; 4],
            slope: 0,
            frame_x36: false,
        }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-7b kinds — one
/// tick.
pub fn find_frame_extra7b(kind: FindFrameExtra7bKind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // `case 473..476` (`NPC.cs:71159-71329`): the Big Mimic state
        // machine. The grounded flip copies `direction`
        // UNCONDITIONALLY (`:71163-71166`, 0 included). States 0/7
        // snap row 0 + zero rotation/counter (`:71167-71172`); state
        // 1 is the ai[1] <6/<12/<18/<24/<30 windup ladder rows 1..6
        // (`:71173-71202`); state 8 ticks the counter with the >= 24
        // wrap and pins row min(6, max(3, 3 + (int)c/6))
        // (`:71203-71212`); states 2/6 are the hop cycle
        // (`:71213-71262` — the grounded row<7 lift to 12, the row<10
        // >8-step with the row==10 counter=12 re-kick, the
        // <6/<12/<18 bands rows 12/11/10 and the >= 23 reset; the air
        // pins row 13); state 3 decays one row per >6 tick while
        // row > 7 (`:71263-71275`); states 4/5 pin row 13 (state 4
        // zeroes rotation only when ai[2] == 1, `:71276-71284`);
        // state 4.1 is the 4-tick landing ladder rows 6/5/4/3/4/5
        // with the >= 23 reset (`:71285-71328`); every OTHER state
        // BREAKS with no write (`:71287-71290`).
        FindFrameExtra7bKind::BigMimicCorruption
        | FindFrameExtra7bKind::BigMimicCrimson
        | FindFrameExtra7bKind::BigMimicHallow
        | FindFrameExtra7bKind::BigMimicJungle => {
            // `:71163-71166`.
            if i.vy == 0.0 {
                s.sprite_dir = i.direction;
            }
            if i.ai[0] == 0.0 || i.ai[0] == 7.0 {
                // `:71167-71172`.
                s.rotation = 0.0;
                s.counter = 0.0;
                s.row = 0;
            } else if i.ai[0] == 1.0 {
                // `:71173-71202`.
                s.rotation = 0.0;
                s.counter = 0.0;
                s.row = if i.ai[1] < 6.0 {
                    1
                } else if i.ai[1] < 12.0 {
                    2
                } else if i.ai[1] < 18.0 {
                    3
                } else if i.ai[1] < 24.0 {
                    4
                } else if i.ai[1] < 30.0 {
                    5
                } else {
                    6
                };
            } else if i.ai[0] == 8.0 {
                // `:71203-71212`.
                s.rotation = 0.0;
                s.counter += 1.0;
                if s.counter >= 24.0 {
                    s.counter = 0.0;
                }
                s.row = (3 + (s.counter as i32) / 6).max(3).min(6) as u32;
            } else if i.ai[0] == 2.0 || i.ai[0] == 6.0 {
                // `:71213-71262`.
                s.rotation = 0.0;
                if i.vy == 0.0 {
                    // `:71216-71256`.
                    s.counter += 1.0;
                    if s.row < 7 {
                        s.row = 12;
                    }
                    if s.row < 10 {
                        if s.counter > 8.0 {
                            s.row += 1;
                            s.counter = 0.0;
                            if s.row == 10 {
                                s.counter = 12.0;
                            }
                        }
                    } else if s.counter < 6.0 {
                        s.row = 12;
                    } else if s.counter < 12.0 {
                        s.row = 11;
                    } else if s.counter < 18.0 {
                        s.row = 10;
                    } else {
                        s.row = 11;
                        if s.counter >= 23.0 {
                            s.counter = 0.0;
                        }
                    }
                } else {
                    // `:71257-71261`.
                    s.row = 13;
                    s.counter = 0.0;
                }
            } else if i.ai[0] == 3.0 {
                // `:71263-71275`.
                s.rotation = 0.0;
                s.counter += 1.0;
                if s.counter > 6.0 {
                    s.counter = 0.0;
                    if s.row > 7 {
                        s.row -= 1;
                    }
                }
            } else if i.ai[0] == 4.0 || i.ai[0] == 5.0 {
                // `:71276-71284`.
                if i.ai[0] == 4.0 && i.ai[2] == 1.0 {
                    s.rotation = 0.0;
                }
                s.row = 13;
                s.counter = 0.0;
            } else {
                // `:71285-71328` — every other state breaks.
                if i.ai[0] != 4.1 {
                    return;
                }
                s.rotation = 0.0;
                if s.row > 6 {
                    s.counter = 0.0;
                }
                s.counter += 1.0;
                if s.counter < 4.0 {
                    s.row = 6;
                    return;
                }
                if s.counter < 8.0 {
                    s.row = 5;
                    return;
                }
                if s.counter < 12.0 {
                    s.row = 4;
                    return;
                }
                if s.counter < 16.0 {
                    s.row = 3;
                    return;
                }
                if s.counter < 20.0 {
                    s.row = 4;
                    return;
                }
                s.row = 5;
                if s.counter >= 23.0 {
                    s.counter = 0.0;
                }
            }
        }
        // `case 477` (`NPC.cs:71330-71342`): Mothron. The counter
        // accumulates `velocity.Length() / 4` PLUS 1 (`:71331-71332`
        // — the sqrt of vx²+vy², the Vector2.Length), the >= 6
        // counter steps a row + zeroes (`:71333-71337`), wrapping at
        // the STRICT row > 5 back to 0 (`:71338-71341` — the full
        // 6-row sheet). No sprite/rotation write.
        FindFrameExtra7bKind::Mothron => {
            // `:71331-71332`.
            s.counter += (i.vx * i.vx + i.vy * i.vy).sqrt() / 4.0;
            s.counter += 1.0;
            // `:71333-71337`.
            if s.counter >= 6.0 {
                s.counter = 0.0;
                s.row += 1;
            }
            // `:71338-71341`.
            if s.row > 5 {
                s.row = 0;
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

    fn rows_over(kind: FindFrameExtra7bKind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra7b(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The frame-count table agrees with the port's verbatim
    /// `NPC_FRAME_COUNT` (`Main.cs:67082` initializer cross-check).
    #[test]
    fn f26s7b_frame_counts_match_the_port_table() {
        let table = crate::render::entities::NPC_FRAME_COUNT;
        for k in [
            FindFrameExtra7bKind::BigMimicCorruption,
            FindFrameExtra7bKind::BigMimicCrimson,
            FindFrameExtra7bKind::BigMimicHallow,
            FindFrameExtra7bKind::BigMimicJungle,
            FindFrameExtra7bKind::Mothron,
        ] {
            assert_eq!(
                k.frame_count(),
                table[k.vanilla_type() as usize],
                "{k:?} ({})",
                k.vanilla_type()
            );
        }
    }

    /// The mimic states 0/7 snap row 0; state 1 is the ai[1] windup
    /// ladder rows 1..6 (`NPC.cs:71167-71202`); the grounded flip is
    /// the UNCONDITIONAL direction copy (0 included), the air leaves
    /// the sprite alone.
    #[test]
    fn f26s7b_the_mimic_idle_and_windup_ladder() {
        for k in [
            FindFrameExtra7bKind::BigMimicCorruption,
            FindFrameExtra7bKind::BigMimicCrimson,
            FindFrameExtra7bKind::BigMimicHallow,
            FindFrameExtra7bKind::BigMimicJungle,
        ] {
            let mut s = FrameState { row: 9, counter: 5.0, sprite_dir: 1, rotation: 0.4, ..Default::default() };
            let mut idle = input(3.0, 0.0);
            idle.direction = 0;
            find_frame_extra7b(k, &mut s, &idle);
            assert_eq!((s.row, s.counter, s.rotation, s.sprite_dir), (0, 0.0, 0.0, 0), "{k:?} state 0");
            idle.ai[0] = 7.0;
            find_frame_extra7b(k, &mut s, &idle);
            assert_eq!(s.row, 0, "{k:?} state 7");
            let mut windup = input(0.0, 0.0);
            windup.ai[0] = 1.0;
            for (a1, row) in [(0.0, 1), (6.0, 2), (12.0, 3), (18.0, 4), (24.0, 5), (30.0, 6), (99.0, 6)] {
                windup.ai[1] = a1;
                find_frame_extra7b(k, &mut s, &windup);
                assert_eq!((s.row, s.counter), (row, 0.0), "{k:?} ai[1] {a1}");
            }
            // The air leaves the flip alone (the vy == 0 gate).
            let mut air = input(0.0, -2.0);
            air.direction = -1;
            find_frame_extra7b(k, &mut s, &air);
            assert_eq!(s.sprite_dir, 0, "{k:?} air keeps the sprite");
        }
    }

    /// The mimic state 8 strip (`NPC.cs:71203-71212`): the counter
    /// ticks with the >= 24 wrap, row = min(6, max(3, 3 + c/6)) —
    /// rows 3 (c 1..5), 4 (6..11), 5 (12..17), 6 (18..23), then the
    /// 24th tick resets to row 3 (replay-derived). State 3 decays one
    /// row per >6 tick while row > 7 and then HOLDS (`:71263-71275`).
    #[test]
    fn f26s7b_the_mimic_state8_strip_and_state3_decay() {
        let k = FindFrameExtra7bKind::BigMimicCorruption;
        let mut i = input(0.0, 0.0);
        i.ai[0] = 8.0;
        let rows = rows_over(k, i, 24);
        assert_eq!(
            rows,
            vec![3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6, 3]
        );
        // State 3 from row 13: one row per 7 ticks down to 7, hold.
        let mut s = FrameState { row: 13, ..Default::default() };
        let mut decay = input(0.0, 0.0);
        decay.ai[0] = 3.0;
        let mut out = Vec::new();
        for _ in 0..49 {
            find_frame_extra7b(k, &mut s, &decay);
            out.push(s.row);
        }
        assert_eq!(out[6], 12, "the first >6 tick decays 13 -> 12");
        assert_eq!(out[41], 7, "the 42nd tick reaches 7");
        assert_eq!(out[48], 7, "row 7 holds (the > 7 gate)");
        assert_eq!(s.counter, 0.0, "the 49th tick is a >6 reset");
    }

    /// The mimic states 4/5 pin row 13 (state 4 zeroes rotation only
    /// when ai[2] == 1; state 5 never touches it, `NPC.cs:71276-
    /// 71284`); state 4.1 is the 4-tick landing ladder with the >= 23
    /// reset (`:71291-71327`); every other state BREAKS with no write
    /// (`:71287-71290`).
    #[test]
    fn f26s7b_the_mimic_landing_ladder_and_dead_states() {
        let k = FindFrameExtra7bKind::BigMimicHallow;
        let mut s = FrameState { row: 2, counter: 9.0, rotation: 0.7, ..Default::default() };
        let mut pin = input(0.0, 0.0);
        pin.ai[0] = 5.0;
        find_frame_extra7b(k, &mut s, &pin);
        assert_eq!((s.row, s.counter, s.rotation), (13, 0.0, 0.7), "state 5 keeps rotation");
        pin.ai[0] = 4.0;
        s.rotation = 0.7;
        find_frame_extra7b(k, &mut s, &pin);
        assert_eq!(s.rotation, 0.7, "state 4 without ai[2]==1 keeps rotation");
        pin.ai[2] = 1.0;
        find_frame_extra7b(k, &mut s, &pin);
        assert_eq!(s.rotation, 0.0, "state 4 + ai[2]==1 zeroes it");
        // State 4.1 from row 13: the >6 row resets the counter, then
        // the 4-tick bands 6/5/4/3/4/5 with the 23-reset (replay).
        let mut land = input(0.0, 0.0);
        land.ai[0] = 4.1;
        assert_eq!(
            rows_over(k, land, 24),
            vec![6, 6, 6, 5, 5, 5, 5, 4, 4, 4, 4, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6]
        );
        // A dead state (9) writes NOTHING.
        let mut s = FrameState { row: 4, counter: 3.0, sprite_dir: -1, rotation: 0.5, ..Default::default() };
        let mut dead = input(2.0, -1.0);
        dead.ai[0] = 9.0;
        find_frame_extra7b(k, &mut s, &dead);
        assert_eq!((s.row, s.counter, s.sprite_dir, s.rotation), (4, 3.0, -1, 0.5));
    }

    /// The mimic state 2/6 hop cycle (`NPC.cs:71216-71256`): from
    /// idle row 0 the row<7 lift snaps to 12, the <6/<12/<18 bands
    /// hold rows 12/11/10, the else branch holds 11 with the >= 23
    /// counter reset — the 24-tick replay; the air pins row 13.
    #[test]
    fn f26s7b_the_mimic_hop_cycle_and_air_pin() {
        for k in [FindFrameExtra7bKind::BigMimicCrimson, FindFrameExtra7bKind::BigMimicJungle] {
            let mut hop = input(0.0, 0.0);
            hop.ai[0] = 2.0;
            assert_eq!(
                rows_over(k, hop, 24),
                vec![12, 12, 12, 12, 12, 11, 11, 11, 11, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 11, 11, 11, 11, 12],
                "{k:?}"
            );
            hop.ai[0] = 6.0;
            assert_eq!(rows_over(k, hop, 6), vec![12, 12, 12, 12, 12, 11], "{k:?} state 6 twin");
            let mut s = FrameState { row: 5, counter: 4.0, ..Default::default() };
            let mut air = input(1.0, -2.0);
            air.ai[0] = 2.0;
            find_frame_extra7b(k, &mut s, &air);
            assert_eq!((s.row, s.counter), (13, 0.0), "{k:?} air row 13");
            // The row<10 >8-step with the row==10 counter re-kick
            // (`:71226-71233`): row 9 + counter 8 steps to 10 and
            // re-kicks the counter to 12.
            let mut s = FrameState { row: 9, counter: 8.0, ..Default::default() };
            find_frame_extra7b(k, &mut s, &hop);
            assert_eq!((s.row, s.counter), (10, 12.0), "{k:?} the row-10 re-kick");
        }
    }

    /// Mothron's length-fed stepper (`NPC.cs:71331-71341`): vx 3 →
    /// +1.75/tick steps every 4th tick over the 6-row sheet, the 24th
    /// tick wraps row 6 → 0 (replay-derived); vy-only velocity feeds
    /// the same sqrt (vx 0, vy 4 → +2/tick, a 3-tick step).
    #[test]
    fn f26s7b_the_mothron_length_stepper() {
        let k = FindFrameExtra7bKind::Mothron;
        assert_eq!(
            rows_over(k, input(3.0, 0.0), 24),
            vec![0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0]
        );
        assert_eq!(rows_over(k, input(0.0, 4.0), 6), vec![0, 0, 1, 1, 1, 2]);
        // The stationary moth still ticks (+1/tick): a 6-tick step.
        assert_eq!(rows_over(k, input(0.0, 0.0), 6), vec![0, 0, 0, 0, 0, 1]);
    }
}
