//! W4-f27 — FindFrame extra wave 4: the NEXT unclaimed missing case
//! ids of `NPC.FindFrame` (`NPC.cs:66684-77370`), counted off the
//! ascending census after the d44-d49/e13/e14/f26 claims (the pushed
//! branches read: d44 owns 37/43/46/47/48/56/57/61/62/63/64/67, d45
//! 69/73/74/83/84/93/94/101/102/103/116/120, d46
//! 150/151/152/156/157/158/159/160/162/163/164/165, d47
//! 166/168/169/170/171/172/173/175/176/178/179/180, d48 355-366, d49
//! 367/374/375/377/378/379/380/430-436/591, e13
//! 121/137/141/142/148/149/181/182/185/195/196/198/199/201/202, e14's
//! declared next-15 203/204/205/209/217/218/219/220/221/224/226/229/
//! 230/231/232, f26 182/185/195/196/198/199 + its declared batch 2
//! 201/202/203/204/205/209 — the overlaps dedupe). The census
//! re-derived mechanically: 602 C# top-level case ids − 248
//! port-covered − 75 d44-d49 claims = 279 remaining (the e13 base);
//! minus the 30 e13/e14-declared ids = 249 unclaimed. This wave owns
//! the FIRST 15 of those — **233/234/235/236/237/238/239/240/241/242/
//! 243/244/251/252/253** — the three leafy hornets, the Jungle
//! Creeper, the three wall crawlers, the Blood Crawler, the Blood
//! Feeder, the Blood Jelly, the Ice Golem, the Rainbow Slime, the
//! Eyezor, the Parrot and the Reaper (`ID/NPCID.cs:11143-11183`).
//! Landed 5 kinds per commit.
//!
//! Shape: the kinds are NOT in `NpcKind` yet (their AI/spawn machines
//! are later rounds), so — like `npc_findframe_a.rs` /
//! `npc_findframe_b.rs` (S1-D-3 waves A/B) and the f26 extra-3 wave —
//! the frame machines land first as pure functions over a small
//! [`FrameState`]/[`FrameInput`] pair. The wiring seam is
//! [`find_frame_kind_extra4`]: when a kind joins `NpcKind`, its
//! `find_frame` arm copies `row`/`counter`/`sprite_dir`/`rotation`
//! out, runs one tick and copies them back (the
//! `Npc::find_frame_wave_a` bridge precedent). No arm draws
//! `Main.rand` — the floater group's only draw (`NPC.cs:74844`) is
//! gated `type == 634`, not 253 — so the wiring cannot move a stream.
//!
//! Row convention: the port tracks frame ROW indices, not pixels —
//! the C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer),
//! counted entry-by-entry in [`FindFrameExtra4Kind::frame_count`] and
//! cross-checked against the port's own `render/entities.rs::
//! NPC_FRAME_COUNT` table.
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "W4-f27 FindFrame
//! extra wave 4"):
//! * the floater group's type-60 dust (`NPC.cs:74839-74843`) and the
//!   type-634 `Main.rand.Next(10)` dust (`:74844-74849`) — neither is
//!   253; the 210/211 double-increment + `vx * 0.2` tilt
//!   (`:74859-74863`) and the 49/51/60/634 `num292--` (`:74867-74870`)
//!   are other kinds' too (253 flaps the FULL 3 rows every 6 ticks);
//! * the wall-crawler group's type-531 `num194 = 0.4` override
//!   (`NPC.cs:73119-73122`) — 531 is a later wave's; this wave's
//!   237/238/240 all ride the 0.5 stride;
//! * the parrot arm's type-301 branch (`NPC.cs:73560-73577` — the
//!   inverted ground flip + the air velocity-sign flip) is 301's,
//!   not 252's; 252 rides the plain `|vx| > 0.5` flip.

// The wiring is a later round (the kinds are not in `NpcKind` yet) —
// the `npc_findframe_a.rs`/`npc_findframe_b.rs` precedent.
#![allow(dead_code)]

