//! W4-f26 slice 7c — FindFrame extra wave 7c: the NEXT 15 TRULY
//! unclaimed missing case ids of `NPC.FindFrame` (`NPC.cs:66684-77370`),
//! counted off the ascending census after the d44-d49/e13/e14/f26/f27
//! claims AND the landed d38/d40 roster arms — re-probed at THIS slice's
//! census time against the LATEST pushed tips (the 7b header read the
//! d40 tip at batch 8; batches 9-12 have since landed).
//!
//! The re-census (mechanical, the 7b method): 602 C# top-level `case`
//! ids (`NPC.cs:66705`'s two-tab labels) − 248 port-covered (the
//! `npc.rs::find_frame` arms mapped through the `npc/data.rs:2992`
//! cs_id hand rows + the bestiary net ids + the wave-A/B module
//! rosters + the five folded segment families 135/246/249/371/373
//! whose arms ride the folded net ids) − 75 d44-d49 claims = 279 (the
//! e13 base); − the 75 deduped e13/e14/f26×3/f27-wave-4 claim ids =
//! 204; − f27 slice-7's pushed claim (316-318/322-324/333-337 +
//! 437/438/440/442, tip `ef07e06b`) = 189; − the d38 LANDED arms (the
//! in-band remainder 297..304 ALL wired on the pushed d38 tip
//! `8176c078`) = 181; − the d40 LANDED arms 443..454/460..470/472
//! (batches 3-8) = 157 (the 7b census); − THIS slice-7b's own pushed
//! claim 473-477/479-488 (15 ids, tip `b9b49f66`) = **142 unclaimed**.
//!
//! The re-probe exclusions (all inside the already-subtracted sets —
//! no double-count):
//! * d40 tip `8fe1d6d3` batches 9-12 landed INLINE `npc.rs::find_frame`
//!   arms for 473..488 — but only 480 Medusa / 481 GreekSkeleton /
//!   482 GraniteGolem are FAITHFUL machines; the other twelve are
//!   row-0 stubs (the aiStyle machines UNWIRED, the style73
//!   precedent). The arm-by-arm keep-one verdict is published in
//!   docs/ANALYSIS.md "W4-f26 FindFrame extra wave 7c": for 480/481/482
//!   the d40 INLINE arms and this branch's extra7b module arms are
//!   BOTH faithful and behaviour-identical (row-index vs pixel-row
//!   convention only) — keep the INLINE arms at merge (already in
//!   `NpcKind` + wired through `npc_rows`), drop the three extra7b
//!   module arms to parity references; for the twelve stub kinds the
//!   extra7b module is the ONLY faithful machine — keep the module
//!   arms, the stubs are superseded (the f27 extra-8 d40-merger-notes
//!   precedent for 443-448).
//! * f27 tip `72448094` slice-8 CLAIMS 443-454/460/461/462 — already
//!   inside the d40-landed subtraction (its faithful arms supersede
//!   the d40 stubs there; the claim set dedupes).
//! * d41/d42 have NO pushed refs (`git ls-remote origin` — only local
//!   branches at the wave-3 land base `a725acb0`, zero FindFrame
//!   arms); d43's pushed tip IS `a725acb0` (no work yet). No 494+
//!   kind has a landed arm on ANY pushed tip.
//!
//! THIS wave owns the FIRST 15 of the remaining ascending list from
//! 494 up — **494/495/496/497/498/499/500/501/502/503/504/505/506/
//! 508/509** — the Crawdad pair, the Giant Shelly pair, the nine
//! Salamanders, the Giant Walking Antlion and the Giant Flying
//! Antlion (`ID/NPCID.cs:11661-11695`). 491 PirateShip / 493
//! LunarTowerStardust / 507 LunarTowerNebula have NO case labels in
//! the switch — never in the census. Landed 5 kinds per commit.
//!
//! Shape: the kinds are NOT in `NpcKind` yet (the d41 roster wave owns
//! them), so — the `npc_findframe_extra7b.rs` precedent — the frame
//! machines land first as pure functions over a small
//! [`FrameState`]/[`FrameInput`] pair. The wiring seam is
//! [`find_frame_kind_extra7c`]: when a kind joins `NpcKind`, its
//! `find_frame` arm copies the state fields out, runs one tick and
//! copies them back (the `Npc::find_frame_wave_a` bridge precedent).
//! No arm draws `Main.rand` (mechanical grep over all five cited body
//! ranges = ZERO), so the wiring cannot move a stream.
//!
//! Row convention: the port tracks frame ROW indices, not pixels —
//! the C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer,
//! `:67132-67133`'s rows), counted entry-by-entry in
//! [`FindFrameExtra7cKind::frame_count`] and cross-checked against the
//! port's own `render/entities.rs::NPC_FRAME_COUNT` table.
//!
//! No world reads this wave: none of the five machines touches
//! `Framing`/`Main.tile` — the `FrameInput` needs only vx/vy/
//! direction/ai (the 7b slope/frame_x36 seams are NOT carried).

