//! W4-f27 — FindFrame extra wave 8 (slice 8): the FIRST 15 unclaimed
//! missing case ids of `NPC.FindFrame` (`NPC.cs:66684-77370`) ascending
//! from 443 (the census claim is the ANALYSIS "W4-f27 FindFrame extra
//! wave 8" section — 200 unclaimed after the d44-d49/e13/e14/f26/f27
//! claims; the f26d co-lane keeps the 297-304 + 351-365 band). The 15:
//! **443/444/445/446/447/448/449/450/451/452/453/454/460/461/462** —
//! the six gold critters, the four bone-throwing skeletons, the
//! Skeleton Merchant, the Cultist Dragon Head, the Butcher, the
//! Creature From The Deep and Fritz (`ID/NPCID.cs:11559-11599`).
//! Landed 5 kinds per commit.
//!
//! Shape: the kinds are NOT in this branch's `NpcKind` (the pushed
//! roster lane `W4-d40-npc-kinds-421-490` owns them — its find_frame
//! arms for 443-448 are ROW-0 STUBS and its 453/460/462 arms ride the
//! `humanoid_town_frame` stand-in), so — like `npc_findframe_a.rs` /
//! `npc_findframe_b.rs` (S1-D-3 waves A/B) and the extra-4/extra-7
//! waves — the frame machines land first as pure functions over a
//! small [`FrameState`]/[`FrameInput`] pair. The wiring seam is
//! [`find_frame_kind_extra8`]: when the roster merges, each kind's
//! `find_frame` arm copies `row`/`counter`/`sprite_dir`/`rotation`
//! out, runs one tick and copies them back (the
//! `Npc::find_frame_wave_a` bridge precedent). No arm draws
//! `Main.rand` — verified mechanically per cited range (the humanoid
//! group's town pose zones with their chat-bubble draws are the
//! documented skip) — so the wiring cannot move a stream.
//!
//! Row convention: the port tracks frame ROW indices, not pixels —
//! the C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer),
//! counted entry-by-entry in [`FindFrameExtra8Kind::frame_count`] and
//! cross-checked against the port's own `render/entities.rs::
//! NPC_FRAME_COUNT` table.
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "W4-f27 FindFrame
//! extra wave 8"):
//! * the grasshopper group's `IsABestiaryIconDummy` branch
//!   (`NPC.cs:70960-70972`) — a display-only dummy path, the port has
//!   no bestiary-dummy NPC entity (the extra-7 439/440 precedent);
//! * the butterfly group's type-356-only colour bank
//!   (`NPC.cs:72768-72771` — the `type != 444 && != 653 && != 661`
//!   gate EXCLUDES 444, the 3-row sheet has one bank);
//! * the worm group's tile-slope probe (`NPC.cs:72807-72826`) — the
//!   resolved slope rides [`FrameInput::slope`] (the d48 wave-C
//!   precedent, `npc_findframe_c.rs`'s caller-resolved
//!   `Framing.GetTileSafely` read; the bridge supplies it);
//! * the humanoid group's town `ai[0]` sit/talk/attack pose zones
//!   (`NPC.cs:68662-69488`) — 460/462 are aiStyle-3 fighters
//!   (`NPC.cs:15498`/`:15524`, no `townNPC`), the wave-B
//!   `humanoid_fighter_frame` keep-one precedent; 453 IS
//!   `isLikeATownNPC` (`NPC.cs:6860` — the hard-coded type gate), so
//!   its walk wraps at count − `ExtraFramesCount[453]` = 26 − 9 = 17
//!   (`NPCID.cs:10464`), the extra-3 Cyborg precedent.

// The wiring is a later round (the roster wave owns the kinds) —
// the `npc_findframe_a.rs`/`npc_findframe_b.rs` precedent.
#![allow(dead_code)]

/// The extra-wave-8 kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra8Kind {
    /// `NPCID.GoldBunny` = 443 (`NPCID.cs:11559`) — the bunny hop
    /// group (`NPC.cs:76855`, shared with 46/47/303/337/464/540/
    /// 614/646-652).
    GoldBunny,
    /// `NPCID.GoldButterfly` = 444 (`NPCID.cs:11561`) — the butterfly
    /// flap group (`NPC.cs:72740`, shared with 356/653/661).
    GoldButterfly,
    /// `NPCID.GoldFrog` = 445 (`NPCID.cs:11563`) — the frog group
    /// (`NPC.cs:76619`, shared with 361/687).
    GoldFrog,
    /// `NPCID.GoldGrasshopper` = 446 (`NPCID.cs:11565`) — the
    /// grasshopper group (`NPC.cs:70959`, shared with 377).
    GoldGrasshopper,
    /// `NPCID.GoldMouse` = 447 (`NPCID.cs:11567`) — the mouse group
    /// (`NPC.cs:76514`, shared with 300).
    GoldMouse,
    /// `NPCID.GoldWorm` = 448 (`NPCID.cs:11569`) — the worm group
    /// (`NPC.cs:72775`, shared with 357/484/606).
    GoldWorm,
    /// `NPCID.BoneThrowingSkeleton` = 449 (`NPCID.cs:11571`) —
    /// `case 449-452` (`NPC.cs:71398-71453`), the bone-toss ladder.
    BoneThrowingSkeleton,
    /// `NPCID.BoneThrowingSkeleton2` = 450 (`NPCID.cs:11573`) — the
    /// same group (`NPC.cs:71399`).
    BoneThrowingSkeleton2,
    /// `NPCID.BoneThrowingSkeleton3` = 451 (`NPCID.cs:11575`) — the
    /// same group (`NPC.cs:71400`).
    BoneThrowingSkeleton3,
    /// `NPCID.BoneThrowingSkeleton4` = 452 (`NPCID.cs:11577`) — the
    /// same group (`NPC.cs:71401`), the ONE 20-row sheet of the four.
    BoneThrowingSkeleton4,
    /// `NPCID.SkeletonMerchant` = 453 (`NPCID.cs:11579`) — the
    /// humanoid-group TOWN member (`NPC.cs:68610`), the hard-coded
    /// `isLikeATownNPC` gate (`NPC.cs:6860`).
    SkeletonMerchant,
    /// `NPCID.CultistDragonHead` = 454 (`NPCID.cs:11581`) —
    /// `case 454` (`NPC.cs:70952-70957`), the counter/2 row.
    CultistDragonHead,
    /// `NPCID.Butcher` = 460 (`NPCID.cs:11593`) — the humanoid-group
    /// fighter (`NPC.cs:68611`, aiStyle 3 `NPC.cs:15498`).
    Butcher,
    /// `NPCID.CreatureFromTheDeep` = 461 (`NPCID.cs:11595`) —
    /// `case 461` (`NPC.cs:75315-75410`), the wet swimmer.
    CreatureFromTheDeep,
    /// `NPCID.Fritz` = 462 (`NPCID.cs:11597`) — the humanoid-group
    /// fighter (`NPC.cs:68612`, aiStyle 3 `NPC.cs:15524`), the ONE
    /// 9-row sheet with the `num108 = 9` stride + the air rows 1/2.
    Fritz,
}

