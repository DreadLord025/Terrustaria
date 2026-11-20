//! W4-d48 FindFrame coverage wave C — the FIRST 12 missing HIGH-id
//! kinds of `NPC.FindFrame` (`NPC.cs:66684-77370`) in the 350-550 band:
//! the surface critters. Ascending census against the port's `npc.rs`
//! `find_frame` match: 350-352 ride the frost arms, 353/368/369 the
//! humanoid town group, so the first gap starts at 355. This module
//! owns exactly the 12 kinds listed in [`FindFrameKind::vanilla_type`];
//! the next 12 (367 Scorpion onward) ride `task/W4-d49-findframe-high-2`.
//!
//! Shape: the kinds are NOT in `NpcKind` yet (the roster wave
//! `task/W4-d39-npc-kinds-351-420` owns them), so — the
//! `npc_findframe_b.rs` (S1-D-3 wave B) precedent — the frame machines
//! land first as pure functions over a [`FrameState`]/[`FrameInput`]
//! pair and the `npc.rs::find_frame` wiring is a separate step: when a
//! kind joins `NpcKind`, its arm calls [`find_frame_c`] with the NPC's
//! fields and copies `row`/`counter`/`sprite_dir`/`rotation`/`local_ai0`
//! back. No RNG is touched (none of these arms draws `Main.rand`), so
//! the wiring cannot move a stream.
//!
//! Row convention: the port tracks frame ROW indices, not pixels — the
//! C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer),
//! counted entry-by-entry in [`FindFrameKind::frame_count`] and
//! cross-checked against the port's `NPC_FRAME_COUNT` in the tests.
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "W4-d48 FindFrame
//! wave C"): the `IsABestiaryIconDummy` branches (the port has no
//! bestiary dummy — none of these 12 arms carries one anyway), the
//! case-group siblings OUTSIDE the 350-550 band or past the 12-kind
//! cut (654 Lavafly, 444 GoldButterfly, 653/661, 448 GoldWorm, 484
//! EnchantedNightcrawler, 606, 655 MagmaSnail, 445 GoldFrog, 687,
//! 608/609 the seagull/grebe swimmers, 367 Scorpion — d49+), and the
//! worm arm's tile probe is an INPUT ([`FrameInput::slope`]): the C#
//! reads `Framing.GetTileSafely((int)Center.X/16, (int)position.Y/16)`
//! and falls through to the tile BELOW when that tile's slope is 0
//! (`NPC.cs:72807-72815`); the caller resolves that probe, the pure
//! arm consumes the resolved slope. The snail arm (359/360) delegates
//! to the already-ported `npc_ai_style67::snail_find_frame`
//! (`NPC.cs:72238-72295`) — one machine, no second copy.

// The `npc.rs::find_frame` wiring is a later wave (the kinds are not
// in `NpcKind` yet) — the `npc_findframe_b.rs` precedent.
#![allow(dead_code)]

/// The 12 wave-C kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows — the port-local
/// `NpcKind` ids do not exist for them yet).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameKind {
    /// `NPCID.Firefly` = 355 (`NPCID.cs:11387`) — the `case 355/358`
    /// firefly arm.
    Firefly,
    /// `NPCID.Butterfly` = 356 — `case 356` (the 8-bank sheet).
    Butterfly,
    /// `NPCID.Worm` = 357 — `case 357` (the slope-aware crawler).
    Worm,
    /// `NPCID.LightningBug` = 358 (`NPCID.cs:11393`) — the `case
    /// 355/358` firefly arm.
    LightningBug,
    /// `NPCID.Snail` = 359 (`NPCID.cs:11395`) — `case 359/360`.
    Snail,
    /// `NPCID.GlowingSnail` = 360 (`NPCID.cs:11397`) — `case 359/360`.
    GlowingSnail,
    /// `NPCID.Frog` = 361 — `case 361`.
    Frog,
    /// `NPCID.Duck` = 362 — `case 362/364` (the walker form).
    Duck,
    /// `NPCID.Duck2` = 363 — `case 363/365` (the swimmer form).
    Duck2,
    /// `NPCID.DuckWhite` = 364 — `case 362/364`.
    DuckWhite,
    /// `NPCID.DuckWhite2` = 365 — `case 363/365`.
    DuckWhite2,
    /// `NPCID.ScorpionBlack` = 366 — `case 366/367` (367 rides d49).
    ScorpionBlack,
}

