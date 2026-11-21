//! W4-f27 — FindFrame extra wave 11 (slice 11): the 550+ band FIX wave
//! plus the LOWER-GAPS 435/436. The census (the ANALYSIS "W4-f27
//! FindFrame extra wave 11" claim) re-derived mechanically against ALL
//! pushed tips: d41 tip 4ed22f0b3 is STILL CLAIM-only (its 550-577 DD2
//! roster rows are SetDefaults/AI — the FindFrame arms there are OUR
//! tree's M4-EVENT-19 v1 stand-ins, the invented `(vx != 0) + 0.6 >= 8`
//! free strips); d42 tip 9b22c091 landed the 578-630 roster WITH
//! FindFrame arms (EXCLUDE that band); d49's high2 seam owns 430-434
//! only — **435/436 are the d49-deferred armed-zombie tail** (its
//! module header's own "rides a later wave" note, no NpcKind on any
//! pushed tip). f26's pushed 7f owns 531/532/534/535/536 — the SAME
//! five our extra-10 owns (the both-lanes keep-one, the ANALYSIS note).
//!
//! THIS wave: the 2 NEW unclaimed ids 435/436 (pure machines, the
//! extra-8 precedent — the roster round wires them) PLUS the 13 FIX
//! kinds 550/552-563 (our own M4-EVENT-19 v1 arms WRONG — the slice-9/
//! 10 471/492/549 stub-fix precedent; the npc.rs arms delegate here in
//! the wiring commits). The 551/564-577 DD2 tail stays DEFERRED (the
//! complex/RNG/emote/dust seams, cited per kind in the ANALYSIS claim).
//!
//! The per-kind notes (the full table is in ANALYSIS):
//! * **435 ArmedZombieTwiggy / 436 ArmedZombieCenx** — the
//!   `case 430..436/591` group (`NPC.cs:70879-70951`), the VERBATIM d49
//!   armed-zombie machine (the 7c ONE-group precedent): the ai[2] > 0
//!   swing ladder rows 3/4/5/6 at the <5/<10/<15 bands breaking BEFORE
//!   the flip, the grounded ±1 one-sided flips, the air-OR-counter-
//!   motion row-2 pin, the idle row-0 snap, the |vx| accumulator with
//!   the <8/<16/<24/<32 bands rows 0/1/2/1 and the >=32 reset-tick
//!   row-HOLD (`:70947-70950` writes no frame). 591 is d42's.
//! * **550 DD2Bartender** — the humanoid-group TOWN walk (`:68616` →
//!   `:68647-69655`): townNPC true (`:16263`, aiStyle 7) so num57 =
//!   `ExtraFramesCount[550]` = 9 (`NPCID.cs:10464`) — the walk wraps at
//!   25 − 9 = 16 (the e8 SkeletonMerchant precedent); num108 = 6,
//!   num109 = row 2, idle row 0 (not the 140/489 pair), air row 1 (not
//!   in the `:69647` pin list). The ai[0] pose zones (`:68662-69488`,
//!   with their chat-bubble Main.rand draws) are the documented skip.
//! * **552-554 the DD2 goblins** — their shared case (`:67452-67492`):
//!   the ai[0] > 0 attack ladder rows 10-13 (the ai[1] == 29 reset, the
//!   ++counter >= 6 step gated on row < 13 — at row 13 the counter
//!   keeps growing), else the grounded walk: the air-OR-backpedal
//!   row-8 pin, the idle row-9 snap, the |vx| accumulator with the
//!   >= 64 reset and the `(int)(counter/8)` row 0-7.
//! * **555-557 the DD2 bombers** — their shared case (`:67797-67837`):
//!   the ai[1] > 0 && ai[0] > 0 attack ladder rows 10-16 (the ai[1] ==
//!   41 reset, the >= 6 step gated on row < 16), else the walk: air
//!   row 1, idle row 0, the |vx| >= 64 reset + `(int)(counter/8) + 2`
//!   row 2-9.
//! * **558-560 the DD2 wyverns** — their shared case (`:67838-67873`):
//!   the vx-sign sprite copies, the `ToRotation` aim (`Utils.cs:1655`
//!   = atan2(vy, vx)) + `(float)Math.PI` when vx < 0 (`:67853` — the
//!   REAL PI, not the eye group's literal 3.14f), OVERRIDDEN to
//!   `vx * 0.1` when ai[0] != 2 (`:67855-67858`); the ai[0] == 2 dive
//!   row-4 pin (counter UNTOUCHED) else the ++counter >= 6 step over
//!   rows 0-3 with the >= 4 wrap. Our v1 arm missed the sprite AND
//!   rotation writes entirely.
//! * **561-563 the DD2 javelineers** — their shared case
//!   (`:67756-67796`): the goblin shape with the ai[1] > 0 && ai[0] > 0
//!   attack gate, rows 10-13, the ai[1] == 89 reset.
//!
//! Shape: the 13 DD2 kinds ARE our kinds — their npc.rs find_frame arms
//! delegate here in the wiring commits (the 549-fix bridge shape).
//! 435/436 are NOT in this branch's NpcKind — their machines land pure
//! (the extra-8 precedent); the wiring seam is
//! [`find_frame_kind_extra11`]. No arm draws `Main.rand` — verified
//! mechanically per cited range (the ANALYSIS claim; the 574/575 RNG
//! dust and the 572/573 emote are the DEFERRED kinds, not this wave's).
//!
//! Row convention: the port tracks frame ROW indices, not pixels — the
//! C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison. The per-kind row
//! counts are `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry
//! initializer), counted entry-by-entry in
//! [`FindFrameExtra11Kind::frame_count`] and cross-checked against the
//! port's own `render/entities.rs::NPC_FRAME_COUNT` table.

