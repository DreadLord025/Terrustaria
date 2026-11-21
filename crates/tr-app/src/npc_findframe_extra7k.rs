//! W4-f26 slice 7k — FindFrame extra wave 7k: the NEXT 6 TRULY
//! unclaimed case ids of `NPC.FindFrame` (`NPC.cs:66684-77370`) after
//! the 7j slice — **657/658/659/660** (QueenSlimeBoss + the
//! Blue/Pink/Purple minions, `ID/NPCID.cs:11991/:11993/:11995/
//! :11997`), **661** (EmpressButterfly, `NPCID.cs:11999`) and **662**
//! (PirateGhost, `NPCID.cs:12001`). SMALL batch by design (the
//! 400-storm brief: the 657-662 window, 6 kinds; the seam contracts
//! pre-probed by the brief and re-verified line-by-line here).
//!
//! The five arms, ported verbatim (657 is the only new large arm;
//! three of the six ids ride ALREADY-PORTED group machines):
//!
//! * 657 QueenSlimeBoss — its own `case 657` (`NPC.cs:66848-66960`,
//!   113 lines, read as two ≤60-line windows): the boss reads
//!   `life <= lifeMax / 2` (`:66850`, C# INTEGER division on the
//!   int pair — [`FrameInput::half_life`] is the RESOLVED flag, the
//!   7b resolved-read precedent), `noGravity` (`:66853`), `ai[0]`
//!   (the 5/4 attack states, `:66907/:66917`) and `ai[1]` (the
//!   attack timers, `(int)ai[1] / 3 % 3` and `(int)ai[1] / 15` —
//!   the C# float→int cast TRUNCATES toward zero, the port's
//!   `as i32`). Three motion branches: the rising/anti-grav flap
//!   (`:66853-66882` — the rows-4..7 cycle with the `-1.0`
//!   counter re-seed and the flag-keyed wrap to 7 (healthy) or 22
//!   (enraged), the rows-20..23 enraged loop at >= 5), the falling
//!   pin (`:66883-66900` — rows 8..10, the >= 8 step CLAMPS at 10,
//!   no wrap), and the grounded branch (`:66901-66958` — the
//!   ai[0]==5 smash rows 13/14/15, the ai[0]==4 windup rows
//!   12/11/10, else the idle walk: the rows-10..12 hop cycle at
//!   stride 6 wrapping at 13, else the rows-0..3 walk at stride
//!   10). The `frame.Width = 180` write (`:66851`) is DRAW-side —
//!   the 668 Deerclops precedent (`npc.rs:8207`: "vestigial ... the
//!   draw builds its own rect"); the frame machine carries rows
//!   only. `Main.npcFrameCount[657]` = **27** (`Main.cs:67149`
//!   entry 7) — the sheet is the 27-row boss grid, rows are NOT
//!   wrapped by the count (the arm's own 4/7/10/13/20/24 bounds
//!   rule).
//! * 658/659 QueenSlimeMinionBlue/Pink — the slime strip group
//!   `case 1/16/59/71/81/138/147/183/184/204/225/302/304/333-336/
//!   535/537/658/659/667/676` (`NPC.cs:70801-70846`, our labels
//!   `:70820-70821`): the num2-fed counter copied VERBATIM from
//!   the 7f SlimeSpiked arm (itself the extra-3 wave-A slime
//!   keep-one) — the `num2` motion class (`NPC.cs:66696-66704`:
//!   aiAction 0 → vy<0 ⇒ 2 / vy>0 ⇒ 3 / vx≠0 ⇒ 1 / still ⇒ 0;
//!   aiAction 1 ⇒ 4; anything else ⇒ 0) doubles the counter while
//!   moving/airborne and triples it while attacking; the >= 8
//!   stride walks the FULL sheet wrapping to 0 (`:70842-70845`).
//!   The `type == 302 || type == 304` spriteDirection snap
//!   (`:70824-70827`) is the town slimes' — 658/659 write no flip.
//!   Counts **16** and **2** (`Main.cs:67149` entry 8 / `:67150`
//!   entry 1).
//! * 660 QueenSlimeMinionPurple — the eye group `case 2/23/121/
//!   169/190-194/317/318/660` (`NPC.cs:73994-74059`, our label
//!   `:74005`): the `localAI[0] == 0f` one-time init LATCH
//!   (`:74014-74018` — `(type == 121 || type == 660)`: sets
//!   `localAI[0] = 1f` AND pins row 2; this arm WRITES
//!   `local_ai[0]`, the wiring copies it back), the vx-sign sprite
//!   gates + `rotation = vx * 0.1f` (`:74032-74043` — the
//!   `else if` is reachable for 660: the first branch's
//!   2/190-194/317/318 list does NOT carry it), the bare ++ and
//!   the >= **6** step (`:74044-74054` — the `type == 660`
//!   override at `:74046-74049`, NOT the group's 8), the FULL-count
//!   wrap to 0 (`:74055-74058`). The `type == 23` dust block
//!   (`:74007-74013`) is the Cursed Eye's — NOT 660's. Count **4**
//!   (`Main.cs:67150` entry 3).
//! * 661 EmpressButterfly — the butterfly group `case 356/444/653/
//!   661` (`NPC.cs:72739-72773`, our label `:72742`): the VERBATIM
//!   7j HellButterfly machine copy (the 7j/7i ONE-group precedent;
//!   the wiring merge keeps one) — `rotation = vx * 0.3f`
//!   (`:72745`), the UNCONDITIONAL sprite copy (`:72746`), the
//!   velocity-fed accumulator `+1.0 + (|vx| + |vy|)/2f` (`:72747`),
//!   the 7-tick bands rows 0/1/2/1 (`:72748-72762`) with the reset
//!   at `7*4-1 = 27` (`:72763-72766`), and NO colour-bank offset —
//!   the `:72768` gate (`type != 444 && type != 653 && type !=
//!   661`) EXCLUDES 661 (356-only; aiStyle 65's prismatic dye is
//!   draw-side). Count **3** (`Main.cs:67150` entry 4).
//! * 662 PirateGhost — the floater group `case 48/49/51/60/82/93/
//!   137/182/210/211/253/316/634/662/664` (`NPC.cs:74823-74884`,
//!   our label `:74836`): the extra-3 FloatyGross machine shape —
//!   the vx-sign sprite gates (`:74850-74857`), `rotation =
//!   vx * 0.1f` (`:74858`), the bare ++ (`:74864` — the 210/211
//!   second ++ does NOT apply), the >= 6 step (`:74875-74879` —
//!   the type-48 `num291 = 5` override does NOT apply) and the
//!   FULL-count wrap (`:74880-74883` — the `:74867` decrement gate
//!   `type == 49 || 51 || 60 || 634` EXCLUDES 662: the ghost walks
//!   all **4** rows, `Main.cs:67150` entry 5, unlike its 634
//!   sibling's count−1 flap). ZERO RNG: the `:74844`
//!   `Main.rand.Next(10)` draw short-circuits on `type == 634` —
//!   662 draws NOTHING (the 7h-published SporeBat seam contract's
//!   other-member clause); the `:74839-74843` dust block is type
//!   60's.
//!
//! Shape: the kinds are NOT in `NpcKind` yet (the roster wave owns
//! them) — the `npc_findframe_extra7j.rs` precedent: pure machines
//! over [`FrameState`]/[`FrameInput`], the wiring seam
//! [`find_frame_kind_extra7k`] hooked later at `npc.rs::find_frame`
//! by `cs_id`.
//!
//! RNG: NONE — the mechanical grep over the cited ranges
//! (`:66848-66960`, `:70801-70846`, `:72739-72773`, `:73994-74059`,
//! `:74823-74884`) hits ONLY the type-23 dust block (`:74009`),
//! the type-60 dust block (`:74841`) and the type-634 RNG draw +
//! dust (`:74844-74846`) — every hit behind a `type ==` gate that
//! is FALSE for this wave's ids; ZERO `Main.rand`/`.Next(` on any
//! 657-662 path, ZERO world reads (`Main.player`/`Main.npc[`/
//! `Collision.`/`WorldGen.`/`Framing.`).
//!
//! Row convention: frame ROW indices, not pixels (the C#'s `num`
//! cancels). Frame counts `Main.npcFrameCount` (`Main.cs:67082`'s
//! 697-entry initializer): 657 = **27** (`:67149` entry 7), 658 =
//! **16** (entry 8), 659 = **2** (`:67150` entry 1), 660 = **4**
//! (entry 3), 661 = **3** (entry 4), 662 = **4** (entry 5) —
//! counted entry-by-entry in [`FindFrameExtra7kKind::frame_count`]
//! and cross-checked against `render/entities.rs::NPC_FRAME_COUNT`
//! (all 697 slots re-diffed = ZERO mismatches, a mechanical
//! text-parse diff).