impl FindFrameExtra8Kind {
    /// The vanilla `NPC.type` — the `FindFrame` switch key.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::GoldBunny => 443,
            Self::GoldButterfly => 444,
            Self::GoldFrog => 445,
            Self::GoldGrasshopper => 446,
            Self::GoldMouse => 447,
            Self::GoldWorm => 448,
            Self::BoneThrowingSkeleton => 449,
            Self::BoneThrowingSkeleton2 => 450,
            Self::BoneThrowingSkeleton3 => 451,
            Self::BoneThrowingSkeleton4 => 452,
            Self::SkeletonMerchant => 453,
            Self::CultistDragonHead => 454,
            Self::Butcher => 460,
            Self::CreatureFromTheDeep => 461,
            Self::Fritz => 462,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// entry `type` on line `67084 + type/10`, slot `type % 10`) —
    /// cross-checked against `render/entities.rs::NPC_FRAME_COUNT`.
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67128 (entries 443-448): 7, 3, 13, 2, 6, 2.
            Self::GoldBunny => 7,
            Self::GoldButterfly => 3,
            Self::GoldFrog => 13,
            Self::GoldGrasshopper => 2,
            Self::GoldMouse => 6,
            Self::GoldWorm => 2,
            // Main.cs:67128 (449): 19; :67129 (450-454): 19, 19, 20, 26, 3.
            Self::BoneThrowingSkeleton => 19,
            Self::BoneThrowingSkeleton2 => 19,
            Self::BoneThrowingSkeleton3 => 19,
            Self::BoneThrowingSkeleton4 => 20,
            Self::SkeletonMerchant => 26,
            Self::CultistDragonHead => 3,
            // Main.cs:67130 (460/461/462): 16, 21, 9.
            Self::Butcher => 16,
            Self::CreatureFromTheDeep => 21,
            Self::Fritz => 9,
        }
    }
}

