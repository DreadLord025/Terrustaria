//! W4-f26 slice 7e — FindFrame extra wave 7e: the NEXT 5 TRULY
//! unclaimed missing case ids of `NPC.FindFrame` (`NPC.cs:66684-77370`)
//! after the 7d slice — **526/527/528/529/530** — the last two Desert
//! Ghouls, the Desert Lamia pair and the walking Desert Scorpion
//! (`ID/NPCID.cs:11729-11737`).
//!
//! The census (the 7c docs method, `docs/research/68` addendum): 122
//! unclaimed after the pushed 7d tip `63417edd`'s 5 claims; − THIS
//! slice's 5 = **117 unclaimed**. The re-probe at THIS slice's census
//! time: the d41 pushed tip `4ed22f0b` is still the CLAIM-only commit
//! (the SetDefaults census, ZERO FindFrame arms — mechanical grep over
//! its `npc.rs`/`npc/data.rs` for all five ids = the shop-item 530 in
//! the Mechanic test list ONLY, no FindFrame arm); the d42 pushed tip
//! `88df5809` is the WIP 578-587 enum checkpoint (same shop-item-only
//! grep result); the f27 pushed tip `0e187428`'s `npc_findframe_extra8`
//! /`extra9` carry NO 526/527/528/529/530 arm (mechanical grep = no
//! match); the d40 pushed tip `8fe1d6d3`'s roster stops at 490; this
//! branch's own `npc.rs` has no arm for any of the five (no
//! `DesertGhoulCrimson`/`DesertGhoulHallow`/`DesertLamia*`/
//! `DesertScorpionWalk` kind exists in `NpcKind` yet — the 530 hit at
//! `npc.rs:24578` is the Mechanic shop ITEM id, not an NPC arm).
//!
//! The 524..527 group completes: 526 DesertGhoulCrimson / 527
//! DesertGhoulHallow are the 7d-deferred pair riding the SAME
//! `case 524/525/526/527` body (`NPC.cs:69908-69945`, labels
//! `:69910-69911`) — the machine is copied VERBATIM from
//! `npc_findframe_extra7d.rs` (the 7c nine-salamander ONE-group
//! precedent: each slice's module carries its own copy, the wiring
//! round keeps one). 528/529 DesertLamiaLight/Dark ride their own
//! group (`:69946-69977`) — the ghoul shape MINUS the lift-to-2, with
//! the walk wrapping at row >= count − 1 back to 1 and the air pinning
//! row count − 1 (the last row is the AIR row, not walkable). 530
//! DesertScorpionWalk rides the `case 164/236/239/530` body
//! (`:73083-73111`, label `:73086`) — the air zeroes the counter and
//! pins row 4 rising / 0 falling and BREAKS (sprite UNTOUCHED); the
//! grounded branch copies the sprite UNCONDITIONALLY (no `!= 0` gate)
//! and feeds a WHILE-loop stepper (multiple rows per tick at high
//! |vx|) over rows 0..3 with the STRICT > 3 wrap. Only 530 is this
//! wave's; 164/236/239 are port-covered (the scorpion group's low-id
//! members). Landed 5 kinds in ONE commit.
//!
//! Shape: the kinds are NOT in `NpcKind` yet (the d41 roster wave owns
//! them), so — the `npc_findframe_extra7d.rs` precedent — the frame
//! machines land first as pure functions over a small
//! [`FrameState`]/[`FrameInput`] pair. The wiring seam is
//! [`find_frame_kind_extra7e`]: when a kind joins `NpcKind`, its
//! `find_frame` arm copies the state fields out, runs one tick and
//! copies them back (the `Npc::find_frame_wave_a` bridge precedent).
//!
//! RNG: NONE — all five arms draw ZERO `Main.rand` (mechanical grep
//! over `NPC.cs:69908-69977` + `:73083-73111` = ZERO). The 7d flare's
//! resolved-draw seams are NOT needed this wave; `FrameInput` carries
//! only vx/vy (the 7c shape). No world reads either: none of the five
//! machines touches `Framing`/`Main.tile`, no `ai`/`localAI` reads —
//! the wiring order is therefore unconstrained beyond the C#'s own
//! tick order (velocity settled → FindFrame; the scorpion's 530→531
//! wall transform at `:58892-58899` rides the AI, NOT FindFrame).
//!
//! Direction: no arm WRITES `Entity.direction` this wave, but the
//! ghoul/lamia grounded copies and the scorpion's unconditional copy
//! READ it, so `direction` rides [`FrameState`] (the 7d read/write
//! shape kept for the uniform wiring copy; the wiring step copies
//! `Npc::facing` to/from it).
//!
//! Row convention: the port tracks frame ROW indices, not pixels —
//! the C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer,
//! `:67136-67137`'s rows), counted entry-by-entry in
//! [`FindFrameExtra7eKind::frame_count`] and cross-checked against the
//! port's own `render/entities.rs::NPC_FRAME_COUNT` table.

