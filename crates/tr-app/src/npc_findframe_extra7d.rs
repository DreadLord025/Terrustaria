//! W4-f26 slice 7d — FindFrame extra wave 7d: the NEXT 5 TRULY
//! unclaimed missing case ids of `NPC.FindFrame` (`NPC.cs:66684-77370`)
//! after the 7c slice — **516/521/523/524/525** — the Solar Flare, the
//! Ancient Cultist Squidhead, the Ancient Doom and the first two Desert
//! Ghouls (`ID/NPCID.cs:11709-11727`).
//!
//! The census (the 7c docs method, `docs/research/68` addendum): 127
//! unclaimed after the pushed 7c tip `a2e47596`'s 15 claims; − THIS
//! slice's 5 = **122 unclaimed**. The re-probe at THIS slice's census
//! time: the d41 pushed tip `4ed22f0b` is a CLAIM-only commit (the
//! SetDefaults census, ZERO FindFrame arms — mechanical grep over its
//! `npc.rs`/`npc/data.rs` for all five ids = no match); the f27 pushed
//! tip `72448094`'s `npc_findframe_extra8.rs` carries NO 516/521/523/
//! 524/525 arm (mechanical grep = no match); this branch's own `npc.rs`
//! has no arm for any of the five (no `SolarFlare`/`AncientDoom`/
//! `DesertGhoul`/`Squidhead` kind exists in `NpcKind` yet).
//!
//! The band skips: 510-515/517/519/522 have NO case labels in the
//! FindFrame switch (the 51419/51462 hits are OTHER switches); 518
//! SolarSpearman / 520 MartianWalker have labels (`NPC.cs:70231` /
//! `:71889`) but ride other lanes; 526/527 DesertGhoulCrimson/Hallow
//! ride the SAME 524..527 group body (`NPC.cs:69910-69911`) — only
//! 524/525 are this wave's, 526/527 ride a later slice (the 7c
//! 508/532/580/582 group precedent). Landed 5 kinds per commit.
//!
//! Shape: the kinds are NOT in `NpcKind` yet (the d41 roster wave owns
//! them), so — the `npc_findframe_extra7c.rs` precedent — the frame
//! machines land first as pure functions over a small
//! [`FrameState`]/[`FrameInput`] pair. The wiring seam is
//! [`find_frame_kind_extra7d`]: when a kind joins `NpcKind`, its
//! `find_frame` arm copies the state fields out, runs one tick and
//! copies them back (the `Npc::find_frame_wave_a` bridge precedent).
//!
//! RNG: the flare arm is the ONE exception to the extra waves'
//! zero-RNG rule — `NPC.cs:70744-70745` draws `Main.rand.Next(3)`
//! TWICE (the init row, then the counter kick) inside the one-time
//! `localAI[1] == 0` latch. The module stays PURE: the two draws ride
//! [`FrameInput`] as RESOLVED seams (the 7b `slope`/`frame_x36`
//! resolved-world-read precedent) — the wiring round must draw them
//! from the caller's `Main.rand` stream IN ORDER (row first, counter
//! second) and only on the latch tick, so the stream position is the
//! caller's to keep. The other four arms draw NOTHING (mechanical
//! grep over `:72103-72137` + `:69908-69945` = ZERO).
//!
//! Direction: the squidhead arm WRITES `Entity.direction` every tick
//! (`NPC.cs:72104-72111`), so this wave's `direction` rides
//! [`FrameState`] (read/write), NOT [`FrameInput`] — the ghoul grounded
//! copy reads the state's direction. The wiring step copies
//! `Npc::facing` to/from it.
//!
//! Row convention: the port tracks frame ROW indices, not pixels —
//! the C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer,
//! `:67135-67136`'s rows), counted entry-by-entry in
//! [`FindFrameExtra7dKind::frame_count`] and cross-checked against the
//! port's own `render/entities.rs::NPC_FRAME_COUNT` table.
//!
//! No world reads this wave: none of the five machines touches
//! `Framing`/`Main.tile` — the `FrameInput` needs only vx/vy plus the
//! two resolved flare draws.

// The wiring is a later round (the kinds are not in `NpcKind` yet) —
// the `npc_findframe_extra7c.rs` precedent.
#![allow(dead_code)]