/// The wiring seam: the vanilla type → this module's kind (the
/// `find_frame_kind_extra7` precedent). `None` outside the slice.
pub fn find_frame_kind_extra8(vanilla_type: i16) -> Option<FindFrameExtra8Kind> {
    Some(match vanilla_type {
        443 => FindFrameExtra8Kind::GoldBunny,
        444 => FindFrameExtra8Kind::GoldButterfly,
        445 => FindFrameExtra8Kind::GoldFrog,
        446 => FindFrameExtra8Kind::GoldGrasshopper,
        447 => FindFrameExtra8Kind::GoldMouse,
        448 => FindFrameExtra8Kind::GoldWorm,
        449 => FindFrameExtra8Kind::BoneThrowingSkeleton,
        450 => FindFrameExtra8Kind::BoneThrowingSkeleton2,
        451 => FindFrameExtra8Kind::BoneThrowingSkeleton3,
        452 => FindFrameExtra8Kind::BoneThrowingSkeleton4,
        453 => FindFrameExtra8Kind::SkeletonMerchant,
        454 => FindFrameExtra8Kind::CultistDragonHead,
        460 => FindFrameExtra8Kind::Butcher,
        461 => FindFrameExtra8Kind::CreatureFromTheDeep,
        462 => FindFrameExtra8Kind::Fritz,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection`,
/// `rotation`, `localAI[0]`. The wiring step copies them to/from
/// `Npc`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter`.
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — the butterfly/frog(never)/worm/swimmer arms
    /// write it; the bunny/grasshopper/mouse/bone/humanoid arms never
    /// touch it.
    pub rotation: f32,
    /// `NPC.localAI[0]` — only the worm arm (448) writes it: the
    /// `-2f` pin each tick, lifted to 0 on a slope
    /// (`NPC.cs:72779`/`:72819`/`:72824`).
    pub local_ai0: f32,
}

/// The per-tick inputs the arms read: `velocity`, `direction`, `wet`,
/// the `ai` slots and the worm's resolved tile slope. `ai_action` is
/// not carried — none of these arms reads the `num2` motion class
/// (`NPC.cs:66696-66704`).
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone in
    /// the arms that gate on `direction == 1` / `== -1`, but the
    /// butterfly/frog/grasshopper arms' unconditional copy writes it
    /// verbatim; 461's wet branch WRITES direction itself from the
    /// vx sign — the input is the pre-tick value).
    pub direction: i32,
    pub wet: bool,
    pub ai: [f32; 4],
    /// The worm arm's resolved tile slope (`NPC.cs:72807-72826`, the
    /// d48 wave-C `npc_findframe_c.rs` precedent): the caller probes
    /// `Framing.GetTileSafely((int)Center.X/16, (int)position.Y/16)`
    /// and, when that tile's `slope() == 0`, the tile one row BELOW;
    /// this is the resulting `slope()` (0 none, 1 → +0.785 rad,
    /// 2 → −0.785, both lifting `localAI[0]` to 0).
    pub slope: i32,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self {
            vx: 0.0,
            vy: 0.0,
            direction: 1,
            wet: false,
            ai: [0.0; 4],
            slope: 0,
        }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-8 kinds — one
/// tick.
pub fn find_frame_extra8(kind: FindFrameExtra8Kind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // `case 443` in the bunny hop group (`NPC.cs:76851-76904`,
        // shared with 46/47/303/337/464/540/614/646-652 — d44's
        // 46/47 and extra-7's 337 are the same machine, keep-one at
        // the merge): the grounded ±1 flip (0 leaves it), the idle
        // snaps row 0 + zeroes the counter (`:76876-76881`), the walk
        // accumulates `|vx| * 1` PLUS 1 (`:76882-76883`), the > 6
        // step walks the FULL 7 rows with the >= count wrap to 0
        // (`:76884-76892`); the air pins row 4 rising / row 6
        // falling, both zeroing the counter (`:76894-76903`).
        FindFrameExtra8Kind::GoldBunny => {
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
                    return;
                }
                // `:76882-76892`.
                s.counter += i.vx.abs();
                s.counter += 1.0;
                if s.counter > 6.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
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
        // `case 444` in the butterfly flap group (`NPC.cs:72739-72773`,
        // shared with 356/653/661 — d48 wave-C's 356 is the same
        // machine, keep-one at the merge): the `vx * 0.3` tilt
        // (`:72745`), the UNCONDITIONAL direction copy (`:72746`),
        // the velocity-fed counter `+1 + (|vx| + |vy|) / 2`
        // (`:72747`), the 7-tick bands rows 0/1/2/1 with the reset at
        // `7*4 - 1 = 27` (`:72748-72767`). The type-356-only colour
        // bank (`:72768-72771`) EXCLUDES 444 — the 3-row sheet has
        // one bank, no offset.
        FindFrameExtra8Kind::GoldButterfly => {
            // `:72745-72747`.
            s.rotation = i.vx * 0.3;
            s.sprite_dir = i.direction;
            s.counter += 1.0 + (i.vx.abs() + i.vy.abs()) / 2.0;
            // `:72748-72767` — num143 = 7.
            if s.counter < 7.0 {
                s.row = 0;
            } else if s.counter < 14.0 {
                s.row = 1;
            } else if s.counter < 21.0 {
                s.row = 2;
            } else {
                s.row = 1;
                if s.counter >= 27.0 {
                    s.counter = 0.0;
                }
            }
        }
        // `case 445` in the frog group (`NPC.cs:76618-76685`, shared
        // with 361/687 — d48 wave-C's 361 is the same machine,
        // keep-one at the merge): the UNCONDITIONAL direction copy
        // (`:76621`). Wet: the counter zeroes and the |vx| bands
        // pick rows 10/11/12 (the 0.25/0.15 deadbands,
        // `:76622-76637`). Grounded idle: the > 6 tick step over rows
        // 0..5 (the `> num*5` wrap runs EVERY tick, not only on the
        // step, `:76640-76653`). Grounded walk: the 6-tick bands
        // 0/6/8/9 with the reset at `6*4 - 1 = 23`
        // (`:76654-76676`). Air: the fall row 9 / rise row 8
        // (`:76677-76684`). No rotation write.
        FindFrameExtra8Kind::GoldFrog => {
            // `:76621`.
            s.sprite_dir = i.direction;
            if i.wet {
                // `:76622-76637`.
                s.counter = 0.0;
                if i.vx > 0.25 || i.vx < -0.25 {
                    s.row = 10;
                } else if i.vx > 0.15 || i.vx < -0.15 {
                    s.row = 11;
                } else {
                    s.row = 12;
                }
            } else if i.vy == 0.0 {
                if i.vx == 0.0 {
                    // `:76640-76653`.
                    s.counter += 1.0;
                    if s.counter > 6.0 {
                        s.counter = 0.0;
                        s.row += 1;
                    }
                    if s.row > 5 {
                        s.row = 0;
                    }
                } else {
                    // `:76654-76676` — num196 = 6.
                    s.counter += 1.0;
                    if s.counter < 6.0 {
                        s.row = 0;
                    } else if s.counter < 12.0 {
                        s.row = 6;
                    } else if s.counter < 18.0 {
                        s.row = 8;
                    } else {
                        s.row = 9;
                        if s.counter >= 23.0 {
                            s.counter = 0.0;
                        }
                    }
                }
            } else if i.vy > 0.0 {
                // `:76677-76680`.
                s.row = 9;
            } else {
                // `:76681-76684`.
                s.row = 8;
            }
        }
        // `case 446` in the grasshopper group (`NPC.cs:70958-70982`,
        // shared with 377 — d49 wave-D's 377 is the same machine,
        // keep-one at the merge): the airborne tick pins row 1, the
        // grounded tick pins row 0 (`:70973-70980`), the
        // UNCONDITIONAL direction copy runs on BOTH paths
        // (`:70981`). The counter is NEVER touched. The
        // `IsABestiaryIconDummy` branch (`:70960-70972`) is the
        // display-only skip (the header). No rotation write.
        FindFrameExtra8Kind::GoldGrasshopper => {
            if i.vy != 0.0 {
                // `:70973-70976`.
                s.row = 1;
            } else {
                // `:70977-70980`.
                s.row = 0;
            }
            // `:70981`.
            s.sprite_dir = i.direction;
        }
        // `case 447` in the mouse group (`NPC.cs:76513-76565`, shared
        // with 300 — f26d's 300 is the same machine, keep-one at the
        // merge): the grounded ±1 flip (`:76517-76524`); the
        // grounded idle ticks the counter and the > 8 step walks
        // rows 0/1 with the `> num*1` wrap to 0 (`:76525-76538`);
        // the walk (grounded OR air — the two tails are
        // byte-identical, `:76539-76549`/`:76551-76564`) accumulates
        // `|vx| * 0.5` PLUS 1, the > 6 step walks the FULL 6 rows
        // with the >= count wrap to row 2 (rows 0/1 are idle-only on
        // the walk). The air branch does NOT flip the sprite. No
        // rotation write.
        FindFrameExtra8Kind::GoldMouse => {
            if i.vy == 0.0 {
                // `:76517-76524`.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                if i.vx == 0.0 {
                    // `:76525-76538`.
                    s.counter += 1.0;
                    if s.counter > 8.0 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                    if s.row > 1 {
                        s.row = 0;
                    }
                    return;
                }
            }
            // `:76539-76549` (grounded) / `:76551-76564` (air) — the
            // same machine.
            s.counter += i.vx.abs() * 0.5;
            s.counter += 1.0;
            if s.counter > 6.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= kind.frame_count() {
                s.row = 2;
            }
        }
        // `case 448` in the worm group (`NPC.cs:72774-72828`, shared
        // with 357/484/606 — d48 wave-C's 357 is the same machine,
        // keep-one at the merge): the `localAI[0] = -2` pin EVERY
        // tick (`:72779`); the grounded tick zeroes the rotation, the
        // idle pins row 1 + zeroes the counter (`:72783-72787`), the
        // crawl's > 12 step alternates rows 1/0 (the `> num` wrap,
        // `:72788-72800`); the airborne tick pins row 1 and
        // accumulates the `direction * 0.1` spin (`:72802-72806`).
        // The slope override (`:72807-72826`) rides the caller-
        // resolved [`FrameInput::slope`] (the d48 precedent): slope 1
        // → +0.785 rad, slope 2 → −0.785, both lifting `localAI[0]`
        // to 0.
        FindFrameExtra8Kind::GoldWorm => {
            // `:72779`.
            s.local_ai0 = -2.0;
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
            // `:72816-72825` — the resolved slope override.
            if i.slope == 1 {
                s.rotation = 0.785;
                s.local_ai0 = 0.0;
            } else if i.slope == 2 {
                s.rotation = -0.785;
                s.local_ai0 = 0.0;
            }
        }
        // `case 449/450/451/452` (`NPC.cs:71398-71453`): the
        // bone-throwing skeletons' toss ladder. The grounded ±1 flip
        // (`:71404-71411`); the `ai[2] > 0` toss pose copies
        // `direction` UNCONDITIONALLY (0 included, `:71414`) and
        // picks row `count - 5 + band` off the `ai[1]` wind-up
        // countdown — >= 50 band 1, >= 43 band 2, >= 36 band 3,
        // >= 29 band 4, else 0 (`:71415-71420` — the C#'s
        // `50 - num55 * k` with num55 = 7), zeroing the counter and
        // BREAKING. The idle snaps row 0 with the counter UNTOUCHED
        // (`:71424-71428`). The walk lifts any row < 2 to 2 (the
        // doubled `if` at `:71429-71436` is idempotent — carried
        // once), accumulates `|vx| * 2` with NO +1 (`:71437` — unlike
        // the humanoid walkers), the > 6 step, and the wrap at the
        // LITERAL 15 back to row 2 (`:71443-71446` — rows 15..19 are
        // the toss poses on the 19-row sheets; 452's 20-row sheet
        // adds one more toss row, the walk band is the SAME). The
        // air zeroes the counter and pins row 1 (`:71448-71452`).
        FindFrameExtra8Kind::BoneThrowingSkeleton
        | FindFrameExtra8Kind::BoneThrowingSkeleton2
        | FindFrameExtra8Kind::BoneThrowingSkeleton3
        | FindFrameExtra8Kind::BoneThrowingSkeleton4 => {
            if i.vy == 0.0 {
                // `:71404-71411`.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                if i.ai[2] > 0.0 {
                    // `:71412-71422` — num53 = count - 5, num55 = 7.
                    s.sprite_dir = i.direction;
                    let band = if i.ai[1] >= 50.0 {
                        1
                    } else if i.ai[1] >= 43.0 {
                        2
                    } else if i.ai[1] >= 36.0 {
                        3
                    } else if i.ai[1] >= 29.0 {
                        4
                    } else {
                        0
                    };
                    s.row = kind.frame_count() - 5 + band;
                    s.counter = 0.0;
                    return;
                }
                if i.vx == 0.0 {
                    // `:71424-71428` — the counter is NOT zeroed.
                    s.row = 0;
                    return;
                }
                // `:71429-71446` — the doubled lift is idempotent.
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
            } else {
                // `:71448-71452`.
                s.counter = 0.0;
                s.row = 1;
            }
        }
        // `case 453` in the humanoid-group case list
        // (`NPC.cs:68610`) → the shared TOWN walk
        // (`NPC.cs:68647-69655`). The Skeleton Merchant IS
        // `isLikeATownNPC` — the hard-coded type gate
        // (`NPC.cs:6860-6863`, NOT the townNPC flag; his SetDefaults
        // row `:14399-14413` sets aiStyle 7, no townNPC), so
        // `num57` = `ExtraFramesCount[453]` = 9 (`NPCID.cs:10464`)
        // and the walk band wraps at row 26 − 9 = 17 (rows 2..16 —
        // the extra-3 Cyborg precedent). He is not a town slime
        // (`num108` = 6, `num109` = row 2), not the 140/489
        // idle-row-1 pair (idle snaps to row 0), and NOT in the
        // `:69647` air pin list (the air holds row 1). The town
        // `ai[0]` sit/talk/attack pose zones (`:68662-69488`, with
        // their chat-bubble `Main.rand` draws) are the documented
        // skip — the `npc.rs::humanoid_town_frame` / extra-3 Cyborg
        // precedent.
        FindFrameExtra8Kind::SkeletonMerchant => {
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
                    if s.row < 2 {
                        s.row = 2;
                    }
                    s.counter += i.vx.abs() * 2.0;
                    s.counter += 1.0;
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
                // `:69642-69646` — the group's air row 1 (453 is not
                // in the `:69647` pin list).
                s.counter = 0.0;
                s.row = 1;
            }
        }
        // `case 454` (`NPC.cs:70952-70957`): the Cultist Dragon
        // Head's row is `(int)(frameCounter / 2)` — the counter is
        // NEVER touched here (the aiStyle-6 chain machine owns it,
        // `NPC.cs:18766-18822`'s segment tick); the 3-row sheet's
        // rows 0/1/2 ride counters 0-1/2-3/4-5. No sprite/rotation
        // write. (The bodies/tail 455-459 have NO FindFrame case —
        // npcFrameCount 1, out of this slice.)
        FindFrameExtra8Kind::CultistDragonHead => {
            // `:70954-70955`.
            s.row = (s.counter / 2.0) as i32 as u32;
        }
        // `case 460` in the humanoid-group case list
        // (`NPC.cs:68611`) → the shared fighter walk
        // (`NPC.cs:68647-69655`) via the wave-B keep-one helper (the
        // extra-7 costume-skeleton delegate precedent). The Butcher
        // is NOT `isLikeATownNPC` (his SetDefaults row
        // `:15494-15506` sets aiStyle 3, no townNPC), so `num57` = 0
        // and the walk wraps at the FULL 16 rows; not a town slime
        // (`num108` = 6, `num109` = row 2), not the 140/489
        // idle-row-1 pair, NOT in the `:69647` air pin list (the air
        // holds row 1). The town pose zones are the documented skip.
        FindFrameExtra8Kind::Butcher => {
            let bi = crate::npc_findframe_b::FrameInput {
                vx: i.vx,
                vy: i.vy,
                direction: i.direction,
                wet: i.wet,
                ai: i.ai,
            };
            let mut bs = crate::npc_findframe_b::FrameState {
                row: s.row,
                counter: s.counter,
                sprite_dir: s.sprite_dir,
                rotation: s.rotation,
            };
            crate::npc_findframe_b::humanoid_fighter_frame(
                &mut bs,
                &bi,
                kind.frame_count(),
                false,
                false,
            );
            s.row = bs.row;
            s.counter = bs.counter;
            s.sprite_dir = bs.sprite_dir;
            s.rotation = bs.rotation;
        }
        // `case 461` (`NPC.cs:75315-75410`): the Creature From The
        // Deep's wet/dry split. WET (`:75316-75360`): the vx sign
        // WRITES direction (0 leaves it), a facing change mirrors the
        // rotation and copies the flip; the aim is
        // `atan2(vy * direction, vx * direction)` with the ±6.28
        // wrap-around when the gap >= 3.14 (the literal), then the
        // `(rotation * 4 + aim) / 5` ease; the counter accumulates
        // `|velocity| + 1`, the > 8 step walks rows 16..20 with the
        // `> 20` wrap to 16 and the `< 16` lift to 19 (the entry
        // quirk — a swimmer starting at row 0 lands 19, not 16).
        // DRY (`:75361-75409`): the rotation decays — the > 3.14
        // wrap first, then the ±0.01 deadband snaps 0 ELSE the ×0.9
        // ease; the grounded ±1 flip, the idle snaps row 0 + zeroes
        // the counter, the walk lifts to 2 + `|vx| * 2 + 1` with the
        // > 6 step and the `> 15` wrap to 2 (rows 16..20 are the
        // swim band on the 21-row sheet); the air zeroes the counter
        // and pins row 1.
        FindFrameExtra8Kind::CreatureFromTheDeep => {
            if i.wet {
                // `:75318-75325` — the vx sign writes direction.
                let mut direction = i.direction;
                if i.vx < 0.0 {
                    direction = -1;
                }
                if i.vx > 0.0 {
                    direction = 1;
                }
                // `:75326-75330`.
                if s.sprite_dir != direction {
                    s.rotation *= -1.0;
                    s.sprite_dir = direction;
                }
                // `:75331-75343` — the aim + wrap + ease (the
                // literals 3.14/6.28, not PI/TAU).
                let aim = (i.vy * direction as f32).atan2(i.vx * direction as f32);
                if (s.rotation - aim).abs() >= 3.14 {
                    if aim < s.rotation {
                        s.rotation -= 6.28;
                    } else {
                        s.rotation += 6.28;
                    }
                }
                s.rotation = (s.rotation * 4.0 + aim) / 5.0;
                // `:75344-75358`.
                s.counter += i.vx.hypot(i.vy);
                s.counter += 1.0;
                if s.counter > 8.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
                if s.row > 20 {
                    s.row = 16;
                } else if s.row < 16 {
                    s.row = 19;
                }
                return;
            }
            // `:75361-75372` — the dry rotation decay.
            if s.rotation > 3.14 {
                s.rotation -= 6.28;
            }
            if s.rotation > -0.01 && s.rotation < 0.01 {
                s.rotation = 0.0;
            } else {
                s.rotation *= 0.9;
            }
            if i.vy == 0.0 {
                // `:75375-75382`.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                if i.vx == 0.0 {
                    // `:75383-75388`.
                    s.row = 0;
                    s.counter = 0.0;
                    return;
                }
                // `:75389-75403`.
                if s.row < 2 {
                    s.row = 2;
                }
                s.counter += i.vx.abs() * 2.0;
                s.counter += 1.0;
                if s.counter > 6.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
                if s.row > 15 {
                    s.row = 2;
                }
            } else {
                // `:75405-75409`.
                s.counter = 0.0;
                s.row = 1;
            }
        }
        // `case 462` in the humanoid-group case list
        // (`NPC.cs:68612`) → the shared fighter walk with Fritz's
        // THREE per-type overrides: the walk stride `num108` = 9
        // (`:69555-69558` — the > 9 step, not the group's > 6), the
        // air branch pins row 1 rising / row 2 falling
        // (`:69588-69600` — BEFORE the group's air tail, an early
        // break), and the sheet is 9 rows (`Main.cs:67130`). NOT
        // `isLikeATownNPC` (aiStyle 3, `:15520-15533`), not a town
        // slime, not the 140/489 pair; the `:69647` pin list is
        // unreachable (the 462 air branch breaks first). The walk
        // accumulator is the group's `|vx| * 2 + 1` (`:69552-69553`),
        // the lift-to-2 and the full-count wrap-to-2 stand.
        FindFrameExtra8Kind::Fritz => {
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
                    // `:69521-69585` — num108 = 9 (the 462 override),
                    // num109 = row 2.
                    if s.row < 2 {
                        s.row = 2;
                    }
                    s.counter += i.vx.abs() * 2.0;
                    s.counter += 1.0;
                    if s.counter > 9.0 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                    if s.row >= kind.frame_count() {
                        s.row = 2;
                    }
                }
            } else {
                // `:69588-69600` — the 462 air branch: rise row 1 /
                // fall row 2, the counter zeroed, the group tail
                // never runs.
                s.counter = 0.0;
                s.row = if i.vy < 0.0 { 1 } else { 2 };
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

    fn rows_over(kind: FindFrameExtra8Kind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra8(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The frame-count table agrees with the port's verbatim
    /// `NPC_FRAME_COUNT` (`Main.cs:67082` initializer cross-check).
    #[test]
    fn f27s8_frame_counts_match_the_port_table() {
        let table = crate::render::entities::NPC_FRAME_COUNT;
        for k in [
            FindFrameExtra8Kind::GoldBunny,
            FindFrameExtra8Kind::GoldButterfly,
            FindFrameExtra8Kind::GoldFrog,
            FindFrameExtra8Kind::GoldGrasshopper,
            FindFrameExtra8Kind::GoldMouse,
            FindFrameExtra8Kind::GoldWorm,
            FindFrameExtra8Kind::BoneThrowingSkeleton,
            FindFrameExtra8Kind::BoneThrowingSkeleton2,
            FindFrameExtra8Kind::BoneThrowingSkeleton3,
            FindFrameExtra8Kind::BoneThrowingSkeleton4,
            FindFrameExtra8Kind::SkeletonMerchant,
            FindFrameExtra8Kind::CultistDragonHead,
            FindFrameExtra8Kind::Butcher,
            FindFrameExtra8Kind::CreatureFromTheDeep,
            FindFrameExtra8Kind::Fritz,
        ] {
            assert_eq!(
                k.frame_count(),
                table[k.vanilla_type() as usize],
                "type {}",
                k.vanilla_type()
            );
        }
    }

    /// 443 GoldBunny: the |vx|+1 hop over the FULL 7 rows, the idle
    /// row-0 snap, the air pins 4/6 (`NPC.cs:76866-76903`).
    /// Expectation re-derived by mechanical replay of the C# order.
    #[test]
    fn f27s8_gold_bunny_hops_the_seven_rows() {
        // vx 1: +2 a tick, the > 6 step lands every 4th tick.
        let rows = rows_over(FindFrameExtra8Kind::GoldBunny, input(1.0, 0.0), 16);
        assert_eq!(rows, vec![0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4]);
        // vx 3: +4 a tick, the step every 2nd tick.
        let rows = rows_over(FindFrameExtra8Kind::GoldBunny, input(3.0, 0.0), 10);
        assert_eq!(rows, vec![0, 1, 1, 2, 2, 3, 3, 4, 4, 5]);
        // The idle snap + the air pins.
        let mut s = FrameState { row: 5, counter: 4.0, ..Default::default() };
        find_frame_extra8(FindFrameExtra8Kind::GoldBunny, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        find_frame_extra8(FindFrameExtra8Kind::GoldBunny, &mut s, &input(1.0, -2.0));
        assert_eq!((s.row, s.counter), (4, 0.0));
        find_frame_extra8(FindFrameExtra8Kind::GoldBunny, &mut s, &input(1.0, 2.0));
        assert_eq!((s.row, s.counter), (6, 0.0));
        // The grounded flip gates on direction ±1; 0 leaves it.
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        let i = FrameInput { direction: 0, ..input(1.0, 0.0) };
        find_frame_extra8(FindFrameExtra8Kind::GoldBunny, &mut s, &i);
        assert_eq!(s.sprite_dir, -1);
    }

    /// 444 GoldButterfly: the velocity-fed 7-tick bands 0/1/2/1 with
    /// the 27 reset, the vx*0.3 tilt, the unconditional direction
    /// copy, NO colour bank (`NPC.cs:72745-72771`).
    #[test]
    fn f27s8_gold_butterfly_flaps_the_three_rows() {
        // vx 1 / vy -1: +2 a tick — bands at 7/14/21, reset at 27.
        let rows = rows_over(FindFrameExtra8Kind::GoldButterfly, input(1.0, -1.0), 20);
        assert_eq!(rows, vec![0, 0, 0, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1]);
        // Still air: +1 a tick — the 27th tick resets, tick 28 row 0.
        let rows = rows_over(FindFrameExtra8Kind::GoldButterfly, input(0.0, 0.0), 29);
        assert_eq!(rows[26], 1, "counter 27 resets on the 27th tick");
        assert_eq!(rows[27], 0);
        let mut s = FrameState::default();
        let i = FrameInput { direction: 0, ..input(2.0, 0.0) };
        find_frame_extra8(FindFrameExtra8Kind::GoldButterfly, &mut s, &i);
        assert_eq!(s.sprite_dir, 0, "the copy is unconditional");
        assert_eq!(s.rotation, 0.6);
        assert!(s.row < 3, "no ai[2] bank offset for 444");
    }

    /// 445 GoldFrog: the wet |vx| bands 10/11/12, the idle > 6 step
    /// over rows 0..5 (the wrap runs every tick), the walk bands
    /// 0/6/8/9 with the 23 reset, the air rows 8/9
    /// (`NPC.cs:76621-76684`).
    #[test]
    fn f27s8_gold_frog_rides_the_three_gaits() {
        let rows = rows_over(FindFrameExtra8Kind::GoldFrog, input(0.0, 0.0), 16);
        assert_eq!(rows, vec![0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2]);
        let rows = rows_over(FindFrameExtra8Kind::GoldFrog, input(1.0, 0.0), 25);
        assert_eq!(
            rows,
            vec![0, 0, 0, 0, 0, 6, 6, 6, 6, 6, 6, 8, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 9, 0, 0]
        );
        // Wet: the 0.25/0.15 deadbands, the counter zeroed.
        for (vx, row) in [(0.3f32, 10u32), (-0.3, 10), (0.2, 11), (-0.2, 11), (0.1, 12), (0.0, 12)] {
            let i = FrameInput { wet: true, ..input(vx, 0.0) };
            let mut s = FrameState { counter: 5.0, ..Default::default() };
            find_frame_extra8(FindFrameExtra8Kind::GoldFrog, &mut s, &i);
            assert_eq!((s.row, s.counter), (row, 0.0), "wet vx {vx}");
        }
        // Air: fall row 9 / rise row 8; the direction copy is
        // unconditional on every path.
        let i = FrameInput { direction: 0, ..input(0.0, 2.0) };
        let mut s = FrameState::default();
        find_frame_extra8(FindFrameExtra8Kind::GoldFrog, &mut s, &i);
        assert_eq!((s.row, s.sprite_dir), (9, 0));
        find_frame_extra8(FindFrameExtra8Kind::GoldFrog, &mut s, &input(0.0, -2.0));
        assert_eq!(s.row, 8);
    }

    /// 446 GoldGrasshopper: the air row 1 / ground row 0 pin, the
    /// unconditional direction copy, the counter NEVER touched
    /// (`NPC.cs:70973-70981`).
    #[test]
    fn f27s8_gold_grasshopper_pins_the_two_rows() {
        let mut s = FrameState { counter: 3.0, ..Default::default() };
        find_frame_extra8(FindFrameExtra8Kind::GoldGrasshopper, &mut s, &input(2.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 3.0), "grounded row 0, counter held");
        find_frame_extra8(FindFrameExtra8Kind::GoldGrasshopper, &mut s, &input(2.0, -1.0));
        assert_eq!((s.row, s.counter), (1, 3.0));
        find_frame_extra8(FindFrameExtra8Kind::GoldGrasshopper, &mut s, &input(2.0, 1.0));
        assert_eq!(s.row, 1, "the fall pins row 1 too");
        let i = FrameInput { direction: 0, ..input(0.0, 0.0) };
        find_frame_extra8(FindFrameExtra8Kind::GoldGrasshopper, &mut s, &i);
        assert_eq!(s.sprite_dir, 0, "the copy is unconditional");
    }

    /// 447 GoldMouse: the idle > 8 two-row blink, the |vx|*0.5+1
    /// walk over the FULL 6 rows wrapping to 2, the air walk keeps
    /// striding with NO flip (`NPC.cs:76515-76564`).
    #[test]
    fn f27s8_gold_mouse_blinks_and_strides() {
        let rows = rows_over(FindFrameExtra8Kind::GoldMouse, input(0.0, 0.0), 20);
        assert_eq!(rows, vec![0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0]);
        // vx 1: +1.5 a tick, the > 6 step every 5th tick.
        let rows = rows_over(FindFrameExtra8Kind::GoldMouse, input(1.0, 0.0), 16);
        assert_eq!(rows, vec![0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3]);
        // The wrap-to-2 at the full count: a 6.0 counter + 1.5 = 7.5 > 6
        // steps row 5 -> 6, which is >= the sheet count, so row 2
        // (`NPC.cs:76545-76548`).
        let mut s = FrameState { row: 5, counter: 6.0, ..Default::default() };
        find_frame_extra8(FindFrameExtra8Kind::GoldMouse, &mut s, &input(1.0, 0.0));
        assert_eq!((s.row, s.counter), (2, 0.0));
        // The air walk: same stride, NO sprite flip.
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        find_frame_extra8(FindFrameExtra8Kind::GoldMouse, &mut s, &input(1.0, 2.0));
        assert_eq!(s.sprite_dir, -1, "the air branch does not flip");
        assert_eq!(s.counter, 1.5);
    }

    /// 448 GoldWorm: the localAI[0] = -2 pin, the idle row 1, the
    /// > 12 two-row crawl (1/0 alternation), the air spin
    /// accumulation, the slope override (`NPC.cs:72779-72826`).
    #[test]
    fn f27s8_gold_worm_crawls_and_pins() {
        // The crawl: the 13th tick steps row 0 → 1, the 26th wraps
        // back to 0 (the > num wrap, rows alternate).
        let rows = rows_over(FindFrameExtra8Kind::GoldWorm, input(1.0, 0.0), 28);
        assert_eq!(rows[11], 0);
        assert_eq!(rows[12], 1, "the 13th tick steps");
        assert_eq!(rows[24], 1);
        assert_eq!(rows[25], 0, "the 26th tick wraps");
        // The idle pin + the localAI[0] = -2 write.
        let mut s = FrameState { row: 0, counter: 5.0, ..Default::default() };
        find_frame_extra8(FindFrameExtra8Kind::GoldWorm, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter, s.local_ai0), (1, 0.0, -2.0));
        assert_eq!(s.rotation, 0.0, "the grounded tick zeroes the tilt");
        // The air spin accumulates direction * 0.1, the row pins 1.
        let mut s = FrameState { rotation: 0.2, ..Default::default() };
        let i = FrameInput { direction: -1, ..input(1.0, -2.0) };
        find_frame_extra8(FindFrameExtra8Kind::GoldWorm, &mut s, &i);
        assert_eq!(s.row, 1);
        assert!((s.rotation - 0.1).abs() < 1e-6);
        // The slope override: both slopes lift localAI[0] to 0.
        for (slope, rot) in [(1i32, 0.785f32), (2, -0.785)] {
            let i = FrameInput { slope, ..input(1.0, 0.0) };
            let mut s = FrameState::default();
            find_frame_extra8(FindFrameExtra8Kind::GoldWorm, &mut s, &i);
            assert_eq!((s.rotation, s.local_ai0), (rot, 0.0), "slope {slope}");
        }
    }

    /// 449-452 the bone throwers: the ai[1] wind-up toss ladder rows
    /// count-5..count-1, the idle row-0 snap with the counter
    /// UNTOUCHED, the |vx|*2 NO-+1 walk wrapping at the literal 15,
    /// the air row 1 (`NPC.cs:71402-71452`). The 452 is the ONE
    /// 20-row sheet — its toss band reaches row 19.
    #[test]
    fn f27s8_bone_throwers_toss_and_stride() {
        for k in [
            FindFrameExtra8Kind::BoneThrowingSkeleton,
            FindFrameExtra8Kind::BoneThrowingSkeleton2,
            FindFrameExtra8Kind::BoneThrowingSkeleton3,
            FindFrameExtra8Kind::BoneThrowingSkeleton4,
        ] {
            let base = k.frame_count() - 5;
            // The toss ladder: ai[1] 50/43/36/29/0 → bands 1/2/3/4/0.
            for (ai1, band) in [(50.0f32, 1u32), (43.0, 2), (36.0, 3), (29.0, 4), (28.9, 0), (0.0, 0)] {
                let i = FrameInput { ai: [0.0, ai1, 1.0, 0.0], direction: 0, ..input(0.0, 0.0) };
                let mut s = FrameState { counter: 4.0, sprite_dir: 1, ..Default::default() };
                find_frame_extra8(k, &mut s, &i);
                assert_eq!(s.row, base + band, "{k:?} ai1 {ai1}");
                assert_eq!(s.counter, 0.0);
                assert_eq!(s.sprite_dir, 0, "the toss copies direction verbatim");
            }
            // The idle snap keeps the counter.
            let mut s = FrameState { row: 7, counter: 3.0, ..Default::default() };
            find_frame_extra8(k, &mut s, &input(0.0, 0.0));
            assert_eq!((s.row, s.counter), (0, 3.0));
            // The walk: vx 1 → +2 a tick, the > 6 step every 4th tick.
            let rows = rows_over(k, input(1.0, 0.0), 10);
            assert_eq!(rows, vec![2, 2, 2, 3, 3, 3, 3, 4, 4, 4]);
            // The wrap at the LITERAL 15 (not the sheet count).
            let mut s = FrameState { row: 14, counter: 5.0, ..Default::default() };
            find_frame_extra8(k, &mut s, &input(1.0, 0.0));
            assert_eq!(s.row, 2, "row 15 wraps to 2 on every sheet");
            // The air pin.
            find_frame_extra8(k, &mut s, &input(1.0, -2.0));
            assert_eq!((s.row, s.counter), (1, 0.0));
        }
        // The 452's one-extra toss row: band 1 lands row 16 (the
        // 19-row trio lands 15) -- `num53 = count - 5` (`NPC.cs:71416`),
        // count 20 vs 19 (`Main.cs:67129`).
        let i = FrameInput { ai: [0.0, 50.0, 1.0, 0.0], ..input(0.0, 0.0) };
        let mut s = FrameState::default();
        find_frame_extra8(FindFrameExtra8Kind::BoneThrowingSkeleton4, &mut s, &i);
        assert_eq!(s.row, 16);
        find_frame_extra8(FindFrameExtra8Kind::BoneThrowingSkeleton, &mut s, &i);
        assert_eq!(s.row, 15);
    }

    /// 453 SkeletonMerchant: the town walk wraps at 26 − 9 = 17
    /// (rows 2..16), the idle snaps row 0, the air holds row 1
    /// (`NPC.cs:69521-69585` + `NPCID.cs:10464`'s
    /// ExtraFramesCount[453] = 9; the extra-3 Cyborg precedent).
    #[test]
    fn f27s8_merchant_town_band_wraps_at_seventeen() {
        let rows = rows_over(FindFrameExtra8Kind::SkeletonMerchant, input(0.0, 0.0), 3);
        assert_eq!(rows, vec![0, 0, 0]);
        // vx 1: +3 a tick, the > 6 step every 3rd tick.
        let rows = rows_over(FindFrameExtra8Kind::SkeletonMerchant, input(1.0, 0.0), 6);
        assert_eq!(rows, vec![2, 2, 3, 3, 3, 4]);
        // vx 3: +7 a tick — the walk reaches row 16 then wraps to 2
        // (row 17 >= 26 - 9).
        let rows = rows_over(FindFrameExtra8Kind::SkeletonMerchant, input(3.0, 0.0), 16);
        assert_eq!(rows[12], 15);
        assert_eq!(rows[13], 16, "the last town row");
        assert_eq!(rows[14], 2, "row 17 wraps to 2");
        let mut s = FrameState { row: 5, counter: 4.0, ..Default::default() };
        find_frame_extra8(FindFrameExtra8Kind::SkeletonMerchant, &mut s, &input(1.0, -2.0));
        assert_eq!((s.row, s.counter), (1, 0.0), "the air holds row 1");
    }

    /// 454 CultistDragonHead: the row is (int)(counter / 2), the
    /// counter NEVER touched (`NPC.cs:70954-70955`).
    #[test]
    fn f27s8_dragon_head_reads_the_counter_halved() {
        for (counter, row) in [(0.0f32, 0u32), (1.0, 0), (2.0, 1), (3.0, 1), (4.0, 2), (5.0, 2)] {
            let mut s = FrameState { counter, ..Default::default() };
            find_frame_extra8(FindFrameExtra8Kind::CultistDragonHead, &mut s, &input(9.0, 9.0));
            assert_eq!(s.row, row, "counter {counter}");
            assert_eq!(s.counter, counter, "the machine never ticks it");
        }
    }

    /// 460 Butcher: the wave-B fighter delegate — the |vx|*2+1
    /// stride, the FULL-16 wrap, the idle row 0, the air row 1
    /// (`NPC.cs:69521-69585`/`:69642-69646`).
    #[test]
    fn f27s8_butcher_walks_the_fighter_bands() {
        let rows = rows_over(FindFrameExtra8Kind::Butcher, input(1.0, 0.0), 6);
        assert_eq!(rows, vec![2, 2, 3, 3, 3, 4]);
        // vx 3: +7 a tick — the walk reaches row 15 then wraps to 2.
        let rows = rows_over(FindFrameExtra8Kind::Butcher, input(3.0, 0.0), 15);
        assert_eq!(rows[11], 14);
        assert_eq!(rows[12], 15);
        assert_eq!(rows[13], 2, "the full-count wrap");
        let mut s = FrameState { row: 5, counter: 4.0, ..Default::default() };
        find_frame_extra8(FindFrameExtra8Kind::Butcher, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        find_frame_extra8(FindFrameExtra8Kind::Butcher, &mut s, &input(1.0, -2.0));
        assert_eq!((s.row, s.counter), (1, 0.0), "not in the pin list");
    }

    /// 461 CreatureFromTheDeep: the wet swim band 16..20 with the
    /// < 16 → 19 entry quirk + the aim ease, the dry walk's > 15
    /// wrap + the rotation decay (`NPC.cs:75316-75409`).
    #[test]
    fn f27s8_creature_swims_and_strides() {
        // Wet vx 3: +4 a tick — the > 8 step every 3rd tick; the
        // first tick lifts row 0 to 19 (the entry quirk).
        let i = FrameInput { wet: true, ..input(3.0, 0.0) };
        let rows = rows_over(FindFrameExtra8Kind::CreatureFromTheDeep, i, 15);
        assert_eq!(rows, vec![19, 19, 20, 20, 20, 16, 16, 16, 17, 17, 17, 18, 18, 18, 19]);
        // The wet aim: swimming right at vy 0 aims 0; the ease is
        // (rot*4 + aim)/5.
        let mut s = FrameState { rotation: 1.0, sprite_dir: 1, ..Default::default() };
        find_frame_extra8(FindFrameExtra8Kind::CreatureFromTheDeep, &mut s, &i);
        assert_eq!(s.rotation, 0.8, "(1*4 + 0)/5");
        // A facing change mirrors the rotation first.
        let mut s = FrameState { rotation: 0.5, sprite_dir: 1, ..Default::default() };
        let i = FrameInput { wet: true, ..input(-3.0, 0.0) };
        find_frame_extra8(FindFrameExtra8Kind::CreatureFromTheDeep, &mut s, &i);
        assert_eq!(s.sprite_dir, -1, "the vx sign writes direction");
        // aim = atan2(0*-1, -3*-1) = 0; rot mirrored to -0.5, eased
        // to (-0.5*4 + 0)/5 = -0.4.
        assert_eq!(s.rotation, -0.4);
        // Dry: the rotation deadband snaps 0, the ×0.9 ease otherwise.
        let mut s = FrameState { rotation: 0.005, ..Default::default() };
        find_frame_extra8(FindFrameExtra8Kind::CreatureFromTheDeep, &mut s, &input(0.0, 0.0));
        assert_eq!(s.rotation, 0.0);
        let mut s = FrameState { rotation: 1.0, ..Default::default() };
        find_frame_extra8(FindFrameExtra8Kind::CreatureFromTheDeep, &mut s, &input(0.0, 0.0));
        assert_eq!(s.rotation, 0.9);
        // The dry walk wraps at > 15 (rows 16..20 are swim-only).
        let rows = rows_over(FindFrameExtra8Kind::CreatureFromTheDeep, input(3.0, 0.0), 15);
        assert_eq!(rows[11], 14);
        assert_eq!(rows[12], 15);
        assert_eq!(rows[13], 2);
        // The dry idle + air pins.
        let mut s = FrameState { row: 9, counter: 4.0, ..Default::default() };
        find_frame_extra8(FindFrameExtra8Kind::CreatureFromTheDeep, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        find_frame_extra8(FindFrameExtra8Kind::CreatureFromTheDeep, &mut s, &input(1.0, 2.0));
        assert_eq!((s.row, s.counter), (1, 0.0));
    }

    /// 462 Fritz: the num108 = 9 stride (the > 9 step), the FULL-9
    /// wrap, the air rows 1 rising / 2 falling (the early-break
    /// branch, `NPC.cs:69555-69558`/`:69588-69600`).
    #[test]
    fn f27s8_fritz_strides_at_nine_and_pins_the_air_pair() {
        // vx 1: +3 a tick — the > 9 step every 4th tick.
        let rows = rows_over(FindFrameExtra8Kind::Fritz, input(1.0, 0.0), 12);
        assert_eq!(rows, vec![2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5]);
        // vx 3: +7 a tick — the walk reaches row 8 then wraps to 2.
        let rows = rows_over(FindFrameExtra8Kind::Fritz, input(3.0, 0.0), 15);
        assert_eq!(rows[10], 7);
        assert_eq!(rows[11], 8);
        assert_eq!(rows[12], 8);
        assert_eq!(rows[13], 2, "the full-9 wrap");
        // The air pair: rise row 1 / fall row 2 (NOT the group's
        // row-1-both tail).
        let mut s = FrameState { row: 5, counter: 4.0, ..Default::default() };
        find_frame_extra8(FindFrameExtra8Kind::Fritz, &mut s, &input(1.0, -2.0));
        assert_eq!((s.row, s.counter), (1, 0.0));
        find_frame_extra8(FindFrameExtra8Kind::Fritz, &mut s, &input(1.0, 2.0));
        assert_eq!((s.row, s.counter), (2, 0.0));
        // The idle snap.
        find_frame_extra8(FindFrameExtra8Kind::Fritz, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
    }

    /// The 15-kind strip sweep: a 5x3x3 vx/vy/direction input grid
    /// (plus the wet/ai[1]/ai[2]/slope variants for 445/448/449-452/
    /// 461), 40 ticks each — every row stays inside its sheet (the
    /// extra-7 precedent).
    #[test]
    fn f27s8_fifteen_kind_strip_sweep_stays_inside_the_sheets() {
        let kinds = [
            FindFrameExtra8Kind::GoldBunny,
            FindFrameExtra8Kind::GoldButterfly,
            FindFrameExtra8Kind::GoldFrog,
            FindFrameExtra8Kind::GoldGrasshopper,
            FindFrameExtra8Kind::GoldMouse,
            FindFrameExtra8Kind::GoldWorm,
            FindFrameExtra8Kind::BoneThrowingSkeleton,
            FindFrameExtra8Kind::BoneThrowingSkeleton2,
            FindFrameExtra8Kind::BoneThrowingSkeleton3,
            FindFrameExtra8Kind::BoneThrowingSkeleton4,
            FindFrameExtra8Kind::SkeletonMerchant,
            FindFrameExtra8Kind::CultistDragonHead,
            FindFrameExtra8Kind::Butcher,
            FindFrameExtra8Kind::CreatureFromTheDeep,
            FindFrameExtra8Kind::Fritz,
        ];
        for k in kinds {
            let count = k.frame_count();
            for vx in [-4.0f32, -1.0, 0.0, 1.0, 4.0] {
                for vy in [-3.0f32, 0.0, 3.0] {
                    for direction in [-1i32, 0, 1] {
                        for &(wet, ai1, ai2, slope) in &[
                            (false, 0.0, 0.0, 0),
                            (true, 45.0, 1.0, 1),
                            (false, 30.0, 2.0, 2),
                        ] {
                            let i = FrameInput {
                                vx,
                                vy,
                                direction,
                                wet,
                                ai: [0.0, ai1, ai2, 0.0],
                                slope,
                            };
                            let mut s = FrameState::default();
                            for tick in 0..40 {
                                find_frame_extra8(k, &mut s, &i);
                                // 454's row is the raw counter/2 —
                                // the chain machine owns the counter
                                // (it never ticks here, so the row
                                // stays 0); every other arm must
                                // stay inside its sheet.
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

    /// The seam maps exactly the slice's vanilla types and nothing
    /// else.
    #[test]
    fn f27s8_seam_covers_exactly_the_slice() {
        for t in 0..697i16 {
            let k = find_frame_kind_extra8(t);
            let claimed = [
                443, 444, 445, 446, 447, 448, 449, 450, 451, 452, 453, 454, 460, 461, 462,
            ];
            assert_eq!(k.is_some(), claimed.contains(&t), "type {t}");
            if let Some(k) = k {
                assert_eq!(k.vanilla_type(), t);
            }
        }
    }
}