// The wiring is a later round (the kinds are not in `NpcKind` yet) —
// the `npc_findframe_extra7d.rs` precedent.
#![allow(dead_code)]

/// The extra-wave-7e kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra7eKind {
    /// `NPCID.DesertGhoulCrimson` = 526 (`NPCID.cs:11729`) — the
    /// 524..527 group (`NPC.cs:69910`); the 7d-deferred pair.
    DesertGhoulCrimson,
    /// `NPCID.DesertGhoulHallow` = 527 (`NPCID.cs:11731`) — the same
    /// group (`NPC.cs:69911`).
    DesertGhoulHallow,
    /// `NPCID.DesertLamiaLight` = 528 (`NPCID.cs:11733`) — the 528/529
    /// group (`NPC.cs:69946`).
    DesertLamiaLight,
    /// `NPCID.DesertLamiaDark` = 529 (`NPCID.cs:11735`) — the same
    /// group (`NPC.cs:69947`).
    DesertLamiaDark,
    /// `NPCID.DesertScorpionWalk` = 530 (`NPCID.cs:11737`) — the
    /// 164/236/239/530 group (`NPC.cs:73086`); only 530 is this
    /// wave's (164/236/239 are port-covered).
    DesertScorpionWalk,
}

impl FindFrameExtra7eKind {
    /// The vanilla `NPC.type` id.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::DesertGhoulCrimson => 526,
            Self::DesertGhoulHallow => 527,
            Self::DesertLamiaLight => 528,
            Self::DesertLamiaDark => 529,
            Self::DesertScorpionWalk => 530,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// counted entry-by-entry): 526/527 = 8 (`:67136`'s 520-529 row,
    /// entries 7/8), 528/529 = 9 (`:67136`, entries 9/10), 530 = 5
    /// (`:67137`'s 530-539 row, entry 1).
    pub fn frame_count(self) -> u32 {
        match self {
            Self::DesertGhoulCrimson | Self::DesertGhoulHallow => 8,
            Self::DesertLamiaLight | Self::DesertLamiaDark => 9,
            Self::DesertScorpionWalk => 5,
        }
    }
}