// The wiring is a later round (the kinds are not in `NpcKind` yet) —
// the `npc_findframe_extra7b.rs` precedent.
#![allow(dead_code)]

/// The extra-wave-7c kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra7cKind {
    /// `NPCID.Crawdad` = 494 (`NPCID.cs:11661`) — the 494/495 group
    /// (`NPC.cs:69657`).
    Crawdad,
    /// `NPCID.Crawdad2` = 495 (`NPCID.cs:11663`) — the same group
    /// (`NPC.cs:69658`).
    Crawdad2,
    /// `NPCID.GiantShelly` = 496 (`NPCID.cs:11665`) — the 496/497
    /// group (`NPC.cs:69994`).
    GiantShelly,
    /// `NPCID.GiantShelly2` = 497 (`NPCID.cs:11667`) — the same group
    /// (`NPC.cs:69995`).
    GiantShelly2,
    /// `NPCID.Salamander` = 498 (`NPCID.cs:11669`) — the 498..506
    /// nine-kind group (`NPC.cs:69849`).
    Salamander,
}

impl FindFrameExtra7cKind {
    /// The vanilla `NPC.type` id.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::Crawdad => 494,
            Self::Crawdad2 => 495,
            Self::GiantShelly => 496,
            Self::GiantShelly2 => 497,
            Self::Salamander => 498,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// counted entry-by-entry): 494/495 = 8 (`:67132`'s 490-499 row,
    /// entries 5/6), 496/497 = 12 (`:67132`, entries 7/8), 498 = 11
    /// (`:67132`, entry 9).
    pub fn frame_count(self) -> u32 {
        match self {
            Self::Crawdad | Self::Crawdad2 => 8,
            Self::GiantShelly | Self::GiantShelly2 => 12,
            Self::Salamander => 11,
        }
    }
}