/// The extra-wave-4 kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra4Kind {
    /// `NPCID.HornetLeafy` = 233 (`NPCID.cs:11143`) — the 42/231/232
    /// hornet flap group (`NPC.cs:74907-74912`).
    HornetLeafy,
    /// `NPCID.HornetSpikey` = 234 (`NPCID.cs:11145`) — the same group.
    HornetSpikey,
    /// `NPCID.HornetStingy` = 235 (`NPCID.cs:11147`) — the same group.
    HornetStingy,
    /// `NPCID.JungleCreeper` = 236 (`NPCID.cs:11149`) — the
    /// 164/530 creeper group (`NPC.cs:73083-73086`).
    JungleCreeper,
    /// `NPCID.BloodCrawler` = 239 (`NPCID.cs:11155`) — the same
    /// creeper group (`NPC.cs:73085`).
    BloodCrawler,
    /// `NPCID.JungleCreeperWall` = 237 (`NPCID.cs:11151`) — the
    /// 165/531 wall-crawler group (`NPC.cs:73112-73116`).
    JungleCreeperWall,
    /// `NPCID.BlackRecluseWall` = 238 (`NPCID.cs:11153`) — the same
    /// wall-crawler group (`NPC.cs:73114`; d46's 165 BlackRecluse
    /// is the walking twin, NOT this arm).
    BlackRecluseWall,
    /// `NPCID.BloodCrawlerWall` = 240 (`NPCID.cs:11157`) — the same
    /// wall-crawler group (`NPC.cs:73115`).
    BloodCrawlerWall,
    /// `NPCID.BloodFeeder` = 241 (`NPCID.cs:11159`) — the
    /// 55/57/58/102/465/592/607 fish group (`NPC.cs:74181-74188`).
    BloodFeeder,
    /// `NPCID.BloodJelly` = 242 (`NPCID.cs:11161`) — the
    /// 63/64/103/256 jellyfish group (`NPC.cs:73916-73920`).
    BloodJelly,
}

impl FindFrameExtra4Kind {
    /// The vanilla `NPC.type` — the `FindFrame` switch key.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::HornetLeafy => 233,
            Self::HornetSpikey => 234,
            Self::HornetStingy => 235,
            Self::JungleCreeper => 236,
            Self::BloodCrawler => 239,
            Self::JungleCreeperWall => 237,
            Self::BlackRecluseWall => 238,
            Self::BloodCrawlerWall => 240,
            Self::BloodFeeder => 241,
            Self::BloodJelly => 242,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// entry `type` on line `67084 + type/10`, slot `type % 10`) —
    /// cross-checked against `render/entities.rs::NPC_FRAME_COUNT`.
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67107 (entries 233-235): 3, 3, 3; (236/239): 5, 5.
            Self::HornetLeafy => 3,
            Self::HornetSpikey => 3,
            Self::HornetStingy => 3,
            Self::JungleCreeper => 5,
            Self::BloodCrawler => 5,
            // Main.cs:67107 (237/238): 4, 4; :67108 (240/241/242): 4, 6, 7.
            Self::JungleCreeperWall => 4,
            Self::BlackRecluseWall => 4,
            Self::BloodCrawlerWall => 4,
            Self::BloodFeeder => 6,
            Self::BloodJelly => 7,
        }
    }
}