// The wiring is a later round (the kinds are not in `NpcKind` yet) —
// the `npc_findframe_extra7j.rs` precedent.
#![allow(dead_code)]

/// The extra-wave-7k kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra7kKind {
    /// `NPCID.QueenSlimeBoss` = 657 (`NPCID.cs:11991`) — its own
    /// boss case (`NPC.cs:66848`).
    QueenSlimeBoss,
    /// `NPCID.QueenSlimeMinionBlue` = 658 (`NPCID.cs:11993`) — the
    /// slime strip group (`NPC.cs:70820`).
    QueenSlimeMinionBlue,
    /// `NPCID.QueenSlimeMinionPink` = 659 (`NPCID.cs:11995`) — the
    /// same group (`NPC.cs:70821`).
    QueenSlimeMinionPink,
    /// `NPCID.QueenSlimeMinionPurple` = 660 (`NPCID.cs:11997`) —
    /// the eye group (`NPC.cs:74005`), the localAI[0] latch.
    QueenSlimeMinionPurple,
    /// `NPCID.EmpressButterfly` = 661 (`NPCID.cs:11999`) — the
    /// butterfly group (`NPC.cs:72742`), no colour bank.
    EmpressButterfly,
    /// `NPCID.PirateGhost` = 662 (`NPCID.cs:12001`) — the floater
    /// group (`NPC.cs:74836`), ZERO draws, the FULL-count walk.
    PirateGhost,
}

impl FindFrameExtra7kKind {
    /// The vanilla `NPC.type` id.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::QueenSlimeBoss => 657,
            Self::QueenSlimeMinionBlue => 658,
            Self::QueenSlimeMinionPink => 659,
            Self::QueenSlimeMinionPurple => 660,
            Self::EmpressButterfly => 661,
            Self::PirateGhost => 662,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// counted entry-by-entry): 657 = 27 (`:67149` entry 7), 658 =
    /// 16 (entry 8), 659 = 2 (`:67150` entry 1), 660 = 4 (entry 3),
    /// 661 = 3 (entry 4), 662 = 4 (entry 5).
    pub fn frame_count(self) -> u32 {
        match self {
            Self::QueenSlimeBoss => 27,
            Self::QueenSlimeMinionBlue => 16,
            Self::QueenSlimeMinionPink => 2,
            Self::QueenSlimeMinionPurple => 4,
            Self::EmpressButterfly => 3,
            Self::PirateGhost => 4,
        }
    }
}

