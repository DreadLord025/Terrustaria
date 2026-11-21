//! W4-f27 — FindFrame extra wave 10 (slice 10): the 531+ band — the
//! FIRST 14 unclaimed case ids ascending from 531 (531/532/534/535/
//! 536/537/538/539/540/541/542/543/544/545) plus the 549 lane-portal
//! FIX (our own kind, the WRONG row-0 pin — the slice-9 471/492
//! precedent). The claim is the ANALYSIS "W4-f27 FindFrame extra wave
//! 10" section — the census re-derived mechanically against ALL pushed
//! tips (d41 CLAIM-only, d42 enum-only WIP, d43 at the wave-3 base —
//! all three carry the SAME 270-kind find_frame set as our tree; the
//! d44-d49 seams stop at 434; f26's pushed 7d tip 63417edd7 owns
//! 516/521/523/524/525 and its g-slice owns 526-530 — this lane stays
//! strictly above 530). Landed 5 kinds per commit, 3 batches.
//!
//! The per-kind notes (the full table is in ANALYSIS):
//! * **531 DesertScorpionWall** — the wall-crawler group
//!   (`NPC.cs:73116` in the 165/237/238/240 case list, `:73112-73145`)
//!   with the `type == 531` **num194 = 0.4** override (`:73119-73122`)
//!   — the e4 237/238/240 machine hard-codes 0.5 (its `:291-292`), so
//!   this is a distinct machine, the e4 header's own "later wave's"
//!   note (`:51-53`).
//! * **532 DesertBeast** — the walking-antlion group (`:70068` in the
//!   508/580/582 case list, `:70067-70103`): the air row-1 + idle
//!   row-0 pins BOTH leave the counter ALONE, the SIGN-GATED sprite
//!   copies, the row<2 lift-to-2 walk + the `1 + |vx|/2` accumulator
//!   with the >6 step wrapping at the 8-row count back to 2. f26's 7c
//!   owns 508 only; 580/582 ride later slices.
//! * **534 DemonTaxCollector** — the humanoid group (`:68615`) with
//!   the **num108 = 12** override (`:69528-69531`) — the wave-B
//!   `humanoid_fighter_frame` helper hard-codes >6, so this is a
//!   distinct machine (the e8 Fritz num108 = 9 precedent). aiStyle 3
//!   (`:15817`, no townNPC) → num57 = 0, the FULL-8 wrap.
//! * **535 SlimeSpiked / 537 SandSlime** — the num2-fed slime strip
//!   (`:70818`/`:70819` in the `:70801-70846` group): +1 always, +1
//!   more while moving/airborne (num2 > 0), +1 more on the attack
//!   class (num2 == 4, aiAction 1); the >=8 step walks the FULL sheet
//!   (2 rows for 535, 3 for 537). The 302/304 town-slime sprite snap
//!   (`:70824-70827`) is the town slimes', not these.
//! * **536 TheBride** — the case-3 walker group (`:76344` in the
//!   `:76324-76391` case list): the grounded ±1 flip, the
//!   air-OR-backpedal row-2 pin, the idle row-0 snap, the |vx|
//!   accumulator with the <8/<16/<24/<32 bands rows 0/1/2/1 and the
//!   >=32 reset-tick row-HOLD (the C# writes no frame there).
//! * **538 SquirrelRed / 539 SquirrelGold** — the squirrel group
//!   (`:76459`/`:76460` in the 299/639-645 case list, `:76458-76512`):
//!   the grounded ±1 flip, the |vx|*0.5+1 >6 walk over the FULL sheet
//!   wrapping at count-1 back to row **1** (`:76497-76500`), the air
//!   row 2 rising / row 4 falling. The aiStyle-3 backpedal branch
//!   (`:76478-76483`) is DEAD for these (style 7, `:15877`/`:15889`)
//!   — not carried, the documented dead branch.
//! * **540 PartyBunny** — the bunny hop group (`:76857` in the
//!   46/47/303/337/443/464 case list, `:76851-76904`): the grounded
//!   ±1 flip, the idle row-0 snap, the |vx|+1 >6 step over the FULL 7
//!   rows, the air pins row 4 rising / row 6 falling (the d44 46/47 +
//!   e8 443 keep-one twin).
//! * **541 SandElemental** — its own case (`:69734-69772`): the
//!   ai[0] > 0 attack ladder (rows 4 / the `(int)(ai[0]/8 % 4 + 5)`
//!   5-8 cycle / 9 / the `(int)((ai[0]-99-15)/7 + 10)` 10-12 climb /
//!   1) with the counter UNTOUCHED, else the `|velocity|*0.1 + 1`
//!   walk (the C# double-promotes the Length product, `:69761`) with
//!   the >=8 step over rows 0..3 wrapping at 4.
//! * **542-545 the sand sharks** — their shared case (`:74681-74692`):
//!   the UNCONDITIONAL direction copy, the counter++ with the >=16
//!   reset, the `(int)(counter/4)` row over the 4-row sheet.
//! * **549 DD2LanePortal** — its own case (`:67980-67992`): the
//!   sprite=1 + rotation=0 snap every tick, the `++counter >= 6`
//!   step over the FULL 8 rows. OUR `npc.rs` row-0 pin was WRONG
//!   (`npcFrameCount[549]` = 8, `Main.cs:67138` slot 9) — the arm is
//!   FIXED this slice (the 439-fix bridge shape).
//!
//! The EXCLUDED faithful arm: 533 DesertDjinn — OUR `npc.rs:8172` arm
//! is line-for-line faithful (the 48-tick hover cycle + the ai[1] > 0
//! +8 cast bank via the `cast_windup` mirror, `npc.rs:10804`); 546/
//! 547/548 have NO FindFrame case (npcFrameCount 1, `Main.cs:67138`).
//!
//! Shape: 549 IS our kind — its `npc.rs` find_frame arm delegates
//! here THIS slice (the 439-fix bridge shape). The other 14 are NOT
//! in this branch's `NpcKind` — their machines land as pure functions
//! over a small [`FrameState`]/[`FrameInput`] pair (the extra-8
//! precedent); the wiring seam is [`find_frame_kind_extra10`] and the
//! roster merge copies row/counter/sprite_dir/rotation out and back.
//! No arm draws `Main.rand` — verified mechanically per cited range
//! (the ANALYSIS claim's 11-range grep = ZERO).
//!
//! Row convention: the port tracks frame ROW indices, not pixels —
//! the C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison. The per-kind row
//! counts are `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry
//! initializer), counted entry-by-entry in
//! [`FindFrameExtra10Kind::frame_count`] and cross-checked against
//! the port's own `render/entities.rs::NPC_FRAME_COUNT` table.
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "W4-f27 FindFrame
//! extra wave 10"):
//! * the humanoid group's town `ai[0]` sit/talk/attack pose zones
//!   (`NPC.cs:68662-69488`) — 534 is an aiStyle-3 fighter
//!   (`NPC.cs:15817`, no townNPC), the wave-B
//!   `humanoid_fighter_frame` keep-one precedent;
//! * the squirrel group's aiStyle-3 backpedal branch
//!   (`NPC.cs:76478-76483`) — 538/539 are style 7, the branch never
//!   fires (the 299/639-645 group members' own wiring owns it);
//! * the slime group's 302/304 town-slime sprite snap
//!   (`NPC.cs:70824-70827`) — the town slimes', not 535/537's.

