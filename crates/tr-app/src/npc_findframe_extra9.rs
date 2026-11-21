//! W4-f27 — FindFrame extra wave 9 (slice 9): the FIVE fix kinds of
//! the 463-493 re-census (the claim is the ANALYSIS "W4-f27 FindFrame
//! extra wave 9" section — the census re-derived mechanically incl
//! the pushed d40 roster tip 8fe1d6d3, STILL the slice-8 tip, and the
//! pushed f26 co-lane tip a2e47596a whose extra-7c slice owns 494+).
//! The 463-493 window has NO unclaimed case id left — every id is
//! port-covered faithful, d40-covered, or f26-claimed — so this wave
//! is the d40-arm-quality FIX set: **463/467/471/472/492** —
//! Nailhead, DeadlySphere, GoblinSummoner, ShadowFlameApparition and
//! the Pirate Ship Cannon (`ID/NPCID.cs:11599-11657`). Landed 5 kinds
//! per commit (ONE batch — the claim is exactly 5).
//!
//! The per-kind verdicts (the full table is in ANALYSIS):
//! * **463 Nailhead** — the d40 arm rides `humanoid_town_frame`
//!   (its npc.rs:9685): WRONG, the same defect as the slice-8
//!   460/462 stand-ins — no grounded flip, no air tail, and its
//!   `humanoid_frame_count` falls to `_ => 1` (d40 data.rs:3578, no
//!   Nailhead row) so the walk wraps at 1 and never leaves row 2.
//!   The C# is the humanoid-group FIGHTER walk (`NPC.cs:68613` in
//!   the case list, aiStyle 3 `:15538`, no townNPC → num57 = 0).
//! * **467 DeadlySphere** — the d40 arm is a ROW-0 STUB (its
//!   npc.rs:9760-9763). The C# (`NPC.cs:75069-75102`) is a two-phase
//!   strip: ai[0] == 1 winds UP rows 0..6 every > 4 ticks and HOLDS
//!   at 6, else winds DOWN (the > 3 clamp first) to 0.
//! * **471 GoblinSummoner** — OUR OWN kind; the v1 `npc.rs:7768`
//!   free `(vx != 0) + 0.6 >= 8` strip over 20 rows is WRONG vs the
//!   C#'s ai[3] machine (`NPC.cs:75532-75617`). The npc.rs arm is
//!   FIXED this slice (the 439-fix precedent).
//! * **472 ShadowFlameApparition** — the d40 arm is a ROW-0 STUB
//!   (its npc.rs:9760-9763). The C# (`NPC.cs:71369-71397`) writes
//!   direction from the vx sign, tilts at the direction-folded
//!   atan2, and steps the 6-row strip every > 4 ticks.
//! * **492 PirateShipCannon** — OUR OWN kind; the v1 `npc.rs:7744`
//!   free strip over 9 rows is WRONG vs the C#'s ONE-LINE
//!   `frame.Y = num * (int)ai[2]` (`NPC.cs:71156-71158` — the aim
//!   octant the style75 machine owns, `NPC.cs:35789-35820`, already
//!   ported verbatim in `npc_ai_style75.rs:594-668`). The npc.rs arm
//!   is FIXED this slice.
//!
//! The EXCLUDED faithful arms (the verdict table's cites): 464
//! CrimsonBunny (`NPC.cs:76851-76904`, the e8 443 bunny twin), 465
//! CrimsonGoldfish (`:74181-74226`, the d44 fish twin), 466 Psycho
//! (`:75466-75531`), 468 DrManFly (`:71516-71577`, the f26 481
//! ladder twin), 469 ThePossessed (`:75665-75732`), 470
//! CrimsonPenguin (`:76395-76456`, the d47 168 twin) — all d40
//! inline arms line-for-line faithful; 489 BloodZombie / 490
//! Drippler — OUR faithful S1-D-3z arms (`npc.rs:6418`/`:6460`);
//! 491 PirateShip / 493 LunarTowerStardust — NO FindFrame case
//! label, npcFrameCount 1 (`Main.cs:67133`), the row-0 pins
//! correct-by-construction (the 478 MothronEgg / tower precedent).
//!
//! Shape: 463/467/472 are d40 roster kinds (NOT in this branch's
//! `NpcKind`) — their machines land as pure functions over a small
//! [`FrameState`]/[`FrameInput`] pair (the extra-8 precedent); the
//! wiring seam is [`find_frame_kind_extra9`] and the roster merge
//! copies row/counter/sprite_dir/rotation/direction out and back.
//! 471/492 ARE our kinds — their `npc.rs` find_frame arms delegate
//! here THIS slice (the 439-fix bridge shape). No arm draws
//! `Main.rand` — verified mechanically per cited range (the
//! humanoid group's town pose zones with their chat-bubble draws
//! are the documented skip; the 463 fighter never enters them).
//!
//! Row convention: the port tracks frame ROW indices, not pixels —
//! the C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison. The per-kind row
//! counts are `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry
//! initializer), counted entry-by-entry in
//! [`FindFrameExtra9Kind::frame_count`] and cross-checked against
//! the port's own `render/entities.rs::NPC_FRAME_COUNT` table.
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "W4-f27 FindFrame
//! extra wave 9"):
//! * the sphere's `IsABestiaryIconDummy` disjunct + reset
//!   (`NPC.cs:75070`/`:75081-75084`) — a display-only dummy path,
//!   the port has no bestiary-dummy NPC entity (the extra-7/8
//!   precedent);
//! * the humanoid group's town `ai[0]` sit/talk/attack pose zones
//!   (`NPC.cs:68662-69488`) — 463 is an aiStyle-3 fighter
//!   (`NPC.cs:15538`, no townNPC), the wave-B
//!   `humanoid_fighter_frame` keep-one precedent.