/// The vanilla-id wiring seam: `Some(kind)` for exactly this wave's
/// ids (the `find_frame_kind_extra7j` precedent). The wiring round
/// hooks it at the `npc.rs::find_frame` head by `cs_id`.
pub fn find_frame_kind_extra7k(vanilla_type: i16) -> Option<FindFrameExtra7kKind> {
    Some(match vanilla_type {
        657 => FindFrameExtra7kKind::QueenSlimeBoss,
        658 => FindFrameExtra7kKind::QueenSlimeMinionBlue,
        659 => FindFrameExtra7kKind::QueenSlimeMinionPink,
        660 => FindFrameExtra7kKind::QueenSlimeMinionPurple,
        661 => FindFrameExtra7kKind::EmpressButterfly,
        662 => FindFrameExtra7kKind::PirateGhost,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection`,
/// `rotation` (660/661/662) and `localAI[0]` (660's init latch
/// WRITES it — the wiring must copy it back; the other kinds never
/// touch it). The `frame.Width = 180` write (`:66851`) is draw-side
/// (the Deerclops `npc.rs:8207` precedent), not carried here.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter` (the C# field is a `double`, `NPC.cs:6485`;
    /// the port's f32 is the existing `npc.rs` convention — the
    /// accumulator feeds are documented per witness).
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — 660/662 write `vx * 0.1f` (`:74042`/
    /// `:74858`), 661 writes `vx * 0.3f` (`:72745`); 657/658/659
    /// never touch it.
    pub rotation: f32,
    /// `NPC.localAI[0]`/`[1]` — 660's latch reads/writes `[0]`
    /// (`:74014-74018`); the other kinds carry it untouched for the
    /// uniform wiring copy.
    pub local_ai: [f32; 2],
}

/// The per-tick inputs the arms read. No RNG draws, no world reads.
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1) — 661 copies it UNCONDITIONALLY
    /// (`:72746`); the other kinds ignore it (660/662 gate on the
    /// vx SIGN, 657 never flips, 658/659's flip is the town
    /// slimes').
    pub direction: i32,
    /// `NPC.ai[0]`/`[1]` — 657 ONLY: the 5/4 attack states and the
    /// attack timers (`:66907-66933`). The other kinds ignore it.
    pub ai: [f32; 2],
    /// `NPC.aiAction` — 658/659 ONLY: the num2 motion class
    /// (`NPC.cs:66696-66704`, the 7f `ai_action` precedent).
    pub ai_action: i32,
    /// The RESOLVED `life <= lifeMax / 2` flag (`:66850` — C#
    /// INTEGER division on the int pair; the wiring computes it
    /// from the live ints, the 7b resolved-read precedent). 657
    /// ONLY.
    pub half_life: bool,
    /// `NPC.noGravity` (`:66853`) — 657 ONLY: with `half_life` it
    /// selects the enraged flap while grounded.
    pub no_gravity: bool,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self {
            vx: 0.0,
            vy: 0.0,
            direction: 1,
            ai: [0.0; 2],
            ai_action: 0,
            half_life: false,
            no_gravity: false,
        }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-7k kinds —
/// one tick.
pub fn find_frame_extra7k(kind: FindFrameExtra7kKind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // The boss case (`NPC.cs:66848-66960`) — the three motion
        // branches, the ai-keyed grounded states.
        FindFrameExtra7kKind::QueenSlimeBoss => {
            // `:66850-66852` — flag11 resolved on the input; the
            // `frame.Width = 180` write is draw-side (the header).
            let flag11 = i.half_life;
            let mut row = s.row;
            if (flag11 && i.no_gravity) || i.vy < 0.0 {
                // `:66855-66881` — the flap.
                if !(20..=23).contains(&row) {
                    if !(4..=7).contains(&row) {
                        row = 4;
                        s.counter = -1.0;
                    }
                    // `:66862-66870` — the ++ THEN >= 4 step; the
                    // >= 7 wrap picks 7 (healthy) / 22 (enraged).
                    s.counter += 1.0;
                    if s.counter >= 4.0 {
                        s.counter = 0.0;
                        row += 1;
                        if row >= 7 {
                            row = if !flag11 { 7 } else { 22 };
                        }
                    }
                } else {
                    // `:66872-66880` — the enraged 20..23 loop at
                    // >= 5, wrapping 24 → 20.
                    s.counter += 1.0;
                    if s.counter >= 5.0 {
                        s.counter = 0.0;
                        row += 1;
                        if row >= 24 {
                            row = 20;
                        }
                    }
                }
                s.row = row;
            } else if i.vy > 0.0 {
                // `:66885-66899` — the fall: rows 8..10, the >= 8
                // step CLAMPS at 10 (no wrap).
                if !(8..=10).contains(&row) {
                    row = 8;
                    s.counter = -1.0;
                }
                s.counter += 1.0;
                if s.counter >= 8.0 {
                    s.counter = 0.0;
                    row += 1;
                    if row >= 10 {
                        row = 10;
                    }
                }
                s.row = row;
            } else {
                // `:66903-66906` — the `velocity.Y != 0f` break is
                // DEAD behind the vy == 0 else (both signs handled
                // above); kept as the C#'s shape.
                if i.ai[0] == 5.0 {
                    // `:66907-66916` — the smash rows 13/14/15 by
                    // `(int)ai[1] / 3 % 3` (the cast truncates).
                    s.counter = 0.0;
                    row = match (i.ai[1] as i32) / 3 % 3 {
                        1 => 14,
                        2 => 15,
                        _ => 13,
                    };
                } else if i.ai[0] == 4.0 {
                    // `:66917-66933` — the windup rows 12/11/10 by
                    // `(int)ai[1] / 15` (2 AND 3 both map to 10).
                    s.counter = 0.0;
                    row = match (i.ai[1] as i32) / 15 {
                        1 => 11,
                        2 | 3 => 10,
                        _ => 12,
                    };
                } else {
                    // `:66936-66955` — the idle walk: the rows
                    // 10..12 hop cycle at stride 6 (wrapping at
                    // 13), else the rows 0..3 walk at stride 10.
                    let flag12 = (10..=12).contains(&row);
                    let stride = if flag12 { 6.0 } else { 10.0 };
                    if !flag12 && row >= 4 {
                        row = 0;
                        s.counter = -1.0;
                    }
                    s.counter += 1.0;
                    if s.counter >= stride {
                        s.counter = 0.0;
                        row += 1;
                        if (!flag12 || row == 13) && row >= 4 {
                            row = 0;
                        }
                    }
                }
                s.row = row;
            }
        }
        // The slime strip group (`NPC.cs:70828-70845`) — the
        // VERBATIM 7f SlimeSpiked machine copy (the extra-3 wave-A
        // slime keep-one; the wiring merge keeps one).
        FindFrameExtra7kKind::QueenSlimeMinionBlue | FindFrameExtra7kKind::QueenSlimeMinionPink => {
            // `NPC.cs:66696-66704` — the num2 motion class.
            let num2 = if i.ai_action == 0 {
                if i.vy < 0.0 {
                    2
                } else if i.vy > 0.0 {
                    3
                } else if i.vx != 0.0 {
                    1
                } else {
                    0
                }
            } else if i.ai_action == 1 {
                4
            } else {
                0
            };
            // `:70828-70845` — the 302/304 sprite snap does NOT
            // apply; the >= 8 stride walks the FULL sheet.
            s.counter += 1.0;
            if num2 > 0 {
                s.counter += 1.0;
            }
            if num2 == 4 {
                s.counter += 1.0;
            }
            if s.counter >= 8.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= kind.frame_count() {
                s.row = 0;
            }
        }
        // The eye group (`NPC.cs:74005-74059`), 660's slice.
        FindFrameExtra7kKind::QueenSlimeMinionPurple => {
            // `:74014-74018` — the one-time init latch (the type
            // gate reduces to 660 here; the 121 sibling is the
            // Demon Eye's own claim). WRITES local_ai[0].
            if s.local_ai[0] == 0.0 {
                s.local_ai[0] = 1.0;
                s.row = 2;
            }
            // `:74032-74043` — the vx-sign gates + the tilt (the
            // first branch's atan2 list does NOT carry 660, so
            // this `else if` IS reachable).
            if i.vx > 0.0 {
                s.sprite_dir = 1;
            }
            if i.vx < 0.0 {
                s.sprite_dir = -1;
            }
            s.rotation = i.vx * 0.1;
            // `:74044-74058` — the bare ++, the >= 6 step (the
            // `:74046-74049` 660 override, NOT the group's 8), the
            // FULL-count wrap.
            s.counter += 1.0;
            if s.counter >= 6.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= kind.frame_count() {
                s.row = 0;
            }
        }
        // The butterfly group (`NPC.cs:72743-72772`), 661's slice —
        // the VERBATIM 7j HellButterfly machine copy (the ONE-group
        // precedent). NO colour bank: the `:72768` gate excludes
        // 444/653/661.
        FindFrameExtra7kKind::EmpressButterfly => {
            let num143 = 7.0f32;
            // `:72745-72746` — the tilt + the UNCONDITIONAL copy.
            s.rotation = i.vx * 0.3;
            s.sprite_dir = i.direction;
            // `:72747` — the velocity-fed accumulator.
            s.counter += 1.0 + (i.vx.abs() + i.vy.abs()) / 2.0;
            // `:72748-72767` — rows 0/1/2/1, reset at 7*4-1 = 27.
            if s.counter < num143 {
                s.row = 0;
            } else if s.counter < num143 * 2.0 {
                s.row = 1;
            } else if s.counter < num143 * 3.0 {
                s.row = 2;
            } else {
                s.row = 1;
                if s.counter >= num143 * 4.0 - 1.0 {
                    s.counter = 0.0;
                }
            }
        }
        // The floater group (`NPC.cs:74836-74884`), 662's slice —
        // the extra-3 FloatyGross machine shape. ZERO draws: the
        // `:74844` RNG gate short-circuits on `type == 634`; the
        // `:74867` decrement gate excludes 662 (the FULL 4 rows).
        FindFrameExtra7kKind::PirateGhost => {
            // `:74850-74857`.
            if i.vx > 0.0 {
                s.sprite_dir = 1;
            }
            if i.vx < 0.0 {
                s.sprite_dir = -1;
            }
            // `:74858`.
            s.rotation = i.vx * 0.1;
            // `:74864-74883` — num291 = 6, num292 = the FULL count.
            s.counter += 1.0;
            if s.counter >= 6.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= kind.frame_count() {
                s.row = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIONS: [FindFrameExtra7kKind; 2] = [
        FindFrameExtra7kKind::QueenSlimeMinionBlue,
        FindFrameExtra7kKind::QueenSlimeMinionPink,
    ];

    const ALL: [FindFrameExtra7kKind; 6] = [
        FindFrameExtra7kKind::QueenSlimeBoss,
        FindFrameExtra7kKind::QueenSlimeMinionBlue,
        FindFrameExtra7kKind::QueenSlimeMinionPink,
        FindFrameExtra7kKind::QueenSlimeMinionPurple,
        FindFrameExtra7kKind::EmpressButterfly,
        FindFrameExtra7kKind::PirateGhost,
    ];

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn rows_over(kind: FindFrameExtra7kKind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra7k(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The frame-count table agrees with the port's verbatim
    /// `NPC_FRAME_COUNT` (`Main.cs:67082` initializer cross-check:
    /// 657=27 :67149 entry 7, 658=16 entry 8, 659=2 :67150 entry 1,
    /// 660=4 entry 3, 661=3 entry 4, 662=4 entry 5).
    #[test]
    fn f26s7k_frame_counts_match_the_port_table() {
        let table = crate::render::entities::NPC_FRAME_COUNT;
        for k in ALL {
            assert_eq!(
                k.frame_count(),
                table[k.vanilla_type() as usize],
                "{k:?} ({})",
                k.vanilla_type()
            );
        }
    }

    /// The boss healthy flap: fresh (row 0 outside 4..7) re-seeds
    /// row 4 + counter -1 → ++ = 0 — rows 4 on ticks 1-4, 5 on
    /// 5-8, 6 on 9-12; tick 13 steps 6 → 7 which is NOT >= 7, so
    /// row 7 shows; tick 17 steps 7 → 8 >= 7 → the healthy wrap
    /// picks 7 (NOT 0, NOT 22). The sprite/rotation are untouched.
    #[test]
    fn f26s7k_the_boss_healthy_flap_wraps_to_seven() {
        let k = FindFrameExtra7kKind::QueenSlimeBoss;
        let mut s = FrameState { sprite_dir: -1, rotation: 9.0, ..Default::default() };
        let i = input(0.0, -2.0);
        let mut rows = Vec::new();
        for _ in 0..17 {
            find_frame_extra7k(k, &mut s, &i);
            rows.push(s.row);
        }
        assert_eq!(
            rows,
            vec![4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 7]
        );
        assert_eq!((s.counter, s.sprite_dir, s.rotation), (0.0, -1, 9.0));
        // The re-seed edge: a row outside both bands lands 4 with
        // counter -1 THEN ++ = 0 in the SAME tick.
        let mut s = FrameState { row: 15, counter: 99.0, ..Default::default() };
        find_frame_extra7k(k, &mut s, &input(0.0, -1.0));
        assert_eq!((s.row, s.counter), (4, 0.0));
    }

    /// The boss enraged flap: half_life && noGravity selects the
    /// flap even GROUNDED (vy == 0); from row 6 the >= 7 wrap picks
    /// **22** (the flag-keyed `:66868`), then the rows-20..23 loop
    /// runs at >= 5 and wraps 24 → 20. Healthy + noGravity alone
    /// does NOT select the flap while grounded (the `&&`).
    #[test]
    fn f26s7k_the_boss_enraged_flap_and_the_twenty_loop() {
        let k = FindFrameExtra7kKind::QueenSlimeBoss;
        let enraged = FrameInput { half_life: true, no_gravity: true, ..Default::default() };
        let mut s = FrameState { row: 6, counter: 3.0, ..Default::default() };
        find_frame_extra7k(k, &mut s, &enraged);
        assert_eq!((s.row, s.counter), (22, 0.0));
        let mut rows = Vec::new();
        for _ in 0..11 {
            find_frame_extra7k(k, &mut s, &enraged);
            rows.push(s.row);
        }
        // 22 on ticks 1-4, 23 on 5-9, tick 10 steps 23 → 24 → 20,
        // tick 11 shows 20.
        assert_eq!(rows, vec![22, 22, 22, 22, 23, 23, 23, 23, 23, 20, 20]);
        // The flag conjunction: noGravity WITHOUT half_life stays
        // grounded (the idle walk, not the flap).
        let mut s = FrameState::default();
        find_frame_extra7k(k, &mut s, &FrameInput { no_gravity: true, ..Default::default() });
        assert_eq!(s.row, 0);
        // half_life alone rising still flaps (vy < 0 suffices).
        let mut s = FrameState { row: 6, counter: 3.0, ..Default::default() };
        find_frame_extra7k(k, &mut s, &FrameInput { vy: -1.0, half_life: true, ..Default::default() });
        assert_eq!(s.row, 22);
    }

    /// The boss fall: fresh re-seeds row 8 + counter -1 → ++ = 0 —
    /// row 8 on ticks 1-8, tick 9 steps to 9, ticks 9-16 show 9,
    /// tick 17 steps 9 → 10 >= 10 → the CLAMP pins 10 (no wrap);
    /// further ticks re-clamp 10 → 11 → 10 every 8. The rising
    /// gate takes priority: half_life && noGravity with vy > 0
    /// still flaps (`:66853`'s OR).
    #[test]
    fn f26s7k_the_boss_fall_clamps_at_ten() {
        let k = FindFrameExtra7kKind::QueenSlimeBoss;
        let mut s = FrameState::default();
        let i = input(0.0, 2.0);
        let mut rows = Vec::new();
        for _ in 0..25 {
            find_frame_extra7k(k, &mut s, &i);
            rows.push(s.row);
        }
        assert_eq!(
            rows,
            vec![
                8, 8, 8, 8, 8, 8, 8, 8, // ticks 1-8
                9, 9, 9, 9, 9, 9, 9, 9, // 9-16
                10, 10, 10, 10, 10, 10, 10, 10, // 17-24
                10, // 25 — the re-clamp
            ]
        );
        let mut s = FrameState { row: 9, counter: 7.0, ..Default::default() };
        find_frame_extra7k(k, &mut s, &FrameInput { vy: 2.0, half_life: true, no_gravity: true, ..Default::default() });
        assert_eq!(s.row, 4, "the flap gate wins over the fall");
    }

    /// The boss grounded attack states: ai[0]==5 pins the smash
    /// rows by `(int)ai[1] / 3 % 3` (13/14/15) and ai[0]==4 the
    /// windup rows by `(int)ai[1] / 15` (12/11/10/10) — both ZERO
    /// the counter every tick; the float→int cast TRUNCATES
    /// (ai[1] = 5.9 → 5 → 5/3 = 1 → row 14).
    #[test]
    fn f26s7k_the_boss_attack_states_pin_rows_and_zero_the_counter() {
        let k = FindFrameExtra7kKind::QueenSlimeBoss;
        for (ai1, want) in [(0.0, 13), (3.0, 14), (6.0, 15), (9.0, 13), (5.9, 14)] {
            let mut s = FrameState { row: 7, counter: 42.0, ..Default::default() };
            let i = FrameInput { ai: [5.0, ai1], ..Default::default() };
            find_frame_extra7k(k, &mut s, &i);
            assert_eq!((s.row, s.counter), (want, 0.0), "smash ai1 {ai1}");
        }
        for (ai1, want) in [(0.0, 12), (14.9, 12), (15.0, 11), (30.0, 10), (45.0, 10), (60.0, 12)] {
            let mut s = FrameState { row: 7, counter: 42.0, ..Default::default() };
            let i = FrameInput { ai: [4.0, ai1], ..Default::default() };
            find_frame_extra7k(k, &mut s, &i);
            assert_eq!((s.row, s.counter), (want, 0.0), "windup ai1 {ai1}");
        }
        // The attack states are grounded-only: airborne vy < 0
        // flaps regardless of ai[0].
        let mut s = FrameState::default();
        find_frame_extra7k(k, &mut s, &FrameInput { vy: -1.0, ai: [5.0, 3.0], ..Default::default() });
        assert_eq!(s.row, 4);
    }

    /// The boss idle walk: fresh rows 0..3 at stride 10 (the
    /// counter -1 re-seed makes the first step land tick 10), the
    /// `(!flag12 || row == 13) && row >= 4` wrap goes 3 → 4 → 0;
    /// the rows-10..12 hop cycle runs at stride 6 and wraps
    /// 12 → 13 → 0 (the row == 13 clause); a row >= 4 outside
    /// 10..12 re-seeds the walk at 0.
    #[test]
    fn f26s7k_the_boss_idle_walk_and_hop_cycle() {
        let k = FindFrameExtra7kKind::QueenSlimeBoss;
        let mut s = FrameState::default();
        let mut rows = Vec::new();
        for _ in 0..41 {
            find_frame_extra7k(k, &mut s, &input(3.0, 0.0));
            rows.push(s.row);
        }
        // rows 0 ticks 1-10, 1 ticks 11-20, 2 ticks 21-30,
        // 3 ticks 31-40, tick 41 wraps to 0.
        assert_eq!(rows[9], 0);
        assert_eq!(rows[10], 1);
        assert_eq!(rows[30], 3);
        assert_eq!(rows[39], 3);
        assert_eq!(rows[40], 0);
        assert_eq!(s.counter, 0.0);
        // The hop cycle: row 10 counter 5 → ++ = 6 >= 6 → 11.
        let mut s = FrameState { row: 10, counter: 5.0, ..Default::default() };
        find_frame_extra7k(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (11, 0.0));
        // The hop wrap: row 12 counter 5 → 13 → the row == 13
        // clause → 0.
        let mut s = FrameState { row: 12, counter: 5.0, ..Default::default() };
        find_frame_extra7k(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        // The walk re-seed: row 15 (>= 4, outside 10..12) → 0,
        // counter -1 → ++ = 0.
        let mut s = FrameState { row: 15, counter: 99.0, ..Default::default() };
        find_frame_extra7k(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        // The stride edge: row 1 counter 8 → ++ = 9 < 10, no step.
        let mut s = FrameState { row: 1, counter: 8.0, ..Default::default() };
        find_frame_extra7k(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (1, 9.0));
    }

    /// The minion strip: still (num2 = 0) feeds +1 — the Blue's
    /// 16-row sheet steps every 8th tick and wraps at row 16; the
    /// Pink's 2-row sheet cycles 0×8 / 1×8. Moving (vx ≠ 0, num2 =
    /// 1) feeds +2 (step every 4th), airborne rising (num2 = 2)
    /// also +2, falling (num2 = 3) +2, attacking (aiAction 1,
    /// num2 = 4) +3 — the 7f vectors verbatim. NO sprite flip
    /// (the 302/304 snap excludes them).
    #[test]
    fn f26s7k_the_minion_strip_num2_feeds() {
        let blue = FindFrameExtra7kKind::QueenSlimeMinionBlue;
        let pink = FindFrameExtra7kKind::QueenSlimeMinionPink;
        // The Pink still cycle: 0 ticks 1-8, 1 ticks 9-16, 0 at 17.
        assert_eq!(
            rows_over(pink, input(0.0, 0.0), 17),
            vec![0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0]
        );
        // The Blue wrap edge: row 15 counter 7 → ++ = 8 → row 16
        // → >= 16 → 0.
        let mut s = FrameState { row: 15, counter: 7.0, sprite_dir: -1, ..Default::default() };
        find_frame_extra7k(blue, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter, s.sprite_dir), (0, 0.0, -1));
        // The num2 feeds on the Pink: moving +2 steps tick 4;
        // attacking +3 steps tick 3 (3+3+3 = 9 >= 8).
        assert_eq!(rows_over(pink, input(2.0, 0.0), 5), vec![0, 0, 0, 1, 1]);
        let attack = FrameInput { ai_action: 1, ..Default::default() };
        assert_eq!(rows_over(pink, attack, 4), vec![0, 0, 1, 1]);
        // vy < 0 → num2 2 (+2); vy > 0 → num2 3 (+2) — same feed.
        assert_eq!(rows_over(pink, input(0.0, -2.0), 4), vec![0, 0, 0, 1]);
        assert_eq!(rows_over(pink, input(0.0, 2.0), 4), vec![0, 0, 0, 1]);
        // aiAction 2 (neither 0 nor 1) → num2 0 (+1).
        let other = FrameInput { ai_action: 2, vx: 5.0, ..Default::default() };
        assert_eq!(rows_over(pink, other, 8), vec![0, 0, 0, 0, 0, 0, 0, 1]);
        // Both minions share the machine tick-for-tick.
        for k in MINIONS {
            let mut s = FrameState::default();
            for _ in 0..8 {
                find_frame_extra7k(k, &mut s, &attack);
            }
            assert_eq!(s.counter, 0.0);
        }
    }

    /// The Purple minion latch: localAI[0] == 0 pins row 2 AND
    /// sets the slot to 1 in the SAME tick (the wiring copies it
    /// back); the counter still runs — the latch tick's ++ makes
    /// counter 1. Once latched (slot 1) the row walks freely at
    /// the >= 6 stride over the FULL 4 rows; vx signs flip the
    /// sprite, vx == 0 leaves it; the tilt is vx * 0.1.
    #[test]
    fn f26s7k_the_purple_minion_latch_and_six_stride() {
        let k = FindFrameExtra7kKind::QueenSlimeMinionPurple;
        let mut s = FrameState { row: 1, counter: 3.0, sprite_dir: -1, ..Default::default() };
        find_frame_extra7k(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter, s.local_ai[0]), (2, 4.0, 1.0));
        // Latched: row 2 counter 4 → ++ = 5 < 6 holds; ++ = 6
        // steps to 3; row 3 + step wraps 4 → 0.
        find_frame_extra7k(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (2, 5.0));
        find_frame_extra7k(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (3, 0.0));
        let mut s = FrameState { row: 3, counter: 5.0, local_ai: [1.0, 0.0], ..Default::default() };
        find_frame_extra7k(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        // The sprite/tilt: vx > 0 → +1, vx < 0 → -1, 0 untouched.
        let mut s = FrameState { sprite_dir: -1, local_ai: [1.0, 0.0], ..Default::default() };
        find_frame_extra7k(k, &mut s, &input(2.0, 0.0));
        assert_eq!((s.sprite_dir, s.rotation), (1, 2.0f32 * 0.1));
        find_frame_extra7k(k, &mut s, &input(-2.0, 0.0));
        assert_eq!((s.sprite_dir, s.rotation), (-1, -2.0f32 * 0.1));
        find_frame_extra7k(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.sprite_dir, s.rotation), (-1, 0.0));
        // The latch fires EXACTLY once: slot 1 never re-pins.
        let mut s = FrameState { row: 0, counter: 0.0, local_ai: [1.0, 0.0], ..Default::default() };
        find_frame_extra7k(k, &mut s, &input(0.0, 0.0));
        assert_eq!(s.row, 0);
    }

    /// The EmpressButterfly still flap — the 7j HellButterfly
    /// vectors VERBATIM (the ONE-group copy): +1.0 a tick, rows 0
    /// (ticks 1-6), 1 (7-13), 2 (14-20), 1 (21-26), the reset tick
    /// 27 SHOWS row 1, tick 28 back to 0; the sprite copy is
    /// UNCONDITIONAL and the tilt vx * 0.3. NO bank: the row never
    /// leaves 0..2 (the `:72768` gate excludes 661).
    #[test]
    fn f26s7k_the_empress_butterfly_still_flap() {
        let k = FindFrameExtra7kKind::EmpressButterfly;
        assert_eq!(
            rows_over(k, input(0.0, 0.0), 28),
            vec![
                0, 0, 0, 0, 0, 0,
                1, 1, 1, 1, 1, 1, 1,
                2, 2, 2, 2, 2, 2, 2,
                1, 1, 1, 1, 1, 1,
                1,
                0,
            ]
        );
        let mut s = FrameState { sprite_dir: 1, rotation: 9.0, ..Default::default() };
        find_frame_extra7k(k, &mut s, &FrameInput { direction: 0, ..Default::default() });
        assert_eq!((s.sprite_dir, s.rotation), (0, 0.0));
        // The velocity feed: vx 2 vy 2 → +3.0/tick (the 7j vector).
        let mut s = FrameState::default();
        let i = input(2.0, 2.0);
        let mut rows = Vec::new();
        for _ in 0..9 {
            find_frame_extra7k(k, &mut s, &i);
            rows.push(s.row);
        }
        assert_eq!(rows, vec![0, 0, 1, 1, 2, 2, 1, 1, 1]);
        assert_eq!(s.counter, 0.0);
        assert_eq!(s.rotation, 2.0f32 * 0.3);
        // The delegation-equality with the 7j twin over a grid.
        for vx in [0.0f32, 1.0, -2.0] {
            for vy in [0.0f32, -2.0, 2.0] {
                for counter in [0.0f32, 6.0, 13.0, 26.0] {
                    let mut a = FrameState { counter, ..Default::default() };
                    let mut b = crate::npc_findframe_extra7j::FrameState {
                        counter,
                        ..Default::default()
                    };
                    let ia = FrameInput { vx, vy, direction: -1, ..Default::default() };
                    let ib = crate::npc_findframe_extra7j::FrameInput { vx, vy, direction: -1, ..Default::default() };
                    find_frame_extra7k(k, &mut a, &ia);
                    crate::npc_findframe_extra7j::find_frame_extra7j(
                        crate::npc_findframe_extra7j::FindFrameExtra7jKind::HellButterfly,
                        &mut b,
                        &ib,
                    );
                    assert_eq!(
                        (a.row, a.counter, a.sprite_dir, a.rotation),
                        (b.row, b.counter, b.sprite_dir, b.rotation),
                        "vx {vx} vy {vy} c {counter}"
                    );
                }
            }
        }
    }

    /// The PirateGhost walk: the vx-sign gates + the vx * 0.1 tilt,
    /// the bare ++ and the >= 6 stride over the FULL 4 rows (the
    /// `:74867` decrement excludes 662 — row 3 IS walked, unlike
    /// the 634 SporeBat's count−1 flap): rows 0 ticks 1-6, 1 ticks
    /// 7-12, 2 ticks 13-18, 3 ticks 19-24, the wrap at tick 25.
    /// ZERO draws — the machine has no RNG seam at all.
    #[test]
    fn f26s7k_the_pirate_ghost_full_count_walk() {
        let k = FindFrameExtra7kKind::PirateGhost;
        let mut s = FrameState::default();
        let i = input(2.0, 0.0);
        let mut rows = Vec::new();
        for _ in 0..25 {
            find_frame_extra7k(k, &mut s, &i);
            rows.push(s.row);
        }
        assert_eq!(
            rows,
            vec![
                0, 0, 0, 0, 0, 0,
                1, 1, 1, 1, 1, 1,
                2, 2, 2, 2, 2, 2,
                3, 3, 3, 3, 3, 3,
                0,
            ]
        );
        assert_eq!((s.counter, s.sprite_dir, s.rotation), (0.0, 1, 2.0f32 * 0.1));
        // The wrap edge: row 3 counter 5 → ++ = 6 → row 4 → >= 4
        // → 0 (the FULL count, NOT count−1).
        let mut s = FrameState { row: 3, counter: 5.0, sprite_dir: 1, ..Default::default() };
        find_frame_extra7k(k, &mut s, &input(-1.0, 0.0));
        assert_eq!((s.row, s.counter, s.sprite_dir), (0, 0.0, -1));
        // vx == 0 leaves the sprite alone; the tilt still zeroes.
        let mut s = FrameState { sprite_dir: -1, rotation: 5.0, ..Default::default() };
        find_frame_extra7k(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.sprite_dir, s.rotation), (-1, 0.0));
        // The stride edge: counter 4 → ++ = 5 < 6, no step.
        let mut s = FrameState { row: 2, counter: 4.0, ..Default::default() };
        find_frame_extra7k(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (2, 5.0));
    }

    /// The seam maps exactly this wave's 6 ids — the slime group's
    /// other members (1/302/304/535/537/667/676), the eye group's
    /// (2/23/121/190/317/318), the butterfly siblings (356/444/
    /// 653) and the floater siblings (48/49/60/182/634/664) stay
    /// `None`.
    #[test]
    fn f26s7k_the_seam_maps_exactly_the_slice() {
        for k in ALL {
            let id = k.vanilla_type();
            assert_eq!(find_frame_kind_extra7k(id), Some(k));
        }
        for id in [1, 302, 304, 535, 537, 667, 676, 2, 23, 121, 190, 317, 318, 356, 444, 653, 48, 49, 60, 182, 634, 664, 656, 663] {
            assert_eq!(find_frame_kind_extra7k(id), None, "type {id}");
        }
    }

    /// The 6-kind strip sweep: every kind ticks 72 input combos
    /// (vx 0/2 × vy 0/-2/2 × ai_action 0/1 × half_life false/true ×
    /// no_gravity false/true) for 60 ticks each from a fresh state
    /// — every row stays inside its own sheet (the boss's 27-row
    /// grid included: the flap/fall/idle bounds never escape).
    #[test]
    fn f26s7k_the_6_kind_strip_sweep_stays_in_sheet() {
        for k in ALL {
            let mut combo = 0;
            for vx in [0.0f32, 2.0] {
                for vy in [0.0f32, -2.0, 2.0] {
                    for ai_action in [0i32, 1] {
                        for half_life in [false, true] {
                            for no_gravity in [false, true] {
                                let mut s = FrameState::default();
                                let i = FrameInput {
                                    vx,
                                    vy,
                                    ai_action,
                                    half_life,
                                    no_gravity,
                                    ..Default::default()
                                };
                                for _ in 0..60 {
                                    find_frame_extra7k(k, &mut s, &i);
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
    }
}
