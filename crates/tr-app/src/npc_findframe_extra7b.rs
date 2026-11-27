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
    /// `NPCID.MothronSpawn` = 479 (`NPCID.cs:11631`) — `case 479`
    /// (`NPC.cs:71343-71368`). 478 MothronEgg has NO case label.
    MothronSpawn,
    /// `NPCID.Medusa` = 480 (`NPCID.cs:11633`) — `case 480`
    /// (`NPC.cs:75103-75167`).
    Medusa,
    /// `NPCID.GreekSkeleton` = 481 (`NPCID.cs:11635`) — `case 481`
    /// (`NPC.cs:71454-71515`).
    GreekSkeleton,
    /// `NPCID.GraniteGolem` = 482 (`NPCID.cs:11637`) — `case 482`
    /// (`NPC.cs:75202-75258`).
    GraniteGolem,
    /// `NPCID.GraniteFlyer` = 483 (`NPCID.cs:11639`) — `case 483`
    /// (`NPC.cs:75168-75201`).
    GraniteFlyer,
    /// `NPCID.EnchantedNightcrawler` = 484 (`NPCID.cs:11641`) — the
    /// 357/448/606 crawler group (`NPC.cs:72776`).
    EnchantedNightcrawler,
    /// `NPCID.Grubby` = 485 (`NPCID.cs:11643`) — the 485..487 grub
    /// group (`NPC.cs:72829`).
    Grubby,
    /// `NPCID.Sluggy` = 486 (`NPCID.cs:11645`) — the same grub group
    /// (`NPC.cs:72830`).
    Sluggy,
    /// `NPCID.Buggy` = 487 (`NPCID.cs:11647`) — the same grub group
    /// (`NPC.cs:72831`).
    Buggy,
    /// `NPCID.TargetDummy` = 488 (`NPCID.cs:11649`) — `case 488`
    /// (`NPC.cs:70847-70878`).
    TargetDummy,
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
            Self::MothronSpawn => 479,
            Self::Medusa => 480,
            Self::GreekSkeleton => 481,
            Self::GraniteGolem => 482,
            Self::GraniteFlyer => 483,
            Self::EnchantedNightcrawler => 484,
            Self::Grubby => 485,
            Self::Sluggy => 486,
            Self::Buggy => 487,
            Self::TargetDummy => 488,
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
            // Main.cs:67131 (entry 479): 3.
            Self::MothronSpawn => 3,
            // Main.cs:67132 (entries 480..483): 25, 19, 20, 22.
            Self::Medusa => 25,
            Self::GreekSkeleton => 19,
            Self::GraniteGolem => 20,
            Self::GraniteFlyer => 22,
            // Main.cs:67132 (entries 484..488): 2, 4, 4, 4, 11.
            Self::EnchantedNightcrawler => 2,
            Self::Grubby => 4,
            Self::Sluggy => 4,
            Self::Buggy => 4,
            Self::TargetDummy => 11,
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
        479 => FindFrameExtra7bKind::MothronSpawn,
        480 => FindFrameExtra7bKind::Medusa,
        481 => FindFrameExtra7bKind::GreekSkeleton,
        482 => FindFrameExtra7bKind::GraniteGolem,
        483 => FindFrameExtra7bKind::GraniteFlyer,
        484 => FindFrameExtra7bKind::EnchantedNightcrawler,
        485 => FindFrameExtra7bKind::Grubby,
        486 => FindFrameExtra7bKind::Sluggy,
        487 => FindFrameExtra7bKind::Buggy,
        488 => FindFrameExtra7bKind::TargetDummy,
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
        // `case 479` (`NPC.cs:71343-71368`): the Mothron Spawn's
        // 4-tick flutter. The counter ticks FIRST (`:71345`), then
        // the < 4 / < 8 / < 12 bands pin rows 0/1/2 (`:71347-71358`),
        // the < 15 band pin row 1 AGAIN (`:71359-71362` — the fourth
        // band's upper edge is 4*4-1 = 15, NOT 16), and the >= 15
        // else zeroes the counter WITHOUT a row write (`:71363-
        // 71366` — the reset tick HOLDS row 1, the Moth 251
        // reset-tick row-hold quirk). No sprite/rotation write.
        FindFrameExtra7bKind::MothronSpawn => {
            // `:71345`.
            s.counter += 1.0;
            // `:71347-71366`.
            if s.counter < 4.0 {
                s.row = 0;
            } else if s.counter < 8.0 {
                s.row = 1;
            } else if s.counter < 12.0 {
                s.row = 2;
            } else if s.counter < 15.0 {
                s.row = 1;
            } else {
                s.counter = 0.0;
            }
        }
        // `case 480` (`NPC.cs:75103-75167`): Medusa. The grounded ±1
        // flip (`:75106-75113`); the `ai[2] < 0` gaze strip
        // (`:75114-75130`) ticks the counter with the > 3 step and
        // BOTH clamps snap row 21 (>= the 25-row count OR < 21 —
        // rows 21..24 are the gaze strip); the idle (`:75131-75143`)
        // ticks with the > 4 step over rows 0..7 with the STRICT
        // > 7 wrap; the walk (`:75144-75160`) accumulates 1 + |vx|
        // (NO 0.5), the > 6 counter steps a row, and BOTH clamps
        // snap row 10 (>= count-4 = 21 OR < 10 — rows 10..20 are the
        // walk strip); the air zeroes the counter and pins row 8
        // (`:75162-75166`). No rotation write.
        FindFrameExtra7bKind::Medusa => {
            if i.vy == 0.0 {
                // `:75106-75113`.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                if i.ai[2] < 0.0 {
                    // `:75114-75130`.
                    s.counter += 1.0;
                    if s.counter > 3.0 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                    if s.row >= kind.frame_count() {
                        s.row = 21;
                    } else if s.row < 21 {
                        s.row = 21;
                    }
                } else if i.vx == 0.0 {
                    // `:75131-75143`.
                    s.counter += 1.0;
                    if s.counter > 4.0 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                    if s.row > 7 {
                        s.row = 0;
                    }
                } else {
                    // `:75144-75160`.
                    s.counter += 1.0 + i.vx.abs();
                    if s.counter > 6.0 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                    if s.row >= kind.frame_count() - 4 {
                        s.row = 10;
                    } else if s.row < 10 {
                        s.row = 10;
                    }
                }
            } else {
                // `:75162-75166`.
                s.counter = 0.0;
                s.row = 8;
            }
        }
        // `case 481` (`NPC.cs:71454-71515`): the Greek Skeleton. The
        // grounded ±1 flip (`:71457-71464`); the `ai[2] > 0` attack
        // pose (`:71465-71487`) copies `direction` UNCONDITIONALLY
        // and pins row 15 + num28 off the ai[1] bands: < 22 → -14
        // (row 1), < 28 → 3 (row 18), < 34 → 2 (row 17), < 40 → 1
        // (row 16), else 0 (row 15) — the counter ZEROED (no break;
        // the else-if chain falls through to the end of the grounded
        // arm). The idle (`:71488-71491`) pins row 1 and does NOT
        // touch the counter; the walk (`:71492-71508`) lifts any
        // row < 2 to 2, accumulates `|vx| * 2` — NO +1 — the > 6
        // counter steps a row, wrapping at row >= 15 back to 2
        // (rows 15..18 are the attack poses). The air zeroes the
        // counter and pins row 0 (`:71510-71514`). No rotation write.
        FindFrameExtra7bKind::GreekSkeleton => {
            if i.vy == 0.0 {
                // `:71457-71464`.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                if i.ai[2] > 0.0 {
                    // `:71465-71487`.
                    s.sprite_dir = i.direction;
                    let num28 = if i.ai[1] < 22.0 {
                        -14
                    } else if i.ai[1] < 28.0 {
                        3
                    } else if i.ai[1] < 34.0 {
                        2
                    } else if i.ai[1] < 40.0 {
                        1
                    } else {
                        0
                    };
                    s.row = (15 + num28) as u32;
                    s.counter = 0.0;
                } else if i.vx == 0.0 {
                    // `:71488-71491` — the counter is NOT touched.
                    s.row = 1;
                } else {
                    // `:71492-71508`.
                    if s.row < 2 {
                        s.row = 2;
                    }
                    s.counter += i.vx.abs() * 2.0;
                    if s.counter > 6.0 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                    if s.row >= 15 {
                        s.row = 2;
                    }
                }
            } else {
                // `:71510-71514`.
                s.counter = 0.0;
                s.row = 0;
            }
        }
        // `case 482` (`NPC.cs:75202-75258`): the Granite Golem. The
        // grounded ±1 flip (`:75205-75212`); the `ai[2] < 0` gaze
        // strip (`:75213-75229`) ticks with the > 3 step and BOTH
        // clamps snap row 11 (>= the 20-row count OR < 11 — rows
        // 11..19 are the gaze strip); the idle (`:75230-75234`)
        // TICKS the counter but snaps row 0 EVERY tick (no wrap —
        // the counter free-runs); the walk (`:75235-75251`)
        // accumulates 0.2 + |vx| (the 0.2 drift, NOT the +1), the
        // > 8 counter steps a row, and BOTH clamps snap row 2
        // (>= count-10 = 10 OR < 2 — rows 2..9 are the walk strip);
        // the air zeroes the counter and pins row 1 (`:75253-75257`).
        // No rotation write.
        FindFrameExtra7bKind::GraniteGolem => {
            if i.vy == 0.0 {
                // `:75205-75212`.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                if i.ai[2] < 0.0 {
                    // `:75213-75229`.
                    s.counter += 1.0;
                    if s.counter > 3.0 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                    if s.row >= kind.frame_count() {
                        s.row = 11;
                    } else if s.row < 11 {
                        s.row = 11;
                    }
                } else if i.vx == 0.0 {
                    // `:75230-75234` — the counter ticks, row 0 snaps.
                    s.counter += 1.0;
                    s.row = 0;
                } else {
                    // `:75235-75251`.
                    s.counter += 0.2 + i.vx.abs();
                    if s.counter > 8.0 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                    if s.row >= kind.frame_count() - 10 {
                        s.row = 2;
                    } else if s.row < 2 {
                        s.row = 2;
                    }
                }
            } else {
                // `:75253-75257`.
                s.counter = 0.0;
                s.row = 1;
            }
        }
        // `case 483` (`NPC.cs:75168-75201`): the Granite Flyer's two
        // strips. The `ai[0] == -1` charged state (`:75169-75186`)
        // ticks with the > 4 step over rows 13..21 with BOTH clamps
        // (> 21 snaps 21, < 13 snaps 13) and ACCUMULATES the tilt
        // `rotation += vx * 0.2`; the normal state (`:75187-75200`)
        // ticks with the > 4 step over rows 0..11 with the STRICT
        // > 11 wrap to 0 and SNAPS `rotation = vx * 0.1`. No sprite
        // write.
        FindFrameExtra7bKind::GraniteFlyer => {
            if i.ai[0] == -1.0 {
                // `:75169-75186`.
                s.counter += 1.0;
                if s.counter > 4.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
                if s.row > 21 {
                    s.row = 21;
                } else if s.row < 13 {
                    s.row = 13;
                }
                s.rotation += i.vx * 0.2;
            } else {
                // `:75187-75200`.
                s.counter += 1.0;
                if s.counter > 4.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
                if s.row > 11 {
                    s.row = 0;
                }
                s.rotation = i.vx * 0.1;
            }
        }
        // `case 484` in the 357/448/606 crawler group (`NPC.cs:
        // 72774-72828` — d48's 357 Worm / d40's 448 GoldWorm / the
        // 606 Maggot are the SAME machine, keep-one at the merge):
        // `localAI[0] = -2` FIRST (`:72779`); the grounded branch
        // zeroes the rotation (`:72782`), the idle pins row 1 +
        // zeroes the counter (`:72783-72787`), the walk ticks the
        // counter with the > 12 step and the STRICT row > 1 wrap to
        // 0 (`:72788-72800` — the 2-row sheet's rows 0/1); the air
        // ACCUMULATES the tilt `rotation += direction * 0.1` and
        // pins row 1 WITHOUT touching the counter (`:72802-72806`);
        // the resolved slope probe (`:72807-72826`, the header's
        // skip) snaps rotation ±0.785 and re-latches `localAI[0]`
        // to 0. No sprite write.
        FindFrameExtra7bKind::EnchantedNightcrawler => {
            // `:72779`.
            s.local_ai[0] = -2.0;
            if i.vy == 0.0 {
                // `:72782`.
                s.rotation = 0.0;
                if i.vx == 0.0 {
                    // `:72783-72787`.
                    s.row = 1;
                    s.counter = 0.0;
                } else {
                    // `:72788-72800`.
                    s.counter += 1.0;
                    if s.counter > 12.0 {
                        s.counter = 0.0;
                        s.row += 1;
                        if s.row > 1 {
                            s.row = 0;
                        }
                    }
                }
            } else {
                // `:72802-72806`.
                s.rotation += i.direction as f32 * 0.1;
                s.row = 1;
            }
            // `:72807-72826` — the resolved slope probe.
            if i.slope == 1 {
                s.rotation = 0.785;
                s.local_ai[0] = 0.0;
            } else if i.slope == 2 {
                s.rotation = -0.785;
                s.local_ai[0] = 0.0;
            }
        }
        // `case 485..487` (`NPC.cs:72829-72883`): the Grubby/Sluggy/
        // Buggy trio — the crawler shape PLUS the UNCONDITIONAL
        // direction copy (`:72833`) and the |vx*5| accumulator
        // (`:72845` — Math.Abs(vx * 5), NOT |vx| * 5 after the abs,
        // identical for reals but cited verbatim) with the > 10
        // step and the STRICT row > 3 wrap to 0 (`:72846-72854` —
        // the 4-row sheet); the air tilt/pin and the slope probe
        // are the crawler group's (`:72857-72881`).
        FindFrameExtra7bKind::Grubby
        | FindFrameExtra7bKind::Sluggy
        | FindFrameExtra7bKind::Buggy => {
            // `:72833-72834`.
            s.sprite_dir = i.direction;
            s.local_ai[0] = -2.0;
            if i.vy == 0.0 {
                // `:72837`.
                s.rotation = 0.0;
                if i.vx == 0.0 {
                    // `:72838-72842`.
                    s.row = 1;
                    s.counter = 0.0;
                } else {
                    // `:72843-72855`.
                    s.counter += (i.vx * 5.0).abs();
                    if s.counter > 10.0 {
                        s.counter = 0.0;
                        s.row += 1;
                        if s.row > 3 {
                            s.row = 0;
                        }
                    }
                }
            } else {
                // `:72857-72861`.
                s.rotation += i.direction as f32 * 0.1;
                s.row = 1;
            }
            // `:72862-72881` — the resolved slope probe.
            if i.slope == 1 {
                s.rotation = 0.785;
                s.local_ai[0] = 0.0;
            } else if i.slope == 2 {
                s.rotation = -0.785;
                s.local_ai[0] = 0.0;
            }
        }
        // `case 488` (`NPC.cs:70847-70878`): the Target Dummy's
        // recoil clock. `num134 = (int)localAI[1]` (the hit
        // direction, NPCHit `:82694`), NEGATED when the weapon-rack
        // tile probe reads frameX >= 36 (`:70849-70853`, the
        // header's resolved [`FrameInput::frame_x36`] skip);
        // `localAI[0]` (the recoil magnitude, NPCHit `:82685-82693`
        // clamps it 20..120) clamps to 24 FIRST then decrements
        // toward 0 (`:70854-70865`); the row is the CEIL of
        // `localAI[0] / num135` where num135 = 4 for the negated
        // direction else 6 (`:70866-70871`), PLUS 5 when nonzero
        // and the direction is +1 (`:70872-70875` — rows 6..10 are
        // the right-facing recoil, rows 1..6 the left). No counter/
        // sprite/rotation write.
        FindFrameExtra7bKind::TargetDummy => {
            // `:70849-70853`.
            let mut num134 = s.local_ai[1] as i32;
            if i.frame_x36 {
                num134 *= -1;
            }
            // `:70854-70865`.
            if s.local_ai[0] > 24.0 {
                s.local_ai[0] = 24.0;
            }
            if s.local_ai[0] > 0.0 {
                s.local_ai[0] -= 1.0;
            }
            if s.local_ai[0] < 0.0 {
                s.local_ai[0] = 0.0;
            }
            // `:70866-70871`.
            let num135 = if num134 == -1 { 4 } else { 6 };
            let mut num136 = (s.local_ai[0] as i32) / num135;
            if s.local_ai[0] % num135 as f32 != 0.0 {
                num136 += 1;
            }
            // `:70872-70875`.
            if num136 != 0 && num134 == 1 {
                num136 += 5;
            }
            s.row = num136 as u32;
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
            FindFrameExtra7bKind::MothronSpawn,
            FindFrameExtra7bKind::Medusa,
            FindFrameExtra7bKind::GreekSkeleton,
            FindFrameExtra7bKind::GraniteGolem,
            FindFrameExtra7bKind::GraniteFlyer,
            FindFrameExtra7bKind::EnchantedNightcrawler,
            FindFrameExtra7bKind::Grubby,
            FindFrameExtra7bKind::Sluggy,
            FindFrameExtra7bKind::Buggy,
            FindFrameExtra7bKind::TargetDummy,
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
            // The air leaves the flip alone (the vy == 0 gate): the last
            // grounded windup tick copied its default direction 1
            // (`NPC.cs:71163-71166`), so the airborne tick keeps 1.
            let mut air = input(0.0, -2.0);
            air.direction = -1;
            find_frame_extra7b(k, &mut s, &air);
            assert_eq!(s.sprite_dir, 1, "{k:?} air keeps the sprite");
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

    /// The Mothron Spawn's 4-tick flutter (`NPC.cs:71345-71366`):
    /// the bands 0/1/2/1 over counters 1..14 and the >= 15 reset
    /// tick HOLDS row 1 (the fourth band's edge is 4*4-1 = 15, NOT
    /// 16 — the reset tick writes no row; replay-derived 15 ticks).
    #[test]
    fn f26s7b_the_mothron_spawn_flutter_holds_the_reset_tick() {
        let k = FindFrameExtra7bKind::MothronSpawn;
        assert_eq!(
            rows_over(k, input(2.0, -1.0), 15),
            vec![0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1]
        );
        let mut s = FrameState { row: 2, counter: 14.0, ..Default::default() };
        find_frame_extra7b(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (2, 0.0), "the reset tick holds the row");
    }

    /// Medusa's three grounded strips (`NPC.cs:75114-75160`): the
    /// ai[2] < 0 gaze strip pins rows 21..24 (both clamps snap 21,
    /// the > 3 step wraps 25 → 21); the idle walks rows 0..7 with
    /// the > 4 step and the strict > 7 wrap; the walk lifts to 10
    /// and strides rows 10..20 at 1+|vx| with the > 6 step (both
    /// clamps snap 10); the air pins row 8.
    #[test]
    fn f26s7b_the_medusa_gaze_idle_walk_strips() {
        let k = FindFrameExtra7bKind::Medusa;
        // The gaze strip from row 0: the < 21 clamp snaps 21, then
        // the > 3 step walks 22/23/24 and wraps 25 → 21 (replay).
        let mut gaze = input(3.0, 0.0);
        gaze.ai[2] = -1.0;
        assert_eq!(rows_over(k, gaze, 17), vec![21, 21, 21, 22, 22, 22, 22, 23, 23, 23, 23, 24, 24, 24, 24, 21, 21]);
        // The idle: the > 4 step over rows 0..7, the 40th tick wraps.
        assert_eq!(
            rows_over(k, input(0.0, 0.0), 41),
            vec![0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 0, 0]
        );
        // The walk at vx 3: +4/tick, the > 6 step fires every 2nd
        // tick; the < 10 clamp lifts to 10 first (replay).
        assert_eq!(rows_over(k, input(3.0, 0.0), 25), vec![10, 11, 11, 12, 12, 13, 13, 14, 14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 10, 10, 11, 11]);
        let mut s = FrameState { row: 15, counter: 2.0, sprite_dir: 1, ..Default::default() };
        let mut air = input(1.0, 2.0);
        air.direction = -1;
        find_frame_extra7b(k, &mut s, &air);
        assert_eq!((s.row, s.counter, s.sprite_dir), (8, 0.0, 1), "air row 8, no flip");
    }

    /// The Greek Skeleton's attack ladder (`NPC.cs:71465-71487`):
    /// ai[2] > 0 pins row 15 + {-14, 3, 2, 1, 0} off the ai[1]
    /// < 22 / < 28 / < 34 / < 40 bands with the UNCONDITIONAL
    /// direction copy; the idle pins row 1 WITHOUT touching the
    /// counter; the walk lifts to 2 at |vx|*2 (NO +1) and wraps at
    /// 15 → 2; the air pins row 0.
    #[test]
    fn f26s7b_the_greek_skeleton_ladder_and_idle_counter_hold() {
        let k = FindFrameExtra7bKind::GreekSkeleton;
        let mut s = FrameState { row: 5, counter: 4.0, sprite_dir: 1, ..Default::default() };
        let mut atk = input(3.0, 0.0);
        atk.ai[2] = 1.0;
        atk.direction = 0;
        for (a1, row) in [(0.0, 1), (22.0, 18), (28.0, 17), (34.0, 16), (40.0, 15), (99.0, 15)] {
            atk.ai[1] = a1;
            find_frame_extra7b(k, &mut s, &atk);
            assert_eq!((s.row, s.counter, s.sprite_dir), (row, 0.0, 0), "ai[1] {a1}");
        }
        // The idle keeps the counter (the C# writes only frame.Y).
        let mut s = FrameState { row: 9, counter: 5.0, ..Default::default() };
        find_frame_extra7b(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (1, 5.0), "idle row 1, counter held");
        // vx 3 → +6/tick: the first step fires on tick 2 (6+6 = 12
        // > 6; the single +6 never exceeds), then every 2nd tick.
        assert_eq!(rows_over(k, input(3.0, 0.0), 3), vec![2, 3, 3]);
        // vx 4 → +8/tick steps every tick: rows 3..14 then 15 → 2.
        assert_eq!(
            rows_over(k, input(4.0, 0.0), 13),
            vec![3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 2]
        );
        let mut s = FrameState { row: 12, counter: 3.0, ..Default::default() };
        find_frame_extra7b(k, &mut s, &input(0.0, -1.0));
        assert_eq!((s.row, s.counter), (0, 0.0), "air row 0");
    }

    /// The Granite Golem (`NPC.cs:75213-75257`): the gaze strip
    /// rows 11..19 (both clamps snap 11, the > 3 step wraps 20 →
    /// 11); the idle TICKS the counter but snaps row 0 every tick
    /// (the free-running counter); the walk accumulates 0.2+|vx|
    /// with the > 8 step over rows 2..9 (both clamps snap 2); the
    /// air pins row 1.
    #[test]
    fn f26s7b_the_granite_golem_strips_and_free_running_idle() {
        let k = FindFrameExtra7bKind::GraniteGolem;
        let mut gaze = input(2.0, 0.0);
        gaze.ai[2] = -0.5;
        assert_eq!(rows_over(k, gaze, 13), vec![11, 11, 11, 12, 12, 12, 12, 13, 13, 13, 13, 14, 14]);
        // The idle: row 0 every tick, the counter free-runs.
        let mut s = FrameState::default();
        for _ in 0..30 {
            find_frame_extra7b(k, &mut s, &input(0.0, 0.0));
            assert_eq!(s.row, 0);
        }
        assert_eq!(s.counter, 30.0, "the idle counter never resets");
        // vx 3 → +3.2/tick: the > 8 step fires on ticks 3/6/9...
        // (9.6 > 8, zeroed); the < 2 clamp lifts to 2 (replay).
        assert_eq!(rows_over(k, input(3.0, 0.0), 10), vec![2, 2, 3, 3, 3, 4, 4, 4, 5, 5]);
        // vx 9 → +9.2/tick steps every tick: the < 2 clamp lifts to
        // 2 first, rows 3..9 then the >= count-10 = 10 clamp wraps.
        assert_eq!(rows_over(k, input(9.0, 0.0), 9), vec![2, 3, 4, 5, 6, 7, 8, 9, 2]);
        let mut s = FrameState { row: 7, counter: 6.0, ..Default::default() };
        find_frame_extra7b(k, &mut s, &input(0.0, 1.0));
        assert_eq!((s.row, s.counter), (1, 0.0), "air row 1");
    }

    /// The Granite Flyer's two strips (`NPC.cs:75169-75200`): the
    /// normal > 4 step over rows 0..11 with the strict > 11 wrap
    /// and the vx*0.1 tilt SNAP; the ai[0] == -1 charged strip
    /// clamps rows 13..21 and ACCUMULATES the vx*0.2 tilt.
    #[test]
    fn f26s7b_the_granite_flyer_two_strips_and_tilt_modes() {
        let k = FindFrameExtra7bKind::GraniteFlyer;
        // Normal: the > 4 step fires every 5th tick, 60 ticks wrap
        // 12 → 0 on the 60th (replay).
        let rows = rows_over(k, input(2.0, 0.0), 61);
        assert_eq!(rows[4], 1, "the 5th tick steps");
        assert_eq!(rows[54], 11, "the 55th tick reaches 11");
        assert_eq!(rows[59], 0, "the 60th tick wraps 12 -> 0");
        let mut s = FrameState { row: 3, rotation: 9.0, ..Default::default() };
        find_frame_extra7b(k, &mut s, &input(2.0, 0.0));
        assert_eq!(s.rotation, 0.2, "the normal tilt SNAPS vx*0.1");
        // Charged: the < 13 clamp lifts to 13, the > 21 clamp holds
        // 21; the tilt ACCUMULATES vx*0.2.
        let mut charged = input(1.0, 0.0);
        charged.ai[0] = -1.0;
        let mut s = FrameState { row: 2, rotation: 0.5, ..Default::default() };
        find_frame_extra7b(k, &mut s, &charged);
        assert_eq!((s.row, s.rotation), (13, 0.7), "clamp to 13, tilt += 0.2");
        let mut s = FrameState { row: 21, counter: 4.0, rotation: 0.0, ..Default::default() };
        find_frame_extra7b(k, &mut s, &charged);
        assert_eq!(s.row, 21, "the > 21 clamp holds 21");
        assert_eq!(s.counter, 0.0, "the step zeroed the counter");
    }

    /// The Enchanted Nightcrawler (`NPC.cs:72779-72826`): the
    /// localAI[0] = -2 latch every tick, the grounded idle row 1,
    /// the > 12 walk step alternating rows 0/1 on the 2-row sheet
    /// (the 13th tick steps, the 26th wraps), the air tilt
    /// ACCUMULATE + row 1 with the counter HELD, and the resolved
    /// slope probe's ±0.785 snap + localAI[0] re-latch.
    #[test]
    fn f26s7b_the_nightcrawler_two_row_walk_and_slope_latch() {
        let k = FindFrameExtra7bKind::EnchantedNightcrawler;
        assert_eq!(
            rows_over(k, input(2.0, 0.0), 27),
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0]
        );
        let mut s = FrameState { row: 0, counter: 5.0, rotation: 0.3, local_ai: [3.0, 0.0], ..Default::default() };
        find_frame_extra7b(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter, s.rotation, s.local_ai[0]), (1, 0.0, 0.0, -2.0), "idle + latch");
        // The air: rotation += direction * 0.1, row 1, counter HELD.
        let mut air = input(0.0, -2.0);
        air.direction = -1;
        find_frame_extra7b(k, &mut s, &air);
        assert_eq!(s.row, 1);
        assert_eq!(s.counter, 0.0, "the air never touches the counter");
        assert!((s.rotation - -0.1).abs() < 1e-6, "the air tilt accumulates");
        // The slope probe snaps ±0.785 and re-latches localAI[0].
        let mut slope = input(2.0, 0.0);
        slope.slope = 1;
        find_frame_extra7b(k, &mut s, &slope);
        assert_eq!((s.rotation, s.local_ai[0]), (0.785, 0.0), "slope 1");
        slope.slope = 2;
        find_frame_extra7b(k, &mut s, &slope);
        assert_eq!((s.rotation, s.local_ai[0]), (-0.785, 0.0), "slope 2");
    }

    /// The Grubby/Sluggy/Buggy trio (`NPC.cs:72833-72881`): the
    /// UNCONDITIONAL direction copy (0 included), the |vx*5|
    /// accumulator with the > 10 step over the 4-row sheet (vx 3 →
    /// +15/tick steps EVERY tick, rows 1/2/3/0 cycling; vx 1 →
    /// +5/tick steps every 3rd), the strict > 3 wrap, the air
    /// row-1 pin + tilt accumulate, and the slope probe.
    #[test]
    fn f26s7b_the_grub_trio_four_row_cycle() {
        for k in [FindFrameExtra7bKind::Grubby, FindFrameExtra7bKind::Sluggy, FindFrameExtra7bKind::Buggy] {
            assert_eq!(rows_over(k, input(3.0, 0.0), 9), vec![1, 2, 3, 0, 1, 2, 3, 0, 1], "{k:?} vx 3");
            let mut slow = input(1.0, 0.0);
            slow.direction = -1;
            assert_eq!(rows_over(k, slow, 9), vec![0, 0, 1, 1, 1, 2, 2, 2, 3], "{k:?} vx 1");
            let mut s = FrameState { row: 2, counter: 7.0, sprite_dir: 1, rotation: 0.2, local_ai: [3.0, 0.0] };
            let mut air = input(0.0, 2.0);
            air.direction = -1;
            find_frame_extra7b(k, &mut s, &air);
            assert_eq!((s.row, s.counter, s.sprite_dir, s.local_ai[0]), (1, 7.0, -1, -2.0), "{k:?} air");
            assert!((s.rotation - 0.1).abs() < 1e-6, "{k:?} tilt accumulate");
            let mut slope = input(3.0, 0.0);
            slope.direction = 0;
            slope.slope = 2;
            find_frame_extra7b(k, &mut s, &slope);
            assert_eq!((s.sprite_dir, s.rotation, s.local_ai[0]), (0, -0.785, 0.0), "{k:?} dir-0 copy + slope");
        }
    }

    /// The Target Dummy's recoil clock (`NPC.cs:70849-70876`): the
    /// localAI[0] 24-clamp + decrement, the ceil(localAI[0]/num135)
    /// row with num135 = 4 (negated hit dir) / 6, the +5 bank for
    /// the +1 direction (rows 6..10 right, 1..6 left), and the
    /// frameX >= 36 probe's sign flip. NPCHit seeds localAI[0]
    /// 20..120 (`:82685-82693`).
    #[test]
    fn f26s7b_the_target_dummy_recoil_banks() {
        let k = FindFrameExtra7bKind::TargetDummy;
        // Right hit: 24 → 23, ceil(23/6) = 4, +5 = row 9.
        let mut s = FrameState { local_ai: [24.0, 1.0], ..Default::default() };
        find_frame_extra7b(k, &mut s, &FrameInput::default());
        assert_eq!((s.row, s.local_ai[0]), (9, 23.0));
        // Left hit: ceil(23/4) = 6, NO +5.
        let mut s = FrameState { local_ai: [24.0, -1.0], ..Default::default() };
        find_frame_extra7b(k, &mut s, &FrameInput::default());
        assert_eq!(s.row, 6);
        // The frameX >= 36 probe flips the sign: right → the 4-row
        // bank.
        let mut probe = FrameInput::default();
        probe.frame_x36 = true;
        let mut s = FrameState { local_ai: [24.0, 1.0], ..Default::default() };
        find_frame_extra7b(k, &mut s, &probe);
        assert_eq!(s.row, 6, "the probe flips +1 to the 4-row bank");
        // The 24-clamp: a 120-magnitude hit reads the same as 24.
        let mut s = FrameState { local_ai: [120.0, 1.0], ..Default::default() };
        find_frame_extra7b(k, &mut s, &FrameInput::default());
        assert_eq!((s.row, s.local_ai[0]), (9, 23.0));
        // The full right decay: rows 9 (5 ticks) / 8 (6) / 7 (6) /
        // 6 (6) then 0 — the ceil bands over the 6-row divisor
        // (replay-derived 26 ticks).
        let mut s = FrameState { local_ai: [24.0, 1.0], ..Default::default() };
        let mut out = Vec::new();
        for _ in 0..26 {
            find_frame_extra7b(k, &mut s, &FrameInput::default());
            out.push(s.row);
        }
        assert_eq!(out, vec![9, 9, 9, 9, 9, 8, 8, 8, 8, 8, 8, 7, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6, 0, 0, 0]);
        assert_eq!(s.local_ai[0], 0.0, "the clock floors at 0");
        // The left decay: the 4-row divisor's bands 6/5/4/3/2/1/0.
        let mut s = FrameState { local_ai: [24.0, -1.0], ..Default::default() };
        let mut out = Vec::new();
        for _ in 0..26 {
            find_frame_extra7b(k, &mut s, &FrameInput::default());
            out.push(s.row);
        }
        assert_eq!(out, vec![6, 6, 6, 5, 5, 5, 5, 4, 4, 4, 4, 3, 3, 3, 3, 2, 2, 2, 2, 1, 1, 1, 1, 0, 0, 0]);
    }

    /// The 15-kind strip sweep: every kind ticks 72 input combos
    /// (ai[0] 0/2/8/4.1 × vx 0/1/3 × vy 0/-2/2 × ai[2] -1/2, ai[1] =
    /// 15, direction alternating) for 10 ticks each — every row
    /// stays inside its own sheet (the slice-6 18-combo sweep
    /// widened for the mimic state axis + the gaze gates,
    /// replay-derived bounds).
    #[test]
    fn f26s7b_the_15_kind_strip_sweep_stays_in_sheet() {
        let kinds = [
            FindFrameExtra7bKind::BigMimicCorruption,
            FindFrameExtra7bKind::BigMimicCrimson,
            FindFrameExtra7bKind::BigMimicHallow,
            FindFrameExtra7bKind::BigMimicJungle,
            FindFrameExtra7bKind::Mothron,
            FindFrameExtra7bKind::MothronSpawn,
            FindFrameExtra7bKind::Medusa,
            FindFrameExtra7bKind::GreekSkeleton,
            FindFrameExtra7bKind::GraniteGolem,
            FindFrameExtra7bKind::GraniteFlyer,
            FindFrameExtra7bKind::EnchantedNightcrawler,
            FindFrameExtra7bKind::Grubby,
            FindFrameExtra7bKind::Sluggy,
            FindFrameExtra7bKind::Buggy,
            FindFrameExtra7bKind::TargetDummy,
        ];
        for k in kinds {
            let mut s = FrameState::default();
            let mut combo = 0;
            for ai0 in [0.0f32, 2.0, 8.0, 4.1] {
                for vx in [0.0f32, 1.0, 3.0] {
                    for vy in [0.0f32, -2.0, 2.0] {
                        for ai2 in [-1.0f32, 2.0] {
                            let i = FrameInput {
                                vx,
                                vy,
                                direction: if combo % 2 == 0 { 1 } else { -1 },
                                ai: [ai0, 15.0, ai2, 0.0],
                                ..Default::default()
                            };
                            for _ in 0..10 {
                                find_frame_extra7b(k, &mut s, &i);
                                assert!(
                                    s.row < k.frame_count(),
                                    "{k:?} combo {combo} row {} escapes the {}-row sheet",
                                    s.row,
                                    k.frame_count()
                                );
                            }
                            combo += 1;
                        }
                    }
                }
            }
        }
    }

    /// The seam maps exactly this slice's ids — the d40-covered
    /// 472, the no-case 478, the next-slice 494 and the port-covered
    /// 305 stay `None`.
    #[test]
    fn f26s7b_the_seam_maps_exactly_the_slice() {
        for (id, k) in [
            (473, FindFrameExtra7bKind::BigMimicCorruption),
            (474, FindFrameExtra7bKind::BigMimicCrimson),
            (475, FindFrameExtra7bKind::BigMimicHallow),
            (476, FindFrameExtra7bKind::BigMimicJungle),
            (477, FindFrameExtra7bKind::Mothron),
            (479, FindFrameExtra7bKind::MothronSpawn),
            (480, FindFrameExtra7bKind::Medusa),
            (481, FindFrameExtra7bKind::GreekSkeleton),
            (482, FindFrameExtra7bKind::GraniteGolem),
            (483, FindFrameExtra7bKind::GraniteFlyer),
            (484, FindFrameExtra7bKind::EnchantedNightcrawler),
            (485, FindFrameExtra7bKind::Grubby),
            (486, FindFrameExtra7bKind::Sluggy),
            (487, FindFrameExtra7bKind::Buggy),
            (488, FindFrameExtra7bKind::TargetDummy),
        ] {
            assert_eq!(find_frame_kind_extra7b(id), Some(k));
            assert_eq!(k.vanilla_type(), id);
        }
        for id in [305, 452, 472, 478, 494] {
            assert_eq!(find_frame_kind_extra7b(id), None, "{id}");
        }
    }
}