// The 463/467/472 wiring is a later round (the roster wave owns the
// kinds) — the `npc_findframe_extra8.rs` precedent.
#![allow(dead_code)]

/// The extra-wave-9 kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra9Kind {
    /// `NPCID.Nailhead` = 463 (`NPCID.cs:11599`) — the humanoid-group
    /// fighter (`NPC.cs:68613`, aiStyle 3 `:15538`).
    Nailhead,
    /// `NPCID.DeadlySphere` = 467 (`NPCID.cs:11607`) —
    /// `case 467` (`NPC.cs:75069-75102`), the wind-up/wind-down strip.
    DeadlySphere,
    /// `NPCID.GoblinSummoner` = 471 (`NPCID.cs:11615`) —
    /// `case 471` (`NPC.cs:75532-75617`), the ai[3] phase machine.
    GoblinSummoner,
    /// `NPCID.ShadowFlameApparition` = 472 (`NPCID.cs:11617`) —
    /// `case 472` (`NPC.cs:71369-71397`), the direction-folded tilt.
    ShadowFlameApparition,
    /// `NPCID.PirateShipCannon` = 492 (`NPCID.cs:11657`, the port's
    /// `FlyingDutchmanCannon`) — `case 492` (`NPC.cs:71156-71158`),
    /// the `(int)ai[2]` aim-octant row.
    PirateShipCannon,
}

impl FindFrameExtra9Kind {
    /// The vanilla `NPC.type` — the `FindFrame` switch key.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::Nailhead => 463,
            Self::DeadlySphere => 467,
            Self::GoblinSummoner => 471,
            Self::ShadowFlameApparition => 472,
            Self::PirateShipCannon => 492,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// entry `type` on line `67084 + type/10`, slot `type % 10`) —
    /// cross-checked against `render/entities.rs::NPC_FRAME_COUNT`.
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67130 (463/467): 16, 13.
            Self::Nailhead => 16,
            Self::DeadlySphere => 13,
            // Main.cs:67131 (471/472): 20, 6.
            Self::GoblinSummoner => 20,
            Self::ShadowFlameApparition => 6,
            // Main.cs:67133 (492): 9.
            Self::PirateShipCannon => 9,
        }
    }
}