/// The vanilla-id wiring seam: `Some(kind)` for exactly this wave's
/// ids (the `find_frame_kind_extra7d` precedent). The wiring round
/// hooks it at the `npc.rs::find_frame` head by `cs_id`.
pub fn find_frame_kind_extra7e(vanilla_type: i16) -> Option<FindFrameExtra7eKind> {
    Some(match vanilla_type {
        526 => FindFrameExtra7eKind::DesertGhoulCrimson,
        527 => FindFrameExtra7eKind::DesertGhoulHallow,
        528 => FindFrameExtra7eKind::DesertLamiaLight,
        529 => FindFrameExtra7eKind::DesertLamiaDark,
        530 => FindFrameExtra7eKind::DesertScorpionWalk,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection` and
/// `Entity.direction` (the grounded copies READ it; no arm writes it).
/// The wiring step copies them to/from `Npc` (`direction` ↔
/// `Npc::facing`). `rotation`/`local_ai` are carried for the uniform
/// 7d wiring copy, never read/written here.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter` (the C# field is a `double`, `NPC.cs:6485`;
    /// the port's f32 is the existing `npc.rs` convention — the integer
    /// accumulators here are exact in both).
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — no arm writes it this wave.
    pub rotation: f32,
    /// `Entity.direction` (±1) — the ghoul/lamia grounded copy is
    /// `!= 0`-gated (`:69914-69917`/`:69950-69953`, 0 leaves the sprite
    /// alone); the scorpion's grounded copy is UNCONDITIONAL
    /// (`:73100`, a 0 direction zeroes the sprite).
    pub direction: i32,
    /// `NPC.localAI[0]`/`[1]` — never read/written this wave; carried
    /// for the uniform wiring copy.
    pub local_ai: [f32; 2],
}

/// The per-tick inputs the arms read: `velocity` only. No `ai` reads,
/// no RNG draws, no world reads this wave (the header's census).
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-7e kinds — one
/// tick.
pub fn find_frame_extra7e(kind: FindFrameExtra7eKind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // `case 524/525/526/527` (`NPC.cs:69908-69945`): the Desert
        // Ghouls (only 526/527 are this wave's — the machine copied
        // VERBATIM from the 7d module). The grounded branch
        // (`:69912-69939`): the `!= 0`-gated direction copy
        // (`:69914-69917` — 0 leaves the sprite alone); the idle
        // (`:69918-69923`) zeroes BOTH row and counter and BREAKS;
        // the walk lifts any row <= 1 to 2 (`:69924-69927`),
        // accumulates `|vx|` THEN `++` (`:69928-69929` — the +1 rides
        // AFTER the |vx| add), the STRICT > 9 counter steps a row
        // (`:69930-69934`), wrapping at row >= the 8-row count back
        // to 2 (`:69935-69938` — rows 2..7 are the walk strip). The
        // air (`:69940-69944`) pins row 1 and zeroes the counter,
        // sprite UNTOUCHED. No rotation write.
        FindFrameExtra7eKind::DesertGhoulCrimson | FindFrameExtra7eKind::DesertGhoulHallow => {
            if i.vy == 0.0 {
                // `:69914-69917`.
                if s.direction != 0 {
                    s.sprite_dir = s.direction;
                }
                if i.vx == 0.0 {
                    // `:69918-69923`.
                    s.row = 0;
                    s.counter = 0.0;
                    return;
                }
                // `:69924-69938`.
                if s.row <= 1 {
                    s.row = 2;
                }
                s.counter += i.vx.abs();
                s.counter += 1.0;
                if s.counter > 9.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
                if s.row >= kind.frame_count() {
                    s.row = 2;
                }
            } else {
                // `:69940-69944`.
                s.row = 1;
                s.counter = 0.0;
            }
        }
        // `case 528/529` (`NPC.cs:69946-69977`): the Desert Lamias.
        // The ghoul shape MINUS the lift-to-2: the grounded branch
        // (`:69948-69971`) keeps the `!= 0`-gated direction copy
        // (`:69950-69953`) and the idle row-0 + counter-0 BREAK
        // (`:69954-69959`); the walk accumulates `|vx|` THEN `++`
        // (`:69960-69961`), the STRICT > 9 counter steps a row
        // (`:69962-69966`) — but the wrap fires at row >= count − 1
        // back to 1 (`:69967-69970` — rows 1..7 are the walk strip on
        // the 9-row sheet, row 0 is idle-only and row 8 is the AIR
        // row). The air (`:69972-69976`) pins row count − 1 (row 8)
        // and zeroes the counter, sprite UNTOUCHED. NO lift: a row-0
        // start walks 0 → 1 → 2 … (the ghoul would snap it to 2).
        FindFrameExtra7eKind::DesertLamiaLight | FindFrameExtra7eKind::DesertLamiaDark => {
            if i.vy == 0.0 {
                // `:69950-69953`.
                if s.direction != 0 {
                    s.sprite_dir = s.direction;
                }
                if i.vx == 0.0 {
                    // `:69954-69959`.
                    s.row = 0;
                    s.counter = 0.0;
                    return;
                }
                // `:69960-69970`.
                s.counter += i.vx.abs();
                s.counter += 1.0;
                if s.counter > 9.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
                if s.row >= kind.frame_count() - 1 {
                    s.row = 1;
                }
            } else {
                // `:69972-69976`.
                s.row = kind.frame_count() - 1;
                s.counter = 0.0;
            }
        }
        // `case 164/236/239/530` (`NPC.cs:73083-73111`): the walking
        // Desert Scorpion (only 530 is this wave's). The air
        // (`:73087-73099`) zeroes the counter FIRST, pins row 4
        // rising (vy < 0) / row 0 falling and BREAKS — sprite
        // UNTOUCHED. The grounded branch copies the sprite
        // UNCONDITIONALLY (`:73100` — no `!= 0` gate; direction 0
        // zeroes the sprite), accumulates `|vx| * 1.1` with NO +1
        // (`:73101`), then the WHILE `>= 6` stepper (`:73102-73110`)
        // subtracts 6 and steps a row PER LOOP — multiple rows per
        // tick at high |vx| — wrapping at row > 3 back to 0 (the
        // STRICT > on the 4-row walk strip; row 4 is the air-rise
        // row only). No idle branch: vx == 0 grounded keeps the
        // counter/row HELD (the accumulator adds 0). No rotation.
        FindFrameExtra7eKind::DesertScorpionWalk => {
            if i.vy != 0.0 {
                // `:73087-73099`.
                s.counter = 0.0;
                s.row = if i.vy < 0.0 { 4 } else { 0 };
                return;
            }
            // `:73100`.
            s.sprite_dir = s.direction;
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy }
    }

    fn rows_over(kind: FindFrameExtra7eKind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra7e(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The frame-count table agrees with the port's verbatim
    /// `NPC_FRAME_COUNT` (`Main.cs:67082` initializer cross-check).
    #[test]
    fn f26s7e_frame_counts_match_the_port_table() {
        let table = crate::render::entities::NPC_FRAME_COUNT;
        for k in [
            FindFrameExtra7eKind::DesertGhoulCrimson,
            FindFrameExtra7eKind::DesertGhoulHallow,
            FindFrameExtra7eKind::DesertLamiaLight,
            FindFrameExtra7eKind::DesertLamiaDark,
            FindFrameExtra7eKind::DesertScorpionWalk,
        ] {
            assert_eq!(
                k.frame_count(),
                table[k.vanilla_type() as usize],
                "{k:?} ({})",
                k.vanilla_type()
            );
        }
    }

    /// The 7d-deferred ghoul pair rides the VERBATIM 7d machine: the
    /// walk lifts to row 2 and accumulates |vx| THEN +1 with the
    /// STRICT > 9 step over rows 2..7 (replay: vx 3 → +4 a tick,
    /// [2,2,3,3,3,4,4,4,5,5,5,6,6,6,7,7,7,2,2]; vx 1 → +2 a tick, the
    /// step lands every 5th tick) — the SAME vectors the 7d
    /// `f26s7d_the_ghoul_walk_bands_and_wrap` pins for 524/525.
    #[test]
    fn f26s7e_the_ghoul_pair_walk_bands_and_wrap() {
        for k in [FindFrameExtra7eKind::DesertGhoulCrimson, FindFrameExtra7eKind::DesertGhoulHallow] {
            assert_eq!(
                rows_over(k, input(3.0, 0.0), 19),
                vec![2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6, 7, 7, 7, 2, 2]
            );
            assert_eq!(
                rows_over(k, input(1.0, 0.0), 22),
                vec![2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 6, 6, 6]
            );
            // The lift: row 1 → 2, the grounded copy rides direction
            // -1, counter 0 + 3 + 1 = 4.
            let mut s = FrameState { row: 1, sprite_dir: 1, direction: -1, ..Default::default() };
            find_frame_extra7e(k, &mut s, &input(-3.0, 0.0));
            assert_eq!((s.row, s.counter, s.sprite_dir), (2, 4.0, -1));
        }
    }

    /// The ghoul idle zeroes BOTH row and counter (the sprite copy
    /// still rides direction; 0 leaves the sprite alone); the air
    /// pins row 1 + zeroes the counter with the sprite UNTOUCHED —
    /// the 7d `f26s7d_the_ghoul_idle_and_air_pins` vectors verbatim.
    #[test]
    fn f26s7e_the_ghoul_pair_idle_and_air_pins() {
        for k in [FindFrameExtra7eKind::DesertGhoulCrimson, FindFrameExtra7eKind::DesertGhoulHallow] {
            // The idle: direction 0 keeps sprite -1, row/counter zeroed.
            let mut s = FrameState { row: 5, counter: 7.0, sprite_dir: -1, direction: 0, ..Default::default() };
            find_frame_extra7e(k, &mut s, &input(0.0, 0.0));
            assert_eq!((s.row, s.counter, s.sprite_dir), (0, 0.0, -1));
            // The air: row 1, counter 0, sprite 1 HELD (direction -1
            // not copied — the air branch never touches the sprite).
            let mut s = FrameState { row: 5, counter: 7.0, sprite_dir: 1, direction: -1, ..Default::default() };
            find_frame_extra7e(k, &mut s, &input(3.0, 2.0));
            assert_eq!((s.row, s.counter, s.sprite_dir), (1, 0.0, 1));
        }
    }

    /// The lamia walk has NO lift-to-2: a row-0 start walks 0 → 1
    /// (replay: vx 3 → +4 a tick, [0,0,1,1,1,2,2,2,3,3,3,4,4,4,5,5,
    /// 5,6,6,6,7,7,7,1,1] — the wrap fires at row >= 9 − 1 = 8 back
    /// to 1, so rows 1..7 are the walk strip and row 8 NEVER lands in
    /// the grounded branch). BOTH lamias ride the ONE group machine.
    #[test]
    fn f26s7e_the_lamia_no_lift_walk_and_count_minus_one_wrap() {
        for k in [FindFrameExtra7eKind::DesertLamiaLight, FindFrameExtra7eKind::DesertLamiaDark] {
            assert_eq!(
                rows_over(k, input(3.0, 0.0), 25),
                vec![0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6, 7, 7, 7, 1, 1]
            );
            // The wrap edge: row 7 + counter 9 → +4 → 13 > 9 → row 8
            // → >= 8 → 1, counter 0 (the count − 1 clamp, NOT the
            // ghoul's count clamp-to-2).
            let mut s = FrameState { row: 7, counter: 9.0, ..Default::default() };
            find_frame_extra7e(k, &mut s, &input(3.0, 0.0));
            assert_eq!((s.row, s.counter), (1, 0.0));
        }
    }

    /// The lamia idle zeroes BOTH row and counter (the `!= 0`-gated
    /// sprite copy still rides direction); the air pins row
    /// count − 1 = 8 (NOT the ghoul's row 1) + zeroes the counter,
    /// sprite UNTOUCHED.
    #[test]
    fn f26s7e_the_lamia_idle_and_last_row_air_pin() {
        for k in [FindFrameExtra7eKind::DesertLamiaLight, FindFrameExtra7eKind::DesertLamiaDark] {
            let mut s = FrameState { row: 5, counter: 7.0, sprite_dir: -1, direction: 0, ..Default::default() };
            find_frame_extra7e(k, &mut s, &input(0.0, 0.0));
            assert_eq!((s.row, s.counter, s.sprite_dir), (0, 0.0, -1));
            let mut s = FrameState { row: 5, counter: 7.0, sprite_dir: 1, direction: -1, ..Default::default() };
            find_frame_extra7e(k, &mut s, &input(3.0, -2.0));
            assert_eq!((s.row, s.counter, s.sprite_dir), (8, 0.0, 1));
            // The grounded copy: direction -1 rides, counter 0+3+1=4.
            let mut s = FrameState { row: 0, sprite_dir: 1, direction: -1, ..Default::default() };
            find_frame_extra7e(k, &mut s, &input(-3.0, 0.0));
            assert_eq!((s.row, s.counter, s.sprite_dir), (0, 4.0, -1));
        }
    }

    /// The scorpion's air branch zeroes the counter FIRST then pins
    /// row 4 rising / row 0 falling and BREAKS — sprite UNTOUCHED
    /// (direction -1 never copies in the air).
    #[test]
    fn f26s7e_the_scorpion_air_pins() {
        let k = FindFrameExtra7eKind::DesertScorpionWalk;
        let mut s = FrameState { row: 2, counter: 5.0, sprite_dir: 1, direction: -1, ..Default::default() };
        find_frame_extra7e(k, &mut s, &input(3.0, -2.0));
        assert_eq!((s.row, s.counter, s.sprite_dir), (4, 0.0, 1));
        let mut s = FrameState { row: 2, counter: 5.0, sprite_dir: 1, direction: -1, ..Default::default() };
        find_frame_extra7e(k, &mut s, &input(3.0, 2.0));
        assert_eq!((s.row, s.counter, s.sprite_dir), (0, 0.0, 1));
    }

    /// The scorpion's grounded walk: the UNCONDITIONAL sprite copy,
    /// the |vx|*1.1 NO-+1 accumulator and the WHILE >= 6 stepper with
    /// the STRICT > 3 wrap over rows 0..3. Replay vx 3 (+3.3 a tick,
    /// the step at ticks 2/4/6/8/10/11 — the 6/3.3 ≈ 1.82-tick
    /// cadence): [0,1,1,2,2,3,3,0,0,1,2,2]; vx 1 (+1.1 a tick, the
    /// step at ticks 6/11 — 6.6 then 6.1 after the 0.6 remainder):
    /// [0,0,0,0,0,1,1,1,1,1,2,2,2]. vx 20 (+22) steps THREE rows in
    /// one tick — the while-loop multi-step the single `if` steppers
    /// can't do.
    #[test]
    fn f26s7e_the_scorpion_while_stepper() {
        let k = FindFrameExtra7eKind::DesertScorpionWalk;
        assert_eq!(
            rows_over(k, input(3.0, 0.0), 12),
            vec![0, 1, 1, 2, 2, 3, 3, 0, 0, 1, 2, 2]
        );
        assert_eq!(
            rows_over(k, input(1.0, 0.0), 13),
            vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2]
        );
        // The multi-step: counter 0 + 11 → row 3+1=4 wraps to 0
        // (counter 5), 5 < 6 stops — ONE subtract, one row.
        let mut s = FrameState { row: 3, counter: 0.0, ..Default::default() };
        find_frame_extra7e(k, &mut s, &input(10.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 5.0));
        // TWO rows in one tick: vx 20 → +22 → row 3+1=4 wraps to 0
        // (counter 16), 16 >= 6 → row 1 (counter 10), 10 >= 6 → row 2
        // (counter 4), 4 < 6 stops — three subtracts, rows 0/1/2.
        let mut s = FrameState { row: 3, counter: 0.0, ..Default::default() };
        find_frame_extra7e(k, &mut s, &input(20.0, 0.0));
        assert_eq!((s.row, s.counter), (2, 4.0));
        // The idle HOLD: vx 0 grounded adds 0 — row/counter kept,
        // the sprite copy still UNCONDITIONAL (direction 0 → 0).
        let mut s = FrameState { row: 2, counter: 5.9, sprite_dir: 1, direction: 0, ..Default::default() };
        find_frame_extra7e(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter, s.sprite_dir), (2, 5.9, 0));
        // The grounded copy rides direction -1.
        let mut s = FrameState { sprite_dir: 1, direction: -1, ..Default::default() };
        find_frame_extra7e(k, &mut s, &input(-3.0, 0.0));
        assert_eq!(s.sprite_dir, -1);
    }

    /// The seam maps exactly this wave's 5 ids — the prior-wave
    /// 524/525, the port-covered group co-members 164/236/239, the
    /// other-lane 531/532/533 and the no-case 515 stay `None`.
    #[test]
    fn f26s7e_the_seam_maps_exactly_the_slice() {
        for (id, k) in [
            (526, FindFrameExtra7eKind::DesertGhoulCrimson),
            (527, FindFrameExtra7eKind::DesertGhoulHallow),
            (528, FindFrameExtra7eKind::DesertLamiaLight),
            (529, FindFrameExtra7eKind::DesertLamiaDark),
            (530, FindFrameExtra7eKind::DesertScorpionWalk),
        ] {
            assert_eq!(find_frame_kind_extra7e(id), Some(k));
            assert_eq!(k.vanilla_type(), id);
        }
        for id in [164, 236, 239, 515, 524, 525, 531, 532, 533] {
            assert_eq!(find_frame_kind_extra7e(id), None, "type {id}");
        }
    }

    /// The 5-kind strip sweep: every kind ticks 27 input combos
    /// (vx 0/1/3 × vy 0/-2/2 × direction 1/-1/0) for 10 ticks each
    /// from a fresh state — every row stays inside its own sheet
    /// (the 7d 540-combo sweep precedent, the resolved-draw axes
    /// dropped with the zero-RNG wave).
    #[test]
    fn f26s7e_the_5_kind_strip_sweep_stays_in_sheet() {
        let kinds = [
            FindFrameExtra7eKind::DesertGhoulCrimson,
            FindFrameExtra7eKind::DesertGhoulHallow,
            FindFrameExtra7eKind::DesertLamiaLight,
            FindFrameExtra7eKind::DesertLamiaDark,
            FindFrameExtra7eKind::DesertScorpionWalk,
        ];
        for k in kinds {
            let mut combo = 0;
            for vx in [0.0f32, 1.0, 3.0] {
                for vy in [0.0f32, -2.0, 2.0] {
                    for direction in [1i32, -1, 0] {
                        let mut s = FrameState { direction, ..Default::default() };
                        let i = input(vx, vy);
                        for _ in 0..10 {
                            find_frame_extra7e(k, &mut s, &i);
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