/// The extra-wave-7d kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra7dKind {
    /// `NPCID.SolarFlare` = 516 (`NPCID.cs:11709`) — the lone
    /// `case 516` (`NPC.cs:70740`).
    SolarFlare,
    /// `NPCID.AncientCultistSquidhead` = 521 (`NPCID.cs:11719`) — the
    /// lone `case 521` (`NPC.cs:72103`).
    AncientCultistSquidhead,
    /// `NPCID.AncientDoom` = 523 (`NPCID.cs:11723`) — the lone
    /// `case 523` (`NPC.cs:72127`).
    AncientDoom,
    /// `NPCID.DesertGhoul` = 524 (`NPCID.cs:11725`) — the 524..527
    /// group (`NPC.cs:69908`); only 524/525 are this wave's (526/527
    /// ride a later slice).
    DesertGhoul,
    /// `NPCID.DesertGhoulCorruption` = 525 (`NPCID.cs:11727`) — the
    /// same group (`NPC.cs:69909`).
    DesertGhoulCorruption,
}

impl FindFrameExtra7dKind {
    /// The vanilla `NPC.type` id.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::SolarFlare => 516,
            Self::AncientCultistSquidhead => 521,
            Self::AncientDoom => 523,
            Self::DesertGhoul => 524,
            Self::DesertGhoulCorruption => 525,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// counted entry-by-entry): 516 = 4 (`:67135`'s 510-519 row,
    /// entry 7), 521 = 4 (`:67136`'s 520-529 row, entry 2), 523 = 5
    /// (`:67136`, entry 4), 524/525 = 8 (`:67136`, entries 5/6).
    pub fn frame_count(self) -> u32 {
        match self {
            Self::SolarFlare | Self::AncientCultistSquidhead => 4,
            Self::AncientDoom => 5,
            Self::DesertGhoul | Self::DesertGhoulCorruption => 8,
        }
    }
}

