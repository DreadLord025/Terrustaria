//! W4-f26 slice 7f — FindFrame extra wave 7f: the NEXT 5 TRULY
//! unclaimed missing case ids of `NPC.FindFrame` (`NPC.cs:66684-77370`)
//! after the 7e slice — **531/532/534/535/536** — the wall-crawling
//! Desert Scorpion, the Desert Beast, the Demon Tax Collector, the
//! Spiked Slime and The Bride (`ID/NPCID.cs:11739-11749`).
//!
//! The census (the 7e docs method, `docs/research/68` addendum): 117
//! unclaimed after the pushed 7e tip `9c2b0a84`'s 5 claims; − THIS
//! slice's 5 = **112 unclaimed**. The re-probe at THIS slice's census
//! time: the f27 pushed tip `0e187428` (slice 9 + docs) carries NO
//! "slice 10" claim band in its ANALYSIS (findstr = no match) and its
//! `npc_findframe_extra7`/`extra8`/`extra9` modules carry NO
//! 531/532/534/535/536 arm (mechanical grep = the f27 extra-7 slime
//! group-comment mention of 535 only, no machine); 533 DesertDjinn is
//! port-covered (the S1-E-5 `npc.rs:8086` arm — EXCLUDED, the 7e
//! addendum's tail note); the d41 pushed tip `4ed22f0b` is STILL the
//! CLAIM-only commit (mechanical grep over its `npc.rs`/`npc/data.rs`
//! for all five ids + kind names = ZERO); the d42 pushed tip
//! `9b22c091` (ADVANCED past the 7e census's `88df5809` — now the
//! 578-630 roster docs) greps ZERO for the five; the d40 tip
//! `8fe1d6d3`'s roster stops at 490; this branch's own `npc.rs` has
//! no arm for any of the five (no `DesertScorpionWall`/`DesertBeast`/
//! `DemonTaxCollector`/`SlimeSpiked`/`TheBride` kind in `NpcKind`).
//!
//! The five arms: 531 DesertScorpionWall rides the
//! `case 165/237/238/240/531` body (`NPC.cs:73112-73145`, label
//! `:73116`) — the wall group's `(|vx| + |vy|) * num194` band machine
//! with the 531-ONLY `num194 = 0.4` override (`:73119-73122`; the
//! co-members 165 d46-self / 237/238/240 f27-WIRED(g7) keep the 0.5
//! default — the d44 master wire-status table). 532 DesertBeast rides
//! the `case 508/532/580/582` walking-antlion body (`:70067-70103`,
//! label `:70068`) — the machine copied VERBATIM from the 7c module
//! (the 7e ghoul ONE-group precedent) with the 8-row count
//! (`Main.cs:67137` entry 3) widening the walk strip to rows 2..7.
//! 534 DemonTaxCollector rides the humanoid-group case list
//! (`:68615`) → the shared fighter walk (`:68647-69655`) with the ONE
//! per-type override `num108 = 12` (`:69528-69531` — the > 12 stride,
//! the f27 Fritz `num108 = 9` inline precedent: the wave-B helper
//! hard-codes > 6, so the override lands as a verbatim inline copy).
//! 535 SlimeSpiked rides the slime strip group (`:70818` →
//! `:70828-70845`) — the num2-fed counter copied VERBATIM from the
//! extra-3 SpikedJungleSlime machine (the wave-A slime keep-one). 536
//! TheBride rides the zombie walker group (`:76344` →
//! `:76347-76390`) — the machine copied VERBATIM from wave A's
//! `zombie_walk_frame` (the S1-D-3z holiday-zombie precedent). Landed
//! 5 kinds in ONE commit.
//!
//! Shape: the kinds are NOT in `NpcKind` yet (the d41 roster wave owns
//! them), so — the `npc_findframe_extra7e.rs` precedent — the frame
//! machines land first as pure functions over a small
//! [`FrameState`]/[`FrameInput`] pair. The wiring seam is
//! [`find_frame_kind_extra7f`]: when a kind joins `NpcKind`, its
//! `find_frame` arm copies the state fields out, runs one tick and
//! copies them back (the `Npc::find_frame_wave_a` bridge precedent).
//!
//! RNG: NONE — all five arms draw ZERO `Main.rand` (mechanical grep
//! over the cited ranges `:73112-73145` + `:70067-70103` +
//! `:69489-69585` + `:70828-70845` + `:76347-76390` = ZERO). No world
//! reads either: no `Framing`/`Main.tile`, no `ai`/`localAI` reads in
//! the ported zones. The humanoid group's `ai[0]` pose zones
//! (`:68662-69488`, with their chat-bubble/emote `Main.rand` draws)
//! are the documented skip (the f27 extra-8 SkeletonMerchant
//! precedent — 534 is aiStyle 3, `:15817`, no townNPC, the zones
//! never fire for it). `FrameInput` carries vx/vy + `ai_action` (the
//! slime group's `num2` motion class, `NPC.cs:66696-66704` — the
//! wave-A shape); `direction` rides [`FrameInput`] (the extra3/wave-A
//! shape — this wave's copies are ±1/sign-gated, never unconditional).
//!
//! Row convention: the port tracks frame ROW indices, not pixels —
//! the C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer,
//! `:67137`'s 530-539 row), counted entry-by-entry in
//! [`FindFrameExtra7fKind::frame_count`] and cross-checked against the
//! port's own `render/entities.rs::NPC_FRAME_COUNT` table.