/// The wiring seam: the vanilla type → this module's kind (the
/// `find_frame_kind_extra8` precedent). `None` outside the slice.
pub fn find_frame_kind_extra9(vanilla_type: i16) -> Option<FindFrameExtra9Kind> {
    Some(match vanilla_type {
        463 => FindFrameExtra9Kind::Nailhead,
        467 => FindFrameExtra9Kind::DeadlySphere,
        471 => FindFrameExtra9Kind::GoblinSummoner,
        472 => FindFrameExtra9Kind::ShadowFlameApparition,
        492 => FindFrameExtra9Kind::PirateShipCannon,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection`,
/// `rotation`, `direction`. The wiring step copies them to/from
/// `Npc` (the 471/492 bridges copy only what their arms write).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter`.
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — only the apparition arm (472) writes it; the
    /// Nailhead/sphere/summoner/cannon arms never touch it (the
    /// summoner's AI owns its rotation, `NPC.cs:56520`'s ×0.9 decay).
    pub rotation: f32,
    /// `Entity.direction` — only the apparition arm (472) WRITES it
    /// (`NPC.cs:71370-71377`: the vx sign, 0 → +1); the other arms
    /// leave it alone. The wiring copies it back for 472.
    pub direction: i32,
}

/// The per-tick inputs the arms read: `velocity`, `direction` and
/// the `ai` slots. `wet`/`ai_action` are not carried — none of these
/// arms reads them (the sphere reads ai[0], the summoner ai[3], the
/// cannon ai[2]; the Nailhead fighter walk reads neither).
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone in
    /// the arms that gate on `direction == 1` / `== -1`; 472 WRITES
    /// direction itself from the vx sign — the input is the pre-tick
    /// value).
    pub direction: i32,
    pub ai: [f32; 4],
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1, ai: [0.0; 4] }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-9 kinds — one
/// tick.
pub fn find_frame_extra9(kind: FindFrameExtra9Kind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // `case 463` in the humanoid-group case list
        // (`NPC.cs:68613`) → the shared fighter walk
        // (`NPC.cs:68647-69655`) via the wave-B keep-one helper (the
        // extra-8 Butcher delegate precedent). Nailhead is NOT
        // `isLikeATownNPC` (his SetDefaults row `:15534-15546` sets
        // aiStyle 3, no townNPC), so `num57` = 0 and the walk wraps
        // at the FULL 16 rows; not a town slime (`num108` = 6,
        // `num109` = row 2), not the 140/489 idle-row-1 pair, NOT in
        // the `:69647` air pin list (the air holds row 1). The town
        // pose zones are the documented skip. The d40
        // `humanoid_town_frame` stand-in is WRONG for him (no flip,
        // no air tail, the count-helper falls to 1) — the merger
        // replaces it with this machine.
        FindFrameExtra9Kind::Nailhead => {
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
        // `case 467` (`NPC.cs:75069-75102`): the Deadly Sphere's
        // two-phase strip. WIND-UP (`ai[0] == 1`, `:75070-75087`):
        // the counter++ every tick, the > 4 step advances the row
        // ONLY while row < 6 — at row 6+ the step zeroes the counter
        // but writes NO row (the hold). WIND-DOWN (else,
        // `:75088-75101`): the counter++ every tick, the > 4 step
        // FIRST clamps any row > 3 down to 3 (`:75092-75095`) THEN
        // decrements while row > 0 (`:75096-75100`) — a sphere at
        // row 9 lands 2 on its first step, not 8. The
        // `IsABestiaryIconDummy` disjunct (`:75070`) + its row-0
        // reset (`:75081-75084`) are the display-only skip. No
        // sprite/rotation/direction write. The d40 row-0 stub is
        // superseded.
        FindFrameExtra9Kind::DeadlySphere => {
            if i.ai[0] == 1.0 {
                // `:75072-75085`.
                s.counter += 1.0;
                if s.counter > 4.0 {
                    s.counter = 0.0;
                    if s.row < 6 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                }
                return;
            }
            // `:75088-75100`.
            s.counter += 1.0;
            if s.counter > 4.0 {
                s.counter = 0.0;
                if s.row > 3 {
                    s.row = 3;
                }
                if s.row > 0 {
                    s.row -= 1;
                    s.counter = 0.0;
                }
            }
        }
        // `case 471` (`NPC.cs:75532-75617`): the Goblin Summoner's
        // ai[3] phase machine. SHADOWFLAME (`ai[3] < 0`,
        // `:75534-75573`): the ±1 flip, the `|velocity| * 0.5 + 1`
        // counter with the < 6 / < 12 / < 18 / < 23 bands picking
        // rows 17/18/19/18 — the >= 23 reset tick zeroes the counter
        // and HOLDS row 18 (the fourth band and the reset write the
        // same row). SUMMONING (`ai[3] == 1`, `:75574-75578`): the
        // row-16 pin + counter zero. GROUNDED (`:75579-75610`): the
        // ±1 flip, the idle row-0 snap + counter zero, the walk
        // `|vx| * 2 + 1` with the > 7 step (NOT the fighter's > 6)
        // and the > 15 wrap-to-2 + the < 2 clamp (rows 16..19 are
        // the summon/shadow bands on the 20-row sheet — NO lift-to-2
        // before the accumulate, the clamps run AFTER the step).
        // AIR (`:75611-75615`): the counter zero + row 1. No
        // rotation write (the AI owns it). The v1 free-strip arm is
        // WRONG — the npc.rs bridge delegates here.
        FindFrameExtra9Kind::GoblinSummoner => {
            if i.ai[3] < 0.0 {
                // `:75541-75548`.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                // `:75549-75550` — Math.Abs(velocity.Length()).
                s.counter += i.vx.hypot(i.vy) * 0.5;
                s.counter += 1.0;
                // `:75551-75572`.
                if s.counter < 6.0 {
                    s.row = 17;
                    return;
                }
                if s.counter < 12.0 {
                    s.row = 18;
                    return;
                }
                if s.counter < 18.0 {
                    s.row = 19;
                    return;
                }
                if s.counter < 23.0 {
                    s.row = 18;
                    return;
                }
                s.row = 18;
                s.counter = 0.0;
            } else if i.ai[3] == 1.0 {
                // `:75576-75577`.
                s.row = 16;
                s.counter = 0.0;
            } else if i.vy == 0.0 {
                // `:75581-75588`.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                if i.vx == 0.0 {
                    // `:75589-75594`.
                    s.row = 0;
                    s.counter = 0.0;
                    return;
                }
                // `:75595-75609`.
                s.counter += i.vx.abs() * 2.0;
                s.counter += 1.0;
                if s.counter > 7.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
                if s.row > 15 {
                    s.row = 2;
                }
                if s.row < 2 {
                    s.row = 2;
                }
            } else {
                // `:75613-75614`.
                s.counter = 0.0;
                s.row = 1;
            }
        }
        // `case 472` (`NPC.cs:71369-71397`): the Shadowflame
        // Apparition. The vx sign WRITES direction (`:71370-71377` —
        // the else covers vx == 0 → +1), the ±1 sprite copy
        // (`:71378-71385`), the tilt is
        // `atan2(vy * direction, vx * direction)` (`:71386` — the
        // direction-folded aim, NO wrap/ease unlike the 461 wet
        // swimmer), the counter++ every tick with the > 4 step over
        // the 6-row sheet and the >= 6 wrap to 0 (`:71387-71396`).
        // The d40 row-0 stub is superseded.
        FindFrameExtra9Kind::ShadowFlameApparition => {
            // `:71370-71377`.
            if i.vx < 0.0 {
                s.direction = -1;
            } else {
                s.direction = 1;
            }
            // `:71378-71385`.
            if s.direction == 1 {
                s.sprite_dir = 1;
            }
            if s.direction == -1 {
                s.sprite_dir = -1;
            }
            // `:71386`.
            s.rotation =
                (i.vy * s.direction as f32).atan2(i.vx * s.direction as f32);
            // `:71387-71396`.
            s.counter += 1.0;
            if s.counter > 4.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= 6 {
                s.row = 0;
            }
        }
        // `case 492` (`NPC.cs:71156-71158`): the Pirate Ship
        // Cannon's row is `(int)ai[2]` — the aim octant the aiStyle-75
        // machine owns (`NPC.cs:35789-35820`: the eight-ray closest
        // probe + 1, the spriteDirection == 1 → 9 − x mirror, the
        // no-sight clear to 0; the port's `npc_ai_style75.rs:594-668`
        // ports that block verbatim). The counter/sprite/rotation
        // are NEVER touched here. The v1 free-strip arm is WRONG —
        // the npc.rs bridge delegates here.
        FindFrameExtra9Kind::PirateShipCannon => {
            // `:71157` — the (int) truncation toward zero.
            s.row = i.ai[2] as i32 as u32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn rows_over(kind: FindFrameExtra9Kind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra9(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The frame-count table agrees with the port's verbatim
    /// `NPC_FRAME_COUNT` (`Main.cs:67082` initializer cross-check).
    #[test]
    fn f27s9_frame_counts_match_the_port_table() {
        let table = crate::render::entities::NPC_FRAME_COUNT;
        for k in [
            FindFrameExtra9Kind::Nailhead,
            FindFrameExtra9Kind::DeadlySphere,
            FindFrameExtra9Kind::GoblinSummoner,
            FindFrameExtra9Kind::ShadowFlameApparition,
            FindFrameExtra9Kind::PirateShipCannon,
        ] {
            assert_eq!(
                k.frame_count(),
                table[k.vanilla_type() as usize],
                "type {}",
                k.vanilla_type()
            );
        }
    }

    /// 463 Nailhead: the fighter walk (the wave-B delegate) — the
    /// |vx|*2+1 > 6 stride over the FULL 16 rows, the air row 1, the
    /// idle row 0, the ±1 flip. Expectations re-derived by mechanical
    /// replay of the C# transcription.
    #[test]
    fn f27s9_nailhead_rides_the_fighter_walk() {
        // vx 1: +3 a tick — the > 6 step every 3rd tick.
        let rows = rows_over(FindFrameExtra9Kind::Nailhead, input(1.0, 0.0), 12);
        assert_eq!(rows, vec![2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6]);
        // vx 3: +7 a tick — the walk reaches row 15 then wraps to 2
        // (the FULL-16 wrap, NOT the d40 stand-in's count-1 wrap).
        let rows = rows_over(FindFrameExtra9Kind::Nailhead, input(3.0, 0.0), 16);
        assert_eq!(
            rows,
            vec![3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 2, 3, 4]
        );
        // The air tail: row 1 both rise and fall (the d40 stand-in
        // writes NOTHING in the air), the counter zeroed.
        let mut s = FrameState { row: 5, counter: 4.0, ..Default::default() };
        find_frame_extra9(FindFrameExtra9Kind::Nailhead, &mut s, &input(1.0, -2.0));
        assert_eq!((s.row, s.counter), (1, 0.0));
        find_frame_extra9(FindFrameExtra9Kind::Nailhead, &mut s, &input(1.0, 2.0));
        assert_eq!((s.row, s.counter), (1, 0.0));
        // The idle snap.
        find_frame_extra9(FindFrameExtra9Kind::Nailhead, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        // The grounded ±1 flip (0 leaves it).
        let mut s = FrameState::default();
        let mut i = input(1.0, 0.0);
        i.direction = -1;
        find_frame_extra9(FindFrameExtra9Kind::Nailhead, &mut s, &i);
        assert_eq!(s.sprite_dir, -1);
        i.direction = 0;
        find_frame_extra9(FindFrameExtra9Kind::Nailhead, &mut s, &i);
        assert_eq!(s.sprite_dir, -1, "direction 0 leaves the sprite");
    }

    /// 467 DeadlySphere: the wind-up holds at row 6, the wind-down
    /// clamps > 3 to 3 FIRST then decrements (row 9 → 2 on the first
    /// step). Expectations re-derived by mechanical replay.
    #[test]
    fn f27s9_sphere_winds_up_to_six_and_down_to_zero() {
        // ai[0] == 1 from row 0: the > 4 step every 5th tick, rows
        // 0..6 then the HOLD (the counter keeps cycling at 6).
        let mut i = input(0.0, 0.0);
        i.ai[0] = 1.0;
        let rows = rows_over(FindFrameExtra9Kind::DeadlySphere, i, 36);
        assert_eq!(
            rows,
            vec![
                0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4,
                4, 4, 4, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6, 6,
            ]
        );
        // The wind-down from row 6: four hold ticks, then the > 3
        // clamp snaps 3 and the decrement lands 2, then 1, then 0.
        let mut s = FrameState { row: 6, ..Default::default() };
        let mut out = Vec::new();
        for _ in 0..26 {
            find_frame_extra9(FindFrameExtra9Kind::DeadlySphere, &mut s, &input(0.0, 0.0));
            out.push(s.row);
        }
        assert_eq!(
            out,
            vec![
                6, 6, 6, 6, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0,
            ]
        );
        // The wind-up at row 9 (>= 6): the step zeroes the counter
        // but writes NO row.
        let mut s = FrameState { row: 9, counter: 4.0, ..Default::default() };
        let mut i = input(0.0, 0.0);
        i.ai[0] = 1.0;
        find_frame_extra9(FindFrameExtra9Kind::DeadlySphere, &mut s, &i);
        assert_eq!((s.row, s.counter), (9, 0.0));
        // The wind-down at row 9: the clamp-3-then-decrement lands 2.
        let mut s = FrameState { row: 9, counter: 4.0, ..Default::default() };
        find_frame_extra9(FindFrameExtra9Kind::DeadlySphere, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (2, 0.0));
    }

    /// 471 GoblinSummoner: the ai[3] machine — the shadowflame
    /// 17/18/19/18 bands with the >= 23 reset-tick row-18 hold, the
    /// summon row-16 pin, the > 7 walk stride over rows 2..15, the
    /// air row 1, the idle row 0. Expectations re-derived by
    /// mechanical replay.
    #[test]
    fn f27s9_summoner_rides_the_ai3_phase_machine() {
        // The grounded walk: vx 1 → +3 a tick, the > 7 step every
        // 3rd tick (NOT the fighter's > 6).
        let rows = rows_over(FindFrameExtra9Kind::GoblinSummoner, input(1.0, 0.0), 12);
        assert_eq!(rows, vec![2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6]);
        // vx 3 → +7 a tick, the > 7 step every 2nd tick.
        let rows = rows_over(FindFrameExtra9Kind::GoblinSummoner, input(3.0, 0.0), 16);
        assert_eq!(
            rows,
            vec![2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10]
        );
        // The shadowflame phase: |v| = 5 → +3.5 a tick, the bands
        // 17/18/18/19/19/18/18(reset)/17... — the reset tick HOLDS 18.
        let mut i = input(3.0, 4.0);
        i.ai[3] = -1.0;
        let rows = rows_over(FindFrameExtra9Kind::GoblinSummoner, i, 24);
        assert_eq!(
            rows,
            vec![
                17, 18, 18, 19, 19, 18, 18, 17, 18, 18, 19, 19, 18, 18, 17, 18,
                18, 19, 19, 18, 18, 17, 18, 18,
            ]
        );
        // The shadowflame counter/sprite detail: the ±1 flip rides
        // the phase, the >= 23 reset zeroes the counter.
        let mut s = FrameState::default();
        let mut i = input(3.0, 4.0);
        i.ai[3] = -2.0;
        i.direction = -1;
        let mut steps = Vec::new();
        for _ in 0..8 {
            find_frame_extra9(FindFrameExtra9Kind::GoblinSummoner, &mut s, &i);
            steps.push((s.row, s.counter, s.sprite_dir));
        }
        assert_eq!(
            steps,
            vec![
                (17, 3.5, -1),
                (18, 7.0, -1),
                (18, 10.5, -1),
                (19, 14.0, -1),
                (19, 17.5, -1),
                (18, 21.0, -1),
                (18, 0.0, -1),
                (17, 3.5, -1),
            ]
        );
        // The summoning pin: row 16 + counter zero.
        let mut s = FrameState { row: 5, counter: 9.0, ..Default::default() };
        let mut i = input(1.0, 0.0);
        i.ai[3] = 1.0;
        find_frame_extra9(FindFrameExtra9Kind::GoblinSummoner, &mut s, &i);
        assert_eq!((s.row, s.counter), (16, 0.0));
        // The air pin: row 1 + counter zero.
        let mut s = FrameState { row: 5, counter: 9.0, ..Default::default() };
        find_frame_extra9(FindFrameExtra9Kind::GoblinSummoner, &mut s, &input(1.0, -2.0));
        assert_eq!((s.row, s.counter), (1, 0.0));
        // The idle snap.
        find_frame_extra9(FindFrameExtra9Kind::GoblinSummoner, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        // The walk from row 19 (a shadow-band row): NO lift-to-2
        // before the accumulate — the > 15 clamp AFTER the step
        // lands 2 (the counter keeps its +3).
        let mut s = FrameState { row: 19, ..Default::default() };
        find_frame_extra9(FindFrameExtra9Kind::GoblinSummoner, &mut s, &input(1.0, 0.0));
        assert_eq!((s.row, s.counter), (2, 3.0));
    }

    /// 472 ShadowFlameApparition: the direction write (vx sign,
    /// 0 → +1), the direction-folded atan2 tilt, the > 4 step over
    /// the 6-row sheet with the >= 6 wrap. Expectations re-derived
    /// by mechanical replay.
    #[test]
    fn f27s9_apparition_tilts_and_steps_the_six_row_strip() {
        let rows = rows_over(FindFrameExtra9Kind::ShadowFlameApparition, input(2.0, 0.0), 12);
        assert_eq!(rows, vec![0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2]);
        // vx < 0: direction WRITES -1, the sprite follows, the tilt
        // is atan2(vy*-1, vx*-1) = atan2(-1, 2).
        let mut s = FrameState::default();
        find_frame_extra9(FindFrameExtra9Kind::ShadowFlameApparition, &mut s, &input(-2.0, 1.0));
        assert_eq!((s.direction, s.sprite_dir), (-1, -1));
        assert_eq!(s.rotation, (-1.0f32).atan2(2.0));
        // vx == 0: the else writes direction +1 (NOT a hold).
        let mut s = FrameState { direction: -1, sprite_dir: -1, ..Default::default() };
        find_frame_extra9(FindFrameExtra9Kind::ShadowFlameApparition, &mut s, &input(0.0, 0.0));
        assert_eq!((s.direction, s.sprite_dir), (1, 1));
        assert_eq!(s.rotation, 0.0);
        // The wrap tick: row 5 + counter 4 → the step lands 6, the
        // >= 6 wrap snaps 0, the counter zeroed.
        let mut s = FrameState { row: 5, counter: 4.0, ..Default::default() };
        find_frame_extra9(FindFrameExtra9Kind::ShadowFlameApparition, &mut s, &input(1.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
    }

    /// 492 PirateShipCannon: the row is `(int)ai[2]` — the counter/
    /// sprite/rotation are NEVER touched. Expectations re-derived by
    /// mechanical replay.
    #[test]
    fn f27s9_cannon_row_rides_the_ai2_aim_octant() {
        for &(ai2, row) in &[(0.0, 0u32), (1.0, 1), (4.0, 4), (8.0, 8), (8.9, 8), (2.7, 2)] {
            let mut s = FrameState {
                row: 3,
                counter: 5.0,
                sprite_dir: -1,
                rotation: 0.5,
                direction: -1,
            };
            let mut i = input(0.0, 0.0);
            i.ai[2] = ai2;
            find_frame_extra9(FindFrameExtra9Kind::PirateShipCannon, &mut s, &i);
            assert_eq!(s.row, row, "ai[2] = {ai2}");
            assert_eq!(s.counter, 5.0, "the counter is never touched");
            assert_eq!(s.sprite_dir, -1, "the sprite is never touched");
            assert_eq!(s.rotation, 0.5, "the rotation is never touched");
            assert_eq!(s.direction, -1, "the direction is never touched");
        }
    }

    /// The 5-kind strip sweep: a 5x3x3 vx/vy/direction input grid
    /// (plus the ai[0]/ai[2]/ai[3] phase variants), 40 ticks each —
    /// every row stays inside its sheet (the extra-8 precedent).
    #[test]
    fn f27s9_five_kind_strip_sweep_stays_inside_the_sheets() {
        let kinds = [
            FindFrameExtra9Kind::Nailhead,
            FindFrameExtra9Kind::DeadlySphere,
            FindFrameExtra9Kind::GoblinSummoner,
            FindFrameExtra9Kind::ShadowFlameApparition,
            FindFrameExtra9Kind::PirateShipCannon,
        ];
        for k in kinds {
            let count = k.frame_count();
            for vx in [-4.0f32, -1.0, 0.0, 1.0, 4.0] {
                for vy in [-3.0f32, 0.0, 3.0] {
                    for direction in [-1i32, 0, 1] {
                        for &(ai0, ai2, ai3) in &[
                            (0.0, 0.0, 0.0),
                            (1.0, 4.0, -1.0),
                            (0.0, 8.0, 1.0),
                        ] {
                            let i = FrameInput {
                                vx,
                                vy,
                                direction,
                                ai: [ai0, 0.0, ai2, ai3],
                            };
                            let mut s = FrameState::default();
                            for tick in 0..40 {
                                find_frame_extra9(k, &mut s, &i);
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
    fn f27s9_seam_covers_exactly_the_slice() {
        for t in 0..697i16 {
            let k = find_frame_kind_extra9(t);
            let claimed = [463, 467, 471, 472, 492];
            assert_eq!(k.is_some(), claimed.contains(&t), "type {t}");
            if let Some(k) = k {
                assert_eq!(k.vanilla_type(), t);
            }
        }
    }
}