/// The vanilla-id wiring seam: `Some(kind)` for exactly this wave's
/// ids (the `find_frame_kind_extra7b` precedent). The wiring round
/// hooks it at the `npc.rs::find_frame` head by `cs_id`.
pub fn find_frame_kind_extra7c(vanilla_type: i16) -> Option<FindFrameExtra7cKind> {
    Some(match vanilla_type {
        494 => FindFrameExtra7cKind::Crawdad,
        495 => FindFrameExtra7cKind::Crawdad2,
        496 => FindFrameExtra7cKind::GiantShelly,
        497 => FindFrameExtra7cKind::GiantShelly2,
        498 => FindFrameExtra7cKind::Salamander,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection` and
/// `rotation` (the shelly state-0 zero snap). The wiring step copies
/// them to/from `Npc`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter`.
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — the shelly state-0 arm zeroes it every tick.
    pub rotation: f32,
}

/// The per-tick inputs the arms read: `velocity`, `direction` and the
/// `ai` slots (the crawdad `ai[2]` attack gate, the shelly `ai[0]`
/// state / `ai[1]` shell bands, the salamander `ai[2]` attack gate /
/// `ai[1]` pose bands). No world reads this wave.
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone in
    /// the arms that gate on `direction == 1` / `== -1`; the shelly
    /// grounded copy is UNCONDITIONAL).
    pub direction: i32,
    pub ai: [f32; 4],
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1, ai: [0.0; 4] }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-7c kinds — one
/// tick.
pub fn find_frame_extra7c(kind: FindFrameExtra7cKind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // `case 494/495` (`NPC.cs:69657-69733`): the Crawdad pair.
        // The `ai[2] > 0` attack pose (`:69659-69673`) pins row 5/6/7
        // off the `ai[2]` <7/<14/<20 bands and BREAKS — the >= 20
        // edge writes NOTHING (the row/counter hold). The grounded
        // ±1 flip (`:69674-69684`); the air OR backpedal branch
        // (`:69685-69697` — `vy != 0` OR direction opposes vx) zeroes
        // the counter and pins row 2 rising (`vy < 0`) / row 3 else;
        // the idle (`:69698-69703`) zeroes the counter and pins row 0.
        // The walk (`:69704-69733`) accumulates |vx| — NO +1 — over
        // the <8/<16/<24/<32/<39 bands rows 0/1/2/3/4; the >= 39
        // else pins row 4 AGAIN and zeroes the counter (the 39-tick
        // cycle, NOT a wrap-to-0). No rotation write.
        FindFrameExtra7cKind::Crawdad | FindFrameExtra7cKind::Crawdad2 => {
            if i.ai[2] > 0.0 {
                // `:69659-69673` — the >= 20 edge holds everything.
                if i.ai[2] < 7.0 {
                    s.row = 5;
                } else if i.ai[2] < 14.0 {
                    s.row = 6;
                } else if i.ai[2] < 20.0 {
                    s.row = 7;
                }
                return;
            }
            if i.vy == 0.0 {
                // `:69674-69684`.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
            }
            if i.vy != 0.0
                || (i.direction == -1 && i.vx > 0.0)
                || (i.direction == 1 && i.vx < 0.0)
            {
                // `:69685-69697`.
                s.counter = 0.0;
                s.row = if i.vy < 0.0 { 2 } else { 3 };
                return;
            }
            if i.vx == 0.0 {
                // `:69698-69703`.
                s.counter = 0.0;
                s.row = 0;
                return;
            }
            // `:69704-69733`.
            s.counter += i.vx.abs();
            if s.counter < 8.0 {
                s.row = 0;
            } else if s.counter < 16.0 {
                s.row = 1;
            } else if s.counter < 24.0 {
                s.row = 2;
            } else if s.counter < 32.0 {
                s.row = 3;
            } else if s.counter < 39.0 {
                s.row = 4;
            } else {
                s.row = 4;
                s.counter = 0.0;
            }
        }
        // `case 496/497` (`NPC.cs:69994-70066`): the Giant Shelly
        // pair. State `ai[0] == 0` (`:69996-70028`) zeroes the
        // rotation EVERY tick, copies `direction` UNCONDITIONALLY
        // while grounded (0 included), zeroes the counter while
        // RISING (`vy < 0` — the fall keeps it), then accumulates
        // `|vx| * 1.1` over the <6/<12/<18/<24 bands rows 0/1/2/3;
        // the >= 24 else zeroes the counter WITHOUT a row write —
        // the reset tick HOLDS the row. State 1 (`:70029-70044`)
        // zeroes the counter and pins row 4/5/6 off the `ai[1]`
        // <10/<20 bands; state 5 (`:70045-70060`) zeroes the counter
        // and pins row 10/11/3 off the same bands (the >= 20 edge
        // falls back to row 3 — the walk's last row); every OTHER
        // state (`:70061-70065`) zeroes the counter and pins row 7.
        // The state arms never touch sprite/rotation (only state 0
        // writes them).
        FindFrameExtra7cKind::GiantShelly | FindFrameExtra7cKind::GiantShelly2 => {
            if i.ai[0] == 0.0 {
                // `:69996-70028`.
                s.rotation = 0.0;
                if i.vy == 0.0 {
                    s.sprite_dir = i.direction;
                } else if i.vy < 0.0 {
                    s.counter = 0.0;
                }
                s.counter += i.vx.abs() * 1.1;
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
            } else if i.ai[0] == 1.0 {
                // `:70029-70044`.
                s.counter = 0.0;
                s.row = if i.ai[1] < 10.0 {
                    4
                } else if i.ai[1] < 20.0 {
                    5
                } else {
                    6
                };
            } else if i.ai[0] == 5.0 {
                // `:70045-70060`.
                s.counter = 0.0;
                s.row = if i.ai[1] < 10.0 {
                    10
                } else if i.ai[1] < 20.0 {
                    11
                } else {
                    3
                };
            } else {
                // `:70061-70065`.
                s.counter = 0.0;
                s.row = 7;
            }
        }
        // `case 498..506` (`NPC.cs:69849-69907`): the nine
        // Salamanders. The grounded ±1 flip (`:69858-69866`); the
        // `ai[2] > 0` attack pose (`:69867-69884`) copies `direction`
        // UNCONDITIONALLY and pins row 8/9/8 off the `ai[1]`
        // <12/<34 bands (the >= 34 edge returns to row 8) with the
        // counter ZEROED; the idle (`:69885-69889`) pins row 0
        // WITHOUT touching the counter; the walk (`:69890-69900`)
        // accumulates `|vx| * 1` — NO +1 — the > 4 counter steps a
        // row, wrapping at the STRICT row > 7 back to 0 (rows 8..10
        // are the attack/air poses); the air (`:69902-69906`) zeroes
        // the counter and pins row 10. No rotation write.
        FindFrameExtra7cKind::Salamander => {
            if i.vy == 0.0 {
                // `:69858-69866`.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                if i.ai[2] > 0.0 {
                    // `:69867-69884`.
                    s.sprite_dir = i.direction;
                    s.row = if i.ai[1] < 12.0 {
                        8
                    } else if i.ai[1] < 34.0 {
                        9
                    } else {
                        8
                    };
                    s.counter = 0.0;
                } else if i.vx == 0.0 {
                    // `:69885-69889` — the counter is NOT touched.
                    s.row = 0;
                } else {
                    // `:69890-69900`.
                    s.counter += i.vx.abs() * 1.0;
                    if s.counter > 4.0 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                    if s.row > 7 {
                        s.row = 0;
                    }
                }
            } else {
                // `:69902-69906`.
                s.counter = 0.0;
                s.row = 10;
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

    fn rows_over(kind: FindFrameExtra7cKind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra7c(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The frame-count table agrees with the port's verbatim
    /// `NPC_FRAME_COUNT` (`Main.cs:67082` initializer cross-check).
    #[test]
    fn f26s7c_frame_counts_match_the_port_table() {
        let table = crate::render::entities::NPC_FRAME_COUNT;
        for k in [
            FindFrameExtra7cKind::Crawdad,
            FindFrameExtra7cKind::Crawdad2,
            FindFrameExtra7cKind::GiantShelly,
            FindFrameExtra7cKind::GiantShelly2,
            FindFrameExtra7cKind::Salamander,
        ] {
            assert_eq!(
                k.frame_count(),
                table[k.vanilla_type() as usize],
                "{k:?} ({})",
                k.vanilla_type()
            );
        }
    }

    /// The crawdad walk is the |vx| NO-+1 accumulator over the
    /// <8/<16/<24/<32/<39 bands; the >= 39 else pins row 4 AGAIN and
    /// zeroes the counter — the cycle restarts at row 0 on the NEXT
    /// tick (replay: vx=3 → [0,0,1,1,1,2,2,3,3,3,4,4,4,0,0,1]).
    #[test]
    fn f26s7c_the_crawdad_walk_bands_and_39_reset() {
        for k in [FindFrameExtra7cKind::Crawdad, FindFrameExtra7cKind::Crawdad2] {
            assert_eq!(
                rows_over(k, input(3.0, 0.0), 16),
                vec![0, 0, 1, 1, 1, 2, 2, 3, 3, 3, 4, 4, 4, 0, 0, 1]
            );
            // The idle zeroes the counter + pins row 0; the grounded
            // flip rides direction -1.
            let mut s = FrameState { row: 4, counter: 2.5, sprite_dir: 1, rotation: 0.0 };
            let i = FrameInput { direction: -1, ..input(0.0, 0.0) };
            find_frame_extra7c(k, &mut s, &i);
            assert_eq!((s.row, s.counter, s.sprite_dir), (0, 0.0, -1));
        }
    }

    /// The crawdad attack ladder pins rows 5/6/7 off the ai[2]
    /// <7/<14/<20 bands; the >= 20 edge writes NOTHING (the row and
    /// counter HOLD — the break-before-write quirk); the air/backpedal
    /// branch zeroes the counter and pins row 2 rising / row 3 else.
    #[test]
    fn f26s7c_the_crawdad_attack_ladder_and_backpedal() {
        let k = FindFrameExtra7cKind::Crawdad;
        for (ai2, row) in [(5.0f32, 5u32), (10.0, 6), (17.0, 7)] {
            let mut s = FrameState::default();
            let i = FrameInput { ai: [0.0, 0.0, ai2, 0.0], ..input(3.0, 0.0) };
            find_frame_extra7c(k, &mut s, &i);
            assert_eq!(s.row, row);
            assert_eq!(s.counter, 0.0);
        }
        // The >= 20 hold: row 4 + counter 2.5 + sprite -1 all survive.
        let mut s = FrameState { row: 4, counter: 2.5, sprite_dir: -1, rotation: 0.0 };
        let i = FrameInput { ai: [0.0, 0.0, 25.0, 0.0], ..input(3.0, 0.0) };
        find_frame_extra7c(k, &mut s, &i);
        assert_eq!((s.row, s.counter, s.sprite_dir), (4, 2.5, -1));
        // The backpedal: direction 1 + vx < 0 pins row 3 grounded.
        let mut s = FrameState { row: 4, counter: 2.5, sprite_dir: 1, rotation: 0.0 };
        find_frame_extra7c(k, &mut s, &input(-3.0, 0.0));
        assert_eq!((s.row, s.counter, s.sprite_dir), (3, 0.0, 1));
        // The air: rising row 2, falling row 3, sprite untouched.
        let mut s = FrameState { row: 4, counter: 2.5, sprite_dir: 1, rotation: 0.0 };
        find_frame_extra7c(k, &mut s, &input(3.0, -2.0));
        assert_eq!((s.row, s.counter, s.sprite_dir), (2, 0.0, 1));
        let mut s = FrameState { row: 4, counter: 2.5, sprite_dir: 1, rotation: 0.0 };
        let i = FrameInput { direction: -1, ..input(3.0, 2.0) };
        find_frame_extra7c(k, &mut s, &i);
        assert_eq!((s.row, s.counter, s.sprite_dir), (3, 0.0, 1));
    }

    /// The shelly state-0 walk is the |vx|*1.1 accumulator over the
    /// <6/<12/<18/<24 bands; the >= 24 reset tick HOLDS the row
    /// (replay: vx=2 → [0,0,1,1,1,2,2,2,3,3,3,0,0,1]); the rotation
    /// zeroes every tick and the grounded sprite copy is
    /// UNCONDITIONAL (0 included).
    #[test]
    fn f26s7c_the_shelly_walk_and_reset_hold() {
        for k in [FindFrameExtra7cKind::GiantShelly, FindFrameExtra7cKind::GiantShelly2] {
            assert_eq!(
                rows_over(k, input(2.0, 0.0), 14),
                vec![0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 0, 0, 1]
            );
            let mut s = FrameState { row: 3, counter: 20.0, sprite_dir: 1, rotation: 0.3 };
            let i = FrameInput { direction: 0, ..input(2.0, 0.0) };
            find_frame_extra7c(k, &mut s, &i);
            assert_eq!((s.row, s.counter, s.sprite_dir, s.rotation), (3, 22.0, 0, 0.0));
        }
    }

    /// The shelly rise zeroes the counter BEFORE the accumulate (the
    /// fall keeps it); states 1/5 pin the shell ladders (the state-5
    /// >= 20 edge falls back to row 3); every other state pins row 7.
    #[test]
    fn f26s7c_the_shelly_rise_and_shell_states() {
        let k = FindFrameExtra7cKind::GiantShelly;
        // Rising: counter 20 → 0 → 2.2, row 0, sprite HELD (air).
        let mut s = FrameState { row: 3, counter: 20.0, sprite_dir: 1, rotation: 0.3 };
        let i = FrameInput { direction: -1, ..input(2.0, -2.0) };
        find_frame_extra7c(k, &mut s, &i);
        assert_eq!((s.row, s.counter, s.sprite_dir), (0, 2.2, 1));
        // Falling: counter 20 + 2.2 = 22.2 < 24 → row 3, sprite HELD.
        let mut s = FrameState { row: 3, counter: 20.0, sprite_dir: 1, rotation: 0.3 };
        let i = FrameInput { direction: -1, ..input(2.0, 2.0) };
        find_frame_extra7c(k, &mut s, &i);
        assert_eq!((s.row, s.counter, s.sprite_dir), (3, 22.2, 1));
        // State 1: ai[1] 5/15/25 → rows 4/5/6, counter zeroed.
        for (a1, row) in [(5.0f32, 4u32), (15.0, 5), (25.0, 6)] {
            let mut s = FrameState { row: 0, counter: 5.0, sprite_dir: 1, rotation: 0.3 };
            let i = FrameInput { ai: [1.0, a1, 0.0, 0.0], ..input(2.0, 0.0) };
            find_frame_extra7c(k, &mut s, &i);
            assert_eq!((s.row, s.counter, s.rotation), (row, 0.0, 0.3));
        }
        // State 5: ai[1] 5/15/25 → rows 10/11/3.
        for (a1, row) in [(5.0f32, 10u32), (15.0, 11), (25.0, 3)] {
            let mut s = FrameState { row: 0, counter: 5.0, sprite_dir: 1, rotation: 0.3 };
            let i = FrameInput { ai: [5.0, a1, 0.0, 0.0], ..input(2.0, 0.0) };
            find_frame_extra7c(k, &mut s, &i);
            assert_eq!((s.row, s.counter), (row, 0.0));
        }
        // Every other state: row 7, counter zeroed, rotation HELD.
        let mut s = FrameState { row: 0, counter: 5.0, sprite_dir: 1, rotation: 0.3 };
        let i = FrameInput { ai: [2.0, 0.0, 0.0, 0.0], ..input(2.0, 0.0) };
        find_frame_extra7c(k, &mut s, &i);
        assert_eq!((s.row, s.counter, s.rotation), (7, 0.0, 0.3));
    }

    /// The salamander walk is the |vx|*1 NO-+1 accumulator with the
    /// > 4 step and the STRICT > 7 wrap (replay: vx=2 →
    /// [0,0,1,1,1,2,2,2,3,3,3,4]); the attack ladder pins rows 8/9/8
    /// off the ai[1] <12/<34 bands with the UNCONDITIONAL direction
    /// copy; the idle pins row 0 WITHOUT touching the counter; the
    /// air pins row 10.
    #[test]
    fn f26s7c_the_salamander_walk_ladder_and_idle_hold() {
        let k = FindFrameExtra7cKind::Salamander;
        assert_eq!(
            rows_over(k, input(2.0, 0.0), 12),
            vec![0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4]
        );
        // The attack ladder: ai[1] 5/20/40 → rows 8/9/8, sprite -1.
        for (a1, row) in [(5.0f32, 8u32), (20.0, 9), (40.0, 8)] {
            let mut s = FrameState { row: 3, counter: 2.0, sprite_dir: 1, rotation: 0.0 };
            let i = FrameInput {
                direction: -1,
                ai: [0.0, a1, 5.0, 0.0],
                ..input(2.0, 0.0)
            };
            find_frame_extra7c(k, &mut s, &i);
            assert_eq!((s.row, s.counter, s.sprite_dir), (row, 0.0, -1));
        }
        // The idle: row 0, counter 2.5 HELD.
        let mut s = FrameState { row: 3, counter: 2.5, sprite_dir: 1, rotation: 0.0 };
        find_frame_extra7c(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 2.5));
        // The air: row 10, counter zeroed.
        let mut s = FrameState { row: 3, counter: 2.5, sprite_dir: 1, rotation: 0.0 };
        find_frame_extra7c(k, &mut s, &input(2.0, -2.0));
        assert_eq!((s.row, s.counter), (10, 0.0));
    }

    /// The seam maps exactly this batch's ids — the no-case 493/507,
    /// the port-covered 490 and the next-batch 499 stay `None`.
    #[test]
    fn f26s7c_the_seam_maps_exactly_the_slice() {
        for (id, k) in [
            (494, FindFrameExtra7cKind::Crawdad),
            (495, FindFrameExtra7cKind::Crawdad2),
            (496, FindFrameExtra7cKind::GiantShelly),
            (497, FindFrameExtra7cKind::GiantShelly2),
            (498, FindFrameExtra7cKind::Salamander),
        ] {
            assert_eq!(find_frame_kind_extra7c(id), Some(k));
            assert_eq!(k.vanilla_type(), id);
        }
        for id in [490, 493, 499, 507, 508] {
            assert_eq!(find_frame_kind_extra7c(id), None, "type {id}");
        }
    }
}