// The wiring is a later round (the kinds are not in `NpcKind` yet) —
// the `npc_findframe_extra7e.rs` precedent.
#![allow(dead_code)]

/// The extra-wave-7f kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra7fKind {
    /// `NPCID.DesertScorpionWall` = 531 (`NPCID.cs:11739`) — the
    /// 165/237/238/240/531 wall group (`NPC.cs:73116`); only 531 is
    /// this wave's (the co-members are WIRED per the d44 table).
    DesertScorpionWall,
    /// `NPCID.DesertBeast` = 532 (`NPCID.cs:11741`) — the
    /// 508/532/580/582 walking-antlion group (`NPC.cs:70068`); the
    /// 7c-deferred id (508 rides the 7c module).
    DesertBeast,
    /// `NPCID.DemonTaxCollector` = 534 (`NPCID.cs:11745`) — the
    /// humanoid-group case list (`NPC.cs:68615`), the `num108 = 12`
    /// stride override (`:69528-69531`).
    DemonTaxCollector,
    /// `NPCID.SlimeSpiked` = 535 (`NPCID.cs:11747`) — the slime strip
    /// group (`NPC.cs:70818`); 537 SandSlime rides a later slice.
    SlimeSpiked,
    /// `NPCID.TheBride` = 536 (`NPCID.cs:11749`) — the zombie walker
    /// group (`NPC.cs:76344`).
    TheBride,
}

impl FindFrameExtra7fKind {
    /// The vanilla `NPC.type` id.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::DesertScorpionWall => 531,
            Self::DesertBeast => 532,
            Self::DemonTaxCollector => 534,
            Self::SlimeSpiked => 535,
            Self::TheBride => 536,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// counted entry-by-entry): `:67137`'s 530-539 row — 531 = 4
    /// (entry 2), 532 = 8 (entry 3), 534 = 8 (entry 5), 535 = 2
    /// (entry 6), 536 = 3 (entry 7).
    pub fn frame_count(self) -> u32 {
        match self {
            Self::DesertScorpionWall => 4,
            Self::DesertBeast => 8,
            Self::DemonTaxCollector => 8,
            Self::SlimeSpiked => 2,
            Self::TheBride => 3,
        }
    }
}