// 435/436's wiring is a later round (the roster wave owns the kinds) —
// the `npc_findframe_extra8.rs` precedent.
#![allow(dead_code)]

/// The extra-wave-11 kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra11Kind {
    /// `NPCID.ArmedZombieTwiggy` = 435 (`NPCID.cs:11545`) — the
    /// `case 430..436/591` group (`NPC.cs:70884`), the d49-deferred
    /// tail.
    ArmedZombieTwiggy,
    /// `NPCID.ArmedZombieCenx` = 436 (`NPCID.cs:11547`) — the same
    /// group (`NPC.cs:70885`).
    ArmedZombieCenx,
    /// `NPCID.DD2Bartender` = 550 (`NPCID.cs:11777`, the port's
    /// `DD2Bartender`) — the humanoid-group town walk (`NPC.cs:68616`).
    DD2Bartender,
    /// `NPCID.DD2GoblinT1` = 552 (`NPCID.cs:11781`) — the goblin strip
    /// (`NPC.cs:67452`).
    DD2GoblinT1,
    /// `NPCID.DD2GoblinT2` = 553 (`NPCID.cs:11783`) — the same strip
    /// (`NPC.cs:67453`).
    DD2GoblinT2,
    /// `NPCID.DD2GoblinT3` = 554 (`NPCID.cs:11785`) — the same strip
    /// (`NPC.cs:67454`).
    DD2GoblinT3,
    /// `NPCID.DD2GoblinBomberT1` = 555 (`NPCID.cs:11787`) — the bomber
    /// strip (`NPC.cs:67797`).
    DD2GoblinBomberT1,
    /// `NPCID.DD2GoblinBomberT2` = 556 (`NPCID.cs:11789`) — the same
    /// strip (`NPC.cs:67798`).
    DD2GoblinBomberT2,
    /// `NPCID.DD2GoblinBomberT3` = 557 (`NPCID.cs:11791`) — the same
    /// strip (`NPC.cs:67799`).
    DD2GoblinBomberT3,
    /// `NPCID.DD2WyvernT1` = 558 (`NPCID.cs:11793`) — the wyvern strip
    /// (`NPC.cs:67838`).
    DD2WyvernT1,
    /// `NPCID.DD2WyvernT2` = 559 (`NPCID.cs:11795`) — the same strip
    /// (`NPC.cs:67839`).
    DD2WyvernT2,
    /// `NPCID.DD2WyvernT3` = 560 (`NPCID.cs:11797`) — the same strip
    /// (`NPC.cs:67840`).
    DD2WyvernT3,
    /// `NPCID.DD2JavelinstT1` = 561 (`NPCID.cs:11799`, the port's
    /// `DD2JavelinT1`) — the javelineer strip (`NPC.cs:67756`).
    DD2JavelinT1,
    /// `NPCID.DD2JavelinstT2` = 562 (`NPCID.cs:11801`) — the same
    /// strip (`NPC.cs:67757`).
    DD2JavelinT2,
    /// `NPCID.DD2JavelinstT3` = 563 (`NPCID.cs:11803`) — the same
    /// strip (`NPC.cs:67758`).
    DD2JavelinT3,
}

impl FindFrameExtra11Kind {
    /// The vanilla `NPC.type` — the `FindFrame` switch key.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::ArmedZombieTwiggy => 435,
            Self::ArmedZombieCenx => 436,
            Self::DD2Bartender => 550,
            Self::DD2GoblinT1 => 552,
            Self::DD2GoblinT2 => 553,
            Self::DD2GoblinT3 => 554,
            Self::DD2GoblinBomberT1 => 555,
            Self::DD2GoblinBomberT2 => 556,
            Self::DD2GoblinBomberT3 => 557,
            Self::DD2WyvernT1 => 558,
            Self::DD2WyvernT2 => 559,
            Self::DD2WyvernT3 => 560,
            Self::DD2JavelinT1 => 561,
            Self::DD2JavelinT2 => 562,
            Self::DD2JavelinT3 => 563,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// entry `type` on line `67084 + type/10`, slot `type % 10`) —
    /// cross-checked against `render/entities.rs::NPC_FRAME_COUNT`.
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67127 (430-439): slots 5/6 — the 7-row armed
            // zombie sheet (the d49 430-434 count).
            Self::ArmedZombieTwiggy | Self::ArmedZombieCenx => 7,
            // Main.cs:67139 (550-559): slots 0/2/3/4/5/6/7/8/9.
            Self::DD2Bartender => 25,
            Self::DD2GoblinT1 | Self::DD2GoblinT2 | Self::DD2GoblinT3 => 14,
            Self::DD2GoblinBomberT1 | Self::DD2GoblinBomberT2 | Self::DD2GoblinBomberT3 => 17,
            Self::DD2WyvernT1 | Self::DD2WyvernT2 | Self::DD2WyvernT3 => 5,
            // Main.cs:67140 (560-569): slots 1/2/3.
            Self::DD2JavelinT1 | Self::DD2JavelinT2 | Self::DD2JavelinT3 => 14,
        }
    }
}

