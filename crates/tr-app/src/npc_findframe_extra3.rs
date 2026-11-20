//! W4-f26 — FindFrame extra wave 3: the NEXT unclaimed missing case
//! ids of `NPC.FindFrame` (`NPC.cs:66684-77370`), counted off the
//! ascending census after the d44/d46-d49/e13/e14 claims (the pushed
//! branches read: d44 owns 37/43/46/47/48/56/57/61/62/63/64/67, d46
//! 150/151/152/156/157/158/159/160/162/163/164/165, d47
//! 166/168/169/170/171/172/173/175/176/178/179/180, d48 355-366, d49
//! 367/374/375/377/378/379/380/430-434, e13 (per e14's header)
//! 37/43/46/47/48/56/61/62/63/64/67/69/73/74/83, e14
//! 84/93/94/101/102/103/116/120/121/137/141/142/148/149/181; the
//! wave-A/wave-B rosters on main cover the rest below 182). Batch 1
//! owns the FIRST 6 — **182/185/195/196/198/199** — the Floaty Gross,
//! the Snow Flinx, the Lost Girl, the Nymph, the Lihzahrd and the
//! Lihzahrd Crawler (`ID/NPCID.cs:11041-11075`); batch 2 adds
//! 201/202/203/204/205/209. Landed 6 kinds per commit.
//!
//! Shape: the kinds are NOT in `NpcKind` yet (their AI/spawn machines
//! are later rounds), so — like `npc_findframe_a.rs` /
//! `npc_findframe_b.rs` (S1-D-3 waves A/B) — the frame machines land
//! first as pure functions over a small [`FrameState`]/[`FrameInput`]
//! pair. The wiring seam is [`find_frame_kind_extra3`]: when a kind
//! joins `NpcKind`, its `find_frame` arm copies `row`/`counter`/
//! `sprite_dir`/`rotation` out, runs one tick and copies them back
//! (the `Npc::find_frame_wave_a` bridge precedent). No arm draws
//! `Main.rand` (the floater group's dust spawns are types 60/634's,
//! `NPC.cs:74839-74849` — neither is 182), so the wiring cannot move
//! a stream.
//!
//! Row convention: the port tracks frame ROW indices, not pixels —
//! the C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer),
//! counted entry-by-entry in [`FindFrameExtra3Kind::frame_count`] and
//! cross-checked against the port's own `render/entities.rs::
//! NPC_FRAME_COUNT` table.
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "W4-f26 FindFrame
//! extra wave 3"):
//! * the floater group's type-60 dust (`NPC.cs:74839-74843`) and the
//!   type-634 `Main.rand.Next(10)` dust (`:74844-74849`) — neither is
//!   182; the 210/211 double-increment + `vx * 0.2` tilt
//!   (`:74859-74863`) and the 49/51/60/634 `num292--` (`:74867-74870`)
//!   are other kinds' too (182 flaps the FULL 5 rows every 6 ticks);
//! * the humanoid group's town `ai[0]` sit/talk/attack pose zones
//!   (`:68662-69488`, including their chat-bubble `Main.rand` draws)
//!   — the port's town machines do not model them (the
//!   `npc.rs::humanoid_town_frame` / W4-e14 Santa precedent); the
//!   walk/idle/air tail (`:69489-69655`) IS carried. The aiStyle-3
//!   fighters (185/196/198) never leave `ai[0]` 0/1 (the turn
//!   counter, `NPC.cs:57381-57397` — the wave-B note).

// The wiring is a later round (the kinds are not in `NpcKind` yet) —
// the `npc_findframe_a.rs`/`npc_findframe_b.rs` precedent.
#![allow(dead_code)]

/// The extra-wave-3 kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra3Kind {
    /// `NPCID.FloatyGross` = 182 (`NPCID.cs:11041`) — `case 182`
    /// (the 48/49/51/60/82/… floater group).
    FloatyGross,
    /// `NPCID.SnowFlinx` = 185 (`NPCID.cs:11047`) — the humanoid
    /// fighter group (`case 185`, `NPC.cs:68575`).
    SnowFlinx,
    /// `NPCID.LostGirl` = 195 (`NPCID.cs:11067`) — `case 195`.
    LostGirl,
    /// `NPCID.Nymph` = 196 (`NPCID.cs:11069`) — the humanoid fighter
    /// group (`case 196`, `NPC.cs:68576`).
    Nymph,
    /// `NPCID.Lihzahrd` = 198 (`NPCID.cs:11073`) — the humanoid
    /// fighter group (`case 198`, `NPC.cs:68578`).
    Lihzahrd,
    /// `NPCID.LihzahrdCrawler` = 199 (`NPCID.cs:11075`) — `case 199`.
    LihzahrdCrawler,
}