impl FindFrameKind {
    /// The vanilla `NPC.type` (`ID/NPCID.cs`'s const rows).
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::Firefly => 355,
            Self::Butterfly => 356,
            Self::Worm => 357,
            Self::LightningBug => 358,
            Self::Snail => 359,
            Self::GlowingSnail => 360,
            Self::Frog => 361,
            Self::Duck => 362,
            Self::Duck2 => 363,
            Self::DuckWhite => 364,
            Self::DuckWhite2 => 365,
            Self::ScorpionBlack => 366,
        }
    }

    /// `Main.npcFrameCount[type]` — the row counts counted entry-by-entry
    /// out of the 697-entry initializer (`Main.cs:67082-67153`; entry
    /// `type` sits on line `67084 + type / 10`, slot `type % 10`).
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67119 (entries 350-359): 355 = 4, 356 = 24,
            // 357 = 2, 358 = 4, 359 = 6.
            Self::Firefly => 4,
            Self::Butterfly => 24,
            Self::Worm => 2,
            Self::LightningBug => 4,
            Self::Snail => 6,
            // Main.cs:67120 (entries 360-369): 360 = 6, 361 = 13,
            // 362/363/364/365 = 15, 366 = 4.
            Self::GlowingSnail => 6,
            Self::Frog => 13,
            Self::Duck => 15,
            Self::Duck2 => 15,
            Self::DuckWhite => 15,
            Self::DuckWhite2 => 15,
            Self::ScorpionBlack => 4,
        }
    }
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection`,
/// `rotation`, and the worm arm's `localAI[0]` write
/// (`NPC.cs:72779`/`:72819`/`:72824`). The wiring step copies them
/// to/from `Npc`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter`.
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — the butterfly/duck-swimmer/worm arms write it.
    pub rotation: f32,
    /// `NPC.localAI[0]` — only the worm arm (357) writes it: the `-2f`
    /// pin each tick, lifted to 0 on a slope (`NPC.cs:72779`).
    pub local_ai0: f32,
}

/// The per-tick inputs the arms read: `velocity`, `direction`, `wet`,
/// the butterfly's `ai[2]` bank selector, the firefly's `localAI[2]`
/// glow flag and the worm's resolved tile slope. `ai_action` is not
/// carried — none of these kinds sets it (the `num2` motion class of
/// `NPC.cs:66696-66704` is unused by every wave-C arm).
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone in the
    /// scorpion arm, which gates on `direction == 1` / `== -1`).
    pub direction: i32,
    pub wet: bool,
    /// `NPC.ai[2]` — the butterfly's colour-bank index (1..8).
    pub ai2: f32,
    /// `NPC.localAI[2]` — the firefly's glow flag: `<= 0` shifts the
    /// strip down two rows (`NPC.cs:72705-72708`).
    pub local_ai2: f32,
    /// The worm arm's resolved tile slope (`NPC.cs:72807-72825`): the
    /// caller probes `Framing.GetTileSafely((int)Center.X/16,
    /// (int)position.Y/16)` and, when that tile's `slope() == 0`, the
    /// tile one row BELOW; this is the resulting `slope()` (0 none,
    /// 1 half-rising-right → +0.785, 2 half-rising-left → −0.785).
    pub slope: i32,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self {
            vx: 0.0,
            vy: 0.0,
            direction: 1,
            wet: false,
            ai2: 1.0,
            local_ai2: 0.0,
            slope: 0,
        }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the wave-C kinds — one tick.