/// The wiring seam: the vanilla type → this module's kind (the
/// `find_frame_kind_extra10` precedent). `None` outside the slice.
pub fn find_frame_kind_extra11(vanilla_type: i16) -> Option<FindFrameExtra11Kind> {
    Some(match vanilla_type {
        435 => FindFrameExtra11Kind::ArmedZombieTwiggy,
        436 => FindFrameExtra11Kind::ArmedZombieCenx,
        550 => FindFrameExtra11Kind::DD2Bartender,
        552 => FindFrameExtra11Kind::DD2GoblinT1,
        553 => FindFrameExtra11Kind::DD2GoblinT2,
        554 => FindFrameExtra11Kind::DD2GoblinT3,
        555 => FindFrameExtra11Kind::DD2GoblinBomberT1,
        556 => FindFrameExtra11Kind::DD2GoblinBomberT2,
        557 => FindFrameExtra11Kind::DD2GoblinBomberT3,
        558 => FindFrameExtra11Kind::DD2WyvernT1,
        559 => FindFrameExtra11Kind::DD2WyvernT2,
        560 => FindFrameExtra11Kind::DD2WyvernT3,
        561 => FindFrameExtra11Kind::DD2JavelinT1,
        562 => FindFrameExtra11Kind::DD2JavelinT2,
        563 => FindFrameExtra11Kind::DD2JavelinT3,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection`,
/// `rotation` (the wyvern aim). The wiring step copies them to/from
/// `Npc` (the 549-fix bridge shape). No arm writes `Entity.direction`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter` (the C# field is a `double`, `NPC.cs:6485`;
    /// the port's f32 is the existing `npc.rs` convention — the
    /// integer accumulators here are exact in both).
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — only the wyvern arms (558-560) write it (the
    /// every-tick aim); the other arms never touch it.
    pub rotation: f32,
}

/// The per-tick inputs the arms read: `velocity`, `direction`, the
/// `ai[0]`/`ai[1]` slots (the DD2 attack gates + resets) and `ai[2]`
/// (the armed zombies' swing band). No other ai slot, no world reads.
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone in the
    /// ±1-gated arms; the DD2 attack/walk copies are UNCONDITIONAL,
    /// 0 included).
    pub direction: i32,
    /// `NPC.ai[0]` — the DD2 attack gates (the goblins' `> 0`, the
    /// bombers'/javelineers' `ai[1] > 0 && ai[0] > 0`, the wyverns'
    /// `== 2` dive).
    pub ai0: f32,
    /// `NPC.ai[1]` — the DD2 attack sub-state (the goblins' 29 reset,
    /// the bombers' 41, the javelineers' 89).
    pub ai1: f32,
    /// `NPC.ai[2]` — the armed zombies' swing band (`> 0` rides rows
    /// 3-6 at the <5/<10/<15 ladder, `NPC.cs:70887-70905`).
    pub ai2: f32,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1, ai0: 0.0, ai1: 0.0, ai2: 0.0 }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-11 kinds — one
/// tick.
pub fn find_frame_extra11(kind: FindFrameExtra11Kind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // `case 435`/`case 436` in the `case 430..436/591` group
        // (`NPC.cs:70884-70885` → `:70879-70951`, the VERBATIM d49
        // armed-zombie machine — the 7c ONE-group precedent; d49's
        // high2 owns 430-434, 591 is d42's): the ai[2] > 0 swing
        // ladder (`:70887-70905`) picks rows 3/4/5/6 at the <5/<10/<15
        // bands and BREAKS before any flip; the grounded one-sided ±1
        // flips (`:70907-70917`); the air-OR-counter-motion row-2 pin
        // (`:70918-70923`); the idle row-0 snap (`:70924-70929`); the
        // |vx| accumulator (`:70930`) with the <8/<16/<24/<32 bands
        // rows 0/1/2/1 (`:70931-70946`) and the >=32 tail zeroes the
        // counter WITHOUT writing the row (`:70947-70950` — the
        // reset-tick row-HOLD quirk).
        FindFrameExtra11Kind::ArmedZombieTwiggy | FindFrameExtra11Kind::ArmedZombieCenx => {
            // `:70887-70905`.
            if i.ai2 > 0.0 {
                s.row = if i.ai2 < 5.0 {
                    3
                } else if i.ai2 < 10.0 {
                    4
                } else if i.ai2 < 15.0 {
                    5
                } else {
                    6
                };
                return;
            }
            // `:70907-70917`.
            if i.vy == 0.0 {
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
            }
            // `:70918-70923`.
            if i.vy != 0.0
                || (i.direction == -1 && i.vx > 0.0)
                || (i.direction == 1 && i.vx < 0.0)
            {
                s.counter = 0.0;
                s.row = 2;
                return;
            }
            // `:70924-70929`.
            if i.vx == 0.0 {
                s.counter = 0.0;
                s.row = 0;
                return;
            }
            // `:70930-70950`.
            s.counter += i.vx.abs();
            if s.counter < 8.0 {
                s.row = 0;
            } else if s.counter < 16.0 {
                s.row = 1;
            } else if s.counter < 24.0 {
                s.row = 2;
            } else if s.counter < 32.0 {
                s.row = 1;
            } else {
                // `:70949` — the row is NOT written here.
                s.counter = 0.0;
            }
        }
        // `case 550` in the humanoid-group case list (`NPC.cs:68616`)
        // → the shared TOWN walk (`:68647-69655`). The bartender IS
        // townNPC (`:16263`, aiStyle 7) so num57 =
        // `ExtraFramesCount[550]` = 9 (`NPCID.cs:10464`) and the walk
        // wraps at 25 − 9 = 16 (rows 2..15 — the e8 SkeletonMerchant
        // precedent). Not a town slime (`NPCID.cs:10119`'s set =
        // 670/678-684 — num108 = 6, num109 = row 2), not the 140/489
        // idle-row-1 pair (idle snaps row 0), NOT in the `:69647` air
        // pin list (the air holds row 1). The ai[0] pose zones
        // (`:68662-69488`, with their chat-bubble Main.rand draws) are
        // the documented skip.
        FindFrameExtra11Kind::DD2Bartender => {
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
                    // `:69521-69585` — num108 = 6, num109 = row 2.
                    // `:69572-69575`.
                    if s.row < 2 {
                        s.row = 2;
                    }
                    // `:69552-69553`.
                    s.counter += i.vx.abs() * 2.0;
                    s.counter += 1.0;
                    // `:69576-69580`.
                    if s.counter > 6.0 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                    // `:69581-69584` — the ExtraFramesCount-9 wrap.
                    if s.row >= kind.frame_count() - 9 {
                        s.row = 2;
                    }
                }
            } else {
                // `:69642-69646` — the group air tail: row 1, the
                // counter zeroed; 550 is NOT in the `:69647` pin list.
                s.counter = 0.0;
                s.row = 1;
            }
        }
        // `case 552`/`case 553`/`case 554` (`NPC.cs:67452-67454` →
        // `:67455-67492`): the ai[0] > 0 attack ladder
        // (`:67455-67469`): the UNCONDITIONAL sprite copy, the row
        // outside 10..13 OR ai[1] == 29 resets to row 10 + counter 0,
        // the ++counter >= 6 step gated on row < 13 (at row 13 the
        // counter keeps growing — the C# writes no reset there); else
        // the grounded walk: the vy == 0 sprite copy (`:67470-67473`),
        // the air-OR-backpedal row-8 pin (`:67474-67479`), the idle
        // row-9 snap (`:67480-67485`), the |vx| accumulator with the
        // >= 64 (or negative) reset (`:67486-67490`) and the
        // `(int)(counter/8)` row (`:67491`) over rows 0-7.
        FindFrameExtra11Kind::DD2GoblinT1
        | FindFrameExtra11Kind::DD2GoblinT2
        | FindFrameExtra11Kind::DD2GoblinT3 => {
            // `:67455-67469`.
            if i.ai0 > 0.0 {
                s.sprite_dir = i.direction;
                if s.row < 10 || s.row > 13 || i.ai1 == 29.0 {
                    s.row = 10;
                    s.counter = 0.0;
                }
                s.counter += 1.0;
                if s.counter >= 6.0 && s.row < 13 {
                    s.counter = 0.0;
                    s.row += 1;
                }
                return;
            }
            // `:67470-67473`.
            if i.vy == 0.0 {
                s.sprite_dir = i.direction;
            }
            // `:67474-67479`.
            if i.vy != 0.0
                || (i.direction == -1 && i.vx > 0.0)
                || (i.direction == 1 && i.vx < 0.0)
            {
                s.counter = 0.0;
                s.row = 8;
                return;
            }
            // `:67480-67485`.
            if i.vx == 0.0 {
                s.counter = 0.0;
                s.row = 9;
                return;
            }
            // `:67486-67491`.
            s.counter += i.vx.abs();
            if s.counter >= 64.0 || s.counter < 0.0 {
                s.counter = 0.0;
            }
            s.row = (s.counter / 8.0) as i32 as u32;
        }
        // `case 555`/`case 556`/`case 557` (`NPC.cs:67797-67799` →
        // `:67800-67837`): the ai[1] > 0 && ai[0] > 0 attack ladder
        // (`:67800-67814`): the UNCONDITIONAL sprite copy, the row
        // outside 10..16 OR ai[1] == 41 resets to row 10 + counter 0,
        // the ++counter >= 6 step gated on row < 16 (at row 16 the
        // counter keeps growing); else the walk: the vy == 0 sprite
        // copy (`:67815-67818`), the air-OR-backpedal row-1 pin
        // (`:67819-67824`), the idle row-0 snap (`:67825-67830`), the
        // |vx| accumulator with the >= 64 (or negative) reset
        // (`:67831-67835`) and the `(int)(counter/8) + 2` row
        // (`:67836`) over rows 2-9.
        FindFrameExtra11Kind::DD2GoblinBomberT1
        | FindFrameExtra11Kind::DD2GoblinBomberT2
        | FindFrameExtra11Kind::DD2GoblinBomberT3 => {
            // `:67800-67814`.
            if i.ai1 > 0.0 && i.ai0 > 0.0 {
                s.sprite_dir = i.direction;
                if s.row < 10 || s.row > 16 || i.ai1 == 41.0 {
                    s.row = 10;
                    s.counter = 0.0;
                }
                s.counter += 1.0;
                if s.counter >= 6.0 && s.row < 16 {
                    s.counter = 0.0;
                    s.row += 1;
                }
                return;
            }
            // `:67815-67818`.
            if i.vy == 0.0 {
                s.sprite_dir = i.direction;
            }
            // `:67819-67824`.
            if i.vy != 0.0
                || (i.direction == -1 && i.vx > 0.0)
                || (i.direction == 1 && i.vx < 0.0)
            {
                s.counter = 0.0;
                s.row = 1;
                return;
            }
            // `:67825-67830`.
            if i.vx == 0.0 {
                s.counter = 0.0;
                s.row = 0;
                return;
            }
            // `:67831-67836`.
            s.counter += i.vx.abs();
            if s.counter >= 64.0 || s.counter < 0.0 {
                s.counter = 0.0;
            }
            s.row = (s.counter / 8.0) as i32 as u32 + 2;
        }
        // `case 558` (`NPC.cs:67838` → `:67841-67873`, shared with
        // 559/560 — batch 3): the vx-sign sprite copies
        // (`:67842-67849`), the `ToRotation` aim (`Utils.cs:1655` =
        // atan2(vy, vx)) + `(float)Math.PI` when vx < 0 (`:67850-67854`
        // — the REAL PI, not the eye group's literal 3.14f),
        // OVERRIDDEN to `vx * 0.1` when ai[0] != 2 (`:67855-67858`);
        // the rotation write EVERY tick (`:67859`); the ai[0] == 2
        // dive row-4 pin with the counter UNTOUCHED (`:67860-67863`)
        // else the ++counter >= 6 step over rows 0-3 with the >= 4
        // wrap (`:67864-67872`). Our v1 arm missed the sprite AND
        // rotation writes entirely.
        FindFrameExtra11Kind::DD2WyvernT1 => {
            // `:67842-67849`.
            if i.vx > 0.0 {
                s.sprite_dir = 1;
            }
            if i.vx < 0.0 {
                s.sprite_dir = -1;
            }
            // `:67850-67854` — ToRotation + PI when moving left.
            let mut aim = i.vy.atan2(i.vx);
            if i.vx < 0.0 {
                aim += std::f32::consts::PI;
            }
            // `:67855-67858` — the non-dive override.
            if i.ai0 != 2.0 {
                aim = i.vx * 0.1;
            }
            // `:67859`.
            s.rotation = aim;
            // `:67860-67872`.
            if i.ai0 == 2.0 {
                s.row = 4;
            } else {
                s.counter += 1.0;
                if s.counter >= 6.0 {
                    s.row += 1;
                    s.counter = 0.0;
                    if s.row >= 4 {
                        s.row = 0;
                    }
                }
            }
        }
        // The batch-3 kinds land in the next commit (the 5-kinds-
        // per-commit rule — the e10 batch-1 catch-all precedent).
        _ => unimplemented!("extra-11 batch 3"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn rows_over(kind: FindFrameExtra11Kind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra11(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The frame-count table agrees with the port's verbatim
    /// `NPC_FRAME_COUNT` (`Main.cs:67082` initializer cross-check).
    #[test]
    fn f27s11_frame_counts_match_the_port_table() {
        let table = crate::render::entities::NPC_FRAME_COUNT;
        for k in [
            FindFrameExtra11Kind::ArmedZombieTwiggy,
            FindFrameExtra11Kind::ArmedZombieCenx,
            FindFrameExtra11Kind::DD2Bartender,
            FindFrameExtra11Kind::DD2GoblinT1,
            FindFrameExtra11Kind::DD2GoblinT2,
            FindFrameExtra11Kind::DD2GoblinT3,
            FindFrameExtra11Kind::DD2GoblinBomberT1,
            FindFrameExtra11Kind::DD2GoblinBomberT2,
            FindFrameExtra11Kind::DD2GoblinBomberT3,
            FindFrameExtra11Kind::DD2WyvernT1,
            FindFrameExtra11Kind::DD2WyvernT2,
            FindFrameExtra11Kind::DD2WyvernT3,
            FindFrameExtra11Kind::DD2JavelinT1,
            FindFrameExtra11Kind::DD2JavelinT2,
            FindFrameExtra11Kind::DD2JavelinT3,
        ] {
            assert_eq!(
                k.frame_count(),
                table[k.vanilla_type() as usize],
                "type {}",
                k.vanilla_type()
            );
        }
    }

    /// 435/436 the armed-zombie tail: the VERBATIM d49 group machine —
    /// the ai[2] swing ladder breaking before the flip, the air/turn
    /// row-2 pin, the |vx| bands with the >=32 reset-tick row-HOLD.
    /// Expectations re-derived by mechanical replay of the C#
    /// transcription (integer accumulators, exact in f32).
    #[test]
    fn f27s11_armed_zombie_tail_rides_the_d49_group_machine() {
        for k in [FindFrameExtra11Kind::ArmedZombieTwiggy, FindFrameExtra11Kind::ArmedZombieCenx] {
            // vx 2: +2 a tick — the <8/<16/<24/<32 bands rows 0/1/2/1,
            // the >=32 reset tick HOLDS row 1, then the cycle restarts.
            let rows = rows_over(k, input(2.0, 0.0), 17);
            assert_eq!(rows, vec![0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 1, 0]);
            // The reset-tick row-HOLD: row 1 + counter 30 → 32 zeroes
            // the counter, the row stays 1 (the C# writes no frame).
            let mut s = FrameState { row: 1, counter: 30.0, ..Default::default() };
            find_frame_extra11(k, &mut s, &input(2.0, 0.0));
            assert_eq!((s.row, s.counter), (1, 0.0));
            // The ai[2] swing ladder: rows 3/4/5/6, the counter AND
            // sprite UNTOUCHED (the break lands before the flip).
            for (ai2, row) in [(3.0, 3u32), (7.0, 4), (12.0, 5), (20.0, 6)] {
                let mut s = FrameState { row: 1, counter: 9.0, sprite_dir: -1, ..Default::default() };
                let mut i = input(2.0, 0.0);
                i.ai2 = ai2;
                find_frame_extra11(k, &mut s, &i);
                assert_eq!((s.row, s.counter, s.sprite_dir), (row, 9.0, -1));
            }
            // The air pin: row 2, counter zeroed.
            let mut s = FrameState { row: 5, counter: 4.0, ..Default::default() };
            find_frame_extra11(k, &mut s, &input(2.0, -3.0));
            assert_eq!((s.row, s.counter), (2, 0.0));
            // The counter-motion pin: direction -1 + vx +2 → row 2,
            // but the grounded flip fires FIRST (sprite -1).
            let mut i = input(2.0, 0.0);
            i.direction = -1;
            find_frame_extra11(k, &mut s, &i);
            assert_eq!((s.row, s.counter, s.sprite_dir), (2, 0.0, -1));
            // The idle snap: row 0, counter zeroed.
            find_frame_extra11(k, &mut s, &input(0.0, 0.0));
            assert_eq!((s.row, s.counter), (0, 0.0));
            // direction 0 leaves the sprite alone (the ±1 gates).
            let mut s = FrameState { sprite_dir: -1, ..Default::default() };
            let mut i = input(2.0, 0.0);
            i.direction = 0;
            find_frame_extra11(k, &mut s, &i);
            assert_eq!(s.sprite_dir, -1);
        }
    }

    /// 550 DD2Bartender: the town walk — num57 = 9 (the wrap at 16),
    /// num108 = 6 (the |vx|*2+1 >6 stride), idle row 0, air row 1, the
    /// ±1 flip. Expectations re-derived by mechanical replay.
    #[test]
    fn f27s11_bartender_walks_the_town_wrap_at_sixteen() {
        let k = FindFrameExtra11Kind::DD2Bartender;
        // vx 1: +3 a tick — the >6 step every 3rd tick, the lift-to-2.
        let rows = rows_over(k, input(1.0, 0.0), 12);
        assert_eq!(rows, vec![2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6]);
        // The wrap: row 15 + counter 6 → +3 = 9 > 6 steps to 16, the
        // >= 25-9 wrap snaps 2.
        let mut s = FrameState { row: 15, counter: 6.0, ..Default::default() };
        find_frame_extra11(k, &mut s, &input(1.0, 0.0));
        assert_eq!((s.row, s.counter), (2, 0.0));
        // The idle snap: row 0 + counter 0 (NOT the 140/489 row-1 pair).
        let mut s = FrameState { row: 7, counter: 4.0, ..Default::default() };
        find_frame_extra11(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        // The air tail: row 1 + counter 0 (NOT in the :69647 pin list).
        let mut s = FrameState { row: 7, counter: 4.0, ..Default::default() };
        find_frame_extra11(k, &mut s, &input(1.0, -3.0));
        assert_eq!((s.row, s.counter), (1, 0.0));
        // The grounded ±1 flip; direction 0 leaves the sprite alone.
        let mut s = FrameState::default();
        let mut i = input(1.0, 0.0);
        i.direction = -1;
        find_frame_extra11(k, &mut s, &i);
        assert_eq!(s.sprite_dir, -1);
        let mut i = input(1.0, 0.0);
        i.direction = 0;
        find_frame_extra11(k, &mut s, &i);
        assert_eq!(s.sprite_dir, -1);
        // No rotation write.
        let mut s = FrameState { rotation: 0.5, ..Default::default() };
        find_frame_extra11(k, &mut s, &input(1.0, 0.0));
        assert_eq!(s.rotation, 0.5);
    }

    /// 552/553/554 the DD2 goblins: the ai[0] attack ladder rows 10-13
    /// (the ai[1] == 29 reset, the row-13 counter-keep-growing quirk)
    /// + the walk (air 8, idle 9, the |vx| >=64 reset + counter/8 row).
    /// Expectations re-derived by mechanical replay.
    #[test]
    fn f27s11_goblins_walk_the_attack_ladder_and_the_64_reset() {
        for k in [
            FindFrameExtra11Kind::DD2GoblinT1,
            FindFrameExtra11Kind::DD2GoblinT2,
            FindFrameExtra11Kind::DD2GoblinT3,
        ] {
            // vx 4: +4 a tick — the (int)(counter/8) row, the >=64
            // reset tick shows row 0.
            let rows = rows_over(k, input(4.0, 0.0), 16);
            assert_eq!(rows, vec![0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 0]);
            // The attack ladder: ai0 1 — rows 10(x5)/11(x6)/12(x6)/13.
            let mut i = input(0.0, 0.0);
            i.ai0 = 1.0;
            let rows = rows_over(k, i, 20);
            assert_eq!(
                rows,
                vec![10, 10, 10, 10, 10, 11, 11, 11, 11, 11, 11, 12, 12, 12, 12, 12, 12, 13, 13, 13]
            );
            // The row-13 quirk: the counter keeps growing (no reset).
            let mut s = FrameState { row: 13, counter: 5.0, ..Default::default() };
            find_frame_extra11(k, &mut s, &i);
            assert_eq!((s.row, s.counter), (13, 6.0));
            // The ai[1] == 29 reset: row 10, counter 0 THEN ++ → 1.
            let mut s = FrameState { row: 12, counter: 3.0, ..Default::default() };
            let mut i29 = i;
            i29.ai1 = 29.0;
            find_frame_extra11(k, &mut s, &i29);
            assert_eq!((s.row, s.counter), (10, 1.0));
            // The attack sprite copy is UNCONDITIONAL (0 included).
            let mut s = FrameState { sprite_dir: -1, ..Default::default() };
            let mut i0 = i;
            i0.direction = 0;
            find_frame_extra11(k, &mut s, &i0);
            assert_eq!(s.sprite_dir, 0);
            // The air pin: row 8 + counter 0.
            let mut s = FrameState { row: 3, counter: 20.0, ..Default::default() };
            find_frame_extra11(k, &mut s, &input(4.0, -3.0));
            assert_eq!((s.row, s.counter), (8, 0.0));
            // The backpedal pin: direction -1 + vx +4 → row 8.
            let mut i = input(4.0, 0.0);
            i.direction = -1;
            find_frame_extra11(k, &mut s, &i);
            assert_eq!((s.row, s.counter), (8, 0.0));
            // The idle snap: row 9 + counter 0.
            find_frame_extra11(k, &mut s, &input(0.0, 0.0));
            assert_eq!((s.row, s.counter), (9, 0.0));
        }
    }

    /// 555-557 the DD2 bombers: the ai[1] > 0 && ai[0] > 0 attack
    /// ladder rows 10-16 (the ai[1] == 41 reset) + the walk (air 1,
    /// idle 0, the |vx| >=64 reset + counter/8+2 strip). Expectations
    /// re-derived by mechanical replay.
    #[test]
    fn f27s11_bombers_walk_the_plus_two_strip_and_the_41_reset() {
        for k in [
            FindFrameExtra11Kind::DD2GoblinBomberT1,
            FindFrameExtra11Kind::DD2GoblinBomberT2,
            FindFrameExtra11Kind::DD2GoblinBomberT3,
        ] {
            // vx 4: the (int)(counter/8)+2 row, the >=64 reset tick
            // shows row 2.
            let rows = rows_over(k, input(4.0, 0.0), 16);
            assert_eq!(rows, vec![2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 2]);
            // The attack ladder: ai0 1 + ai1 1 — rows 10(x5)/11-15(x6
            // each)/16 (the row-16 counter keeps growing).
            let mut i = input(0.0, 0.0);
            i.ai0 = 1.0;
            i.ai1 = 1.0;
            let rows = rows_over(k, i, 40);
            assert_eq!(
                rows,
                vec![
                    10, 10, 10, 10, 10, 11, 11, 11, 11, 11, 11, 12, 12, 12, 12, 12, 12, 13,
                    13, 13, 13, 13, 13, 14, 14, 14, 14, 14, 14, 15, 15, 15, 15, 15, 15, 16,
                    16, 16, 16, 16,
                ]
            );
            // The ai[1] == 41 reset: row 10, counter 0 THEN ++ → 1.
            let mut s = FrameState { row: 14, counter: 3.0, ..Default::default() };
            let mut i41 = i;
            i41.ai1 = 41.0;
            find_frame_extra11(k, &mut s, &i41);
            assert_eq!((s.row, s.counter), (10, 1.0));
            // The attack gate needs BOTH ai slots > 0 — ai1 alone
            // falls to the walk.
            let mut s = FrameState { row: 12, counter: 3.0, ..Default::default() };
            let mut i_only1 = input(0.0, 0.0);
            i_only1.ai1 = 1.0;
            find_frame_extra11(k, &mut s, &i_only1);
            assert_eq!((s.row, s.counter), (0, 0.0), "the idle snap, not the ladder");
            // The air pin: row 1 + counter 0.
            let mut s = FrameState { row: 5, counter: 20.0, ..Default::default() };
            find_frame_extra11(k, &mut s, &input(4.0, -3.0));
            assert_eq!((s.row, s.counter), (1, 0.0));
            // The idle snap: row 0 + counter 0.
            find_frame_extra11(k, &mut s, &input(0.0, 0.0));
            assert_eq!((s.row, s.counter), (0, 0.0));
        }
    }

    /// 558 the DD2 wyvern: the vx-sign sprite + the ToRotation aim
    /// (+PI when vx < 0, the vx*0.1 non-dive override) + the dive
    /// row-4 pin / the >=6 full-4 step. Expectations re-derived by
    /// mechanical replay (the rotation is the one f32 transcendental).
    #[test]
    fn f27s11_wyvern_aims_and_steps_the_full_four_rows() {
        let k = FindFrameExtra11Kind::DD2WyvernT1;
        // The >=6 step over rows 0-3 (row 4 is the dive pin only).
        let rows = rows_over(k, input(1.0, 0.0), 20);
        assert_eq!(
            rows,
            vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3]
        );
        // The wrap: row 3 + counter 5 → the step lands 4, the >=4 wrap
        // snaps 0.
        let mut s = FrameState { row: 3, counter: 5.0, ..Default::default() };
        find_frame_extra11(k, &mut s, &input(1.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        // The dive pin: ai0 == 2 → row 4, the counter UNTOUCHED, the
        // sprite -1 (vx < 0), the rotation the aim + PI.
        let mut s = FrameState { row: 1, counter: 3.0, ..Default::default() };
        let mut i = input(-2.0, -2.0);
        i.ai0 = 2.0;
        find_frame_extra11(k, &mut s, &i);
        assert_eq!((s.row, s.counter, s.sprite_dir), (4, 3.0, -1));
        let expect = (-2.0f32).atan2(-2.0) + std::f32::consts::PI;
        assert!((s.rotation - expect).abs() < 1e-6);
        // The non-dive override: rotation = vx*0.1 (the f32 product).
        let mut s = FrameState::default();
        let mut i = input(3.0, 9.0);
        i.ai0 = 0.0;
        find_frame_extra11(k, &mut s, &i);
        assert_eq!(s.rotation, 3.0 * 0.1);
        assert_eq!(s.sprite_dir, 1);
        // vx == 0: neither sprite if fires (the sprite holds), the
        // rotation still writes (the override vx*0.1 = 0).
        let mut s = FrameState { sprite_dir: -1, rotation: 0.7, ..Default::default() };
        find_frame_extra11(k, &mut s, &input(0.0, 5.0));
        assert_eq!(s.sprite_dir, -1);
        assert_eq!(s.rotation, 0.0);
    }

    /// The seam maps exactly the slice's vanilla types (the full
    /// 15-id set from batch 1 — the e10 stable-seam precedent; the
    /// batch-2/3 arms land behind the catch-all, the seam is stable).
    #[test]
    fn f27s11_seam_covers_exactly_the_slice() {
        for t in 0..697i16 {
            let k = find_frame_kind_extra11(t);
            let claimed = [
                435, 436, 550, 552, 553, 554, 555, 556, 557, 558, 559, 560, 561, 562, 563,
            ];
            assert_eq!(k.is_some(), claimed.contains(&t), "type {t}");
            if let Some(k) = k {
                assert_eq!(k.vanilla_type(), t);
            }
        }
    }
}