// The 14 new kinds' wiring is a later round (the roster wave owns the
// kinds) — the `npc_findframe_extra8.rs` precedent.
#![allow(dead_code)]

/// The extra-wave-10 kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra10Kind {
    /// `NPCID.DesertScorpionWall` = 531 (`NPCID.cs:11739`) — the
    /// wall-crawler group's 0.4 stride (`NPC.cs:73116`).
    DesertScorpionWall,
    /// `NPCID.DesertBeast` = 532 (`NPCID.cs:11741`) — the
    /// walking-antlion group (`NPC.cs:70068`).
    DesertBeast,
    /// `NPCID.DemonTaxCollector` = 534 (`NPCID.cs:11745`) — the
    /// humanoid group's num108 = 12 stride (`NPC.cs:68615`).
    DemonTaxCollector,
    /// `NPCID.SlimeSpiked` = 535 (`NPCID.cs:11747`) — the num2-fed
    /// slime strip (`NPC.cs:70818`).
    SlimeSpiked,
    /// `NPCID.TheBride` = 536 (`NPCID.cs:11749`) — the case-3 walker
    /// group (`NPC.cs:76344`).
    TheBride,
    /// `NPCID.SandSlime` = 537 (`NPCID.cs:11751`) — the num2-fed
    /// slime strip (`NPC.cs:70819`).
    SandSlime,
    /// `NPCID.SquirrelRed` = 538 (`NPCID.cs:11753`) — the squirrel
    /// group (`NPC.cs:76459`).
    SquirrelRed,
    /// `NPCID.SquirrelGold` = 539 (`NPCID.cs:11755`) — the squirrel
    /// group (`NPC.cs:76460`).
    SquirrelGold,
    /// `NPCID.PartyBunny` = 540 (`NPCID.cs:11757`) — the bunny hop
    /// group (`NPC.cs:76857`).
    PartyBunny,
    /// `NPCID.SandElemental` = 541 (`NPCID.cs:11759`) — the ai[0]
    /// attack ladder + the hover walk (`NPC.cs:69734-69772`).
    SandElemental,
    /// `NPCID.SandShark` = 542 (`NPCID.cs:11761`) — the shark strip
    /// (`NPC.cs:74681`).
    SandShark,
    /// `NPCID.SandsharkCorrupt` = 543 (`NPCID.cs:11763`) — the same
    /// strip (`NPC.cs:74682`).
    SandsharkCorrupt,
    /// `NPCID.SandsharkCrimson` = 544 (`NPCID.cs:11765`) — the same
    /// strip (`NPC.cs:74683`).
    SandsharkCrimson,
    /// `NPCID.SandsharkHallow` = 545 (`NPCID.cs:11767`) — the same
    /// strip (`NPC.cs:74684`).
    SandsharkHallow,
    /// `NPCID.DD2LanePortal` = 549 (`NPCID.cs:11775`, the port's
    /// `DD2LanePortal`) — `case 549` (`NPC.cs:67980-67992`), the
    /// sprite/rotation snap + the 6-tick full-8 strip.
    DD2LanePortal,
}

impl FindFrameExtra10Kind {
    /// The vanilla `NPC.type` — the `FindFrame` switch key.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::DesertScorpionWall => 531,
            Self::DesertBeast => 532,
            Self::DemonTaxCollector => 534,
            Self::SlimeSpiked => 535,
            Self::TheBride => 536,
            Self::SandSlime => 537,
            Self::SquirrelRed => 538,
            Self::SquirrelGold => 539,
            Self::PartyBunny => 540,
            Self::SandElemental => 541,
            Self::SandShark => 542,
            Self::SandsharkCorrupt => 543,
            Self::SandsharkCrimson => 544,
            Self::SandsharkHallow => 545,
            Self::DD2LanePortal => 549,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// entry `type` on line `67084 + type/10`, slot `type % 10`) —
    /// cross-checked against `render/entities.rs::NPC_FRAME_COUNT`.
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67137 (531-539): slots 1/2/4/5/6/7/8/9.
            Self::DesertScorpionWall => 4,
            Self::DesertBeast => 8,
            Self::DemonTaxCollector => 8,
            Self::SlimeSpiked => 2,
            Self::TheBride => 3,
            Self::SandSlime => 3,
            Self::SquirrelRed => 6,
            Self::SquirrelGold => 6,
            // Main.cs:67138 (540-545): slots 0-5; 549 slot 9.
            Self::PartyBunny => 7,
            Self::SandElemental => 13,
            Self::SandShark => 4,
            Self::SandsharkCorrupt => 4,
            Self::SandsharkCrimson => 4,
            Self::SandsharkHallow => 4,
            Self::DD2LanePortal => 8,
        }
    }
}