pub fn find_frame_c(kind: FindFrameKind, s: &mut FrameState, i: &FrameInput) {
    let count = kind.frame_count();
    match kind {
        // `case 355 : case 358` (`NPC.cs:72688-72709`, shared with 654
        // the Lavafly — out of band): the 4/7 two-row blink, then the
        // `localAI[2] <= 0` glow-off shift of +2 rows (the 4-row sheet
        // is two 2-row banks: lit on top, unlit below).
        FindFrameKind::Firefly | FindFrameKind::LightningBug => {
            s.sprite_dir = i.direction;
            s.counter += 1.0;
            if s.counter < 4.0 {
                s.row = 0;
            } else {
                s.row = 1;
                if s.counter >= 7.0 {
                    s.counter = 0.0;
                }
            }
            if i.local_ai2 <= 0.0 {
                s.row += 2;
            }
        }
        // `case 356` (`NPC.cs:72739-72772`, shared with 444/653/661):
        // the velocity-fed 7-tick flap (rows 0/1/2/1, reset at
        // `7*4-1 = 27`), the `vx * 0.3` tilt, and the type-356-only
        // colour bank: `frame.Y += (int)(num*3 * (ai[2]-1))` — the
        // 24-row sheet is eight 3-row banks. The C# cast truncates
        // toward zero; vanilla `ai[2]` is 1..8 so the offset is >= 0
        // (the clamp below only guards a non-vanilla negative bank).
        FindFrameKind::Butterfly => {
            let num143 = 7.0f32;
            s.rotation = i.vx * 0.3;
            s.sprite_dir = i.direction;
            s.counter += 1.0 + (i.vx.abs() + i.vy.abs()) / 2.0;
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
            let bank = (3.0f32 * (i.ai2 - 1.0)) as i32;
            s.row = (s.row as i32 + bank).max(0) as u32;
        }
        // `case 357` (`NPC.cs:72774-72828`, shared with 448/484/606):
        // the `localAI[0] = -2` pin, the grounded idle row 1, the
        // 13-tick two-row crawl (`frame.Y > num` wraps to 0 — rows
        // alternate 1/0), the airborne row 1 with the `direction*0.1`
        // spin accumulation, and the slope override (the resolved
        // probe rides `FrameInput::slope`): slope 1 → +0.785 rad,
        // slope 2 → −0.785, both lifting `localAI[0]` to 0.
        FindFrameKind::Worm => {
            s.local_ai0 = -2.0;
            if i.vy == 0.0 {
                s.rotation = 0.0;
                if i.vx == 0.0 {
                    s.row = 1;
                    s.counter = 0.0;
                } else {
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
                s.rotation += i.direction as f32 * 0.1;
                s.row = 1;
            }
            if i.slope == 1 {
                s.rotation = 0.785;
                s.local_ai0 = 0.0;
            } else if i.slope == 2 {
                s.rotation = -0.785;
                s.local_ai0 = 0.0;
            }
        }
        // `case 359 : case 360` (`NPC.cs:72238-72295`, shared with 655
        // the MagmaSnail): ALREADY PORTED by the aiStyle-67 module —
        // `snail_find_frame` is the verbatim twin (the `vy > 1` fall
        // rows 4/5 at the 6-tick phase, the 10-tick six-phase walk
        // 0/1/2/3/2/1, the zero-velocity freeze). Delegate, no second
        // copy (the house shared-helper rule).
        FindFrameKind::Snail | FindFrameKind::GlowingSnail => {
            let (row, counter) =
                crate::npc_ai_style67::snail_find_frame(i.vx, i.vy, s.row, s.counter);
            s.row = row;
            s.counter = counter;
        }
        // `case 361` (`NPC.cs:76618-76685`, shared with 445/687): the
        // frog's three gaits. Wet: the counter zeroes and the |vx|
        // bands pick rows 10/11/12 (the 0.25/0.15 deadbands). Grounded
        // idle: the >6 tick step over rows 0..5 (the `> num*5` wrap
        // runs EVERY tick, not only on the step). Grounded walk: the
        // 6-tick bands 0/6/8/9 with the reset at `6*4-1 = 23`. Air:
        // the fall row 9 / rise row 8.
        FindFrameKind::Frog => {
            s.sprite_dir = i.direction;
            if i.wet {
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
                    s.counter += 1.0;
                    if s.counter > 6.0 {
                        s.counter = 0.0;
                        s.row += 1;
                    }
                    if s.row > 5 {
                        s.row = 0;
                    }
                } else {
                    s.counter += 1.0;
                    let num196 = 6.0f32;
                    if s.counter < num196 {
                        s.row = 0;
                    } else if s.counter < num196 * 2.0 {
                        s.row = 6;
                    } else if s.counter < num196 * 3.0 {
                        s.row = 8;
                    } else {
                        s.row = 9;
                        if s.counter >= num196 * 4.0 - 1.0 {
                            s.counter = 0.0;
                        }
                    }
                }
            } else if i.vy > 0.0 {
                s.row = 9;
            } else {
                s.row = 8;
            }
        }
        // `case 362 : case 364` (`NPC.cs:73747-73771`, shared with 608
        // the seagull): the duck walk — the idle snaps row 0 and zeroes
        // the counter; the 4-tick stride climbs rows 3..9 (the wrap at
        // `num*10` returns to 3, the floor clamp lifts any lower row).
        FindFrameKind::Duck | FindFrameKind::DuckWhite => {
            s.sprite_dir = i.direction;
            if i.vx == 0.0 {
                s.counter = 0.0;
                s.row = 0;
            } else {
                s.counter += 1.0;
                if s.counter >= 4.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
                if s.row >= 10 {
                    s.row = 3;
                }
                if s.row < 3 {
                    s.row = 3;
                }
            }
        }
        // `case 363 : case 365` (`NPC.cs:73772-73818`, shared with 609
        // the grebe): the duck swimmer — wet paddles rows 1/2 at the
        // 8-tick stride (rotation zeroed); the dry stand pins row 1;
        // the dry flight tilts `vx * 0.1` and strides rows 11..14 at 4
        // ticks, wrapping at `npcFrameCount` (15) back to 11.
        FindFrameKind::Duck2 | FindFrameKind::DuckWhite2 => {
            s.sprite_dir = i.direction;
            if i.wet {
                s.counter += 1.0;
                if s.counter >= 8.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
                if s.row > 2 {
                    s.row = 1;
                }
                if s.row < 1 {
                    s.row = 1;
                }
                s.rotation = 0.0;
            } else if i.vx == 0.0 && i.vy == 0.0 {
                s.row = 1;
                s.counter = 0.0;
                s.rotation = 0.0;
            } else {
                s.rotation = i.vx * 0.1;
                s.counter += 1.0;
                if s.counter >= 4.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
                if s.row >= count {
                    s.row = 11;
                }
                if s.row < 11 {
                    s.row = 11;
                }
            }
        }
        // `case 366` (`NPC.cs:76820-76850`, shared with 367 — d49's
        // kind): the scuttle — the flip rides the GROUND only (the two
        // one-sided direction ifs); the idle snaps row 0; the walk
        // accumulator is `|vx| + 1` a tick stepping at > 6 over the
        // full 4 rows (`frame.Y / num >= npcFrameCount` wraps to 0).
        FindFrameKind::ScorpionBlack => {
            if i.vy == 0.0 {
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
            }
            if i.vx == 0.0 {
                s.row = 0;
                s.counter = 0.0;
            } else {
                s.counter += i.vx.abs();
                s.counter += 1.0;
                if s.counter > 6.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
                if s.row >= count {
                    s.row = 0;
                }
            }
        }
    }
}