impl FindFrameExtra3Kind {
    /// The vanilla `NPC.type` — the `FindFrame` switch key.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::FloatyGross => 182,
            Self::SnowFlinx => 185,
            Self::LostGirl => 195,
            Self::Nymph => 196,
            Self::Lihzahrd => 198,
            Self::LihzahrdCrawler => 199,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// entry `type` on line `67084 + type/10`, slot `type % 10`) —
    /// cross-checked against `render/entities.rs::NPC_FRAME_COUNT`.
    pub fn frame_count(self) -> u32 {
        match self {
            Self::FloatyGross => 5,
            Self::SnowFlinx => 12,
            Self::LostGirl => 7,
            Self::Nymph => 14,
            Self::Lihzahrd => 16,
            Self::LihzahrdCrawler => 8,
        }
    }
}

/// The wiring seam: the vanilla type → this module's kind (the
/// `find_frame_kind_high2` precedent). `None` outside the slice.
pub fn find_frame_kind_extra3(vanilla_type: i16) -> Option<FindFrameExtra3Kind> {
    Some(match vanilla_type {
        182 => FindFrameExtra3Kind::FloatyGross,
        185 => FindFrameExtra3Kind::SnowFlinx,
        195 => FindFrameExtra3Kind::LostGirl,
        196 => FindFrameExtra3Kind::Nymph,
        198 => FindFrameExtra3Kind::Lihzahrd,
        199 => FindFrameExtra3Kind::LihzahrdCrawler,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection`,
/// `rotation`. The wiring step copies them to/from `Npc`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter`.
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — only the floater arm writes it (the vx tilt).
    pub rotation: f32,
}

/// The per-tick inputs the arms read: `velocity`, `direction` and
/// the `ai` slots. `ai_action` is not carried yet — none of batch
/// 1's arms reads the `num2` motion class (`NPC.cs:66696-66704`);
/// batch 2's slime arm adds it.
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone in
    /// the arms that gate on `direction == 1` / `== -1`).
    pub direction: i32,
    pub ai: [f32; 4],
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1, ai: [0.0; 4] }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-3 kinds — one
/// tick.
pub fn find_frame_extra3(kind: FindFrameExtra3Kind, s: &mut FrameState, i: &FrameInput) {
    let count = kind.frame_count();
    match kind {
        // `case 182` in the floater group (`NPC.cs:74823-74885`,
        // shared with 48/49/51/60/82/93/137/210/211/253/316/634/662/
        // 664): faces by velocity sign (0 leaves the flip alone),
        // tilts `vx * 0.1`, and the 6-tick counter walks the FULL 5
        // rows — 182 is not in the `num292--` list (49/51/60/634,
        // `:74867-74870`), not the type-48 `num291 = 5` override
        // (`:74871-74874`), and not the 210/211 double-increment +
        // 0.2 tilt (`:74859-74863`). The group's dust spawns are
        // types 60/634's (the header's skip).
        FindFrameExtra3Kind::FloatyGross => {
            // `:74850-74857`.
            if i.vx > 0.0 {
                s.sprite_dir = 1;
            }
            if i.vx < 0.0 {
                s.sprite_dir = -1;
            }
            // `:74858`.
            s.rotation = i.vx * 0.1;
            // `:74864-74883` — num291 = 6, num292 = the full count.
            s.counter += 1.0;
            if s.counter >= 6.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= count {
                s.row = 0;
            }
        }
        // The humanoid-group case list (`NPC.cs:68575-68578` carries
        // 185/196/198) → the shared fighter walk
        // (`NPC.cs:68647-69655`) via the wave-B keep-one helper (the
        // e14 ChaosElemental/FaceMonster delegate precedent). None is
        // `isLikeATownNPC` (`NPC.cs:6856-6866` — only 453 + the town
        // flag; the three SetDefaults arms `:11365`/`:11510`/`:11537`
        // set aiStyle 3, no townNPC), so `num57` = 0 and the walk
        // wraps at the FULL `npcFrameCount`; none is a town slime, so
        // `num108` = 6 and `num109` = row 2. None of the three is in
        // the airborne pin list (`:69647`) — the air holds the
        // group's row 1. None is the 140/489 idle-row-1 pair.
        FindFrameExtra3Kind::SnowFlinx
        | FindFrameExtra3Kind::Nymph
        | FindFrameExtra3Kind::Lihzahrd => {
            let bi = crate::npc_findframe_b::FrameInput {
                vx: i.vx,
                vy: i.vy,
                direction: i.direction,
                wet: false,
                ai: i.ai,
            };
            let mut bs = crate::npc_findframe_b::FrameState {
                row: s.row,
                counter: s.counter,
                sprite_dir: s.sprite_dir,
                rotation: s.rotation,
            };
            crate::npc_findframe_b::humanoid_fighter_frame(&mut bs, &bi, count, false, false);
            s.row = bs.row;
            s.counter = bs.counter;
            s.sprite_dir = bs.sprite_dir;
            s.rotation = bs.rotation;
        }
        // `case 195` (`NPC.cs:73146-73171`): the Lost Girl's transform
        // ladder — `ai[0]` selects the row in six 3-tick bands
        // (<3/<6/<9/<12/<15 → rows 0..4, else row 5). No counter,
        // sprite or rotation write; the 7th sheet row is never shown
        // by this arm.
        FindFrameExtra3Kind::LostGirl => {
            s.row = if i.ai[0] < 3.0 {
                0
            } else if i.ai[0] < 6.0 {
                1
            } else if i.ai[0] < 9.0 {
                2
            } else if i.ai[0] < 12.0 {
                3
            } else if i.ai[0] < 15.0 {
                4
            } else {
                5
            };
        }
        // `case 199` (`NPC.cs:75030-75068`): the Lihzahrd Crawler.
        // The grounded flip writes only on direction ±1 (`:75031-
        // 75041`). While the row is below 3 the plain counter steps
        // every > 6 ticks (`:75042-75050` — the crawl-in band); at
        // row >= 3 the AIRBORNE tick breaks with NO write (the row
        // holds, `:75053-75056`), and the grounded tick accumulates
        // `|vx|` stepping every > 12 over rows 3..7, wrapping > 7
        // back to 3 (`:75057-75066`).
        FindFrameExtra3Kind::LihzahrdCrawler => {
            // `:75031-75041`.
            if i.vy == 0.0 {
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
            }
            if s.row < 3 {
                // `:75042-75050`.
                s.counter += 1.0;
                if s.counter > 6.0 {
                    s.counter = 0.0;
                    s.row += 1;
                }
            } else {
                // `:75053-75056` — the air hold.
                if i.vy != 0.0 {
                    return;
                }
                // `:75057-75066`.
                s.counter += i.vx.abs();
                if s.counter > 12.0 {
                    s.counter = 0.0;
                    s.row += 1;
                    if s.row > 7 {
                        s.row = 3;
                    }
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

    fn rows_over(kind: FindFrameExtra3Kind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra3(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The frame-count table agrees with the port's verbatim
    /// `NPC_FRAME_COUNT` (`Main.cs:67082` initializer cross-check).
    #[test]
    fn f26_frame_counts_match_the_port_table() {
        let table = crate::render::entities::NPC_FRAME_COUNT;
        for k in [
            FindFrameExtra3Kind::FloatyGross,
            FindFrameExtra3Kind::SnowFlinx,
            FindFrameExtra3Kind::LostGirl,
            FindFrameExtra3Kind::Nymph,
            FindFrameExtra3Kind::Lihzahrd,
            FindFrameExtra3Kind::LihzahrdCrawler,
        ] {
            assert_eq!(
                k.frame_count(),
                table[k.vanilla_type() as usize],
                "type {}",
                k.vanilla_type()
            );
        }
    }

    /// 182 FloatyGross: the 6-tick stride over the FULL 5 rows, the
    /// vx-sign flip (0 holds) and the `vx * 0.1` tilt
    /// (`NPC.cs:74850-74883`).
    #[test]
    fn f26_floaty_gross_flaps_the_full_five_rows() {
        let rows = rows_over(FindFrameExtra3Kind::FloatyGross, input(2.0, -1.0), 31);
        // ticks 1-5 hold row 0, tick 6 steps to 1, … tick 30 wraps to 0.
        assert_eq!(&rows[..6], &[0, 0, 0, 0, 0, 1]);
        assert_eq!(rows[29], 0);
        assert_eq!(rows[30], 0);
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        find_frame_extra3(FindFrameExtra3Kind::FloatyGross, &mut s, &input(0.0, 0.0));
        assert_eq!(s.sprite_dir, -1, "vx == 0 leaves the flip alone");
        assert_eq!(s.rotation, 0.0);
        find_frame_extra3(FindFrameExtra3Kind::FloatyGross, &mut s, &input(-3.0, 0.0));
        assert_eq!(s.sprite_dir, -1);
        assert_eq!(s.rotation, -0.3);
    }

    /// 185 SnowFlinx: the fighter walk — idle row 0, the |vx|*2+1
    /// accumulator over rows 2..11 (the FULL 12), the air row 1 (185
    /// is NOT in the `:69647` pin list).
    #[test]
    fn f26_snow_flinx_walks_the_full_twelve() {
        let idle = rows_over(FindFrameExtra3Kind::SnowFlinx, input(0.0, 0.0), 3);
        assert_eq!(idle, vec![0, 0, 0]);
        let walk = rows_over(FindFrameExtra3Kind::SnowFlinx, input(1.0, 0.0), 40);
        assert!(walk.iter().all(|&r| (2..12).contains(&r)));
        assert!(walk.windows(2).any(|w| w[0] == 11 && w[1] == 2), "wraps at 12");
        let air = rows_over(FindFrameExtra3Kind::SnowFlinx, input(1.0, 2.0), 3);
        assert_eq!(air, vec![1, 1, 1]);
    }

    /// 195 LostGirl: the ai[0] transform ladder — six bands, rows
    /// 0..5, no counter/sprite writes (`NPC.cs:73146-73171`).
    #[test]
    fn f26_lost_girl_ladder_reads_ai0() {
        for (ai0, row) in
            [(0.0, 0), (2.9, 0), (3.0, 1), (6.0, 2), (9.0, 3), (12.0, 4), (15.0, 5), (99.0, 5)]
        {
            let mut s = FrameState { row: 6, counter: 4.0, sprite_dir: -1, rotation: 0.5 };
            let i = FrameInput { ai: [ai0, 0.0, 0.0, 0.0], ..Default::default() };
            find_frame_extra3(FindFrameExtra3Kind::LostGirl, &mut s, &i);
            assert_eq!(s.row, row, "ai[0] = {ai0}");
            assert_eq!(s.counter, 4.0, "no counter write");
            assert_eq!(s.sprite_dir, -1, "no sprite write");
            assert_eq!(s.rotation, 0.5, "no rotation write");
        }
    }

    /// 196 Nymph / 198 Lihzahrd: the same fighter walk over their own
    /// full sheets (14/16 rows); neither is in the `:69647` air pin
    /// list.
    #[test]
    fn f26_nymph_and_lihzahrd_walk_their_full_sheets() {
        for (k, top) in [(FindFrameExtra3Kind::Nymph, 14), (FindFrameExtra3Kind::Lihzahrd, 16)] {
            let walk = rows_over(k, input(1.0, 0.0), 80);
            assert!(walk.iter().all(|&r| (2..top).contains(&r)), "type {}", k.vanilla_type());
            assert!(
                walk.windows(2).any(|w| w[0] == top - 1 && w[1] == 2),
                "type {} wraps at {top}",
                k.vanilla_type()
            );
            let air = rows_over(k, input(1.0, -2.0), 3);
            assert_eq!(air, vec![1, 1, 1]);
        }
    }

    /// 199 LihzahrdCrawler: the crawl-in band (rows 0..3 at > 6
    /// ticks), then the |vx| walk over rows 3..7 at > 12; the air
    /// tick at row >= 3 HOLDS everything (`NPC.cs:75030-75068`).
    #[test]
    fn f26_lihzahrd_crawler_two_bands_and_the_air_hold() {
        // Grounded idle: the plain counter climbs 0..3 (7 ticks a row).
        let rows = rows_over(FindFrameExtra3Kind::LihzahrdCrawler, input(0.0, 0.0), 22);
        assert_eq!(&rows[..7], &[0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(rows[21], 3);
        // At row 3 the |vx| band takes over: vx 3 → 5 ticks a step.
        let mut s = FrameState { row: 3, ..Default::default() };
        let mut seen = Vec::new();
        for _ in 0..25 {
            find_frame_extra3(FindFrameExtra3Kind::LihzahrdCrawler, &mut s, &input(3.0, 0.0));
            seen.push(s.row);
        }
        assert!(seen.iter().all(|&r| (3..=7).contains(&r)));
        assert!(seen.windows(2).any(|w| w[0] == 7 && w[1] == 3), "wraps > 7 to 3");
        // The air hold at row >= 3: no row/counter write.
        let mut s = FrameState { row: 5, counter: 7.0, ..Default::default() };
        find_frame_extra3(FindFrameExtra3Kind::LihzahrdCrawler, &mut s, &input(3.0, -2.0));
        assert_eq!((s.row, s.counter), (5, 7.0));
        // Below row 3 the air still ticks the crawl-in counter.
        let mut s = FrameState { row: 1, counter: 6.0, ..Default::default() };
        find_frame_extra3(FindFrameExtra3Kind::LihzahrdCrawler, &mut s, &input(0.0, -2.0));
        assert_eq!((s.row, s.counter), (2, 0.0));
    }

    /// The seam maps exactly the batch's vanilla types and nothing
    /// else.
    #[test]
    fn f26_seam_covers_exactly_the_slice() {
        for t in 0..697i16 {
            let k = find_frame_kind_extra3(t);
            let claimed = [182, 185, 195, 196, 198, 199];
            assert_eq!(k.is_some(), claimed.contains(&t), "type {t}");
            if let Some(k) = k {
                assert_eq!(k.vanilla_type(), t);
            }
        }
    }
}