/// The wiring seam: the vanilla type → this module's kind (the
/// `find_frame_kind_extra9` precedent). `None` outside the slice.
pub fn find_frame_kind_extra10(vanilla_type: i16) -> Option<FindFrameExtra10Kind> {
    Some(match vanilla_type {
        531 => FindFrameExtra10Kind::DesertScorpionWall,
        532 => FindFrameExtra10Kind::DesertBeast,
        534 => FindFrameExtra10Kind::DemonTaxCollector,
        535 => FindFrameExtra10Kind::SlimeSpiked,
        536 => FindFrameExtra10Kind::TheBride,
        537 => FindFrameExtra10Kind::SandSlime,
        538 => FindFrameExtra10Kind::SquirrelRed,
        539 => FindFrameExtra10Kind::SquirrelGold,
        540 => FindFrameExtra10Kind::PartyBunny,
        541 => FindFrameExtra10Kind::SandElemental,
        542 => FindFrameExtra10Kind::SandShark,
        543 => FindFrameExtra10Kind::SandsharkCorrupt,
        544 => FindFrameExtra10Kind::SandsharkCrimson,
        545 => FindFrameExtra10Kind::SandsharkHallow,
        549 => FindFrameExtra10Kind::DD2LanePortal,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection`,
/// `rotation`. The wiring step copies them to/from `Npc` (the 549
/// bridge copies all four — its arm writes sprite/rotation every
/// tick). No arm writes `Entity.direction`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter`.
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — only the lane-portal arm (549) writes it (the
    /// every-tick zero snap); the other arms never touch it.
    pub rotation: f32,
}

/// The per-tick inputs the arms read: `velocity`, `direction`, the
/// `ai[0]` slot (the SandElemental ladder) and `aiAction` (the slime
/// strip's num2 class). No other ai slot, no world reads.
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone in
    /// the arms that gate on `direction == 1` / `== -1`; the shark
    /// strip copies it UNCONDITIONALLY, 0 included).
    pub direction: i32,
    /// `NPC.ai[0]` — only the SandElemental ladder reads it.
    pub ai0: f32,
    /// `NPC.aiAction` — 0 the motion class, 1 the attack (`num2 = 4`,
    /// `NPC.cs:66696-66704`); only the slime strip reads it.
    pub ai_action: i32,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1, ai0: 0.0, ai_action: 0 }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-10 kinds — one
/// tick.
pub fn find_frame_extra10(kind: FindFrameExtra10Kind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // `case 531` in the wall-crawler group (`NPC.cs:73116` →
        // `:73117-73145`): the `(|vx| + |vy|) * num194` accumulator
        // with the **type-531 num194 = 0.4 override** (`:73119-73122`
        // — the e4 237/238/240 machine's 0.5 is NOT this kind's
        // stride) over the <6/<12/<18/<24 bands rows 0/1/2/3; the
        // >= 24 tail zeroes the counter and HOLDS the row
        // (`:73140-73143` writes no frame). No sprite/rotation write,
        // no air split — the walls crawl in every direction.
        FindFrameExtra10Kind::DesertScorpionWall => {
            // `:73118-73123` — num194 = 0.4 for 531.
            s.counter += (i.vx.abs() + i.vy.abs()) * 0.4;
            // `:73124-73143`.
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
        }
        // `case 532` in the walking-antlion group (`NPC.cs:70068` →
        // `:70067-70103`, shared with 508/580/582 — f26's 7c owns 508
        // only): the air row-1 pin (`:70071-70075`) and the idle
        // row-0 pin (`:70076-70080`) BOTH leave the counter ALONE;
        // the SIGN-GATED sprite copies (`:70081-70088` — direction
        // AND vx must agree, the mismatch keeps the old sprite); the
        // row<2 lift-to-2 (`:70089-70092`), the `1 + |vx|/2`
        // accumulator with the >6 step (`:70093-70098`) and the
        // >= count wrap back to row 2 (`:70099-70102`).
        FindFrameExtra10Kind::DesertBeast => {
            // `:70071-70075`.
            if i.vy != 0.0 {
                s.row = 1;
                return;
            }
            // `:70076-70080`.
            if i.vx == 0.0 {
                s.row = 0;
                return;
            }
            // `:70081-70088`.
            if i.direction < 0 && i.vx < 0.0 {
                s.sprite_dir = -1;
            }
            if i.direction > 0 && i.vx > 0.0 {
                s.sprite_dir = 1;
            }
            // `:70089-70092`.
            if s.row < 2 {
                s.row = 2;
            }
            // `:70093-70098`.
            s.counter += 1.0 + i.vx.abs() / 2.0;
            if s.counter > 6.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            // `:70099-70102`.
            if s.row >= kind.frame_count() {
                s.row = 2;
            }
        }
        // `case 534` in the humanoid group (`NPC.cs:68615` →
        // `:68647-69655`): the grounded ±1 flip (`:68649-68656`), the
        // idle row-0 snap (`:69515-69519` — NOT the 140/489 row-1
        // pair, NOT the 638 blink), the walk with the **num108 = 12
        // override** (`:69528-69531` — the wave-B fighter helper's
        // >6 is NOT this kind's stride, the e8 Fritz precedent): the
        // lift-to-2 (`:69572-69575`), the `|vx| * 2 + 1` accumulator
        // (`:69552-69553` — NOT the 489 |vx|+0.5 branch), the >12
        // step (`:69576-69580`), the FULL-8 wrap back to 2
        // (`:69581-69584` — num57 = 0, aiStyle 3 `:15817` no
        // townNPC); the air row 1 + counter zero (`:69642-69646` —
        // NOT in the `:69647` pin list). The town pose zones are the
        // documented skip.
        FindFrameExtra10Kind::DemonTaxCollector => {
            if i.vy == 0.0 {
                // `:68649-68656`.
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
                    // `:69572-69575` — num109 = row 2.
                    if s.row < 2 {
                        s.row = 2;
                    }
                    // `:69552-69553`.
                    s.counter += i.vx.abs() * 2.0;
                    s.counter += 1.0;
                    // `:69576-69580` — num108 = 12 (the 534 override).
                    if s.counter > 12.0 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                    // `:69581-69584`.
                    if s.row >= kind.frame_count() {
                        s.row = 2;
                    }
                }
            } else {
                // `:69642-69646`.
                s.counter = 0.0;
                s.row = 1;
            }
        }
        // `case 535`/`case 537` in the num2-fed slime strip group
        // (`NPC.cs:70818`/`:70819` → `:70801-70846`, the port's Slime
        // arm + wave-A roster + e7 ribbons are the same machine,
        // keep-one at the merge): the counter feeds on the `num2`
        // motion class (`NPC.cs:66696-66704`): +1 always, +1 more
        // while moving/airborne (num2 > 0), +1 more on the attack
        // class (num2 == 4, aiAction 1); the >= 8 step walks the FULL
        // sheet with the >= count wrap. The 302/304 sprite snap
        // (`:70824-70827`) is the town slimes', not these. No
        // sprite/rotation write.
        FindFrameExtra10Kind::SlimeSpiked | FindFrameExtra10Kind::SandSlime => {
            // `:66696-66704` — the num2 class.
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
            // `:70828-70836`.
            s.counter += 1.0;
            if num2 > 0 {
                s.counter += 1.0;
            }
            if num2 == 4 {
                s.counter += 1.0;
            }
            // `:70837-70845`.
            if s.counter >= 8.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= kind.frame_count() {
                s.row = 0;
            }
        }
        // `case 536` in the case-3 walker group (`NPC.cs:76344` →
        // `:76324-76391`, the `npc.rs::walker_frame` / wave-A zombie
        // machine is the same arm, keep-one at the merge): the
        // grounded ±1 flip (`:76347-76357`), the air-OR-backpedal
        // row-2 pin (`:76358-76363`), the idle row-0 snap
        // (`:76364-76369`), the |vx| accumulator (`:76370`) with the
        // <8/<16/<24/<32 bands rows 0/1/2/1 (`:76371-76386`) and the
        // >= 32 reset-tick row-HOLD (`:76387-76390` writes no frame).
        FindFrameExtra10Kind::TheBride => {
            // `:76347-76357`.
            if i.vy == 0.0 {
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
            }
            // `:76358-76363`.
            if i.vy != 0.0
                || (i.direction == -1 && i.vx > 0.0)
                || (i.direction == 1 && i.vx < 0.0)
            {
                s.counter = 0.0;
                s.row = 2;
                return;
            }
            // `:76364-76369`.
            if i.vx == 0.0 {
                s.counter = 0.0;
                s.row = 0;
                return;
            }
            // `:76370-76390`.
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
                s.counter = 0.0;
            }
        }
        // `case 537` rides the SAME num2-fed slime strip arm as 535
        // (the `SlimeSpiked | SandSlime` pattern above — the C#'s
        // `:70818`/`:70819` labels share one body).
        // `case 538`/`case 539` in the squirrel group
        // (`NPC.cs:76459`/`:76460` → `:76458-76512`, shared with
        // 299/639-645): the grounded ±1 flip (`:76468-76477`), the
        // idle row-0 snap (`:76484-76489`), the `|vx| * 0.5 + 1`
        // accumulator with the >6 step (`:76490-76496`) and the
        // >= count-1 wrap back to row **1** (`:76497-76500` — NOT
        // row 0, the walk rows are 1..count-2 with row 0 the idle
        // pose); the air row 2 rising (`:76502-76506`) / row 4
        // falling (`:76507-76511`), both counter-zeroed. The
        // aiStyle-3 backpedal branch (`:76478-76483`) is DEAD for
        // these (style 7, `:15877`/`:15889`) — not carried, the
        // documented dead branch.
        FindFrameExtra10Kind::SquirrelRed | FindFrameExtra10Kind::SquirrelGold => {
            if i.vy == 0.0 {
                // `:76468-76477`.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                // `:76484-76489`.
                if i.vx == 0.0 {
                    s.row = 0;
                    s.counter = 0.0;
                    return;
                }
                // `:76490-76496`.
                s.counter += i.vx.abs() * 0.5;
                s.counter += 1.0;
                if s.counter > 6.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
                // `:76497-76500`.
                if s.row >= kind.frame_count() - 1 {
                    s.row = 1;
                }
            } else if i.vy < 0.0 {
                // `:76502-76506`.
                s.counter = 0.0;
                s.row = 2;
            } else {
                // `:76507-76511`.
                s.counter = 0.0;
                s.row = 4;
            }
        }
        // `case 540` in the bunny hop group (`NPC.cs:76857` →
        // `:76851-76904`, shared with 46/47/303/337/443/464 — d44's
        // 46/47 + e7's 337 + e8's 443 are the same machine, keep-one
        // at the merge): the grounded ±1 flip (`:76866-76875`), the
        // idle row-0 snap (`:76876-76881`), the `|vx| + 1`
        // accumulator with the >6 step (`:76882-76888`) over the
        // FULL 7 rows with the >= count wrap (`:76889-76892`); the
        // air pins row 4 rising (`:76894-76898`) / row 6 falling
        // (`:76899-76903`), both counter-zeroed.
        FindFrameExtra10Kind::PartyBunny => {
            if i.vy == 0.0 {
                // `:76866-76875`.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                // `:76876-76881`.
                if i.vx == 0.0 {
                    s.row = 0;
                    s.counter = 0.0;
                    return;
                }
                // `:76882-76888`.
                s.counter += i.vx.abs() * 1.0;
                s.counter += 1.0;
                if s.counter > 6.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
                // `:76889-76892`.
                if s.row >= kind.frame_count() {
                    s.row = 0;
                }
            } else if i.vy < 0.0 {
                // `:76894-76898`.
                s.counter = 0.0;
                s.row = 4;
            } else {
                // `:76899-76903`.
                s.counter = 0.0;
                s.row = 6;
            }
        }
        // `case 541` (`NPC.cs:69734-69772`): the Sand Elemental.
        // ATTACK (`ai[0] > 0`, `:69735-69758`): the row ladder —
        // < 6 → row 4, < 105 → the `(int)(ai[0]/8 % 4 + 5)` 5-8
        // cycle, < 114 → row 9, < 135 → the
        // `(int)((ai[0]-99-15)/7 + 10)` 10-12 climb, else row 1;
        // the counter is NEVER touched in the attack. WALK (else,
        // `:69759-69771`): the `frameCounter + (double)(|velocity|
        // * 0.1f) + 1.0` accumulator (`:69761` — the C# promotes
        // the f32 Length product to double for the add; the port
        // keeps the f32 counter, the wave-B precedent — at these
        // magnitudes the >= 8 cadence compares identically), the
        // >= 8 step over rows 0..3 with the >= 4 wrap
        // (`:69762-69770`). No sprite/rotation write.
        FindFrameExtra10Kind::SandElemental => {
            if i.ai0 > 0.0 {
                // `:69737-69757`.
                let n = i.ai0;
                if n < 6.0 {
                    s.row = 4;
                } else if n < 105.0 {
                    s.row = (n / 8.0 % 4.0 + 5.0) as i32 as u32;
                } else if n < 114.0 {
                    s.row = 9;
                } else if n < 135.0 {
                    s.row = ((n - 99.0 - 15.0) / 7.0 + 10.0) as i32 as u32;
                } else {
                    s.row = 1;
                }
            } else {
                // `:69761-69770`.
                s.counter += i.vx.hypot(i.vy) * 0.1 + 1.0;
                if s.counter >= 8.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
                if s.row >= 4 {
                    s.row = 0;
                }
            }
        }
        // `case 542-545` (`NPC.cs:74681-74692`): the sand sharks'
        // shared strip — `spriteDirection = direction`
        // UNCONDITIONALLY (0 included, `:74685`), the counter++
        // (`:74686`) with the >= 16 reset (`:74687-74690`), the row
        // `(int)(counter / 4)` (`:74691`) over the 4-row sheet. No
        // velocity read, no air split — the sharks swim behind the
        // tiles (`behindTiles`, `:15925`) at a fixed cadence.
        FindFrameExtra10Kind::SandShark
        | FindFrameExtra10Kind::SandsharkCorrupt
        | FindFrameExtra10Kind::SandsharkCrimson
        | FindFrameExtra10Kind::SandsharkHallow => {
            // `:74685`.
            s.sprite_dir = i.direction;
            // `:74686-74690`.
            s.counter += 1.0;
            if s.counter >= 16.0 {
                s.counter = 0.0;
            }
            // `:74691`.
            s.row = (s.counter / 4.0) as i32 as u32;
        }
        // `case 549` (`NPC.cs:67980-67992`): the Old One's Army lane
        // portal — `spriteDirection = 1` + `rotation = 0` every tick
        // (`:67981-67982`), the `++frameCounter >= 6` step
        // (`:67983-67991`) over the FULL 8 rows with the >= count
        // wrap. OUR `npc.rs` row-0 pin was WRONG (`npcFrameCount[549]`
        // = 8, `Main.cs:67138` slot 9) — the arm delegates here (the
        // 439-fix bridge shape).
        FindFrameExtra10Kind::DD2LanePortal => {
            // `:67981-67982`.
            s.sprite_dir = 1;
            s.rotation = 0.0;
            // `:67983-67991` — the ++ then the >= 6 check.
            s.counter += 1.0;
            if s.counter >= 6.0 {
                s.row += 1;
                s.counter = 0.0;
                if s.row >= kind.frame_count() {
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

    fn rows_over(kind: FindFrameExtra10Kind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra10(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The frame-count table agrees with the port's verbatim
    /// `NPC_FRAME_COUNT` (`Main.cs:67082` initializer cross-check).
    #[test]
    fn f27s10_frame_counts_match_the_port_table() {
        let table = crate::render::entities::NPC_FRAME_COUNT;
        for k in [
            FindFrameExtra10Kind::DesertScorpionWall,
            FindFrameExtra10Kind::DesertBeast,
            FindFrameExtra10Kind::DemonTaxCollector,
            FindFrameExtra10Kind::SlimeSpiked,
            FindFrameExtra10Kind::TheBride,
            FindFrameExtra10Kind::SandSlime,
            FindFrameExtra10Kind::SquirrelRed,
            FindFrameExtra10Kind::SquirrelGold,
            FindFrameExtra10Kind::PartyBunny,
            FindFrameExtra10Kind::SandElemental,
            FindFrameExtra10Kind::SandShark,
            FindFrameExtra10Kind::SandsharkCorrupt,
            FindFrameExtra10Kind::SandsharkCrimson,
            FindFrameExtra10Kind::SandsharkHallow,
            FindFrameExtra10Kind::DD2LanePortal,
        ] {
            assert_eq!(
                k.frame_count(),
                table[k.vanilla_type() as usize],
                "type {}",
                k.vanilla_type()
            );
        }
    }

    /// 531 DesertScorpionWall: the 0.4 stride (NOT the e4 group's
    /// 0.5) over the <6/<12/<18/<24 bands with the >=24 reset-tick
    /// row-HOLD. Expectations re-derived by mechanical replay of the
    /// C# transcription (f32 round-trip verified).
    #[test]
    fn f27s10_scorpion_wall_crawls_at_the_04_stride() {
        // vx 2: +0.8 a tick — row 1 at tick 8 (6.4), row 2 at tick 15
        // (12.0 — the <12 band ends exactly there).
        let rows = rows_over(FindFrameExtra10Kind::DesertScorpionWall, input(2.0, 0.0), 16);
        assert_eq!(rows, vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 2, 2]);
        // vx 1 + vy 1: the SAME +0.8 — the wall crawl has no air split.
        let rows = rows_over(FindFrameExtra10Kind::DesertScorpionWall, input(1.0, 1.0), 16);
        assert_eq!(rows, vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 2, 2]);
        // vx 5: +2 a tick — the bands 0/1/2/3 then the >=24 reset.
        let rows = rows_over(FindFrameExtra10Kind::DesertScorpionWall, input(5.0, 0.0), 12);
        assert_eq!(rows, vec![0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 3]);
        // The reset tick HOLDS the row (the C# writes no frame).
        let mut s = FrameState { row: 3, counter: 23.0, ..Default::default() };
        find_frame_extra10(FindFrameExtra10Kind::DesertScorpionWall, &mut s, &input(5.0, 0.0));
        assert_eq!((s.row, s.counter), (3, 0.0));
        // No sprite/rotation write.
        let mut s = FrameState { sprite_dir: -1, rotation: 0.5, ..Default::default() };
        find_frame_extra10(FindFrameExtra10Kind::DesertScorpionWall, &mut s, &input(2.0, 0.0));
        assert_eq!((s.sprite_dir, s.rotation), (-1, 0.5));
    }

    /// 532 DesertBeast: the antlion walk — the counter-HELD air/idle
    /// pins, the SIGN-GATED sprite copies, the 1+|vx|/2 lift-to-2
    /// stride with the >6 step and the full-8 wrap-to-2.
    /// Expectations re-derived by mechanical replay.
    #[test]
    fn f27s10_desert_beast_rides_the_antlion_walk() {
        // vx 2: +2 a tick — the >6 step every 4th tick.
        let rows = rows_over(FindFrameExtra10Kind::DesertBeast, input(2.0, 0.0), 20);
        assert_eq!(
            rows,
            vec![2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7]
        );
        // vx 1: +1.5 a tick — the >6 step every 5th tick.
        let rows = rows_over(FindFrameExtra10Kind::DesertBeast, input(1.0, 0.0), 20);
        assert_eq!(
            rows,
            vec![2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 6]
        );
        // The wrap: row 7 + counter 5 → the step lands 8, the >=8
        // wrap snaps 2.
        let mut s = FrameState { row: 7, counter: 5.0, ..Default::default() };
        find_frame_extra10(FindFrameExtra10Kind::DesertBeast, &mut s, &input(2.0, 0.0));
        assert_eq!((s.row, s.counter), (2, 0.0));
        // The air pin: row 1, the counter ALONE (held).
        let mut s = FrameState { row: 5, counter: 4.0, ..Default::default() };
        find_frame_extra10(FindFrameExtra10Kind::DesertBeast, &mut s, &input(2.0, -3.0));
        assert_eq!((s.row, s.counter), (1, 4.0));
        // The idle pin: row 0, the counter held.
        find_frame_extra10(FindFrameExtra10Kind::DesertBeast, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 4.0));
        // The SIGN-GATED sprite: direction -1 + vx -2 agrees → -1;
        // the mismatch (direction -1, vx +2) keeps the old sprite.
        let mut s = FrameState::default();
        let mut i = input(-2.0, 0.0);
        i.direction = -1;
        find_frame_extra10(FindFrameExtra10Kind::DesertBeast, &mut s, &i);
        assert_eq!(s.sprite_dir, -1);
        let mut i = input(2.0, 0.0);
        i.direction = -1;
        find_frame_extra10(FindFrameExtra10Kind::DesertBeast, &mut s, &i);
        assert_eq!(s.sprite_dir, -1, "the mismatch keeps the old sprite");
    }

    /// 534 DemonTaxCollector: the num108 = 12 stride (NOT the wave-B
    /// helper's >6) over the FULL 8 rows, the air row 1, the idle
    /// row 0, the ±1 flip. Expectations re-derived by mechanical
    /// replay.
    #[test]
    fn f27s10_demon_tax_collector_strides_at_the_12_threshold() {
        // vx 1: +3 a tick — the >12 step every 5th tick (the >6
        // helper would step every 3rd — the witness pins the 12).
        let rows = rows_over(FindFrameExtra10Kind::DemonTaxCollector, input(1.0, 0.0), 20);
        assert_eq!(
            rows,
            vec![2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 6]
        );
        // vx 3: +7 a tick — the >12 step every 2nd tick, the FULL-8
        // wrap back to 2 at row 8.
        let rows = rows_over(FindFrameExtra10Kind::DemonTaxCollector, input(3.0, 0.0), 20);
        assert_eq!(
            rows,
            vec![2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 2, 2, 3, 3, 4, 4, 5, 5, 6]
        );
        // The air tail: row 1 both rise and fall, the counter zeroed.
        let mut s = FrameState { row: 5, counter: 9.0, ..Default::default() };
        find_frame_extra10(FindFrameExtra10Kind::DemonTaxCollector, &mut s, &input(1.0, -2.0));
        assert_eq!((s.row, s.counter), (1, 0.0));
        find_frame_extra10(FindFrameExtra10Kind::DemonTaxCollector, &mut s, &input(1.0, 2.0));
        assert_eq!((s.row, s.counter), (1, 0.0));
        // The idle snap.
        find_frame_extra10(FindFrameExtra10Kind::DemonTaxCollector, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        // The grounded ±1 flip (0 leaves it).
        let mut s = FrameState::default();
        let mut i = input(1.0, 0.0);
        i.direction = -1;
        find_frame_extra10(FindFrameExtra10Kind::DemonTaxCollector, &mut s, &i);
        assert_eq!(s.sprite_dir, -1);
        i.direction = 0;
        find_frame_extra10(FindFrameExtra10Kind::DemonTaxCollector, &mut s, &i);
        assert_eq!(s.sprite_dir, -1, "direction 0 leaves the sprite");
    }

    /// 535 SlimeSpiked: the num2-fed 2-row strip — +1 idle / +2
    /// moving / +3 attack, the >=8 step, the >=2 wrap. Expectations
    /// re-derived by mechanical replay.
    #[test]
    fn f27s10_spiked_slime_rides_the_num2_fed_strip() {
        // Idle (+1): the step every 8th tick.
        let rows = rows_over(FindFrameExtra10Kind::SlimeSpiked, input(0.0, 0.0), 20);
        assert_eq!(
            rows,
            vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0]
        );
        // Moving (+2): the step every 4th tick.
        let rows = rows_over(FindFrameExtra10Kind::SlimeSpiked, input(1.0, 0.0), 20);
        assert_eq!(
            rows,
            vec![0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1]
        );
        // Attack (+3, aiAction 1 → num2 4): the step every 3rd tick
        // (the 8/3 cadence — 3, 6, 9 → the 3rd tick).
        let mut i = input(0.0, 0.0);
        i.ai_action = 1;
        let rows = rows_over(FindFrameExtra10Kind::SlimeSpiked, i, 20);
        assert_eq!(
            rows,
            vec![0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0]
        );
        // No sprite/rotation write (the 302/304 snap is the town
        // slimes').
        let mut s = FrameState { sprite_dir: -1, rotation: 0.5, ..Default::default() };
        find_frame_extra10(FindFrameExtra10Kind::SlimeSpiked, &mut s, &input(1.0, 0.0));
        assert_eq!((s.sprite_dir, s.rotation), (-1, 0.5));
    }

    /// 536 TheBride: the case-3 walker bands 0/1/2/1 with the >=32
    /// reset-tick row-HOLD, the air-OR-backpedal row-2 pin, the idle
    /// row-0 snap. Expectations re-derived by mechanical replay.
    #[test]
    fn f27s10_the_bride_rides_the_walker_bands() {
        // vx 2: +2 a tick — the bands 0(x4)/1(x4)/2(x4)/1(x4) then
        // the >=32 reset HOLDS row 1.
        let rows = rows_over(FindFrameExtra10Kind::TheBride, input(2.0, 0.0), 20);
        assert_eq!(
            rows,
            vec![0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 1, 0, 0, 0, 1]
        );
        // vx 4: +4 a tick — the 0/1/2/1 lap at 8 ticks.
        let rows = rows_over(FindFrameExtra10Kind::TheBride, input(4.0, 0.0), 20);
        assert_eq!(
            rows,
            vec![0, 1, 1, 2, 2, 1, 1, 1, 0, 1, 1, 2, 2, 1, 1, 1, 0, 1, 1, 2]
        );
        // The air pin: row 2 + counter zero (rise and fall alike).
        let mut s = FrameState { row: 1, counter: 9.0, ..Default::default() };
        find_frame_extra10(FindFrameExtra10Kind::TheBride, &mut s, &input(2.0, -3.0));
        assert_eq!((s.row, s.counter), (2, 0.0));
        // The backpedal pin: direction -1 + vx +2 → row 2.
        let mut s = FrameState::default();
        let mut i = input(2.0, 0.0);
        i.direction = -1;
        find_frame_extra10(FindFrameExtra10Kind::TheBride, &mut s, &i);
        assert_eq!((s.row, s.counter), (2, 0.0));
        // The idle snap.
        find_frame_extra10(FindFrameExtra10Kind::TheBride, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        // The grounded ±1 flip.
        let mut s = FrameState::default();
        let mut i = input(2.0, 0.0);
        i.direction = -1;
        find_frame_extra10(FindFrameExtra10Kind::TheBride, &mut s, &i);
        assert_eq!(s.sprite_dir, -1, "the flip precedes the backpedal pin");
    }

    /// 537 SandSlime: the SAME num2-fed strip over the 3-row sheet —
    /// the moving +2 cadence walks 0/1/2 with the >=3 wrap.
    /// Expectations re-derived by mechanical replay.
    #[test]
    fn f27s10_sand_slime_rides_the_three_row_strip() {
        let rows = rows_over(FindFrameExtra10Kind::SandSlime, input(1.0, 0.0), 24);
        assert_eq!(
            rows,
            vec![0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 0]
        );
        // The attack class (+3) over 3 rows.
        let mut i = input(0.0, 0.0);
        i.ai_action = 1;
        let rows = rows_over(FindFrameExtra10Kind::SandSlime, i, 12);
        assert_eq!(rows, vec![0, 0, 1, 1, 1, 2, 2, 2, 0, 0, 0, 1]);
    }

    /// 538/539 the squirrels: the |vx|*0.5+1 >6 walk over rows
    /// 0..count-2 wrapping at count-1 back to row **1** (NOT 0), the
    /// air row 2 rising / row 4 falling, the idle row-0 snap.
    /// Expectations re-derived by mechanical replay.
    #[test]
    fn f27s10_squirrels_walk_to_the_count_minus_one_wrap() {
        // vx 2: +2 a tick — the >6 step every 4th tick; row 5 (the
        // count-1 boundary) snaps back to 1.
        let rows = rows_over(FindFrameExtra10Kind::SquirrelRed, input(2.0, 0.0), 20);
        assert_eq!(
            rows,
            vec![0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 1]
        );
        // vx 1: +1.5 a tick — the >6 step every 5th tick.
        let rows = rows_over(FindFrameExtra10Kind::SquirrelGold, input(1.0, 0.0), 20);
        assert_eq!(
            rows,
            vec![0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4]
        );
        // The wrap tick: row 4 + counter 5 → the step lands 5, the
        // >= count-1 (5) wrap snaps 1.
        let mut s = FrameState { row: 4, counter: 5.0, ..Default::default() };
        find_frame_extra10(FindFrameExtra10Kind::SquirrelRed, &mut s, &input(2.0, 0.0));
        assert_eq!((s.row, s.counter), (1, 0.0));
        // The air pins: row 2 rising / row 4 falling, counter zeroed.
        let mut s = FrameState { row: 3, counter: 4.0, ..Default::default() };
        find_frame_extra10(FindFrameExtra10Kind::SquirrelRed, &mut s, &input(1.0, -2.0));
        assert_eq!((s.row, s.counter), (2, 0.0));
        find_frame_extra10(FindFrameExtra10Kind::SquirrelGold, &mut s, &input(1.0, 2.0));
        assert_eq!((s.row, s.counter), (4, 0.0));
        // The idle snap.
        find_frame_extra10(FindFrameExtra10Kind::SquirrelRed, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        // The grounded ±1 flip (0 leaves it).
        let mut s = FrameState::default();
        let mut i = input(1.0, 0.0);
        i.direction = -1;
        find_frame_extra10(FindFrameExtra10Kind::SquirrelRed, &mut s, &i);
        assert_eq!(s.sprite_dir, -1);
        i.direction = 0;
        find_frame_extra10(FindFrameExtra10Kind::SquirrelRed, &mut s, &i);
        assert_eq!(s.sprite_dir, -1, "direction 0 leaves the sprite");
    }

    /// 540 PartyBunny: the |vx|+1 >6 hop over the FULL 7 rows, the
    /// air pins row 4 rising / row 6 falling, the idle row-0 snap.
    /// Expectations re-derived by mechanical replay.
    #[test]
    fn f27s10_party_bunny_hops_the_full_seven_rows() {
        // vx 2: +3 a tick — the >6 step every 3rd tick.
        let rows = rows_over(FindFrameExtra10Kind::PartyBunny, input(2.0, 0.0), 20);
        assert_eq!(
            rows,
            vec![0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6]
        );
        // vx 1: +2 a tick — the >6 step every 4th tick; the full-7
        // wrap lands 0 at tick 25 (row 7 → 0).
        let rows = rows_over(FindFrameExtra10Kind::PartyBunny, input(1.0, 0.0), 24);
        assert_eq!(
            rows,
            vec![0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6]
        );
        let mut s = FrameState { row: 6, counter: 5.0, ..Default::default() };
        find_frame_extra10(FindFrameExtra10Kind::PartyBunny, &mut s, &input(2.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0), "the full-7 wrap");
        // The air pins: row 4 rising / row 6 falling.
        let mut s = FrameState { row: 2, counter: 4.0, ..Default::default() };
        find_frame_extra10(FindFrameExtra10Kind::PartyBunny, &mut s, &input(1.0, -2.0));
        assert_eq!((s.row, s.counter), (4, 0.0));
        find_frame_extra10(FindFrameExtra10Kind::PartyBunny, &mut s, &input(1.0, 2.0));
        assert_eq!((s.row, s.counter), (6, 0.0));
        // The idle snap.
        find_frame_extra10(FindFrameExtra10Kind::PartyBunny, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
    }

    /// 541 SandElemental: the ai[0] attack ladder (the counter
    /// UNTOUCHED) + the |v|*0.1+1 hover walk over rows 0..3.
    /// Expectations re-derived by mechanical replay.
    #[test]
    fn f27s10_sand_elemental_rides_the_ai0_ladder_and_the_hover_walk() {
        // The ladder: every band boundary pinned.
        for &(ai0, row) in &[
            (1.0, 4u32),
            (5.0, 4),
            (6.0, 5),
            (8.0, 6),
            (16.0, 7),
            (32.0, 5),
            (40.0, 6),
            (64.0, 5),
            (96.0, 5),
            (104.0, 6),
            (105.0, 9),
            (113.0, 9),
            (114.0, 10),
            (120.0, 10),
            (121.0, 11),
            (128.0, 12),
            (134.0, 12),
            (135.0, 1),
            (200.0, 1),
        ] {
            let mut s = FrameState { row: 3, counter: 7.0, ..Default::default() };
            let mut i = input(0.0, 0.0);
            i.ai0 = ai0;
            find_frame_extra10(FindFrameExtra10Kind::SandElemental, &mut s, &i);
            assert_eq!(s.row, row, "ai[0] = {ai0}");
            assert_eq!(s.counter, 7.0, "the attack never touches the counter");
        }
        // The hover walk: |v| = 5 → +1.5 a tick, the >=8 step every
        // 6th tick (1.5*6 = 9), rows 0..3 wrapping at 4.
        let rows = rows_over(FindFrameExtra10Kind::SandElemental, input(3.0, 4.0), 20);
        assert_eq!(
            rows,
            vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3]
        );
        // The still hover: +1 a tick, the step every 8th tick.
        let rows = rows_over(FindFrameExtra10Kind::SandElemental, input(0.0, 0.0), 20);
        assert_eq!(
            rows,
            vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2]
        );
        // The walk wrap: row 3 + counter 7 → the step lands 4, the
        // >=4 wrap snaps 0.
        let mut s = FrameState { row: 3, counter: 7.0, ..Default::default() };
        find_frame_extra10(FindFrameExtra10Kind::SandElemental, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        // No sprite/rotation write.
        let mut s = FrameState { sprite_dir: -1, rotation: 0.5, ..Default::default() };
        find_frame_extra10(FindFrameExtra10Kind::SandElemental, &mut s, &input(3.0, 4.0));
        assert_eq!((s.sprite_dir, s.rotation), (-1, 0.5));
    }

    /// 542-545 the sand sharks: the UNCONDITIONAL direction copy +
    /// the 16-tick 4-band strip (the counter resets at >=16 BEFORE
    /// the row read, so the reset tick shows row 0). Expectations
    /// re-derived by mechanical replay.
    #[test]
    fn f27s10_sand_sharks_swim_the_sixteen_tick_strip() {
        let rows = rows_over(FindFrameExtra10Kind::SandShark, input(0.0, 0.0), 20);
        assert_eq!(
            rows,
            vec![0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 0, 0, 0, 0, 1]
        );
        // All four sharks ride the SAME machine (the shared case).
        for k in [
            FindFrameExtra10Kind::SandsharkCorrupt,
            FindFrameExtra10Kind::SandsharkCrimson,
            FindFrameExtra10Kind::SandsharkHallow,
        ] {
            let rows = rows_over(k, input(3.0, -2.0), 20);
            assert_eq!(
                rows,
                vec![0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 0, 0, 0, 0, 1],
                "type {}",
                k.vanilla_type()
            );
        }
        // The UNCONDITIONAL direction copy (0 included).
        for d in [-1i32, 0, 1] {
            let mut s = FrameState { sprite_dir: 1, ..Default::default() };
            let mut i = input(0.0, 0.0);
            i.direction = d;
            find_frame_extra10(FindFrameExtra10Kind::SandShark, &mut s, &i);
            assert_eq!(s.sprite_dir, d, "direction {d}");
        }
        // No rotation write.
        let mut s = FrameState { rotation: 0.5, ..Default::default() };
        find_frame_extra10(FindFrameExtra10Kind::SandShark, &mut s, &input(0.0, 0.0));
        assert_eq!(s.rotation, 0.5);
    }

    /// 549 DD2LanePortal: the sprite=1 + rotation=0 snap every tick +
    /// the ++counter >=6 step over the FULL 8 rows. Expectations
    /// re-derived by mechanical replay.
    #[test]
    fn f27s10_lane_portal_steps_the_full_eight_rows() {
        let rows = rows_over(FindFrameExtra10Kind::DD2LanePortal, input(0.0, 0.0), 20);
        assert_eq!(
            rows,
            vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3]
        );
        // The sprite/rotation snap fires EVERY tick (even the hold
        // ticks), the counter untouched by the snap.
        let mut s = FrameState {
            row: 3,
            counter: 2.0,
            sprite_dir: -1,
            rotation: 0.7,
        };
        find_frame_extra10(FindFrameExtra10Kind::DD2LanePortal, &mut s, &input(5.0, -3.0));
        assert_eq!((s.row, s.counter), (3, 3.0));
        assert_eq!((s.sprite_dir, s.rotation), (1, 0.0));
        // The wrap: row 7 + counter 5 → the step lands 8, the >=8
        // wrap snaps 0.
        let mut s = FrameState { row: 7, counter: 5.0, ..Default::default() };
        find_frame_extra10(FindFrameExtra10Kind::DD2LanePortal, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
    }

    /// The 15-kind strip sweep: a 5x3x3 vx/vy/direction input grid
    /// (plus the aiAction/ai0 variants), 40 ticks each — every row
    /// stays inside its sheet (the extra-9 precedent).
    #[test]
    fn f27s10_fifteen_kind_strip_sweep_stays_inside_the_sheets() {
        let kinds = [
            FindFrameExtra10Kind::DesertScorpionWall,
            FindFrameExtra10Kind::DesertBeast,
            FindFrameExtra10Kind::DemonTaxCollector,
            FindFrameExtra10Kind::SlimeSpiked,
            FindFrameExtra10Kind::TheBride,
            FindFrameExtra10Kind::SandSlime,
            FindFrameExtra10Kind::SquirrelRed,
            FindFrameExtra10Kind::SquirrelGold,
            FindFrameExtra10Kind::PartyBunny,
            FindFrameExtra10Kind::SandElemental,
            FindFrameExtra10Kind::SandShark,
            FindFrameExtra10Kind::SandsharkCorrupt,
            FindFrameExtra10Kind::SandsharkCrimson,
            FindFrameExtra10Kind::SandsharkHallow,
            FindFrameExtra10Kind::DD2LanePortal,
        ];
        for k in kinds {
            let count = k.frame_count();
            for vx in [-4.0f32, -1.0, 0.0, 1.0, 4.0] {
                for vy in [-3.0f32, 0.0, 3.0] {
                    for direction in [-1i32, 0, 1] {
                        for ai_action in [0i32, 1] {
                            for ai0 in [0.0f32, 50.0, 110.0, 130.0] {
                                let i = FrameInput {
                                    vx,
                                    vy,
                                    direction,
                                    ai0,
                                    ai_action,
                                };
                                let mut s = FrameState::default();
                                for tick in 0..40 {
                                    find_frame_extra10(k, &mut s, &i);
                                    assert!(
                                        s.row < count,
                                        "type {} tick {tick} row {} >= {count} (vx {vx}, vy {vy}, dir {direction})",
                                        k.vanilla_type(),
                                        s.row
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// The seam maps exactly the slice's vanilla types (the full
    /// 15-id set from batch 1 — the e8 precedent; the batch-2/3 arms
    /// land behind the catch-all, the seam is stable).
    #[test]
    fn f27s10_seam_covers_exactly_the_slice() {
        for t in 0..697i16 {
            let k = find_frame_kind_extra10(t);
            let claimed = [
                531, 532, 534, 535, 536, 537, 538, 539, 540, 541, 542, 543, 544,
                545, 549,
            ];
            assert_eq!(k.is_some(), claimed.contains(&t), "type {t}");
            if let Some(k) = k {
                assert_eq!(k.vanilla_type(), t);
            }
        }
    }
}