/// The wiring seam: the vanilla `NPC.type` → wave-C kind map (the
/// `npc_findframe_a::find_frame_kind` shape, keyed on the vanilla id
/// because the `NpcKind` rows do not exist yet — the roster wave
/// `task/W4-d39-npc-kinds-351-420` owns them). Returns `None` for every
/// id outside this wave's 12.
pub fn find_frame_kind_c(vanilla_type: i16) -> Option<FindFrameKind> {
    Some(match vanilla_type {
        355 => FindFrameKind::Firefly,
        356 => FindFrameKind::Butterfly,
        357 => FindFrameKind::Worm,
        358 => FindFrameKind::LightningBug,
        359 => FindFrameKind::Snail,
        360 => FindFrameKind::GlowingSnail,
        361 => FindFrameKind::Frog,
        362 => FindFrameKind::Duck,
        363 => FindFrameKind::Duck2,
        364 => FindFrameKind::DuckWhite,
        365 => FindFrameKind::DuckWhite2,
        366 => FindFrameKind::ScorpionBlack,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn rows_over(kind: FindFrameKind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_c(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The 12 `Main.npcFrameCount` rows, counted entry-by-entry out of
    /// the `Main.cs:67082` initializer (the line/slot arithmetic in
    /// `frame_count`'s doc) and cross-checked against the port's
    /// verbatim `NPC_FRAME_COUNT` table.
    #[test]
    fn the_wave_c_frame_counts_match_the_main_initializer() {
        let table = [
            (FindFrameKind::Firefly, 355, 4),
            (FindFrameKind::Butterfly, 356, 24),
            (FindFrameKind::Worm, 357, 2),
            (FindFrameKind::LightningBug, 358, 4),
            (FindFrameKind::Snail, 359, 6),
            (FindFrameKind::GlowingSnail, 360, 6),
            (FindFrameKind::Frog, 361, 13),
            (FindFrameKind::Duck, 362, 15),
            (FindFrameKind::Duck2, 363, 15),
            (FindFrameKind::DuckWhite, 364, 15),
            (FindFrameKind::DuckWhite2, 365, 15),
            (FindFrameKind::ScorpionBlack, 366, 4),
        ];
        for (kind, vanilla, count) in table {
            assert_eq!(kind.vanilla_type(), vanilla, "{kind:?} id");
            assert_eq!(kind.frame_count(), count, "{kind:?} npcFrameCount");
            assert_eq!(
                crate::render::entities::npc_rows(vanilla as u16),
                count,
                "{kind:?} NPC_FRAME_COUNT cross-check"
            );
            assert_eq!(find_frame_kind_c(vanilla), Some(kind), "{kind:?} seam");
        }
        // The band edges stay outside this wave (d49 owns 367+).
        assert_eq!(find_frame_kind_c(354), None);
        assert_eq!(find_frame_kind_c(367), None);
        assert_eq!(find_frame_kind_c(550), None);
    }

    /// The firefly blink (`NPC.cs:72691-72708`): rows 0 for counters
    /// 1..3, row 1 for 4..6, the reset at 7; the glow-off `localAI[2]
    /// <= 0` shift adds 2 rows (the unlit bank).
    #[test]
    fn the_firefly_blinks_on_the_four_seven_clock_and_the_glow_bank() {
        let mut i = input(0.5, 0.0);
        i.local_ai2 = 1.0;
        assert_eq!(
            rows_over(FindFrameKind::Firefly, i, 8),
            [0, 0, 0, 1, 1, 1, 1, 0]
        );
        i.local_ai2 = 0.0;
        assert_eq!(
            rows_over(FindFrameKind::LightningBug, i, 7),
            [2, 2, 2, 3, 3, 3, 3]
        );
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        find_frame_c(FindFrameKind::Firefly, &mut s, &input(0.0, 0.0));
        assert_eq!(s.sprite_dir, 1, "spriteDirection = direction (:72691)");
    }

    /// The butterfly flap (`NPC.cs:72744-72767`): the counter feeds
    /// `1 + (|vx|+|vy|)/2` a tick — at vx 1 that is 1.5, so the 7-tick
    /// bands land on ticks 5/10/14 and the reset at 27 on tick 18.
    #[test]
    fn the_butterfly_flap_feeds_on_the_velocity_magnitude() {
        let rows = rows_over(FindFrameKind::Butterfly, input(1.0, 0.0), 19);
        assert_eq!(
            rows,
            [0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 1, 0]
        );
        let mut s = FrameState::default();
        find_frame_c(FindFrameKind::Butterfly, &mut s, &input(2.0, 0.0));
        assert_eq!(s.rotation, 0.6, "vx * 0.3 (:72745)");
        assert_eq!(s.sprite_dir, 1);
        // The still hover steps 1 a tick: six ticks row 0, tick 7 row 1.
        assert_eq!(
            rows_over(FindFrameKind::Butterfly, input(0.0, 0.0), 7),
            [0, 0, 0, 0, 0, 0, 1]
        );
    }

    /// The butterfly colour bank (`NPC.cs:72768-72771`): type 356 adds
    /// `(int)(num*3 * (ai[2]-1))` — bank 3 shifts +6 rows over the
    /// 24-row sheet's eight 3-row banks.
    #[test]
    fn the_butterfly_colour_bank_shifts_three_rows_per_index() {
        let mut i = input(0.0, 0.0);
        i.ai2 = 3.0;
        let mut s = FrameState::default();
        find_frame_c(FindFrameKind::Butterfly, &mut s, &i);
        assert_eq!(s.row, 6);
        i.ai2 = 8.0;
        find_frame_c(FindFrameKind::Butterfly, &mut s, &i);
        assert_eq!(s.row, 21);
    }

    /// The worm crawl (`NPC.cs:72779-72806`): the grounded walk steps
    /// at counter > 12 alternating rows 1/0 (the `frame.Y > num` wrap),
    /// the idle pins row 1, the air pins row 1 and spins
    /// `direction * 0.1` a tick; `localAI[0]` is pinned -2.
    #[test]
    fn the_worm_crawls_two_rows_on_the_thirteen_tick_clock() {
        let rows = rows_over(FindFrameKind::Worm, input(1.0, 0.0), 26);
        assert_eq!(rows[12], 1, "the first step at counter 13");
        assert_eq!(rows[25], 0, "the second step wraps 2 -> 0");
        let mut s = FrameState::default();
        find_frame_c(FindFrameKind::Worm, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter, s.local_ai0), (1, 0.0, -2.0));
        let mut s = FrameState::default();
        let air = input(0.0, 2.0);
        for _ in 0..3 {
            find_frame_c(FindFrameKind::Worm, &mut s, &air);
        }
        assert_eq!(s.row, 1);
        assert!((s.rotation - 0.3).abs() < 1e-6, "3 x 0.1 spin");
    }

    /// The worm slope override (`NPC.cs:72816-72825`): slope 1 tilts
    /// +0.785 and slope 2 -0.785, both lifting `localAI[0]` to 0; the
    /// resolved probe rides `FrameInput::slope`.
    #[test]
    fn the_worm_slope_probe_overrides_the_rotation_and_lifts_the_pin() {
        let mut i = input(1.0, 0.0);
        i.slope = 1;
        let mut s = FrameState::default();
        find_frame_c(FindFrameKind::Worm, &mut s, &i);
        assert_eq!((s.rotation, s.local_ai0), (0.785, 0.0));
        i.slope = 2;
        find_frame_c(FindFrameKind::Worm, &mut s, &i);
        assert_eq!((s.rotation, s.local_ai0), (-0.785, 0.0));
    }

    /// The snail delegate (`NPC.cs:72238-72295` via
    /// `npc_ai_style67::snail_find_frame`): the fall rows 4/5 at the
    /// 6-tick phase (reset at 11), the 10-tick six-phase walk, and the
    /// zero-velocity freeze — identical outputs to the style-67 twin.
    #[test]
    fn the_snail_delegates_to_the_style67_twin() {
        assert_eq!(
            rows_over(FindFrameKind::Snail, input(0.0, 2.0), 11),
            [4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5]
        );
        let walk = input(1.0, 0.0);
        let rows = rows_over(FindFrameKind::GlowingSnail, walk, 59);
        assert_eq!(rows[8], 0);
        assert_eq!(rows[9], 1);
        assert_eq!(rows[29], 3);
        assert_eq!(rows[39], 2);
        assert_eq!(rows[58], 1, "the reset tick holds row 1");
        let mut s = FrameState { row: 3, counter: 7.0, ..Default::default() };
        find_frame_c(FindFrameKind::Snail, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (3, 7.0), "the freeze (:72258)");
        // Byte-parity with the delegated machine on a mixed run.
        let (mut row, mut counter) = (0u32, 0.0f32);
        let mut s = FrameState::default();
        for t in 0..40 {
            let vx = if t % 7 == 0 { 0.0 } else { 0.6 };
            let vy = if t % 11 == 0 { 2.0 } else { 0.0 };
            (row, counter) =
                crate::npc_ai_style67::snail_find_frame(vx, vy, row, counter);
            find_frame_c(FindFrameKind::Snail, &mut s, &input(vx, vy));
            assert_eq!((s.row, s.counter), (row, counter), "tick {t}");
        }
    }

    /// The frog's wet bands (`NPC.cs:76622-76637`): |vx| > 0.25 row 10,
    /// > 0.15 row 11, else row 12; the counter zeroes each wet tick.
    #[test]
    fn the_frog_wet_bands_pick_the_swim_rows() {
        for (vx, row) in [(0.3, 10), (-0.3, 10), (0.2, 11), (-0.2, 11), (0.1, 12)] {
            let mut i = input(vx, 0.0);
            i.wet = true;
            let mut s = FrameState { counter: 5.0, ..Default::default() };
            find_frame_c(FindFrameKind::Frog, &mut s, &i);
            assert_eq!((s.row, s.counter), (row, 0.0), "vx {vx}");
        }
    }

    /// The frog's grounded gaits (`NPC.cs:76638-76676`): the idle
    /// steps every 7th tick over rows 0..5 (the wrap runs every tick);
    /// the walk rides the 6-tick bands 0/6/8/9 with the reset at 23.
    #[test]
    fn the_frog_idle_and_walk_ride_their_own_clocks() {
        assert_eq!(
            rows_over(FindFrameKind::Frog, input(0.0, 0.0), 14),
            [0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 2]
        );
        let rows = rows_over(FindFrameKind::Frog, input(1.0, 0.0), 24);
        assert_eq!(
            rows,
            [0, 0, 0, 0, 0, 6, 6, 6, 6, 6, 6, 8, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 9, 0]
        );
        // The idle wrap: row 6 (a walk leftover) snaps to 0 even on a
        // non-step tick (`:76648-76651` runs outside the counter gate).
        let mut s = FrameState { row: 6, ..Default::default() };
        find_frame_c(FindFrameKind::Frog, &mut s, &input(0.0, 0.0));
        assert_eq!(s.row, 0);
        // The air rows (`:76677-76684`).
        let mut s = FrameState::default();
        find_frame_c(FindFrameKind::Frog, &mut s, &input(0.0, 2.0));
        assert_eq!(s.row, 9);
        find_frame_c(FindFrameKind::Frog, &mut s, &input(0.0, -2.0));
        assert_eq!(s.row, 8);
    }

    /// The duck walk (`NPC.cs:73750-73770`): the idle snaps row 0; the
    /// 4-tick stride clamps into rows 3..9 and wraps 10 -> 3.
    #[test]
    fn the_duck_walk_strides_rows_three_to_nine() {
        let mut s = FrameState::default();
        find_frame_c(FindFrameKind::Duck, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        assert_eq!(
            rows_over(FindFrameKind::DuckWhite, input(1.0, 0.0), 8),
            [3, 3, 3, 4, 4, 4, 4, 5]
        );
        let mut s = FrameState { row: 9, counter: 3.0, ..Default::default() };
        find_frame_c(FindFrameKind::Duck, &mut s, &input(1.0, 0.0));
        assert_eq!((s.row, s.counter), (3, 0.0), "the 10 -> 3 wrap");
    }

    /// The duck swimmer (`NPC.cs:73775-73817`): wet paddles rows 1/2
    /// at 8 ticks (rotation zeroed), the dry stand pins row 1, the dry
    /// flight strides rows 11..14 at 4 ticks with the 15 -> 11 wrap.
    #[test]
    fn the_duck_swimmer_paddles_wet_and_flies_dry() {
        let mut wet = input(0.5, 0.0);
        wet.wet = true;
        let mut s = FrameState { row: 1, rotation: 0.4, ..Default::default() };
        let mut rows = Vec::new();
        for _ in 0..16 {
            find_frame_c(FindFrameKind::Duck2, &mut s, &wet);
            rows.push(s.row);
        }
        assert_eq!(rows, [1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 1]);
        assert_eq!(s.rotation, 0.0);
        let mut s =
            FrameState { row: 7, counter: 3.0, rotation: 0.4, ..Default::default() };
        find_frame_c(FindFrameKind::DuckWhite2, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter, s.rotation), (1, 0.0, 0.0));
        let mut s = FrameState::default();
        let mut rows = Vec::new();
        for _ in 0..16 {
            find_frame_c(FindFrameKind::Duck2, &mut s, &input(2.0, 0.0));
            rows.push(s.row);
        }
        assert_eq!(
            rows,
            [11, 11, 11, 12, 12, 12, 12, 13, 13, 13, 13, 14, 14, 14, 14, 11]
        );
        assert_eq!(s.rotation, 0.2, "vx * 0.1 (:73802)");
    }

    /// The scorpion scuttle (`NPC.cs:76822-76849`): the `|vx| + 1`
    /// accumulator steps at > 6 over the full 4 rows; the flip rides
    /// the ground only; the idle snaps row 0.
    #[test]
    fn the_scorpion_scuttles_on_the_speed_fed_accumulator() {
        assert_eq!(
            rows_over(FindFrameKind::ScorpionBlack, input(1.0, 0.0), 16),
            [0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 0]
        );
        let mut i = input(1.0, 0.0);
        i.direction = -1;
        let mut s = FrameState::default();
        find_frame_c(FindFrameKind::ScorpionBlack, &mut s, &i);
        assert_eq!(s.sprite_dir, -1);
        // Airborne: the flip is NOT written (`:76822`'s vy gate).
        let mut s = FrameState { sprite_dir: 1, ..Default::default() };
        i.vy = 2.0;
        find_frame_c(FindFrameKind::ScorpionBlack, &mut s, &i);
        assert_eq!(s.sprite_dir, 1);
        // The idle snap.
        let mut s = FrameState { row: 3, counter: 5.0, ..Default::default() };
        find_frame_c(FindFrameKind::ScorpionBlack, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
    }
}