/// The vanilla-id wiring seam: `Some(kind)` for exactly this wave's
/// ids (the `find_frame_kind_extra7c` precedent). The wiring round
/// hooks it at the `npc.rs::find_frame` head by `cs_id`.
pub fn find_frame_kind_extra7d(vanilla_type: i16) -> Option<FindFrameExtra7dKind> {
    Some(match vanilla_type {
        516 => FindFrameExtra7dKind::SolarFlare,
        521 => FindFrameExtra7dKind::AncientCultistSquidhead,
        523 => FindFrameExtra7dKind::AncientDoom,
        524 => FindFrameExtra7dKind::DesertGhoul,
        525 => FindFrameExtra7dKind::DesertGhoulCorruption,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection`,
/// `rotation` (the squidhead aim), `Entity.direction` (the squidhead
/// WRITES it every tick; the ghoul grounded copy reads it) and the
/// `localAI[1]` flare latch. The wiring step copies them to/from
/// `Npc` (`direction` ↔ `Npc::facing`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter`.
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — the squidhead atan2 aim + vx drift; the other
    /// four arms never write it.
    pub rotation: f32,
    /// `Entity.direction` (±1) — the squidhead sets it off the vx
    /// sign EVERY tick (`NPC.cs:72104-72111`); the ghoul grounded copy
    /// is `!= 0`-gated (`:69914-69917`, 0 leaves the sprite alone).
    pub direction: i32,
    /// `NPC.localAI[0]`/`[1]` — only `[1]` rides this wave: the
    /// flare's one-time init latch (`NPC.cs:70741-70746`). `[0]` is
    /// carried for the uniform wiring copy, never read/written here.
    pub local_ai: [f32; 2],
}

/// The per-tick inputs the arms read: `velocity` plus the two
/// RESOLVED flare RNG draws (the header's RNG note). No `ai` reads
/// this wave, no world reads.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// The RESOLVED `Main.rand.Next(3)` init-row draw
    /// (`NPC.cs:70744`, `frame.Y = num * Main.rand.Next(3)`). Only the
    /// flare's latch tick consumes it; the wiring round draws it from
    /// the caller's stream FIRST.
    pub flare_row_draw: u32,
    /// The RESOLVED `Main.rand.Next(3)` counter-kick draw
    /// (`NPC.cs:70745`, `frameCounter += Main.rand.Next(3)`). Drawn
    /// SECOND, same latch tick only.
    pub flare_counter_draw: u32,
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-7d kinds — one
/// tick.
pub fn find_frame_extra7d(kind: FindFrameExtra7dKind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // `case 516` (`NPC.cs:70740-70757`): the Solar Flare. The
        // one-time `localAI[1] == 0` latch (`:70741-70746`) sets the
        // latch to 1, pins the row to the FIRST resolved draw and
        // kicks the counter by the SECOND (both `Main.rand.Next(3)`
        // in the C#, drawn in that order — the header's RNG seam).
        // Then EVERY tick (the latch tick included) `++frameCounter`
        // (`:70747`) with the STRICT > 3 step (`:70748-70756`): the
        // row steps and the wrap check is `frame.Y > num * 3` — row
        // > 3, NOT >= the count (identical on the 4-row sheet, kept
        // verbatim). No sprite/rotation/direction write.
        FindFrameExtra7dKind::SolarFlare => {
            if s.local_ai[1] == 0.0 {
                // `:70741-70746`.
                s.local_ai[1] = 1.0;
                s.row = i.flare_row_draw;
                s.counter += i.flare_counter_draw as f32;
            }
            // `:70747-70756`.
            s.counter += 1.0;
            if s.counter > 3.0 {
                s.counter = 0.0;
                s.row += 1;
                if s.row > 3 {
                    s.row = 0;
                }
            }
        }
        // `case 521` (`NPC.cs:72103-72126`): the Ancient Cultist
        // Squidhead. `Entity.direction` is SET off the vx sign every
        // tick (`:72104-72111` — vx < 0 → -1, ELSE 1, so vx == 0
        // faces +1); the `!= 0` sprite copy (`:72112-72115`) is
        // therefore always taken. The rotation is the atan2 aim
        // MIRRORED by the sprite sign plus the vx * 0.1 drift
        // (`:72116` — `Math.Atan2(vy * spriteDir, vx * spriteDir) +
        // vx * 0.1f`; the f32 `atan2` is the `npc.rs:7888` eye-aim
        // precedent). The `++frameCounter >= 5` step (`:72117-72125`)
        // walks the FULL 4-row sheet, wrapping at row >= the count
        // back to 0. No velocity gate — the squid spins in place.
        FindFrameExtra7dKind::AncientCultistSquidhead => {
            // `:72104-72111`.
            s.direction = if i.vx < 0.0 { -1 } else { 1 };
            // `:72112-72115` — always taken (direction just set ±1).
            if s.direction != 0 {
                s.sprite_dir = s.direction;
            }
            // `:72116`.
            s.rotation = (i.vy * s.sprite_dir as f32)
                .atan2(i.vx * s.sprite_dir as f32)
                + i.vx * 0.1;
            // `:72117-72125`.
            s.counter += 1.0;
            if s.counter >= 5.0 {
                s.row += 1;
                s.counter = 0.0;
                if s.row >= kind.frame_count() {
                    s.row = 0;
                }
            }
        }
        // `case 523` (`NPC.cs:72127-72137`): the Ancient Doom. The
        // bare `++frameCounter >= 6` stepper over the FULL 5-row
        // sheet, wrapping at row >= the count back to 0. NOTHING
        // else — no velocity/sprite/rotation/direction touch.
        FindFrameExtra7dKind::AncientDoom => {
            // `:72128-72136`.
            s.counter += 1.0;
            if s.counter >= 6.0 {
                s.row += 1;
                s.counter = 0.0;
                if s.row >= kind.frame_count() {
                    s.row = 0;
                }
            }
        }
        // `case 524/525/526/527` (`NPC.cs:69908-69945`): the Desert
        // Ghouls (only 524/525 are this wave's). The grounded branch
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
        FindFrameExtra7dKind::DesertGhoul | FindFrameExtra7dKind::DesertGhoulCorruption => {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn rows_over(kind: FindFrameExtra7dKind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra7d(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The frame-count table agrees with the port's verbatim
    /// `NPC_FRAME_COUNT` (`Main.cs:67082` initializer cross-check).
    #[test]
    fn f26s7d_frame_counts_match_the_port_table() {
        let table = crate::render::entities::NPC_FRAME_COUNT;
        for k in [
            FindFrameExtra7dKind::SolarFlare,
            FindFrameExtra7dKind::AncientCultistSquidhead,
            FindFrameExtra7dKind::AncientDoom,
            FindFrameExtra7dKind::DesertGhoul,
            FindFrameExtra7dKind::DesertGhoulCorruption,
        ] {
            assert_eq!(
                k.frame_count(),
                table[k.vanilla_type() as usize],
                "{k:?} ({})",
                k.vanilla_type()
            );
        }
    }

    /// The flare's one-time latch consumes BOTH resolved draws (row
    /// first, counter second) then ticks the counter the SAME tick;
    /// the latch never re-fires (later draws are ignored). Replay
    /// (draws 2/2): row 2 + counter 2 + 1 = 3 on tick 1, then the
    /// > 3 cycle [2,3,3,3,3,0,0,0,0,1,1,1,1,2,2,2].
    #[test]
    fn f26s7d_the_flare_latch_draws_and_cycle() {
        let k = FindFrameExtra7dKind::SolarFlare;
        let mut s = FrameState::default();
        let i = FrameInput { flare_row_draw: 2, flare_counter_draw: 2, ..input(0.0, 0.0) };
        let mut rows = Vec::new();
        for t in 0..16 {
            // After the latch tick the draws change — the latch must
            // NOT re-fire (localAI[1] stays 1).
            let tick_i = if t == 0 { i } else { input(0.0, 0.0) };
            find_frame_extra7d(k, &mut s, &tick_i);
            rows.push(s.row);
        }
        assert_eq!(rows, vec![2, 3, 3, 3, 3, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2]);
        assert_eq!(s.local_ai[1], 1.0);
        // Draws 0/0: the plain 4-tick 4-row cycle from row 0.
        assert_eq!(
            rows_over(k, input(0.0, 0.0), 17),
            vec![0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 0, 0]
        );
        // The STRICT > 3 wrap: row 3 + counter 3 → counter 4 → row 4
        // → row > 3 → 0 (the latch already fired, draws ignored).
        let mut s = FrameState { row: 3, counter: 3.0, local_ai: [0.0, 1.0], ..Default::default() };
        find_frame_extra7d(k, &mut s, &i);
        assert_eq!((s.row, s.counter), (0, 0.0));
    }

    /// The squidhead sets direction off the vx sign (vx == 0 → +1),
    /// copies it to the sprite and aims the rotation
    /// `atan2(vy*sprite, vx*sprite) + vx*0.1` — vx -3 / vy 4 mirrors
    /// to atan2(-4, 3) - 0.3 ≈ -1.2272952.
    #[test]
    fn f26s7d_the_squidhead_direction_and_aim() {
        let k = FindFrameExtra7dKind::AncientCultistSquidhead;
        let mut s = FrameState { sprite_dir: 1, direction: 1, ..Default::default() };
        find_frame_extra7d(k, &mut s, &input(-3.0, 4.0));
        assert_eq!((s.direction, s.sprite_dir), (-1, -1));
        assert!(
            (s.rotation - -1.2272952).abs() < 1e-5,
            "rotation {}",
            s.rotation
        );
        // vx == 0 faces +1 even from direction -1; atan2(0, 0) = 0.
        let mut s = FrameState { sprite_dir: -1, direction: -1, rotation: 9.0, ..Default::default() };
        find_frame_extra7d(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.direction, s.sprite_dir, s.rotation), (1, 1, 0.0));
    }

    /// The squidhead's >= 5 step walks the FULL 4-row sheet with no
    /// velocity gate (replay: vx 3 → 21 ticks
    /// [0,0,0,0,1,1,1,1,1,2,2,2,2,2,3,3,3,3,3,0,0]).
    #[test]
    fn f26s7d_the_squidhead_full_sheet_spin() {
        let k = FindFrameExtra7dKind::AncientCultistSquidhead;
        assert_eq!(
            rows_over(k, input(3.0, 0.0), 21),
            vec![0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 0, 0]
        );
    }

    /// The doom's bare >= 6 stepper walks the FULL 5-row sheet
    /// (replay: 31 ticks — five row-0 ticks, then six each of rows
    /// 1..4, then the wrap back to 0).
    #[test]
    fn f26s7d_the_doom_bare_stepper() {
        let k = FindFrameExtra7dKind::AncientDoom;
        assert_eq!(
            rows_over(k, input(0.0, 0.0), 31),
            vec![
                0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3,
                3, 3, 4, 4, 4, 4, 4, 4, 0, 0
            ]
        );
        // NOTHING else is touched: sprite/rotation/direction hold.
        let mut s = FrameState { sprite_dir: -1, direction: -1, rotation: 0.7, ..Default::default() };
        find_frame_extra7d(k, &mut s, &input(3.0, 2.0));
        assert_eq!((s.sprite_dir, s.direction, s.rotation), (-1, -1, 0.7));
    }

    /// The ghoul walk lifts to row 2 and accumulates |vx| THEN +1
    /// with the STRICT > 9 step over rows 2..7 (replay: vx 3 → +4 a
    /// tick, [2,2,3,3,3,4,4,4,5,5,5,6,6,6,7,7,7,2,2]; vx 1 → +2 a
    /// tick, the step lands every 5th tick). BOTH ghouls ride the ONE
    /// group machine.
    #[test]
    fn f26s7d_the_ghoul_walk_bands_and_wrap() {
        for k in [FindFrameExtra7dKind::DesertGhoul, FindFrameExtra7dKind::DesertGhoulCorruption] {
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
            find_frame_extra7d(k, &mut s, &input(-3.0, 0.0));
            assert_eq!((s.row, s.counter, s.sprite_dir), (2, 4.0, -1));
        }
    }

    /// The ghoul idle zeroes BOTH row and counter (the sprite copy
    /// still rides direction; 0 leaves the sprite alone); the air
    /// pins row 1 + zeroes the counter with the sprite UNTOUCHED.
    #[test]
    fn f26s7d_the_ghoul_idle_and_air_pins() {
        let k = FindFrameExtra7dKind::DesertGhoul;
        // The idle: direction 0 keeps sprite -1, row/counter zeroed.
        let mut s = FrameState { row: 5, counter: 7.0, sprite_dir: -1, direction: 0, ..Default::default() };
        find_frame_extra7d(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter, s.sprite_dir), (0, 0.0, -1));
        // The air: row 1, counter 0, sprite 1 HELD (direction -1 not
        // copied — the air branch never touches the sprite).
        let mut s = FrameState { row: 5, counter: 7.0, sprite_dir: 1, direction: -1, ..Default::default() };
        find_frame_extra7d(k, &mut s, &input(3.0, 2.0));
        assert_eq!((s.row, s.counter, s.sprite_dir), (1, 0.0, 1));
    }

    /// The seam maps exactly this wave's 5 ids — the no-case
    /// 515/517/522, the other-lane 518/520, the later-slice group
    /// members 526/527 and the prior-wave 494 stay `None`.
    #[test]
    fn f26s7d_the_seam_maps_exactly_the_slice() {
        for (id, k) in [
            (516, FindFrameExtra7dKind::SolarFlare),
            (521, FindFrameExtra7dKind::AncientCultistSquidhead),
            (523, FindFrameExtra7dKind::AncientDoom),
            (524, FindFrameExtra7dKind::DesertGhoul),
            (525, FindFrameExtra7dKind::DesertGhoulCorruption),
        ] {
            assert_eq!(find_frame_kind_extra7d(id), Some(k));
            assert_eq!(k.vanilla_type(), id);
        }
        for id in [494, 515, 517, 518, 520, 522, 526, 527] {
            assert_eq!(find_frame_kind_extra7d(id), None, "type {id}");
        }
    }

    /// The 5-kind strip sweep: every kind ticks 108 input combos
    /// (vx 0/1/3 × vy 0/-2/2 × direction 1/-1/0 × flare draws 0/2 ×
    /// 0/2) for 10 ticks each from a fresh state — every row stays
    /// inside its own sheet (the 7c 72-combo sweep precedent, the ai
    /// axes swapped for the direction + resolved-draw axes; the full
    /// 540-combo × 10-tick sweep replayed against the line-for-line
    /// C# transcriptions before writing).
    #[test]
    fn f26s7d_the_5_kind_strip_sweep_stays_in_sheet() {
        let kinds = [
            FindFrameExtra7dKind::SolarFlare,
            FindFrameExtra7dKind::AncientCultistSquidhead,
            FindFrameExtra7dKind::AncientDoom,
            FindFrameExtra7dKind::DesertGhoul,
            FindFrameExtra7dKind::DesertGhoulCorruption,
        ];
        for k in kinds {
            let mut combo = 0;
            for vx in [0.0f32, 1.0, 3.0] {
                for vy in [0.0f32, -2.0, 2.0] {
                    for direction in [1i32, -1, 0] {
                        for row_draw in [0u32, 2] {
                            for counter_draw in [0u32, 2] {
                                let mut s = FrameState { direction, ..Default::default() };
                                let i = FrameInput {
                                    vx,
                                    vy,
                                    flare_row_draw: row_draw,
                                    flare_counter_draw: counter_draw,
                                };
                                for _ in 0..10 {
                                    find_frame_extra7d(k, &mut s, &i);
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
    }
}