/// The vanilla-id wiring seam: `Some(kind)` for exactly this wave's
/// ids (the `find_frame_kind_extra7e` precedent). The wiring round
/// hooks it at the `npc.rs::find_frame` head by `cs_id`.
pub fn find_frame_kind_extra7f(vanilla_type: i16) -> Option<FindFrameExtra7fKind> {
    Some(match vanilla_type {
        531 => FindFrameExtra7fKind::DesertScorpionWall,
        532 => FindFrameExtra7fKind::DesertBeast,
        534 => FindFrameExtra7fKind::DemonTaxCollector,
        535 => FindFrameExtra7fKind::SlimeSpiked,
        536 => FindFrameExtra7fKind::TheBride,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter` and `spriteDirection`.
/// The wiring step copies them to/from `Npc`. `rotation`/`local_ai`
/// are carried for the uniform 7e wiring copy, never read/written
/// here.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter` (the C# field is a `double`, `NPC.cs:6485`;
    /// the port's f32 is the existing `npc.rs` convention — the
    /// integer/half accumulators here are exact in both).
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — no arm writes it this wave.
    pub rotation: f32,
    /// `NPC.localAI[0]`/`[1]` — never read/written this wave; carried
    /// for the uniform wiring copy.
    pub local_ai: [f32; 2],
}

/// The per-tick inputs the arms read: `velocity`, `direction` and
/// `aiAction` (the slime group's `num2` motion class). No `ai` reads,
/// no RNG draws, no world reads this wave (the header's census).
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone —
    /// every copy this wave is ±1-gated or sign-gated, none
    /// unconditional).
    pub direction: i32,
    /// `NPC.aiAction` — 0 the motion class, 1 the attack (`num2 = 4`).
    pub ai_action: i32,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1, ai_action: 0 }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-7f kinds — one
/// tick.
pub fn find_frame_extra7f(kind: FindFrameExtra7fKind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // `case 165/237/238/240/531` (`NPC.cs:73112-73145`): the wall
        // crawlers (only 531 is this wave's). `num194` defaults 0.5
        // (`:73118`) but 531 overrides it to 0.4 (`:73119-73122`).
        // The accumulator adds `(|vx| + |vy|) * num194` EVERY tick
        // (`:73123` — no grounded/air split, no idle branch); the
        // <6/<12/<18/<24 bands pin rows 0/1/2/3 (`:73124-73139`);
        // the else (`:73140-73143`) zeroes the counter WITHOUT
        // writing the row — the vanilla quirk: the row HOLDS 3 for
        // that tick, then the cycle restarts at row 0 (the extra-3
        // Moth precedent). No sprite/rotation write.
        FindFrameExtra7fKind::DesertScorpionWall => {
            // `:73118-73122` — the 531-only 0.4 override.
            let num194: f32 = 0.4;
            // `:73123`.
            s.counter += (i.vx.abs() + i.vy.abs()) * num194;
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
                // `:73142` — the row is NOT written here.
                s.counter = 0.0;
            }
        }
        // `case 508/532/580/582` (`NPC.cs:70067-70103`): the Desert
        // Beast (only 532 is this wave's — the machine copied VERBATIM
        // from the 7c GiantWalkingAntlion). The air (`:70071-70075`)
        // pins row 1 WITHOUT touching the counter; the idle
        // (`:70076-70080`) pins row 0, counter untouched too; the
        // sprite copies are SIGN-GATED (`:70081-70088` — direction
        // AND vx must agree, the mismatch keeps the old sprite); the
        // walk lifts any row < 2 to 2 (`:70089-70092`), accumulates
        // `1 + |vx| / 2` (`:70093`), the > 6 counter steps a row,
        // wrapping at row >= the count back to 2 (`:70098-70101`) —
        // the 8-row count widens the walk strip to rows 2..7 (the 7c
        // 508's 6-row sheet walks 2..5). No rotation write.
        FindFrameExtra7fKind::DesertBeast => {
            if i.vy != 0.0 {
                // `:70071-70075` — the counter is NOT touched.
                s.row = 1;
                return;
            }
            if i.vx == 0.0 {
                // `:70076-70080` — the counter is NOT touched.
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
            // `:70089-70101`.
            if s.row < 2 {
                s.row = 2;
            }
            s.counter += 1.0 + i.vx.abs() / 2.0;
            if s.counter > 6.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= kind.frame_count() {
                s.row = 2;
            }
        }
        // `case 534` in the humanoid-group case list (`NPC.cs:68615`)
        // → the shared fighter walk (`:68647-69655`) with the ONE
        // per-type override: the walk stride `num108 = 12`
        // (`:69528-69531` — the > 12 step, not the group's > 6; the
        // f27 Fritz num108=9 inline precedent — the wave-B helper
        // hard-codes > 6). NOT `isLikeATownNPC` (aiStyle 3,
        // `:15813-15828`, no townNPC — `NPC.cs:6856-6866` gates on
        // 453 + the town flag), so `num57` = 0 and the wrap runs at
        // the FULL 8-row count; NOT a town slime (`NPCID.cs:10119`'s
        // set = 670/678-684), so `num109` = row 2; NOT the 140/489
        // idle-row-1 pair (`:69510`); NOT in the `:69647` air pin
        // list — the group's air tail pins row 1 (`:69644-69645`).
        // The accumulator is the group's `|vx| * 2` THEN `++`
        // (`:69552-69553` — not the 489 variant). The ai[0] pose
        // zones (`:68662-69488`) are the documented skip (the f27
        // SkeletonMerchant precedent).
        FindFrameExtra7fKind::DemonTaxCollector => {
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
                    // `:69521-69585` — num108 = 12 (the 534 override),
                    // num109 = row 2.
                    if s.row < 2 {
                        s.row = 2;
                    }
                    s.counter += i.vx.abs() * 2.0;
                    s.counter += 1.0;
                    if s.counter > 12.0 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                    if s.row >= kind.frame_count() {
                        s.row = 2;
                    }
                }
            } else {
                // `:69642-69646` — the group air tail: row 1, the
                // counter zeroed; 534 is NOT in the `:69647` pin list.
                s.counter = 0.0;
                s.row = 1;
            }
        }
        // `case 535` in the slime strip group (`NPC.cs:70818` →
        // `:70828-70845`; shared with 1/16/59/71/81/138/147/183/184/
        // 204/225/302/304/333-336/537/658/659/667/676): the num2-fed
        // counter — the machine copied VERBATIM from the extra-3
        // SpikedJungleSlime (the wave-A slime keep-one). The `num2`
        // motion class (`NPC.cs:66696-66704`) doubles the counter
        // while moving/airborne and triples it while attacking; the
        // >= 8 stride walks the FULL 2-row sheet wrapping to 0. The
        // `type == 302 || type == 304` spriteDirection snap
        // (`:70824-70827`) is the town slimes' — 535 writes no flip.
        FindFrameExtra7fKind::SlimeSpiked => {
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
            // `:70828-70845`.
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
        // `case 536` in the zombie walker group (`NPC.cs:76344` →
        // `:76347-76390`; shared with 3/52/53/132/161/186-189/200/223/
        // 251/254/255/319-321/331/332/342/590/691): the machine copied
        // VERBATIM from wave A's `zombie_walk_frame` (the S1-D-3z
        // holiday-zombie precedent). The grounded ±1 flip
        // (`:76347-76357` — 0 leaves the sprite alone); the air OR
        // turn-around pin (`:76358-76363` — moving against the facing)
        // zeroes the counter and pins row 2; the stand (`:76364-76369`)
        // zeroes both to row 0; the walk accumulates `|vx|` with NO +1
        // (`:76370`) and the <8/<16/<24/<32 bands pick rows 0/1/2/1
        // (`:76371-76386`); the final else (`:76387-76390`) zeroes the
        // counter WITHOUT writing the row — the lap HOLDS row 1 for
        // that tick (the wave-A verbatim quirk). No rotation write.
        FindFrameExtra7fKind::TheBride => {
            // `:76347-76357`.
            if i.vy == 0.0 {
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
            }
            // `:76358-76390`.
            if i.vy != 0.0 || (i.direction == -1 && i.vx > 0.0) || (i.direction == 1 && i.vx < 0.0) {
                s.counter = 0.0;
                s.row = 2;
            } else if i.vx == 0.0 {
                s.counter = 0.0;
                s.row = 0;
            } else {
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
                    // The C# leaves `frame.Y` alone here
                    // (`:76387-76390`).
                    s.counter = 0.0;
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

    fn facing(vx: f32, vy: f32, direction: i32) -> FrameInput {
        FrameInput { vx, vy, direction, ai_action: 0 }
    }

    fn rows_over(kind: FindFrameExtra7fKind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra7f(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The frame-count table agrees with the port's verbatim
    /// `NPC_FRAME_COUNT` (`Main.cs:67082` initializer cross-check).
    #[test]
    fn f26s7f_frame_counts_match_the_port_table() {
        let table = crate::render::entities::NPC_FRAME_COUNT;
        for k in [
            FindFrameExtra7fKind::DesertScorpionWall,
            FindFrameExtra7fKind::DesertBeast,
            FindFrameExtra7fKind::DemonTaxCollector,
            FindFrameExtra7fKind::SlimeSpiked,
            FindFrameExtra7fKind::TheBride,
        ] {
            assert_eq!(
                k.frame_count(),
                table[k.vanilla_type() as usize],
                "{k:?} ({})",
                k.vanilla_type()
            );
        }
    }

    /// The wall scorpion's 0.4 accumulator: vx 2.5 grounded adds
    /// EXACTLY 1.0 a tick (2.5f * 0.4f rounds to 1.0f — the pinned
    /// cadence avoids the 1.2/tick f32-vs-double ulp drift at the
    /// 6/12/18/24 band edges) — the row-1 band lands at tick 6
    /// (counter 6.0), row 2 at tick 12, row 3 at tick 18; tick 24
    /// hits the >= 24 else: the counter zeroes but the row HOLDS 3,
    /// then tick 25 restarts at row 0.
    #[test]
    fn f26s7f_the_wall_scorpion_bands_and_row_hold() {
        let k = FindFrameExtra7fKind::DesertScorpionWall;
        assert_eq!(
            rows_over(k, input(2.5, 0.0), 25),
            vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 0]
        );
        // The else-quirk edge: counter 23 + 1.0 = 24 >= 24 → the
        // counter zeroes, the row HOLDS 3 (no write).
        let mut s = FrameState { row: 3, counter: 23.0, ..Default::default() };
        find_frame_extra7f(k, &mut s, &input(2.5, 0.0));
        assert_eq!((s.row, s.counter), (3, 0.0));
        // The AIR feeds the same accumulator: vx 0, vy -2 adds 0.8 a
        // tick — row 1 lands at tick 8 (counter 6.4). No air branch.
        assert_eq!(rows_over(k, input(0.0, -2.0), 8), vec![0, 0, 0, 0, 0, 0, 0, 1]);
        // The idle adds 0 — row/counter HELD (no idle branch).
        let mut s = FrameState { row: 2, counter: 13.0, ..Default::default() };
        find_frame_extra7f(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (2, 13.0));
    }

    /// The Desert Beast rides the VERBATIM 7c antlion machine on the
    /// 8-row sheet: vx 3 adds 2.5 a tick, the > 6 step lands every
    /// 3rd tick over rows 2..7, wrapping at row >= 8 back to 2.
    /// Replay: [2,2,3,3,3,4,4,4,5,5,5,6,6,6,7,7,7,2,2].
    #[test]
    fn f26s7f_the_beast_walk_bands_and_eight_row_wrap() {
        let k = FindFrameExtra7fKind::DesertBeast;
        assert_eq!(
            rows_over(k, input(3.0, 0.0), 19),
            vec![2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6, 7, 7, 7, 2, 2]
        );
        // The wrap edge: row 7 + counter 5 → +2.5 = 7.5 > 6 → row 8
        // → >= 8 → 2, counter 0 (the 8-row count, NOT the 7c 6).
        let mut s = FrameState { row: 7, counter: 5.0, ..Default::default() };
        find_frame_extra7f(k, &mut s, &input(3.0, 0.0));
        assert_eq!((s.row, s.counter), (2, 0.0));
    }

    /// The beast's air/idle pins leave the counter UNTOUCHED (the 7c
    /// verbatim quirk); the sprite copies are SIGN-GATED — direction
    /// and vx must agree, the mismatch keeps the old sprite.
    #[test]
    fn f26s7f_the_beast_pins_and_sign_gates() {
        let k = FindFrameExtra7fKind::DesertBeast;
        // The air: row 1, counter 5 HELD, sprite UNTOUCHED.
        let mut s = FrameState { row: 4, counter: 5.0, sprite_dir: 1, ..Default::default() };
        find_frame_extra7f(k, &mut s, &input(-3.0, 2.0));
        assert_eq!((s.row, s.counter, s.sprite_dir), (1, 5.0, 1));
        // The idle: row 0, counter 5 HELD.
        let mut s = FrameState { row: 4, counter: 5.0, ..Default::default() };
        find_frame_extra7f(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 5.0));
        // The mismatch: direction +1, vx -3 — the sprite keeps -1,
        // the walk still runs (counter 0 + 1 + 1.5 = 2.5).
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        find_frame_extra7f(k, &mut s, &facing(-3.0, 0.0, 1));
        assert_eq!((s.row, s.counter, s.sprite_dir), (2, 2.5, -1));
        // The agreement: direction -1, vx -3 → sprite -1.
        let mut s = FrameState { sprite_dir: 1, ..Default::default() };
        find_frame_extra7f(k, &mut s, &facing(-3.0, 0.0, -1));
        assert_eq!(s.sprite_dir, -1);
    }

    /// The Demon Tax Collector's num108 = 12 stride: vx 1 adds 3 a
    /// tick, the > 12 step lands every 5th tick (vs the group's > 6
    /// every 3rd); the lift-to-2 and the FULL-8 wrap-to-2 stand.
    /// Replay: [2,2,2,2,3,3,3,3,3,4,4,4,4,4,5].
    #[test]
    fn f26s7f_the_tax_collector_twelve_stride() {
        let k = FindFrameExtra7fKind::DemonTaxCollector;
        assert_eq!(
            rows_over(k, input(1.0, 0.0), 15),
            vec![2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5]
        );
        // The wrap edge: row 7 + counter 11 → +3 = 14 > 12 → row 8
        // → >= 8 → 2, counter 0.
        let mut s = FrameState { row: 7, counter: 11.0, ..Default::default() };
        find_frame_extra7f(k, &mut s, &input(1.0, 0.0));
        assert_eq!((s.row, s.counter), (2, 0.0));
        // The grounded flip: direction -1 rides (vx -1 agrees).
        let mut s = FrameState { sprite_dir: 1, ..Default::default() };
        find_frame_extra7f(k, &mut s, &facing(-1.0, 0.0, -1));
        assert_eq!(s.sprite_dir, -1);
    }

    /// The tax collector's idle zeroes BOTH row and counter (NOT the
    /// 140/489 row-1 pair); the air pins row 1 + zeroes the counter
    /// (NOT in the :69647 row-0 pin list) with the sprite UNTOUCHED.
    #[test]
    fn f26s7f_the_tax_collector_idle_and_air_pins() {
        let k = FindFrameExtra7fKind::DemonTaxCollector;
        let mut s = FrameState { row: 5, counter: 7.0, sprite_dir: -1, ..Default::default() };
        find_frame_extra7f(k, &mut s, &facing(0.0, 0.0, 0));
        assert_eq!((s.row, s.counter, s.sprite_dir), (0, 0.0, -1));
        let mut s = FrameState { row: 5, counter: 7.0, sprite_dir: 1, ..Default::default() };
        find_frame_extra7f(k, &mut s, &facing(3.0, -2.0, -1));
        assert_eq!((s.row, s.counter, s.sprite_dir), (1, 0.0, 1));
        let mut s = FrameState { row: 5, counter: 7.0, ..Default::default() };
        find_frame_extra7f(k, &mut s, &input(3.0, 2.0));
        assert_eq!((s.row, s.counter), (1, 0.0));
    }

    /// The spiked slime's num2 motion class (the extra-3 verbatim
    /// twin): idle +1 a tick steps every 8th; moving/airborne +2
    /// steps every 4th; the attack +3 steps at ticks 3/6/8 — all over
    /// the FULL 2-row sheet wrapping to 0.
    #[test]
    fn f26s7f_the_spiked_slime_num2_feed() {
        let k = FindFrameExtra7fKind::SlimeSpiked;
        assert_eq!(rows_over(k, input(0.0, 0.0), 9), vec![0, 0, 0, 0, 0, 0, 0, 1, 1]);
        assert_eq!(rows_over(k, input(1.0, 0.0), 5), vec![0, 0, 0, 1, 1]);
        assert_eq!(rows_over(k, input(0.0, -2.0), 5), vec![0, 0, 0, 1, 1]);
        let i = FrameInput { ai_action: 1, ..Default::default() };
        assert_eq!(rows_over(k, i, 9), vec![0, 0, 1, 1, 1, 0, 0, 0, 1]);
        // No sprite write (the 302/304 snap is the town slimes').
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        find_frame_extra7f(k, &mut s, &input(2.0, 0.0));
        assert_eq!(s.sprite_dir, -1);
    }

    /// The Bride rides the VERBATIM wave-A zombie walker: vx 3 adds 3
    /// a tick — rows 0 (ticks 1-2, counter < 8), 1 (3-5, < 16), 2
    /// (6-7, < 24), 1 (8-10, < 32 — tick 8's counter 24 lands the
    /// FOURTH band, not row 2); tick 11 hits the >= 32 else: the
    /// counter zeroes but the row HOLDS 1, then tick 12 restarts at
    /// row 0.
    #[test]
    fn f26s7f_the_bride_walk_bands_and_lap_hold() {
        let k = FindFrameExtra7fKind::TheBride;
        assert_eq!(
            rows_over(k, input(3.0, 0.0), 12),
            vec![0, 0, 1, 1, 1, 2, 2, 1, 1, 1, 1, 0]
        );
        // The lap edge: counter 30 + 3 = 33 >= 32 → zero, row HOLDS 1.
        let mut s = FrameState { row: 1, counter: 30.0, ..Default::default() };
        find_frame_extra7f(k, &mut s, &input(3.0, 0.0));
        assert_eq!((s.row, s.counter), (1, 0.0));
    }

    /// The Bride's air/turn pin: airborne OR moving against the
    /// facing zeroes the counter and pins row 2; the grounded flip is
    /// ±1-gated (0 leaves the sprite); the stand zeroes both.
    #[test]
    fn f26s7f_the_bride_air_turn_and_stand_pins() {
        let k = FindFrameExtra7fKind::TheBride;
        // The air: row 2, counter 0, sprite UNTOUCHED (no grounded
        // flip — direction -1 never copies in the air).
        let mut s = FrameState { row: 0, counter: 9.0, sprite_dir: 1, ..Default::default() };
        find_frame_extra7f(k, &mut s, &facing(-3.0, 2.0, -1));
        assert_eq!((s.row, s.counter, s.sprite_dir), (2, 0.0, 1));
        // The turn-around: grounded, facing -1 but vx +3 → row 2.
        let mut s = FrameState { row: 1, counter: 9.0, sprite_dir: 1, ..Default::default() };
        find_frame_extra7f(k, &mut s, &facing(3.0, 0.0, -1));
        assert_eq!((s.row, s.counter, s.sprite_dir), (2, 0.0, -1));
        // The stand: row 0 + counter 0; direction 0 keeps sprite -1.
        let mut s = FrameState { row: 2, counter: 9.0, sprite_dir: -1, ..Default::default() };
        find_frame_extra7f(k, &mut s, &facing(0.0, 0.0, 0));
        assert_eq!((s.row, s.counter, s.sprite_dir), (0, 0.0, -1));
    }

    /// The seam maps exactly this wave's 5 ids — the port-covered
    /// 533, the 7c-covered 508, the prior-wave 526/530, the group
    /// co-members 165/237/238/240/3/537 and the no-case 515 stay
    /// `None`.
    #[test]
    fn f26s7f_the_seam_maps_exactly_the_slice() {
        for (id, k) in [
            (531, FindFrameExtra7fKind::DesertScorpionWall),
            (532, FindFrameExtra7fKind::DesertBeast),
            (534, FindFrameExtra7fKind::DemonTaxCollector),
            (535, FindFrameExtra7fKind::SlimeSpiked),
            (536, FindFrameExtra7fKind::TheBride),
        ] {
            assert_eq!(find_frame_kind_extra7f(id), Some(k));
            assert_eq!(k.vanilla_type(), id);
        }
        for id in [3, 165, 237, 238, 240, 508, 515, 526, 530, 533, 537] {
            assert_eq!(find_frame_kind_extra7f(id), None, "type {id}");
        }
    }

    /// The 5-kind strip sweep: every kind ticks 27 input combos
    /// (vx 0/1/3 × vy 0/-2/2 × direction 1/-1/0) for 40 ticks each
    /// from a fresh state — every row stays inside its own sheet
    /// (the 7e 10-tick sweep widened to 40: the wall scorpion's
    /// 20-tick band cycle + the bride's 11-tick lap need the longer
    /// window to exercise the else-quirks).
    #[test]
    fn f26s7f_the_5_kind_strip_sweep_stays_in_sheet() {
        let kinds = [
            FindFrameExtra7fKind::DesertScorpionWall,
            FindFrameExtra7fKind::DesertBeast,
            FindFrameExtra7fKind::DemonTaxCollector,
            FindFrameExtra7fKind::SlimeSpiked,
            FindFrameExtra7fKind::TheBride,
        ];
        for k in kinds {
            let mut combo = 0;
            for vx in [0.0f32, 1.0, 3.0] {
                for vy in [0.0f32, -2.0, 2.0] {
                    for direction in [1i32, -1, 0] {
                        let mut s = FrameState::default();
                        let i = FrameInput { vx, vy, direction, ai_action: 0 };
                        for _ in 0..40 {
                            find_frame_extra7f(k, &mut s, &i);
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