/// The wiring seam: the vanilla type → this module's kind (the
/// `find_frame_kind_extra3` precedent). `None` outside the slice.
pub fn find_frame_kind_extra4(vanilla_type: i16) -> Option<FindFrameExtra4Kind> {
    Some(match vanilla_type {
        233 => FindFrameExtra4Kind::HornetLeafy,
        234 => FindFrameExtra4Kind::HornetSpikey,
        235 => FindFrameExtra4Kind::HornetStingy,
        236 => FindFrameExtra4Kind::JungleCreeper,
        239 => FindFrameExtra4Kind::BloodCrawler,
        237 => FindFrameExtra4Kind::JungleCreeperWall,
        238 => FindFrameExtra4Kind::BlackRecluseWall,
        240 => FindFrameExtra4Kind::BloodCrawlerWall,
        241 => FindFrameExtra4Kind::BloodFeeder,
        242 => FindFrameExtra4Kind::BloodJelly,
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
    /// `NPC.rotation` — only the parrot/floater arms write it (the
    /// vx tilt); batch 1's arms never touch it.
    pub rotation: f32,
}

/// The per-tick inputs the arms read: `velocity`, `direction`, the
/// `wet` flag, the `ai` slots and `aiAction`. Batch 1's arms read
/// only `vx`/`vy`/`direction`; the later batches add `wet` (241),
/// `ai[1]` (242) and the `num2` motion class (244).
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone in
    /// the arms that gate on `direction == 1` / `== -1`).
    pub direction: i32,
    /// `Entity.wet` — the fish group's swim/flop split.
    pub wet: bool,
    pub ai: [f32; 4],
    /// `NPC.aiAction` — 0 the motion class, 1 the attack (`num2 = 4`,
    /// `NPC.cs:66696-66704`); only the Rainbow Slime reads it.
    pub ai_action: i32,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1, wet: false, ai: [0.0; 4], ai_action: 0 }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-4 kinds — one
/// tick.
pub fn find_frame_extra4(kind: FindFrameExtra4Kind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // `case 233/234/235` in the hornet group (`NPC.cs:74907-74934`,
        // shared with 42/231/232): the 8-tick band cycle — counter++
        // then <2 row 0, <4 row 1, <6 row 2, <8 row 1, else the
        // counter resets (the row HOLDS at 1 on the reset tick —
        // `:74930-74933` writes no frame). No sprite/rotation write.
        FindFrameExtra4Kind::HornetLeafy
        | FindFrameExtra4Kind::HornetSpikey
        | FindFrameExtra4Kind::HornetStingy => {
            // `:74913-74933`.
            s.counter += 1.0;
            if s.counter < 2.0 {
                s.row = 0;
            } else if s.counter < 4.0 {
                s.row = 1;
            } else if s.counter < 6.0 {
                s.row = 2;
            } else if s.counter < 8.0 {
                s.row = 1;
            } else {
                s.counter = 0.0;
            }
        }
        // `case 236/239` in the creeper group (`NPC.cs:73083-73111`,
        // shared with 164/530 — 164 is d46's WallCreeper, the same
        // machine, keep-one at the merge): the AIR tick zeroes the
        // counter and pins row 4 rising / row 0 falling (`:73087-
        // 73099`, the 5-row sheet's top row is air-only); the ground
        // tick copies `direction` UNCONDITIONALLY (0 included) and
        // accumulates `|vx| * 1.1`, the WHILE loop stepping a row
        // every 6 with the remainder kept (`:73101-73110` — a fast
        // tick can step twice), wrapping past row 3 to 0.
        FindFrameExtra4Kind::JungleCreeper | FindFrameExtra4Kind::BloodCrawler => {
            // `:73087-73099`.
            if i.vy != 0.0 {
                s.counter = 0.0;
                s.row = if i.vy < 0.0 { 4 } else { 0 };
                return;
            }
            // `:73100`.
            s.sprite_dir = i.direction;
            // `:73101-73110`.
            s.counter += i.vx.abs() * 1.1;
            while s.counter >= 6.0 {
                s.counter -= 6.0;
                s.row += 1;
                if s.row > 3 {
                    s.row = 0;
                }
            }
        }
        // `case 237/238/240` in the wall-crawler group (`NPC.cs:73112-
        // 73145`, shared with 165/531 — d46's 165 WallCreeperWall is
        // the same machine, keep-one at the merge): the
        // `(|vx| + |vy|) * 0.5` accumulator (the 531-only 0.4 override
        // `:73119-73122` is a later wave's) rides the <6/<12/<18/<24
        // bands over rows 0/1/2/3; the >= 24 tail zeroes the counter
        // and HOLDS the row (`:73140-73143` writes no frame). No
        // sprite/rotation write, no air split — the walls crawl in
        // every direction.
        FindFrameExtra4Kind::JungleCreeperWall
        | FindFrameExtra4Kind::BlackRecluseWall
        | FindFrameExtra4Kind::BloodCrawlerWall => {
            // `:73118-73123` — num194 = 0.5 for all three.
            s.counter += (i.vx.abs() + i.vy.abs()) * 0.5;
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
        // `case 241` in the fish group (`NPC.cs:74181-74226`, shared
        // with 55/57/58/102/465/592/607 — d44's goldfish arm and
        // e14/d45's 102 AnglerFish are the same machine, keep-one at
        // the merge): `spriteDirection = direction` UNCONDITIONALLY
        // (0 included, `:74189`), then the wet/dry split — wet rides
        // <6/<12/<18/<24 over rows 0/1/2/3 (`:74193-74212`), dry
        // <6/<12 over rows 4/5 (`:74214-74221`); BOTH tails zero the
        // counter and HOLD the row (the row-hold quirk, `:74211`/
        // `:74224`).
        FindFrameExtra4Kind::BloodFeeder => {
            // `:74189-74190`.
            s.sprite_dir = i.direction;
            s.counter += 1.0;
            if i.wet {
                // `:74193-74212`.
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
            } else if s.counter < 6.0 {
                // `:74214-74225`.
                s.row = 4;
            } else if s.counter < 12.0 {
                s.row = 5;
            } else {
                s.counter = 0.0;
            }
        }
        // `case 242` in the jellyfish group (`NPC.cs:73916-73960`,
        // shared with 63/64/103/256 — d44's 63/64 and e14/d45's 103
        // are the same machine, keep-one at the merge): the pulse
        // gate (`:73921` — 242 IS in the 63/64/103/242 list, 256 is
        // not) fires on `ai[1] == 1` — the >4 counter cycles rows
        // 4..6 with BOTH clamps (the first pulse tick lifts any row
        // into the band) and RETURNS early (`:73923-73937`); else
        // the idle <6/<12/<18 bands over rows 0/1/2 with the row-3
        // tail resetting at >= 23 (`:73939-73959`). No sprite/
        // rotation write.
        FindFrameExtra4Kind::BloodJelly => {
            // `:73921-73938`.
            if i.ai[1] == 1.0 {
                s.counter += 1.0;
                if s.counter > 4.0 {
                    s.counter = 0.0;
                    s.row += 1;
                }
                if s.row > 6 {
                    s.row = 4;
                }
                if s.row < 4 {
                    s.row = 4;
                }
                return;
            }
            // `:73939-73959`.
            s.counter += 1.0;
            if s.counter < 6.0 {
                s.row = 0;
                return;
            }
            if s.counter < 12.0 {
                s.row = 1;
                return;
            }
            if s.counter < 18.0 {
                s.row = 2;
                return;
            }
            s.row = 3;
            if s.counter >= 23.0 {
                s.counter = 0.0;
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

    fn rows_over(kind: FindFrameExtra4Kind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra4(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The frame-count table agrees with the port's verbatim
    /// `NPC_FRAME_COUNT` (`Main.cs:67082` initializer cross-check).
    #[test]
    fn f27_frame_counts_match_the_port_table() {
        let table = crate::render::entities::NPC_FRAME_COUNT;
        for k in [
            FindFrameExtra4Kind::HornetLeafy,
            FindFrameExtra4Kind::HornetSpikey,
            FindFrameExtra4Kind::HornetStingy,
            FindFrameExtra4Kind::JungleCreeper,
            FindFrameExtra4Kind::BloodCrawler,
            FindFrameExtra4Kind::JungleCreeperWall,
            FindFrameExtra4Kind::BlackRecluseWall,
            FindFrameExtra4Kind::BloodCrawlerWall,
            FindFrameExtra4Kind::BloodFeeder,
            FindFrameExtra4Kind::BloodJelly,
        ] {
            assert_eq!(
                k.frame_count(),
                table[k.vanilla_type() as usize],
                "type {}",
                k.vanilla_type()
            );
        }
    }

    /// 233/234/235 the hornets: the 8-tick band cycle 0/1/1/2/2/1/1
    /// then the reset tick HOLDS row 1 (`NPC.cs:74913-74933` — the
    /// else writes no frame); no sprite/rotation write. Expectation
    /// re-derived by mechanical replay of the C# order.
    #[test]
    fn f27_hornets_cycle_the_eight_tick_bands() {
        for k in [
            FindFrameExtra4Kind::HornetLeafy,
            FindFrameExtra4Kind::HornetSpikey,
            FindFrameExtra4Kind::HornetStingy,
        ] {
            let rows = rows_over(k, input(2.0, -1.0), 16);
            assert_eq!(rows, vec![0, 1, 1, 2, 2, 1, 1, 1, 0, 1, 1, 2, 2, 1, 1, 1]);
            let mut s = FrameState { sprite_dir: -1, rotation: 0.5, ..Default::default() };
            find_frame_extra4(k, &mut s, &input(-3.0, 2.0));
            assert_eq!(s.sprite_dir, -1, "no sprite write");
            assert_eq!(s.rotation, 0.5, "no rotation write");
        }
    }

    /// 236/239 the creepers: the |vx|*1.1 WHILE-loop stepper keeps
    /// the remainder (`NPC.cs:73101-73110`) — vx 6 steps every tick
    /// and the 10th tick steps TWICE (counter 6.0 → row +2); the
    /// wrap fires past row 3 (row 4 is air-only on the 5-row sheet).
    #[test]
    fn f27_creepers_step_with_the_remainder() {
        for k in [FindFrameExtra4Kind::JungleCreeper, FindFrameExtra4Kind::BloodCrawler] {
            let mut s = FrameState::default();
            let mut seen = Vec::new();
            for _ in 0..12 {
                find_frame_extra4(k, &mut s, &input(6.0, 0.0));
                seen.push(s.row);
            }
            assert_eq!(seen, vec![1, 2, 3, 0, 1, 2, 3, 0, 1, 3, 0, 1]);
            assert!((s.counter - 1.2).abs() < 1e-3, "remainder kept: {}", s.counter);
            // vx 2: three ticks a step (2.2/tick), wrap 3 → 0.
            let rows = rows_over(k, input(2.0, 0.0), 20);
            assert_eq!(rows, vec![0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3]);
        }
    }

    /// 236/239 the creepers' air pins: rising row 4, falling row 0,
    /// counter zeroed (`NPC.cs:73087-73099`); the ground flip copies
    /// direction 0 too (`:73100`).
    #[test]
    fn f27_creepers_air_pins_and_the_direction_zero_copy() {
        for k in [FindFrameExtra4Kind::JungleCreeper, FindFrameExtra4Kind::BloodCrawler] {
            let mut s = FrameState { row: 2, counter: 5.0, ..Default::default() };
            find_frame_extra4(k, &mut s, &input(3.0, -2.0));
            assert_eq!((s.row, s.counter), (4, 0.0));
            let mut s = FrameState { row: 4, counter: 5.0, ..Default::default() };
            find_frame_extra4(k, &mut s, &input(3.0, 2.0));
            assert_eq!((s.row, s.counter), (0, 0.0));
            let mut s = FrameState { sprite_dir: 1, ..Default::default() };
            let i = FrameInput { vx: 1.0, direction: 0, ..Default::default() };
            find_frame_extra4(k, &mut s, &i);
            assert_eq!(s.sprite_dir, 0, "the unconditional copy carries 0");
        }
    }

    /// 237/238/240 the wall crawlers: the (|vx|+|vy|)*0.5 accumulator
    /// over the <6/<12/<18/<24 bands (`NPC.cs:73123-73143`) — vx 4 +
    /// vy 2 steps 3/tick, the >= 24 tail zeroes the counter and HOLDS
    /// row 3 for one tick. No sprite/rotation write.
    #[test]
    fn f27_wall_crawlers_ride_the_half_speed_bands() {
        for k in [
            FindFrameExtra4Kind::JungleCreeperWall,
            FindFrameExtra4Kind::BlackRecluseWall,
            FindFrameExtra4Kind::BloodCrawlerWall,
        ] {
            let rows = rows_over(k, input(4.0, 2.0), 14);
            assert_eq!(rows, vec![0, 1, 1, 2, 2, 3, 3, 3, 0, 1, 1, 2, 2, 3]);
            let mut s = FrameState { row: 3, counter: 21.0, sprite_dir: -1, rotation: 0.5 };
            find_frame_extra4(k, &mut s, &input(4.0, 2.0));
            assert_eq!((s.row, s.counter), (3, 0.0), "the tail holds row 3");
            assert_eq!(s.sprite_dir, -1, "no sprite write");
            assert_eq!(s.rotation, 0.5, "no rotation write");
            // Still water-crawling: vx 0 vy 0 never leaves row 0.
            assert_eq!(rows_over(k, input(0.0, 0.0), 5), vec![0, 0, 0, 0, 0]);
        }
    }

    /// 241 BloodFeeder: the fish wet/dry split (`NPC.cs:74189-74225`)
    /// — wet rows 0..3 at 6 ticks a band, dry rows 4/5; BOTH tails
    /// zero the counter and HOLD the row; the direction copy carries
    /// 0. Expectation re-derived by mechanical replay.
    #[test]
    fn f27_blood_feeder_wet_dry_hold_tails() {
        let wet_i = FrameInput { vx: 1.0, wet: true, ..Default::default() };
        let rows = rows_over(FindFrameExtra4Kind::BloodFeeder, wet_i, 26);
        assert_eq!(
            rows,
            vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 0, 0]
        );
        let dry = rows_over(FindFrameExtra4Kind::BloodFeeder, input(1.0, 0.0), 14);
        assert_eq!(dry, vec![4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 4, 4]);
        let mut s = FrameState { sprite_dir: 1, ..Default::default() };
        let i = FrameInput { direction: 0, wet: true, ..Default::default() };
        find_frame_extra4(FindFrameExtra4Kind::BloodFeeder, &mut s, &i);
        assert_eq!(s.sprite_dir, 0, "the unconditional copy carries 0");
    }

    /// 242 BloodJelly: the ai[1]==1 pulse gate cycles rows 4..6 on
    /// the >4 counter with BOTH clamps (`NPC.cs:73921-73937` — the
    /// first pulse tick lifts row 0 into the band); the idle bands
    /// ride rows 0..3 with the >= 23 reset (`:73939-73959`).
    #[test]
    fn f27_blood_jelly_pulse_and_idle_bands() {
        let pulse_i = FrameInput { ai: [0.0, 1.0, 0.0, 0.0], ..Default::default() };
        let rows = rows_over(FindFrameExtra4Kind::BloodJelly, pulse_i, 16);
        assert_eq!(rows, vec![4, 4, 4, 4, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 4, 4]);
        let idle = rows_over(FindFrameExtra4Kind::BloodJelly, input(0.0, 0.0), 24);
        assert_eq!(
            idle,
            vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 0]
        );
        // The upper clamp: a stray row 6+ wraps to 4 on the step.
        let mut s = FrameState { row: 6, counter: 4.0, ..Default::default() };
        find_frame_extra4(FindFrameExtra4Kind::BloodJelly, &mut s, &pulse_i);
        assert_eq!((s.row, s.counter), (4, 0.0));
    }

    /// The seam maps exactly the batch's vanilla types and nothing
    /// else.
    #[test]
    fn f27_seam_covers_exactly_the_slice() {
        for t in 0..697i16 {
            let k = find_frame_kind_extra4(t);
            let claimed = [233, 234, 235, 236, 237, 238, 239, 240, 241, 242];
            assert_eq!(k.is_some(), claimed.contains(&t), "type {t}");
            if let Some(k) = k {
                assert_eq!(k.vanilla_type(), t);
            }
        }
    }
}
