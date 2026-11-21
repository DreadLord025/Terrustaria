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
        // The batch-2/3 kinds land in the next commits (the 5-kinds-
        // per-commit rule).
        _ => unimplemented!("extra-8 batch 2/3"),
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
        // The wrap-to-2 at the full count (rows 0/1 idle-only).
        let mut s = FrameState { row: 5, ..Default::default() };
        find_frame_extra8(FindFrameExtra8Kind::GoldMouse, &mut s, &input(1.0, 0.0));
        assert_eq!(s.row, 2);
        // The air walk: same stride, NO sprite flip.
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        find_frame_extra8(FindFrameExtra8Kind::GoldMouse, &mut s, &input(1.0, 2.0));
        assert_eq!(s.sprite_dir, -1, "the air branch does not flip");
        assert_eq!(s.counter, 1.5);
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
